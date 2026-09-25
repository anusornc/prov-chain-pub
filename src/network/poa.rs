//! Deterministic PoA turns and crash-safe proposal coordination for reference nodes.
//!
//! The schedule is immutable network truth: it resolves the ordered validator
//! keys declared by the active [`NetworkProfile`] to active validator members
//! in the governance-authenticated manifest. Wall-clock time and node-local
//! rotation state never participate in the answer.
//!
//! A validator's coordinator completes read-only preflight, durably fences one
//! exact unsigned body, and only then invokes Ed25519 signing. The fence is
//! non-authoritative safety state: every selected proposal still commits solely
//! through the ledger's sealed proposal adapter and universal Final Admission.
//! Receivers also durably lock the first valid Scheduled Authority proposal
//! digest before deterministic admission, so rejection or restart cannot make
//! a distinct signed body eligible for the same turn.

use ed25519_dalek::{SigningKey, VerifyingKey};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
#[cfg(unix)]
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

use crate::bridge::BridgeExportTargetV1;
use crate::ledger::{
    AdmissionCandidate, AdmissionOutcome, AdmittedBlockEnvelope, Ledger, LedgerError, LedgerHash,
    LedgerTip,
};
use crate::network::canonical::hash_domain_separated_parts;
use crate::network::membership::{ActiveMembership, MemberRole, MemberStatus};
use crate::network::profile::NetworkProfile;
use crate::privacy::PrivacyControlTransition;

pub(crate) const SIGNING_FENCE_FILENAME: &str = "poa-signing-fence.journal";

/// Sealed exact-envelope ingress selected after PoA acceptance.
pub(crate) enum ExactEnvelopeIngress {
    /// One live producer broadcast.
    Replication,
    /// One bounded historical catch-up record.
    Synchronization,
}

const SIGNING_FENCE_MAGIC: &[u8] = b"PROVCHAIN_POA_SIGNING_FENCE_V1";
const SIGNING_FENCE_ANCHOR_MAGIC: &[u8] = b"PROVCHAIN_POA_SIGNING_FENCE_ANCHOR_V1";
const SIGNING_FENCE_TRANSITION_MAGIC: &[u8] = b"PROVCHAIN_POA_SIGNING_FENCE_TRANSITION_V1";
const SIGNING_FENCE_VERSION: u16 = 1;
const FENCE_RECORD_SELECTED: u8 = 1;
const FENCE_RECORD_RETIRED: u8 = 2;
const FENCE_RECORD_OBSERVED_SIGNED: u8 = 3;
const FENCE_FRAME_CHECKSUM_BYTES: usize = 32;
const FENCE_FRAME_LENGTH_BYTES: usize = 4;
const MAX_FENCE_RECORD_BYTES: usize = 32 * 1024 * 1024;
const MAX_FENCE_REQUEST_BYTES: usize = 16 * 1024 * 1024;
const MAX_FENCE_PROPOSAL_BYTES: usize = 16 * 1024 * 1024;
const REQUEST_MAGIC: &[u8] = b"PROVCHAIN_POA_REQUEST_V1";
const PRIVACY_REQUEST_MAGIC: &[u8] = b"PROVCHAIN_POA_PRIVACY_REQUEST_V1";
const BRIDGE_EXPORT_REQUEST_MAGIC: &[u8] = b"PROVCHAIN_POA_BRIDGE_EXPORT_REQUEST_V1";
const BRIDGE_IMPORT_REQUEST_MAGIC: &[u8] = b"PROVCHAIN_POA_BRIDGE_IMPORT_REQUEST_V1";
const REQUEST_DIGEST_DOMAIN: &[u8] = b"provchain/poa-request-digest/v1";
const FENCE_FRAME_DOMAIN: &[u8] = b"provchain/poa-signing-fence-frame/v1";
const FENCE_TRANSITION_DOMAIN: &[u8] = b"provchain/poa-signing-fence-transition/v1";
const FENCE_LEDGER_BINDING_DOMAIN: &[u8] = b"provchain/poa-signing-fence-ledger/v1";

/// One manifest-authorized validator selected for a ledger height.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduledAuthority {
    node_id: Uuid,
    validator_public_key: [u8; 32],
}

/// Exact journal-derived ledger position assigned to one Scheduled Authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoATurn {
    height: u64,
    previous_envelope_hash: LedgerHash,
    previous_state_commitment: LedgerHash,
}

impl PoATurn {
    pub(crate) fn from_tip(tip: LedgerTip) -> Self {
        Self {
            height: tip.index.map_or(0, |index| index + 1),
            previous_envelope_hash: tip.envelope_hash,
            previous_state_commitment: tip.state_commitment,
        }
    }

    /// Ledger height assigned by this turn.
    pub fn height(&self) -> u64 {
        self.height
    }

    /// Expected committed parent envelope identity.
    pub fn previous_envelope_hash(&self) -> LedgerHash {
        self.previous_envelope_hash
    }

    /// Expected parent public-state commitment.
    pub fn previous_state_commitment(&self) -> LedgerHash {
        self.previous_state_commitment
    }
}

/// One unsigned closed request bound to an explicit pending PoA turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoAProposalRequest {
    turn: PoATurn,
    timestamp_millis: u64,
    payload: PoARequestPayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PoARequestPayload {
    OrdinaryProvenance(Vec<u8>),
    PrivacyControl(Vec<u8>),
    BridgeExport {
        public_provenance: Vec<u8>,
        target: BridgeExportTargetV1,
    },
    BridgeImport {
        public_provenance: Vec<u8>,
        origin_evidence: Vec<u8>,
    },
}

impl PoAProposalRequest {
    /// Bind exact public provenance bytes and timestamp to a pending turn.
    pub fn ordinary(
        turn: PoATurn,
        timestamp_millis: u64,
        public_provenance: impl AsRef<[u8]>,
    ) -> Self {
        Self {
            turn,
            timestamp_millis,
            payload: PoARequestPayload::OrdinaryProvenance(public_provenance.as_ref().to_vec()),
        }
    }

    /// Bind one complete canonical privacy transition and timestamp to a pending turn.
    pub fn privacy(
        turn: PoATurn,
        timestamp_millis: u64,
        transition: &PrivacyControlTransition,
    ) -> Self {
        Self {
            turn,
            timestamp_millis,
            payload: PoARequestPayload::PrivacyControl(transition.canonical_bytes()),
        }
    }

    /// Bind exact public provenance bytes and one profile-pinned bridge target.
    pub fn bridge_export(
        turn: PoATurn,
        timestamp_millis: u64,
        public_provenance: impl AsRef<[u8]>,
        target: BridgeExportTargetV1,
    ) -> Self {
        Self {
            turn,
            timestamp_millis,
            payload: PoARequestPayload::BridgeExport {
                public_provenance: public_provenance.as_ref().to_vec(),
                target,
            },
        }
    }

    /// Bind exact target payload and proof-derived origin evidence to one pending turn.
    pub(crate) fn bridge_import(
        turn: PoATurn,
        timestamp_millis: u64,
        public_provenance: impl AsRef<[u8]>,
        origin_evidence: impl AsRef<[u8]>,
    ) -> Self {
        Self {
            turn,
            timestamp_millis,
            payload: PoARequestPayload::BridgeImport {
                public_provenance: public_provenance.as_ref().to_vec(),
                origin_evidence: origin_evidence.as_ref().to_vec(),
            },
        }
    }

    pub(crate) fn timestamp_millis(&self) -> u64 {
        self.timestamp_millis
    }

    #[cfg(test)]
    fn public_provenance(&self) -> &[u8] {
        match &self.payload {
            PoARequestPayload::OrdinaryProvenance(bytes) => bytes,
            PoARequestPayload::PrivacyControl(_) => &[],
            PoARequestPayload::BridgeExport {
                public_provenance, ..
            } => public_provenance,
            PoARequestPayload::BridgeImport {
                public_provenance, ..
            } => public_provenance,
        }
    }

    fn canonical_bytes(&self) -> Result<Vec<u8>, PoAError> {
        let (magic, payload) = match &self.payload {
            PoARequestPayload::OrdinaryProvenance(bytes) => {
                (REQUEST_MAGIC, std::borrow::Cow::Borrowed(bytes.as_slice()))
            }
            PoARequestPayload::PrivacyControl(bytes) => (
                PRIVACY_REQUEST_MAGIC,
                std::borrow::Cow::Borrowed(bytes.as_slice()),
            ),
            PoARequestPayload::BridgeExport {
                public_provenance,
                target,
            } => (
                BRIDGE_EXPORT_REQUEST_MAGIC,
                std::borrow::Cow::Owned(bridge_export_request_payload(public_provenance, target)?),
            ),
            PoARequestPayload::BridgeImport {
                public_provenance,
                origin_evidence,
            } => (
                BRIDGE_IMPORT_REQUEST_MAGIC,
                std::borrow::Cow::Owned(bridge_import_request_payload(
                    public_provenance,
                    origin_evidence,
                )?),
            ),
        };
        let payload_len = u32::try_from(payload.len()).map_err(|_| PoAError::ProposalTooLarge {
            bytes: payload.len(),
            limit: u32::MAX as usize,
        })?;
        let mut bytes = Vec::with_capacity(magic.len() + 8 + 32 + 32 + 8 + 4 + payload.len());
        bytes.extend_from_slice(magic);
        bytes.extend_from_slice(&self.turn.height.to_be_bytes());
        bytes.extend_from_slice(&self.turn.previous_envelope_hash);
        bytes.extend_from_slice(&self.turn.previous_state_commitment);
        bytes.extend_from_slice(&self.timestamp_millis.to_be_bytes());
        bytes.extend_from_slice(&payload_len.to_be_bytes());
        bytes.extend_from_slice(&payload);
        Ok(bytes)
    }

    fn payload_len(&self) -> usize {
        match &self.payload {
            PoARequestPayload::OrdinaryProvenance(bytes)
            | PoARequestPayload::PrivacyControl(bytes) => bytes.len(),
            PoARequestPayload::BridgeExport {
                public_provenance, ..
            } => public_provenance.len(),
            PoARequestPayload::BridgeImport {
                public_provenance,
                origin_evidence,
            } => public_provenance
                .len()
                .saturating_add(origin_evidence.len()),
        }
    }

    fn digest(&self) -> Result<LedgerHash, PoAError> {
        Ok(request_digest(&self.canonical_bytes()?))
    }
}

fn bridge_export_request_payload(
    public_provenance: &[u8],
    target: &BridgeExportTargetV1,
) -> Result<Vec<u8>, PoAError> {
    let target_network = target.network_id().as_bytes();
    let target_network_len =
        u16::try_from(target_network.len()).map_err(|_| PoAError::ProposalTooLarge {
            bytes: target_network.len(),
            limit: u16::MAX as usize,
        })?;
    let payload_len =
        u32::try_from(public_provenance.len()).map_err(|_| PoAError::ProposalTooLarge {
            bytes: public_provenance.len(),
            limit: u32::MAX as usize,
        })?;
    let mut bytes = Vec::with_capacity(2 + target_network.len() + 32 + 4 + public_provenance.len());
    bytes.extend_from_slice(&target_network_len.to_be_bytes());
    bytes.extend_from_slice(target_network);
    bytes.extend_from_slice(&target.ledger_instance_id().bytes());
    bytes.extend_from_slice(&payload_len.to_be_bytes());
    bytes.extend_from_slice(public_provenance);
    Ok(bytes)
}

fn bridge_import_request_payload(
    public_provenance: &[u8],
    origin_evidence: &[u8],
) -> Result<Vec<u8>, PoAError> {
    let public_len =
        u32::try_from(public_provenance.len()).map_err(|_| PoAError::ProposalTooLarge {
            bytes: public_provenance.len(),
            limit: u32::MAX as usize,
        })?;
    let origin_len =
        u32::try_from(origin_evidence.len()).map_err(|_| PoAError::ProposalTooLarge {
            bytes: origin_evidence.len(),
            limit: u32::MAX as usize,
        })?;
    let mut bytes = Vec::with_capacity(8 + public_provenance.len() + origin_evidence.len());
    bytes.extend_from_slice(&public_len.to_be_bytes());
    bytes.extend_from_slice(public_provenance);
    bytes.extend_from_slice(&origin_len.to_be_bytes());
    bytes.extend_from_slice(origin_evidence);
    Ok(bytes)
}

fn request_digest(request_bytes: &[u8]) -> LedgerHash {
    hash_domain_separated_parts(REQUEST_DIGEST_DOMAIN, &[request_bytes])
}

fn ledger_index(height: u64) -> Result<usize, PoAError> {
    usize::try_from(height).map_err(|_| {
        PoAError::CorruptSigningFence(format!("PoA height {height} cannot index the local ledger"))
    })
}

fn ensure_matching_proposal(
    height: u64,
    selected_proposal_digest: LedgerHash,
    candidate_proposal_digest: LedgerHash,
) -> Result<(), PoAError> {
    if selected_proposal_digest == candidate_proposal_digest {
        return Ok(());
    }
    Err(PoAError::Equivocation {
        height,
        selected_proposal_digest,
        conflicting_proposal_digest: candidate_proposal_digest,
    })
}

fn validate_block_interval(
    ledger: &Ledger,
    height: u64,
    timestamp_millis: u64,
    minimum_block_interval_millis: u64,
) -> Result<(), PoAError> {
    if height == 0 {
        return Ok(());
    }
    let parent = ledger
        .committed_envelopes()
        .get(ledger_index(height - 1)?)
        .ok_or(PoAError::StaleTurn)?;
    let minimum_timestamp_millis = parent
        .timestamp_millis
        .checked_add(minimum_block_interval_millis)
        .ok_or(PoAError::ProposalTimestampOverflow { height })?;
    if timestamp_millis < minimum_timestamp_millis {
        return Err(PoAError::ProposalTooEarly {
            height,
            timestamp_millis,
            minimum_timestamp_millis,
        });
    }
    Ok(())
}

/// Bind one local authority fence to one exact activated ledger instance.
pub(crate) fn signing_fence_ledger_binding(
    profile: &NetworkProfile,
    manifest_digest: LedgerHash,
    local_node_id: Uuid,
    journal_path: &Path,
) -> LedgerHash {
    let node_id = local_node_id.as_bytes();
    let journal_path = journal_path.to_string_lossy();
    hash_domain_separated_parts(
        FENCE_LEDGER_BINDING_DOMAIN,
        &[
            profile.network_id.as_bytes(),
            profile.profile_id.as_bytes(),
            &manifest_digest,
            node_id,
            journal_path.as_bytes(),
        ],
    )
}

impl ScheduledAuthority {
    /// Manifest-bound logical validator identity.
    pub fn node_id(&self) -> Uuid {
        self.node_id
    }

    /// Separate PoA proposal verification key selected for this height.
    pub fn validator_public_key(&self) -> [u8; 32] {
        self.validator_public_key
    }
}

/// Immutable height-derived schedule resolved through active membership.
pub(crate) struct AuthoritySchedule {
    ordered_authorities: Vec<ScheduledAuthority>,
    minimum_block_interval_millis: u64,
}

#[derive(Clone, PartialEq, Eq)]
struct SelectedProposal {
    turn: PoATurn,
    validator_public_key: [u8; 32],
    request_bytes: Vec<u8>,
    request_digest: LedgerHash,
    unsigned_body: Vec<u8>,
    proposal_digest: LedgerHash,
    retired_envelope_hash: Option<LedgerHash>,
    outcome: Option<AdmissionOutcome>,
}

impl SelectedProposal {
    fn has_same_fenced_selection(&self, other: &Self) -> bool {
        self.turn == other.turn
            && self.validator_public_key == other.validator_public_key
            && self.request_bytes == other.request_bytes
            && self.request_digest == other.request_digest
            && self.unsigned_body == other.unsigned_body
            && self.proposal_digest == other.proposal_digest
    }
}

struct SigningFenceJournal {
    path: PathBuf,
    transition_path: PathBuf,
    file: File,
    ledger_binding: LedgerHash,
    uncertain: bool,
    fail_next_write: bool,
    fail_next_fsync: bool,
}

struct RecoveredCoordinatorState {
    selected: BTreeMap<u64, SelectedProposal>,
    locked_proposal_digests: BTreeMap<u64, LedgerHash>,
}

enum FenceRecord {
    Selected(Box<SelectedProposal>),
    Retired {
        height: u64,
        proposal_digest: LedgerHash,
        envelope_hash: LedgerHash,
    },
    ObservedSigned {
        height: u64,
        proposal_digest: LedgerHash,
    },
}

impl SigningFenceJournal {
    fn open(path: impl AsRef<Path>, ledger_binding: LedgerHash) -> Result<Self, PoAError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let anchor_path = signing_fence_anchor_path(&path);
        let transition_path = signing_fence_transition_path(&path);
        let fence_existed = path.try_exists()?;
        let anchor_existed = anchor_path.try_exists()?;
        if !fence_existed && anchor_existed {
            return Err(PoAError::CorruptSigningFence(
                "signing-fence journal is missing while its durable anchor remains".to_string(),
            ));
        }
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&path)?;
        lock_signing_fence(&file)?;
        if file.metadata()?.len() == 0 {
            if fence_existed {
                return Err(PoAError::CorruptSigningFence(
                    "existing signing-fence journal is empty".to_string(),
                ));
            }
            file.write_all(SIGNING_FENCE_MAGIC)?;
            file.sync_all()?;
            sync_parent_directory(&path)?;
        }
        if anchor_existed {
            validate_signing_fence_anchor(&anchor_path, ledger_binding)?;
        } else {
            create_signing_fence_anchor(&anchor_path, ledger_binding)?;
        }
        let mut journal = Self {
            path,
            transition_path,
            file,
            ledger_binding,
            uncertain: false,
            fail_next_write: false,
            fail_next_fsync: false,
        };
        let recovered = journal.replay()?;
        journal.resolve_selection_transition(&recovered)?;
        Ok(journal)
    }

    fn replay(&self) -> Result<RecoveredCoordinatorState, PoAError> {
        let bytes = fs::read(&self.path)?;
        if bytes.len() < SIGNING_FENCE_MAGIC.len()
            || &bytes[..SIGNING_FENCE_MAGIC.len()] != SIGNING_FENCE_MAGIC
        {
            return Err(PoAError::CorruptSigningFence(
                "invalid or missing signing-fence header".to_string(),
            ));
        }

        let mut offset = SIGNING_FENCE_MAGIC.len();
        let mut selected = BTreeMap::<u64, SelectedProposal>::new();
        let mut locked_proposal_digests = BTreeMap::<u64, LedgerHash>::new();
        while offset < bytes.len() {
            let frame_start = offset;
            if bytes.len() - offset < FENCE_FRAME_LENGTH_BYTES {
                return Err(PoAError::CorruptSigningFence(format!(
                    "truncated signing-fence frame length at byte {frame_start}"
                )));
            }
            let frame_len = u32::from_be_bytes(
                bytes[offset..offset + FENCE_FRAME_LENGTH_BYTES]
                    .try_into()
                    .map_err(|_| {
                        PoAError::CorruptSigningFence(
                            "invalid signing-fence frame length".to_string(),
                        )
                    })?,
            ) as usize;
            offset += FENCE_FRAME_LENGTH_BYTES;
            if frame_len == 0 || frame_len > MAX_FENCE_RECORD_BYTES {
                return Err(PoAError::CorruptSigningFence(format!(
                    "invalid signing-fence frame length {frame_len} at byte {frame_start}"
                )));
            }
            let frame_total = frame_len
                .checked_add(FENCE_FRAME_CHECKSUM_BYTES)
                .ok_or_else(|| {
                    PoAError::CorruptSigningFence("signing-fence frame length overflow".to_string())
                })?;
            if bytes.len() - offset < frame_total {
                return Err(PoAError::CorruptSigningFence(format!(
                    "truncated signing-fence frame at byte {frame_start}"
                )));
            }

            let record_bytes = &bytes[offset..offset + frame_len];
            offset += frame_len;
            let checksum = &bytes[offset..offset + FENCE_FRAME_CHECKSUM_BYTES];
            offset += FENCE_FRAME_CHECKSUM_BYTES;
            let expected_checksum =
                hash_domain_separated_parts(FENCE_FRAME_DOMAIN, &[record_bytes]);
            if checksum != expected_checksum {
                return Err(PoAError::CorruptSigningFence(format!(
                    "signing-fence frame checksum mismatch at byte {frame_start}"
                )));
            }

            match decode_fence_record(record_bytes, self.ledger_binding)? {
                FenceRecord::Selected(record) => {
                    let height = record.turn.height;
                    let proposal_digest = record.proposal_digest;
                    if let Some(existing_digest) = locked_proposal_digests.get(&height) {
                        ensure_matching_proposal(height, *existing_digest, proposal_digest)?;
                    }
                    if let Some(existing) = selected.get(&height) {
                        if !existing.has_same_fenced_selection(&record) {
                            return Err(PoAError::Equivocation {
                                height,
                                selected_proposal_digest: existing.proposal_digest,
                                conflicting_proposal_digest: proposal_digest,
                            });
                        }
                    } else {
                        selected.insert(height, *record);
                    }
                    locked_proposal_digests.insert(height, proposal_digest);
                }
                FenceRecord::Retired {
                    height,
                    proposal_digest,
                    envelope_hash,
                } => {
                    let existing = selected.get_mut(&height).ok_or_else(|| {
                        PoAError::CorruptSigningFence(format!(
                            "retirement precedes selection at height {height}"
                        ))
                    })?;
                    if existing.proposal_digest != proposal_digest {
                        return Err(PoAError::CorruptSigningFence(format!(
                            "retirement proposal mismatch at height {height}"
                        )));
                    }
                    if let Some(retired) = existing.retired_envelope_hash {
                        if retired != envelope_hash {
                            return Err(PoAError::CorruptSigningFence(format!(
                                "conflicting retirement at height {height}"
                            )));
                        }
                    } else {
                        existing.retired_envelope_hash = Some(envelope_hash);
                    }
                }
                FenceRecord::ObservedSigned {
                    height,
                    proposal_digest,
                } => {
                    if let Some(existing) = selected.get(&height) {
                        ensure_matching_proposal(
                            height,
                            existing.proposal_digest,
                            proposal_digest,
                        )?;
                    }
                    if let Some(existing_digest) = locked_proposal_digests.get(&height) {
                        ensure_matching_proposal(height, *existing_digest, proposal_digest)?;
                    }
                    locked_proposal_digests.insert(height, proposal_digest);
                }
            }
        }
        Ok(RecoveredCoordinatorState {
            selected,
            locked_proposal_digests,
        })
    }

    fn append_selected(&mut self, selected: &SelectedProposal) -> Result<(), PoAError> {
        let record = encode_selected_record(self.ledger_binding, selected)?;
        self.begin_selection_transition(&record)?;
        self.append_frame(record)?;
        self.finish_selection_transition()
    }

    fn append_observed_signed(
        &mut self,
        height: u64,
        proposal_digest: LedgerHash,
    ) -> Result<(), PoAError> {
        let record = encode_observed_signed_record(self.ledger_binding, height, proposal_digest);
        self.begin_selection_transition(&record)?;
        self.append_frame(record)?;
        self.finish_selection_transition()
    }

    fn begin_selection_transition(&mut self, record: &[u8]) -> Result<(), PoAError> {
        if self.uncertain {
            return Err(PoAError::SigningFenceUncertain(
                "a prior fence persistence result is unresolved".to_string(),
            ));
        }
        if self.transition_path.try_exists()? {
            self.uncertain = true;
            return Err(PoAError::SigningFenceUncertain(
                "a prior selection transition is unresolved".to_string(),
            ));
        }
        let transition_bytes = encode_selection_transition(record)?;
        let result = (|| -> io::Result<()> {
            let mut transition = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&self.transition_path)?;
            transition.write_all(&transition_bytes)?;
            transition.sync_all()?;
            sync_parent_directory(&self.transition_path)
        })();
        if let Err(error) = result {
            self.uncertain = true;
            return Err(PoAError::SigningFenceUncertain(format!(
                "selection-transition persistence is uncertain: {error}"
            )));
        }
        Ok(())
    }

    fn finish_selection_transition(&mut self) -> Result<(), PoAError> {
        let result = (|| -> io::Result<()> {
            fs::remove_file(&self.transition_path)?;
            sync_parent_directory(&self.transition_path)
        })();
        if let Err(error) = result {
            self.uncertain = true;
            return Err(PoAError::SigningFenceUncertain(format!(
                "selection-transition retirement is uncertain: {error}"
            )));
        }
        Ok(())
    }

    fn resolve_selection_transition(
        &mut self,
        recovered: &RecoveredCoordinatorState,
    ) -> Result<(), PoAError> {
        let Some(transition) =
            read_selection_transition(&self.transition_path, self.ledger_binding)?
        else {
            return Ok(());
        };
        let transition_complete = match transition {
            FenceRecord::Selected(transition) => recovered
                .selected
                .get(&transition.turn.height)
                .is_some_and(|persisted| persisted.has_same_fenced_selection(&transition)),
            FenceRecord::ObservedSigned {
                height,
                proposal_digest,
            } => recovered.locked_proposal_digests.get(&height) == Some(&proposal_digest),
            FenceRecord::Retired { .. } => {
                return Err(PoAError::CorruptSigningFence(
                    "selection-transition marker contains a retirement".to_string(),
                ));
            }
        };
        if !transition_complete {
            self.uncertain = true;
            return Err(PoAError::SigningFenceUncertain(
                "proposal transition has no matching complete fence record".to_string(),
            ));
        }
        if let Err(error) = self.file.sync_all() {
            self.uncertain = true;
            return Err(PoAError::SigningFenceUncertain(format!(
                "recovered fence synchronization is uncertain: {error}"
            )));
        }
        self.finish_selection_transition()
    }

    fn append_retired(
        &mut self,
        height: u64,
        proposal_digest: LedgerHash,
        envelope_hash: LedgerHash,
    ) -> Result<(), PoAError> {
        self.append_frame(encode_retired_record(
            self.ledger_binding,
            height,
            proposal_digest,
            envelope_hash,
        ))
    }

    fn append_frame(&mut self, record_bytes: Vec<u8>) -> Result<(), PoAError> {
        if self.uncertain {
            return Err(PoAError::SigningFenceUncertain(
                "a prior fence persistence result is unresolved".to_string(),
            ));
        }
        if record_bytes.is_empty() || record_bytes.len() > MAX_FENCE_RECORD_BYTES {
            return Err(PoAError::CorruptSigningFence(format!(
                "invalid encoded signing-fence record length {}",
                record_bytes.len()
            )));
        }
        self.replay()?;
        if self.fail_next_write {
            self.fail_next_write = false;
            self.uncertain = true;
            return Err(PoAError::SigningFenceUncertain(
                "injected signing-fence write failure".to_string(),
            ));
        }

        let frame_len = u32::try_from(record_bytes.len()).map_err(|_| {
            PoAError::CorruptSigningFence("signing-fence record is oversized".to_string())
        })?;
        let checksum = hash_domain_separated_parts(FENCE_FRAME_DOMAIN, &[&record_bytes]);
        if let Err(error) = (|| -> io::Result<()> {
            self.file.seek(SeekFrom::End(0))?;
            self.file.write_all(&frame_len.to_be_bytes())?;
            self.file.write_all(&record_bytes)?;
            self.file.write_all(&checksum)?;
            Ok(())
        })() {
            self.uncertain = true;
            return Err(PoAError::SigningFenceUncertain(format!(
                "signing-fence write outcome is uncertain: {error}"
            )));
        }
        if self.fail_next_fsync {
            self.fail_next_fsync = false;
            self.uncertain = true;
            return Err(PoAError::SigningFenceUncertain(
                "injected signing-fence fsync failure".to_string(),
            ));
        }
        if let Err(error) = self.file.sync_all() {
            self.uncertain = true;
            return Err(PoAError::SigningFenceUncertain(format!(
                "signing-fence fsync outcome is uncertain: {error}"
            )));
        }
        Ok(())
    }

    #[cfg(test)]
    fn inject_next_write_failure(&mut self) {
        self.fail_next_write = true;
    }

    #[cfg(test)]
    fn inject_next_fsync_failure(&mut self) {
        self.fail_next_fsync = true;
    }
}

fn signing_fence_anchor_path(fence_path: &Path) -> PathBuf {
    let mut file_name = fence_path.file_name().unwrap_or_default().to_os_string();
    file_name.push(".anchor");
    fence_path.with_file_name(file_name)
}

fn signing_fence_transition_path(fence_path: &Path) -> PathBuf {
    let mut file_name = fence_path.file_name().unwrap_or_default().to_os_string();
    file_name.push(".transition");
    fence_path.with_file_name(file_name)
}

fn signing_fence_anchor_bytes(ledger_binding: LedgerHash) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(SIGNING_FENCE_ANCHOR_MAGIC.len() + ledger_binding.len());
    bytes.extend_from_slice(SIGNING_FENCE_ANCHOR_MAGIC);
    bytes.extend_from_slice(&ledger_binding);
    bytes
}

fn create_signing_fence_anchor(
    anchor_path: &Path,
    ledger_binding: LedgerHash,
) -> Result<(), PoAError> {
    let mut anchor = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(anchor_path)?;
    anchor.write_all(&signing_fence_anchor_bytes(ledger_binding))?;
    anchor.sync_all()?;
    sync_parent_directory(anchor_path)?;
    Ok(())
}

fn validate_signing_fence_anchor(
    anchor_path: &Path,
    ledger_binding: LedgerHash,
) -> Result<(), PoAError> {
    if fs::read(anchor_path)? != signing_fence_anchor_bytes(ledger_binding) {
        return Err(PoAError::CorruptSigningFence(
            "signing-fence anchor is missing, corrupt, or bound to another ledger".to_string(),
        ));
    }
    Ok(())
}

fn sync_parent_directory(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}

fn encode_selection_transition(record: &[u8]) -> Result<Vec<u8>, PoAError> {
    if record.is_empty() || record.len() > MAX_FENCE_RECORD_BYTES {
        return Err(PoAError::CorruptSigningFence(format!(
            "invalid proposal-transition record length {}",
            record.len()
        )));
    }
    let record_len = u32::try_from(record.len()).map_err(|_| {
        PoAError::CorruptSigningFence("selection transition is oversized".to_string())
    })?;
    let checksum = hash_domain_separated_parts(FENCE_TRANSITION_DOMAIN, &[record]);
    let mut bytes = Vec::with_capacity(
        SIGNING_FENCE_TRANSITION_MAGIC.len()
            + FENCE_FRAME_LENGTH_BYTES
            + record.len()
            + FENCE_FRAME_CHECKSUM_BYTES,
    );
    bytes.extend_from_slice(SIGNING_FENCE_TRANSITION_MAGIC);
    bytes.extend_from_slice(&record_len.to_be_bytes());
    bytes.extend_from_slice(record);
    bytes.extend_from_slice(&checksum);
    Ok(bytes)
}

fn read_selection_transition(
    transition_path: &Path,
    ledger_binding: LedgerHash,
) -> Result<Option<FenceRecord>, PoAError> {
    if !transition_path.try_exists()? {
        return Ok(None);
    }
    let bytes = fs::read(transition_path)?;
    let header_len = SIGNING_FENCE_TRANSITION_MAGIC.len() + FENCE_FRAME_LENGTH_BYTES;
    let minimum_len = header_len + FENCE_FRAME_CHECKSUM_BYTES;
    if bytes.len() < minimum_len
        || &bytes[..SIGNING_FENCE_TRANSITION_MAGIC.len()] != SIGNING_FENCE_TRANSITION_MAGIC
    {
        return Err(PoAError::CorruptSigningFence(
            "invalid or truncated selection-transition marker".to_string(),
        ));
    }
    let record_len = u32::from_be_bytes(
        bytes[SIGNING_FENCE_TRANSITION_MAGIC.len()..header_len]
            .try_into()
            .map_err(|_| {
                PoAError::CorruptSigningFence(
                    "invalid selection-transition record length".to_string(),
                )
            })?,
    ) as usize;
    if record_len == 0 || record_len > MAX_FENCE_RECORD_BYTES {
        return Err(PoAError::CorruptSigningFence(format!(
            "invalid selection-transition record length {record_len}"
        )));
    }
    let expected_len = header_len
        .checked_add(record_len)
        .and_then(|length| length.checked_add(FENCE_FRAME_CHECKSUM_BYTES))
        .ok_or_else(|| {
            PoAError::CorruptSigningFence("selection-transition length overflow".to_string())
        })?;
    if bytes.len() != expected_len {
        return Err(PoAError::CorruptSigningFence(
            "truncated or trailing selection-transition bytes".to_string(),
        ));
    }
    let record = &bytes[header_len..header_len + record_len];
    let checksum = &bytes[header_len + record_len..];
    let expected_checksum = hash_domain_separated_parts(FENCE_TRANSITION_DOMAIN, &[record]);
    if checksum != expected_checksum {
        return Err(PoAError::CorruptSigningFence(
            "selection-transition checksum mismatch".to_string(),
        ));
    }
    Ok(Some(decode_fence_record(record, ledger_binding)?))
}

fn encode_selected_record(
    ledger_binding: LedgerHash,
    selected: &SelectedProposal,
) -> Result<Vec<u8>, PoAError> {
    let request_len = u32::try_from(selected.request_bytes.len())
        .map_err(|_| PoAError::CorruptSigningFence("fenced request is oversized".to_string()))?;
    let body_len = u32::try_from(selected.unsigned_body.len()).map_err(|_| {
        PoAError::CorruptSigningFence("fenced proposal body is oversized".to_string())
    })?;
    let mut bytes = Vec::with_capacity(
        2 + 1
            + 32
            + 8
            + 32
            + 32
            + 32
            + 32
            + 32
            + 4
            + selected.request_bytes.len()
            + 4
            + selected.unsigned_body.len(),
    );
    bytes.extend_from_slice(&SIGNING_FENCE_VERSION.to_be_bytes());
    bytes.push(FENCE_RECORD_SELECTED);
    bytes.extend_from_slice(&ledger_binding);
    bytes.extend_from_slice(&selected.turn.height.to_be_bytes());
    bytes.extend_from_slice(&selected.turn.previous_envelope_hash);
    bytes.extend_from_slice(&selected.turn.previous_state_commitment);
    bytes.extend_from_slice(&selected.validator_public_key);
    bytes.extend_from_slice(&selected.request_digest);
    bytes.extend_from_slice(&selected.proposal_digest);
    bytes.extend_from_slice(&request_len.to_be_bytes());
    bytes.extend_from_slice(&selected.request_bytes);
    bytes.extend_from_slice(&body_len.to_be_bytes());
    bytes.extend_from_slice(&selected.unsigned_body);
    Ok(bytes)
}

fn encode_retired_record(
    ledger_binding: LedgerHash,
    height: u64,
    proposal_digest: LedgerHash,
    envelope_hash: LedgerHash,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(2 + 1 + 32 + 8 + 32 + 32);
    bytes.extend_from_slice(&SIGNING_FENCE_VERSION.to_be_bytes());
    bytes.push(FENCE_RECORD_RETIRED);
    bytes.extend_from_slice(&ledger_binding);
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&proposal_digest);
    bytes.extend_from_slice(&envelope_hash);
    bytes
}

fn encode_observed_signed_record(
    ledger_binding: LedgerHash,
    height: u64,
    proposal_digest: LedgerHash,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(2 + 1 + 32 + 8 + 32);
    bytes.extend_from_slice(&SIGNING_FENCE_VERSION.to_be_bytes());
    bytes.push(FENCE_RECORD_OBSERVED_SIGNED);
    bytes.extend_from_slice(&ledger_binding);
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&proposal_digest);
    bytes
}

fn decode_fence_record(
    bytes: &[u8],
    expected_ledger_binding: LedgerHash,
) -> Result<FenceRecord, PoAError> {
    let mut reader = FenceReader::new(bytes);
    let version = reader.u16("record version")?;
    if version != SIGNING_FENCE_VERSION {
        return Err(PoAError::CorruptSigningFence(format!(
            "unsupported signing-fence version {version}"
        )));
    }
    let record_kind = reader.u8("record kind")?;
    let ledger_binding = reader.fixed::<32>("ledger binding")?;
    if ledger_binding != expected_ledger_binding {
        return Err(PoAError::CorruptSigningFence(
            "signing fence belongs to another ledger instance".to_string(),
        ));
    }
    match record_kind {
        FENCE_RECORD_SELECTED => {
            let turn = PoATurn {
                height: reader.u64("turn height")?,
                previous_envelope_hash: reader.fixed::<32>("previous envelope hash")?,
                previous_state_commitment: reader.fixed::<32>("previous state commitment")?,
            };
            let validator_public_key = reader.fixed::<32>("validator public key")?;
            let request_digest_value = reader.fixed::<32>("request digest")?;
            let proposal_digest = reader.fixed::<32>("proposal digest")?;
            let request_bytes = reader.bytes(MAX_FENCE_REQUEST_BYTES, "canonical request bytes")?;
            let unsigned_body = reader.bytes(MAX_FENCE_PROPOSAL_BYTES, "unsigned proposal body")?;
            reader.finish()?;
            if request_digest(&request_bytes) != request_digest_value {
                return Err(PoAError::CorruptSigningFence(
                    "fenced request digest mismatch".to_string(),
                ));
            }
            validate_fenced_request(&request_bytes, turn)?;
            let candidate = AdmissionCandidate::from_proposal_bytes(&unsigned_body)?;
            if candidate.index != turn.height
                || candidate.previous_envelope_hash != turn.previous_envelope_hash
                || candidate.previous_state_commitment != turn.previous_state_commitment
                || candidate.proposer_public_key != validator_public_key
                || candidate.proposal_digest != proposal_digest
            {
                return Err(PoAError::CorruptSigningFence(
                    "fenced proposal body does not match its record".to_string(),
                ));
            }
            Ok(FenceRecord::Selected(Box::new(SelectedProposal {
                turn,
                validator_public_key,
                request_bytes,
                request_digest: request_digest_value,
                unsigned_body,
                proposal_digest,
                retired_envelope_hash: None,
                outcome: None,
            })))
        }
        FENCE_RECORD_RETIRED => {
            let height = reader.u64("retired height")?;
            let proposal_digest = reader.fixed::<32>("retired proposal digest")?;
            let envelope_hash = reader.fixed::<32>("retired envelope hash")?;
            reader.finish()?;
            Ok(FenceRecord::Retired {
                height,
                proposal_digest,
                envelope_hash,
            })
        }
        FENCE_RECORD_OBSERVED_SIGNED => {
            let height = reader.u64("observed signed height")?;
            let proposal_digest = reader.fixed::<32>("observed signed proposal digest")?;
            reader.finish()?;
            if proposal_digest == [0; 32] {
                return Err(PoAError::CorruptSigningFence(format!(
                    "observed signed proposal digest is empty at height {height}"
                )));
            }
            Ok(FenceRecord::ObservedSigned {
                height,
                proposal_digest,
            })
        }
        _ => Err(PoAError::CorruptSigningFence(format!(
            "unknown signing-fence record kind {record_kind}"
        ))),
    }
}

fn validate_fenced_request(bytes: &[u8], expected_turn: PoATurn) -> Result<(), PoAError> {
    let mut reader = FenceReader::new(bytes);
    let expected_magic = if bytes.starts_with(REQUEST_MAGIC) {
        REQUEST_MAGIC
    } else if bytes.starts_with(PRIVACY_REQUEST_MAGIC) {
        PRIVACY_REQUEST_MAGIC
    } else if bytes.starts_with(BRIDGE_EXPORT_REQUEST_MAGIC) {
        BRIDGE_EXPORT_REQUEST_MAGIC
    } else if bytes.starts_with(BRIDGE_IMPORT_REQUEST_MAGIC) {
        BRIDGE_IMPORT_REQUEST_MAGIC
    } else {
        return Err(PoAError::CorruptSigningFence(
            "invalid fenced request magic".to_string(),
        ));
    };
    let magic = reader.take(expected_magic.len(), "request magic")?;
    if magic != expected_magic {
        return Err(PoAError::CorruptSigningFence(
            "invalid fenced request magic".to_string(),
        ));
    }
    let height = reader.u64("request height")?;
    let previous_envelope_hash = reader.fixed::<32>("request previous envelope hash")?;
    let previous_state_commitment = reader.fixed::<32>("request previous state commitment")?;
    reader.u64("request timestamp")?;
    reader.bytes(MAX_FENCE_REQUEST_BYTES, "request payload")?;
    reader.finish()?;
    if height != expected_turn.height
        || previous_envelope_hash != expected_turn.previous_envelope_hash
        || previous_state_commitment != expected_turn.previous_state_commitment
    {
        return Err(PoAError::CorruptSigningFence(
            "fenced request does not match its PoA turn".to_string(),
        ));
    }
    Ok(())
}

struct FenceReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> FenceReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, len: usize, field: &str) -> Result<&'a [u8], PoAError> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or_else(|| PoAError::CorruptSigningFence(format!("{field} length overflow")))?;
        if end > self.bytes.len() {
            return Err(PoAError::CorruptSigningFence(format!("truncated {field}")));
        }
        let value = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(value)
    }

    fn u8(&mut self, field: &str) -> Result<u8, PoAError> {
        Ok(self.take(1, field)?[0])
    }

    fn u16(&mut self, field: &str) -> Result<u16, PoAError> {
        Ok(u16::from_be_bytes(
            self.take(2, field)?
                .try_into()
                .map_err(|_| PoAError::CorruptSigningFence(format!("invalid {field}")))?,
        ))
    }

    fn u32(&mut self, field: &str) -> Result<u32, PoAError> {
        Ok(u32::from_be_bytes(
            self.take(4, field)?
                .try_into()
                .map_err(|_| PoAError::CorruptSigningFence(format!("invalid {field}")))?,
        ))
    }

    fn u64(&mut self, field: &str) -> Result<u64, PoAError> {
        Ok(u64::from_be_bytes(
            self.take(8, field)?
                .try_into()
                .map_err(|_| PoAError::CorruptSigningFence(format!("invalid {field}")))?,
        ))
    }

    fn fixed<const N: usize>(&mut self, field: &str) -> Result<[u8; N], PoAError> {
        self.take(N, field)?
            .try_into()
            .map_err(|_| PoAError::CorruptSigningFence(format!("invalid fixed-width {field}")))
    }

    fn bytes(&mut self, max: usize, field: &str) -> Result<Vec<u8>, PoAError> {
        let len = self.u32(&format!("{field} length"))? as usize;
        if len > max {
            return Err(PoAError::CorruptSigningFence(format!(
                "{field} exceeds {max} bytes"
            )));
        }
        Ok(self.take(len, field)?.to_vec())
    }

    fn finish(&self) -> Result<(), PoAError> {
        if self.offset != self.bytes.len() {
            return Err(PoAError::CorruptSigningFence(
                "trailing signing-fence record bytes".to_string(),
            ));
        }
        Ok(())
    }
}

fn lock_signing_fence(file: &File) -> io::Result<()> {
    #[cfg(unix)]
    {
        // SAFETY: `file` is an open descriptor owned by this journal and the
        // call applies only a non-blocking advisory lock to that descriptor.
        let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if result == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
    #[cfg(not(unix))]
    {
        let _ = file;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "durable signing-fence locking is unsupported on this platform",
        ))
    }
}

/// One per-ledger selector and exclusive durable signing-fence writer.
pub(crate) struct ProposalCoordinatorState {
    selected: BTreeMap<u64, SelectedProposal>,
    locked_proposal_digests: BTreeMap<u64, LedgerHash>,
    signing_fence: SigningFenceJournal,
    #[cfg(test)]
    crash_after_fence: bool,
    #[cfg(test)]
    crash_after_signing: bool,
    #[cfg(test)]
    crash_before_retirement: bool,
    #[cfg(test)]
    signing_invocations: usize,
}

impl ProposalCoordinatorState {
    pub(crate) fn open(
        path: impl AsRef<Path>,
        ledger_binding: LedgerHash,
    ) -> Result<Self, PoAError> {
        let signing_fence = SigningFenceJournal::open(path, ledger_binding)?;
        let recovered = signing_fence.replay()?;
        Ok(Self {
            selected: recovered.selected,
            locked_proposal_digests: recovered.locked_proposal_digests,
            signing_fence,
            #[cfg(test)]
            crash_after_fence: false,
            #[cfg(test)]
            crash_after_signing: false,
            #[cfg(test)]
            crash_before_retirement: false,
            #[cfg(test)]
            signing_invocations: 0,
        })
    }

    #[cfg(test)]
    fn open_validator(
        path: impl AsRef<Path>,
        ledger_binding: LedgerHash,
    ) -> Result<Self, PoAError> {
        Self::open(path, ledger_binding)
    }

    /// Reconcile every durable proposal lock with any envelope already committed at its height.
    pub(crate) fn validate_committed_locks(&self, ledger: &Ledger) -> Result<(), PoAError> {
        for (height, proposal_digest) in &self.locked_proposal_digests {
            let index = ledger_index(*height)?;
            if let Some(envelope) = ledger.committed_envelopes().get(index) {
                ensure_matching_proposal(*height, *proposal_digest, envelope.proposal_digest)?;
            }
        }
        Ok(())
    }

    pub(crate) fn recover(
        &mut self,
        ledger: &mut Ledger,
        authority_schedule: &AuthoritySchedule,
        local_node_id: Uuid,
        signing_key: &SigningKey,
    ) -> Result<(), PoAError> {
        let heights: Vec<u64> = self.selected.keys().copied().collect();
        for height in heights {
            let selected = self
                .selected
                .get(&height)
                .cloned()
                .ok_or(PoAError::CoordinatorUnavailable)?;
            if let Some(envelope) = ledger
                .committed_envelopes()
                .get(ledger_index(height)?)
                .cloned()
            {
                ensure_matching_proposal(
                    height,
                    selected.proposal_digest,
                    envelope.proposal_digest,
                )?;
                let expected_envelope_hash = selected
                    .retired_envelope_hash
                    .unwrap_or(envelope.envelope_hash);
                self.finish_committed_selection(
                    ledger,
                    height,
                    selected.proposal_digest,
                    expected_envelope_hash,
                    AdmissionOutcome::Committed {
                        envelope: Box::new(envelope),
                    },
                )?;
                continue;
            }

            let pending_turn = PoATurn::from_tip(ledger.tip());
            if selected.turn != pending_turn {
                return Err(PoAError::CorruptSigningFence(format!(
                    "uncommitted fenced height {height} is not the pending turn"
                )));
            }
            self.process_selected(
                selected,
                ledger,
                authority_schedule,
                local_node_id,
                signing_key,
            )?;
        }
        Ok(())
    }

    pub(crate) fn submit(
        &mut self,
        request: PoAProposalRequest,
        ledger: &mut Ledger,
        authority_schedule: &AuthoritySchedule,
        local_node_id: Uuid,
        signing_key: &SigningKey,
        max_block_size: usize,
    ) -> Result<AdmissionOutcome, PoAError> {
        let request_bytes = request.canonical_bytes()?;
        let request_digest = request.digest()?;
        if let Some(selected) = self.selected.get(&request.turn.height).cloned() {
            if selected.request_digest != request_digest || selected.request_bytes != request_bytes
            {
                return Err(PoAError::ProposalConflict {
                    height: request.turn.height,
                    selected_proposal_digest: selected.proposal_digest,
                });
            }
            if let Some(outcome) = selected.outcome {
                return Ok(outcome);
            }
            return self.process_selected(
                selected,
                ledger,
                authority_schedule,
                local_node_id,
                signing_key,
            );
        }

        let pending_turn = PoATurn::from_tip(ledger.tip());
        if request.turn != pending_turn {
            return Err(PoAError::StaleTurn);
        }
        let scheduled = authority_schedule.authority_at(pending_turn.height);
        Self::validate_local_authority(pending_turn.height, scheduled, local_node_id, signing_key)?;
        if request.payload_len() > max_block_size {
            return Err(PoAError::ProposalTooLarge {
                bytes: request.payload_len(),
                limit: max_block_size,
            });
        }
        validate_block_interval(
            ledger,
            request.turn.height,
            request.timestamp_millis,
            authority_schedule.minimum_block_interval_millis,
        )?;

        // Complete all deterministic, read-only preflight before selection.
        let unsigned_candidate = match &request.payload {
            PoARequestPayload::OrdinaryProvenance(public_provenance) => ledger
                .create_unsigned_ordinary_candidate(
                    public_provenance,
                    request.timestamp_millis(),
                    signing_key.verifying_key().to_bytes(),
                )?,
            PoARequestPayload::PrivacyControl(privacy_control) => {
                let transition = PrivacyControlTransition::decode(privacy_control)
                    .map_err(|error| LedgerError::MalformedCandidate(error.to_string()))?;
                ledger.create_unsigned_privacy_candidate(
                    &transition,
                    request.timestamp_millis(),
                    signing_key.verifying_key().to_bytes(),
                )?
            }
            PoARequestPayload::BridgeExport {
                public_provenance,
                target,
            } => ledger.create_unsigned_bridge_export_candidate(
                public_provenance,
                target.clone(),
                request.timestamp_millis(),
                signing_key.verifying_key().to_bytes(),
            )?,
            PoARequestPayload::BridgeImport {
                public_provenance,
                origin_evidence,
            } => ledger.create_unsigned_bridge_import_candidate(
                public_provenance,
                origin_evidence,
                request.timestamp_millis(),
                signing_key.verifying_key().to_bytes(),
            )?,
        };
        ledger.preflight_candidate_envelope_size(&unsigned_candidate)?;
        let unsigned_body = unsigned_candidate.proposal_bytes()?;
        if request_bytes.len() > MAX_FENCE_REQUEST_BYTES {
            return Err(PoAError::ProposalTooLarge {
                bytes: request_bytes.len(),
                limit: MAX_FENCE_REQUEST_BYTES,
            });
        }
        if unsigned_body.len() > MAX_FENCE_PROPOSAL_BYTES {
            return Err(PoAError::ProposalTooLarge {
                bytes: unsigned_body.len(),
                limit: MAX_FENCE_PROPOSAL_BYTES,
            });
        }
        let proposal_digest = unsigned_candidate.proposal_digest;
        if let Some(locked_digest) = self.locked_proposal_digests.get(&request.turn.height) {
            ensure_matching_proposal(request.turn.height, *locked_digest, proposal_digest)?;
        }
        let selected = SelectedProposal {
            turn: request.turn,
            validator_public_key: signing_key.verifying_key().to_bytes(),
            request_bytes,
            request_digest,
            unsigned_body,
            proposal_digest,
            retired_envelope_hash: None,
            outcome: None,
        };

        // This append plus sync_all is the irreversible signing boundary.
        self.persist_selection(&selected)?;
        self.selected.insert(selected.turn.height, selected.clone());
        self.locked_proposal_digests
            .insert(selected.turn.height, selected.proposal_digest);
        #[cfg(test)]
        if self.crash_after_fence {
            self.crash_after_fence = false;
            return Err(PoAError::SigningFenceUncertain(
                "injected process crash after durable fence".to_string(),
            ));
        }
        self.process_selected(
            selected,
            ledger,
            authority_schedule,
            local_node_id,
            signing_key,
        )
    }

    pub(crate) fn accept_signed(
        &mut self,
        candidate: AdmissionCandidate,
        ledger: &mut Ledger,
        authority_schedule: &AuthoritySchedule,
        max_block_size: usize,
    ) -> Result<AdmissionOutcome, PoAError> {
        let (height, proposal_digest) =
            self.prepare_signed_candidate(&candidate, ledger, authority_schedule, max_block_size)?;
        let outcome = ledger.proposal_adapter().submit(candidate)?;
        self.finish_received_outcome(ledger, height, proposal_digest, outcome)
    }

    /// Verify a producer's exact committed envelope, durably lock its PoA
    /// digest, and submit those unchanged bytes to universal Final Admission.
    pub(crate) fn accept_exact_envelope(
        &mut self,
        envelope_bytes: &[u8],
        ingress: ExactEnvelopeIngress,
        ledger: &mut Ledger,
        authority_schedule: &AuthoritySchedule,
        max_block_size: usize,
    ) -> Result<AdmissionOutcome, PoAError> {
        let envelope = AdmittedBlockEnvelope::decode(envelope_bytes)?;
        let candidate = AdmissionCandidate::from_admitted_envelope(&envelope);
        let (height, proposal_digest) =
            self.prepare_signed_candidate(&candidate, ledger, authority_schedule, max_block_size)?;
        let outcome = match ingress {
            ExactEnvelopeIngress::Replication => ledger
                .replication_adapter()
                .submit_exact_envelope(envelope_bytes)?,
            ExactEnvelopeIngress::Synchronization => ledger
                .synchronization_adapter()
                .submit_exact_envelope(envelope_bytes)?,
        };
        self.finish_received_outcome(ledger, height, proposal_digest, outcome)
    }

    fn prepare_signed_candidate(
        &mut self,
        candidate: &AdmissionCandidate,
        ledger: &mut Ledger,
        authority_schedule: &AuthoritySchedule,
        max_block_size: usize,
    ) -> Result<(u64, LedgerHash), PoAError> {
        candidate
            .verify_signed_proposal()
            .map_err(|error| PoAError::InvalidSignedProposal(error.to_string()))?;
        let scheduled = authority_schedule.authority_at(candidate.index);
        if candidate.proposer_public_key != scheduled.validator_public_key {
            return Err(PoAError::UnscheduledSigner {
                height: candidate.index,
                scheduled_validator_public_key: scheduled.validator_public_key,
                attempted_validator_public_key: candidate.proposer_public_key,
            });
        }

        let height = candidate.index;
        let proposal_digest = candidate.proposal_digest;
        if let Some(selected) = self.selected.get(&candidate.index) {
            ensure_matching_proposal(height, selected.proposal_digest, proposal_digest)?;
            if selected.unsigned_body != candidate.proposal_bytes()? {
                return Err(PoAError::CorruptSigningFence(format!(
                    "proposal digest collision at fenced height {}",
                    candidate.index
                )));
            }
        }
        if let Some(locked_digest) = self.locked_proposal_digests.get(&height) {
            ensure_matching_proposal(height, *locked_digest, proposal_digest)?;
        }
        if let Ok(index) = usize::try_from(height) {
            if let Some(existing) = ledger.committed_envelopes().get(index) {
                ensure_matching_proposal(height, existing.proposal_digest, proposal_digest)?;
            }
        }

        let proposal_turn = PoATurn {
            height,
            previous_envelope_hash: candidate.previous_envelope_hash,
            previous_state_commitment: candidate.previous_state_commitment,
        };
        let pending_turn = PoATurn::from_tip(ledger.tip());
        let is_exact_committed_retry = usize::try_from(height)
            .ok()
            .and_then(|index| ledger.committed_envelopes().get(index))
            .is_some_and(|existing| existing.proposal_digest == proposal_digest);
        if proposal_turn != pending_turn && !is_exact_committed_retry {
            return Err(PoAError::StaleTurn);
        }

        if !self.locked_proposal_digests.contains_key(&height) {
            self.persist_observed_signed(height, proposal_digest)?;
            self.locked_proposal_digests.insert(height, proposal_digest);
        }

        // A valid scheduled signature fixes the turn even when a later
        // deterministic admission check rejects it.
        let candidate_payload_bytes = candidate
            .privacy_control
            .as_ref()
            .map_or(candidate.public_provenance.len(), Vec::len);
        if candidate_payload_bytes > max_block_size {
            return Err(PoAError::ProposalTooLarge {
                bytes: candidate_payload_bytes,
                limit: max_block_size,
            });
        }
        validate_block_interval(
            ledger,
            height,
            candidate.timestamp_millis,
            authority_schedule.minimum_block_interval_millis,
        )?;
        ledger.preflight_candidate_envelope_size(candidate)?;

        Ok((height, proposal_digest))
    }

    fn finish_received_outcome(
        &mut self,
        ledger: &mut Ledger,
        height: u64,
        proposal_digest: LedgerHash,
        outcome: AdmissionOutcome,
    ) -> Result<AdmissionOutcome, PoAError> {
        let envelope_hash = match &outcome {
            AdmissionOutcome::Committed { envelope } => envelope.envelope_hash,
            AdmissionOutcome::Rejected { reason } => {
                return Err(PoAError::FinalAdmissionRejected {
                    height,
                    reason: reason.clone(),
                });
            }
        };

        if self.selected.contains_key(&height) {
            self.finish_committed_selection(
                ledger,
                height,
                proposal_digest,
                envelope_hash,
                outcome.clone(),
            )?;
        }
        Ok(outcome)
    }

    fn process_selected(
        &mut self,
        selected: SelectedProposal,
        ledger: &mut Ledger,
        authority_schedule: &AuthoritySchedule,
        local_node_id: Uuid,
        signing_key: &SigningKey,
    ) -> Result<AdmissionOutcome, PoAError> {
        let scheduled = authority_schedule.authority_at(selected.turn.height);
        Self::validate_local_authority(
            selected.turn.height,
            scheduled,
            local_node_id,
            signing_key,
        )?;
        if selected.validator_public_key != signing_key.verifying_key().to_bytes() {
            return Err(PoAError::CorruptSigningFence(format!(
                "fenced validator key mismatch at height {}",
                selected.turn.height
            )));
        }

        let unsigned_candidate = AdmissionCandidate::from_proposal_bytes(&selected.unsigned_body)?;
        if unsigned_candidate.index != selected.turn.height
            || unsigned_candidate.previous_envelope_hash != selected.turn.previous_envelope_hash
            || unsigned_candidate.previous_state_commitment
                != selected.turn.previous_state_commitment
            || unsigned_candidate.proposer_public_key != selected.validator_public_key
            || unsigned_candidate.proposal_digest != selected.proposal_digest
        {
            return Err(PoAError::CorruptSigningFence(format!(
                "fenced proposal facts mismatch at height {}",
                selected.turn.height
            )));
        }
        validate_block_interval(
            ledger,
            selected.turn.height,
            unsigned_candidate.timestamp_millis,
            authority_schedule.minimum_block_interval_millis,
        )?;
        ledger.preflight_candidate_envelope_size(&unsigned_candidate)?;

        if let Some(existing) = ledger
            .committed_envelopes()
            .get(ledger_index(selected.turn.height)?)
        {
            ensure_matching_proposal(
                selected.turn.height,
                selected.proposal_digest,
                existing.proposal_digest,
            )?;
        }

        // Ed25519 signing happens only after the exact body is durably fenced.
        #[cfg(test)]
        {
            self.signing_invocations += 1;
        }
        let signed_candidate = unsigned_candidate.sign(signing_key)?;
        #[cfg(test)]
        if self.crash_after_signing {
            self.crash_after_signing = false;
            return Err(PoAError::SigningFenceUncertain(
                "injected process crash after deterministic signing".to_string(),
            ));
        }
        let outcome = ledger.proposal_adapter().submit(signed_candidate)?;
        #[cfg(test)]
        if self.crash_before_retirement {
            self.crash_before_retirement = false;
            return Err(PoAError::SigningFenceUncertain(
                "injected process crash after ledger commit response".to_string(),
            ));
        }
        let envelope_hash = match &outcome {
            AdmissionOutcome::Committed { envelope } => envelope.as_ref(),
            AdmissionOutcome::Rejected { reason } => {
                return Err(PoAError::FinalAdmissionRejected {
                    height: selected.turn.height,
                    reason: reason.clone(),
                });
            }
        }
        .envelope_hash;

        // Retirement follows Verified Journal Replay and exact digest/hash proof.
        self.finish_committed_selection(
            ledger,
            selected.turn.height,
            selected.proposal_digest,
            envelope_hash,
            outcome.clone(),
        )?;
        Ok(outcome)
    }

    fn finish_committed_selection(
        &mut self,
        ledger: &mut Ledger,
        height: u64,
        proposal_digest: LedgerHash,
        envelope_hash: LedgerHash,
        outcome: AdmissionOutcome,
    ) -> Result<(), PoAError> {
        let replay = ledger.replay()?;
        let verified = replay.envelopes.get(ledger_index(height)?).ok_or_else(|| {
            PoAError::CorruptSigningFence(format!(
                "committed fence has no replayed envelope at height {height}"
            ))
        })?;
        ensure_matching_proposal(height, proposal_digest, verified.proposal_digest)?;
        if verified.envelope_hash != envelope_hash {
            return Err(PoAError::CorruptSigningFence(format!(
                "replayed envelope hash mismatch at height {height}"
            )));
        }
        self.retire(height, verified.envelope_hash)?;
        self.complete(height, outcome)
    }

    fn validate_local_authority(
        height: u64,
        scheduled: ScheduledAuthority,
        local_node_id: Uuid,
        signing_key: &SigningKey,
    ) -> Result<(), PoAError> {
        if scheduled.node_id != local_node_id {
            return Err(PoAError::OutOfTurn {
                height,
                scheduled_node_id: scheduled.node_id,
                attempted_node_id: local_node_id,
            });
        }
        if scheduled.validator_public_key != signing_key.verifying_key().to_bytes() {
            return Err(PoAError::LocalValidatorKeyMismatch(local_node_id));
        }
        Ok(())
    }

    fn persist_selection(&mut self, selected: &SelectedProposal) -> Result<(), PoAError> {
        self.signing_fence.append_selected(selected)
    }

    fn persist_observed_signed(
        &mut self,
        height: u64,
        proposal_digest: LedgerHash,
    ) -> Result<(), PoAError> {
        self.signing_fence
            .append_observed_signed(height, proposal_digest)
    }

    fn retire(&mut self, height: u64, envelope_hash: LedgerHash) -> Result<(), PoAError> {
        let selected = self
            .selected
            .get(&height)
            .ok_or(PoAError::CoordinatorUnavailable)?;
        if let Some(existing) = selected.retired_envelope_hash {
            if existing == envelope_hash {
                return Ok(());
            }
            return Err(PoAError::CorruptSigningFence(format!(
                "conflicting retirement at height {height}"
            )));
        }
        let proposal_digest = selected.proposal_digest;
        self.signing_fence
            .append_retired(height, proposal_digest, envelope_hash)?;
        self.selected
            .get_mut(&height)
            .ok_or(PoAError::CoordinatorUnavailable)?
            .retired_envelope_hash = Some(envelope_hash);
        Ok(())
    }

    fn complete(&mut self, height: u64, outcome: AdmissionOutcome) -> Result<(), PoAError> {
        let selected = self
            .selected
            .get_mut(&height)
            .ok_or(PoAError::CoordinatorUnavailable)?;
        selected.outcome = Some(outcome);
        Ok(())
    }

    #[cfg(test)]
    fn inject_crash_after_fence(&mut self) {
        self.crash_after_fence = true;
    }

    #[cfg(test)]
    fn inject_crash_after_signing(&mut self) {
        self.crash_after_signing = true;
    }

    #[cfg(test)]
    fn inject_crash_before_retirement(&mut self) {
        self.crash_before_retirement = true;
    }

    #[cfg(test)]
    fn signing_invocations(&self) -> usize {
        self.signing_invocations
    }
}

impl AuthoritySchedule {
    pub(crate) fn from_contract(
        profile: &NetworkProfile,
        membership: &ActiveMembership,
    ) -> Result<Self, PoAError> {
        if profile.consensus.consensus_type != "poa" {
            return Err(PoAError::UnsupportedConsensus(
                profile.consensus.consensus_type.clone(),
            ));
        }
        if profile.consensus.authority_keys.is_empty() {
            return Err(PoAError::EmptyAuthorityOrder);
        }
        let minimum_block_interval_millis =
            profile.consensus.block_interval.checked_mul(1_000).ok_or(
                PoAError::BlockIntervalOverflow(profile.consensus.block_interval),
            )?;

        let manifest = &membership.signed_manifest().manifest;
        let mut ordered_authorities = Vec::with_capacity(profile.consensus.authority_keys.len());
        for encoded_key in &profile.consensus.authority_keys {
            let decoded = hex::decode(encoded_key)
                .map_err(|_| PoAError::InvalidAuthorityKey(encoded_key.clone()))?;
            let validator_public_key: [u8; 32] = decoded
                .try_into()
                .map_err(|_| PoAError::InvalidAuthorityKey(encoded_key.clone()))?;
            VerifyingKey::from_bytes(&validator_public_key)
                .map_err(|_| PoAError::InvalidAuthorityKey(encoded_key.clone()))?;

            let member = manifest
                .members
                .iter()
                .find(|member| {
                    member.status == MemberStatus::Active
                        && member.roles.contains(&MemberRole::Validator)
                        && member.validator_public_key == Some(validator_public_key)
                })
                .ok_or_else(|| PoAError::UnresolvedAuthority(encoded_key.clone()))?;
            ordered_authorities.push(ScheduledAuthority {
                node_id: member.node_id,
                validator_public_key,
            });
        }

        Ok(Self {
            ordered_authorities,
            minimum_block_interval_millis,
        })
    }

    pub(crate) fn authority_at(&self, height: u64) -> ScheduledAuthority {
        let offset = (height % self.ordered_authorities.len() as u64) as usize;
        self.ordered_authorities[offset]
    }

    pub(crate) fn ordered_authorities(&self) -> &[ScheduledAuthority] {
        &self.ordered_authorities
    }
}

/// Fail-closed PoA scheduling and coordination errors.
#[derive(Debug, Error)]
pub enum PoAError {
    /// The reference schedule is defined only for PoA.
    #[error("scheduled authority is unavailable for consensus type {0}")]
    UnsupportedConsensus(String),
    /// A PoA profile must declare at least one manifest-authorized validator.
    #[error("PoA authority order is empty")]
    EmptyAuthorityOrder,
    /// The seconds-based profile interval cannot be represented in milliseconds.
    #[error("PoA block_interval {0} seconds overflows millisecond timestamps")]
    BlockIntervalOverflow(u64),
    /// One configured validator key is malformed.
    #[error("invalid PoA authority key {0}")]
    InvalidAuthorityKey(String),
    /// One configured validator key does not resolve to an active manifest role.
    #[error("PoA authority key does not resolve through active membership: {0}")]
    UnresolvedAuthority(String),
    /// This node has no activated proposal-signing capability.
    #[error("local validator signing key is not activated")]
    MissingLocalSigningKey,
    /// A validator other than the Scheduled Authority attempted this turn.
    #[error(
        "validator {attempted_node_id} is out of turn at height {height}; scheduled validator is {scheduled_node_id}"
    )]
    OutOfTurn {
        /// Ledger height whose authority was checked.
        height: u64,
        /// Unique manifest-authorized validator for the height.
        scheduled_node_id: Uuid,
        /// Local validator that attempted to propose.
        attempted_node_id: Uuid,
    },
    /// The request was built for a different journal-derived turn.
    #[error("proposal request does not match the pending PoA turn")]
    StaleTurn,
    /// Selection already fixed another exact unsigned body for this turn.
    #[error(
        "PoA turn {height} is already locked to proposal {}",
        hex::encode(selected_proposal_digest)
    )]
    ProposalConflict {
        /// Ledger height whose selection is immutable.
        height: u64,
        /// Digest of the exact body selected first.
        selected_proposal_digest: LedgerHash,
    },
    /// The request or proposal exceeds the active deterministic size bound.
    #[error("PoA proposal uses {bytes} bytes, exceeding the {limit}-byte limit")]
    ProposalTooLarge {
        /// Observed byte length.
        bytes: usize,
        /// Active upper bound.
        limit: usize,
    },
    /// The candidate timestamp does not satisfy the profile's minimum spacing.
    #[error(
        "PoA proposal at height {height} uses timestamp {timestamp_millis}, before minimum {minimum_timestamp_millis}"
    )]
    ProposalTooEarly {
        /// Ledger height whose predecessor fixes the minimum.
        height: u64,
        /// Candidate timestamp in Unix milliseconds.
        timestamp_millis: u64,
        /// Earliest eligible timestamp in Unix milliseconds.
        minimum_timestamp_millis: u64,
    },
    /// No representable timestamp can satisfy the configured spacing at this height.
    #[error("PoA timestamp spacing overflows at height {height}")]
    ProposalTimestampOverflow {
        /// Ledger height whose predecessor timestamp overflowed.
        height: u64,
    },
    /// The activated local proposal key differs from the scheduled manifest key.
    #[error("local validator {0} has the wrong PoA signing key")]
    LocalValidatorKeyMismatch(Uuid),
    /// A received candidate lacks valid canonical proposer evidence.
    #[error("invalid signed PoA proposal: {0}")]
    InvalidSignedProposal(String),
    /// A signed proposal came from a validator other than the scheduled one.
    #[error(
        "proposal signer {} is not scheduled at height {height}; scheduled key is {}",
        hex::encode(attempted_validator_public_key),
        hex::encode(scheduled_validator_public_key)
    )]
    UnscheduledSigner {
        /// Ledger height whose authority was checked.
        height: u64,
        /// Validator key fixed by the height-derived authority order.
        scheduled_validator_public_key: [u8; 32],
        /// Validator key carried by the received signed proposal.
        attempted_validator_public_key: [u8; 32],
    },
    /// Durable signing-fence bytes are missing, malformed, or conflict.
    #[error("corrupt PoA signing fence: {0}")]
    CorruptSigningFence(String),
    /// A fence persistence result cannot be proven safe without recovery.
    #[error("PoA signing-fence durability is unresolved: {0}")]
    SigningFenceUncertain(String),
    /// Distinct proposal digests exist for one PoA turn.
    #[error(
        "PoA equivocation at height {height}: selected {}, conflicting {}",
        hex::encode(selected_proposal_digest),
        hex::encode(conflicting_proposal_digest)
    )]
    Equivocation {
        /// Ledger height containing the conflict.
        height: u64,
        /// Digest locked by the coordinator or durable fence.
        selected_proposal_digest: LedgerHash,
        /// Different signed or committed proposal digest.
        conflicting_proposal_digest: LedgerHash,
    },
    /// A selected and signed proposal failed deterministic Final Admission.
    #[error("selected PoA proposal at height {height} was rejected: {reason}")]
    FinalAdmissionRejected {
        /// Ledger height that must remain stalled.
        height: u64,
        /// Deterministic rejection reason.
        reason: String,
    },
    /// Filesystem failure while opening or reading the durable fence.
    #[error("PoA signing-fence I/O error: {0}")]
    SigningFenceIo(#[from] io::Error),
    /// A poisoned coordinator lock cannot be treated as safe state.
    #[error("PoA coordinator state is unavailable")]
    CoordinatorUnavailable,
    /// Universal Final Admission or ledger recovery failed.
    #[error(transparent)]
    Ledger(#[from] LedgerError),
}

/// Result of processing one selected PoA proposal locally.
pub type PoAProposalOutcome = AdmissionOutcome;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::LedgerProfile;
    use crate::ontology::semantic_admission::activated_test_package;
    use tempfile::{tempdir, TempDir};

    fn test_ledger_profile(profile_id: &str) -> LedgerProfile {
        let package = activated_test_package();
        LedgerProfile::new("issue5.unit", profile_id).with_semantic_package(
            package.package_id(),
            package.package_version(),
            package.package_hash(),
        )
    }

    struct CoordinatorFixture {
        _data_dir: TempDir,
        ledger: Ledger,
        coordinator: ProposalCoordinatorState,
        schedule: AuthoritySchedule,
        node_id: Uuid,
        signing_key: SigningKey,
        fence_path: PathBuf,
        ledger_binding: LedgerHash,
    }

    fn fixture(seed: u8) -> CoordinatorFixture {
        let data_dir = tempdir().expect("temporary coordinator directory");
        let ledger = Ledger::open_in_dir(
            data_dir.path(),
            test_ledger_profile("issue5.unit.profile"),
            activated_test_package(),
        )
        .expect("open test ledger");
        let signing_key = SigningKey::from_bytes(&[seed; 32]);
        let node_id = Uuid::from_u128(seed as u128 + 1);
        let schedule = AuthoritySchedule {
            ordered_authorities: vec![ScheduledAuthority {
                node_id,
                validator_public_key: signing_key.verifying_key().to_bytes(),
            }],
            minimum_block_interval_millis: 10_000,
        };
        let fence_path = data_dir.path().join(SIGNING_FENCE_FILENAME);
        let ledger_binding = [seed.wrapping_add(1); 32];
        let coordinator = ProposalCoordinatorState::open_validator(&fence_path, ledger_binding)
            .expect("open test coordinator");
        CoordinatorFixture {
            _data_dir: data_dir,
            ledger,
            coordinator,
            schedule,
            node_id,
            signing_key,
            fence_path,
            ledger_binding,
        }
    }

    fn request(ledger: &Ledger, timestamp_millis: u64, value: &str) -> PoAProposalRequest {
        PoAProposalRequest::ordinary(
            PoATurn::from_tip(ledger.tip()),
            timestamp_millis,
            format!("<urn:issue5:unit> <urn:issue5:value> \"{value}\" ."),
        )
    }

    #[test]
    fn fence_write_and_fsync_failures_stall_before_signing_or_commit() {
        for fail_fsync in [false, true] {
            let mut fixture = fixture(if fail_fsync { 81 } else { 80 });
            let request = request(&fixture.ledger, 80_000, "failure");
            let fence = &mut fixture.coordinator.signing_fence;
            if fail_fsync {
                fence.inject_next_fsync_failure();
            } else {
                fence.inject_next_write_failure();
            }

            assert!(matches!(
                fixture.coordinator.submit(
                    request.clone(),
                    &mut fixture.ledger,
                    &fixture.schedule,
                    fixture.node_id,
                    &fixture.signing_key,
                    1_048_576,
                ),
                Err(PoAError::SigningFenceUncertain(_))
            ));
            assert_eq!(fixture.coordinator.signing_invocations(), 0);
            assert_eq!(
                fixture.ledger.journal_frame_count().expect("frame count"),
                0
            );
            assert!(matches!(
                fixture.coordinator.submit(
                    request,
                    &mut fixture.ledger,
                    &fixture.schedule,
                    fixture.node_id,
                    &fixture.signing_key,
                    1_048_576,
                ),
                Err(PoAError::SigningFenceUncertain(_))
            ));
            assert_eq!(fixture.coordinator.signing_invocations(), 0);
        }
    }

    #[test]
    fn failed_fence_write_cannot_become_refreshable_after_restart() {
        let mut fixture = fixture(89);
        let request = request(&fixture.ledger, 89_000, "write-failure");
        fixture
            .coordinator
            .signing_fence
            .inject_next_write_failure();
        assert!(matches!(
            fixture.coordinator.submit(
                request,
                &mut fixture.ledger,
                &fixture.schedule,
                fixture.node_id,
                &fixture.signing_key,
                1_048_576,
            ),
            Err(PoAError::SigningFenceUncertain(_))
        ));
        drop(fixture.coordinator);
        drop(fixture.ledger);

        assert!(matches!(
            ProposalCoordinatorState::open_validator(&fixture.fence_path, fixture.ledger_binding,),
            Err(PoAError::SigningFenceUncertain(_))
        ));
    }

    #[test]
    fn ambiguous_fsync_restart_recovers_only_the_exact_persisted_selection() {
        let mut fixture = fixture(93);
        let request = request(&fixture.ledger, 93_000, "fsync-failure");
        let expected_digest = fixture
            .ledger
            .create_unsigned_ordinary_candidate(
                request.public_provenance(),
                request.timestamp_millis(),
                fixture.signing_key.verifying_key().to_bytes(),
            )
            .expect("preflight expected candidate")
            .proposal_digest;
        fixture
            .coordinator
            .signing_fence
            .inject_next_fsync_failure();
        assert!(matches!(
            fixture.coordinator.submit(
                request,
                &mut fixture.ledger,
                &fixture.schedule,
                fixture.node_id,
                &fixture.signing_key,
                1_048_576,
            ),
            Err(PoAError::SigningFenceUncertain(_))
        ));
        drop(fixture.coordinator);
        drop(fixture.ledger);

        let mut ledger = Ledger::open_in_dir(
            fixture._data_dir.path(),
            test_ledger_profile("issue5.unit.profile"),
            activated_test_package(),
        )
        .expect("reopen test ledger");
        let mut coordinator =
            ProposalCoordinatorState::open_validator(&fixture.fence_path, fixture.ledger_binding)
                .expect("verify the exact persisted selection");
        coordinator
            .recover(
                &mut ledger,
                &fixture.schedule,
                fixture.node_id,
                &fixture.signing_key,
            )
            .expect("recover only the persisted selection");
        assert_eq!(
            ledger.committed_envelopes()[0].proposal_digest,
            expected_digest
        );
        assert_eq!(ledger.journal_frame_count().expect("frame count"), 1);
    }

    #[test]
    fn crash_after_durable_fence_recovers_and_signs_only_the_exact_body() {
        let mut fixture = fixture(82);
        let fenced_request = request(&fixture.ledger, 82_000, "fenced");
        let expected_digest = fixture
            .ledger
            .create_unsigned_ordinary_candidate(
                fenced_request.public_provenance(),
                fenced_request.timestamp_millis(),
                fixture.signing_key.verifying_key().to_bytes(),
            )
            .expect("preflight expected candidate")
            .proposal_digest;
        fixture.coordinator.inject_crash_after_fence();
        assert!(matches!(
            fixture.coordinator.submit(
                fenced_request,
                &mut fixture.ledger,
                &fixture.schedule,
                fixture.node_id,
                &fixture.signing_key,
                1_048_576,
            ),
            Err(PoAError::SigningFenceUncertain(_))
        ));
        assert_eq!(fixture.coordinator.signing_invocations(), 0);
        assert_eq!(
            fixture.ledger.journal_frame_count().expect("frame count"),
            0
        );
        let replacement = request(&fixture.ledger, 82_001, "replacement");
        assert!(matches!(
            fixture.coordinator.submit(
                replacement,
                &mut fixture.ledger,
                &fixture.schedule,
                fixture.node_id,
                &fixture.signing_key,
                1_048_576,
            ),
            Err(PoAError::ProposalConflict { height: 0, .. })
        ));
        assert_eq!(fixture.coordinator.signing_invocations(), 0);

        drop(fixture.coordinator);
        drop(fixture.ledger);
        let mut ledger = Ledger::open_in_dir(
            fixture._data_dir.path(),
            test_ledger_profile("issue5.unit.profile"),
            activated_test_package(),
        )
        .expect("reopen test ledger");
        let mut coordinator =
            ProposalCoordinatorState::open_validator(&fixture.fence_path, fixture.ledger_binding)
                .expect("reopen fenced coordinator");
        coordinator
            .recover(
                &mut ledger,
                &fixture.schedule,
                fixture.node_id,
                &fixture.signing_key,
            )
            .expect("recover exact fenced proposal");

        assert_eq!(coordinator.signing_invocations(), 1);
        assert_eq!(ledger.journal_frame_count().expect("frame count"), 1);
        assert_eq!(
            ledger.committed_envelopes()[0].proposal_digest,
            expected_digest
        );
    }

    #[test]
    fn crash_after_signature_recovers_the_identical_deterministic_signature() {
        let mut fixture = fixture(86);
        let request = request(&fixture.ledger, 86_000, "signed-before-crash");
        let expected = fixture
            .ledger
            .create_unsigned_ordinary_candidate(
                request.public_provenance(),
                request.timestamp_millis(),
                fixture.signing_key.verifying_key().to_bytes(),
            )
            .expect("preflight expected candidate")
            .sign(&fixture.signing_key)
            .expect("sign expected candidate");
        fixture.coordinator.inject_crash_after_signing();
        assert!(matches!(
            fixture.coordinator.submit(
                request,
                &mut fixture.ledger,
                &fixture.schedule,
                fixture.node_id,
                &fixture.signing_key,
                1_048_576,
            ),
            Err(PoAError::SigningFenceUncertain(_))
        ));
        assert_eq!(fixture.coordinator.signing_invocations(), 1);
        assert_eq!(
            fixture.ledger.journal_frame_count().expect("frame count"),
            0
        );

        drop(fixture.coordinator);
        drop(fixture.ledger);
        let mut ledger = Ledger::open_in_dir(
            fixture._data_dir.path(),
            test_ledger_profile("issue5.unit.profile"),
            activated_test_package(),
        )
        .expect("reopen test ledger");
        let mut coordinator =
            ProposalCoordinatorState::open_validator(&fixture.fence_path, fixture.ledger_binding)
                .expect("reopen fenced coordinator");
        coordinator
            .recover(
                &mut ledger,
                &fixture.schedule,
                fixture.node_id,
                &fixture.signing_key,
            )
            .expect("recover deterministically signed proposal");

        let envelope = &ledger.committed_envelopes()[0];
        assert_eq!(envelope.proposal_digest, expected.proposal_digest);
        assert_eq!(envelope.proposer_signature, expected.proposer_signature);
        assert_eq!(ledger.journal_frame_count().expect("frame count"), 1);
    }

    #[test]
    fn crash_after_ledger_commit_retires_only_the_exact_replayed_fence() {
        let mut fixture = fixture(87);
        let request = request(&fixture.ledger, 87_000, "committed-before-retirement");
        let expected_digest = fixture
            .ledger
            .create_unsigned_ordinary_candidate(
                request.public_provenance(),
                request.timestamp_millis(),
                fixture.signing_key.verifying_key().to_bytes(),
            )
            .expect("preflight expected candidate")
            .proposal_digest;
        fixture.coordinator.inject_crash_before_retirement();
        assert!(matches!(
            fixture.coordinator.submit(
                request,
                &mut fixture.ledger,
                &fixture.schedule,
                fixture.node_id,
                &fixture.signing_key,
                1_048_576,
            ),
            Err(PoAError::SigningFenceUncertain(_))
        ));
        assert_eq!(fixture.coordinator.signing_invocations(), 1);
        assert_eq!(
            fixture.ledger.journal_frame_count().expect("frame count"),
            1
        );

        drop(fixture.coordinator);
        drop(fixture.ledger);
        let mut ledger = Ledger::open_in_dir(
            fixture._data_dir.path(),
            test_ledger_profile("issue5.unit.profile"),
            activated_test_package(),
        )
        .expect("reopen committed ledger");
        let mut coordinator =
            ProposalCoordinatorState::open_validator(&fixture.fence_path, fixture.ledger_binding)
                .expect("reopen unretired fence");
        coordinator
            .recover(
                &mut ledger,
                &fixture.schedule,
                fixture.node_id,
                &fixture.signing_key,
            )
            .expect("retire matching committed fence");

        assert_eq!(coordinator.signing_invocations(), 0);
        assert_eq!(ledger.journal_frame_count().expect("frame count"), 1);
        assert_eq!(
            ledger.committed_envelopes()[0].proposal_digest,
            expected_digest
        );
        assert!(coordinator.selected[&0].retired_envelope_hash.is_some());
    }

    #[test]
    fn a_distinct_same_turn_journal_envelope_is_recovery_equivocation() {
        let mut fixture = fixture(88);
        let fenced_request = request(&fixture.ledger, 88_000, "fenced");
        fixture.coordinator.inject_crash_after_fence();
        let _ = fixture.coordinator.submit(
            fenced_request,
            &mut fixture.ledger,
            &fixture.schedule,
            fixture.node_id,
            &fixture.signing_key,
            1_048_576,
        );
        let conflicting = fixture
            .ledger
            .create_ordinary_candidate(
                b"<urn:issue5:unit> <urn:issue5:value> \"divergent\" .",
                88_001,
                &fixture.signing_key,
            )
            .expect("build distinct same-turn candidate");
        assert!(fixture
            .ledger
            .proposal_adapter()
            .submit(conflicting)
            .expect("simulate divergent committed envelope")
            .is_committed());

        drop(fixture.coordinator);
        drop(fixture.ledger);
        let mut ledger = Ledger::open_in_dir(
            fixture._data_dir.path(),
            test_ledger_profile("issue5.unit.profile"),
            activated_test_package(),
        )
        .expect("reopen divergent ledger");
        let mut coordinator =
            ProposalCoordinatorState::open_validator(&fixture.fence_path, fixture.ledger_binding)
                .expect("reopen fenced coordinator");
        assert!(matches!(
            coordinator.recover(
                &mut ledger,
                &fixture.schedule,
                fixture.node_id,
                &fixture.signing_key,
            ),
            Err(PoAError::Equivocation { height: 0, .. })
        ));
        assert_eq!(ledger.journal_frame_count().expect("frame count"), 1);
    }

    #[test]
    fn fence_ownership_and_recovery_state_fail_closed() {
        let missing = fixture(83);
        assert!(matches!(
            ProposalCoordinatorState::open_validator(&missing.fence_path, missing.ledger_binding,),
            Err(PoAError::SigningFenceIo(_))
        ));
        drop(missing.coordinator);
        fs::remove_file(&missing.fence_path)
            .expect("remove fence to simulate missing recovery state");
        assert!(matches!(
            ProposalCoordinatorState::open_validator(&missing.fence_path, missing.ledger_binding,),
            Err(PoAError::CorruptSigningFence(_))
        ));

        let corrupt_anchor = fixture(94);
        drop(corrupt_anchor.coordinator);
        let anchor_path = signing_fence_anchor_path(&corrupt_anchor.fence_path);
        OpenOptions::new()
            .write(true)
            .open(&anchor_path)
            .expect("open anchor for corruption")
            .set_len(1)
            .expect("corrupt fence anchor");
        assert!(matches!(
            ProposalCoordinatorState::open_validator(
                &corrupt_anchor.fence_path,
                corrupt_anchor.ledger_binding,
            ),
            Err(PoAError::CorruptSigningFence(_))
        ));
    }

    #[test]
    fn truncated_and_conflicting_fence_records_fail_closed() {
        let mut truncated = fixture(84);
        let truncated_request = request(&truncated.ledger, 84_000, "truncated");
        truncated.coordinator.inject_crash_after_fence();
        let _ = truncated.coordinator.submit(
            truncated_request,
            &mut truncated.ledger,
            &truncated.schedule,
            truncated.node_id,
            &truncated.signing_key,
            1_048_576,
        );
        drop(truncated.coordinator);
        let fence = OpenOptions::new()
            .write(true)
            .open(&truncated.fence_path)
            .expect("open fence for truncation");
        let truncated_len = fence.metadata().expect("fence metadata").len() - 1;
        fence.set_len(truncated_len).expect("truncate fence frame");
        assert!(matches!(
            ProposalCoordinatorState::open_validator(
                &truncated.fence_path,
                truncated.ledger_binding,
            ),
            Err(PoAError::CorruptSigningFence(_))
        ));

        let mut conflicting = fixture(85);
        let turn = PoATurn::from_tip(conflicting.ledger.tip());
        let make_selected = |timestamp_millis, value: &str| {
            let request = request(&conflicting.ledger, timestamp_millis, value);
            let request_bytes = request.canonical_bytes().expect("canonical request");
            let candidate = conflicting
                .ledger
                .create_unsigned_ordinary_candidate(
                    request.public_provenance(),
                    request.timestamp_millis(),
                    conflicting.signing_key.verifying_key().to_bytes(),
                )
                .expect("preflight selected proposal");
            SelectedProposal {
                turn,
                validator_public_key: conflicting.signing_key.verifying_key().to_bytes(),
                request_digest: request_digest(&request_bytes),
                request_bytes,
                unsigned_body: candidate.proposal_bytes().expect("proposal body"),
                proposal_digest: candidate.proposal_digest,
                retired_envelope_hash: None,
                outcome: None,
            }
        };
        let first = make_selected(85_000, "first");
        let second = make_selected(85_001, "second");
        let fence = &mut conflicting.coordinator.signing_fence;
        fence
            .append_selected(&first)
            .expect("append first selection");
        fence
            .append_selected(&second)
            .expect("append conflicting bytes for recovery test");
        drop(conflicting.coordinator);
        assert!(matches!(
            ProposalCoordinatorState::open_validator(
                &conflicting.fence_path,
                conflicting.ledger_binding,
            ),
            Err(PoAError::Equivocation { height: 0, .. })
        ));
    }

    #[test]
    fn observed_signed_write_failure_stalls_and_ambiguous_fsync_recovers_exact_digest() {
        let mut failed_write = fixture(95);
        let failed_candidate = failed_write
            .ledger
            .create_ordinary_candidate(
                b"<urn:issue5:observed> <urn:issue5:value> \"write-failure\" .",
                95_000,
                &failed_write.signing_key,
            )
            .expect("build observed signed candidate");
        failed_write
            .coordinator
            .signing_fence
            .inject_next_write_failure();
        assert!(matches!(
            failed_write.coordinator.accept_signed(
                failed_candidate,
                &mut failed_write.ledger,
                &failed_write.schedule,
                1_048_576,
            ),
            Err(PoAError::SigningFenceUncertain(_))
        ));
        drop(failed_write.coordinator);
        assert!(matches!(
            ProposalCoordinatorState::open_validator(
                &failed_write.fence_path,
                failed_write.ledger_binding,
            ),
            Err(PoAError::SigningFenceUncertain(_))
        ));

        let mut ambiguous_fsync = fixture(96);
        let selected = ambiguous_fsync
            .ledger
            .create_ordinary_candidate(
                b"<urn:issue5:observed> <urn:issue5:value> \"fsync-failure\" .",
                96_000,
                &ambiguous_fsync.signing_key,
            )
            .expect("build ambiguously persisted signed candidate");
        let selected_digest = selected.proposal_digest;
        ambiguous_fsync
            .coordinator
            .signing_fence
            .inject_next_fsync_failure();
        assert!(matches!(
            ambiguous_fsync.coordinator.accept_signed(
                selected,
                &mut ambiguous_fsync.ledger,
                &ambiguous_fsync.schedule,
                1_048_576,
            ),
            Err(PoAError::SigningFenceUncertain(_))
        ));
        drop(ambiguous_fsync.coordinator);

        let mut recovered = ProposalCoordinatorState::open_validator(
            &ambiguous_fsync.fence_path,
            ambiguous_fsync.ledger_binding,
        )
        .expect("recover the complete observed-proposal record");
        assert_eq!(
            recovered.locked_proposal_digests.get(&0),
            Some(&selected_digest)
        );
        let conflicting = ambiguous_fsync
            .ledger
            .create_ordinary_candidate(
                b"<urn:issue5:observed> <urn:issue5:value> \"replacement\" .",
                96_001,
                &ambiguous_fsync.signing_key,
            )
            .expect("build conflicting signed candidate");
        assert!(matches!(
            recovered.accept_signed(
                conflicting,
                &mut ambiguous_fsync.ledger,
                &ambiguous_fsync.schedule,
                1_048_576,
            ),
            Err(PoAError::Equivocation { height: 0, .. })
        ));
        assert_eq!(
            ambiguous_fsync
                .ledger
                .journal_frame_count()
                .expect("no committed envelope"),
            0
        );
    }

    #[test]
    fn durable_observation_is_checked_against_replayed_committed_history() {
        let mut fixture = fixture(97);
        let wrong_profile_dir = tempdir().expect("wrong-profile builder directory");
        let wrong_profile = test_ledger_profile("issue5.wrong-profile");
        let wrong_profile_ledger = Ledger::open_in_dir(
            wrong_profile_dir.path(),
            wrong_profile,
            activated_test_package(),
        )
        .expect("open wrong-profile builder");
        let rejected = wrong_profile_ledger
            .create_ordinary_candidate(
                b"<urn:issue5:observed> <urn:issue5:value> \"rejected\" .",
                97_000,
                &fixture.signing_key,
            )
            .expect("build deterministically rejected signed candidate");
        assert!(matches!(
            fixture.coordinator.accept_signed(
                rejected,
                &mut fixture.ledger,
                &fixture.schedule,
                1_048_576,
            ),
            Err(PoAError::FinalAdmissionRejected { height: 0, .. })
        ));

        let conflicting = fixture
            .ledger
            .create_ordinary_candidate(
                b"<urn:issue5:observed> <urn:issue5:value> \"committed-conflict\" .",
                97_001,
                &fixture.signing_key,
            )
            .expect("build conflicting committed candidate");
        assert!(fixture
            .ledger
            .proposal_adapter()
            .submit(conflicting)
            .expect("simulate divergent local commit")
            .is_committed());
        assert!(matches!(
            fixture
                .coordinator
                .validate_committed_locks(&fixture.ledger),
            Err(PoAError::Equivocation { height: 0, .. })
        ));
    }
}
