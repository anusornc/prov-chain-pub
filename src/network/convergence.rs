//! Exact committed-envelope replication and receipt-backed convergence evidence.
//!
//! This module deliberately owns evidence, not commit authority. A producer may
//! expose an envelope only after it appears in its fsynced Ledger Journal. A
//! follower verifies those exact bytes and still commits them through Ledger
//! Final Admission. Node Identity Keys sign Commit Receipts only after the
//! receiver can read the corresponding durable local frame.

use std::collections::BTreeSet;
use std::str;

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use thiserror::Error;
use uuid::Uuid;

use crate::ledger::{AdmittedBlockEnvelope, Ledger, LedgerError, LedgerHash, MAX_ENVELOPE_BYTES};
use crate::network::canonical::hash_domain_separated_parts;
use crate::network::membership::{MemberRole, MemberStatus, SignedMembershipManifest};
use crate::network::peer_session::PeerSessionError;
use crate::network::poa::PoAError;

const RECORD_MAGIC: &[u8] = b"PROVCHAIN_COMMITTED_ENVELOPE_RECORD_V1";
const CHECKPOINT_MAGIC: &[u8] = b"PROVCHAIN_LEDGER_PREFIX_CHECKPOINT_V1";
const RANGE_REQUEST_MAGIC: &[u8] = b"PROVCHAIN_COMMITTED_RANGE_REQUEST_V1";
const RANGE_MAGIC: &[u8] = b"PROVCHAIN_COMMITTED_ENVELOPE_RANGE_V1";
const RECEIPT_MAGIC: &[u8] = b"PROVCHAIN_COMMIT_RECEIPT_V1";
const PRIVACY_RECEIPT_MAGIC: &[u8] = b"PROVCHAIN_PRIVACY_STATE_RECEIPT_V1";
const FORMAT_VERSION: u16 = 1;
const MAX_CONTRACT_TEXT_BYTES: usize = 4_096;
const PREFIX_GENESIS_DOMAIN: &[u8] = b"provchain/ledger-prefix-genesis/v1";
const PREFIX_STEP_DOMAIN: &[u8] = b"provchain/ledger-prefix-step/v1";
const RECEIPT_SIGNATURE_DOMAIN: &[u8] = b"provchain/commit-receipt-signature/v1";
const PRIVACY_RECEIPT_SIGNATURE_DOMAIN: &[u8] = b"provchain/privacy-state-receipt-signature/v1";

/// The pinned reference topology required by Issue #6.
pub const REQUIRED_REFERENCE_RECEIPTS: usize = 3;

/// Maximum envelopes served or admitted by one catch-up exchange.
pub const MAX_COMMITTED_RANGE_RECORDS: usize = 64;

const MAX_COMMIT_RECEIPT_BYTES: usize =
    RECEIPT_MAGIC.len() + 2 + 16 + (4 + MAX_CONTRACT_TEXT_BYTES) * 3 + 8 + 32 + 8 + 32 + 32 + 64;

/// Exact committed envelope bytes plus their ordered-ledger prefix commitment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommittedEnvelopeRecord {
    envelope_bytes: Vec<u8>,
    index: u64,
    envelope_hash: LedgerHash,
    ledger_prefix_hash: LedgerHash,
}

impl CommittedEnvelopeRecord {
    fn new(
        envelope_bytes: Vec<u8>,
        ledger_prefix_hash: LedgerHash,
    ) -> Result<Self, ConvergenceError> {
        let envelope = AdmittedBlockEnvelope::decode(&envelope_bytes)?;
        Ok(Self {
            index: envelope.index,
            envelope_hash: envelope.envelope_hash,
            envelope_bytes,
            ledger_prefix_hash,
        })
    }

    /// Position of this envelope in the journal-derived ledger.
    pub fn index(&self) -> u64 {
        self.index
    }

    /// Exact canonical envelope bytes committed by the producer.
    pub fn envelope_bytes(&self) -> &[u8] {
        &self.envelope_bytes
    }

    /// Cryptographic identity carried by the exact envelope.
    pub fn envelope_hash(&self) -> LedgerHash {
        self.envelope_hash
    }

    /// Commitment to the exact ordered prefix ending at this envelope.
    pub fn ledger_prefix_hash(&self) -> LedgerHash {
        self.ledger_prefix_hash
    }

    /// Encode one bounded record for transport or durable retry queues.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes =
            Vec::with_capacity(RECORD_MAGIC.len() + 2 + 4 + self.envelope_bytes.len() + 32 + 32);
        bytes.extend_from_slice(RECORD_MAGIC);
        bytes.extend_from_slice(&FORMAT_VERSION.to_be_bytes());
        bytes.extend_from_slice(&(self.envelope_bytes.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&self.envelope_bytes);
        bytes.extend_from_slice(&self.envelope_hash);
        bytes.extend_from_slice(&self.ledger_prefix_hash);
        bytes
    }

    /// Decode and authenticate one canonical committed-envelope record.
    pub fn decode(bytes: &[u8]) -> Result<Self, ConvergenceError> {
        let max = RECORD_MAGIC.len() + 2 + 4 + MAX_ENVELOPE_BYTES + 32 + 32;
        if bytes.len() > max {
            return Err(ConvergenceError::MalformedRecord(format!(
                "record exceeds {max} bytes"
            )));
        }
        let mut reader = CanonicalReader::new(bytes);
        reader.magic(RECORD_MAGIC, "committed-envelope record")?;
        reader.version()?;
        let envelope_bytes = reader.bytes(MAX_ENVELOPE_BYTES, "envelope")?;
        let encoded_envelope_hash = reader.fixed::<32>("envelope hash")?;
        let ledger_prefix_hash = reader.fixed::<32>("ledger prefix hash")?;
        reader.finish()?;
        let record = Self::new(envelope_bytes, ledger_prefix_hash)?;
        if record.envelope_hash != encoded_envelope_hash {
            return Err(ConvergenceError::MalformedRecord(
                "record envelope hash does not match exact envelope bytes".to_string(),
            ));
        }
        if record.canonical_bytes() != bytes {
            return Err(ConvergenceError::NonCanonicalEvidence);
        }
        Ok(record)
    }
}

/// Journal-derived anchor from which a peer requests its next contiguous range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LedgerPrefixCheckpoint {
    next_index: u64,
    ledger_prefix_hash: LedgerHash,
}

impl LedgerPrefixCheckpoint {
    /// First ledger position absent from the local durable prefix.
    pub fn next_index(&self) -> u64 {
        self.next_index
    }

    /// Commitment to every exact envelope before `next_index`.
    pub fn ledger_prefix_hash(&self) -> LedgerHash {
        self.ledger_prefix_hash
    }

    /// Encode the fixed checkpoint for peer transport.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(CHECKPOINT_MAGIC.len() + 2 + 8 + 32);
        bytes.extend_from_slice(CHECKPOINT_MAGIC);
        bytes.extend_from_slice(&FORMAT_VERSION.to_be_bytes());
        bytes.extend_from_slice(&self.next_index.to_be_bytes());
        bytes.extend_from_slice(&self.ledger_prefix_hash);
        bytes
    }

    /// Decode one canonical prefix checkpoint.
    pub fn decode(bytes: &[u8]) -> Result<Self, ConvergenceError> {
        let mut reader = CanonicalReader::new(bytes);
        reader.magic(CHECKPOINT_MAGIC, "ledger prefix checkpoint")?;
        reader.version()?;
        let checkpoint = Self {
            next_index: reader.u64("checkpoint next index")?,
            ledger_prefix_hash: reader.fixed::<32>("checkpoint prefix hash")?,
        };
        reader.finish()?;
        if checkpoint.canonical_bytes() != bytes {
            return Err(ConvergenceError::NonCanonicalEvidence);
        }
        Ok(checkpoint)
    }
}

/// Bounded request for a contiguous committed-envelope range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommittedRangeRequest {
    start_index: u64,
    expected_previous_prefix_hash: LedgerHash,
    max_records: usize,
}

impl CommittedRangeRequest {
    /// Request records immediately after a locally verified checkpoint.
    pub fn new(
        checkpoint: LedgerPrefixCheckpoint,
        max_records: usize,
    ) -> Result<Self, ConvergenceError> {
        if max_records == 0 || max_records > MAX_COMMITTED_RANGE_RECORDS {
            return Err(ConvergenceError::InvalidRangeLimit(max_records));
        }
        Ok(Self {
            start_index: checkpoint.next_index,
            expected_previous_prefix_hash: checkpoint.ledger_prefix_hash,
            max_records,
        })
    }

    /// First requested ledger position.
    pub fn start_index(&self) -> u64 {
        self.start_index
    }

    /// Requester's commitment to the prefix immediately before `start_index`.
    pub fn expected_previous_prefix_hash(&self) -> LedgerHash {
        self.expected_previous_prefix_hash
    }

    /// Maximum number of records the responder may return.
    pub fn max_records(&self) -> usize {
        self.max_records
    }

    /// Encode the bounded request for peer transport.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(RANGE_REQUEST_MAGIC.len() + 2 + 8 + 32 + 2);
        bytes.extend_from_slice(RANGE_REQUEST_MAGIC);
        bytes.extend_from_slice(&FORMAT_VERSION.to_be_bytes());
        bytes.extend_from_slice(&self.start_index.to_be_bytes());
        bytes.extend_from_slice(&self.expected_previous_prefix_hash);
        bytes.extend_from_slice(&(self.max_records as u16).to_be_bytes());
        bytes
    }

    /// Decode one canonical bounded range request.
    pub fn decode(bytes: &[u8]) -> Result<Self, ConvergenceError> {
        let mut reader = CanonicalReader::new(bytes);
        reader.magic(RANGE_REQUEST_MAGIC, "committed range request")?;
        reader.version()?;
        let start_index = reader.u64("range request start index")?;
        let expected_previous_prefix_hash =
            reader.fixed::<32>("range request previous prefix hash")?;
        let max_records =
            u16::from_be_bytes(reader.fixed::<2>("range request maximum records")?) as usize;
        reader.finish()?;
        if max_records == 0 || max_records > MAX_COMMITTED_RANGE_RECORDS {
            return Err(ConvergenceError::InvalidRangeLimit(max_records));
        }
        let request = Self {
            start_index,
            expected_previous_prefix_hash,
            max_records,
        };
        if request.canonical_bytes() != bytes {
            return Err(ConvergenceError::NonCanonicalEvidence);
        }
        Ok(request)
    }
}

/// One bounded contiguous exact-envelope catch-up response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommittedEnvelopeRange {
    start_index: u64,
    previous_prefix_hash: LedgerHash,
    records: Vec<CommittedEnvelopeRecord>,
}

impl CommittedEnvelopeRange {
    fn new(
        start_index: u64,
        previous_prefix_hash: LedgerHash,
        records: Vec<CommittedEnvelopeRecord>,
    ) -> Result<Self, ConvergenceError> {
        if records.len() > MAX_COMMITTED_RANGE_RECORDS {
            return Err(ConvergenceError::RangeTooLarge(records.len()));
        }
        let mut expected_index = start_index;
        let mut expected_prefix = previous_prefix_hash;
        for record in &records {
            if record.index != expected_index {
                return Err(ConvergenceError::NonContiguousRange {
                    expected_index,
                    received_index: record.index,
                });
            }
            expected_prefix =
                extend_ledger_prefix_hash(expected_prefix, record.index, record.envelope_bytes());
            if record.ledger_prefix_hash != expected_prefix {
                return Err(ConvergenceError::DivergentPrefix {
                    index: record.index,
                    reason: "range record does not extend the preceding exact prefix".to_string(),
                });
            }
            expected_index = expected_index.checked_add(1).ok_or_else(|| {
                ConvergenceError::MalformedRecord("range index overflow".to_string())
            })?;
        }
        Ok(Self {
            start_index,
            previous_prefix_hash,
            records,
        })
    }

    /// First ledger position in this response.
    pub fn start_index(&self) -> u64 {
        self.start_index
    }

    /// Prefix anchor immediately before the first returned record.
    pub fn previous_prefix_hash(&self) -> LedgerHash {
        self.previous_prefix_hash
    }

    /// Exact contiguous committed records, bounded by the protocol maximum.
    pub fn records(&self) -> &[CommittedEnvelopeRecord] {
        &self.records
    }

    /// Encode the bounded range for peer transport.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let encoded_records: Vec<_> = self
            .records
            .iter()
            .map(CommittedEnvelopeRecord::canonical_bytes)
            .collect();
        let capacity = RANGE_MAGIC.len()
            + 2
            + 8
            + 32
            + 2
            + encoded_records
                .iter()
                .map(|record| 4 + record.len())
                .sum::<usize>();
        let mut bytes = Vec::with_capacity(capacity);
        bytes.extend_from_slice(RANGE_MAGIC);
        bytes.extend_from_slice(&FORMAT_VERSION.to_be_bytes());
        bytes.extend_from_slice(&self.start_index.to_be_bytes());
        bytes.extend_from_slice(&self.previous_prefix_hash);
        bytes.extend_from_slice(&(encoded_records.len() as u16).to_be_bytes());
        for record in encoded_records {
            bytes.extend_from_slice(&(record.len() as u32).to_be_bytes());
            bytes.extend_from_slice(&record);
        }
        bytes
    }

    /// Decode and verify one bounded, contiguous canonical range.
    pub fn decode(bytes: &[u8]) -> Result<Self, ConvergenceError> {
        let max_record_bytes = RECORD_MAGIC.len() + 2 + 4 + MAX_ENVELOPE_BYTES + 32 + 32;
        let max_range_bytes = RANGE_MAGIC.len()
            + 2
            + 8
            + 32
            + 2
            + MAX_COMMITTED_RANGE_RECORDS * (4 + max_record_bytes);
        if bytes.len() > max_range_bytes {
            return Err(ConvergenceError::RangeTooLarge(
                MAX_COMMITTED_RANGE_RECORDS + 1,
            ));
        }
        let mut reader = CanonicalReader::new(bytes);
        reader.magic(RANGE_MAGIC, "committed-envelope range")?;
        reader.version()?;
        let start_index = reader.u64("range start index")?;
        let previous_prefix_hash = reader.fixed::<32>("range previous prefix hash")?;
        let record_count = u16::from_be_bytes(reader.fixed::<2>("range record count")?) as usize;
        if record_count > MAX_COMMITTED_RANGE_RECORDS {
            return Err(ConvergenceError::RangeTooLarge(record_count));
        }
        let mut records = Vec::with_capacity(record_count);
        for _ in 0..record_count {
            let record_bytes = reader.bytes(max_record_bytes, "committed record")?;
            records.push(CommittedEnvelopeRecord::decode(&record_bytes)?);
        }
        reader.finish()?;
        let range = Self::new(start_index, previous_prefix_hash, records)?;
        if range.canonical_bytes() != bytes {
            return Err(ConvergenceError::NonCanonicalEvidence);
        }
        Ok(range)
    }
}

/// Node Identity Key evidence emitted only for a locally durable journal frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitReceipt {
    node_id: Uuid,
    network_id: String,
    profile_id: String,
    manifest_id: String,
    manifest_version: u64,
    manifest_digest: LedgerHash,
    ledger_position: u64,
    envelope_hash: LedgerHash,
    ledger_prefix_hash: LedgerHash,
    node_signature: [u8; 64],
}

impl CommitReceipt {
    fn sign(
        node_id: Uuid,
        network_id: &str,
        profile_id: &str,
        manifest: &SignedMembershipManifest,
        record: &CommittedEnvelopeRecord,
        identity_key: &SigningKey,
    ) -> Result<Self, ConvergenceError> {
        validate_text(network_id, "network_id")?;
        validate_text(profile_id, "profile_id")?;
        validate_text(&manifest.manifest.manifest_id, "manifest_id")?;
        let mut receipt = Self {
            node_id,
            network_id: network_id.to_string(),
            profile_id: profile_id.to_string(),
            manifest_id: manifest.manifest.manifest_id.clone(),
            manifest_version: manifest.manifest.version,
            manifest_digest: manifest.digest(),
            ledger_position: record.index,
            envelope_hash: record.envelope_hash,
            ledger_prefix_hash: record.ledger_prefix_hash,
            node_signature: [0; 64],
        };
        receipt.node_signature = identity_key.sign(&receipt.signature_digest()).to_bytes();
        Ok(receipt)
    }

    /// Logical node whose dedicated Node Identity Key signed this receipt.
    pub fn node_id(&self) -> Uuid {
        self.node_id
    }

    /// Network identity bound into the evidence.
    pub fn network_id(&self) -> &str {
        &self.network_id
    }

    /// Network Profile identity bound into the evidence.
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    /// Governance manifest lineage bound into the evidence.
    pub fn manifest_id(&self) -> &str {
        &self.manifest_id
    }

    /// Governance manifest version bound into the evidence.
    pub fn manifest_version(&self) -> u64 {
        self.manifest_version
    }

    /// Canonical active-manifest digest bound into the evidence.
    pub fn manifest_digest(&self) -> LedgerHash {
        self.manifest_digest
    }

    /// Journal position acknowledged by this node.
    pub fn ledger_position(&self) -> u64 {
        self.ledger_position
    }

    /// Exact committed envelope identity acknowledged by this node.
    pub fn envelope_hash(&self) -> LedgerHash {
        self.envelope_hash
    }

    /// Exact ordered-ledger prefix acknowledged by this node.
    pub fn ledger_prefix_hash(&self) -> LedgerHash {
        self.ledger_prefix_hash
    }

    /// Node Identity Key signature over the canonical receipt digest.
    pub fn node_signature(&self) -> [u8; 64] {
        self.node_signature
    }

    /// Encode the complete signed receipt for durable storage or transport.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = self.unsigned_bytes();
        bytes.extend_from_slice(&self.node_signature);
        bytes
    }

    /// Decode one canonical receipt. Contract and signature verification occurs
    /// when a reference node evaluates convergence.
    pub fn decode(bytes: &[u8]) -> Result<Self, ConvergenceError> {
        if bytes.len() > MAX_COMMIT_RECEIPT_BYTES {
            return Err(ConvergenceError::MalformedReceipt(format!(
                "receipt exceeds {MAX_COMMIT_RECEIPT_BYTES} bytes"
            )));
        }
        let mut reader = CanonicalReader::new(bytes);
        reader.magic(RECEIPT_MAGIC, "commit receipt")?;
        reader.version()?;
        let node_id = Uuid::from_bytes(reader.fixed::<16>("node id")?);
        let network_id = reader.text("network_id")?;
        let profile_id = reader.text("profile_id")?;
        let manifest_id = reader.text("manifest_id")?;
        let manifest_version = reader.u64("manifest version")?;
        if manifest_version == 0 {
            return Err(ConvergenceError::MalformedReceipt(
                "manifest version must be nonzero".to_string(),
            ));
        }
        let manifest_digest = reader.fixed::<32>("manifest digest")?;
        let ledger_position = reader.u64("ledger position")?;
        let envelope_hash = reader.fixed::<32>("envelope hash")?;
        let ledger_prefix_hash = reader.fixed::<32>("ledger prefix hash")?;
        let node_signature = reader.fixed::<64>("node signature")?;
        reader.finish()?;
        if manifest_digest == [0; 32]
            || envelope_hash == [0; 32]
            || ledger_prefix_hash == [0; 32]
            || node_signature == [0; 64]
        {
            return Err(ConvergenceError::MalformedReceipt(
                "receipt contains empty cryptographic evidence".to_string(),
            ));
        }
        let receipt = Self {
            node_id,
            network_id,
            profile_id,
            manifest_id,
            manifest_version,
            manifest_digest,
            ledger_position,
            envelope_hash,
            ledger_prefix_hash,
            node_signature,
        };
        if receipt.canonical_bytes() != bytes {
            return Err(ConvergenceError::NonCanonicalEvidence);
        }
        Ok(receipt)
    }

    fn unsigned_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(RECEIPT_MAGIC);
        bytes.extend_from_slice(&FORMAT_VERSION.to_be_bytes());
        bytes.extend_from_slice(self.node_id.as_bytes());
        write_text(&mut bytes, &self.network_id);
        write_text(&mut bytes, &self.profile_id);
        write_text(&mut bytes, &self.manifest_id);
        bytes.extend_from_slice(&self.manifest_version.to_be_bytes());
        bytes.extend_from_slice(&self.manifest_digest);
        bytes.extend_from_slice(&self.ledger_position.to_be_bytes());
        bytes.extend_from_slice(&self.envelope_hash);
        bytes.extend_from_slice(&self.ledger_prefix_hash);
        bytes
    }

    fn signature_digest(&self) -> LedgerHash {
        hash_domain_separated_parts(RECEIPT_SIGNATURE_DOMAIN, &[&self.unsigned_bytes()])
    }
}

/// Node Identity Key evidence that binds the same durable envelope receipt to
/// the canonical Effective Privacy State digest at that exact prefix.
///
/// Issue #6's generic [`CommitReceipt`] remains unchanged. Privacy release
/// requires this additional signed statement from every pinned reference node,
/// so a matching envelope prefix cannot be mistaken for a matching privacy
/// projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivacyStateReceipt {
    commit_receipt: CommitReceipt,
    privacy_state_digest: LedgerHash,
    node_signature: [u8; 64],
}

impl PrivacyStateReceipt {
    fn sign(
        node_id: Uuid,
        network_id: &str,
        profile_id: &str,
        manifest: &SignedMembershipManifest,
        record: &CommittedEnvelopeRecord,
        privacy_state_digest: LedgerHash,
        identity_key: &SigningKey,
    ) -> Result<Self, ConvergenceError> {
        if privacy_state_digest == [0; 32] {
            return Err(ConvergenceError::MalformedReceipt(
                "privacy state digest cannot be empty".to_string(),
            ));
        }
        let commit_receipt = CommitReceipt::sign(
            node_id,
            network_id,
            profile_id,
            manifest,
            record,
            identity_key,
        )?;
        let mut receipt = Self {
            commit_receipt,
            privacy_state_digest,
            node_signature: [0; 64],
        };
        receipt.node_signature = identity_key.sign(&receipt.signature_digest()).to_bytes();
        Ok(receipt)
    }

    /// The generic durable-envelope receipt nested in this privacy receipt.
    pub fn commit_receipt(&self) -> &CommitReceipt {
        &self.commit_receipt
    }

    /// Logical node whose dedicated Node Identity Key signed this receipt.
    pub fn node_id(&self) -> Uuid {
        self.commit_receipt.node_id()
    }

    /// Network identity bound into the nested receipt.
    pub fn network_id(&self) -> &str {
        self.commit_receipt.network_id()
    }

    /// Network Profile identity bound into the nested receipt.
    pub fn profile_id(&self) -> &str {
        self.commit_receipt.profile_id()
    }

    /// Journal position acknowledged by this node.
    pub fn ledger_position(&self) -> u64 {
        self.commit_receipt.ledger_position()
    }

    /// Exact envelope identity acknowledged by this node.
    pub fn envelope_hash(&self) -> LedgerHash {
        self.commit_receipt.envelope_hash()
    }

    /// Exact ordered-ledger prefix acknowledged by this node.
    pub fn ledger_prefix_hash(&self) -> LedgerHash {
        self.commit_receipt.ledger_prefix_hash()
    }

    /// Canonical Effective Privacy State digest signed at this prefix.
    pub fn privacy_state_digest(&self) -> LedgerHash {
        self.privacy_state_digest
    }

    /// Dedicated Node Identity Key signature over the nested receipt and state digest.
    pub fn node_signature(&self) -> [u8; 64] {
        self.node_signature
    }

    /// Encode the complete nested privacy-state receipt.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let commit_receipt = self.commit_receipt.canonical_bytes();
        let mut bytes = Vec::with_capacity(
            PRIVACY_RECEIPT_MAGIC.len() + 2 + 4 + commit_receipt.len() + 32 + 64,
        );
        bytes.extend_from_slice(PRIVACY_RECEIPT_MAGIC);
        bytes.extend_from_slice(&FORMAT_VERSION.to_be_bytes());
        bytes.extend_from_slice(&(commit_receipt.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&commit_receipt);
        bytes.extend_from_slice(&self.privacy_state_digest);
        bytes.extend_from_slice(&self.node_signature);
        bytes
    }

    /// Decode one canonical privacy-state receipt.
    pub fn decode(bytes: &[u8]) -> Result<Self, ConvergenceError> {
        let max = PRIVACY_RECEIPT_MAGIC.len() + 2 + 4 + MAX_COMMIT_RECEIPT_BYTES + 32 + 64;
        if bytes.len() > max {
            return Err(ConvergenceError::MalformedReceipt(format!(
                "privacy state receipt exceeds {max} bytes"
            )));
        }
        let mut reader = CanonicalReader::new(bytes);
        reader.magic(PRIVACY_RECEIPT_MAGIC, "privacy state receipt")?;
        reader.version()?;
        let commit_receipt = CommitReceipt::decode(
            &reader.bytes(MAX_COMMIT_RECEIPT_BYTES, "nested commit receipt")?,
        )?;
        let privacy_state_digest = reader.fixed::<32>("privacy state digest")?;
        let node_signature = reader.fixed::<64>("privacy state receipt signature")?;
        reader.finish()?;
        if privacy_state_digest == [0; 32] || node_signature == [0; 64] {
            return Err(ConvergenceError::MalformedReceipt(
                "privacy state receipt contains empty cryptographic evidence".to_string(),
            ));
        }
        let receipt = Self {
            commit_receipt,
            privacy_state_digest,
            node_signature,
        };
        if receipt.canonical_bytes() != bytes {
            return Err(ConvergenceError::NonCanonicalEvidence);
        }
        Ok(receipt)
    }

    fn signature_digest(&self) -> LedgerHash {
        hash_domain_separated_parts(
            PRIVACY_RECEIPT_SIGNATURE_DOMAIN,
            &[
                &self.commit_receipt.canonical_bytes(),
                &self.privacy_state_digest,
            ],
        )
    }
}

/// Verifiable evidence that all three pinned reference nodes committed one prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkConvergenceEvidence {
    ledger_position: u64,
    envelope_hash: LedgerHash,
    ledger_prefix_hash: LedgerHash,
    receipts: Vec<CommitReceipt>,
    privacy_state_digest: Option<LedgerHash>,
    privacy_receipts: Vec<PrivacyStateReceipt>,
}

impl NetworkConvergenceEvidence {
    /// Position proven identical by all reference nodes.
    pub fn ledger_position(&self) -> u64 {
        self.ledger_position
    }

    /// Exact envelope identity proven identical by all reference nodes.
    pub fn envelope_hash(&self) -> LedgerHash {
        self.envelope_hash
    }

    /// Exact ordered-prefix commitment proven identical by all reference nodes.
    pub fn ledger_prefix_hash(&self) -> LedgerHash {
        self.ledger_prefix_hash
    }

    /// Canonically node-sorted receipts forming the convergence evidence.
    pub fn receipts(&self) -> &[CommitReceipt] {
        &self.receipts
    }

    /// Canonical Effective Privacy State digest, when the evidence is suitable
    /// for the Issue #10 privacy release barrier.
    pub fn privacy_state_digest(&self) -> Option<LedgerHash> {
        self.privacy_state_digest
    }

    /// Per-node signed privacy-state receipts forming the Issue #10 barrier.
    pub fn privacy_receipts(&self) -> &[PrivacyStateReceipt] {
        &self.privacy_receipts
    }
}

/// Externally observable state that never conflates local commit with convergence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitStatus {
    /// The queried envelope is locally durable, but three matching receipts are absent.
    LocalCommitted {
        /// Number of valid, distinct matching receipts supplied so far.
        matching_receipts: usize,
        /// Fixed reference-node receipt count required for convergence.
        required_receipts: usize,
    },
    /// All three pinned nodes signed the identical envelope and ordered prefix.
    NetworkConverged {
        /// Complete receipt-backed evidence.
        evidence: NetworkConvergenceEvidence,
    },
}

/// Fail-closed replication, synchronization, or convergence-evidence failure.
#[derive(Debug, Error)]
pub enum ConvergenceError {
    /// The requested position is not committed in the local journal.
    #[error("ledger position {0} is not locally committed")]
    MissingCommittedEnvelope(u64),
    /// A received position skipped the local next position.
    #[error("replication gap: expected index {expected_index}, received {received_index}")]
    Gap {
        /// Local next position.
        expected_index: u64,
        /// Received record position.
        received_index: u64,
    },
    /// A catch-up request must remain within the fixed protocol bound.
    #[error("committed range limit must be between 1 and {MAX_COMMITTED_RANGE_RECORDS}, got {0}")]
    InvalidRangeLimit(usize),
    /// A response exceeded the fixed record-count bound.
    #[error("committed range contains {0} records, maximum is {MAX_COMMITTED_RANGE_RECORDS}")]
    RangeTooLarge(usize),
    /// A response did not match the request whose checkpoint authorized it.
    #[error("committed range response does not match request field: {0}")]
    RangeRequestMismatch(&'static str),
    /// A response exceeded the bound chosen by its originating request.
    #[error("committed range returned {received} records, request allowed {requested}")]
    RangeExceedsRequestedLimit {
        /// Maximum response length authorized by the request.
        requested: usize,
        /// Actual response length.
        received: usize,
    },
    /// Range positions must be contiguous and ordered.
    #[error(
        "non-contiguous committed range: expected index {expected_index}, received {received_index}"
    )]
    NonContiguousRange {
        /// Required next range position.
        expected_index: u64,
        /// Actual record position.
        received_index: u64,
    },
    /// Local and received history differ at a committed position or prefix anchor.
    #[error("divergent ledger prefix at index {index}: {reason}")]
    DivergentPrefix {
        /// First observed conflicting position.
        index: u64,
        /// Stable diagnostic detail.
        reason: String,
    },
    /// Replication evidence is malformed.
    #[error("malformed committed-envelope record: {0}")]
    MalformedRecord(String),
    /// Commit receipt evidence is malformed.
    #[error("malformed commit receipt: {0}")]
    MalformedReceipt(String),
    /// Evidence did not use the unique canonical representation.
    #[error("non-canonical convergence evidence")]
    NonCanonicalEvidence,
    /// A receipt does not bind the active contract and exact record.
    #[error("commit receipt from {node_id} does not match: {field}")]
    ReceiptMismatch {
        /// Receipt signer.
        node_id: Uuid,
        /// Mismatching contract field.
        field: &'static str,
    },
    /// One signer appeared more than once.
    #[error("duplicate commit receipt from node {0}")]
    DuplicateReceipt(Uuid),
    /// The receipt signer is not one of the three pinned reference validators.
    #[error("commit receipt signer {0} is not an active reference validator")]
    UnauthorizedReceiptSigner(Uuid),
    /// Node Identity Key signature verification failed.
    #[error("invalid Node Identity Key signature on receipt from {0}")]
    InvalidReceiptSignature(Uuid),
    /// Issue #6 intentionally supports exactly three reference validators.
    #[error("reference convergence requires exactly 3 validators, profile declares {0}")]
    InvalidReferenceTopology(usize),
    /// An authenticated transport belongs to another local node or manifest.
    #[error("authenticated peer transport does not match the active local contract")]
    TransportBindingMismatch,
    /// Peer-session quarantine rejected the message.
    #[error(transparent)]
    PeerSession(#[from] PeerSessionError),
    /// Universal Final Admission or journal replay failed.
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    /// Scheduled PoA validation or durable signing-fence handling failed.
    #[error(transparent)]
    PoA(#[from] PoAError),
}

/// Return the fixed virtual-genesis prefix commitment.
pub fn genesis_ledger_prefix_hash() -> LedgerHash {
    hash_domain_separated_parts(PREFIX_GENESIS_DOMAIN, &[])
}

/// Extend one prefix with the exact canonical bytes at a fixed position.
pub fn extend_ledger_prefix_hash(
    previous_prefix_hash: LedgerHash,
    index: u64,
    envelope_bytes: &[u8],
) -> LedgerHash {
    hash_domain_separated_parts(
        PREFIX_STEP_DOMAIN,
        &[&previous_prefix_hash, &index.to_be_bytes(), envelope_bytes],
    )
}

struct JournalCommittedSnapshot {
    records: Vec<CommittedEnvelopeRecord>,
    prefix_hashes: Vec<LedgerHash>,
}

impl JournalCommittedSnapshot {
    fn from_ledger(ledger: &Ledger) -> Result<Self, ConvergenceError> {
        let envelope_bytes = ledger.verified_committed_envelope_bytes()?;
        let mut records = Vec::with_capacity(envelope_bytes.len());
        let mut prefix_hashes = Vec::with_capacity(envelope_bytes.len() + 1);
        let mut prefix = genesis_ledger_prefix_hash();
        prefix_hashes.push(prefix);
        for (expected_index, bytes) in envelope_bytes.into_iter().enumerate() {
            let expected_index = u64::try_from(expected_index).map_err(|_| {
                ConvergenceError::MalformedRecord("local ledger length exceeds u64".to_string())
            })?;
            let envelope = AdmittedBlockEnvelope::decode(&bytes)?;
            if envelope.index != expected_index {
                return Err(ConvergenceError::MalformedRecord(format!(
                    "journal frame index {}, expected {expected_index}",
                    envelope.index
                )));
            }
            prefix = extend_ledger_prefix_hash(prefix, expected_index, &bytes);
            records.push(CommittedEnvelopeRecord::new(bytes, prefix)?);
            prefix_hashes.push(prefix);
        }
        Ok(Self {
            records,
            prefix_hashes,
        })
    }

    fn len_u64(&self) -> Result<u64, ConvergenceError> {
        u64::try_from(self.records.len()).map_err(|_| {
            ConvergenceError::MalformedRecord("local ledger length exceeds u64".to_string())
        })
    }

    fn prefix_before(&self, next_index: u64) -> Result<LedgerHash, ConvergenceError> {
        let local_len = self.len_u64()?;
        if next_index > local_len {
            return Err(ConvergenceError::Gap {
                expected_index: local_len,
                received_index: next_index,
            });
        }
        let next_index = usize::try_from(next_index).map_err(|_| {
            ConvergenceError::MalformedRecord("ledger index exceeds usize".to_string())
        })?;
        self.prefix_hashes.get(next_index).copied().ok_or_else(|| {
            ConvergenceError::MalformedRecord("ledger prefix index is unavailable".to_string())
        })
    }

    fn record(&self, index: u64) -> Result<&CommittedEnvelopeRecord, ConvergenceError> {
        usize::try_from(index)
            .ok()
            .and_then(|index| self.records.get(index))
            .ok_or(ConvergenceError::MissingCommittedEnvelope(index))
    }
}

pub(crate) fn ledger_prefix_checkpoint(
    ledger: &Ledger,
) -> Result<LedgerPrefixCheckpoint, ConvergenceError> {
    let snapshot = JournalCommittedSnapshot::from_ledger(ledger)?;
    let next_index = snapshot.len_u64()?;
    Ok(LedgerPrefixCheckpoint {
        next_index,
        ledger_prefix_hash: snapshot.prefix_before(next_index)?,
    })
}

pub(crate) fn export_committed_range(
    ledger: &Ledger,
    request: CommittedRangeRequest,
) -> Result<CommittedEnvelopeRange, ConvergenceError> {
    let snapshot = JournalCommittedSnapshot::from_ledger(ledger)?;
    let local_len = snapshot.len_u64()?;
    if request.start_index > local_len {
        return Err(ConvergenceError::Gap {
            expected_index: local_len,
            received_index: request.start_index,
        });
    }
    let actual_previous_prefix = snapshot.prefix_before(request.start_index)?;
    if actual_previous_prefix != request.expected_previous_prefix_hash {
        return Err(ConvergenceError::DivergentPrefix {
            index: request.start_index,
            reason: format!(
                "request anchor {}, local anchor {}",
                hex::encode(request.expected_previous_prefix_hash),
                hex::encode(actual_previous_prefix)
            ),
        });
    }
    let start = usize::try_from(request.start_index)
        .map_err(|_| ConvergenceError::MalformedRecord("range start exceeds usize".to_string()))?;
    let end = start
        .saturating_add(request.max_records)
        .min(snapshot.records.len());
    let records = snapshot.records[start..end].to_vec();
    CommittedEnvelopeRange::new(request.start_index, actual_previous_prefix, records)
}

pub(crate) fn validate_range_for_admission(
    ledger: &Ledger,
    request: CommittedRangeRequest,
    range: &CommittedEnvelopeRange,
) -> Result<(), ConvergenceError> {
    if range.start_index != request.start_index {
        return Err(ConvergenceError::RangeRequestMismatch("start_index"));
    }
    if range.previous_prefix_hash != request.expected_previous_prefix_hash {
        return Err(ConvergenceError::RangeRequestMismatch(
            "expected_previous_prefix_hash",
        ));
    }
    if range.records.len() > request.max_records {
        return Err(ConvergenceError::RangeExceedsRequestedLimit {
            requested: request.max_records,
            received: range.records.len(),
        });
    }
    // Construction and decode already validate record contiguity and each
    // prefix step; repeating it here protects values cloned in-process too.
    CommittedEnvelopeRange::new(
        range.start_index,
        range.previous_prefix_hash,
        range.records.clone(),
    )?;

    let snapshot = JournalCommittedSnapshot::from_ledger(ledger)?;
    let local_len = snapshot.len_u64()?;
    if range.start_index > local_len {
        return Err(ConvergenceError::Gap {
            expected_index: local_len,
            received_index: range.start_index,
        });
    }
    let local_anchor = snapshot.prefix_before(range.start_index)?;
    if local_anchor != range.previous_prefix_hash {
        return Err(ConvergenceError::DivergentPrefix {
            index: range.start_index,
            reason: format!(
                "range anchor {}, local anchor {}",
                hex::encode(range.previous_prefix_hash),
                hex::encode(local_anchor)
            ),
        });
    }
    for record in range
        .records
        .iter()
        .take_while(|record| record.index < local_len)
    {
        let local = snapshot.record(record.index)?;
        if local != record {
            return Err(ConvergenceError::DivergentPrefix {
                index: record.index,
                reason: format!(
                    "local envelope/prefix {} differs from received {}",
                    hex::encode(local.envelope_hash),
                    hex::encode(record.envelope_hash)
                ),
            });
        }
    }
    Ok(())
}

pub(crate) fn committed_record_at(
    ledger: &Ledger,
    index: u64,
) -> Result<CommittedEnvelopeRecord, ConvergenceError> {
    Ok(JournalCommittedSnapshot::from_ledger(ledger)?
        .record(index)?
        .clone())
}

pub(crate) fn committed_records_at(
    ledger: &Ledger,
    start_index: u64,
    record_count: usize,
) -> Result<Vec<CommittedEnvelopeRecord>, ConvergenceError> {
    let snapshot = JournalCommittedSnapshot::from_ledger(ledger)?;
    let start = usize::try_from(start_index)
        .map_err(|_| ConvergenceError::MalformedRecord("range start exceeds usize".to_string()))?;
    let end = start.checked_add(record_count).ok_or_else(|| {
        ConvergenceError::MalformedRecord("range record count overflow".to_string())
    })?;
    if end > snapshot.records.len() {
        return Err(ConvergenceError::MissingCommittedEnvelope(
            start_index.saturating_add(record_count.saturating_sub(1) as u64),
        ));
    }
    Ok(snapshot.records[start..end].to_vec())
}

pub(crate) fn validate_record_for_admission(
    ledger: &Ledger,
    record: &CommittedEnvelopeRecord,
) -> Result<(), ConvergenceError> {
    let snapshot = JournalCommittedSnapshot::from_ledger(ledger)?;
    let local_len = snapshot.len_u64()?;
    if record.index < local_len {
        let local = snapshot.record(record.index)?;
        if local == record {
            return Ok(());
        }
        return Err(ConvergenceError::DivergentPrefix {
            index: record.index,
            reason: format!(
                "local envelope/prefix {} differs from received {}",
                hex::encode(local.envelope_hash),
                hex::encode(record.envelope_hash)
            ),
        });
    }
    if record.index > local_len {
        return Err(ConvergenceError::Gap {
            expected_index: local_len,
            received_index: record.index,
        });
    }
    let previous_prefix = snapshot.prefix_before(local_len)?;
    let expected =
        extend_ledger_prefix_hash(previous_prefix, record.index, record.envelope_bytes());
    if record.ledger_prefix_hash != expected {
        return Err(ConvergenceError::DivergentPrefix {
            index: record.index,
            reason: format!(
                "received prefix {}, expected {}",
                hex::encode(record.ledger_prefix_hash),
                hex::encode(expected)
            ),
        });
    }
    Ok(())
}

pub(crate) fn issue_local_receipt(
    node_id: Uuid,
    network_id: &str,
    profile_id: &str,
    manifest: &SignedMembershipManifest,
    identity_key: &SigningKey,
    record: &CommittedEnvelopeRecord,
) -> Result<CommitReceipt, ConvergenceError> {
    CommitReceipt::sign(
        node_id,
        network_id,
        profile_id,
        manifest,
        record,
        identity_key,
    )
}

pub(crate) fn issue_local_privacy_state_receipt(
    node_id: Uuid,
    network_id: &str,
    profile_id: &str,
    manifest: &SignedMembershipManifest,
    identity_key: &SigningKey,
    record: &CommittedEnvelopeRecord,
    privacy_state_digest: LedgerHash,
) -> Result<PrivacyStateReceipt, ConvergenceError> {
    PrivacyStateReceipt::sign(
        node_id,
        network_id,
        profile_id,
        manifest,
        record,
        privacy_state_digest,
        identity_key,
    )
}

pub(crate) fn evaluate_commit_status(
    local_node_id: Uuid,
    network_id: &str,
    profile_id: &str,
    manifest: &SignedMembershipManifest,
    expected_node_ids: &[Uuid],
    record: &CommittedEnvelopeRecord,
    receipts: &[CommitReceipt],
) -> Result<CommitStatus, ConvergenceError> {
    if expected_node_ids.len() != REQUIRED_REFERENCE_RECEIPTS {
        return Err(ConvergenceError::InvalidReferenceTopology(
            expected_node_ids.len(),
        ));
    }
    let expected_nodes: BTreeSet<_> = expected_node_ids.iter().copied().collect();
    if expected_nodes.len() != REQUIRED_REFERENCE_RECEIPTS {
        return Err(ConvergenceError::InvalidReferenceTopology(
            expected_nodes.len(),
        ));
    }
    if !expected_nodes.contains(&local_node_id) {
        return Err(ConvergenceError::InvalidReferenceTopology(
            expected_nodes.len(),
        ));
    }

    let mut seen = BTreeSet::new();
    let mut verified = Vec::with_capacity(receipts.len());
    for receipt in receipts {
        verify_receipt_contract(receipt, network_id, profile_id, manifest, record)?;
        if !expected_nodes.contains(&receipt.node_id) {
            return Err(ConvergenceError::UnauthorizedReceiptSigner(receipt.node_id));
        }
        if !seen.insert(receipt.node_id) {
            return Err(ConvergenceError::DuplicateReceipt(receipt.node_id));
        }
        verify_receipt_signature(receipt, manifest)?;
        verified.push(receipt.clone());
    }

    if verified.len() < REQUIRED_REFERENCE_RECEIPTS {
        return Ok(CommitStatus::LocalCommitted {
            matching_receipts: verified.len(),
            required_receipts: REQUIRED_REFERENCE_RECEIPTS,
        });
    }
    if verified.len() > REQUIRED_REFERENCE_RECEIPTS {
        return Err(ConvergenceError::MalformedReceipt(
            "receipt set exceeds the pinned reference topology".to_string(),
        ));
    }
    verified.sort_by_key(|receipt| *receipt.node_id.as_bytes());
    Ok(CommitStatus::NetworkConverged {
        evidence: NetworkConvergenceEvidence {
            ledger_position: record.index,
            envelope_hash: record.envelope_hash,
            ledger_prefix_hash: record.ledger_prefix_hash,
            receipts: verified,
            privacy_state_digest: None,
            privacy_receipts: Vec::new(),
        },
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn evaluate_privacy_commit_status(
    local_node_id: Uuid,
    network_id: &str,
    profile_id: &str,
    manifest: &SignedMembershipManifest,
    expected_node_ids: &[Uuid],
    record: &CommittedEnvelopeRecord,
    receipts: &[CommitReceipt],
    privacy_receipts: &[PrivacyStateReceipt],
) -> Result<CommitStatus, ConvergenceError> {
    let status = evaluate_commit_status(
        local_node_id,
        network_id,
        profile_id,
        manifest,
        expected_node_ids,
        record,
        receipts,
    )?;
    let CommitStatus::NetworkConverged { mut evidence } = status else {
        return Ok(status);
    };
    if privacy_receipts.len() != REQUIRED_REFERENCE_RECEIPTS {
        return Err(ConvergenceError::MalformedReceipt(format!(
            "privacy convergence requires exactly {REQUIRED_REFERENCE_RECEIPTS} privacy-state receipts"
        )));
    }
    let expected_nodes: BTreeSet<_> = expected_node_ids.iter().copied().collect();
    let mut seen = BTreeSet::new();
    let mut verified = Vec::with_capacity(privacy_receipts.len());
    let mut digest = None;
    for receipt in privacy_receipts {
        let commit_receipt = receipt.commit_receipt();
        verify_receipt_contract(commit_receipt, network_id, profile_id, manifest, record)?;
        if !expected_nodes.contains(&receipt.node_id()) {
            return Err(ConvergenceError::UnauthorizedReceiptSigner(
                receipt.node_id(),
            ));
        }
        if !seen.insert(receipt.node_id()) {
            return Err(ConvergenceError::DuplicateReceipt(receipt.node_id()));
        }
        let matching_commit_receipt = evidence
            .receipts
            .iter()
            .find(|candidate| candidate.node_id() == receipt.node_id())
            .ok_or(ConvergenceError::ReceiptMismatch {
                node_id: receipt.node_id(),
                field: "privacy_state_receipt_node",
            })?;
        if matching_commit_receipt != commit_receipt {
            return Err(ConvergenceError::ReceiptMismatch {
                node_id: receipt.node_id(),
                field: "privacy_state_receipt_commit",
            });
        }
        verify_receipt_signature(commit_receipt, manifest)?;
        let member = manifest
            .manifest
            .members
            .iter()
            .find(|member| member.node_id == receipt.node_id())
            .filter(|member| {
                member.status == MemberStatus::Active
                    && member.roles.contains(&MemberRole::Peer)
                    && member.roles.contains(&MemberRole::Validator)
            })
            .ok_or(ConvergenceError::UnauthorizedReceiptSigner(
                receipt.node_id(),
            ))?;
        let verifying_key = VerifyingKey::from_bytes(&member.identity_public_key)
            .map_err(|_| ConvergenceError::InvalidReceiptSignature(receipt.node_id()))?;
        verifying_key
            .verify_strict(
                &receipt.signature_digest(),
                &Signature::from_bytes(&receipt.node_signature),
            )
            .map_err(|_| ConvergenceError::InvalidReceiptSignature(receipt.node_id()))?;
        let receipt_digest = receipt.privacy_state_digest();
        if receipt_digest == [0; 32] {
            return Err(ConvergenceError::ReceiptMismatch {
                node_id: receipt.node_id(),
                field: "privacy_state_digest",
            });
        }
        if let Some(expected_digest) = digest {
            if expected_digest != receipt_digest {
                return Err(ConvergenceError::ReceiptMismatch {
                    node_id: receipt.node_id(),
                    field: "privacy_state_digest",
                });
            }
        } else {
            digest = Some(receipt_digest);
        }
        verified.push(receipt.clone());
    }
    verified.sort_by_key(|receipt| *receipt.node_id().as_bytes());
    evidence.privacy_state_digest = digest;
    evidence.privacy_receipts = verified;
    Ok(CommitStatus::NetworkConverged { evidence })
}

fn verify_receipt_signature(
    receipt: &CommitReceipt,
    manifest: &SignedMembershipManifest,
) -> Result<(), ConvergenceError> {
    let member = manifest
        .manifest
        .members
        .iter()
        .find(|member| member.node_id == receipt.node_id)
        .filter(|member| {
            member.status == MemberStatus::Active
                && member.roles.contains(&MemberRole::Peer)
                && member.roles.contains(&MemberRole::Validator)
        })
        .ok_or(ConvergenceError::UnauthorizedReceiptSigner(receipt.node_id))?;
    let verifying_key = VerifyingKey::from_bytes(&member.identity_public_key)
        .map_err(|_| ConvergenceError::InvalidReceiptSignature(receipt.node_id))?;
    verifying_key
        .verify_strict(
            &receipt.signature_digest(),
            &Signature::from_bytes(&receipt.node_signature),
        )
        .map_err(|_| ConvergenceError::InvalidReceiptSignature(receipt.node_id))
}

fn verify_receipt_contract(
    receipt: &CommitReceipt,
    network_id: &str,
    profile_id: &str,
    manifest: &SignedMembershipManifest,
    record: &CommittedEnvelopeRecord,
) -> Result<(), ConvergenceError> {
    let fields = [
        (receipt.network_id == network_id, "network_id"),
        (receipt.profile_id == profile_id, "profile_id"),
        (
            receipt.manifest_id == manifest.manifest.manifest_id,
            "manifest_id",
        ),
        (
            receipt.manifest_version == manifest.manifest.version,
            "manifest_version",
        ),
        (
            receipt.manifest_digest == manifest.digest(),
            "manifest_digest",
        ),
        (receipt.ledger_position == record.index, "ledger_position"),
        (
            receipt.envelope_hash == record.envelope_hash,
            "envelope_hash",
        ),
        (
            receipt.ledger_prefix_hash == record.ledger_prefix_hash,
            "ledger_prefix_hash",
        ),
    ];
    for (matches, field) in fields {
        if !matches {
            return Err(ConvergenceError::ReceiptMismatch {
                node_id: receipt.node_id,
                field,
            });
        }
    }
    Ok(())
}

fn validate_text(value: &str, field: &'static str) -> Result<(), ConvergenceError> {
    if value.is_empty() || value.len() > MAX_CONTRACT_TEXT_BYTES {
        return Err(ConvergenceError::MalformedReceipt(format!(
            "{field} must contain between 1 and {MAX_CONTRACT_TEXT_BYTES} UTF-8 bytes"
        )));
    }
    Ok(())
}

fn write_text(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u32).to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

struct CanonicalReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> CanonicalReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn magic(&mut self, expected: &[u8], kind: &'static str) -> Result<(), ConvergenceError> {
        if self.take(expected.len(), kind)? != expected {
            return Err(ConvergenceError::MalformedRecord(format!(
                "invalid {kind} magic"
            )));
        }
        Ok(())
    }

    fn version(&mut self) -> Result<(), ConvergenceError> {
        let version = u16::from_be_bytes(self.fixed::<2>("format version")?);
        if version != FORMAT_VERSION {
            return Err(ConvergenceError::MalformedRecord(format!(
                "unsupported evidence version {version}"
            )));
        }
        Ok(())
    }

    fn u32(&mut self, field: &'static str) -> Result<u32, ConvergenceError> {
        Ok(u32::from_be_bytes(self.fixed::<4>(field)?))
    }

    fn u64(&mut self, field: &'static str) -> Result<u64, ConvergenceError> {
        Ok(u64::from_be_bytes(self.fixed::<8>(field)?))
    }

    fn fixed<const N: usize>(&mut self, field: &'static str) -> Result<[u8; N], ConvergenceError> {
        self.take(N, field)?
            .try_into()
            .map_err(|_| ConvergenceError::MalformedRecord(format!("invalid {field}")))
    }

    fn bytes(&mut self, max: usize, field: &'static str) -> Result<Vec<u8>, ConvergenceError> {
        let len = self.u32(field)? as usize;
        if len == 0 || len > max {
            return Err(ConvergenceError::MalformedRecord(format!(
                "{field} length must be between 1 and {max}"
            )));
        }
        Ok(self.take(len, field)?.to_vec())
    }

    fn text(&mut self, field: &'static str) -> Result<String, ConvergenceError> {
        let raw = self.bytes(MAX_CONTRACT_TEXT_BYTES, field)?;
        let value = str::from_utf8(&raw).map_err(|_| {
            ConvergenceError::MalformedReceipt(format!("{field} is not valid UTF-8"))
        })?;
        validate_text(value, field)?;
        Ok(value.to_string())
    }

    fn take(&mut self, len: usize, field: &'static str) -> Result<&'a [u8], ConvergenceError> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or_else(|| ConvergenceError::MalformedRecord(format!("{field} length overflow")))?;
        if end > self.bytes.len() {
            return Err(ConvergenceError::MalformedRecord(format!(
                "truncated {field}"
            )));
        }
        let value = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(value)
    }

    fn finish(&self) -> Result<(), ConvergenceError> {
        if self.offset != self.bytes.len() {
            return Err(ConvergenceError::MalformedRecord(
                "trailing convergence evidence bytes".to_string(),
            ));
        }
        Ok(())
    }
}
