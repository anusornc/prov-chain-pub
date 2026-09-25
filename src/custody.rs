//! Client-only participant key custody and protected-object construction.
//!
//! The ledger and node paths never depend on this module. It owns private participant
//! material, commits encrypted whole snapshots before disclosing prepared public bytes,
//! and emits only canonical [`crate::privacy::PrivacyControlTransition`] values.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{DirBuilderExt, FileExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use argon2::{Algorithm, Argon2, Block, Params, Version};
use chacha20poly1305_v11::aead::{Aead, KeyInit, Payload};
use chacha20poly1305_v11::ChaCha20Poly1305;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use hpke::aead::ChaCha20Poly1305 as HpkeChaCha20Poly1305;
use hpke::kdf::HkdfSha256;
use hpke::kem::DhP256HkdfSha256;
use hpke::{
    setup_receiver, setup_sender_with_rng, Deserializable, Kem as KemTrait, OpModeR, OpModeS,
    Serializable,
};
use p256::ecdsa::{
    signature::Signer as _, Signature as P256Signature, SigningKey as P256SigningKey,
};
use rand_core_v10::{Infallible, TryCryptoRng, TryRng};
use rustix::fs::{
    open as rustix_open, openat2, renameat_with, unlinkat, AtFlags, Dir, Mode, OFlags, RenameFlags,
    ResolveFlags,
};
use sha2::{Digest, Sha256};
use sha2_v11::{Digest as DigestV11, Sha256 as Sha256V11};
use thiserror::Error;
use uuid::Uuid;
use zeroize::{Zeroize, Zeroizing};

use crate::ledger::{NetworkConvergedPrivacyPrefix, VerifiedPrivacyLedgerPrefix};
use crate::privacy::{
    encrypted_payload_commitment, grant_envelope, grant_envelope_header_and_context,
    owner_envelope, owner_envelope_header_and_info, protected_domain_hash, protected_encode_record,
    protected_encode_record_refs, protected_object_context, protected_payload,
    EffectivePrivacyState, KeyLifecycleStatus, LivePrivacyReleasePath, LivePrivacyReleaseResponse,
    ParticipantKeyPurpose, ParticipantKeyReference, PrivacyAdmissionAnchor,
    PrivacyControlTransition, PrivacyError, PrivacyGrantStatus, PrivacyKeyUse,
    PrivacyLifecycleProfile, PrivacyPermission, PrivacyReleaseEnvelope, ProtectedObjectRecord,
    UnsignedPrivacyTransition, MAX_PROTECTED_CONTENT_BYTES, PROTECTED_DATA_SUITE_V1,
};

const KEYSTORE_MAGIC: &[u8; 4] = b"PKV1";
const FILE_MAGIC: &[u8; 4] = b"PKS1";
const KEYSTORE_SUITE: &str = "ParticipantKeystoreSuiteV1";
const KEYSTORE_CODEC: &str = "CanonicalParticipantKeystoreEncodingV1";
const LIVE_NAME: &str = "participant-keystore.pks1";
const LOCK_NAME: &str = ".participant-keystore.lock";
const TEMP_PREFIX: &str = ".participant-keystore.tmp.";
const BACKUP_TEMP_PREFIX: &str = ".participant-keystore-backup.tmp.";
const BACKUP_PREFIX: &str = "participant-keystore-backup.";
const RESTORE_DIRECTORY: &str = "restore-quarantine";
const RESTORE_TEMP_PREFIX: &str = ".participant-keystore-restore-source.tmp.";
const RESTORE_PREFIX: &str = "participant-keystore-restore-source.";
const HEADER_TAG: u8 = 0x01;
const FILE_KEY_INFO_TAG: u8 = 0x02;
const SNAPSHOT_TAG: u8 = 0x10;
const ENTRY_LIST_TAG: u8 = 0x11;
const PRIVATE_MATERIAL_TAG: u8 = 0x20;
const BOUND_KEY_TAG: u8 = 0x21;
const TOMBSTONE_TAG: u8 = 0x22;
const BINDING_REQUEST_TAG: u8 = 0x23;
const PREPARED_GOVERNANCE_TAG: u8 = 0x24;
const PREPARED_NODE_TAG: u8 = 0x25;
const HEADER_BYTES: usize = 269;
const AAD_BYTES: usize = 277;
const MAX_SNAPSHOT_BYTES: usize = 262_144;
const MAX_FILE_BYTES: usize = 262_437;
const ARGON_BLOCKS: usize = 65_536;
const HKDF_EXTRACT_SALT: &[u8] = b"provchain/participant-keystore/hkdf-extract-salt/v1";
const PAYLOAD_EXTRACT_SALT_DOMAIN: &str = "provchain/protected-data/dek-extract-salt/v1";
const RESOLVE_WITHIN_DIRECTORY: ResolveFlags = ResolveFlags::BENEATH
    .union(ResolveFlags::NO_SYMLINKS)
    .union(ResolveFlags::NO_MAGICLINKS)
    .union(ResolveFlags::NO_XDEV);

/// One exact HPKE ephemeral-key seed obtained through the fallible system RNG.
///
/// `hpke` 0.14 accepts only an infallible `CryptoRng`, although its pinned P-256
/// KEM consumes exactly one 32-byte IKM buffer. This adapter acquires those bytes
/// fallibly before HPKE runs and rejects the result unless that exact contract was
/// observed. It never expands a deterministic seed or hides an OS entropy failure.
struct OneShotHpkeRng {
    entropy: Zeroizing<[u8; 32]>,
    consumed: bool,
    invalid_use: bool,
}

impl OneShotHpkeRng {
    fn from_system() -> Result<Self, CustodyError> {
        let mut entropy = Zeroizing::new([0_u8; 32]);
        random_fill(entropy.as_mut())?;
        Ok(Self {
            entropy,
            consumed: false,
            invalid_use: false,
        })
    }

    fn exactly_consumed(&self) -> bool {
        self.consumed && !self.invalid_use
    }
}

impl TryRng for OneShotHpkeRng {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        self.invalid_use = true;
        Ok(0)
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        self.invalid_use = true;
        Ok(0)
    }

    fn try_fill_bytes(&mut self, destination: &mut [u8]) -> Result<(), Self::Error> {
        if self.consumed || destination.len() != self.entropy.len() {
            self.invalid_use = true;
            destination.fill(0);
            return Ok(());
        }
        destination.copy_from_slice(self.entropy.as_slice());
        self.entropy.zeroize();
        self.consumed = true;
        Ok(())
    }
}

impl TryCryptoRng for OneShotHpkeRng {}

/// Exact printable-ASCII passphrase accepted only by the participant client.
///
/// This type deliberately implements neither `Clone`, `Debug`, `Display`, nor serialization.
pub struct CustodyPassphrase(Zeroizing<Vec<u8>>);

impl CustodyPassphrase {
    /// Validate and retain exact bytes without trimming or normalization.
    pub fn new(bytes: &[u8]) -> Result<Self, CustodyError> {
        if !(15..=128).contains(&bytes.len())
            || !bytes.iter().all(|byte| (0x20..=0x7e).contains(byte))
        {
            return Err(CustodyError::InvalidPassphrase);
        }
        Ok(Self(Zeroizing::new(bytes.to_vec())))
    }

    fn expose(&self) -> &[u8] {
        self.0.as_slice()
    }
}

/// Public governance request returned only after its secret and exact request are durable.
#[derive(Clone, PartialEq, Eq)]
pub struct PreparedGovernanceRequest {
    principal: Uuid,
    authorization_public_key: [u8; 32],
    authorization_digest: [u8; 32],
    canonical_bytes: Vec<u8>,
}

impl PreparedGovernanceRequest {
    /// Newly generated non-nil Participant Principal.
    pub fn principal(&self) -> Uuid {
        self.principal
    }

    /// Public Ed25519 key generated and retained only with encrypted private custody.
    pub fn authorization_public_key(&self) -> &[u8; 32] {
        &self.authorization_public_key
    }

    /// Exact raw digest the bootstrap governance key must sign.
    pub fn authorization_digest(&self) -> [u8; 32] {
        self.authorization_digest
    }

    /// Exact durable request bytes suitable for an external governance signer.
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

/// Public node submission returned only from a committed prepared snapshot.
#[derive(Clone, PartialEq, Eq)]
pub struct PreparedNodeSubmission {
    transition: PrivacyControlTransition,
    canonical_bytes: Vec<u8>,
}

impl PreparedNodeSubmission {
    /// Complete transition accepted by public Final Admission.
    pub fn transition(&self) -> &PrivacyControlTransition {
        &self.transition
    }

    /// Exact retry bytes retained in the encrypted snapshot.
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

/// Reconciliation result at one explicitly supplied ledger-derived state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustodyReconciliation {
    gaps: Vec<ParticipantKeyReference>,
    changed_snapshot: bool,
}

impl CustodyReconciliation {
    /// Ledger-recorded keys absent from this store.
    pub fn gaps(&self) -> &[ParticipantKeyReference] {
        &self.gaps
    }

    /// Whether reconciliation committed a successor snapshot.
    pub fn changed_snapshot(&self) -> bool {
        self.changed_snapshot
    }
}

/// Result of the mandatory startup recovery gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustodyRecovery {
    /// No live store, durable candidate, or pending restore intent exists.
    AbsentReady,
    /// One authenticated live snapshot is ready at the supplied ledger-derived state.
    Ready(CustodyReconciliation),
}

/// Complete public identity of one encrypted custody snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustodySnapshotReference {
    file_id: [u8; 16],
    generation: u64,
    digest: [u8; 32],
    network_id: String,
    principal: Uuid,
}

impl CustodySnapshotReference {
    /// Construct an expected reference supplied independently of untrusted backup bytes.
    pub fn new(
        file_id: [u8; 16],
        generation: u64,
        digest: [u8; 32],
        network_id: impl Into<String>,
        principal: Uuid,
    ) -> Result<Self, CustodyError> {
        let network_id = network_id.into();
        validate_network_id(&network_id)?;
        if file_id == [0; 16] || generation == 0 || principal.is_nil() {
            return Err(CustodyError::InvalidSnapshot);
        }
        Ok(Self {
            file_id,
            generation,
            digest,
            network_id,
            principal,
        })
    }

    /// Stable logical store identifier.
    pub fn file_id(&self) -> [u8; 16] {
        self.file_id
    }

    /// Monotonic whole-snapshot generation.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// SHA-256 of the exact encrypted file bytes.
    pub fn digest(&self) -> [u8; 32] {
        self.digest
    }

    /// Network identity encrypted inside the authenticated snapshot.
    pub fn network_id(&self) -> &str {
        &self.network_id
    }

    /// Participant identity encrypted inside the authenticated snapshot.
    pub fn principal(&self) -> Uuid {
        self.principal
    }
}

/// Durable byte-identical backup selected from one committed live snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustodyBackup {
    path: PathBuf,
    reference: CustodySnapshotReference,
}

impl CustodyBackup {
    /// Canonical immutable backup pathname.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Exact independently recordable snapshot reference.
    pub fn reference(&self) -> &CustodySnapshotReference {
        &self.reference
    }
}

mod commit_failpoint {
    /// Deterministic mutation boundary used by crash-recovery acceptance tests.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CustodyCommitFailpoint {
        /// Definite failure after a durable candidate but before publication.
        BeforePublish,
        /// Simulated process crash after candidate durability and before target CAS.
        CrashAfterCandidateDurable,
        /// Indeterminate failure after rename and before custody-directory `fsync`.
        AfterPublishBeforeDirectorySync,
        /// Committed failure after directory `fsync` but before authenticated reopen.
        AfterCommitBeforeReopen,
    }
}

#[cfg(not(feature = "privacy-conformance"))]
use commit_failpoint::CustodyCommitFailpoint;
#[cfg(feature = "privacy-conformance")]
#[doc(hidden)]
pub use commit_failpoint::CustodyCommitFailpoint;

/// Participant-local custody handle. It stores only a path and transient state, never a secret.
pub struct ParticipantCustody {
    directory: PathBuf,
    next_failpoint: Option<CustodyCommitFailpoint>,
    blocked: bool,
}

impl ParticipantCustody {
    /// Address one participant-owned custody directory.
    pub fn new(directory: impl AsRef<Path>) -> Self {
        Self {
            directory: directory.as_ref().to_path_buf(),
            next_failpoint: None,
            blocked: false,
        }
    }

    /// Exact live encrypted snapshot path. This contains no plaintext material.
    pub fn live_path(&self) -> PathBuf {
        self.directory.join(LIVE_NAME)
    }

    /// Inject one deterministic commit-boundary failure for conformance tests.
    #[cfg(feature = "privacy-conformance")]
    #[doc(hidden)]
    pub fn inject_next_commit_failpoint(&mut self, failpoint: CustodyCommitFailpoint) {
        self.next_failpoint = Some(failpoint);
    }

    /// Generate a principal and initial Ed25519 key and commit Prepared Governance first.
    pub fn initialize(
        &mut self,
        prefix: &VerifiedPrivacyLedgerPrefix<'_>,
        passphrase: &CustodyPassphrase,
    ) -> Result<PreparedGovernanceRequest, CustodyError> {
        self.ensure_not_blocked()?;
        let profile = prefix.profile();
        let anchor = prefix.next_anchor().clone();
        let network_id = anchor.network_id().to_string();
        profile.validate()?;
        validate_network_id(&network_id)?;
        validate_anchor_identity(&network_id, profile, &anchor)?;
        prepare_directory(&self.directory)?;
        let lock = acquire_lock(&self.directory)?;
        if named_file_exists(&lock.directory, OsStr::new(LIVE_NAME))? {
            return Err(CustodyError::RecoveryConflict);
        }
        if !list_candidates_in(&self.directory, &lock.directory)?.is_empty() {
            return Err(CustodyError::RecoveryConflict);
        }
        if has_restore_intent(&self.directory, &lock.directory)? {
            return Err(CustodyError::RecoveryConflict);
        }

        let principal = random_uuid_v4()?;
        let mut seed = Zeroizing::new([0_u8; 32]);
        random_fill(seed.as_mut())?;
        let signing_key = SigningKey::from_bytes(&seed);
        let public_key = signing_key.verifying_key().to_bytes();
        let unsigned =
            UnsignedPrivacyTransition::register_principal(profile, anchor, principal, public_key)?;
        let possession = signing_key
            .sign(
                &unsigned
                    .possession_digest()
                    .ok_or(CustodyError::InvalidPreparedState)?,
            )
            .to_bytes();
        let material = KeyMaterial::authorization(principal, 1, seed)?;
        let request = PreparedRequest::new(
            0x01,
            profile.bootstrap_reference_bytes(),
            unsigned,
            0x01,
            possession,
        )?;
        request.validate_material(&material)?;
        let public_request = PreparedGovernanceRequest {
            principal,
            authorization_public_key: public_key,
            authorization_digest: request.unsigned.authorization_digest(),
            canonical_bytes: request.encode(),
        };
        let mut entries = BTreeMap::new();
        entries.insert(
            material.sort_key(),
            CustodyEntry::PreparedGovernance { material, request },
        );
        let snapshot = Snapshot {
            network_id,
            principal,
            entries,
        };
        self.commit_snapshot(
            None,
            snapshot,
            passphrase,
            SnapshotPublication::Normal,
            &lock,
        )?;
        Ok(public_request)
    }

    /// Verify a governance signature, commit the exact node submission, then disclose it.
    pub fn accept_governance_signature(
        &mut self,
        prefix: &VerifiedPrivacyLedgerPrefix<'_>,
        passphrase: &CustodyPassphrase,
        governance_signature: [u8; 64],
    ) -> Result<PreparedNodeSubmission, CustodyError> {
        self.ensure_not_blocked()?;
        let profile = prefix.profile();
        let lock = acquire_lock(&self.directory)?;
        ensure_ready_for_normal_operation(&self.directory, &lock.directory)?;
        let mut opened = open_live_store_in(&lock.directory, passphrase)?;
        validate_snapshot_prefix(&opened.snapshot, prefix)?;
        if opened.snapshot.entries.len() != 1 {
            return Err(CustodyError::InvalidPreparedState);
        }
        let (key, entry) = opened
            .snapshot
            .entries
            .pop_first()
            .ok_or(CustodyError::InvalidPreparedState)?;
        let CustodyEntry::PreparedGovernance { material, request } = entry else {
            return Err(CustodyError::InvalidPreparedState);
        };
        validate_prepared_current(prefix, &request, None)?;
        if !request.unsigned.is_registration()
            || request.authorizer_reference != profile.bootstrap_reference_bytes()
        {
            return Err(CustodyError::InvalidPreparedState);
        }
        let verifier = VerifyingKey::from_bytes(&profile.bootstrap_public_key())
            .map_err(|_| CustodyError::InvalidPreparedState)?;
        verifier
            .verify_strict(
                &request.unsigned.authorization_digest(),
                &ed25519_dalek::Signature::from_bytes(&governance_signature),
            )
            .map_err(|_| CustodyError::InvalidPreparedState)?;
        let transition = request
            .unsigned
            .clone()
            .complete_registration(governance_signature, request.possession_proof)?;
        let canonical_bytes = transition.canonical_bytes();
        let submission = PreparedNodeSubmission {
            transition: transition.clone(),
            canonical_bytes: canonical_bytes.clone(),
        };
        opened.snapshot.entries.insert(
            key,
            CustodyEntry::PreparedNode {
                material,
                request: Box::new(request),
                transition,
            },
        );
        let parent = opened.parent_metadata();
        self.commit_snapshot(
            Some(parent),
            opened.snapshot,
            passphrase,
            SnapshotPublication::Normal,
            &lock,
        )?;
        Ok(submission)
    }

    /// Return only exact bytes already stored in a committed Prepared Node snapshot.
    pub fn pending_submission(
        &mut self,
        prefix: &VerifiedPrivacyLedgerPrefix<'_>,
        passphrase: &CustodyPassphrase,
    ) -> Result<Option<PreparedNodeSubmission>, CustodyError> {
        self.ensure_not_blocked()?;
        let lock = acquire_lock(&self.directory)?;
        ensure_ready_for_normal_operation(&self.directory, &lock.directory)?;
        let opened = open_live_store_in(&lock.directory, passphrase)?;
        validate_snapshot_prefix(&opened.snapshot, prefix)?;
        let mut pending = None;
        for entry in opened.snapshot.entries.values() {
            if let CustodyEntry::PreparedNode {
                request,
                transition,
                ..
            } = entry
            {
                if pending.is_some() {
                    return Err(CustodyError::RecoveryConflict);
                }
                validate_prepared_current(prefix, request, Some(transition))?;
                pending = Some(PreparedNodeSubmission {
                    transition: transition.clone(),
                    canonical_bytes: transition.canonical_bytes(),
                });
            }
        }
        Ok(pending)
    }

    /// Return only an exact governance request already stored in a committed snapshot.
    pub fn pending_governance_request(
        &mut self,
        prefix: &VerifiedPrivacyLedgerPrefix<'_>,
        passphrase: &CustodyPassphrase,
    ) -> Result<Option<PreparedGovernanceRequest>, CustodyError> {
        self.ensure_not_blocked()?;
        let profile = prefix.profile();
        let lock = acquire_lock(&self.directory)?;
        ensure_ready_for_normal_operation(&self.directory, &lock.directory)?;
        let opened = open_live_store_in(&lock.directory, passphrase)?;
        validate_snapshot_prefix(&opened.snapshot, prefix)?;
        let mut pending = None;
        for entry in opened.snapshot.entries.values() {
            if let CustodyEntry::PreparedGovernance { material, request } = entry {
                if pending.is_some()
                    || !request.unsigned.is_registration()
                    || request.authorizer_reference != profile.bootstrap_reference_bytes()
                {
                    return Err(CustodyError::RecoveryConflict);
                }
                validate_prepared_current(prefix, request, None)?;
                let public_key = material
                    .public_key
                    .as_slice()
                    .try_into()
                    .map_err(|_| CustodyError::InvalidPreparedState)?;
                pending = Some(PreparedGovernanceRequest {
                    principal: opened.snapshot.principal,
                    authorization_public_key: public_key,
                    authorization_digest: request.unsigned.authorization_digest(),
                    canonical_bytes: request.encode(),
                });
            }
        }
        Ok(pending)
    }

    /// Resolve live and temporary snapshot artifacts before any other custody operation.
    pub fn recover(
        &mut self,
        prefix: &VerifiedPrivacyLedgerPrefix<'_>,
        passphrase: &CustodyPassphrase,
    ) -> Result<CustodyRecovery, CustodyError> {
        self.recover_inner(prefix, false, passphrase)
    }

    /// Resume a quarantined restore only with receipt-backed current ledger truth.
    pub fn recover_restore(
        &mut self,
        prefix: &NetworkConvergedPrivacyPrefix<'_>,
        passphrase: &CustodyPassphrase,
    ) -> Result<CustodyRecovery, CustodyError> {
        self.recover_inner(prefix.verified(), true, passphrase)
    }

    fn recover_inner(
        &mut self,
        prefix: &VerifiedPrivacyLedgerPrefix<'_>,
        restore_authorized: bool,
        passphrase: &CustodyPassphrase,
    ) -> Result<CustodyRecovery, CustodyError> {
        let profile = prefix.profile();
        let state = prefix.state();
        profile.validate()?;
        prepare_directory(&self.directory)?;
        let lock = acquire_lock(&self.directory)?;
        self.next_failpoint = None;

        let live_path = self.live_path();
        let candidates = list_candidates_in(&self.directory, &lock.directory)?;
        if !named_file_exists(&lock.directory, OsStr::new(LIVE_NAME))? {
            let quarantine = self.directory.join(RESTORE_DIRECTORY);
            let sources = if quarantine.try_exists().map_err(io_error)? {
                prepare_existing_directory(&quarantine)?;
                cleanup_restore_temps(&quarantine)?;
                list_restore_sources(&quarantine)?
            } else {
                Vec::new()
            };
            if sources.len() > 1 {
                return Err(CustodyError::RecoveryConflict);
            }
            if let Some(source_path) = sources.first() {
                if !restore_authorized {
                    return Err(CustodyError::NetworkConvergenceRequired);
                }
                validate_file_in_directory(&quarantine, source_path)?;
                let source =
                    open_store(source_path, passphrase).map_err(redact_recovery_open_error)?;
                validate_snapshot_prefix(&source.snapshot, prefix)
                    .map_err(|_| CustodyError::RecoveryConflict)?;
                let source_reference = source.snapshot_reference();
                if source_path.file_name().and_then(|name| name.to_str())
                    != Some(restore_source_name(&source_reference).as_str())
                {
                    return Err(CustodyError::RecoveryConflict);
                }
                let parent = source.parent_metadata();
                let reconciled = reconcile_restore_entries(source.snapshot, prefix)?;
                if reconciled.snapshot.entries.is_empty() {
                    return Err(CustodyError::RestoreNoInstallableState);
                }
                if candidates.is_empty() {
                    self.commit_snapshot(
                        Some(parent),
                        reconciled.snapshot,
                        passphrase,
                        SnapshotPublication::Restore,
                        &lock,
                    )?;
                } else {
                    if candidates.len() != 1 {
                        return Err(CustodyError::RecoveryConflict);
                    }
                    validate_candidate_in(&lock.directory, &candidates[0])?;
                    let candidate = open_store_in(
                        &lock.directory,
                        candidates[0]
                            .file_name()
                            .ok_or(CustodyError::RecoveryConflict)?,
                        passphrase,
                    )
                    .map_err(redact_recovery_open_error)?;
                    validate_snapshot_prefix(&candidate.snapshot, prefix)
                        .map_err(|_| CustodyError::RecoveryConflict)?;
                    let expected_snapshot = reconciled.snapshot.encode()?;
                    let actual_snapshot = candidate.snapshot.encode()?;
                    if candidate.header.file_id != parent.reference.file_id
                        || candidate.header.generation
                            != parent
                                .reference
                                .generation
                                .checked_add(1)
                                .ok_or(CustodyError::RecoveryConflict)?
                        || candidate.header.previous != digest_reference(parent.reference.digest)
                        || actual_snapshot.as_slice() != expected_snapshot.as_slice()
                    {
                        return Err(CustodyError::RecoveryConflict);
                    }
                    sync_named_file(
                        &lock.directory,
                        candidates[0]
                            .file_name()
                            .ok_or(CustodyError::RecoveryConflict)?,
                    )?;
                    self.publish_recovery_candidate(
                        &candidates[0],
                        &live_path,
                        &ExpectedCustodyTarget::Absent,
                        passphrase,
                        true,
                        &lock,
                    )?;
                }
            } else if candidates.is_empty() {
                lock.revalidate(&self.directory)?;
                lock.sync_directory()?;
                self.blocked = false;
                return Ok(CustodyRecovery::AbsentReady);
            } else {
                if candidates.len() != 1 {
                    return Err(CustodyError::RecoveryConflict);
                }
                validate_candidate_in(&lock.directory, &candidates[0])?;
                let candidate = open_store_in(
                    &lock.directory,
                    candidates[0]
                        .file_name()
                        .ok_or(CustodyError::RecoveryConflict)?,
                    passphrase,
                )
                .map_err(redact_recovery_open_error)?;
                validate_snapshot_prefix(&candidate.snapshot, prefix)
                    .map_err(|_| CustodyError::RecoveryConflict)?;
                if candidate.header.generation != 1 || candidate.header.previous != [0; 33] {
                    return Err(CustodyError::RecoveryConflict);
                }
                sync_named_file(
                    &lock.directory,
                    candidates[0]
                        .file_name()
                        .ok_or(CustodyError::RecoveryConflict)?,
                )?;
                self.publish_recovery_candidate(
                    &candidates[0],
                    &live_path,
                    &ExpectedCustodyTarget::Absent,
                    passphrase,
                    true,
                    &lock,
                )?;
            }
        } else {
            validate_candidate_in(&lock.directory, &live_path)?;
            let live = open_store_in(&lock.directory, OsStr::new(LIVE_NAME), passphrase)
                .map_err(redact_recovery_open_error)?;
            validate_snapshot_prefix(&live.snapshot, prefix)
                .map_err(|_| CustodyError::RecoveryConflict)?;
            let mut removable = Vec::new();
            let mut eligible = Vec::new();
            for path in candidates {
                validate_candidate_in(&lock.directory, &path)?;
                match open_store_in(
                    &lock.directory,
                    path.file_name().ok_or(CustodyError::RecoveryConflict)?,
                    passphrase,
                ) {
                    Ok(candidate)
                        if candidate.header.file_id == live.header.file_id
                            && candidate.header.generation
                                == live
                                    .header
                                    .generation
                                    .checked_add(1)
                                    .ok_or(CustodyError::RecoveryConflict)?
                            && candidate.header.previous == digest_reference(live.digest)
                            && candidate.snapshot.network_id == live.snapshot.network_id
                            && candidate.snapshot.principal == live.snapshot.principal =>
                    {
                        eligible.push(path)
                    }
                    Ok(_) => return Err(CustodyError::RecoveryConflict),
                    Err(
                        CustodyError::InvalidSnapshot
                        | CustodyError::InvalidPreparedState
                        | CustodyError::ProtectedDataOpenFailed
                        | CustodyError::PublicProtocol,
                    ) => removable.push(path),
                    Err(error) => return Err(error),
                }
            }
            if eligible.len() > 1 {
                return Err(CustodyError::RecoveryConflict);
            }
            for path in removable {
                cleanup_candidate_in(
                    &lock.directory,
                    path.file_name().ok_or(CustodyError::RecoveryConflict)?,
                )?;
            }
            if let Some(path) = eligible.pop() {
                sync_named_file(
                    &lock.directory,
                    path.file_name().ok_or(CustodyError::RecoveryConflict)?,
                )?;
                let expected = ExpectedCustodyTarget::Present(live.snapshot_reference());
                self.publish_recovery_candidate(
                    &path, &live_path, &expected, passphrase, false, &lock,
                )?;
            } else {
                sync_named_file(&lock.directory, OsStr::new(LIVE_NAME))?;
                lock.revalidate(&self.directory)?;
                lock.sync_directory()?;
            }
        }

        let opened = open_store_in(&lock.directory, OsStr::new(LIVE_NAME), passphrase)
            .map_err(redact_recovery_open_error)?;
        validate_snapshot_prefix(&opened.snapshot, prefix)
            .map_err(|_| CustodyError::RecoveryConflict)?;
        let parent = opened.parent_metadata();
        let reconciled = reconcile_entries(opened.snapshot, state)?;
        let gaps = reconciled.gaps.clone();
        let changed = reconciled.changed;
        if changed {
            self.commit_snapshot(
                Some(parent),
                reconciled.snapshot,
                passphrase,
                SnapshotPublication::Normal,
                &lock,
            )?;
        }
        self.blocked = false;
        Ok(CustodyRecovery::Ready(CustodyReconciliation {
            gaps,
            changed_snapshot: changed,
        }))
    }

    /// Copy one authenticated committed snapshot byte-for-byte into an immutable backup.
    pub fn create_backup(
        &mut self,
        backup_directory: impl AsRef<Path>,
        prefix: &VerifiedPrivacyLedgerPrefix<'_>,
        passphrase: &CustodyPassphrase,
    ) -> Result<CustodyBackup, CustodyError> {
        self.ensure_not_blocked()?;
        let backup_directory = backup_directory.as_ref();
        prepare_directory(backup_directory)?;
        if fs::canonicalize(backup_directory).map_err(io_error)?
            == fs::canonicalize(&self.directory).map_err(io_error)?
        {
            return Err(CustodyError::UnsupportedStorageProfile);
        }
        let lock = acquire_lock(&self.directory)?;
        ensure_ready_for_normal_operation(&self.directory, &lock.directory)?;
        validate_candidate_in(&lock.directory, &self.live_path())?;
        let selected = open_live_store_in(&lock.directory, passphrase)?;
        validate_snapshot_prefix(&selected.snapshot, prefix)?;
        let reference = selected.snapshot_reference();
        let raw = selected.raw;
        if <[u8; 32]>::from(Sha256::digest(&raw)) != reference.digest {
            return Err(CustodyError::RecoveryConflict);
        }
        let final_name = backup_name(&reference);
        let path = commit_opaque_copy(backup_directory, BACKUP_TEMP_PREFIX, &final_name, &raw)?;
        Ok(CustodyBackup { path, reference })
    }

    /// Independently authenticate and reconcile an opaque backup without consulting a live store.
    pub fn verify_backup(
        backup_path: impl AsRef<Path>,
        expected: &CustodySnapshotReference,
        prefix: &NetworkConvergedPrivacyPrefix<'_>,
        passphrase: &CustodyPassphrase,
    ) -> Result<CustodyReconciliation, CustodyError> {
        let prefix = prefix.verified();
        let backup_path = backup_path.as_ref();
        let directory = backup_path
            .parent()
            .ok_or(CustodyError::UnsupportedStorageProfile)?;
        prepare_existing_directory(directory)?;
        validate_file_in_directory(directory, backup_path).map_err(redact_keystore_open_error)?;
        let opened = open_expected_store(backup_path, expected, prefix, passphrase)?;
        let reconciled = reconcile_restore_entries(opened.snapshot, prefix)?;
        Ok(CustodyReconciliation {
            gaps: reconciled.gaps,
            changed_snapshot: reconciled.changed,
        })
    }

    /// Restore one expected backup only through durable quarantine into an absent live target.
    pub fn restore_backup(
        &mut self,
        backup_path: impl AsRef<Path>,
        expected: &CustodySnapshotReference,
        prefix: &NetworkConvergedPrivacyPrefix<'_>,
        passphrase: &CustodyPassphrase,
    ) -> Result<CustodyReconciliation, CustodyError> {
        let prefix = prefix.verified();
        self.ensure_not_blocked()?;
        let profile = prefix.profile();
        profile.validate()?;
        prepare_directory(&self.directory)?;
        let lock = acquire_lock(&self.directory)?;
        if named_file_exists(&lock.directory, OsStr::new(LIVE_NAME))?
            || !list_candidates_in(&self.directory, &lock.directory)?.is_empty()
        {
            return Err(CustodyError::RecoveryConflict);
        }

        let backup_path = backup_path.as_ref();
        let backup_directory = backup_path
            .parent()
            .ok_or(CustodyError::UnsupportedStorageProfile)?;
        prepare_existing_directory(backup_directory)?;
        validate_file_in_directory(backup_directory, backup_path)
            .map_err(redact_keystore_open_error)?;
        let opened = open_expected_store(backup_path, expected, prefix, passphrase)?;
        let raw = opened.raw;

        let quarantine = self.directory.join(RESTORE_DIRECTORY);
        prepare_directory(&quarantine)?;
        let final_name = restore_source_name(expected);
        let sources = list_restore_sources(&quarantine)?;
        if sources.iter().any(|path| {
            path.file_name().and_then(|name| name.to_str()) != Some(final_name.as_str())
        }) {
            return Err(CustodyError::RecoveryConflict);
        }
        let source_path =
            match commit_opaque_copy(&quarantine, RESTORE_TEMP_PREFIX, &final_name, &raw) {
                Ok(path) => path,
                Err(CustodyError::CommitIndeterminate) => {
                    self.blocked = true;
                    return Err(CustodyError::CommitIndeterminate);
                }
                Err(error) => return Err(error),
            };
        let source = open_expected_store(&source_path, expected, prefix, passphrase)?;
        let parent = source.parent_metadata();
        let reconciled = reconcile_restore_entries(source.snapshot, prefix)?;
        if reconciled.snapshot.entries.is_empty() {
            return Err(CustodyError::RestoreNoInstallableState);
        }
        let gaps = reconciled.gaps.clone();
        self.commit_snapshot(
            Some(parent),
            reconciled.snapshot,
            passphrase,
            SnapshotPublication::Restore,
            &lock,
        )?;
        self.blocked = false;
        Ok(CustodyReconciliation {
            gaps,
            changed_snapshot: true,
        })
    }

    /// Generate and durably prepare a purpose-separated P-256 wrapping-key binding.
    pub fn prepare_wrapping_key(
        &mut self,
        prefix: &VerifiedPrivacyLedgerPrefix<'_>,
        passphrase: &CustodyPassphrase,
    ) -> Result<PreparedNodeSubmission, CustodyError> {
        self.ensure_not_blocked()?;
        let profile = prefix.profile();
        let state = prefix.state();
        let anchor = prefix.next_anchor().clone();
        let lock = acquire_lock(&self.directory)?;
        ensure_ready_for_normal_operation(&self.directory, &lock.directory)?;
        let mut opened = open_live_store_in(&lock.directory, passphrase)?;
        validate_snapshot_prefix(&opened.snapshot, prefix)?;
        let reconciliation = reconcile_entries(opened.snapshot, state)?;
        if !reconciliation.gaps.is_empty() || reconciliation.has_prepared {
            return Err(CustodyError::CustodyGap);
        }
        opened.snapshot = reconciliation.snapshot;
        validate_anchor_identity(&opened.snapshot.network_id, profile, &anchor)?;
        let authorization_reference = state
            .active_key(
                opened.snapshot.principal,
                ParticipantKeyPurpose::PrivacyAuthorization,
            )
            .cloned()
            .ok_or(CustodyError::CustodyGap)?;
        let authorization_material = opened
            .snapshot
            .bound_material(&authorization_reference)
            .ok_or(CustodyError::CustodyGap)?;
        let authorization_key = authorization_material.authorization_signing_key()?;

        let scalar = generate_p256_scalar()?;
        let wrapping_material = KeyMaterial::wrapping(opened.snapshot.principal, 1, scalar)?;
        let unsigned = UnsignedPrivacyTransition::bind_wrapping_key(
            profile,
            anchor,
            opened.snapshot.principal,
            1,
            wrapping_material.public_key_65()?,
            authorization_reference.clone(),
        )?;
        let possession_digest = unsigned
            .possession_digest()
            .ok_or(CustodyError::InvalidPreparedState)?;
        let p256_key = wrapping_material.wrapping_signing_key()?;
        let possession: P256Signature = p256_key.sign(&possession_digest);
        let possession = possession.normalize_s().to_bytes().into();
        let request = PreparedRequest::new(
            0x02,
            authorization_reference.canonical_bytes(),
            unsigned,
            0x02,
            possession,
        )?;
        request.validate_material(&wrapping_material)?;
        let authorization_signature = authorization_key
            .sign(&request.unsigned.authorization_digest())
            .to_bytes();
        let transition = request
            .unsigned
            .clone()
            .complete_binding(authorization_signature, possession)?;
        let submission = PreparedNodeSubmission {
            canonical_bytes: transition.canonical_bytes(),
            transition: transition.clone(),
        };
        opened.snapshot.entries.insert(
            wrapping_material.sort_key(),
            CustodyEntry::PreparedNode {
                material: wrapping_material,
                request: Box::new(request),
                transition,
            },
        );
        let parent = opened.parent_metadata();
        self.commit_snapshot(
            Some(parent),
            opened.snapshot,
            passphrase,
            SnapshotPublication::Normal,
            &lock,
        )?;
        Ok(submission)
    }

    /// Reconcile every local key against one supplied journal-derived privacy state.
    pub fn reconcile(
        &mut self,
        prefix: &VerifiedPrivacyLedgerPrefix<'_>,
        passphrase: &CustodyPassphrase,
    ) -> Result<CustodyReconciliation, CustodyError> {
        self.ensure_not_blocked()?;
        let state = prefix.state();
        let lock = acquire_lock(&self.directory)?;
        ensure_ready_for_normal_operation(&self.directory, &lock.directory)?;
        let opened = open_live_store_in(&lock.directory, passphrase)?;
        validate_snapshot_prefix(&opened.snapshot, prefix)?;
        let parent = opened.parent_metadata();
        let reconciled = reconcile_entries(opened.snapshot, state)?;
        if !reconciled.changed {
            return Ok(CustodyReconciliation {
                gaps: reconciled.gaps,
                changed_snapshot: false,
            });
        }
        let gaps = reconciled.gaps.clone();
        self.commit_snapshot(
            Some(parent),
            reconciled.snapshot,
            passphrase,
            SnapshotPublication::Normal,
            &lock,
        )?;
        Ok(CustodyReconciliation {
            gaps,
            changed_snapshot: true,
        })
    }

    /// Construct, self-open, and owner-authorize one fresh immutable protected object.
    pub fn create_protected_object(
        &mut self,
        prefix: &VerifiedPrivacyLedgerPrefix<'_>,
        passphrase: &CustodyPassphrase,
        protected_content: &[u8],
    ) -> Result<PrivacyControlTransition, CustodyError> {
        self.ensure_not_blocked()?;
        let profile = prefix.profile();
        let state = prefix.state();
        let anchor = prefix.next_anchor().clone();
        if !(1..=MAX_PROTECTED_CONTENT_BYTES).contains(&protected_content.len()) {
            return Err(CustodyError::InvalidProtectedContent);
        }
        let lock = acquire_lock(&self.directory)?;
        ensure_ready_for_normal_operation(&self.directory, &lock.directory)?;
        let opened = open_live_store_in(&lock.directory, passphrase)?;
        validate_snapshot_prefix(&opened.snapshot, prefix)?;
        let reconciled = reconcile_entries(opened.snapshot, state)?;
        if reconciled.changed || !reconciled.gaps.is_empty() || reconciled.has_prepared {
            return Err(CustodyError::CustodyGap);
        }
        let snapshot = reconciled.snapshot;
        validate_anchor_identity(&snapshot.network_id, profile, &anchor)?;
        let principal = snapshot.principal;
        let authorization_reference = state
            .active_key(principal, ParticipantKeyPurpose::PrivacyAuthorization)
            .cloned()
            .ok_or(CustodyError::CustodyGap)?;
        let wrapping_reference = state
            .active_key(principal, ParticipantKeyPurpose::PrivacyKeyWrapping)
            .cloned()
            .ok_or(CustodyError::CustodyGap)?;
        if !state.key_is_eligible(&authorization_reference, PrivacyKeyUse::AuthorizeTransition)
            || !state.key_is_eligible(&wrapping_reference, PrivacyKeyUse::CreateEnvelope)
        {
            return Err(CustodyError::CustodyGap);
        }
        let authorization_key = snapshot
            .bound_material(&authorization_reference)
            .ok_or(CustodyError::CustodyGap)?
            .authorization_signing_key()?;
        let wrapping_material = snapshot
            .bound_material(&wrapping_reference)
            .ok_or(CustodyError::CustodyGap)?;
        let wrapping_public: [u8; 65] = state
            .participant_public_key(&wrapping_reference)
            .ok_or(CustodyError::CustodyGap)?
            .try_into()
            .map_err(|_| CustodyError::CustodyGap)?;
        construct_protected_object(
            profile,
            anchor,
            state,
            principal,
            authorization_reference,
            wrapping_reference,
            &authorization_key,
            wrapping_public,
            wrapping_material.private_bytes(),
            protected_content,
        )
    }

    /// Recover an existing object's DEK in client custody and prepare one fresh owner grant.
    ///
    /// The object ciphertext and DEK are never changed or persisted in the node. Only the
    /// resulting canonical GrantAccess transition is disclosed after the existing custody
    /// snapshot has been authenticated.
    pub fn grant_access(
        &mut self,
        prefix: &VerifiedPrivacyLedgerPrefix<'_>,
        passphrase: &CustodyPassphrase,
        object_id: Uuid,
        grantee: Uuid,
    ) -> Result<PrivacyControlTransition, CustodyError> {
        self.ensure_not_blocked()?;
        let profile = prefix.profile();
        let state = prefix.state();
        let anchor = prefix.next_anchor().clone();
        let lock = acquire_lock(&self.directory)?;
        ensure_ready_for_normal_operation(&self.directory, &lock.directory)?;
        let opened = open_live_store_in(&lock.directory, passphrase)?;
        validate_snapshot_prefix(&opened.snapshot, prefix)?;
        let reconciliation = reconcile_entries(opened.snapshot, state)?;
        if reconciliation.changed || !reconciliation.gaps.is_empty() || reconciliation.has_prepared
        {
            return Err(CustodyError::CustodyGap);
        }
        let snapshot = reconciliation.snapshot;
        let owner = snapshot.principal;
        let object = state
            .protected_object(object_id)
            .ok_or(CustodyError::ProtectedDataOpenFailed)?;
        if object.owner() != owner || grantee == owner || !state.contains_principal(grantee) {
            return Err(CustodyError::ProtectedDataOpenFailed);
        }
        let authorization_reference = state
            .active_key(owner, ParticipantKeyPurpose::PrivacyAuthorization)
            .cloned()
            .ok_or(CustodyError::CustodyGap)?;
        let authorization_key = snapshot
            .bound_material(&authorization_reference)
            .ok_or(CustodyError::CustodyGap)?
            .authorization_signing_key()?;
        let owner_wrapping = snapshot
            .bound_material(object.wrapping_key())
            .ok_or(CustodyError::CustodyGap)?;
        if !state.key_is_eligible(object.wrapping_key(), PrivacyKeyUse::HistoricalRelease) {
            return Err(CustodyError::ProtectedDataOpenFailed);
        }
        let (dek, _) = recover_object_dek(object, owner_wrapping.private_bytes())?;

        let grantee_wrapping_reference = state
            .active_key(grantee, ParticipantKeyPurpose::PrivacyKeyWrapping)
            .cloned()
            .ok_or(CustodyError::CustodyGap)?;
        if !state.key_is_eligible(&grantee_wrapping_reference, PrivacyKeyUse::CreateEnvelope) {
            return Err(CustodyError::CustodyGap);
        }
        let grantee_wrapping_public: [u8; 65] = state
            .participant_public_key(&grantee_wrapping_reference)
            .ok_or(CustodyError::CustodyGap)?
            .try_into()
            .map_err(|_| CustodyError::CustodyGap)?;
        let (header, context, info) = grant_envelope_header_and_context(
            &anchor,
            object.object_id(),
            owner,
            grantee,
            PrivacyPermission::ReadProtectedObjectV1,
            grantee_wrapping_reference,
            object.payload().protected_content_commitment(),
            object.encrypted_payload_commitment(),
        );
        let (encapped, wrapped) =
            seal_dek_for_public_key(&grantee_wrapping_public, &info, dek.as_slice())?;
        let delivery = grant_envelope(context, header, encapped, wrapped)?;
        let unsigned = UnsignedPrivacyTransition::grant_access(
            profile,
            anchor,
            object.object_id(),
            owner,
            grantee,
            PrivacyPermission::ReadProtectedObjectV1,
            delivery,
            authorization_reference,
        )?;
        let signature = authorization_key
            .sign(&unsigned.authorization_digest())
            .to_bytes();
        Ok(unsigned.complete_grant_access(signature)?)
    }

    /// Prepare and owner-authorize one terminal prospective grant revocation.
    ///
    /// Revocation changes only future managed releases. The immutable grant
    /// history, payload ciphertext, and grant envelope remain untouched.
    pub fn revoke_grant(
        &mut self,
        prefix: &VerifiedPrivacyLedgerPrefix<'_>,
        passphrase: &CustodyPassphrase,
        grant_id: [u8; 32],
    ) -> Result<PrivacyControlTransition, CustodyError> {
        self.ensure_not_blocked()?;
        let profile = prefix.profile();
        let state = prefix.state();
        let anchor = prefix.next_anchor().clone();
        let lock = acquire_lock(&self.directory)?;
        ensure_ready_for_normal_operation(&self.directory, &lock.directory)?;
        let opened = open_live_store_in(&lock.directory, passphrase)?;
        validate_snapshot_prefix(&opened.snapshot, prefix)?;
        let reconciliation = reconcile_entries(opened.snapshot, state)?;
        if reconciliation.changed || !reconciliation.gaps.is_empty() || reconciliation.has_prepared
        {
            return Err(CustodyError::CustodyGap);
        }
        let snapshot = reconciliation.snapshot;
        let owner = snapshot.principal;
        let grant = state
            .privacy_grant(&grant_id)
            .filter(|grant| grant.status() == crate::privacy::PrivacyGrantStatus::Active)
            .ok_or(CustodyError::ProtectedDataOpenFailed)?;
        if grant.owner() != owner {
            return Err(CustodyError::ProtectedDataOpenFailed);
        }
        let authorization_reference = state
            .active_key(owner, ParticipantKeyPurpose::PrivacyAuthorization)
            .cloned()
            .ok_or(CustodyError::CustodyGap)?;
        if !state.key_is_eligible(&authorization_reference, PrivacyKeyUse::AuthorizeTransition) {
            return Err(CustodyError::CustodyGap);
        }
        let authorization_key = snapshot
            .bound_material(&authorization_reference)
            .ok_or(CustodyError::CustodyGap)?
            .authorization_signing_key()?;
        let unsigned = UnsignedPrivacyTransition::revoke_grant(
            profile,
            anchor,
            grant_id,
            grant.object_id(),
            grant.grantee(),
            grant.owner(),
            grant.permission(),
            grant.creation_ledger_position(),
            authorization_reference,
        )?;
        let signature = authorization_key
            .sign(&unsigned.authorization_digest())
            .to_bytes();
        Ok(unsigned.complete_grant_revocation(signature)?)
    }

    /// Open one opaque live-release response in participant custody.
    ///
    /// The caller supplies the public object context because the release response
    /// deliberately contains only ciphertext, one recipient envelope, and
    /// convergence evidence. This method is client-side only; a ledger node has
    /// no access to the snapshot or private key material used here.
    pub fn open_live_release(
        &self,
        prefix: &NetworkConvergedPrivacyPrefix<'_>,
        passphrase: &CustodyPassphrase,
        response: &LivePrivacyReleaseResponse,
        object_context: [u8; 32],
    ) -> Result<Vec<u8>, CustodyError> {
        self.ensure_not_blocked()?;
        validate_live_release_response(prefix, response, object_context)?;
        let lock = acquire_lock(&self.directory)?;
        ensure_ready_for_normal_operation(&self.directory, &lock.directory)?;
        let opened = open_live_store_in(&lock.directory, passphrase)?;
        validate_snapshot_prefix(&opened.snapshot, prefix.verified())?;
        let reconciliation = reconcile_entries(opened.snapshot, prefix.state())?;
        if reconciliation.changed || !reconciliation.gaps.is_empty() || reconciliation.has_prepared
        {
            return Err(CustodyError::CustodyGap);
        }
        let snapshot = reconciliation.snapshot;
        let evidence = response.evidence();
        if snapshot.principal != evidence.requester()
            || evidence.recipient_key().purpose() != ParticipantKeyPurpose::PrivacyKeyWrapping
        {
            return Err(CustodyError::ProtectedDataOpenFailed);
        }
        let material = snapshot
            .bound_material(evidence.recipient_key())
            .ok_or(CustodyError::ProtectedDataOpenFailed)?;
        let dek = match response.envelope() {
            PrivacyReleaseEnvelope::Owner(envelope) => {
                if evidence.path() != LivePrivacyReleasePath::Owner
                    || envelope.object_context() != object_context
                    || envelope.wrapping_key() != evidence.recipient_key()
                {
                    return Err(CustodyError::ProtectedDataOpenFailed);
                }
                let header_hash = protected_domain_hash(
                    "provchain/protected-data/hpke-owner-header/v1",
                    envelope.header_bytes(),
                );
                let mut info = [0_u8; 64];
                info[..32].copy_from_slice(&object_context);
                info[32..].copy_from_slice(&header_hash);
                open_dek_for_private_key(
                    material.private_bytes(),
                    envelope.encapsulated_key(),
                    &info,
                    envelope.wrapped_dek_ciphertext(),
                )?
            }
            PrivacyReleaseEnvelope::Grant(envelope) => {
                if evidence.path() != LivePrivacyReleasePath::Grantee
                    || envelope.wrapping_key() != evidence.recipient_key()
                    || envelope.header().object_id() != evidence.object_id()
                    || envelope.header().grantee() != evidence.requester()
                {
                    return Err(CustodyError::ProtectedDataOpenFailed);
                }
                let info = envelope.hpke_info();
                open_dek_for_private_key(
                    material.private_bytes(),
                    envelope.encapsulated_key(),
                    &info,
                    envelope.wrapped_dek_ciphertext(),
                )?
            }
        };
        open_release_payload(
            dek.as_slice(),
            object_context,
            response.payload(),
            response.encrypted_payload_commitment(),
        )
    }

    fn ensure_not_blocked(&self) -> Result<(), CustodyError> {
        if self.blocked {
            Err(CustodyError::CommitIndeterminate)
        } else {
            Ok(())
        }
    }

    fn publish_recovery_candidate(
        &mut self,
        candidate: &Path,
        target: &Path,
        expected: &ExpectedCustodyTarget,
        passphrase: &CustodyPassphrase,
        no_replace: bool,
        lock: &LockGuard,
    ) -> Result<(), CustodyError> {
        lock.revalidate(&self.directory)?;
        verify_expected_target(
            &lock.directory,
            target.file_name().ok_or(CustodyError::RecoveryConflict)?,
            expected,
            passphrase,
        )?;
        if publish_candidate(candidate, target, no_replace, lock).is_err() {
            self.blocked = true;
            return Err(CustodyError::CommitIndeterminate);
        }
        if lock.sync_directory().is_err() {
            self.blocked = true;
            return Err(CustodyError::CommitIndeterminate);
        }
        Ok(())
    }

    fn commit_snapshot(
        &mut self,
        parent: Option<ParentMetadata>,
        snapshot: Snapshot,
        passphrase: &CustodyPassphrase,
        publication: SnapshotPublication,
        lock: &LockGuard,
    ) -> Result<(), CustodyError> {
        let failpoint = self.next_failpoint.take();
        let (file_id, generation, previous, expected) = match parent.as_ref() {
            Some(parent) => (
                parent.reference.file_id,
                parent
                    .reference
                    .generation
                    .checked_add(1)
                    .ok_or(CustodyError::InvalidSnapshot)?,
                digest_reference(parent.reference.digest),
                if publication == SnapshotPublication::Restore {
                    ExpectedCustodyTarget::Absent
                } else {
                    ExpectedCustodyTarget::Present(parent.reference.clone())
                },
            ),
            None => (
                random_nonzero_16()?,
                1,
                [0_u8; 33],
                ExpectedCustodyTarget::Absent,
            ),
        };
        let candidate = seal_snapshot(file_id, generation, previous, snapshot, passphrase)?;
        let temp_name = format!("{TEMP_PREFIX}{}", hex::encode(random_16()?));
        let temp_path = self.directory.join(&temp_name);
        if write_new_file_in(&lock.directory, OsStr::new(&temp_name), &candidate.raw).is_err() {
            return self.abort_prepublication(&temp_path, &expected, passphrase, lock);
        }

        if failpoint == Some(CustodyCommitFailpoint::BeforePublish) {
            return self.abort_prepublication(&temp_path, &expected, passphrase, lock);
        }
        if failpoint == Some(CustodyCommitFailpoint::CrashAfterCandidateDurable) {
            self.blocked = true;
            return Err(CustodyError::CommitIndeterminate);
        }
        if lock.revalidate(&self.directory).is_err()
            || verify_expected_target(
                &lock.directory,
                OsStr::new(LIVE_NAME),
                &expected,
                passphrase,
            )
            .is_err()
        {
            return self.abort_prepublication(&temp_path, &expected, passphrase, lock);
        }
        let no_replace = matches!(expected, ExpectedCustodyTarget::Absent)
            || publication == SnapshotPublication::Restore;
        if publish_candidate(&temp_path, &self.live_path(), no_replace, lock).is_err() {
            self.blocked = true;
            return Err(CustodyError::CommitIndeterminate);
        }
        if failpoint == Some(CustodyCommitFailpoint::AfterPublishBeforeDirectorySync) {
            self.blocked = true;
            return Err(CustodyError::CommitIndeterminate);
        }
        if lock.sync_directory().is_err() {
            self.blocked = true;
            return Err(CustodyError::CommitIndeterminate);
        }
        if failpoint == Some(CustodyCommitFailpoint::AfterCommitBeforeReopen) {
            self.blocked = true;
            return Err(CustodyError::CommittedButQuarantined);
        }
        let reopened = match open_store_in(&lock.directory, OsStr::new(LIVE_NAME), passphrase) {
            Ok(reopened) => reopened,
            Err(_) => {
                self.blocked = true;
                return Err(CustodyError::CommittedButQuarantined);
            }
        };
        if reopened.digest != candidate.digest {
            self.blocked = true;
            return Err(CustodyError::CommittedButQuarantined);
        }
        Ok(())
    }

    fn abort_prepublication(
        &mut self,
        candidate: &Path,
        expected: &ExpectedCustodyTarget,
        passphrase: &CustodyPassphrase,
        lock: &LockGuard,
    ) -> Result<(), CustodyError> {
        let cleanup = (|| {
            let candidate_name = candidate
                .file_name()
                .ok_or(CustodyError::RecoveryConflict)?;
            match open_file_in_raw(
                &lock.directory,
                candidate_name,
                OFlags::RDONLY,
                Mode::empty(),
            ) {
                Ok(_) => cleanup_candidate_in(&lock.directory, candidate_name)?,
                Err(rustix::io::Errno::NOENT) => {}
                Err(_) => return Err(CustodyError::RecoveryConflict),
            }
            verify_expected_target(&lock.directory, OsStr::new(LIVE_NAME), expected, passphrase)
        })();
        if cleanup.is_ok() {
            Err(CustodyError::Aborted)
        } else {
            self.blocked = true;
            Err(CustodyError::CommitIndeterminate)
        }
    }
}

struct Snapshot {
    network_id: String,
    principal: Uuid,
    entries: BTreeMap<(u8, u32), CustodyEntry>,
}

impl Snapshot {
    fn bound_material(&self, reference: &ParticipantKeyReference) -> Option<&KeyMaterial> {
        self.entries.values().find_map(|entry| match entry {
            CustodyEntry::Bound(material) if &material.reference == reference => Some(material),
            _ => None,
        })
    }

    fn encode(&self) -> Result<Zeroizing<Vec<u8>>, CustodyError> {
        if self.entries.is_empty() || self.entries.len() > 1_024 {
            return Err(CustodyError::InvalidSnapshot);
        }
        let mut sequence = Zeroizing::new(Vec::new());
        let mut seen_public = BTreeSet::new();
        let mut seen_fingerprints = BTreeSet::new();
        for (key, entry) in &self.entries {
            if *key != entry.sort_key()
                || !seen_public.insert(entry.public_key().to_vec())
                || !seen_fingerprints.insert(entry.fingerprint())
            {
                return Err(CustodyError::InvalidSnapshot);
            }
            let encoded = entry.encode()?;
            if encoded.is_empty() || encoded.len() > 20_480 {
                return Err(CustodyError::InvalidSnapshot);
            }
            sequence.extend_from_slice(
                &u32::try_from(encoded.len())
                    .map_err(|_| CustodyError::InvalidSnapshot)?
                    .to_be_bytes(),
            );
            sequence.extend_from_slice(&encoded);
        }
        let count = u32::try_from(self.entries.len())
            .map_err(|_| CustodyError::InvalidSnapshot)?
            .to_be_bytes();
        let list = Zeroizing::new(pk_encode_record_refs(
            ENTRY_LIST_TAG,
            &[&count, sequence.as_slice()],
        )?);
        let snapshot = Zeroizing::new(pk_encode_record_refs(
            SNAPSHOT_TAG,
            &[
                self.network_id.as_bytes(),
                self.principal.as_bytes(),
                list.as_slice(),
            ],
        )?);
        if snapshot.len() > MAX_SNAPSHOT_BYTES {
            return Err(CustodyError::InvalidSnapshot);
        }
        Ok(snapshot)
    }

    fn decode(bytes: &[u8]) -> Result<Self, CustodyError> {
        if bytes.is_empty() || bytes.len() > MAX_SNAPSHOT_BYTES {
            return Err(CustodyError::InvalidSnapshot);
        }
        let fields = pk_decode_record(bytes, SNAPSHOT_TAG, 3)?;
        let network_id = std::str::from_utf8(fields[0])
            .map_err(|_| CustodyError::InvalidSnapshot)?
            .to_string();
        validate_network_id(&network_id)?;
        let principal = Uuid::from_bytes(fixed::<16>(fields[1])?);
        if principal.is_nil() {
            return Err(CustodyError::InvalidSnapshot);
        }
        let list = pk_decode_record(fields[2], ENTRY_LIST_TAG, 2)?;
        let count = u32::from_be_bytes(fixed::<4>(list[0])?) as usize;
        if !(1..=1_024).contains(&count) {
            return Err(CustodyError::InvalidSnapshot);
        }
        let mut cursor = 0_usize;
        let mut entries = BTreeMap::new();
        let mut seen_public = BTreeSet::new();
        let mut seen_fingerprints = BTreeSet::new();
        let mut previous_key = None;
        for _ in 0..count {
            let end = cursor.checked_add(4).ok_or(CustodyError::InvalidSnapshot)?;
            if end > list[1].len() {
                return Err(CustodyError::InvalidSnapshot);
            }
            let length = u32::from_be_bytes(fixed::<4>(&list[1][cursor..end])?) as usize;
            if !(1..=20_480).contains(&length) {
                return Err(CustodyError::InvalidSnapshot);
            }
            cursor = end;
            let end = cursor
                .checked_add(length)
                .ok_or(CustodyError::InvalidSnapshot)?;
            if end > list[1].len() {
                return Err(CustodyError::InvalidSnapshot);
            }
            let entry = CustodyEntry::decode(&list[1][cursor..end], principal)?;
            cursor = end;
            let sort_key = entry.sort_key();
            if previous_key.is_some_and(|previous| previous >= sort_key) {
                return Err(CustodyError::InvalidSnapshot);
            }
            previous_key = Some(sort_key);
            if !seen_public.insert(entry.public_key().to_vec())
                || !seen_fingerprints.insert(entry.fingerprint())
                || entries.insert(sort_key, entry).is_some()
            {
                return Err(CustodyError::InvalidSnapshot);
            }
        }
        if cursor != list[1].len() {
            return Err(CustodyError::InvalidSnapshot);
        }
        Ok(Self {
            network_id,
            principal,
            entries,
        })
    }
}

enum CustodyEntry {
    Bound(KeyMaterial),
    Tombstone(KeyTombstone),
    PreparedGovernance {
        material: KeyMaterial,
        request: PreparedRequest,
    },
    PreparedNode {
        material: KeyMaterial,
        request: Box<PreparedRequest>,
        transition: PrivacyControlTransition,
    },
}

impl CustodyEntry {
    fn sort_key(&self) -> (u8, u32) {
        match self {
            Self::Bound(material)
            | Self::PreparedGovernance { material, .. }
            | Self::PreparedNode { material, .. } => material.sort_key(),
            Self::Tombstone(tombstone) => tombstone.sort_key(),
        }
    }

    fn reference(&self) -> &ParticipantKeyReference {
        match self {
            Self::Bound(material)
            | Self::PreparedGovernance { material, .. }
            | Self::PreparedNode { material, .. } => &material.reference,
            Self::Tombstone(tombstone) => &tombstone.reference,
        }
    }

    fn public_key(&self) -> &[u8] {
        match self {
            Self::Bound(material)
            | Self::PreparedGovernance { material, .. }
            | Self::PreparedNode { material, .. } => &material.public_key,
            Self::Tombstone(tombstone) => &tombstone.public_key,
        }
    }

    fn fingerprint(&self) -> [u8; 32] {
        self.reference().fingerprint()
    }

    fn encode(&self) -> Result<Zeroizing<Vec<u8>>, CustodyError> {
        let bytes = match self {
            Self::Bound(material) => {
                let material = material.encode()?;
                pk_encode_record_refs(BOUND_KEY_TAG, &[material.as_slice()])?
            }
            Self::Tombstone(tombstone) => tombstone.encode()?,
            Self::PreparedGovernance { material, request } => {
                let material = material.encode()?;
                let request = request.encode();
                pk_encode_record_refs(
                    PREPARED_GOVERNANCE_TAG,
                    &[material.as_slice(), request.as_slice()],
                )?
            }
            Self::PreparedNode {
                material,
                request,
                transition,
            } => {
                let material = material.encode()?;
                let request = request.encode();
                let transition = transition.canonical_bytes();
                pk_encode_record_refs(
                    PREPARED_NODE_TAG,
                    &[
                        material.as_slice(),
                        request.as_slice(),
                        transition.as_slice(),
                    ],
                )?
            }
        };
        Ok(Zeroizing::new(bytes))
    }

    fn decode(bytes: &[u8], principal: Uuid) -> Result<Self, CustodyError> {
        let tag = *bytes.get(4).ok_or(CustodyError::InvalidSnapshot)?;
        match tag {
            BOUND_KEY_TAG => {
                let fields = pk_decode_record(bytes, BOUND_KEY_TAG, 1)?;
                Ok(Self::Bound(KeyMaterial::decode(fields[0], principal)?))
            }
            TOMBSTONE_TAG => Ok(Self::Tombstone(KeyTombstone::decode(bytes, principal)?)),
            PREPARED_GOVERNANCE_TAG => {
                let fields = pk_decode_record(bytes, PREPARED_GOVERNANCE_TAG, 2)?;
                let material = KeyMaterial::decode(fields[0], principal)?;
                let request = PreparedRequest::decode(fields[1])?;
                if !request.unsigned.is_registration() || request.authorizer_tag != 0x01 {
                    return Err(CustodyError::InvalidPreparedState);
                }
                request.validate_material(&material)?;
                Ok(Self::PreparedGovernance { material, request })
            }
            PREPARED_NODE_TAG => {
                let fields = pk_decode_record(bytes, PREPARED_NODE_TAG, 3)?;
                let material = KeyMaterial::decode(fields[0], principal)?;
                let request = PreparedRequest::decode(fields[1])?;
                request.validate_material(&material)?;
                let transition = PrivacyControlTransition::decode(fields[2])?;
                if transition.transition_id() != request.unsigned.transition_id()
                    || transition.unsigned_core_bytes() != request.unsigned.core_bytes()
                    || transition.introduced_key_reference().as_ref() != Some(&material.reference)
                    || transition.introduced_key_public_bytes()
                        != Some(material.public_key.as_slice())
                {
                    return Err(CustodyError::InvalidPreparedState);
                }
                Ok(Self::PreparedNode {
                    material,
                    request: Box::new(request),
                    transition,
                })
            }
            _ => Err(CustodyError::InvalidSnapshot),
        }
    }
}

struct KeyMaterial {
    purpose: ParticipantKeyPurpose,
    version: u32,
    reference: ParticipantKeyReference,
    public_key: Vec<u8>,
    private_key: Zeroizing<[u8; 32]>,
}

impl KeyMaterial {
    fn authorization(
        principal: Uuid,
        version: u32,
        seed: Zeroizing<[u8; 32]>,
    ) -> Result<Self, CustodyError> {
        let signing = SigningKey::from_bytes(&seed);
        let public_key = signing.verifying_key().to_bytes();
        Ok(Self {
            purpose: ParticipantKeyPurpose::PrivacyAuthorization,
            version,
            reference: ParticipantKeyReference::for_authorization(principal, version, public_key)?,
            public_key: public_key.to_vec(),
            private_key: seed,
        })
    }

    fn wrapping(
        principal: Uuid,
        version: u32,
        scalar: Zeroizing<[u8; 32]>,
    ) -> Result<Self, CustodyError> {
        let signing = P256SigningKey::from_slice(scalar.as_slice())
            .map_err(|_| CustodyError::InvalidSnapshot)?;
        let public_key: [u8; 65] = signing
            .verifying_key()
            .to_sec1_point(false)
            .as_bytes()
            .try_into()
            .map_err(|_| CustodyError::InvalidSnapshot)?;
        Ok(Self {
            purpose: ParticipantKeyPurpose::PrivacyKeyWrapping,
            version,
            reference: ParticipantKeyReference::for_wrapping(principal, version, public_key)?,
            public_key: public_key.to_vec(),
            private_key: scalar,
        })
    }

    fn sort_key(&self) -> (u8, u32) {
        (self.purpose as u8, self.version)
    }

    fn public_key_65(&self) -> Result<[u8; 65], CustodyError> {
        self.public_key
            .as_slice()
            .try_into()
            .map_err(|_| CustodyError::InvalidSnapshot)
    }

    fn private_bytes(&self) -> &[u8; 32] {
        &self.private_key
    }

    fn authorization_signing_key(&self) -> Result<SigningKey, CustodyError> {
        if self.purpose != ParticipantKeyPurpose::PrivacyAuthorization {
            return Err(CustodyError::InvalidSnapshot);
        }
        Ok(SigningKey::from_bytes(&self.private_key))
    }

    fn wrapping_signing_key(&self) -> Result<P256SigningKey, CustodyError> {
        if self.purpose != ParticipantKeyPurpose::PrivacyKeyWrapping {
            return Err(CustodyError::InvalidSnapshot);
        }
        P256SigningKey::from_slice(self.private_key.as_slice())
            .map_err(|_| CustodyError::InvalidSnapshot)
    }

    fn encode(&self) -> Result<Zeroizing<Vec<u8>>, CustodyError> {
        let (algorithm, public_encoding, private_encoding) = match self.purpose {
            ParticipantKeyPurpose::PrivacyAuthorization => (0x01, 0x01, 0x01),
            ParticipantKeyPurpose::PrivacyKeyWrapping => (0x02, 0x02, 0x02),
        };
        let purpose = [self.purpose as u8];
        let version = self.version.to_be_bytes();
        let algorithm = [algorithm];
        let public_encoding = [public_encoding];
        let fingerprint = self.reference.fingerprint();
        let private_encoding = [private_encoding];
        Ok(Zeroizing::new(pk_encode_record_refs(
            PRIVATE_MATERIAL_TAG,
            &[
                &purpose,
                &version,
                &algorithm,
                &public_encoding,
                self.public_key.as_slice(),
                &fingerprint,
                &private_encoding,
                self.private_key.as_slice(),
            ],
        )?))
    }

    fn decode(bytes: &[u8], principal: Uuid) -> Result<Self, CustodyError> {
        let fields = pk_decode_record(bytes, PRIVATE_MATERIAL_TAG, 8)?;
        let purpose = match one(fields[0])? {
            0x01 => ParticipantKeyPurpose::PrivacyAuthorization,
            0x02 => ParticipantKeyPurpose::PrivacyKeyWrapping,
            _ => return Err(CustodyError::InvalidSnapshot),
        };
        let version = u32::from_be_bytes(fixed::<4>(fields[1])?);
        if version == 0 {
            return Err(CustodyError::InvalidSnapshot);
        }
        let private = Zeroizing::new(fixed::<32>(fields[7])?);
        let material = match purpose {
            ParticipantKeyPurpose::PrivacyAuthorization => {
                if one(fields[2])? != 0x01
                    || one(fields[3])? != 0x01
                    || one(fields[6])? != 0x01
                    || fields[4].len() != 32
                {
                    return Err(CustodyError::InvalidSnapshot);
                }
                let derived = SigningKey::from_bytes(&private).verifying_key().to_bytes();
                if fields[4] != derived {
                    return Err(CustodyError::InvalidSnapshot);
                }
                Self::authorization(principal, version, private)?
            }
            ParticipantKeyPurpose::PrivacyKeyWrapping => {
                if one(fields[2])? != 0x02
                    || one(fields[3])? != 0x02
                    || one(fields[6])? != 0x02
                    || fields[4].len() != 65
                {
                    return Err(CustodyError::InvalidSnapshot);
                }
                let signing = P256SigningKey::from_slice(private.as_slice())
                    .map_err(|_| CustodyError::InvalidSnapshot)?;
                let derived = signing.verifying_key().to_sec1_point(false);
                if fields[4] != derived.as_bytes() {
                    return Err(CustodyError::InvalidSnapshot);
                }
                Self::wrapping(principal, version, private)?
            }
        };
        if material.public_key.as_slice() != fields[4]
            || material.reference.fingerprint() != fixed::<32>(fields[5])?
            || material.encode()?.as_slice() != bytes
        {
            return Err(CustodyError::InvalidSnapshot);
        }
        Ok(material)
    }
}

struct KeyTombstone {
    purpose: ParticipantKeyPurpose,
    version: u32,
    reference: ParticipantKeyReference,
    public_key: Vec<u8>,
}

impl KeyTombstone {
    fn sort_key(&self) -> (u8, u32) {
        (self.purpose as u8, self.version)
    }

    fn encode(&self) -> Result<Vec<u8>, CustodyError> {
        let (algorithm, encoding) = match self.purpose {
            ParticipantKeyPurpose::PrivacyAuthorization => (0x01, 0x01),
            ParticipantKeyPurpose::PrivacyKeyWrapping => (0x02, 0x02),
        };
        pk_encode_record(
            TOMBSTONE_TAG,
            vec![
                vec![self.purpose as u8],
                self.version.to_be_bytes().to_vec(),
                vec![algorithm],
                vec![encoding],
                self.public_key.clone(),
                self.reference.fingerprint().to_vec(),
            ],
        )
    }

    fn decode(bytes: &[u8], principal: Uuid) -> Result<Self, CustodyError> {
        let fields = pk_decode_record(bytes, TOMBSTONE_TAG, 6)?;
        let purpose = match one(fields[0])? {
            0x01 => ParticipantKeyPurpose::PrivacyAuthorization,
            0x02 => ParticipantKeyPurpose::PrivacyKeyWrapping,
            _ => return Err(CustodyError::InvalidSnapshot),
        };
        let version = u32::from_be_bytes(fixed::<4>(fields[1])?);
        if version == 0 {
            return Err(CustodyError::InvalidSnapshot);
        }
        let (reference, public_key) = match purpose {
            ParticipantKeyPurpose::PrivacyAuthorization => {
                if one(fields[2])? != 0x01 || one(fields[3])? != 0x01 {
                    return Err(CustodyError::InvalidSnapshot);
                }
                let public = fixed::<32>(fields[4])?;
                VerifyingKey::from_bytes(&public).map_err(|_| CustodyError::InvalidSnapshot)?;
                (
                    ParticipantKeyReference::for_authorization(principal, version, public)?,
                    public.to_vec(),
                )
            }
            ParticipantKeyPurpose::PrivacyKeyWrapping => {
                if one(fields[2])? != 0x02 || one(fields[3])? != 0x02 {
                    return Err(CustodyError::InvalidSnapshot);
                }
                let public = fixed::<65>(fields[4])?;
                (
                    ParticipantKeyReference::for_wrapping(principal, version, public)?,
                    public.to_vec(),
                )
            }
        };
        if reference.fingerprint() != fixed::<32>(fields[5])? {
            return Err(CustodyError::InvalidSnapshot);
        }
        let tombstone = Self {
            purpose,
            version,
            reference,
            public_key,
        };
        if tombstone.encode()? != bytes {
            return Err(CustodyError::InvalidSnapshot);
        }
        Ok(tombstone)
    }
}

#[derive(Clone)]
struct PreparedRequest {
    authorizer_tag: u8,
    authorizer_reference: Vec<u8>,
    unsigned: UnsignedPrivacyTransition,
    possession_scheme: u8,
    possession_proof: [u8; 64],
}

impl PreparedRequest {
    fn new(
        authorizer_tag: u8,
        authorizer_reference: Vec<u8>,
        unsigned: UnsignedPrivacyTransition,
        possession_scheme: u8,
        possession_proof: [u8; 64],
    ) -> Result<Self, CustodyError> {
        let request = Self {
            authorizer_tag,
            authorizer_reference,
            unsigned,
            possession_scheme,
            possession_proof,
        };
        request.validate_contract()?;
        Ok(request)
    }

    fn encode(&self) -> Vec<u8> {
        pk_encode_record(
            BINDING_REQUEST_TAG,
            vec![
                vec![self.authorizer_tag],
                self.authorizer_reference.clone(),
                self.unsigned.core_bytes().to_vec(),
                self.unsigned.transition_id().to_vec(),
                vec![self.possession_scheme],
                self.possession_proof.to_vec(),
                self.unsigned.authorization_digest().to_vec(),
            ],
        )
        .expect("closed bounded prepared request")
    }

    fn decode(bytes: &[u8]) -> Result<Self, CustodyError> {
        if bytes.len() > 9_216 {
            return Err(CustodyError::InvalidPreparedState);
        }
        let fields = pk_decode_record(bytes, BINDING_REQUEST_TAG, 7)?;
        let unsigned = UnsignedPrivacyTransition::from_prepared_core(fields[2])?;
        let request = Self {
            authorizer_tag: one(fields[0])?,
            authorizer_reference: fields[1].to_vec(),
            unsigned,
            possession_scheme: one(fields[4])?,
            possession_proof: fixed::<64>(fields[5])?,
        };
        if request.unsigned.transition_id() != fixed::<32>(fields[3])?
            || request.unsigned.authorization_digest() != fixed::<32>(fields[6])?
            || request.encode() != bytes
        {
            return Err(CustodyError::InvalidPreparedState);
        }
        request.validate_contract()?;
        Ok(request)
    }

    fn validate_contract(&self) -> Result<(), CustodyError> {
        let (authorizer_tag, authorizer_reference) = self
            .unsigned
            .prepared_authorizer_contract()
            .ok_or(CustodyError::InvalidPreparedState)?;
        if self.authorizer_tag != authorizer_tag
            || self.authorizer_reference != authorizer_reference
            || !matches!(self.possession_scheme, 0x01 | 0x02)
        {
            return Err(CustodyError::InvalidPreparedState);
        }
        Ok(())
    }

    fn validate_material(&self, material: &KeyMaterial) -> Result<(), CustodyError> {
        if self.unsigned.introduced_key_reference() != Some(&material.reference)
            || self.unsigned.introduced_key_public_bytes() != Some(material.public_key.as_slice())
            || self.possession_scheme
                != match material.purpose {
                    ParticipantKeyPurpose::PrivacyAuthorization => 0x01,
                    ParticipantKeyPurpose::PrivacyKeyWrapping => 0x02,
                }
        {
            return Err(CustodyError::InvalidPreparedState);
        }
        self.unsigned
            .verify_stored_possession(self.possession_proof)
            .map_err(|_| CustodyError::InvalidPreparedState)?;
        Ok(())
    }
}

struct Header {
    file_id: [u8; 16],
    snapshot_id: [u8; 16],
    generation: u64,
    previous: [u8; 33],
    salt: [u8; 16],
    plaintext_length: u32,
}

impl Header {
    fn encode(&self) -> Result<Vec<u8>, CustodyError> {
        let bytes = pk_encode_record(
            HEADER_TAG,
            vec![
                KEYSTORE_SUITE.as_bytes().to_vec(),
                KEYSTORE_CODEC.as_bytes().to_vec(),
                self.file_id.to_vec(),
                self.snapshot_id.to_vec(),
                self.generation.to_be_bytes().to_vec(),
                self.previous.to_vec(),
                vec![0x02],
                vec![0x13],
                65_536_u32.to_be_bytes().to_vec(),
                3_u32.to_be_bytes().to_vec(),
                4_u32.to_be_bytes().to_vec(),
                32_u32.to_be_bytes().to_vec(),
                self.salt.to_vec(),
                vec![0x01],
                vec![0x01],
                vec![0x01],
                self.plaintext_length.to_be_bytes().to_vec(),
            ],
        )?;
        if bytes.len() != HEADER_BYTES {
            return Err(CustodyError::InvalidSnapshot);
        }
        Ok(bytes)
    }

    fn decode(bytes: &[u8]) -> Result<Self, CustodyError> {
        if bytes.len() != HEADER_BYTES {
            return Err(CustodyError::InvalidSnapshot);
        }
        let fields = pk_decode_record(bytes, HEADER_TAG, 17)?;
        if fields[0] != KEYSTORE_SUITE.as_bytes()
            || fields[1] != KEYSTORE_CODEC.as_bytes()
            || one(fields[6])? != 0x02
            || one(fields[7])? != 0x13
            || u32::from_be_bytes(fixed::<4>(fields[8])?) != 65_536
            || u32::from_be_bytes(fixed::<4>(fields[9])?) != 3
            || u32::from_be_bytes(fixed::<4>(fields[10])?) != 4
            || u32::from_be_bytes(fixed::<4>(fields[11])?) != 32
            || one(fields[13])? != 0x01
            || one(fields[14])? != 0x01
            || one(fields[15])? != 0x01
        {
            return Err(CustodyError::InvalidSnapshot);
        }
        let generation = u64::from_be_bytes(fixed::<8>(fields[4])?);
        let previous = fixed::<33>(fields[5])?;
        if generation == 0
            || (generation == 1 && previous != [0; 33])
            || (generation > 1 && (previous[0] != 1 || previous[1..] == [0; 32]))
        {
            return Err(CustodyError::InvalidSnapshot);
        }
        let file_id = fixed::<16>(fields[2])?;
        let snapshot_id = fixed::<16>(fields[3])?;
        let salt = fixed::<16>(fields[12])?;
        if file_id == [0; 16] || snapshot_id == [0; 16] {
            return Err(CustodyError::InvalidSnapshot);
        }
        let plaintext_length = u32::from_be_bytes(fixed::<4>(fields[16])?);
        if !(1..=MAX_SNAPSHOT_BYTES as u32).contains(&plaintext_length) {
            return Err(CustodyError::InvalidSnapshot);
        }
        let header = Self {
            file_id,
            snapshot_id,
            generation,
            previous,
            salt,
            plaintext_length,
        };
        if header.encode()? != bytes {
            return Err(CustodyError::InvalidSnapshot);
        }
        Ok(header)
    }
}

struct OpenedStore {
    header: Header,
    snapshot: Snapshot,
    digest: [u8; 32],
    raw: Vec<u8>,
}

impl OpenedStore {
    fn parent_metadata(&self) -> ParentMetadata {
        ParentMetadata {
            reference: self.snapshot_reference(),
        }
    }

    fn snapshot_reference(&self) -> CustodySnapshotReference {
        CustodySnapshotReference {
            file_id: self.header.file_id,
            generation: self.header.generation,
            digest: self.digest,
            network_id: self.snapshot.network_id.clone(),
            principal: self.snapshot.principal,
        }
    }
}

struct ParentMetadata {
    reference: CustodySnapshotReference,
}

enum ExpectedCustodyTarget {
    Absent,
    Present(CustodySnapshotReference),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SnapshotPublication {
    Normal,
    Restore,
}

struct SealedStore {
    raw: Vec<u8>,
    digest: [u8; 32],
}

struct ReconciledSnapshot {
    snapshot: Snapshot,
    gaps: Vec<ParticipantKeyReference>,
    changed: bool,
    has_prepared: bool,
}

fn reconcile_entries(
    snapshot: Snapshot,
    state: &EffectivePrivacyState,
) -> Result<ReconciledSnapshot, CustodyError> {
    let mut entries = BTreeMap::new();
    let mut changed = false;
    let mut has_prepared = false;
    for (key, entry) in snapshot.entries {
        let reference = entry.reference().clone();
        let ledger_public = state.participant_public_key(&reference);
        let status = state.key_status(&reference);
        let next = match entry {
            CustodyEntry::PreparedNode { material, .. }
                if ledger_public == Some(material.public_key.as_slice())
                    && matches!(
                        status,
                        Some(KeyLifecycleStatus::Active | KeyLifecycleStatus::Retired)
                    ) =>
            {
                changed = true;
                CustodyEntry::Bound(material)
            }
            CustodyEntry::PreparedNode {
                material,
                request,
                transition,
            } => {
                has_prepared = true;
                CustodyEntry::PreparedNode {
                    material,
                    request,
                    transition,
                }
            }
            CustodyEntry::PreparedGovernance { material, request } => {
                has_prepared = true;
                CustodyEntry::PreparedGovernance { material, request }
            }
            CustodyEntry::Bound(material) if status == Some(KeyLifecycleStatus::Revoked) => {
                if ledger_public != Some(material.public_key.as_slice()) {
                    return Err(CustodyError::LedgerConflict);
                }
                changed = true;
                CustodyEntry::Tombstone(KeyTombstone {
                    purpose: material.purpose,
                    version: material.version,
                    reference: material.reference,
                    public_key: material.public_key,
                })
            }
            CustodyEntry::Bound(material)
                if ledger_public == Some(material.public_key.as_slice())
                    && matches!(
                        status,
                        Some(KeyLifecycleStatus::Active | KeyLifecycleStatus::Retired)
                    ) =>
            {
                CustodyEntry::Bound(material)
            }
            CustodyEntry::Tombstone(tombstone)
                if ledger_public == Some(tombstone.public_key.as_slice())
                    && status == Some(KeyLifecycleStatus::Revoked) =>
            {
                CustodyEntry::Tombstone(tombstone)
            }
            _ => return Err(CustodyError::LedgerConflict),
        };
        entries.insert(key, next);
    }
    let present: BTreeSet<_> = entries
        .values()
        .map(|entry| entry.reference().clone())
        .collect();
    let gaps = state
        .participant_keys(snapshot.principal)
        .into_iter()
        .filter(|record| !present.contains(&record.reference))
        .map(|record| record.reference)
        .collect();
    Ok(ReconciledSnapshot {
        snapshot: Snapshot {
            network_id: snapshot.network_id,
            principal: snapshot.principal,
            entries,
        },
        gaps,
        changed,
        has_prepared,
    })
}

fn reconcile_restore_entries(
    snapshot: Snapshot,
    prefix: &VerifiedPrivacyLedgerPrefix<'_>,
) -> Result<ReconciledSnapshot, CustodyError> {
    let state = prefix.state();
    let mut entries = BTreeMap::new();
    let mut changed = false;
    let mut has_prepared = false;
    for (key, entry) in snapshot.entries {
        let reference = entry.reference().clone();
        let ledger_public = state.participant_public_key(&reference);
        let status = state.key_status(&reference);
        let next = match entry {
            CustodyEntry::Bound(material)
            | CustodyEntry::PreparedGovernance { material, .. }
            | CustodyEntry::PreparedNode { material, .. }
                if ledger_public == Some(material.public_key.as_slice())
                    && matches!(
                        status,
                        Some(KeyLifecycleStatus::Active | KeyLifecycleStatus::Retired)
                    ) =>
            {
                changed = true;
                Some(CustodyEntry::Bound(material))
            }
            CustodyEntry::Bound(material)
            | CustodyEntry::PreparedGovernance { material, .. }
            | CustodyEntry::PreparedNode { material, .. }
                if ledger_public == Some(material.public_key.as_slice())
                    && status == Some(KeyLifecycleStatus::Revoked) =>
            {
                changed = true;
                Some(CustodyEntry::Tombstone(KeyTombstone {
                    purpose: material.purpose,
                    version: material.version,
                    reference: material.reference,
                    public_key: material.public_key,
                }))
            }
            CustodyEntry::Tombstone(tombstone)
                if ledger_public == Some(tombstone.public_key.as_slice())
                    && status == Some(KeyLifecycleStatus::Revoked) =>
            {
                Some(CustodyEntry::Tombstone(tombstone))
            }
            CustodyEntry::PreparedGovernance { material, request }
                if request.unsigned.is_registration()
                    && state.revision() == 0
                    && !state.contains_principal(snapshot.principal)
                    && validate_prepared_current(prefix, &request, None).is_ok() =>
            {
                has_prepared = true;
                Some(CustodyEntry::PreparedGovernance { material, request })
            }
            CustodyEntry::PreparedNode {
                material,
                request,
                transition,
            } if validate_prepared_current(prefix, &request, Some(&transition)).is_ok() => {
                has_prepared = true;
                Some(CustodyEntry::PreparedNode {
                    material,
                    request,
                    transition,
                })
            }
            _ => {
                changed = true;
                None
            }
        };
        if let Some(next) = next {
            entries.insert(key, next);
        }
    }
    let present: BTreeSet<_> = entries
        .values()
        .map(|entry| entry.reference().clone())
        .collect();
    let gaps = state
        .participant_keys(snapshot.principal)
        .into_iter()
        .filter(|record| !present.contains(&record.reference))
        .map(|record| record.reference)
        .collect();
    Ok(ReconciledSnapshot {
        snapshot: Snapshot {
            network_id: snapshot.network_id,
            principal: snapshot.principal,
            entries,
        },
        gaps,
        changed,
        has_prepared,
    })
}

fn validate_live_release_response(
    prefix: &NetworkConvergedPrivacyPrefix<'_>,
    response: &LivePrivacyReleaseResponse,
    object_context: [u8; 32],
) -> Result<(), CustodyError> {
    let evidence = response.evidence();
    let current_position = prefix
        .current_ledger_position()
        .ok_or(CustodyError::ProtectedDataOpenFailed)?;
    let current_prefix_hash = prefix.current_ledger_prefix_hash();
    let profile = prefix.profile();
    let anchor = prefix.next_anchor();
    if evidence.network_id() != anchor.network_id()
        || evidence.profile_id() != anchor.profile_id()
        || evidence.profile_content_hash() != profile.network_profile_content_hash()
        || evidence.converged_ledger_position() != current_position
        || evidence.ledger_prefix_hash() != current_prefix_hash
    {
        return Err(CustodyError::ProtectedDataOpenFailed);
    }

    let state = prefix.state();
    let object = state
        .protected_object(evidence.object_id())
        .ok_or(CustodyError::ProtectedDataOpenFailed)?;
    if object.object_context() != object_context
        || response.payload() != object.payload()
        || response.encrypted_payload_commitment() != object.encrypted_payload_commitment()
    {
        return Err(CustodyError::ProtectedDataOpenFailed);
    }

    match (evidence.path(), evidence.grant_id(), response.envelope()) {
        (LivePrivacyReleasePath::Owner, None, PrivacyReleaseEnvelope::Owner(envelope))
            if evidence.requester() == object.owner()
                && evidence.recipient_key() == object.wrapping_key()
                && state.key_is_eligible(
                    evidence.recipient_key(),
                    PrivacyKeyUse::HistoricalRelease,
                )
                && envelope.canonical_bytes() == object.owner_envelope().canonical_bytes() => {}
        (
            LivePrivacyReleasePath::Grantee,
            Some(grant_id),
            PrivacyReleaseEnvelope::Grant(envelope),
        ) => {
            let grant = state
                .active_privacy_grant(object.object_id(), evidence.requester())
                .filter(|grant| {
                    grant.grant_id() == grant_id && grant.status() == PrivacyGrantStatus::Active
                })
                .ok_or(CustodyError::ProtectedDataOpenFailed)?;
            if grant.permission() != PrivacyPermission::ReadProtectedObjectV1
                || evidence.recipient_key() != grant.delivery().wrapping_key()
                || !state
                    .key_is_eligible(evidence.recipient_key(), PrivacyKeyUse::HistoricalRelease)
                || envelope.canonical_bytes() != grant.delivery().canonical_bytes()
            {
                return Err(CustodyError::ProtectedDataOpenFailed);
            }
        }
        _ => return Err(CustodyError::ProtectedDataOpenFailed),
    }
    Ok(())
}

fn validate_snapshot_prefix(
    snapshot: &Snapshot,
    prefix: &VerifiedPrivacyLedgerPrefix<'_>,
) -> Result<(), CustodyError> {
    let profile = prefix.profile();
    profile.validate()?;
    validate_network_id(&snapshot.network_id)?;
    if snapshot.network_id != prefix.next_anchor().network_id() {
        return Err(CustodyError::InvalidSnapshot);
    }
    validate_anchor_identity(&snapshot.network_id, profile, prefix.next_anchor())?;
    Ok(())
}

fn validate_anchor_identity(
    network_id: &str,
    profile: &PrivacyLifecycleProfile,
    anchor: &PrivacyAdmissionAnchor,
) -> Result<(), CustodyError> {
    if anchor.network_id() != network_id
        || anchor.profile_content_hash() != profile.network_profile_content_hash()
    {
        return Err(CustodyError::LedgerConflict);
    }
    Ok(())
}

fn validate_prepared_current(
    prefix: &VerifiedPrivacyLedgerPrefix<'_>,
    request: &PreparedRequest,
    transition: Option<&PrivacyControlTransition>,
) -> Result<(), CustodyError> {
    let profile = prefix.profile();
    let state = prefix.state();
    let expected_anchor = prefix.next_anchor();
    validate_anchor_identity(expected_anchor.network_id(), profile, expected_anchor)?;
    if request.unsigned.admission_anchor() != expected_anchor {
        return Err(CustodyError::LedgerConflict);
    }
    if let Some(transition) = transition {
        if transition.admission_anchor() != expected_anchor
            || transition.unsigned_core_bytes() != request.unsigned.core_bytes()
            || state
                .stage_transition(profile, expected_anchor, transition)
                .is_err()
        {
            return Err(CustodyError::LedgerConflict);
        }
    } else if !request.unsigned.is_registration()
        || state.revision() != 0
        || request
            .unsigned
            .admission_anchor()
            .expected_privacy_revision()
            != 1
    {
        return Err(CustodyError::LedgerConflict);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn construct_protected_object(
    profile: &PrivacyLifecycleProfile,
    anchor: PrivacyAdmissionAnchor,
    state: &EffectivePrivacyState,
    owner: Uuid,
    authorization_reference: ParticipantKeyReference,
    wrapping_reference: ParticipantKeyReference,
    authorization_key: &SigningKey,
    wrapping_public: [u8; 65],
    wrapping_private: &[u8; 32],
    protected_content: &[u8],
) -> Result<PrivacyControlTransition, CustodyError> {
    let object_id = loop {
        let candidate = random_uuid_v4()?;
        if state.protected_object(candidate).is_none() {
            break candidate;
        }
    };
    let mut dek = Zeroizing::new([0_u8; 32]);
    random_fill(dek.as_mut())?;
    let extract_salt: [u8; 32] = Sha256V11::digest(PAYLOAD_EXTRACT_SALT_DOMAIN.as_bytes()).into();
    let (_, hkdf) = Hkdf::<Sha256V11>::extract(Some(&extract_salt), dek.as_slice());
    let pcc_info = protected_encode_record(
        0x02,
        vec![
            b"provchain/protected-data/pcc-key/v1".to_vec(),
            PROTECTED_DATA_SUITE_V1.as_bytes().to_vec(),
        ],
    );
    let mut pcc_key = Zeroizing::new([0_u8; 32]);
    hkdf.expand(&pcc_info, pcc_key.as_mut())
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let mac_input = Zeroizing::new(protected_encode_record_refs(
        0x04,
        &[b"provchain/protected-data/pcc/v1", protected_content],
    ));
    let mut mac = <Hmac<Sha256V11> as hmac::KeyInit>::new_from_slice(pcc_key.as_slice())
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    mac.update(&mac_input);
    let pcc: [u8; 32] = mac.finalize().into_bytes().into();
    let object_context =
        protected_object_context(&anchor, object_id, owner, &wrapping_reference, pcc);
    let payload_info = protected_encode_record(
        0x03,
        vec![
            b"provchain/protected-data/payload-key/v1".to_vec(),
            PROTECTED_DATA_SUITE_V1.as_bytes().to_vec(),
            object_context.to_vec(),
        ],
    );
    let mut payload_key = Zeroizing::new([0_u8; 32]);
    hkdf.expand(&payload_info, payload_key.as_mut())
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let payload_cipher = ChaCha20Poly1305::new_from_slice(payload_key.as_slice())
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let ciphertext = payload_cipher
        .encrypt(
            &[0_u8; 12].into(),
            Payload {
                msg: protected_content,
                aad: &object_context,
            },
        )
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let payload = protected_payload(pcc, ciphertext)?;
    let epc = encrypted_payload_commitment(&payload);
    let (header, owner_info) = owner_envelope_header_and_info(
        &anchor,
        object_id,
        owner,
        wrapping_reference.clone(),
        pcc,
        epc,
        object_context,
    );
    type Kem = DhP256HkdfSha256;
    type HpkeAead = HpkeChaCha20Poly1305;
    type HpkeKdf = HkdfSha256;
    let recipient_public = <Kem as KemTrait>::PublicKey::from_bytes(&wrapping_public)
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let mut rng = OneShotHpkeRng::from_system()?;
    let (encapped, mut sender) = setup_sender_with_rng::<HpkeAead, HpkeKdf, Kem>(
        &OpModeS::Base,
        &recipient_public,
        &owner_info,
        &mut rng,
    )
    .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    if !rng.exactly_consumed() {
        return Err(CustodyError::EntropyUnavailable);
    }
    let wrapped = sender
        .seal(dek.as_slice(), b"")
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let encapped: [u8; 65] = encapped
        .to_bytes()
        .as_slice()
        .try_into()
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let wrapped: [u8; 48] = wrapped
        .try_into()
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let envelope = owner_envelope(object_context, header, encapped, wrapped)?;

    // Mandatory local owner-envelope and payload self-check before any bytes are returned.
    let recipient_private = <Kem as KemTrait>::PrivateKey::from_bytes(wrapping_private)
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let encapped_key = <Kem as KemTrait>::EncappedKey::from_bytes(&encapped)
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let mut receiver = setup_receiver::<HpkeAead, HpkeKdf, Kem>(
        &OpModeR::Base,
        &recipient_private,
        &encapped_key,
        &owner_info,
    )
    .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let mut recovered_dek = Zeroizing::new(
        receiver
            .open(&wrapped, b"")
            .map_err(|_| CustodyError::ProtectedDataOpenFailed)?,
    );
    if recovered_dek.as_slice() != dek.as_slice() {
        return Err(CustodyError::ProtectedDataOpenFailed);
    }
    let (_, recovered_hkdf) =
        Hkdf::<Sha256V11>::extract(Some(&extract_salt), recovered_dek.as_slice());
    let mut recovered_pcc_key = Zeroizing::new([0_u8; 32]);
    recovered_hkdf
        .expand(&pcc_info, recovered_pcc_key.as_mut())
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let mut recovered_mac =
        <Hmac<Sha256V11> as hmac::KeyInit>::new_from_slice(recovered_pcc_key.as_slice())
            .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    recovered_mac.update(&mac_input);
    let recovered_pcc: [u8; 32] = recovered_mac.finalize().into_bytes().into();
    let mut recovered_payload_key = Zeroizing::new([0_u8; 32]);
    recovered_hkdf
        .expand(&payload_info, recovered_payload_key.as_mut())
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let recovered = Zeroizing::new(
        ChaCha20Poly1305::new_from_slice(recovered_payload_key.as_slice())
            .map_err(|_| CustodyError::ProtectedDataOpenFailed)?
            .decrypt(
                &[0_u8; 12].into(),
                Payload {
                    msg: payload.ciphertext_and_tag(),
                    aad: &object_context,
                },
            )
            .map_err(|_| CustodyError::ProtectedDataOpenFailed)?,
    );
    if recovered.as_slice() != protected_content
        || recovered_pcc != pcc
        || encrypted_payload_commitment(&payload) != epc
        || protected_domain_hash(
            "provchain/protected-data/encrypted-payload-commitment/v1",
            &payload.canonical_bytes(),
        ) != epc
    {
        return Err(CustodyError::ProtectedDataOpenFailed);
    }
    recovered_dek.zeroize();

    let unsigned = UnsignedPrivacyTransition::create_protected_object(
        profile,
        anchor,
        object_id,
        owner,
        authorization_reference,
        wrapping_reference,
        object_context,
        payload,
        epc,
        envelope,
    )?;
    let signature = authorization_key
        .sign(&unsigned.authorization_digest())
        .to_bytes();
    Ok(unsigned.complete_object_creation(signature)?)
}

type RecoveredProtectedObject = (Zeroizing<[u8; 32]>, Zeroizing<Vec<u8>>);

fn recover_object_dek(
    object: &ProtectedObjectRecord,
    wrapping_private: &[u8; 32],
) -> Result<RecoveredProtectedObject, CustodyError> {
    type Kem = DhP256HkdfSha256;
    type HpkeAead = HpkeChaCha20Poly1305;
    type HpkeKdf = HkdfSha256;
    let recipient_private = <Kem as KemTrait>::PrivateKey::from_bytes(wrapping_private)
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let encapsulated =
        <Kem as KemTrait>::EncappedKey::from_bytes(&object.owner_envelope().encapsulated_key())
            .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let header_hash = protected_domain_hash(
        "provchain/protected-data/hpke-owner-header/v1",
        object.owner_envelope().header_bytes(),
    );
    let mut owner_info = [0_u8; 64];
    owner_info[..32].copy_from_slice(&object.owner_envelope().object_context());
    owner_info[32..].copy_from_slice(&header_hash);
    let mut receiver = setup_receiver::<HpkeAead, HpkeKdf, Kem>(
        &OpModeR::Base,
        &recipient_private,
        &encapsulated,
        &owner_info,
    )
    .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let recovered = receiver
        .open(&object.owner_envelope().wrapped_dek_ciphertext(), b"")
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let recovered_dek: [u8; 32] = recovered
        .as_slice()
        .try_into()
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let recovered_dek = Zeroizing::new(recovered_dek);

    let extract_salt: [u8; 32] = Sha256V11::digest(PAYLOAD_EXTRACT_SALT_DOMAIN.as_bytes()).into();
    let (_, hkdf) = Hkdf::<Sha256V11>::extract(Some(&extract_salt), recovered_dek.as_slice());
    let pcc_info = protected_encode_record(
        0x02,
        vec![
            b"provchain/protected-data/pcc-key/v1".to_vec(),
            PROTECTED_DATA_SUITE_V1.as_bytes().to_vec(),
        ],
    );
    let mut pcc_key = Zeroizing::new([0_u8; 32]);
    hkdf.expand(&pcc_info, pcc_key.as_mut())
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let payload_info = protected_encode_record(
        0x03,
        vec![
            b"provchain/protected-data/payload-key/v1".to_vec(),
            PROTECTED_DATA_SUITE_V1.as_bytes().to_vec(),
            object.object_context().to_vec(),
        ],
    );
    let mut payload_key = Zeroizing::new([0_u8; 32]);
    hkdf.expand(&payload_info, payload_key.as_mut())
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let plaintext = Zeroizing::new(
        ChaCha20Poly1305::new_from_slice(payload_key.as_slice())
            .map_err(|_| CustodyError::ProtectedDataOpenFailed)?
            .decrypt(
                &[0_u8; 12].into(),
                Payload {
                    msg: object.payload().ciphertext_and_tag(),
                    aad: &object.object_context(),
                },
            )
            .map_err(|_| CustodyError::ProtectedDataOpenFailed)?,
    );
    let mac_input = Zeroizing::new(protected_encode_record_refs(
        0x04,
        &[b"provchain/protected-data/pcc/v1", plaintext.as_slice()],
    ));
    let mut mac = <Hmac<Sha256V11> as hmac::KeyInit>::new_from_slice(pcc_key.as_slice())
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    mac.update(&mac_input);
    let pcc: [u8; 32] = mac.finalize().into_bytes().into();
    if pcc != object.payload().protected_content_commitment()
        || encrypted_payload_commitment(object.payload()) != object.encrypted_payload_commitment()
    {
        return Err(CustodyError::ProtectedDataOpenFailed);
    }
    Ok((recovered_dek, plaintext))
}

fn open_dek_for_private_key(
    private_bytes: &[u8; 32],
    encapsulated_bytes: [u8; 65],
    info: &[u8; 64],
    wrapped_bytes: [u8; 48],
) -> Result<Zeroizing<[u8; 32]>, CustodyError> {
    type Kem = DhP256HkdfSha256;
    type HpkeAead = HpkeChaCha20Poly1305;
    type HpkeKdf = HkdfSha256;
    let recipient_private = <Kem as KemTrait>::PrivateKey::from_bytes(private_bytes)
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let encapsulated = <Kem as KemTrait>::EncappedKey::from_bytes(&encapsulated_bytes)
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let mut receiver = setup_receiver::<HpkeAead, HpkeKdf, Kem>(
        &OpModeR::Base,
        &recipient_private,
        &encapsulated,
        info,
    )
    .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let recovered = receiver
        .open(&wrapped_bytes, b"")
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let recovered: [u8; 32] = recovered
        .as_slice()
        .try_into()
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    Ok(Zeroizing::new(recovered))
}

fn open_release_payload(
    dek: &[u8],
    object_context: [u8; 32],
    payload: &crate::privacy::ProtectedPayloadCiphertext,
    expected_encrypted_payload_commitment: [u8; 32],
) -> Result<Vec<u8>, CustodyError> {
    type PayloadCipher = ChaCha20Poly1305;
    if dek.len() != 32
        || encrypted_payload_commitment(payload) != expected_encrypted_payload_commitment
    {
        return Err(CustodyError::ProtectedDataOpenFailed);
    }
    let extract_salt: [u8; 32] = Sha256V11::digest(PAYLOAD_EXTRACT_SALT_DOMAIN.as_bytes()).into();
    let (_, hkdf) = Hkdf::<Sha256V11>::extract(Some(&extract_salt), dek);
    let pcc_info = protected_encode_record(
        0x02,
        vec![
            b"provchain/protected-data/pcc-key/v1".to_vec(),
            PROTECTED_DATA_SUITE_V1.as_bytes().to_vec(),
        ],
    );
    let mut pcc_key = Zeroizing::new([0_u8; 32]);
    hkdf.expand(&pcc_info, pcc_key.as_mut())
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let payload_info = protected_encode_record(
        0x03,
        vec![
            b"provchain/protected-data/payload-key/v1".to_vec(),
            PROTECTED_DATA_SUITE_V1.as_bytes().to_vec(),
            object_context.to_vec(),
        ],
    );
    let mut payload_key = Zeroizing::new([0_u8; 32]);
    hkdf.expand(&payload_info, payload_key.as_mut())
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let plaintext = Zeroizing::new(
        PayloadCipher::new_from_slice(payload_key.as_slice())
            .map_err(|_| CustodyError::ProtectedDataOpenFailed)?
            .decrypt(
                &[0_u8; 12].into(),
                Payload {
                    msg: payload.ciphertext_and_tag(),
                    aad: &object_context,
                },
            )
            .map_err(|_| CustodyError::ProtectedDataOpenFailed)?,
    );
    let mac_input = Zeroizing::new(protected_encode_record_refs(
        0x04,
        &[b"provchain/protected-data/pcc/v1", plaintext.as_slice()],
    ));
    let mut mac = <Hmac<Sha256V11> as hmac::KeyInit>::new_from_slice(pcc_key.as_slice())
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    mac.update(&mac_input);
    let pcc: [u8; 32] = mac.finalize().into_bytes().into();
    if pcc != payload.protected_content_commitment() {
        return Err(CustodyError::ProtectedDataOpenFailed);
    }
    Ok(plaintext.to_vec())
}

fn seal_dek_for_public_key(
    recipient_public_bytes: &[u8; 65],
    info: &[u8; 64],
    dek: &[u8],
) -> Result<([u8; 65], [u8; 48]), CustodyError> {
    type Kem = DhP256HkdfSha256;
    type HpkeAead = HpkeChaCha20Poly1305;
    type HpkeKdf = HkdfSha256;
    let recipient_public = <Kem as KemTrait>::PublicKey::from_bytes(recipient_public_bytes)
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let mut rng = OneShotHpkeRng::from_system()?;
    let (encapped, mut sender) = setup_sender_with_rng::<HpkeAead, HpkeKdf, Kem>(
        &OpModeS::Base,
        &recipient_public,
        info,
        &mut rng,
    )
    .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    if !rng.exactly_consumed() {
        return Err(CustodyError::EntropyUnavailable);
    }
    let wrapped = sender
        .seal(dek, b"")
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let encapped: [u8; 65] = encapped
        .to_bytes()
        .as_slice()
        .try_into()
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let wrapped: [u8; 48] = wrapped
        .try_into()
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    Ok((encapped, wrapped))
}

fn seal_snapshot(
    file_id: [u8; 16],
    generation: u64,
    previous: [u8; 33],
    snapshot: Snapshot,
    passphrase: &CustodyPassphrase,
) -> Result<SealedStore, CustodyError> {
    let plaintext = snapshot.encode()?;
    let header = Header {
        file_id,
        snapshot_id: random_nonzero_16()?,
        generation,
        previous,
        salt: random_16()?,
        plaintext_length: u32::try_from(plaintext.len())
            .map_err(|_| CustodyError::InvalidSnapshot)?,
    };
    let header_bytes = header.encode()?;
    let mut aad = Vec::with_capacity(AAD_BYTES);
    aad.extend_from_slice(FILE_MAGIC);
    aad.extend_from_slice(&(HEADER_BYTES as u32).to_be_bytes());
    aad.extend_from_slice(&header_bytes);
    let mut key = derive_file_key(&header, passphrase)?;
    let cipher = ChaCha20Poly1305::new_from_slice(key.as_slice())
        .map_err(|_| CustodyError::InvalidSnapshot)?;
    let ciphertext = cipher
        .encrypt(
            &[0_u8; 12].into(),
            Payload {
                msg: plaintext.as_slice(),
                aad: &aad,
            },
        )
        .map_err(|_| CustodyError::InvalidSnapshot)?;
    key.zeroize();
    let mut raw = aad;
    raw.extend_from_slice(&ciphertext);
    if raw.len() > MAX_FILE_BYTES {
        return Err(CustodyError::InvalidSnapshot);
    }
    let digest = Sha256::digest(&raw).into();
    Ok(SealedStore { raw, digest })
}

fn open_store(path: &Path, passphrase: &CustodyPassphrase) -> Result<OpenedStore, CustodyError> {
    let directory = open_qualified_directory(
        path.parent()
            .ok_or(CustodyError::UnsupportedStorageProfile)?,
    )?;
    let name = path
        .file_name()
        .ok_or(CustodyError::UnsupportedStorageProfile)?;
    open_store_in(&directory, name, passphrase)
}

fn open_store_in(
    directory: &File,
    name: &OsStr,
    passphrase: &CustodyPassphrase,
) -> Result<OpenedStore, CustodyError> {
    let file = open_file_in(directory, name, OFlags::RDONLY, Mode::empty())?;
    validate_file_metadata(&file.metadata().map_err(io_error)?)?;
    let raw = read_open_file(&file)?;
    if raw.len() < AAD_BYTES + 17 || raw.len() > MAX_FILE_BYTES || &raw[..4] != FILE_MAGIC {
        return Err(CustodyError::InvalidSnapshot);
    }
    let header_length = u32::from_be_bytes(fixed::<4>(&raw[4..8])?) as usize;
    if header_length != HEADER_BYTES {
        return Err(CustodyError::InvalidSnapshot);
    }
    let header = Header::decode(&raw[8..AAD_BYTES])?;
    let expected = AAD_BYTES
        .checked_add(header.plaintext_length as usize)
        .and_then(|length| length.checked_add(16))
        .ok_or(CustodyError::InvalidSnapshot)?;
    if raw.len() != expected {
        return Err(CustodyError::InvalidSnapshot);
    }
    let mut key = derive_file_key(&header, passphrase)?;
    let plaintext = ChaCha20Poly1305::new_from_slice(key.as_slice())
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?
        .decrypt(
            &[0_u8; 12].into(),
            Payload {
                msg: &raw[AAD_BYTES..],
                aad: &raw[..AAD_BYTES],
            },
        )
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    key.zeroize();
    let mut plaintext = Zeroizing::new(plaintext);
    let snapshot = Snapshot::decode(&plaintext)?;
    plaintext.zeroize();
    let digest = Sha256::digest(&raw).into();
    Ok(OpenedStore {
        header,
        snapshot,
        digest,
        raw,
    })
}

fn open_live_store_in(
    directory: &File,
    passphrase: &CustodyPassphrase,
) -> Result<OpenedStore, CustodyError> {
    open_store_in(directory, OsStr::new(LIVE_NAME), passphrase).map_err(redact_keystore_open_error)
}

fn open_expected_store(
    path: &Path,
    expected: &CustodySnapshotReference,
    prefix: &VerifiedPrivacyLedgerPrefix<'_>,
    passphrase: &CustodyPassphrase,
) -> Result<OpenedStore, CustodyError> {
    let opened = open_store(path, passphrase).map_err(redact_keystore_open_error)?;
    validate_snapshot_prefix(&opened.snapshot, prefix)
        .map_err(|_| CustodyError::KeystoreOpenFailed)?;
    let actual = opened.snapshot_reference();
    if &actual != expected {
        return Err(CustodyError::KeystoreOpenFailed);
    }
    Ok(opened)
}

fn redact_keystore_open_error(error: CustodyError) -> CustodyError {
    match error {
        CustodyError::InvalidSnapshot
        | CustodyError::InvalidPreparedState
        | CustodyError::LedgerConflict
        | CustodyError::ProtectedDataOpenFailed
        | CustodyError::PublicProtocol => CustodyError::KeystoreOpenFailed,
        error => error,
    }
}

fn redact_recovery_open_error(error: CustodyError) -> CustodyError {
    match error {
        CustodyError::InvalidSnapshot
        | CustodyError::InvalidPreparedState
        | CustodyError::LedgerConflict
        | CustodyError::KeystoreOpenFailed
        | CustodyError::ProtectedDataOpenFailed
        | CustodyError::PublicProtocol => CustodyError::RecoveryConflict,
        error => error,
    }
}

struct ArgonArena(Vec<Block>);

impl ArgonArena {
    fn allocate() -> Result<Self, CustodyError> {
        let mut blocks = Vec::new();
        blocks
            .try_reserve_exact(ARGON_BLOCKS)
            .map_err(|_| CustodyError::ResourceUnavailable)?;
        blocks.resize(ARGON_BLOCKS, Block::default());
        Ok(Self(blocks))
    }
}

impl Drop for ArgonArena {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

fn derive_file_key(
    header: &Header,
    passphrase: &CustodyPassphrase,
) -> Result<Zeroizing<[u8; 32]>, CustodyError> {
    let params = Params::new(65_536, 3, 4, Some(32)).map_err(|_| CustodyError::InvalidSnapshot)?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut arena = ArgonArena::allocate()?;
    let mut root = Zeroizing::new([0_u8; 32]);
    argon
        .hash_password_into_with_memory(
            passphrase.expose(),
            &header.salt,
            root.as_mut(),
            &mut arena.0,
        )
        .map_err(|_| CustodyError::ProtectedDataOpenFailed)?;
    let (_, hkdf) = Hkdf::<Sha256V11>::extract(Some(HKDF_EXTRACT_SALT), root.as_slice());
    let info = pk_encode_record(
        FILE_KEY_INFO_TAG,
        vec![
            b"provchain/participant-keystore/file-aead-key/v1".to_vec(),
            KEYSTORE_SUITE.as_bytes().to_vec(),
            KEYSTORE_CODEC.as_bytes().to_vec(),
            header.file_id.to_vec(),
            header.snapshot_id.to_vec(),
            header.generation.to_be_bytes().to_vec(),
            32_u32.to_be_bytes().to_vec(),
        ],
    )?;
    if info.len() != 196 {
        return Err(CustodyError::InvalidSnapshot);
    }
    let mut key = Zeroizing::new([0_u8; 32]);
    hkdf.expand(&info, key.as_mut())
        .map_err(|_| CustodyError::InvalidSnapshot)?;
    Ok(key)
}

fn pk_encode_record(tag: u8, fields: Vec<Vec<u8>>) -> Result<Vec<u8>, CustodyError> {
    let fields = fields.iter().map(Vec::as_slice).collect::<Vec<_>>();
    pk_encode_record_refs(tag, &fields)
}

fn pk_encode_record_refs(tag: u8, fields: &[&[u8]]) -> Result<Vec<u8>, CustodyError> {
    let field_count = u8::try_from(fields.len()).map_err(|_| CustodyError::InvalidSnapshot)?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(KEYSTORE_MAGIC);
    bytes.push(tag);
    bytes.push(field_count);
    for (index, field) in fields.iter().enumerate() {
        if field.is_empty() {
            return Err(CustodyError::InvalidSnapshot);
        }
        bytes.push(u8::try_from(index + 1).map_err(|_| CustodyError::InvalidSnapshot)?);
        bytes.extend_from_slice(
            &u32::try_from(field.len())
                .map_err(|_| CustodyError::InvalidSnapshot)?
                .to_be_bytes(),
        );
        bytes.extend_from_slice(field);
    }
    Ok(bytes)
}

fn pk_decode_record(
    bytes: &[u8],
    expected_tag: u8,
    expected_fields: u8,
) -> Result<Vec<&[u8]>, CustodyError> {
    if bytes.len() < 6
        || &bytes[..4] != KEYSTORE_MAGIC
        || bytes[4] != expected_tag
        || bytes[5] != expected_fields
    {
        return Err(CustodyError::InvalidSnapshot);
    }
    let mut cursor = 6_usize;
    let mut fields = Vec::with_capacity(expected_fields as usize);
    for tag in 1..=expected_fields {
        let header_end = cursor.checked_add(5).ok_or(CustodyError::InvalidSnapshot)?;
        if header_end > bytes.len() || bytes[cursor] != tag {
            return Err(CustodyError::InvalidSnapshot);
        }
        let length = u32::from_be_bytes(fixed::<4>(&bytes[cursor + 1..header_end])?) as usize;
        if length == 0 {
            return Err(CustodyError::InvalidSnapshot);
        }
        cursor = header_end;
        let end = cursor
            .checked_add(length)
            .ok_or(CustodyError::InvalidSnapshot)?;
        if end > bytes.len() {
            return Err(CustodyError::InvalidSnapshot);
        }
        fields.push(&bytes[cursor..end]);
        cursor = end;
    }
    if cursor != bytes.len() {
        return Err(CustodyError::InvalidSnapshot);
    }
    Ok(fields)
}

fn one(bytes: &[u8]) -> Result<u8, CustodyError> {
    Ok(fixed::<1>(bytes)?[0])
}

fn fixed<const N: usize>(bytes: &[u8]) -> Result<[u8; N], CustodyError> {
    bytes.try_into().map_err(|_| CustodyError::InvalidSnapshot)
}

fn random_fill(bytes: &mut [u8]) -> Result<(), CustodyError> {
    getrandom::SysRng
        .try_fill_bytes(bytes)
        .map_err(|_| CustodyError::EntropyUnavailable)
}

fn random_16() -> Result<[u8; 16], CustodyError> {
    let mut value = [0_u8; 16];
    random_fill(&mut value)?;
    Ok(value)
}

fn random_nonzero_16() -> Result<[u8; 16], CustodyError> {
    loop {
        let value = random_16()?;
        if value != [0; 16] {
            return Ok(value);
        }
    }
}

fn random_uuid_v4() -> Result<Uuid, CustodyError> {
    let mut bytes = random_nonzero_16()?;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Ok(Uuid::from_bytes(bytes))
}

fn generate_p256_scalar() -> Result<Zeroizing<[u8; 32]>, CustodyError> {
    loop {
        let mut scalar = Zeroizing::new([0_u8; 32]);
        random_fill(scalar.as_mut())?;
        if P256SigningKey::from_slice(scalar.as_slice()).is_ok() {
            return Ok(scalar);
        }
    }
}

fn digest_reference(digest: [u8; 32]) -> [u8; 33] {
    let mut reference = [0_u8; 33];
    reference[0] = 1;
    reference[1..].copy_from_slice(&digest);
    reference
}

fn validate_network_id(value: &str) -> Result<(), CustodyError> {
    let bytes = value.as_bytes();
    let valid = !bytes.is_empty()
        && bytes.len() <= 128
        && value.is_ascii()
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'_' | b':' | b'-')
        });
    if valid {
        Ok(())
    } else {
        Err(CustodyError::InvalidSnapshot)
    }
}

fn open_qualified_directory(path: &Path) -> Result<File, CustodyError> {
    let (base, relative) = if path.is_absolute() {
        (
            rustix_open(
                "/",
                OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map(File::from)
            .map_err(|_| CustodyError::UnsupportedStorageProfile)?,
            path.strip_prefix("/")
                .map_err(|_| CustodyError::UnsupportedStorageProfile)?,
        )
    } else {
        (
            rustix_open(
                ".",
                OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map(File::from)
            .map_err(|_| CustodyError::UnsupportedStorageProfile)?,
            path,
        )
    };
    if relative.as_os_str().is_empty() {
        return Err(CustodyError::UnsupportedStorageProfile);
    }
    let directory = openat2(
        &base,
        relative,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
        RESOLVE_WITHIN_DIRECTORY,
    )
    .map_err(|_| CustodyError::UnsupportedStorageProfile)?;
    Ok(File::from(directory))
}

fn open_file_in_raw(
    directory: &File,
    name: &OsStr,
    flags: OFlags,
    mode: Mode,
) -> Result<File, rustix::io::Errno> {
    if name.as_bytes().is_empty() || name.as_bytes().contains(&b'/') {
        return Err(rustix::io::Errno::INVAL);
    }
    openat2(
        directory,
        name,
        flags | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        mode,
        RESOLVE_WITHIN_DIRECTORY,
    )
    .map(File::from)
}

fn open_file_in(
    directory: &File,
    name: &OsStr,
    flags: OFlags,
    mode: Mode,
) -> Result<File, CustodyError> {
    open_file_in_raw(directory, name, flags, mode).map_err(|_| CustodyError::Io)
}

fn named_file_exists(directory: &File, name: &OsStr) -> Result<bool, CustodyError> {
    match open_file_in_raw(directory, name, OFlags::RDONLY, Mode::empty()) {
        Ok(_) => Ok(true),
        Err(rustix::io::Errno::NOENT) => Ok(false),
        Err(_) => Err(CustodyError::RecoveryConflict),
    }
}

fn directory_entry_names(directory: &File) -> Result<Vec<OsString>, CustodyError> {
    let mut stream = Dir::read_from(directory).map_err(|_| CustodyError::Io)?;
    let mut names = Vec::new();
    while let Some(entry) = stream.read() {
        let entry = entry.map_err(|_| CustodyError::Io)?;
        let bytes = entry.file_name().to_bytes();
        if matches!(bytes, b"." | b"..") {
            continue;
        }
        names.push(OsStr::from_bytes(bytes).to_os_string());
    }
    Ok(names)
}

fn read_open_file(file: &File) -> Result<Vec<u8>, CustodyError> {
    let length = usize::try_from(file.metadata().map_err(io_error)?.len())
        .map_err(|_| CustodyError::InvalidSnapshot)?;
    if length > MAX_FILE_BYTES {
        return Err(CustodyError::InvalidSnapshot);
    }
    let mut bytes = Vec::<u8>::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| CustodyError::ResourceUnavailable)?;
    bytes.resize(length, 0);
    let mut offset = 0_usize;
    while offset < length {
        let file_offset = u64::try_from(offset).map_err(|_| CustodyError::Io)?;
        match file.read_at(&mut bytes[offset..], file_offset) {
            Ok(0) => return Err(CustodyError::Io),
            Ok(read) => offset = offset.checked_add(read).ok_or(CustodyError::Io)?,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return Err(CustodyError::Io),
        }
    }
    if usize::try_from(file.metadata().map_err(io_error)?.len()) != Ok(length) {
        return Err(CustodyError::Io);
    }
    Ok(bytes)
}

fn prepare_directory(path: &Path) -> Result<(), CustodyError> {
    if !path.try_exists().map_err(io_error)? {
        fs::DirBuilder::new()
            .mode(0o700)
            .create(path)
            .map_err(io_error)?;
        sync_directory(
            path.parent()
                .ok_or(CustodyError::UnsupportedStorageProfile)?,
        )?;
    }
    let directory = open_qualified_directory(path)?;
    let metadata = directory.metadata().map_err(io_error)?;
    if !metadata.file_type().is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o777 != 0o700
    {
        return Err(CustodyError::UnsupportedStorageProfile);
    }
    validate_ext4_mount(path, metadata.dev())
}

fn prepare_existing_directory(path: &Path) -> Result<(), CustodyError> {
    if !path.try_exists().map_err(io_error)? {
        return Err(CustodyError::UnsupportedStorageProfile);
    }
    prepare_directory(path)
}

fn validate_ext4_mount(path: &Path, device: u64) -> Result<(), CustodyError> {
    let major = libc::major(device as libc::dev_t);
    let minor = libc::minor(device as libc::dev_t);
    let device = format!("{major}:{minor}");
    let canonical_path = fs::canonicalize(path).map_err(io_error)?;
    let mountinfo = fs::read_to_string("/proc/self/mountinfo").map_err(io_error)?;
    let mut matches = Vec::new();
    for line in mountinfo.lines() {
        let Some((head, tail)) = line.split_once(" - ") else {
            continue;
        };
        let mut fields = head.split_whitespace();
        let (Some(_mount_id), Some(_parent_id), Some(entry_device)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        if entry_device != device {
            continue;
        }
        let _root = fields
            .next()
            .ok_or(CustodyError::UnsupportedStorageProfile)?;
        let mountpoint = decode_mountinfo_path(
            fields
                .next()
                .ok_or(CustodyError::UnsupportedStorageProfile)?,
        )
        .ok_or(CustodyError::UnsupportedStorageProfile)?;
        let filesystem = tail
            .split_whitespace()
            .next()
            .ok_or(CustodyError::UnsupportedStorageProfile)?;
        matches.push((mountpoint, filesystem));
    }
    let [(mountpoint, filesystem)] = matches.as_slice() else {
        return Err(CustodyError::UnsupportedStorageProfile);
    };
    if *filesystem != "ext4" || !canonical_path.starts_with(mountpoint) {
        return Err(CustodyError::UnsupportedStorageProfile);
    }
    Ok(())
}

fn decode_mountinfo_path(value: &str) -> Option<PathBuf> {
    let mut decoded = String::with_capacity(value.len());
    let mut bytes = value.as_bytes().iter().copied().peekable();
    while let Some(byte) = bytes.next() {
        if byte != b'\\' {
            decoded.push(char::from(byte));
            continue;
        }
        let first = bytes.next()?;
        let second = bytes.next()?;
        let third = bytes.next()?;
        if !(first.is_ascii_digit() && second.is_ascii_digit() && third.is_ascii_digit()) {
            return None;
        }
        let octal = (first - b'0') * 64 + (second - b'0') * 8 + (third - b'0');
        decoded.push(char::from(octal));
    }
    Some(PathBuf::from(decoded))
}

type DirectoryIdentity = (u64, u64);

fn held_directories() -> &'static Mutex<BTreeSet<DirectoryIdentity>> {
    static HELD: OnceLock<Mutex<BTreeSet<DirectoryIdentity>>> = OnceLock::new();
    HELD.get_or_init(|| Mutex::new(BTreeSet::new()))
}

struct LockGuard {
    file: File,
    directory: File,
    directory_identity: DirectoryIdentity,
    lock_identity: DirectoryIdentity,
    _directory_claim: DirectoryClaim,
}

impl LockGuard {
    fn revalidate(&self, directory: &Path) -> Result<(), CustodyError> {
        let held_directory = self.directory.metadata().map_err(io_error)?;
        if (held_directory.dev(), held_directory.ino()) != self.directory_identity {
            return Err(CustodyError::RecoveryConflict);
        }
        validate_ext4_mount(directory, held_directory.dev())?;
        let current_directory = open_qualified_directory(directory)?;
        let current_directory = current_directory.metadata().map_err(io_error)?;
        if (current_directory.dev(), current_directory.ino()) != self.directory_identity {
            return Err(CustodyError::RecoveryConflict);
        }
        let current_lock = openat2(
            &self.directory,
            LOCK_NAME,
            OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOFOLLOW,
            Mode::empty(),
            RESOLVE_WITHIN_DIRECTORY,
        )
        .map(File::from)
        .map_err(|_| CustodyError::RecoveryConflict)?;
        let current_lock = current_lock.metadata().map_err(io_error)?;
        validate_file_metadata(&current_lock)?;
        let held_lock = self.file.metadata().map_err(io_error)?;
        validate_file_metadata(&held_lock)?;
        if (current_lock.dev(), current_lock.ino()) != self.lock_identity
            || (held_lock.dev(), held_lock.ino()) != self.lock_identity
        {
            return Err(CustodyError::RecoveryConflict);
        }
        Ok(())
    }

    fn sync_directory(&self) -> Result<(), CustodyError> {
        self.directory.sync_all().map_err(io_error)
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        unsafe {
            libc::flock(self.file.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

struct DirectoryClaim(DirectoryIdentity);

impl DirectoryClaim {
    fn acquire(identity: DirectoryIdentity) -> Result<Self, CustodyError> {
        let mut held = held_directories()
            .lock()
            .map_err(|_| CustodyError::WriterBusy)?;
        if !held.insert(identity) {
            return Err(CustodyError::WriterBusy);
        }
        Ok(Self(identity))
    }
}

impl Drop for DirectoryClaim {
    fn drop(&mut self) {
        if let Ok(mut held) = held_directories().lock() {
            held.remove(&self.0);
        }
    }
}

fn acquire_lock(directory: &Path) -> Result<LockGuard, CustodyError> {
    let directory_file = open_qualified_directory(directory)?;
    let directory_metadata = directory_file.metadata().map_err(io_error)?;
    let directory_identity = (directory_metadata.dev(), directory_metadata.ino());
    let directory_claim = DirectoryClaim::acquire(directory_identity)?;
    let created = openat2(
        &directory_file,
        LOCK_NAME,
        OFlags::RDWR | OFlags::CREATE | OFlags::EXCL | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::RUSR | Mode::WUSR,
        RESOLVE_WITHIN_DIRECTORY,
    );
    let (file, existed) = match created {
        Ok(file) => (File::from(file), false),
        Err(rustix::io::Errno::EXIST) => {
            let file = openat2(
                &directory_file,
                LOCK_NAME,
                OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOFOLLOW,
                Mode::empty(),
                RESOLVE_WITHIN_DIRECTORY,
            )
            .map(File::from)
            .map_err(|_| CustodyError::RecoveryConflict)?;
            (file, true)
        }
        Err(_) => return Err(CustodyError::Io),
    };
    if !existed {
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(io_error)?;
    }
    let lock_metadata = file.metadata().map_err(io_error)?;
    validate_file_metadata(&lock_metadata)?;
    let lock_identity = (lock_metadata.dev(), lock_metadata.ino());
    if !existed {
        file.sync_all().map_err(io_error)?;
        directory_file.sync_all().map_err(io_error)?;
    }
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if result != 0 {
        return Err(CustodyError::WriterBusy);
    }
    Ok(LockGuard {
        file,
        directory: directory_file,
        directory_identity,
        lock_identity,
        _directory_claim: directory_claim,
    })
}

fn validate_file_in_directory(directory: &Path, path: &Path) -> Result<(), CustodyError> {
    if path.parent() != Some(directory) {
        return Err(CustodyError::RecoveryConflict);
    }
    let directory_file = open_qualified_directory(directory)?;
    let directory_metadata = directory_file.metadata().map_err(io_error)?;
    let metadata = open_file_in(
        &directory_file,
        path.file_name().ok_or(CustodyError::RecoveryConflict)?,
        OFlags::RDONLY,
        Mode::empty(),
    )?
    .metadata()
    .map_err(io_error)?;
    validate_file_metadata(&metadata)?;
    if metadata.dev() != directory_metadata.dev() {
        return Err(CustodyError::RecoveryConflict);
    }
    Ok(())
}

fn validate_file_metadata(metadata: &fs::Metadata) -> Result<(), CustodyError> {
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o777 != 0o600
        || metadata.nlink() != 1
    {
        return Err(CustodyError::RecoveryConflict);
    }
    Ok(())
}

fn write_new_file_in(directory: &File, name: &OsStr, bytes: &[u8]) -> Result<File, CustodyError> {
    let file = open_file_in_raw(
        directory,
        name,
        OFlags::RDWR | OFlags::CREATE | OFlags::EXCL,
        Mode::RUSR | Mode::WUSR,
    )
    .map_err(|_| CustodyError::Io)?;
    file.set_permissions(fs::Permissions::from_mode(0o600))
        .map_err(io_error)?;
    let mut offset = 0_usize;
    while offset < bytes.len() {
        let file_offset = u64::try_from(offset).map_err(|_| CustodyError::Io)?;
        match file.write_at(&bytes[offset..], file_offset) {
            Ok(0) => return Err(CustodyError::Io),
            Ok(written) => offset = offset.checked_add(written).ok_or(CustodyError::Io)?,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return Err(CustodyError::Io),
        }
    }
    let reread = read_open_file(&file)?;
    if reread != bytes {
        return Err(CustodyError::Io);
    }
    file.sync_all().map_err(io_error)?;
    let file_metadata = file.metadata().map_err(io_error)?;
    let directory_metadata = directory.metadata().map_err(io_error)?;
    validate_file_metadata(&file_metadata)?;
    if file_metadata.dev() != directory_metadata.dev() {
        return Err(CustodyError::RecoveryConflict);
    }
    Ok(file)
}

fn sync_named_file(directory: &File, name: &OsStr) -> Result<(), CustodyError> {
    open_file_in(directory, name, OFlags::RDONLY, Mode::empty())?
        .sync_all()
        .map_err(io_error)
}

fn sync_directory(path: &Path) -> Result<(), CustodyError> {
    open_qualified_directory(path)?.sync_all().map_err(io_error)
}

fn cleanup_candidate_in(directory: &File, name: &OsStr) -> Result<(), CustodyError> {
    let candidate = open_file_in(directory, name, OFlags::RDONLY, Mode::empty())?;
    let metadata = candidate.metadata().map_err(io_error)?;
    let directory_metadata = directory.metadata().map_err(io_error)?;
    validate_file_metadata(&metadata)?;
    if metadata.dev() != directory_metadata.dev() {
        return Err(CustodyError::RecoveryConflict);
    }
    unlinkat(directory, name, AtFlags::empty()).map_err(|_| CustodyError::Io)?;
    directory.sync_all().map_err(io_error)
}

fn verify_expected_target(
    directory: &File,
    name: &OsStr,
    expected: &ExpectedCustodyTarget,
    passphrase: &CustodyPassphrase,
) -> Result<(), CustodyError> {
    match expected {
        ExpectedCustodyTarget::Absent => {
            match open_file_in_raw(directory, name, OFlags::RDONLY, Mode::empty()) {
                Err(rustix::io::Errno::NOENT) => Ok(()),
                _ => Err(CustodyError::RecoveryConflict),
            }
        }
        ExpectedCustodyTarget::Present(expected_reference) => {
            let opened = open_store_in(directory, name, passphrase)?;
            if opened.snapshot_reference() == *expected_reference {
                Ok(())
            } else {
                Err(CustodyError::RecoveryConflict)
            }
        }
    }
}

fn publish_candidate(
    source: &Path,
    target: &Path,
    no_replace: bool,
    lock: &LockGuard,
) -> Result<(), CustodyError> {
    let directory = source
        .parent()
        .filter(|directory| Some(*directory) == target.parent())
        .ok_or(CustodyError::UnsupportedStorageProfile)?;
    lock.revalidate(directory)?;
    let directory_file = &lock.directory;
    let source_name = source
        .file_name()
        .ok_or(CustodyError::UnsupportedStorageProfile)?;
    let target_name = target
        .file_name()
        .ok_or(CustodyError::UnsupportedStorageProfile)?;
    let candidate = openat2(
        directory_file,
        source_name,
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
        RESOLVE_WITHIN_DIRECTORY,
    )
    .map(File::from)
    .map_err(|_| CustodyError::Io)?;
    let candidate_metadata = candidate.metadata().map_err(io_error)?;
    validate_file_metadata(&candidate_metadata)?;
    let flags = if no_replace {
        RenameFlags::NOREPLACE
    } else {
        RenameFlags::empty()
    };
    renameat_with(
        directory_file,
        source_name,
        directory_file,
        target_name,
        flags,
    )
    .map_err(|_| CustodyError::Io)?;
    let published = openat2(
        directory_file,
        target_name,
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
        RESOLVE_WITHIN_DIRECTORY,
    )
    .map(File::from)
    .map_err(|_| CustodyError::CommitIndeterminate)?;
    let published_metadata = published.metadata().map_err(io_error)?;
    if (candidate_metadata.dev(), candidate_metadata.ino())
        != (published_metadata.dev(), published_metadata.ino())
    {
        return Err(CustodyError::CommitIndeterminate);
    }
    Ok(())
}

fn publish_opaque_candidate(
    directory: &File,
    source_name: &OsStr,
    target_name: &OsStr,
    candidate: &File,
) -> Result<(), CustodyError> {
    let candidate_metadata = candidate.metadata().map_err(io_error)?;
    validate_file_metadata(&candidate_metadata)?;
    renameat_with(
        directory,
        source_name,
        directory,
        target_name,
        RenameFlags::NOREPLACE,
    )
    .map_err(|_| CustodyError::CommitIndeterminate)?;
    let published = open_file_in_raw(directory, target_name, OFlags::RDONLY, Mode::empty())
        .map_err(|_| CustodyError::CommitIndeterminate)?;
    let published_metadata = published
        .metadata()
        .map_err(|_| CustodyError::CommitIndeterminate)?;
    validate_file_metadata(&published_metadata).map_err(|_| CustodyError::CommitIndeterminate)?;
    if (candidate_metadata.dev(), candidate_metadata.ino())
        != (published_metadata.dev(), published_metadata.ino())
    {
        return Err(CustodyError::CommitIndeterminate);
    }
    Ok(())
}

fn backup_name(reference: &CustodySnapshotReference) -> String {
    format!(
        "{BACKUP_PREFIX}{}.g{}.{}.pks1",
        hex::encode(reference.file_id),
        reference.generation,
        hex::encode(reference.digest)
    )
}

fn restore_source_name(reference: &CustodySnapshotReference) -> String {
    format!(
        "{RESTORE_PREFIX}{}.g{}.{}.pks1",
        hex::encode(reference.file_id),
        reference.generation,
        hex::encode(reference.digest)
    )
}

fn commit_opaque_copy(
    directory: &Path,
    temporary_prefix: &str,
    final_name: &str,
    raw: &[u8],
) -> Result<PathBuf, CustodyError> {
    if raw.is_empty() || raw.len() > MAX_FILE_BYTES {
        return Err(CustodyError::KeystoreOpenFailed);
    }
    prepare_existing_directory(directory)?;
    let directory_file = open_qualified_directory(directory)?;
    let final_path = directory.join(final_name);
    match open_file_in_raw(
        &directory_file,
        OsStr::new(final_name),
        OFlags::RDONLY,
        Mode::empty(),
    ) {
        Ok(final_file) => {
            validate_file_metadata(&final_file.metadata().map_err(io_error)?)?;
            if read_open_file(&final_file)? == raw {
                return Ok(final_path);
            }
            return Err(CustodyError::RecoveryConflict);
        }
        Err(rustix::io::Errno::NOENT) => {}
        Err(_) => return Err(CustodyError::Io),
    }
    for name in directory_entry_names(&directory_file)? {
        let Some(name) = name.to_str() else {
            return Err(CustodyError::RecoveryConflict);
        };
        if name.starts_with(temporary_prefix) {
            return Err(CustodyError::CommitIndeterminate);
        }
    }
    let temporary_name = format!("{temporary_prefix}{}", hex::encode(random_16()?));
    let temporary_path = directory.join(temporary_name);
    let temporary_file = write_new_file_in(
        &directory_file,
        temporary_path
            .file_name()
            .ok_or(CustodyError::UnsupportedStorageProfile)?,
        raw,
    )?;
    if read_open_file(&temporary_file)? != raw {
        unlinkat(
            &directory_file,
            temporary_path
                .file_name()
                .ok_or(CustodyError::UnsupportedStorageProfile)?,
            AtFlags::empty(),
        )
        .map_err(|_| CustodyError::CommitIndeterminate)?;
        directory_file
            .sync_all()
            .map_err(|_| CustodyError::CommitIndeterminate)?;
        return Err(CustodyError::Aborted);
    }
    publish_opaque_candidate(
        &directory_file,
        temporary_path
            .file_name()
            .ok_or(CustodyError::UnsupportedStorageProfile)?,
        OsStr::new(final_name),
        &temporary_file,
    )?;
    directory_file
        .sync_all()
        .map_err(|_| CustodyError::CommitIndeterminate)?;
    let final_file = open_file_in_raw(
        &directory_file,
        OsStr::new(final_name),
        OFlags::RDONLY,
        Mode::empty(),
    )
    .map_err(|_| CustodyError::CommitIndeterminate)?;
    let final_metadata = final_file
        .metadata()
        .map_err(|_| CustodyError::CommitIndeterminate)?;
    validate_file_metadata(&final_metadata).map_err(|_| CustodyError::CommitIndeterminate)?;
    if read_open_file(&final_file).map_err(|_| CustodyError::CommitIndeterminate)? != raw {
        return Err(CustodyError::CommitIndeterminate);
    }
    Ok(final_path)
}

fn list_candidates_in(
    directory: &Path,
    directory_file: &File,
) -> Result<Vec<PathBuf>, CustodyError> {
    let mut candidates = Vec::new();
    for name in directory_entry_names(directory_file)? {
        let Some(name) = name.to_str() else {
            return Err(CustodyError::RecoveryConflict);
        };
        if let Some(suffix) = name.strip_prefix(TEMP_PREFIX) {
            if suffix.len() != 32
                || !suffix
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                return Err(CustodyError::RecoveryConflict);
            }
            candidates.push(directory.join(name));
        }
    }
    candidates.sort();
    Ok(candidates)
}

fn list_restore_sources(directory: &Path) -> Result<Vec<PathBuf>, CustodyError> {
    let directory_file = open_qualified_directory(directory)?;
    let mut sources = Vec::new();
    for name in directory_entry_names(&directory_file)? {
        let Some(name) = name.to_str() else {
            return Err(CustodyError::RecoveryConflict);
        };
        if name.starts_with(RESTORE_TEMP_PREFIX) {
            continue;
        }
        if name.starts_with(RESTORE_PREFIX) {
            let path = directory.join(name);
            validate_candidate_in(&directory_file, &path)?;
            sources.push(path);
        }
    }
    sources.sort();
    Ok(sources)
}

fn cleanup_restore_temps(directory: &Path) -> Result<(), CustodyError> {
    let directory_file = open_qualified_directory(directory)?;
    let directory_metadata = directory_file.metadata().map_err(io_error)?;
    let mut removed = false;
    for name in directory_entry_names(&directory_file)? {
        let Some(name_text) = name.to_str() else {
            return Err(CustodyError::RecoveryConflict);
        };
        let Some(suffix) = name_text.strip_prefix(RESTORE_TEMP_PREFIX) else {
            continue;
        };
        if suffix.len() != 32
            || !suffix
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(CustodyError::RecoveryConflict);
        }
        let temporary = open_file_in(&directory_file, &name, OFlags::RDONLY, Mode::empty())?;
        let metadata = temporary.metadata().map_err(io_error)?;
        validate_file_metadata(&metadata)?;
        if metadata.dev() != directory_metadata.dev() {
            return Err(CustodyError::RecoveryConflict);
        }
        unlinkat(&directory_file, &name, AtFlags::empty()).map_err(|_| CustodyError::Io)?;
        removed = true;
    }
    if removed {
        directory_file.sync_all().map_err(io_error)?;
    }
    Ok(())
}

fn validate_candidate_in(directory: &File, path: &Path) -> Result<(), CustodyError> {
    let name = path.file_name().ok_or(CustodyError::RecoveryConflict)?;
    let file = open_file_in(directory, name, OFlags::RDONLY, Mode::empty())?;
    let metadata = file.metadata().map_err(io_error)?;
    let directory_metadata = directory.metadata().map_err(io_error)?;
    validate_file_metadata(&metadata)?;
    if metadata.dev() != directory_metadata.dev() {
        return Err(CustodyError::RecoveryConflict);
    }
    Ok(())
}

fn ensure_no_candidates(directory: &Path, directory_file: &File) -> Result<(), CustodyError> {
    if list_candidates_in(directory, directory_file)?.is_empty() {
        Ok(())
    } else {
        Err(CustodyError::CommitIndeterminate)
    }
}

fn ensure_ready_for_normal_operation(
    directory: &Path,
    directory_file: &File,
) -> Result<(), CustodyError> {
    ensure_no_candidates(directory, directory_file)?;
    if has_restore_intent(directory, directory_file)? {
        Err(CustodyError::CommitIndeterminate)
    } else {
        Ok(())
    }
}

fn has_restore_intent(directory: &Path, directory_file: &File) -> Result<bool, CustodyError> {
    if named_file_exists(directory_file, OsStr::new(LIVE_NAME))? {
        return Ok(false);
    }
    let quarantine = directory.join(RESTORE_DIRECTORY);
    if !quarantine.try_exists().map_err(io_error)? {
        return Ok(false);
    }
    prepare_existing_directory(&quarantine)?;
    Ok(!list_restore_sources(&quarantine)?.is_empty())
}

fn io_error(_: std::io::Error) -> CustodyError {
    CustodyError::Io
}

/// Redacted participant-custody failure contract.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CustodyError {
    /// Passphrase violates the exact printable-ASCII bound.
    #[error("invalid participant custody passphrase")]
    InvalidPassphrase,
    /// OS entropy could not be obtained; no candidate may be emitted.
    #[error("participant custody entropy unavailable")]
    EntropyUnavailable,
    /// Required bounded memory could not be allocated.
    #[error("participant custody resource unavailable")]
    ResourceUnavailable,
    /// Platform, filesystem, path, or metadata is outside the qualified profile.
    #[error("unsupported participant custody storage profile")]
    UnsupportedStorageProfile,
    /// Another writer holds the stable custody lock.
    #[error("participant custody writer is busy")]
    WriterBusy,
    /// Snapshot or nested record violates the closed format.
    #[error("invalid participant custody snapshot")]
    InvalidSnapshot,
    /// Existing encrypted bytes could not be opened or did not match expected identity.
    #[error("participant keystore open failed")]
    KeystoreOpenFailed,
    /// Durable prepared bytes are missing or inconsistent.
    #[error("invalid participant custody prepared state")]
    InvalidPreparedState,
    /// Supplied ledger state conflicts with the custody identity or material.
    #[error("participant custody conflicts with verified ledger state")]
    LedgerConflict,
    /// Restore installation requires receipt-backed current ledger truth.
    #[error("participant custody restore requires a network-converged ledger prefix")]
    NetworkConvergenceRequired,
    /// One or more ledger-required keys are unavailable locally.
    #[error("participant custody gap")]
    CustodyGap,
    /// Current profile does not activate this operation.
    #[error("participant custody operation is unsupported by the profile")]
    UnsupportedProfile,
    /// Protected content violates the fixed nonempty bound.
    #[error("invalid protected content")]
    InvalidProtectedContent,
    /// Secret-side construction or opening failed without an oracle detail.
    #[error("protected data open failed")]
    ProtectedDataOpenFailed,
    /// Definite pre-publication failure; the candidate was durably removed.
    #[error("participant custody mutation aborted")]
    Aborted,
    /// Rename may have happened but the directory commit was not observed.
    #[error("participant custody commit indeterminate")]
    CommitIndeterminate,
    /// Directory commit succeeded but authenticated reopen did not complete.
    #[error("participant custody committed but quarantined")]
    CommittedButQuarantined,
    /// Startup found an ambiguous or unsafe live/candidate combination.
    #[error("participant custody recovery conflict")]
    RecoveryConflict,
    /// A durable restore source reconciled to no installable live entry.
    #[error("participant custody restore has no installable state")]
    RestoreNoInstallableState,
    /// Redacted local I/O failure.
    #[error("participant custody I/O failure")]
    Io,
    /// Public privacy protocol failure without private diagnostic material.
    #[error("participant custody public protocol failure")]
    PublicProtocol,
}

impl From<PrivacyError> for CustodyError {
    fn from(_: PrivacyError) -> Self {
        Self::PublicProtocol
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_decoder_rejects_out_of_order_entries() {
        let principal = Uuid::from_u128(9);
        let authorization =
            KeyMaterial::authorization(principal, 1, Zeroizing::new([1; 32])).unwrap();
        let wrapping = KeyMaterial::wrapping(principal, 1, Zeroizing::new([2; 32])).unwrap();
        let entries = BTreeMap::from([
            (authorization.sort_key(), CustodyEntry::Bound(authorization)),
            (wrapping.sort_key(), CustodyEntry::Bound(wrapping)),
        ]);
        let encoded = Snapshot {
            network_id: "provchain.issue9".to_string(),
            principal,
            entries,
        }
        .encode()
        .unwrap();
        assert!(Snapshot::decode(&encoded).is_ok());

        let snapshot_fields = pk_decode_record(&encoded, SNAPSHOT_TAG, 3).unwrap();
        let list_fields = pk_decode_record(snapshot_fields[2], ENTRY_LIST_TAG, 2).unwrap();
        let first_length = u32::from_be_bytes(fixed::<4>(&list_fields[1][..4]).unwrap()) as usize;
        let first_end = 4 + first_length;
        let mut reversed = list_fields[1][first_end..].to_vec();
        reversed.extend_from_slice(&list_fields[1][..first_end]);
        let list =
            pk_encode_record(ENTRY_LIST_TAG, vec![list_fields[0].to_vec(), reversed]).unwrap();
        let out_of_order = pk_encode_record(
            SNAPSHOT_TAG,
            vec![
                snapshot_fields[0].to_vec(),
                snapshot_fields[1].to_vec(),
                list,
            ],
        )
        .unwrap();
        assert!(matches!(
            Snapshot::decode(&out_of_order),
            Err(CustodyError::InvalidSnapshot)
        ));
    }

    #[test]
    fn prepared_request_revalidates_authorizer_and_possession_contract() {
        let bootstrap = SigningKey::from_bytes(&[3; 32]);
        let profile =
            PrivacyLifecycleProfile::new([4; 32], bootstrap.verifying_key().to_bytes()).unwrap();
        let principal = Uuid::from_u128(10);
        let material = KeyMaterial::authorization(principal, 1, Zeroizing::new([5; 32])).unwrap();
        let anchor = PrivacyAdmissionAnchor::genesis(
            "provchain.issue9",
            "issue9.reference",
            profile.network_profile_content_hash(),
            0,
            1,
        )
        .unwrap();
        let unsigned = UnsignedPrivacyTransition::register_principal(
            &profile,
            anchor,
            principal,
            material.public_key.as_slice().try_into().unwrap(),
        )
        .unwrap();
        let possession = SigningKey::from_bytes(&[5; 32])
            .sign(&unsigned.possession_digest().unwrap())
            .to_bytes();
        let request = PreparedRequest::new(
            0x01,
            profile.bootstrap_reference_bytes(),
            unsigned,
            0x01,
            possession,
        )
        .unwrap();
        request.validate_material(&material).unwrap();
        assert!(PreparedRequest::decode(&request.encode()).is_ok());

        let mut wrong_authorizer = request.clone();
        wrong_authorizer.authorizer_tag = 0x02;
        assert_eq!(
            wrong_authorizer.validate_contract().unwrap_err(),
            CustodyError::InvalidPreparedState
        );
        let mut wrong_possession = request;
        wrong_possession.possession_proof[0] ^= 1;
        assert_eq!(
            wrong_possession.validate_material(&material).unwrap_err(),
            CustodyError::InvalidPreparedState
        );
    }

    #[test]
    fn stable_lock_also_excludes_same_process_writers() {
        let root = tempfile::tempdir().unwrap();
        let directory = root.path().join("custody");
        prepare_directory(&directory).unwrap();
        let first = acquire_lock(&directory).unwrap();
        assert!(matches!(
            acquire_lock(&directory),
            Err(CustodyError::WriterBusy)
        ));
        drop(first);
        assert!(acquire_lock(&directory).is_ok());
    }

    #[test]
    fn open_error_redaction_preserves_operational_recovery_categories() {
        for error in [
            CustodyError::Io,
            CustodyError::ResourceUnavailable,
            CustodyError::UnsupportedStorageProfile,
            CustodyError::RecoveryConflict,
            CustodyError::NetworkConvergenceRequired,
        ] {
            assert_eq!(redact_keystore_open_error(error.clone()), error);
            assert_eq!(redact_recovery_open_error(error.clone()), error);
        }
        assert_eq!(
            redact_keystore_open_error(CustodyError::ProtectedDataOpenFailed),
            CustodyError::KeystoreOpenFailed
        );
        assert_eq!(
            redact_recovery_open_error(CustodyError::InvalidSnapshot),
            CustodyError::RecoveryConflict
        );
    }
}
