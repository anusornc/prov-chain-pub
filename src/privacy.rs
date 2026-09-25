//! Closed participant privacy-key lifecycle protocol.
//!
//! This module owns the canonical public bytes, proof inputs, and effective
//! lifecycle reducer used by Final Admission. It deliberately exposes no
//! participant private-key, passphrase, plaintext, or data-encryption-key type.

use ed25519_dalek::{Signature, VerifyingKey};
use p256::ecdsa::{
    signature::Verifier as _, Signature as P256Signature, VerifyingKey as P256VerifyingKey,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use uuid::Uuid;

/// Outer admission-kind identifier fixed by CanonicalPrivacyEncodingV1.
pub const PRIVACY_CONTROL_V1: &str = "PrivacyControlV1";
/// Closed canonical transition-schema identifier.
pub const CANONICAL_PRIVACY_ENCODING_V1: &str = "CanonicalPrivacyEncodingV1";
/// The protected-data suite named by the architecture but not fully activated by Issue #8.
pub const PROTECTED_DATA_SUITE_V1: &str = "ProtectedDataSuiteV1";

const RECORD_MAGIC: &[u8; 4] = b"PCV1";
const DOMAIN_HASH_INPUT_TAG: u8 = 0x01;
const KEY_FINGERPRINT_INPUT_TAG: u8 = 0x05;
const PARTICIPANT_KEY_BINDING_TAG: u8 = 0x10;
const PARTICIPANT_KEY_REFERENCE_TAG: u8 = 0x11;
const BOOTSTRAP_GOVERNANCE_REFERENCE_TAG: u8 = 0x12;
const OBJECT_ENCRYPTION_CONTEXT_TAG: u8 = 0x20;
const PROTECTED_PAYLOAD_CIPHERTEXT_TAG: u8 = 0x21;
const OWNER_DEK_HEADER_TAG: u8 = 0x22;
const OWNER_DEK_ENVELOPE_TAG: u8 = 0x23;
const GRANT_DELIVERY_CONTEXT_TAG: u8 = 0x24;
const GRANT_DEK_HEADER_TAG: u8 = 0x25;
const GRANT_DEK_ENVELOPE_TAG: u8 = 0x26;
const REGISTER_CORE_TAG: u8 = 0x30;
const BIND_CORE_TAG: u8 = 0x31;
const REVOKE_CORE_TAG: u8 = 0x32;
const CREATE_OBJECT_CORE_TAG: u8 = 0x33;
const GRANT_ACCESS_CORE_TAG: u8 = 0x34;
const REVOKE_GRANT_CORE_TAG: u8 = 0x35;
const REGISTER_COMPLETE_TAG: u8 = 0x40;
const BIND_COMPLETE_TAG: u8 = 0x41;
const REVOKE_COMPLETE_TAG: u8 = 0x42;
const CREATE_OBJECT_COMPLETE_TAG: u8 = 0x43;
const GRANT_ACCESS_COMPLETE_TAG: u8 = 0x44;
const REVOKE_GRANT_COMPLETE_TAG: u8 = 0x45;
const OWNER_RELEASE_EVIDENCE_TAG: u8 = 0x50;
const GRANTEE_RELEASE_EVIDENCE_TAG: u8 = 0x51;
const LIVE_RELEASE_RESPONSE_TAG: u8 = 0x52;
const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_RECORD_BYTES: usize = 8192;
/// Consensus-visible maximum for one complete `CreateProtectedObject` transition.
pub const MAX_CREATE_PROTECTED_OBJECT_BYTES: usize = 270_336;
/// Maximum opaque plaintext accepted by a participant-side v1 constructor.
pub const MAX_PROTECTED_CONTENT_BYTES: usize = 262_144;
const MAX_PROTECTED_PAYLOAD_BYTES: usize = 262_250;
const MAX_OWNER_HEADER_BYTES: usize = 2048;
const MAX_OWNER_ENVELOPE_BYTES: usize = 2304;
const MAX_GRANT_HEADER_BYTES: usize = 2048;
const MAX_GRANT_ENVELOPE_BYTES: usize = 2304;
const MAX_LIVE_RELEASE_EVIDENCE_BYTES: usize = 4096;
const MAX_LIVE_RELEASE_RESPONSE_BYTES: usize = 270_336;

const KEY_FINGERPRINT_DOMAIN: &str = "provchain/privacy-key/fingerprint/v1";
const BOOTSTRAP_KEY_SCHEME: &str = "PrivacyBootstrapGovernanceEd25519V1";
const AUTHORIZATION_KEY_SCHEME: &str = "PrivacyAuthorizationEd25519V1";
const REGISTER_TRANSITION_ID_DOMAIN: &str = "provchain/privacy-bootstrap/transition-id/v1";
const REGISTER_AUTHORIZATION_DOMAIN: &str =
    "provchain/privacy-bootstrap/governance-authorization/v1";
const REGISTER_POSSESSION_DOMAIN: &str = "provchain/privacy-bootstrap/key-possession/v1";
const BIND_TRANSITION_ID_DOMAIN: &str = "provchain/privacy-key-bind/transition-id/v1";
const BIND_AUTHORIZATION_DOMAIN: &str = "provchain/privacy-key-bind/participant-authorization/v1";
const BIND_POSSESSION_DOMAIN: &str = "provchain/privacy-key-bind/key-possession/v1";
const REVOKE_TRANSITION_ID_DOMAIN: &str = "provchain/privacy-key-revoke/transition-id/v1";
const REVOKE_AUTHORIZATION_DOMAIN: &str =
    "provchain/privacy-key-revoke/participant-authorization/v1";
const OBJECT_CONTEXT_DOMAIN: &str = "provchain/protected-object/encryption-context/v1";
const ENCRYPTED_PAYLOAD_COMMITMENT_DOMAIN: &str =
    "provchain/protected-data/encrypted-payload-commitment/v1";
const CREATE_OBJECT_TRANSITION_ID_DOMAIN: &str =
    "provchain/protected-object/create-transition-id/v1";
const CREATE_OBJECT_AUTHORIZATION_DOMAIN: &str =
    "provchain/protected-object/owner-authorization/v1";
const GRANT_DELIVERY_CONTEXT_DOMAIN: &str = "provchain/privacy-grant/delivery-context/v1";
const GRANT_ID_DOMAIN: &str = "provchain/privacy-grant/grant-id/v1";
const GRANT_AUTHORIZATION_DOMAIN: &str = "provchain/privacy-grant/owner-authorization/v1";
const REVOKE_GRANT_TRANSITION_ID_DOMAIN: &str = "provchain/privacy-grant/revoke-transition-id/v1";
const REVOKE_GRANT_AUTHORIZATION_DOMAIN: &str =
    "provchain/privacy-grant/revoke-owner-authorization/v1";
const GRANT_HEADER_HASH_DOMAIN: &str = "provchain/protected-data/hpke-grant-header/v1";

/// A profile-pinned bootstrap key and content identity for the Issue #8 lifecycle slice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PrivacyLifecycleProfile {
    network_profile_content_hash: [u8; 32],
    bootstrap_governance_key: BootstrapGovernanceKeyBinding,
    #[serde(default)]
    network_role_public_keys: BTreeSet<[u8; 32]>,
}

/// A lifecycle verifier whose profile digest and complete non-participant key-role
/// set were checked against a governance-authenticated Network Profile context.
///
/// The inner binding is intentionally opaque so a ledger cannot activate privacy
/// admission from caller-chosen hash bytes or an incomplete reserved-key set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivatedPrivacyLifecycleProfile {
    profile: PrivacyLifecycleProfile,
}

impl ActivatedPrivacyLifecycleProfile {
    /// Return the exact verified lifecycle contract used by codecs and reducers.
    pub fn profile(&self) -> &PrivacyLifecycleProfile {
        &self.profile
    }
}

/// Governance-verified privacy slice used only by direct conformance ledgers.
///
/// This capability is deliberately distinct from [`ActivatedPrivacyLifecycleProfile`].
/// Reference-node startup never accepts it, so feature-gated conformance coverage
/// cannot be confused with deployable profile activation.
#[cfg(feature = "privacy-conformance")]
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivacyLifecycleConformanceProfile {
    profile: PrivacyLifecycleProfile,
}

#[cfg(feature = "privacy-conformance")]
impl PrivacyLifecycleConformanceProfile {
    pub(crate) fn new(profile: PrivacyLifecycleProfile) -> Self {
        Self { profile }
    }

    /// Return the exact verified contract exercised by the conformance ledger.
    pub fn profile(&self) -> &PrivacyLifecycleProfile {
        &self.profile
    }
}

#[cfg(feature = "privacy-conformance")]
impl std::ops::Deref for PrivacyLifecycleConformanceProfile {
    type Target = PrivacyLifecycleProfile;

    fn deref(&self) -> &Self::Target {
        &self.profile
    }
}

impl std::ops::Deref for ActivatedPrivacyLifecycleProfile {
    type Target = PrivacyLifecycleProfile;

    fn deref(&self) -> &Self::Target {
        &self.profile
    }
}

impl PrivacyLifecycleProfile {
    /// Bind the exact parent Network Profile hash to one active Ed25519 bootstrap key.
    pub fn new(
        network_profile_content_hash: [u8; 32],
        bootstrap_public_key: [u8; 32],
    ) -> Result<Self, PrivacyError> {
        if network_profile_content_hash == [0; 32] {
            return Err(PrivacyError::InvalidProfile(
                "network profile content hash is empty".to_string(),
            ));
        }
        let verifier = VerifyingKey::from_bytes(&bootstrap_public_key).map_err(|error| {
            PrivacyError::InvalidProfile(format!(
                "invalid privacy bootstrap governance Ed25519 key: {error}"
            ))
        })?;
        if verifier.is_weak() {
            return Err(PrivacyError::InvalidProfile(
                "privacy bootstrap governance Ed25519 key is weak".to_string(),
            ));
        }
        let fingerprint = key_fingerprint(
            BOOTSTRAP_KEY_SCHEME,
            KeyAlgorithm::Ed25519,
            PublicKeyEncoding::Ed25519Raw32,
            &bootstrap_public_key,
        );
        Ok(Self {
            network_profile_content_hash,
            bootstrap_governance_key: BootstrapGovernanceKeyBinding {
                version: 1,
                public_key: bootstrap_public_key,
                fingerprint,
                active: true,
            },
            network_role_public_keys: BTreeSet::new(),
        })
    }

    /// Bind every non-privacy Ed25519 key role from the verified Network Profile
    /// and its governance-authenticated Membership Manifest.
    pub fn with_network_role_public_keys(
        mut self,
        keys: impl IntoIterator<Item = [u8; 32]>,
    ) -> Result<Self, PrivacyError> {
        self.network_role_public_keys = keys.into_iter().collect();
        self.validate()?;
        Ok(self)
    }

    /// Validate a deserialized lifecycle profile before activation.
    pub fn validate(&self) -> Result<(), PrivacyError> {
        if self.network_profile_content_hash == [0; 32] {
            return Err(PrivacyError::InvalidProfile(
                "network profile content hash is empty".to_string(),
            ));
        }
        if self.bootstrap_governance_key.version != 1 || !self.bootstrap_governance_key.active {
            return Err(PrivacyError::InvalidProfile(
                "privacy bootstrap governance key must be active version 1".to_string(),
            ));
        }
        let verifier = VerifyingKey::from_bytes(&self.bootstrap_governance_key.public_key)
            .map_err(|error| {
                PrivacyError::InvalidProfile(format!(
                    "invalid privacy bootstrap governance Ed25519 key: {error}"
                ))
            })?;
        if verifier.is_weak() {
            return Err(PrivacyError::InvalidProfile(
                "privacy bootstrap governance Ed25519 key is weak".to_string(),
            ));
        }
        let expected = key_fingerprint(
            BOOTSTRAP_KEY_SCHEME,
            KeyAlgorithm::Ed25519,
            PublicKeyEncoding::Ed25519Raw32,
            &self.bootstrap_governance_key.public_key,
        );
        if self.bootstrap_governance_key.fingerprint != expected {
            return Err(PrivacyError::InvalidProfile(
                "privacy bootstrap governance key fingerprint mismatch".to_string(),
            ));
        }
        for public_key in &self.network_role_public_keys {
            if public_key == &self.bootstrap_governance_key.public_key {
                return Err(PrivacyError::InvalidProfile(
                    "privacy bootstrap governance key reuses another Network Profile key role"
                        .to_string(),
                ));
            }
            let verifier = VerifyingKey::from_bytes(public_key).map_err(|error| {
                PrivacyError::InvalidProfile(format!(
                    "invalid Network Profile role Ed25519 key: {error}"
                ))
            })?;
            if verifier.is_weak() {
                return Err(PrivacyError::InvalidProfile(
                    "Network Profile role Ed25519 key is weak".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Exact Network Profile content identity bound into every lifecycle transition.
    pub fn network_profile_content_hash(&self) -> [u8; 32] {
        self.network_profile_content_hash
    }

    /// Public key material reserved for non-participant profile roles.
    pub fn network_role_public_keys(&self) -> &BTreeSet<[u8; 32]> {
        &self.network_role_public_keys
    }

    /// Public bootstrap governance verification key.
    pub fn bootstrap_public_key(&self) -> [u8; 32] {
        self.bootstrap_governance_key.public_key
    }

    /// Scheme-derived bootstrap governance key fingerprint.
    pub fn bootstrap_fingerprint(&self) -> [u8; 32] {
        self.bootstrap_governance_key.fingerprint
    }

    pub(crate) fn bind_network_profile_content_hash(&mut self, content_hash: [u8; 32]) {
        self.network_profile_content_hash = content_hash;
    }

    pub(crate) fn key_reuses_profile_role(&self, public_key: &[u8]) -> bool {
        public_key == self.bootstrap_governance_key.public_key
            || self
                .network_role_public_keys
                .iter()
                .any(|reserved| public_key == reserved)
    }

    fn bootstrap_reference(&self) -> BootstrapGovernanceKeyReference {
        BootstrapGovernanceKeyReference {
            version: self.bootstrap_governance_key.version,
            fingerprint: self.bootstrap_governance_key.fingerprint,
        }
    }

    pub(crate) fn bootstrap_reference_bytes(&self) -> Vec<u8> {
        self.bootstrap_reference().canonical_bytes()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct BootstrapGovernanceKeyBinding {
    version: u32,
    public_key: [u8; 32],
    fingerprint: [u8; 32],
    active: bool,
}

/// The exact verified parent facts covered by one privacy transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivacyAdmissionAnchor {
    network_id: String,
    profile_id: String,
    profile_content_hash: [u8; 32],
    expected_ledger_position: u64,
    expected_privacy_revision: u64,
    parent_ledger_prefix_hash: [u8; 32],
    parent_envelope_hash: Option<[u8; 32]>,
}

impl PrivacyAdmissionAnchor {
    /// Construct the anchor for a first ledger record at the virtual genesis parent.
    pub fn genesis(
        network_id: impl Into<String>,
        profile_id: impl Into<String>,
        profile_content_hash: [u8; 32],
        expected_ledger_position: u64,
        expected_privacy_revision: u64,
    ) -> Result<Self, PrivacyError> {
        if expected_ledger_position != 0 {
            return Err(PrivacyError::Malformed(
                "genesis anchor must target ledger position zero".to_string(),
            ));
        }
        Self::new(
            network_id,
            profile_id,
            profile_content_hash,
            expected_ledger_position,
            expected_privacy_revision,
            crate::network::convergence::genesis_ledger_prefix_hash(),
            None,
        )
    }

    /// Construct an exact parent anchor for a later ledger position.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        network_id: impl Into<String>,
        profile_id: impl Into<String>,
        profile_content_hash: [u8; 32],
        expected_ledger_position: u64,
        expected_privacy_revision: u64,
        parent_ledger_prefix_hash: [u8; 32],
        parent_envelope_hash: Option<[u8; 32]>,
    ) -> Result<Self, PrivacyError> {
        let anchor = Self {
            network_id: network_id.into(),
            profile_id: profile_id.into(),
            profile_content_hash,
            expected_ledger_position,
            expected_privacy_revision,
            parent_ledger_prefix_hash,
            parent_envelope_hash,
        };
        anchor.validate()?;
        Ok(anchor)
    }

    fn validate(&self) -> Result<(), PrivacyError> {
        validate_identifier(&self.network_id, "network_id")?;
        validate_identifier(&self.profile_id, "profile_id")?;
        if self.profile_content_hash == [0; 32] {
            return Err(PrivacyError::Malformed(
                "profile content hash is empty".to_string(),
            ));
        }
        if self.expected_privacy_revision == 0 {
            return Err(PrivacyError::Malformed(
                "expected privacy revision must be positive".to_string(),
            ));
        }
        match (self.expected_ledger_position, self.parent_envelope_hash) {
            (0, None) => {}
            (0, Some(_)) => {
                return Err(PrivacyError::Malformed(
                    "ledger position zero must use the genesis parent reference".to_string(),
                ));
            }
            (_, None) => {
                return Err(PrivacyError::Malformed(
                    "non-genesis position requires a parent envelope hash".to_string(),
                ));
            }
            (_, Some(hash)) if hash == [0; 32] => {
                return Err(PrivacyError::Malformed(
                    "parent envelope hash is empty".to_string(),
                ));
            }
            (_, Some(_)) => {}
        }
        Ok(())
    }

    /// Network identity covered by this exact ledger-parent anchor.
    pub fn network_id(&self) -> &str {
        &self.network_id
    }

    /// Network Profile identifier covered by this exact ledger-parent anchor.
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    /// Network Profile content identity covered by this exact ledger-parent anchor.
    pub fn profile_content_hash(&self) -> [u8; 32] {
        self.profile_content_hash
    }

    /// Ledger position this candidate must occupy.
    pub fn expected_ledger_position(&self) -> u64 {
        self.expected_ledger_position
    }

    /// Privacy revision this candidate must create from its verified parent.
    pub fn expected_privacy_revision(&self) -> u64 {
        self.expected_privacy_revision
    }

    /// Exact committed-ledger prefix hash this candidate extends.
    pub fn parent_ledger_prefix_hash(&self) -> [u8; 32] {
        self.parent_ledger_prefix_hash
    }

    /// Exact parent envelope hash, or `None` only at virtual genesis.
    pub fn parent_envelope_hash(&self) -> Option<[u8; 32]> {
        self.parent_envelope_hash
    }

    fn common_fields(&self, variant: TransitionVariant) -> Vec<Vec<u8>> {
        vec![
            PRIVACY_CONTROL_V1.as_bytes().to_vec(),
            CANONICAL_PRIVACY_ENCODING_V1.as_bytes().to_vec(),
            vec![variant as u8],
            self.network_id.as_bytes().to_vec(),
            self.profile_id.as_bytes().to_vec(),
            self.profile_content_hash.to_vec(),
            self.expected_ledger_position.to_be_bytes().to_vec(),
            self.expected_privacy_revision.to_be_bytes().to_vec(),
            self.parent_ledger_prefix_hash.to_vec(),
            encode_parent_envelope_reference(self.parent_envelope_hash).to_vec(),
        ]
    }
}

/// Closed participant-key purpose namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ParticipantKeyPurpose {
    /// Ed25519 authorization of participant-controlled transitions.
    PrivacyAuthorization = 0x01,
    /// P-256 wrapping/decapsulation key used by later protected-data tickets.
    PrivacyKeyWrapping = 0x02,
}

/// Permission namespace accepted by the Issue #10 grant lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum PrivacyPermission {
    /// Read one protected object through the converged opaque release path.
    ReadProtectedObjectV1 = 0x01,
}

impl PrivacyPermission {
    fn decode(bytes: &[u8]) -> Result<Self, PrivacyError> {
        match fixed::<1>(bytes, "privacy grant permission")?[0] {
            0x01 => Ok(Self::ReadProtectedObjectV1),
            value => Err(PrivacyError::Malformed(format!(
                "unsupported privacy grant permission value {value}"
            ))),
        }
    }
}

/// Ledger-derived status of one immutable privacy grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum PrivacyGrantStatus {
    /// The grant can authorize a current converged live release.
    Active = 0x01,
    /// The grant is terminally revoked for future releases.
    Revoked = 0x02,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum TransitionVariant {
    RegisterPrincipal = 0x01,
    BindParticipantKey = 0x02,
    RevokeParticipantKey = 0x03,
    CreateProtectedObject = 0x04,
    GrantAccess = 0x05,
    RevokeGrant = 0x06,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum KeyAlgorithm {
    Ed25519 = 0x01,
    P256 = 0x02,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum PublicKeyEncoding {
    Ed25519Raw32 = 0x01,
    Sec1UncompressedP256 = 0x02,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum PossessionProofScheme {
    Ed25519Raw64 = 0x01,
    EcdsaP256Sha256RawLowS64 = 0x02,
}

/// Immutable canonical reference to one participant key version.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParticipantKeyReference {
    principal: Uuid,
    purpose: ParticipantKeyPurpose,
    version: u32,
    algorithm: KeyAlgorithm,
    encoding: PublicKeyEncoding,
    fingerprint: [u8; 32],
}

impl ParticipantKeyReference {
    /// Referenced participant principal.
    pub fn principal(&self) -> Uuid {
        self.principal
    }

    /// Purpose-specific namespace of this key version.
    pub fn purpose(&self) -> ParticipantKeyPurpose {
        self.purpose
    }

    /// Positive contiguous version within the principal and purpose.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Scheme-derived canonical fingerprint.
    pub fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }

    /// Exact `ParticipantKeyReferenceV1` bytes used by public contexts and envelopes.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        encode_record(
            PARTICIPANT_KEY_REFERENCE_TAG,
            vec![
                self.principal.as_bytes().to_vec(),
                vec![self.purpose as u8],
                self.version.to_be_bytes().to_vec(),
                vec![self.algorithm as u8],
                vec![self.encoding as u8],
                self.fingerprint.to_vec(),
            ],
        )
    }

    /// Strictly decode one exact public key reference.
    pub fn decode(bytes: &[u8]) -> Result<Self, PrivacyError> {
        decode_participant_key_reference(bytes)
    }

    pub(crate) fn for_authorization(
        principal: Uuid,
        version: u32,
        public_key: [u8; 32],
    ) -> Result<Self, PrivacyError> {
        Ok(ParticipantKeyBinding::authorization(principal, version, public_key)?.reference)
    }

    pub(crate) fn for_wrapping(
        principal: Uuid,
        version: u32,
        public_key: [u8; 65],
    ) -> Result<Self, PrivacyError> {
        Ok(ParticipantKeyBinding::wrapping(principal, version, public_key)?.reference)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParticipantKeyBinding {
    reference: ParticipantKeyReference,
    possession_scheme: PossessionProofScheme,
    public_key: Vec<u8>,
}

impl ParticipantKeyBinding {
    fn authorization(
        principal: Uuid,
        version: u32,
        public_key: [u8; 32],
    ) -> Result<Self, PrivacyError> {
        validate_principal(principal)?;
        if version == 0 {
            return Err(PrivacyError::Malformed(
                "participant key version must be positive".to_string(),
            ));
        }
        let verifier = VerifyingKey::from_bytes(&public_key).map_err(|error| {
            PrivacyError::Malformed(format!("invalid participant Ed25519 key: {error}"))
        })?;
        if verifier.is_weak() {
            return Err(PrivacyError::Malformed(
                "participant Ed25519 key is weak".to_string(),
            ));
        }
        let fingerprint = key_fingerprint(
            AUTHORIZATION_KEY_SCHEME,
            KeyAlgorithm::Ed25519,
            PublicKeyEncoding::Ed25519Raw32,
            &public_key,
        );
        Ok(Self {
            reference: ParticipantKeyReference {
                principal,
                purpose: ParticipantKeyPurpose::PrivacyAuthorization,
                version,
                algorithm: KeyAlgorithm::Ed25519,
                encoding: PublicKeyEncoding::Ed25519Raw32,
                fingerprint,
            },
            possession_scheme: PossessionProofScheme::Ed25519Raw64,
            public_key: public_key.to_vec(),
        })
    }

    fn wrapping(principal: Uuid, version: u32, public_key: [u8; 65]) -> Result<Self, PrivacyError> {
        validate_principal(principal)?;
        if version == 0 {
            return Err(PrivacyError::Malformed(
                "participant key version must be positive".to_string(),
            ));
        }
        if public_key[0] != 0x04 {
            return Err(PrivacyError::Malformed(
                "wrapping key must use uncompressed SEC1 encoding".to_string(),
            ));
        }
        P256VerifyingKey::from_sec1_bytes(&public_key).map_err(|error| {
            PrivacyError::Malformed(format!("invalid P-256 wrapping public key: {error}"))
        })?;
        let fingerprint = key_fingerprint(
            PROTECTED_DATA_SUITE_V1,
            KeyAlgorithm::P256,
            PublicKeyEncoding::Sec1UncompressedP256,
            &public_key,
        );
        Ok(Self {
            reference: ParticipantKeyReference {
                principal,
                purpose: ParticipantKeyPurpose::PrivacyKeyWrapping,
                version,
                algorithm: KeyAlgorithm::P256,
                encoding: PublicKeyEncoding::Sec1UncompressedP256,
                fingerprint,
            },
            possession_scheme: PossessionProofScheme::EcdsaP256Sha256RawLowS64,
            public_key: public_key.to_vec(),
        })
    }

    fn canonical_bytes(&self) -> Vec<u8> {
        encode_record(
            PARTICIPANT_KEY_BINDING_TAG,
            vec![
                self.reference.principal.as_bytes().to_vec(),
                vec![self.reference.purpose as u8],
                self.reference.version.to_be_bytes().to_vec(),
                vec![self.reference.algorithm as u8],
                vec![self.reference.encoding as u8],
                self.reference.fingerprint.to_vec(),
                vec![self.possession_scheme as u8],
                self.public_key.clone(),
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BootstrapGovernanceKeyReference {
    version: u32,
    fingerprint: [u8; 32],
}

impl BootstrapGovernanceKeyReference {
    fn canonical_bytes(&self) -> Vec<u8> {
        encode_record(
            BOOTSTRAP_GOVERNANCE_REFERENCE_TAG,
            vec![
                vec![0x01],
                self.version.to_be_bytes().to_vec(),
                vec![KeyAlgorithm::Ed25519 as u8],
                vec![PublicKeyEncoding::Ed25519Raw32 as u8],
                self.fingerprint.to_vec(),
            ],
        )
    }
}

/// Canonical immutable payload bytes committed by `CreateProtectedObject`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedPayloadCiphertext {
    protected_content_commitment: [u8; 32],
    ciphertext_and_tag: Vec<u8>,
    canonical_bytes: Vec<u8>,
}

impl ProtectedPayloadCiphertext {
    /// Strictly decode `ProtectedPayloadCiphertextV1` without decrypting it.
    pub fn decode(bytes: &[u8]) -> Result<Self, PrivacyError> {
        if !(107..=MAX_PROTECTED_PAYLOAD_BYTES).contains(&bytes.len()) {
            return Err(PrivacyError::Malformed(format!(
                "protected payload record must contain 107..={MAX_PROTECTED_PAYLOAD_BYTES} bytes"
            )));
        }
        let fields = decode_record(bytes, PROTECTED_PAYLOAD_CIPHERTEXT_TAG, 4)?;
        if fields[0] != PROTECTED_DATA_SUITE_V1.as_bytes() {
            return Err(PrivacyError::Malformed(
                "unsupported protected payload suite".to_string(),
            ));
        }
        if fields[1] != [0_u8; 12] {
            return Err(PrivacyError::Malformed(
                "protected payload must record the fixed zero nonce".to_string(),
            ));
        }
        let protected_content_commitment = fixed::<32>(fields[2], "protected content commitment")?;
        if !(17..=262_160).contains(&fields[3].len()) {
            return Err(PrivacyError::Malformed(
                "payload ciphertext and tag must contain 17..=262160 bytes".to_string(),
            ));
        }
        let payload = Self {
            protected_content_commitment,
            ciphertext_and_tag: fields[3].to_vec(),
            canonical_bytes: bytes.to_vec(),
        };
        if payload.canonical_bytes() != bytes {
            return Err(PrivacyError::NonCanonicalRecord);
        }
        Ok(payload)
    }

    /// Exact canonical record bytes stored in the ledger journal.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        self.canonical_bytes.clone()
    }

    /// Keyed protected-content commitment carried by this record.
    pub fn protected_content_commitment(&self) -> [u8; 32] {
        self.protected_content_commitment
    }

    /// Public ciphertext plus its full 16-byte authentication tag.
    pub fn ciphertext_and_tag(&self) -> &[u8] {
        &self.ciphertext_and_tag
    }

    pub(crate) fn new(
        protected_content_commitment: [u8; 32],
        ciphertext_and_tag: Vec<u8>,
    ) -> Result<Self, PrivacyError> {
        let bytes = encode_record(
            PROTECTED_PAYLOAD_CIPHERTEXT_TAG,
            vec![
                PROTECTED_DATA_SUITE_V1.as_bytes().to_vec(),
                vec![0; 12],
                protected_content_commitment.to_vec(),
                ciphertext_and_tag,
            ],
        );
        Self::decode(&bytes)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OwnerDekHeader {
    network_id: String,
    profile_id: String,
    profile_content_hash: [u8; 32],
    expected_ledger_position: u64,
    expected_privacy_revision: u64,
    parent_ledger_prefix_hash: [u8; 32],
    parent_envelope_hash: Option<[u8; 32]>,
    object_id: Uuid,
    owner: Uuid,
    wrapping_key: ParticipantKeyReference,
    protected_content_commitment: [u8; 32],
    encrypted_payload_commitment: [u8; 32],
    canonical_bytes: Vec<u8>,
}

impl OwnerDekHeader {
    fn decode(bytes: &[u8]) -> Result<Self, PrivacyError> {
        if bytes.len() > MAX_OWNER_HEADER_BYTES {
            return Err(PrivacyError::Malformed(format!(
                "owner DEK header exceeds {MAX_OWNER_HEADER_BYTES} bytes"
            )));
        }
        let fields = decode_record(bytes, OWNER_DEK_HEADER_TAG, 13)?;
        if fields[0] != PROTECTED_DATA_SUITE_V1.as_bytes() {
            return Err(PrivacyError::Malformed(
                "unsupported owner DEK header suite".to_string(),
            ));
        }
        let wrapping_key = decode_participant_key_reference(fields[10])?;
        if wrapping_key.purpose != ParticipantKeyPurpose::PrivacyKeyWrapping {
            return Err(PrivacyError::Malformed(
                "owner envelope recipient must be a privacy-key-wrapping key".to_string(),
            ));
        }
        let header = Self {
            network_id: ascii_identifier(fields[1], "owner header network_id")?,
            profile_id: ascii_identifier(fields[2], "owner header profile_id")?,
            profile_content_hash: fixed::<32>(fields[3], "owner header profile hash")?,
            expected_ledger_position: u64_field(fields[4], "owner header ledger position")?,
            expected_privacy_revision: u64_field(fields[5], "owner header privacy revision")?,
            parent_ledger_prefix_hash: fixed::<32>(
                fields[6],
                "owner header parent ledger prefix hash",
            )?,
            parent_envelope_hash: decode_parent_envelope_reference(fields[7])?,
            object_id: object_uuid(fields[8], "owner header object identifier")?,
            owner: uuid(fields[9], "owner header principal")?,
            wrapping_key,
            protected_content_commitment: fixed::<32>(
                fields[11],
                "owner header protected content commitment",
            )?,
            encrypted_payload_commitment: fixed::<32>(
                fields[12],
                "owner header encrypted payload commitment",
            )?,
            canonical_bytes: bytes.to_vec(),
        };
        if header.encode() != bytes {
            return Err(PrivacyError::NonCanonicalRecord);
        }
        Ok(header)
    }

    fn from_parts(
        anchor: &PrivacyAdmissionAnchor,
        object_id: Uuid,
        owner: Uuid,
        wrapping_key: ParticipantKeyReference,
        protected_content_commitment: [u8; 32],
        encrypted_payload_commitment: [u8; 32],
    ) -> Self {
        let mut header = Self {
            network_id: anchor.network_id.clone(),
            profile_id: anchor.profile_id.clone(),
            profile_content_hash: anchor.profile_content_hash,
            expected_ledger_position: anchor.expected_ledger_position,
            expected_privacy_revision: anchor.expected_privacy_revision,
            parent_ledger_prefix_hash: anchor.parent_ledger_prefix_hash,
            parent_envelope_hash: anchor.parent_envelope_hash,
            object_id,
            owner,
            wrapping_key,
            protected_content_commitment,
            encrypted_payload_commitment,
            canonical_bytes: Vec::new(),
        };
        header.canonical_bytes = header.encode();
        header
    }

    fn encode(&self) -> Vec<u8> {
        encode_record(
            OWNER_DEK_HEADER_TAG,
            vec![
                PROTECTED_DATA_SUITE_V1.as_bytes().to_vec(),
                self.network_id.as_bytes().to_vec(),
                self.profile_id.as_bytes().to_vec(),
                self.profile_content_hash.to_vec(),
                self.expected_ledger_position.to_be_bytes().to_vec(),
                self.expected_privacy_revision.to_be_bytes().to_vec(),
                self.parent_ledger_prefix_hash.to_vec(),
                encode_parent_envelope_reference(self.parent_envelope_hash).to_vec(),
                self.object_id.as_bytes().to_vec(),
                self.owner.as_bytes().to_vec(),
                self.wrapping_key.canonical_bytes(),
                self.protected_content_commitment.to_vec(),
                self.encrypted_payload_commitment.to_vec(),
            ],
        )
    }
}

/// Canonical owner-addressed opaque HPKE envelope committed with one object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerDekEnvelope {
    object_context: [u8; 32],
    header: OwnerDekHeader,
    encapsulated_key: [u8; 65],
    wrapped_dek_ciphertext: [u8; 48],
    canonical_bytes: Vec<u8>,
}

impl OwnerDekEnvelope {
    /// Strictly decode the public `OwnerDekEnvelopeV1` structure.
    pub fn decode(bytes: &[u8]) -> Result<Self, PrivacyError> {
        if bytes.len() > MAX_OWNER_ENVELOPE_BYTES {
            return Err(PrivacyError::Malformed(format!(
                "owner DEK envelope exceeds {MAX_OWNER_ENVELOPE_BYTES} bytes"
            )));
        }
        let fields = decode_record(bytes, OWNER_DEK_ENVELOPE_TAG, 5)?;
        if fields[0] != PROTECTED_DATA_SUITE_V1.as_bytes() {
            return Err(PrivacyError::Malformed(
                "unsupported owner DEK envelope suite".to_string(),
            ));
        }
        let encapsulated_key = fixed::<65>(fields[3], "HPKE encapsulated P-256 key")?;
        if encapsulated_key[0] != 0x04
            || P256VerifyingKey::from_sec1_bytes(&encapsulated_key).is_err()
        {
            return Err(PrivacyError::Malformed(
                "HPKE encapsulated key is not a canonical uncompressed P-256 point".to_string(),
            ));
        }
        let envelope = Self {
            object_context: fixed::<32>(fields[1], "object encryption context")?,
            header: OwnerDekHeader::decode(fields[2])?,
            encapsulated_key,
            wrapped_dek_ciphertext: fixed::<48>(fields[4], "wrapped DEK ciphertext")?,
            canonical_bytes: bytes.to_vec(),
        };
        if envelope.encode() != bytes {
            return Err(PrivacyError::NonCanonicalRecord);
        }
        Ok(envelope)
    }

    /// Exact canonical public envelope bytes stored in the journal.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        self.canonical_bytes.clone()
    }

    /// Object encryption context bound into the HPKE info construction.
    pub fn object_context(&self) -> [u8; 32] {
        self.object_context
    }

    /// Exact owner envelope header bytes used by the HPKE info construction.
    pub(crate) fn header_bytes(&self) -> &[u8] {
        &self.header.canonical_bytes
    }

    /// Exact HPKE encapsulated public key needed by participant-side opening.
    pub(crate) fn encapsulated_key(&self) -> [u8; 65] {
        self.encapsulated_key
    }

    /// Exact opaque wrapped-DEK ciphertext; it is never opened by a ledger node.
    pub(crate) fn wrapped_dek_ciphertext(&self) -> [u8; 48] {
        self.wrapped_dek_ciphertext
    }

    /// Exact wrapping-key reference addressed by this owner envelope.
    pub fn wrapping_key(&self) -> &ParticipantKeyReference {
        &self.header.wrapping_key
    }

    pub(crate) fn new(
        object_context: [u8; 32],
        header: OwnerDekHeader,
        encapsulated_key: [u8; 65],
        wrapped_dek_ciphertext: [u8; 48],
    ) -> Result<Self, PrivacyError> {
        let bytes = encode_record(
            OWNER_DEK_ENVELOPE_TAG,
            vec![
                PROTECTED_DATA_SUITE_V1.as_bytes().to_vec(),
                object_context.to_vec(),
                header.canonical_bytes.clone(),
                encapsulated_key.to_vec(),
                wrapped_dek_ciphertext.to_vec(),
            ],
        );
        Self::decode(&bytes)
    }

    fn encode(&self) -> Vec<u8> {
        encode_record(
            OWNER_DEK_ENVELOPE_TAG,
            vec![
                PROTECTED_DATA_SUITE_V1.as_bytes().to_vec(),
                self.object_context.to_vec(),
                self.header.canonical_bytes.clone(),
                self.encapsulated_key.to_vec(),
                self.wrapped_dek_ciphertext.to_vec(),
            ],
        )
    }
}

/// Canonical public header for one owner-to-grantee DEK delivery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantDekHeader {
    network_id: String,
    profile_id: String,
    profile_content_hash: [u8; 32],
    expected_ledger_position: u64,
    expected_privacy_revision: u64,
    parent_ledger_prefix_hash: [u8; 32],
    parent_envelope_hash: Option<[u8; 32]>,
    object_id: Uuid,
    owner: Uuid,
    grantee: Uuid,
    permission: PrivacyPermission,
    wrapping_key: ParticipantKeyReference,
    protected_content_commitment: [u8; 32],
    encrypted_payload_commitment: [u8; 32],
    canonical_bytes: Vec<u8>,
}

impl GrantDekHeader {
    #[allow(clippy::too_many_arguments)]
    fn from_parts(
        anchor: &PrivacyAdmissionAnchor,
        object_id: Uuid,
        owner: Uuid,
        grantee: Uuid,
        permission: PrivacyPermission,
        wrapping_key: ParticipantKeyReference,
        protected_content_commitment: [u8; 32],
        encrypted_payload_commitment: [u8; 32],
    ) -> Self {
        let mut header = Self {
            network_id: anchor.network_id.clone(),
            profile_id: anchor.profile_id.clone(),
            profile_content_hash: anchor.profile_content_hash,
            expected_ledger_position: anchor.expected_ledger_position,
            expected_privacy_revision: anchor.expected_privacy_revision,
            parent_ledger_prefix_hash: anchor.parent_ledger_prefix_hash,
            parent_envelope_hash: anchor.parent_envelope_hash,
            object_id,
            owner,
            grantee,
            permission,
            wrapping_key,
            protected_content_commitment,
            encrypted_payload_commitment,
            canonical_bytes: Vec::new(),
        };
        header.canonical_bytes = header.encode();
        header
    }

    /// Strictly decode one canonical grant header.
    pub fn decode(bytes: &[u8]) -> Result<Self, PrivacyError> {
        if bytes.len() > MAX_GRANT_HEADER_BYTES {
            return Err(PrivacyError::Malformed(format!(
                "grant DEK header exceeds {MAX_GRANT_HEADER_BYTES} bytes"
            )));
        }
        let fields = decode_record(bytes, GRANT_DEK_HEADER_TAG, 15)?;
        if fields[0] != PROTECTED_DATA_SUITE_V1.as_bytes() {
            return Err(PrivacyError::Malformed(
                "unsupported grant DEK header suite".to_string(),
            ));
        }
        let wrapping_key = decode_participant_key_reference(fields[12])?;
        if wrapping_key.purpose != ParticipantKeyPurpose::PrivacyKeyWrapping {
            return Err(PrivacyError::Malformed(
                "grant envelope recipient must be a privacy-key-wrapping key".to_string(),
            ));
        }
        let header = Self {
            network_id: ascii_identifier(fields[1], "grant header network_id")?,
            profile_id: ascii_identifier(fields[2], "grant header profile_id")?,
            profile_content_hash: fixed::<32>(fields[3], "grant header profile hash")?,
            expected_ledger_position: u64_field(fields[4], "grant header ledger position")?,
            expected_privacy_revision: u64_field(fields[5], "grant header privacy revision")?,
            parent_ledger_prefix_hash: fixed::<32>(
                fields[6],
                "grant header parent ledger prefix hash",
            )?,
            parent_envelope_hash: decode_parent_envelope_reference(fields[7])?,
            object_id: object_uuid(fields[8], "grant header object identifier")?,
            owner: uuid(fields[9], "grant header owner principal")?,
            grantee: uuid(fields[10], "grant header grantee principal")?,
            permission: PrivacyPermission::decode(fields[11])?,
            wrapping_key,
            protected_content_commitment: fixed::<32>(
                fields[13],
                "grant header protected content commitment",
            )?,
            encrypted_payload_commitment: fixed::<32>(
                fields[14],
                "grant header encrypted payload commitment",
            )?,
            canonical_bytes: bytes.to_vec(),
        };
        if header.encode() != bytes {
            return Err(PrivacyError::NonCanonicalRecord);
        }
        Ok(header)
    }

    /// Exact canonical grant-header bytes.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        self.canonical_bytes.clone()
    }

    /// Network identity bound into this delivery.
    pub fn network_id(&self) -> &str {
        &self.network_id
    }

    /// Profile identity bound into this delivery.
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    /// Profile content hash bound into this delivery.
    pub fn profile_content_hash(&self) -> [u8; 32] {
        self.profile_content_hash
    }

    /// Ledger position at which the grant transition must commit.
    pub fn expected_ledger_position(&self) -> u64 {
        self.expected_ledger_position
    }

    /// Privacy revision at which the grant transition must commit.
    pub fn expected_privacy_revision(&self) -> u64 {
        self.expected_privacy_revision
    }

    /// Exact parent ledger prefix covered by this delivery.
    pub fn parent_ledger_prefix_hash(&self) -> [u8; 32] {
        self.parent_ledger_prefix_hash
    }

    /// Exact parent envelope hash, or `None` for a genesis anchor.
    pub fn parent_envelope_hash(&self) -> Option<[u8; 32]> {
        self.parent_envelope_hash
    }

    /// Protected object addressed by this grant.
    pub fn object_id(&self) -> Uuid {
        self.object_id
    }

    /// Owner derived from the protected object parent state.
    pub fn owner(&self) -> Uuid {
        self.owner
    }

    /// Participant receiving the grant.
    pub fn grantee(&self) -> Uuid {
        self.grantee
    }

    /// Permission carried by this grant.
    pub fn permission(&self) -> PrivacyPermission {
        self.permission
    }

    /// Exact grantee wrapping-key reference addressed by the envelope.
    pub fn wrapping_key(&self) -> &ParticipantKeyReference {
        &self.wrapping_key
    }

    /// Protected-content commitment copied from the immutable object.
    pub fn protected_content_commitment(&self) -> [u8; 32] {
        self.protected_content_commitment
    }

    /// Encrypted-payload commitment copied from the immutable object.
    pub fn encrypted_payload_commitment(&self) -> [u8; 32] {
        self.encrypted_payload_commitment
    }

    fn anchor(&self) -> Result<PrivacyAdmissionAnchor, PrivacyError> {
        PrivacyAdmissionAnchor::new(
            self.network_id.clone(),
            self.profile_id.clone(),
            self.profile_content_hash,
            self.expected_ledger_position,
            self.expected_privacy_revision,
            self.parent_ledger_prefix_hash,
            self.parent_envelope_hash,
        )
    }

    fn encode(&self) -> Vec<u8> {
        encode_record(
            GRANT_DEK_HEADER_TAG,
            vec![
                PROTECTED_DATA_SUITE_V1.as_bytes().to_vec(),
                self.network_id.as_bytes().to_vec(),
                self.profile_id.as_bytes().to_vec(),
                self.profile_content_hash.to_vec(),
                self.expected_ledger_position.to_be_bytes().to_vec(),
                self.expected_privacy_revision.to_be_bytes().to_vec(),
                self.parent_ledger_prefix_hash.to_vec(),
                encode_parent_envelope_reference(self.parent_envelope_hash).to_vec(),
                self.object_id.as_bytes().to_vec(),
                self.owner.as_bytes().to_vec(),
                self.grantee.as_bytes().to_vec(),
                vec![self.permission as u8],
                self.wrapping_key.canonical_bytes(),
                self.protected_content_commitment.to_vec(),
                self.encrypted_payload_commitment.to_vec(),
            ],
        )
    }
}

/// Canonical opaque HPKE envelope delivering the existing object DEK to a grantee.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantDekEnvelope {
    grant_delivery_context: [u8; 32],
    header: GrantDekHeader,
    encapsulated_key: [u8; 65],
    wrapped_dek_ciphertext: [u8; 48],
    canonical_bytes: Vec<u8>,
}

impl GrantDekEnvelope {
    /// Strictly decode one `GrantDekEnvelopeV1`.
    pub fn decode(bytes: &[u8]) -> Result<Self, PrivacyError> {
        if bytes.len() > MAX_GRANT_ENVELOPE_BYTES {
            return Err(PrivacyError::Malformed(format!(
                "grant DEK envelope exceeds {MAX_GRANT_ENVELOPE_BYTES} bytes"
            )));
        }
        let fields = decode_record(bytes, GRANT_DEK_ENVELOPE_TAG, 5)?;
        if fields[0] != PROTECTED_DATA_SUITE_V1.as_bytes() {
            return Err(PrivacyError::Malformed(
                "unsupported grant DEK envelope suite".to_string(),
            ));
        }
        let header = GrantDekHeader::decode(fields[2])?;
        let grant_delivery_context = fixed::<32>(fields[1], "grant delivery context")?;
        let expected_context = grant_delivery_context_digest(&header)?;
        if grant_delivery_context != expected_context {
            return Err(PrivacyError::Malformed(
                "grant delivery context mismatch".to_string(),
            ));
        }
        let encapsulated_key = fixed::<65>(fields[3], "grant HPKE encapsulated P-256 key")?;
        if encapsulated_key[0] != 0x04
            || P256VerifyingKey::from_sec1_bytes(&encapsulated_key).is_err()
        {
            return Err(PrivacyError::Malformed(
                "grant HPKE encapsulated key is not a canonical uncompressed P-256 point"
                    .to_string(),
            ));
        }
        let envelope = Self {
            grant_delivery_context,
            header,
            encapsulated_key,
            wrapped_dek_ciphertext: fixed::<48>(fields[4], "grant wrapped DEK ciphertext")?,
            canonical_bytes: bytes.to_vec(),
        };
        if envelope.encode() != bytes {
            return Err(PrivacyError::NonCanonicalRecord);
        }
        Ok(envelope)
    }

    /// Exact canonical grant-envelope bytes used in the transition and release response.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        self.canonical_bytes.clone()
    }

    /// Exact delivery-context digest `Q`.
    pub fn grant_delivery_context(&self) -> [u8; 32] {
        self.grant_delivery_context
    }

    /// Canonical header addressed by this delivery.
    pub fn header(&self) -> &GrantDekHeader {
        &self.header
    }

    /// Exact recipient wrapping key reference.
    pub fn wrapping_key(&self) -> &ParticipantKeyReference {
        self.header.wrapping_key()
    }

    /// Exact HPKE encapsulated public key.
    pub(crate) fn encapsulated_key(&self) -> [u8; 65] {
        self.encapsulated_key
    }

    /// Exact opaque wrapped-DEK ciphertext.
    pub(crate) fn wrapped_dek_ciphertext(&self) -> [u8; 48] {
        self.wrapped_dek_ciphertext
    }

    /// HPKE info bytes bound to `Q` and the complete grant header.
    pub(crate) fn hpke_info(&self) -> [u8; 64] {
        grant_hpke_info(self.grant_delivery_context, &self.header)
    }

    pub(crate) fn new(
        grant_delivery_context: [u8; 32],
        header: GrantDekHeader,
        encapsulated_key: [u8; 65],
        wrapped_dek_ciphertext: [u8; 48],
    ) -> Result<Self, PrivacyError> {
        let bytes = encode_record(
            GRANT_DEK_ENVELOPE_TAG,
            vec![
                PROTECTED_DATA_SUITE_V1.as_bytes().to_vec(),
                grant_delivery_context.to_vec(),
                header.canonical_bytes(),
                encapsulated_key.to_vec(),
                wrapped_dek_ciphertext.to_vec(),
            ],
        );
        Self::decode(&bytes)
    }

    fn encode(&self) -> Vec<u8> {
        encode_record(
            GRANT_DEK_ENVELOPE_TAG,
            vec![
                PROTECTED_DATA_SUITE_V1.as_bytes().to_vec(),
                self.grant_delivery_context.to_vec(),
                self.header.canonical_bytes(),
                self.encapsulated_key.to_vec(),
                self.wrapped_dek_ciphertext.to_vec(),
            ],
        )
    }
}

/// The one opaque recipient envelope returned by a live privacy release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrivacyReleaseEnvelope {
    /// The immutable owner envelope committed with the object.
    Owner(OwnerDekEnvelope),
    /// The immutable grant envelope committed with the GrantAccess transition.
    Grant(GrantDekEnvelope),
}

impl PrivacyReleaseEnvelope {
    /// Exact canonical bytes of the single opaque recipient envelope.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        match self {
            Self::Owner(envelope) => envelope.canonical_bytes(),
            Self::Grant(envelope) => envelope.canonical_bytes(),
        }
    }
}

/// Recipient path selected for a live protected-object release.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LivePrivacyReleasePath {
    /// Release to the immutable object owner envelope.
    Owner = 0x01,
    /// Release to one currently active privacy grant.
    Grantee = 0x02,
}

impl LivePrivacyReleasePath {
    fn decode(bytes: &[u8]) -> Result<Self, PrivacyError> {
        match fixed::<1>(bytes, "live release path")?[0] {
            0x01 => Ok(Self::Owner),
            0x02 => Ok(Self::Grantee),
            value => Err(PrivacyError::Malformed(format!(
                "unsupported live release path value {value}"
            ))),
        }
    }
}

/// A client-supplied request for one converged opaque protected-object release.
///
/// The request carries only public identifiers and a recipient key reference. It
/// never carries plaintext, a DEK, or private key material; the ledger validates
/// it against its current journal-derived state at the Network-Converged barrier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LivePrivacyReleaseRequest {
    object_id: Uuid,
    requester: Uuid,
    recipient_key: ParticipantKeyReference,
    path: LivePrivacyReleasePath,
    grant_id: Option<[u8; 32]>,
}

impl LivePrivacyReleaseRequest {
    /// Request release to the immutable owner envelope.
    pub fn owner(
        object_id: Uuid,
        requester: Uuid,
        recipient_key: ParticipantKeyReference,
    ) -> Result<Self, PrivacyError> {
        validate_object_id(object_id)?;
        validate_principal(requester)?;
        if recipient_key.purpose != ParticipantKeyPurpose::PrivacyKeyWrapping
            || recipient_key.principal != requester
        {
            return Err(PrivacyError::Malformed(
                "release recipient must be a privacy-key-wrapping key".to_string(),
            ));
        }
        Ok(Self {
            object_id,
            requester,
            recipient_key,
            path: LivePrivacyReleasePath::Owner,
            grant_id: None,
        })
    }

    /// Request release through one exact active grant envelope.
    pub fn grantee(
        object_id: Uuid,
        requester: Uuid,
        recipient_key: ParticipantKeyReference,
        grant_id: [u8; 32],
    ) -> Result<Self, PrivacyError> {
        let mut request = Self::owner(object_id, requester, recipient_key)?;
        if grant_id == [0; 32] {
            return Err(PrivacyError::Malformed(
                "release grant identifier cannot be empty".to_string(),
            ));
        }
        request.path = LivePrivacyReleasePath::Grantee;
        request.grant_id = Some(grant_id);
        Ok(request)
    }

    /// Protected object requested by this release.
    pub fn object_id(&self) -> Uuid {
        self.object_id
    }

    /// Participant requesting the release.
    pub fn requester(&self) -> Uuid {
        self.requester
    }

    /// Exact recipient wrapping-key reference selected by the client.
    pub fn recipient_key(&self) -> &ParticipantKeyReference {
        &self.recipient_key
    }

    /// Requested owner or grantee release path.
    pub fn path(&self) -> LivePrivacyReleasePath {
        self.path
    }

    /// Exact grant identifier for a grantee path, or `None` for an owner path.
    pub fn grant_id(&self) -> Option<[u8; 32]> {
        self.grant_id
    }
}

/// Canonical evidence binding one opaque release to an exact converged prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LivePrivacyReleaseEvidence {
    /// Owner release evidence; no grant identifier is present.
    Owner {
        network_id: String,
        profile_id: String,
        profile_content_hash: [u8; 32],
        converged_ledger_position: u64,
        ledger_prefix_hash: [u8; 32],
        object_id: Uuid,
        requester: Uuid,
        recipient_key: ParticipantKeyReference,
    },
    /// Active-grantee release evidence bound to one exact immutable grant ID.
    Grantee {
        network_id: String,
        profile_id: String,
        profile_content_hash: [u8; 32],
        converged_ledger_position: u64,
        ledger_prefix_hash: [u8; 32],
        object_id: Uuid,
        requester: Uuid,
        recipient_key: ParticipantKeyReference,
        grant_id: [u8; 32],
    },
}

impl LivePrivacyReleaseEvidence {
    /// Construct exact owner-path release evidence.
    #[allow(clippy::too_many_arguments)]
    pub fn owner(
        network_id: impl Into<String>,
        profile_id: impl Into<String>,
        profile_content_hash: [u8; 32],
        converged_ledger_position: u64,
        ledger_prefix_hash: [u8; 32],
        object_id: Uuid,
        requester: Uuid,
        recipient_key: ParticipantKeyReference,
    ) -> Result<Self, PrivacyError> {
        let network_id = network_id.into();
        let profile_id = profile_id.into();
        validate_identifier(&network_id, "release network_id")?;
        validate_identifier(&profile_id, "release profile_id")?;
        if profile_content_hash == [0; 32] || ledger_prefix_hash == [0; 32] {
            return Err(PrivacyError::Malformed(
                "release evidence contains an empty or invalid commitment".to_string(),
            ));
        }
        validate_object_id(object_id)?;
        validate_principal(requester)?;
        if recipient_key.purpose != ParticipantKeyPurpose::PrivacyKeyWrapping
            || recipient_key.principal != requester
        {
            return Err(PrivacyError::Malformed(
                "release recipient must be a privacy-key-wrapping key".to_string(),
            ));
        }
        Ok(Self::Owner {
            network_id,
            profile_id,
            profile_content_hash,
            converged_ledger_position,
            ledger_prefix_hash,
            object_id,
            requester,
            recipient_key,
        })
    }

    /// Construct exact grantee-path release evidence for one grant identifier.
    #[allow(clippy::too_many_arguments)]
    pub fn grantee(
        network_id: impl Into<String>,
        profile_id: impl Into<String>,
        profile_content_hash: [u8; 32],
        converged_ledger_position: u64,
        ledger_prefix_hash: [u8; 32],
        object_id: Uuid,
        requester: Uuid,
        recipient_key: ParticipantKeyReference,
        grant_id: [u8; 32],
    ) -> Result<Self, PrivacyError> {
        let evidence = Self::owner(
            network_id,
            profile_id,
            profile_content_hash,
            converged_ledger_position,
            ledger_prefix_hash,
            object_id,
            requester,
            recipient_key,
        )?;
        let Self::Owner {
            network_id,
            profile_id,
            profile_content_hash,
            converged_ledger_position,
            ledger_prefix_hash,
            object_id,
            requester,
            recipient_key,
        } = evidence
        else {
            unreachable!("owner constructor returns the owner evidence variant")
        };
        if grant_id == [0; 32] {
            return Err(PrivacyError::Malformed(
                "release grant identifier cannot be empty".to_string(),
            ));
        }
        Ok(Self::Grantee {
            network_id,
            profile_id,
            profile_content_hash,
            converged_ledger_position,
            ledger_prefix_hash,
            object_id,
            requester,
            recipient_key,
            grant_id,
        })
    }

    /// Release path encoded by this evidence.
    pub fn path(&self) -> LivePrivacyReleasePath {
        match self {
            Self::Owner { .. } => LivePrivacyReleasePath::Owner,
            Self::Grantee { .. } => LivePrivacyReleasePath::Grantee,
        }
    }

    /// Network identity covered by this evidence.
    pub fn network_id(&self) -> &str {
        match self {
            Self::Owner { network_id, .. } | Self::Grantee { network_id, .. } => network_id,
        }
    }

    /// Profile identity covered by this evidence.
    pub fn profile_id(&self) -> &str {
        match self {
            Self::Owner { profile_id, .. } | Self::Grantee { profile_id, .. } => profile_id,
        }
    }

    /// Profile content hash covered by this evidence.
    pub fn profile_content_hash(&self) -> [u8; 32] {
        match self {
            Self::Owner {
                profile_content_hash,
                ..
            }
            | Self::Grantee {
                profile_content_hash,
                ..
            } => *profile_content_hash,
        }
    }

    /// Exact converged ledger position covered by this evidence.
    pub fn converged_ledger_position(&self) -> u64 {
        match self {
            Self::Owner {
                converged_ledger_position,
                ..
            }
            | Self::Grantee {
                converged_ledger_position,
                ..
            } => *converged_ledger_position,
        }
    }

    /// Exact ordered ledger prefix hash covered by this evidence.
    pub fn ledger_prefix_hash(&self) -> [u8; 32] {
        match self {
            Self::Owner {
                ledger_prefix_hash, ..
            }
            | Self::Grantee {
                ledger_prefix_hash, ..
            } => *ledger_prefix_hash,
        }
    }

    /// Protected object addressed by this evidence.
    pub fn object_id(&self) -> Uuid {
        match self {
            Self::Owner { object_id, .. } | Self::Grantee { object_id, .. } => *object_id,
        }
    }

    /// Requesting participant bound into this evidence.
    pub fn requester(&self) -> Uuid {
        match self {
            Self::Owner { requester, .. } | Self::Grantee { requester, .. } => *requester,
        }
    }

    /// Exact recipient key selected for the release.
    pub fn recipient_key(&self) -> &ParticipantKeyReference {
        match self {
            Self::Owner { recipient_key, .. } | Self::Grantee { recipient_key, .. } => {
                recipient_key
            }
        }
    }

    /// Grant ID for a grantee release, or `None` for an owner release.
    pub fn grant_id(&self) -> Option<[u8; 32]> {
        match self {
            Self::Owner { .. } => None,
            Self::Grantee { grant_id, .. } => Some(*grant_id),
        }
    }

    /// Exact canonical evidence record bytes.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        match self {
            Self::Owner {
                network_id,
                profile_id,
                profile_content_hash,
                converged_ledger_position,
                ledger_prefix_hash,
                object_id,
                requester,
                recipient_key,
            } => encode_record(
                OWNER_RELEASE_EVIDENCE_TAG,
                vec![
                    network_id.as_bytes().to_vec(),
                    profile_id.as_bytes().to_vec(),
                    profile_content_hash.to_vec(),
                    converged_ledger_position.to_be_bytes().to_vec(),
                    ledger_prefix_hash.to_vec(),
                    object_id.as_bytes().to_vec(),
                    requester.as_bytes().to_vec(),
                    recipient_key.canonical_bytes(),
                    vec![LivePrivacyReleasePath::Owner as u8],
                ],
            ),
            Self::Grantee {
                network_id,
                profile_id,
                profile_content_hash,
                converged_ledger_position,
                ledger_prefix_hash,
                object_id,
                requester,
                recipient_key,
                grant_id,
            } => encode_record(
                GRANTEE_RELEASE_EVIDENCE_TAG,
                vec![
                    network_id.as_bytes().to_vec(),
                    profile_id.as_bytes().to_vec(),
                    profile_content_hash.to_vec(),
                    converged_ledger_position.to_be_bytes().to_vec(),
                    ledger_prefix_hash.to_vec(),
                    object_id.as_bytes().to_vec(),
                    requester.as_bytes().to_vec(),
                    recipient_key.canonical_bytes(),
                    vec![LivePrivacyReleasePath::Grantee as u8],
                    grant_id.to_vec(),
                ],
            ),
        }
    }

    /// Strictly decode one owner or grantee release evidence record.
    pub fn decode(bytes: &[u8]) -> Result<Self, PrivacyError> {
        if bytes.len() > MAX_LIVE_RELEASE_EVIDENCE_BYTES {
            return Err(PrivacyError::Malformed(format!(
                "live release evidence exceeds {MAX_LIVE_RELEASE_EVIDENCE_BYTES} bytes"
            )));
        }
        let evidence = match bytes.get(4).copied() {
            Some(OWNER_RELEASE_EVIDENCE_TAG) => {
                let fields = decode_record(bytes, OWNER_RELEASE_EVIDENCE_TAG, 9)?;
                if LivePrivacyReleasePath::decode(fields[8])? != LivePrivacyReleasePath::Owner {
                    return Err(PrivacyError::Malformed(
                        "owner release evidence has the wrong path".to_string(),
                    ));
                }
                Self::owner(
                    ascii_identifier(fields[0], "release network_id")?,
                    ascii_identifier(fields[1], "release profile_id")?,
                    fixed::<32>(fields[2], "release profile hash")?,
                    u64_field(fields[3], "release ledger position")?,
                    fixed::<32>(fields[4], "release prefix hash")?,
                    object_uuid(fields[5], "release object identifier")?,
                    uuid(fields[6], "release requester")?,
                    decode_participant_key_reference(fields[7])?,
                )?
            }
            Some(GRANTEE_RELEASE_EVIDENCE_TAG) => {
                let fields = decode_record(bytes, GRANTEE_RELEASE_EVIDENCE_TAG, 10)?;
                if LivePrivacyReleasePath::decode(fields[8])? != LivePrivacyReleasePath::Grantee {
                    return Err(PrivacyError::Malformed(
                        "grantee release evidence has the wrong path".to_string(),
                    ));
                }
                Self::grantee(
                    ascii_identifier(fields[0], "release network_id")?,
                    ascii_identifier(fields[1], "release profile_id")?,
                    fixed::<32>(fields[2], "release profile hash")?,
                    u64_field(fields[3], "release ledger position")?,
                    fixed::<32>(fields[4], "release prefix hash")?,
                    object_uuid(fields[5], "release object identifier")?,
                    uuid(fields[6], "release requester")?,
                    decode_participant_key_reference(fields[7])?,
                    fixed::<32>(fields[9], "release grant identifier")?,
                )?
            }
            Some(tag) => return Err(PrivacyError::UnknownTransitionRecord(tag)),
            None => {
                return Err(PrivacyError::Malformed(
                    "empty live release evidence".to_string(),
                ))
            }
        };
        if evidence.canonical_bytes() != bytes {
            return Err(PrivacyError::NonCanonicalRecord);
        }
        Ok(evidence)
    }
}

/// Exact opaque response for a converged live privacy release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LivePrivacyReleaseResponse {
    evidence: LivePrivacyReleaseEvidence,
    payload: ProtectedPayloadCiphertext,
    envelope: PrivacyReleaseEnvelope,
    encrypted_payload_commitment: [u8; 32],
    canonical_bytes: Vec<u8>,
}

impl LivePrivacyReleaseResponse {
    /// Construct and cross-check one release response without opening its secrets.
    pub fn new(
        evidence: LivePrivacyReleaseEvidence,
        payload: ProtectedPayloadCiphertext,
        envelope: PrivacyReleaseEnvelope,
        encoded_encrypted_payload_commitment: [u8; 32],
    ) -> Result<Self, PrivacyError> {
        if encoded_encrypted_payload_commitment != encrypted_payload_commitment(&payload) {
            return Err(PrivacyError::Malformed(
                "live release encrypted-payload commitment mismatch".to_string(),
            ));
        }
        match (&evidence, &envelope) {
            (
                LivePrivacyReleaseEvidence::Owner {
                    object_id,
                    requester,
                    recipient_key,
                    ..
                },
                PrivacyReleaseEnvelope::Owner(envelope),
            ) if envelope.header.object_id == *object_id
                && envelope.header.owner == *requester
                && envelope.header.wrapping_key == *recipient_key
                && envelope.header.protected_content_commitment
                    == payload.protected_content_commitment
                && envelope.header.encrypted_payload_commitment
                    == encoded_encrypted_payload_commitment => {}
            (
                LivePrivacyReleaseEvidence::Grantee {
                    object_id,
                    requester,
                    recipient_key,
                    ..
                },
                PrivacyReleaseEnvelope::Grant(envelope),
            ) if envelope.header.object_id == *object_id
                && envelope.header.grantee == *requester
                && envelope.header.wrapping_key == *recipient_key
                && envelope.header.protected_content_commitment
                    == payload.protected_content_commitment
                && envelope.header.encrypted_payload_commitment
                    == encoded_encrypted_payload_commitment => {}
            _ => {
                return Err(PrivacyError::Malformed(
                    "release evidence and recipient envelope do not match".to_string(),
                ))
            }
        }
        let bytes = encode_record(
            LIVE_RELEASE_RESPONSE_TAG,
            vec![
                evidence.canonical_bytes(),
                payload.canonical_bytes(),
                envelope.canonical_bytes(),
                encoded_encrypted_payload_commitment.to_vec(),
            ],
        );
        if bytes.len() > MAX_LIVE_RELEASE_RESPONSE_BYTES {
            return Err(PrivacyError::Malformed(
                "live release response exceeds the protected response bound".to_string(),
            ));
        }
        Ok(Self {
            evidence,
            payload,
            envelope,
            encrypted_payload_commitment: encoded_encrypted_payload_commitment,
            canonical_bytes: bytes,
        })
    }

    /// Strictly decode one opaque live-release response.
    pub fn decode(bytes: &[u8]) -> Result<Self, PrivacyError> {
        if bytes.len() > MAX_LIVE_RELEASE_RESPONSE_BYTES {
            return Err(PrivacyError::Malformed(
                "live release response exceeds the protected response bound".to_string(),
            ));
        }
        let fields = decode_record(bytes, LIVE_RELEASE_RESPONSE_TAG, 4)?;
        let evidence = LivePrivacyReleaseEvidence::decode(fields[0])?;
        let payload = ProtectedPayloadCiphertext::decode(fields[1])?;
        let envelope = match evidence.path() {
            LivePrivacyReleasePath::Owner => {
                PrivacyReleaseEnvelope::Owner(OwnerDekEnvelope::decode(fields[2])?)
            }
            LivePrivacyReleasePath::Grantee => {
                PrivacyReleaseEnvelope::Grant(GrantDekEnvelope::decode(fields[2])?)
            }
        };
        let response = Self::new(
            evidence,
            payload,
            envelope,
            fixed::<32>(fields[3], "release encrypted-payload commitment")?,
        )?;
        if response.canonical_bytes != bytes {
            return Err(PrivacyError::NonCanonicalRecord);
        }
        Ok(response)
    }

    /// Exact response bytes; no plaintext, DEK, or private key is included.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        self.canonical_bytes.clone()
    }

    /// Exact converged release evidence.
    pub fn evidence(&self) -> &LivePrivacyReleaseEvidence {
        &self.evidence
    }

    /// Immutable encrypted payload returned by the release.
    pub fn payload(&self) -> &ProtectedPayloadCiphertext {
        &self.payload
    }

    /// Exactly one opaque owner or grant envelope returned by the release.
    pub fn envelope(&self) -> &PrivacyReleaseEnvelope {
        &self.envelope
    }

    /// Encrypted-payload commitment repeated in the response.
    pub fn encrypted_payload_commitment(&self) -> [u8; 32] {
        self.encrypted_payload_commitment
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CreateProtectedObjectCore {
    anchor: PrivacyAdmissionAnchor,
    object_id: Uuid,
    owner: Uuid,
    authorization_key: ParticipantKeyReference,
    wrapping_key: ParticipantKeyReference,
    object_context: [u8; 32],
    payload: ProtectedPayloadCiphertext,
    encrypted_payload_commitment: [u8; 32],
    owner_envelope: OwnerDekEnvelope,
}

impl CreateProtectedObjectCore {
    fn canonical_bytes(&self) -> Vec<u8> {
        let mut fields = self
            .anchor
            .common_fields(TransitionVariant::CreateProtectedObject);
        fields.extend([
            self.object_id.as_bytes().to_vec(),
            self.owner.as_bytes().to_vec(),
            self.authorization_key.canonical_bytes(),
            self.wrapping_key.canonical_bytes(),
            self.object_context.to_vec(),
            self.payload.canonical_bytes(),
            self.encrypted_payload_commitment.to_vec(),
            self.owner_envelope.canonical_bytes(),
            vec![0x02],
        ]);
        encode_record(CREATE_OBJECT_CORE_TAG, fields)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GrantAccessCore {
    anchor: PrivacyAdmissionAnchor,
    object_id: Uuid,
    owner: Uuid,
    grantee: Uuid,
    permission: PrivacyPermission,
    delivery: GrantDekEnvelope,
    authorization_key: ParticipantKeyReference,
}

impl GrantAccessCore {
    fn canonical_bytes(&self) -> Vec<u8> {
        let mut fields = self.anchor.common_fields(TransitionVariant::GrantAccess);
        fields.extend([
            self.object_id.as_bytes().to_vec(),
            self.owner.as_bytes().to_vec(),
            self.grantee.as_bytes().to_vec(),
            vec![self.permission as u8],
            self.delivery.canonical_bytes(),
            vec![0x02],
            self.authorization_key.canonical_bytes(),
        ]);
        encode_record(GRANT_ACCESS_CORE_TAG, fields)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RevokeGrantCore {
    anchor: PrivacyAdmissionAnchor,
    grant_id: [u8; 32],
    object_id: Uuid,
    grantee: Uuid,
    owner: Uuid,
    permission: PrivacyPermission,
    creation_ledger_position: u64,
    authorization_key: ParticipantKeyReference,
}

impl RevokeGrantCore {
    fn canonical_bytes(&self) -> Vec<u8> {
        let mut fields = self.anchor.common_fields(TransitionVariant::RevokeGrant);
        fields.extend([
            self.grant_id.to_vec(),
            self.object_id.as_bytes().to_vec(),
            self.grantee.as_bytes().to_vec(),
            self.owner.as_bytes().to_vec(),
            vec![self.permission as u8],
            self.creation_ledger_position.to_be_bytes().to_vec(),
            vec![0x02],
            self.authorization_key.canonical_bytes(),
        ]);
        encode_record(REVOKE_GRANT_CORE_TAG, fields)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegisterPrincipalCore {
    anchor: PrivacyAdmissionAnchor,
    principal: Uuid,
    initial_authorization_key: ParticipantKeyBinding,
    bootstrap_governance_key: BootstrapGovernanceKeyReference,
}

impl RegisterPrincipalCore {
    fn canonical_bytes(&self) -> Vec<u8> {
        let mut fields = self
            .anchor
            .common_fields(TransitionVariant::RegisterPrincipal);
        fields.extend([
            self.principal.as_bytes().to_vec(),
            self.initial_authorization_key.canonical_bytes(),
            vec![0x01],
            self.bootstrap_governance_key.canonical_bytes(),
        ]);
        encode_record(REGISTER_CORE_TAG, fields)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BindParticipantKeyCore {
    anchor: PrivacyAdmissionAnchor,
    affected_principal: Uuid,
    introduced_key: ParticipantKeyBinding,
    authorizer_key: ParticipantKeyReference,
}

impl BindParticipantKeyCore {
    fn canonical_bytes(&self) -> Vec<u8> {
        let mut fields = self
            .anchor
            .common_fields(TransitionVariant::BindParticipantKey);
        fields.extend([
            self.affected_principal.as_bytes().to_vec(),
            self.introduced_key.canonical_bytes(),
            vec![0x02],
            self.authorizer_key.canonical_bytes(),
        ]);
        encode_record(BIND_CORE_TAG, fields)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RevokeParticipantKeyCore {
    anchor: PrivacyAdmissionAnchor,
    affected_principal: Uuid,
    target_key: ParticipantKeyLocator,
    authorizer_key: ParticipantKeyReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParticipantKeyLocator {
    principal: Uuid,
    purpose: ParticipantKeyPurpose,
    version: u32,
}

impl RevokeParticipantKeyCore {
    fn canonical_bytes(&self) -> Vec<u8> {
        let mut fields = self
            .anchor
            .common_fields(TransitionVariant::RevokeParticipantKey);
        fields.extend([
            self.affected_principal.as_bytes().to_vec(),
            self.target_key.principal.as_bytes().to_vec(),
            vec![self.target_key.purpose as u8],
            self.target_key.version.to_be_bytes().to_vec(),
            vec![0x02],
            self.authorizer_key.canonical_bytes(),
        ]);
        encode_record(REVOKE_CORE_TAG, fields)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum UnsignedTransitionKind {
    RegisterPrincipal(RegisterPrincipalCore),
    BindParticipantKey(BindParticipantKeyCore),
    RevokeParticipantKey(RevokeParticipantKeyCore),
    CreateProtectedObject(Box<CreateProtectedObjectCore>),
    GrantAccess(Box<GrantAccessCore>),
    RevokeGrant(Box<RevokeGrantCore>),
}

/// A complete unsigned core plus its exact domain-separated proof inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsignedPrivacyTransition {
    kind: UnsignedTransitionKind,
    core_bytes: Vec<u8>,
    transition_id: [u8; 32],
    authorization_digest: [u8; 32],
    possession_digest: Option<[u8; 32]>,
}

impl UnsignedPrivacyTransition {
    pub(crate) fn from_prepared_core(core_bytes: &[u8]) -> Result<Self, PrivacyError> {
        let kind = match core_bytes.get(4).copied() {
            Some(REGISTER_CORE_TAG) => {
                UnsignedTransitionKind::RegisterPrincipal(decode_register_core(core_bytes)?)
            }
            Some(BIND_CORE_TAG) => {
                UnsignedTransitionKind::BindParticipantKey(decode_bind_core(core_bytes)?)
            }
            _ => {
                return Err(PrivacyError::Malformed(
                    "prepared custody request contains an unsupported unsigned core".to_string(),
                ));
            }
        };
        let (transition_domain, authorization_domain, possession_domain) = match &kind {
            UnsignedTransitionKind::RegisterPrincipal(_) => (
                REGISTER_TRANSITION_ID_DOMAIN,
                REGISTER_AUTHORIZATION_DOMAIN,
                REGISTER_POSSESSION_DOMAIN,
            ),
            UnsignedTransitionKind::BindParticipantKey(_) => (
                BIND_TRANSITION_ID_DOMAIN,
                BIND_AUTHORIZATION_DOMAIN,
                BIND_POSSESSION_DOMAIN,
            ),
            _ => unreachable!("closed prepared key-binding variants"),
        };
        let core_bytes = core_bytes.to_vec();
        let transition_id = domain_hash(transition_domain, &core_bytes);
        let proof_payload = concat_digest_and_record(transition_id, &core_bytes);
        Ok(Self {
            kind,
            core_bytes,
            transition_id,
            authorization_digest: domain_hash(authorization_domain, &proof_payload),
            possession_digest: Some(domain_hash(possession_domain, &proof_payload)),
        })
    }

    pub(crate) fn core_bytes(&self) -> &[u8] {
        &self.core_bytes
    }

    pub(crate) fn introduced_key_reference(&self) -> Option<&ParticipantKeyReference> {
        match &self.kind {
            UnsignedTransitionKind::RegisterPrincipal(core) => {
                Some(&core.initial_authorization_key.reference)
            }
            UnsignedTransitionKind::BindParticipantKey(core) => {
                Some(&core.introduced_key.reference)
            }
            _ => None,
        }
    }

    pub(crate) fn introduced_key_public_bytes(&self) -> Option<&[u8]> {
        match &self.kind {
            UnsignedTransitionKind::RegisterPrincipal(core) => {
                Some(core.initial_authorization_key.public_key.as_slice())
            }
            UnsignedTransitionKind::BindParticipantKey(core) => {
                Some(core.introduced_key.public_key.as_slice())
            }
            _ => None,
        }
    }

    pub(crate) fn is_registration(&self) -> bool {
        matches!(self.kind, UnsignedTransitionKind::RegisterPrincipal(_))
    }

    pub(crate) fn admission_anchor(&self) -> &PrivacyAdmissionAnchor {
        match &self.kind {
            UnsignedTransitionKind::RegisterPrincipal(core) => &core.anchor,
            UnsignedTransitionKind::BindParticipantKey(core) => &core.anchor,
            UnsignedTransitionKind::RevokeParticipantKey(core) => &core.anchor,
            UnsignedTransitionKind::CreateProtectedObject(core) => &core.anchor,
            UnsignedTransitionKind::GrantAccess(core) => &core.anchor,
            UnsignedTransitionKind::RevokeGrant(core) => &core.anchor,
        }
    }

    pub(crate) fn prepared_authorizer_contract(&self) -> Option<(u8, Vec<u8>)> {
        match &self.kind {
            UnsignedTransitionKind::RegisterPrincipal(core) => {
                Some((0x01, core.bootstrap_governance_key.canonical_bytes()))
            }
            UnsignedTransitionKind::BindParticipantKey(core) => {
                Some((0x02, core.authorizer_key.canonical_bytes()))
            }
            _ => None,
        }
    }

    pub(crate) fn verify_stored_possession(&self, proof: [u8; 64]) -> Result<(), PrivacyError> {
        let digest = self.possession_digest.ok_or_else(|| {
            PrivacyError::Malformed("prepared transition has no possession proof".to_string())
        })?;
        match &self.kind {
            UnsignedTransitionKind::RegisterPrincipal(core) => {
                verify_possession_proof(&core.initial_authorization_key, &digest, proof)
            }
            UnsignedTransitionKind::BindParticipantKey(core) => {
                verify_possession_proof(&core.introduced_key, &digest, proof)
            }
            _ => Err(PrivacyError::Malformed(
                "prepared transition is not a key introduction".to_string(),
            )),
        }
    }

    /// Construct an atomic principal plus initial authorization-key registration core.
    pub fn register_principal(
        profile: &PrivacyLifecycleProfile,
        anchor: PrivacyAdmissionAnchor,
        principal: Uuid,
        initial_authorization_public_key: [u8; 32],
    ) -> Result<Self, PrivacyError> {
        profile.validate()?;
        anchor.validate()?;
        if anchor.profile_content_hash != profile.network_profile_content_hash {
            return Err(PrivacyError::Malformed(
                "transition profile content hash does not match the lifecycle profile".to_string(),
            ));
        }
        validate_principal(principal)?;
        if profile.key_reuses_profile_role(&initial_authorization_public_key) {
            return Err(PrivacyError::Malformed(
                "introduced participant key reuses a Network Profile key role".to_string(),
            ));
        }
        let core = RegisterPrincipalCore {
            anchor,
            principal,
            initial_authorization_key: ParticipantKeyBinding::authorization(
                principal,
                1,
                initial_authorization_public_key,
            )?,
            bootstrap_governance_key: profile.bootstrap_reference(),
        };
        let core_bytes = core.canonical_bytes();
        let transition_id = domain_hash(REGISTER_TRANSITION_ID_DOMAIN, &core_bytes);
        let proof_payload = concat_digest_and_record(transition_id, &core_bytes);
        Ok(Self {
            kind: UnsignedTransitionKind::RegisterPrincipal(core),
            core_bytes,
            transition_id,
            authorization_digest: domain_hash(REGISTER_AUTHORIZATION_DOMAIN, &proof_payload),
            possession_digest: Some(domain_hash(REGISTER_POSSESSION_DOMAIN, &proof_payload)),
        })
    }

    /// Construct a later contiguous Ed25519 authorization-key binding core.
    pub fn bind_authorization_key(
        profile: &PrivacyLifecycleProfile,
        anchor: PrivacyAdmissionAnchor,
        affected_principal: Uuid,
        version: u32,
        introduced_public_key: [u8; 32],
        authorizer_key: ParticipantKeyReference,
    ) -> Result<Self, PrivacyError> {
        profile.validate()?;
        anchor.validate()?;
        if anchor.profile_content_hash != profile.network_profile_content_hash {
            return Err(PrivacyError::Malformed(
                "transition profile content hash does not match the lifecycle profile".to_string(),
            ));
        }
        validate_principal(affected_principal)?;
        if profile.key_reuses_profile_role(&introduced_public_key) {
            return Err(PrivacyError::Malformed(
                "introduced participant key reuses a Network Profile key role".to_string(),
            ));
        }
        let core = BindParticipantKeyCore {
            anchor,
            affected_principal,
            introduced_key: ParticipantKeyBinding::authorization(
                affected_principal,
                version,
                introduced_public_key,
            )?,
            authorizer_key,
        };
        let core_bytes = core.canonical_bytes();
        let transition_id = domain_hash(BIND_TRANSITION_ID_DOMAIN, &core_bytes);
        let proof_payload = concat_digest_and_record(transition_id, &core_bytes);
        Ok(Self {
            kind: UnsignedTransitionKind::BindParticipantKey(core),
            core_bytes,
            transition_id,
            authorization_digest: domain_hash(BIND_AUTHORIZATION_DOMAIN, &proof_payload),
            possession_digest: Some(domain_hash(BIND_POSSESSION_DOMAIN, &proof_payload)),
        })
    }

    /// Construct a purpose-separated P-256 wrapping-key binding core.
    pub fn bind_wrapping_key(
        profile: &PrivacyLifecycleProfile,
        anchor: PrivacyAdmissionAnchor,
        affected_principal: Uuid,
        version: u32,
        introduced_public_key: [u8; 65],
        authorizer_key: ParticipantKeyReference,
    ) -> Result<Self, PrivacyError> {
        profile.validate()?;
        anchor.validate()?;
        if anchor.profile_content_hash != profile.network_profile_content_hash {
            return Err(PrivacyError::Malformed(
                "transition profile content hash does not match the lifecycle profile".to_string(),
            ));
        }
        validate_principal(affected_principal)?;
        let core = BindParticipantKeyCore {
            anchor,
            affected_principal,
            introduced_key: ParticipantKeyBinding::wrapping(
                affected_principal,
                version,
                introduced_public_key,
            )?,
            authorizer_key,
        };
        let core_bytes = core.canonical_bytes();
        let transition_id = domain_hash(BIND_TRANSITION_ID_DOMAIN, &core_bytes);
        let proof_payload = concat_digest_and_record(transition_id, &core_bytes);
        Ok(Self {
            kind: UnsignedTransitionKind::BindParticipantKey(core),
            core_bytes,
            transition_id,
            authorization_digest: domain_hash(BIND_AUTHORIZATION_DOMAIN, &proof_payload),
            possession_digest: Some(domain_hash(BIND_POSSESSION_DOMAIN, &proof_payload)),
        })
    }

    /// Construct an emergency Active-to-Revoked lifecycle core.
    pub fn revoke_participant_key(
        profile: &PrivacyLifecycleProfile,
        anchor: PrivacyAdmissionAnchor,
        affected_principal: Uuid,
        target_key: ParticipantKeyReference,
        authorizer_key: ParticipantKeyReference,
    ) -> Result<Self, PrivacyError> {
        profile.validate()?;
        anchor.validate()?;
        if anchor.profile_content_hash != profile.network_profile_content_hash {
            return Err(PrivacyError::Malformed(
                "transition profile content hash does not match the lifecycle profile".to_string(),
            ));
        }
        validate_principal(affected_principal)?;
        let core = RevokeParticipantKeyCore {
            anchor,
            affected_principal,
            target_key: ParticipantKeyLocator {
                principal: target_key.principal,
                purpose: target_key.purpose,
                version: target_key.version,
            },
            authorizer_key,
        };
        let core_bytes = core.canonical_bytes();
        let transition_id = domain_hash(REVOKE_TRANSITION_ID_DOMAIN, &core_bytes);
        let proof_payload = concat_digest_and_record(transition_id, &core_bytes);
        Ok(Self {
            kind: UnsignedTransitionKind::RevokeParticipantKey(core),
            core_bytes,
            transition_id,
            authorization_digest: domain_hash(REVOKE_AUTHORIZATION_DOMAIN, &proof_payload),
            possession_digest: None,
        })
    }

    /// Construct the public, keyless-verifiable core for one immutable protected object.
    ///
    /// This validates canonical opaque material and its public commitments. It deliberately
    /// cannot attest to DEK freshness, payload correctness, or HPKE decryptability.
    #[allow(clippy::too_many_arguments)]
    pub fn create_protected_object(
        profile: &PrivacyLifecycleProfile,
        anchor: PrivacyAdmissionAnchor,
        object_id: Uuid,
        owner: Uuid,
        authorization_key: ParticipantKeyReference,
        wrapping_key: ParticipantKeyReference,
        object_context: [u8; 32],
        payload: ProtectedPayloadCiphertext,
        encrypted_payload_commitment: [u8; 32],
        owner_envelope: OwnerDekEnvelope,
    ) -> Result<Self, PrivacyError> {
        profile.validate()?;
        anchor.validate()?;
        if anchor.profile_content_hash != profile.network_profile_content_hash {
            return Err(PrivacyError::Malformed(
                "transition profile content hash does not match the lifecycle profile".to_string(),
            ));
        }
        validate_object_id(object_id)?;
        validate_principal(owner)?;
        if authorization_key.principal != owner
            || authorization_key.purpose != ParticipantKeyPurpose::PrivacyAuthorization
        {
            return Err(PrivacyError::Malformed(
                "object authorizer must be the owner's privacy-authorization key".to_string(),
            ));
        }
        if wrapping_key.principal != owner
            || wrapping_key.purpose != ParticipantKeyPurpose::PrivacyKeyWrapping
        {
            return Err(PrivacyError::Malformed(
                "owner envelope must name the owner's privacy-key-wrapping key".to_string(),
            ));
        }
        validate_protected_object_material(
            &anchor,
            object_id,
            owner,
            &wrapping_key,
            object_context,
            &payload,
            encrypted_payload_commitment,
            &owner_envelope,
        )?;
        let core = CreateProtectedObjectCore {
            anchor,
            object_id,
            owner,
            authorization_key,
            wrapping_key,
            object_context,
            payload,
            encrypted_payload_commitment,
            owner_envelope,
        };
        let core_bytes = core.canonical_bytes();
        let transition_id = domain_hash(CREATE_OBJECT_TRANSITION_ID_DOMAIN, &core_bytes);
        let proof_payload = concat_digest_and_record(transition_id, &core_bytes);
        Ok(Self {
            kind: UnsignedTransitionKind::CreateProtectedObject(Box::new(core)),
            core_bytes,
            transition_id,
            authorization_digest: domain_hash(CREATE_OBJECT_AUTHORIZATION_DOMAIN, &proof_payload),
            possession_digest: None,
        })
    }

    /// Construct an owner-authorized grant that delivers the existing object DEK exactly once.
    ///
    /// The delivery envelope is opaque public material. The reducer binds every field back to
    /// the immutable object and the exact active grantee wrapping key, so this constructor does
    /// not perform or imply server-side decryption.
    #[allow(clippy::too_many_arguments)]
    pub fn grant_access(
        profile: &PrivacyLifecycleProfile,
        anchor: PrivacyAdmissionAnchor,
        object_id: Uuid,
        owner: Uuid,
        grantee: Uuid,
        permission: PrivacyPermission,
        delivery: GrantDekEnvelope,
        authorization_key: ParticipantKeyReference,
    ) -> Result<Self, PrivacyError> {
        profile.validate()?;
        anchor.validate()?;
        if anchor.profile_content_hash != profile.network_profile_content_hash {
            return Err(PrivacyError::Malformed(
                "transition profile content hash does not match the lifecycle profile".to_string(),
            ));
        }
        validate_object_id(object_id)?;
        validate_principal(owner)?;
        validate_principal(grantee)?;
        if owner == grantee {
            return Err(PrivacyError::Malformed(
                "privacy grant cannot target the object owner".to_string(),
            ));
        }
        if authorization_key.principal != owner
            || authorization_key.purpose != ParticipantKeyPurpose::PrivacyAuthorization
        {
            return Err(PrivacyError::Malformed(
                "grant authorizer must be the owner's privacy-authorization key".to_string(),
            ));
        }
        validate_grant_delivery_material(
            &anchor, object_id, owner, grantee, permission, &delivery, None, None,
        )?;
        let core = GrantAccessCore {
            anchor,
            object_id,
            owner,
            grantee,
            permission,
            delivery,
            authorization_key,
        };
        let core_bytes = core.canonical_bytes();
        let grant_id = domain_hash(GRANT_ID_DOMAIN, &core_bytes);
        let proof_payload = concat_digest_and_record(grant_id, &core_bytes);
        Ok(Self {
            kind: UnsignedTransitionKind::GrantAccess(Box::new(core)),
            core_bytes,
            transition_id: grant_id,
            authorization_digest: domain_hash(GRANT_AUTHORIZATION_DOMAIN, &proof_payload),
            possession_digest: None,
        })
    }

    /// Construct an owner-authorized terminal prospective revocation for one grant identifier.
    #[allow(clippy::too_many_arguments)]
    pub fn revoke_grant(
        profile: &PrivacyLifecycleProfile,
        anchor: PrivacyAdmissionAnchor,
        grant_id: [u8; 32],
        object_id: Uuid,
        grantee: Uuid,
        owner: Uuid,
        permission: PrivacyPermission,
        creation_ledger_position: u64,
        authorization_key: ParticipantKeyReference,
    ) -> Result<Self, PrivacyError> {
        profile.validate()?;
        anchor.validate()?;
        if anchor.profile_content_hash != profile.network_profile_content_hash {
            return Err(PrivacyError::Malformed(
                "transition profile content hash does not match the lifecycle profile".to_string(),
            ));
        }
        if grant_id == [0; 32] {
            return Err(PrivacyError::Malformed(
                "grant identifier cannot be empty".to_string(),
            ));
        }
        validate_object_id(object_id)?;
        validate_principal(owner)?;
        validate_principal(grantee)?;
        if authorization_key.principal != owner
            || authorization_key.purpose != ParticipantKeyPurpose::PrivacyAuthorization
        {
            return Err(PrivacyError::Malformed(
                "grant revocation authorizer must be the owner's privacy-authorization key"
                    .to_string(),
            ));
        }
        let core = RevokeGrantCore {
            anchor,
            grant_id,
            object_id,
            grantee,
            owner,
            permission,
            creation_ledger_position,
            authorization_key,
        };
        let core_bytes = core.canonical_bytes();
        let transition_id = domain_hash(REVOKE_GRANT_TRANSITION_ID_DOMAIN, &core_bytes);
        let proof_payload = concat_digest_and_record(transition_id, &core_bytes);
        Ok(Self {
            kind: UnsignedTransitionKind::RevokeGrant(Box::new(core)),
            core_bytes,
            transition_id,
            authorization_digest: domain_hash(REVOKE_GRANT_AUTHORIZATION_DOMAIN, &proof_payload),
            possession_digest: None,
        })
    }

    /// Derived identifier committed by the complete transition.
    pub fn transition_id(&self) -> [u8; 32] {
        self.transition_id
    }

    /// Exact raw 32-byte digest signed by the transition authorizer.
    pub fn authorization_digest(&self) -> [u8; 32] {
        self.authorization_digest
    }

    /// Exact raw possession-proof digest, when the variant introduces a key.
    pub fn possession_digest(&self) -> Option<[u8; 32]> {
        self.possession_digest
    }

    /// Complete a registration with independent governance and introduced-key proofs.
    pub fn complete_registration(
        self,
        governance_signature: [u8; 64],
        possession_signature: [u8; 64],
    ) -> Result<PrivacyControlTransition, PrivacyError> {
        let UnsignedTransitionKind::RegisterPrincipal(core) = self.kind else {
            return Err(PrivacyError::Malformed(
                "registration proofs supplied for a different transition variant".to_string(),
            ));
        };
        Ok(PrivacyControlTransition {
            kind: PrivacyTransitionKind::RegisterPrincipal {
                core,
                transition_id: self.transition_id,
                governance_signature,
                possession_signature,
            },
        })
    }

    /// Complete a key binding with parent authorization and introduced-key possession proofs.
    pub fn complete_binding(
        self,
        authorization_signature: [u8; 64],
        possession_proof: [u8; 64],
    ) -> Result<PrivacyControlTransition, PrivacyError> {
        let UnsignedTransitionKind::BindParticipantKey(core) = self.kind else {
            return Err(PrivacyError::Malformed(
                "binding proofs supplied for a different transition variant".to_string(),
            ));
        };
        Ok(PrivacyControlTransition {
            kind: PrivacyTransitionKind::BindParticipantKey {
                core,
                transition_id: self.transition_id,
                authorization_signature,
                possession_proof,
            },
        })
    }

    /// Complete an emergency key revocation with the parent authorization proof.
    pub fn complete_revocation(
        self,
        authorization_signature: [u8; 64],
    ) -> Result<PrivacyControlTransition, PrivacyError> {
        let UnsignedTransitionKind::RevokeParticipantKey(core) = self.kind else {
            return Err(PrivacyError::Malformed(
                "revocation proof supplied for a different transition variant".to_string(),
            ));
        };
        Ok(PrivacyControlTransition {
            kind: PrivacyTransitionKind::RevokeParticipantKey {
                core,
                transition_id: self.transition_id,
                authorization_signature,
            },
        })
    }

    /// Complete protected-object creation with the owner's exact Ed25519 authorization proof.
    pub fn complete_object_creation(
        self,
        authorization_signature: [u8; 64],
    ) -> Result<PrivacyControlTransition, PrivacyError> {
        let UnsignedTransitionKind::CreateProtectedObject(core) = self.kind else {
            return Err(PrivacyError::Malformed(
                "object-creation proof supplied for a different transition variant".to_string(),
            ));
        };
        Ok(PrivacyControlTransition {
            kind: PrivacyTransitionKind::CreateProtectedObject {
                core,
                transition_id: self.transition_id,
                authorization_signature,
            },
        })
    }

    /// Complete GrantAccess with the owner's exact active authorization proof.
    pub fn complete_grant_access(
        self,
        authorization_signature: [u8; 64],
    ) -> Result<PrivacyControlTransition, PrivacyError> {
        let UnsignedTransitionKind::GrantAccess(core) = self.kind else {
            return Err(PrivacyError::Malformed(
                "grant proof supplied for a different transition variant".to_string(),
            ));
        };
        Ok(PrivacyControlTransition {
            kind: PrivacyTransitionKind::GrantAccess {
                core,
                grant_id: self.transition_id,
                authorization_signature,
            },
        })
    }

    /// Complete RevokeGrant with the owner's exact active authorization proof.
    pub fn complete_grant_revocation(
        self,
        authorization_signature: [u8; 64],
    ) -> Result<PrivacyControlTransition, PrivacyError> {
        let UnsignedTransitionKind::RevokeGrant(core) = self.kind else {
            return Err(PrivacyError::Malformed(
                "grant revocation proof supplied for a different transition variant".to_string(),
            ));
        };
        Ok(PrivacyControlTransition {
            kind: PrivacyTransitionKind::RevokeGrant {
                core,
                transition_id: self.transition_id,
                authorization_signature,
            },
        })
    }
}

/// One closed, canonical public privacy-control transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivacyControlTransition {
    kind: PrivacyTransitionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PrivacyTransitionKind {
    RegisterPrincipal {
        core: RegisterPrincipalCore,
        transition_id: [u8; 32],
        governance_signature: [u8; 64],
        possession_signature: [u8; 64],
    },
    BindParticipantKey {
        core: BindParticipantKeyCore,
        transition_id: [u8; 32],
        authorization_signature: [u8; 64],
        possession_proof: [u8; 64],
    },
    RevokeParticipantKey {
        core: RevokeParticipantKeyCore,
        transition_id: [u8; 32],
        authorization_signature: [u8; 64],
    },
    CreateProtectedObject {
        core: Box<CreateProtectedObjectCore>,
        transition_id: [u8; 32],
        authorization_signature: [u8; 64],
    },
    GrantAccess {
        core: Box<GrantAccessCore>,
        grant_id: [u8; 32],
        authorization_signature: [u8; 64],
    },
    RevokeGrant {
        core: Box<RevokeGrantCore>,
        transition_id: [u8; 32],
        authorization_signature: [u8; 64],
    },
}

impl PrivacyControlTransition {
    /// Derived transition identifier carried by every complete privacy transition.
    pub fn transition_id(&self) -> [u8; 32] {
        match &self.kind {
            PrivacyTransitionKind::RegisterPrincipal { transition_id, .. }
            | PrivacyTransitionKind::BindParticipantKey { transition_id, .. }
            | PrivacyTransitionKind::RevokeParticipantKey { transition_id, .. }
            | PrivacyTransitionKind::CreateProtectedObject { transition_id, .. }
            | PrivacyTransitionKind::RevokeGrant { transition_id, .. } => *transition_id,
            PrivacyTransitionKind::GrantAccess { grant_id, .. } => *grant_id,
        }
    }

    pub(crate) fn admission_anchor(&self) -> &PrivacyAdmissionAnchor {
        match &self.kind {
            PrivacyTransitionKind::RegisterPrincipal { core, .. } => &core.anchor,
            PrivacyTransitionKind::BindParticipantKey { core, .. } => &core.anchor,
            PrivacyTransitionKind::RevokeParticipantKey { core, .. } => &core.anchor,
            PrivacyTransitionKind::CreateProtectedObject { core, .. } => &core.anchor,
            PrivacyTransitionKind::GrantAccess { core, .. } => &core.anchor,
            PrivacyTransitionKind::RevokeGrant { core, .. } => &core.anchor,
        }
    }

    pub(crate) fn unsigned_core_bytes(&self) -> Vec<u8> {
        match &self.kind {
            PrivacyTransitionKind::RegisterPrincipal { core, .. } => core.canonical_bytes(),
            PrivacyTransitionKind::BindParticipantKey { core, .. } => core.canonical_bytes(),
            PrivacyTransitionKind::RevokeParticipantKey { core, .. } => core.canonical_bytes(),
            PrivacyTransitionKind::CreateProtectedObject { core, .. } => core.canonical_bytes(),
            PrivacyTransitionKind::GrantAccess { core, .. } => core.canonical_bytes(),
            PrivacyTransitionKind::RevokeGrant { core, .. } => core.canonical_bytes(),
        }
    }

    /// Encode the complete transition with CanonicalPrivacyEncodingV1.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        match &self.kind {
            PrivacyTransitionKind::RegisterPrincipal {
                core,
                transition_id,
                governance_signature,
                possession_signature,
            } => encode_record(
                REGISTER_COMPLETE_TAG,
                vec![
                    core.canonical_bytes(),
                    transition_id.to_vec(),
                    governance_signature.to_vec(),
                    possession_signature.to_vec(),
                ],
            ),
            PrivacyTransitionKind::BindParticipantKey {
                core,
                transition_id,
                authorization_signature,
                possession_proof,
            } => encode_record(
                BIND_COMPLETE_TAG,
                vec![
                    core.canonical_bytes(),
                    transition_id.to_vec(),
                    authorization_signature.to_vec(),
                    possession_proof.to_vec(),
                ],
            ),
            PrivacyTransitionKind::RevokeParticipantKey {
                core,
                transition_id,
                authorization_signature,
            } => encode_record(
                REVOKE_COMPLETE_TAG,
                vec![
                    core.canonical_bytes(),
                    transition_id.to_vec(),
                    authorization_signature.to_vec(),
                ],
            ),
            PrivacyTransitionKind::CreateProtectedObject {
                core,
                transition_id,
                authorization_signature,
            } => encode_record(
                CREATE_OBJECT_COMPLETE_TAG,
                vec![
                    core.canonical_bytes(),
                    transition_id.to_vec(),
                    authorization_signature.to_vec(),
                ],
            ),
            PrivacyTransitionKind::GrantAccess {
                core,
                grant_id,
                authorization_signature,
            } => encode_record(
                GRANT_ACCESS_COMPLETE_TAG,
                vec![
                    core.canonical_bytes(),
                    grant_id.to_vec(),
                    authorization_signature.to_vec(),
                ],
            ),
            PrivacyTransitionKind::RevokeGrant {
                core,
                transition_id,
                authorization_signature,
            } => encode_record(
                REVOKE_GRANT_COMPLETE_TAG,
                vec![
                    core.canonical_bytes(),
                    transition_id.to_vec(),
                    authorization_signature.to_vec(),
                ],
            ),
        }
    }

    /// Key reference introduced by a Register or Bind transition.
    pub fn introduced_key_reference(&self) -> Option<ParticipantKeyReference> {
        match &self.kind {
            PrivacyTransitionKind::RegisterPrincipal { core, .. } => {
                Some(core.initial_authorization_key.reference.clone())
            }
            PrivacyTransitionKind::BindParticipantKey { core, .. } => {
                Some(core.introduced_key.reference.clone())
            }
            PrivacyTransitionKind::RevokeParticipantKey { .. } => None,
            PrivacyTransitionKind::CreateProtectedObject { .. }
            | PrivacyTransitionKind::GrantAccess { .. }
            | PrivacyTransitionKind::RevokeGrant { .. } => None,
        }
    }

    pub(crate) fn introduced_key_public_bytes(&self) -> Option<&[u8]> {
        match &self.kind {
            PrivacyTransitionKind::RegisterPrincipal { core, .. } => {
                Some(core.initial_authorization_key.public_key.as_slice())
            }
            PrivacyTransitionKind::BindParticipantKey { core, .. } => {
                Some(core.introduced_key.public_key.as_slice())
            }
            _ => None,
        }
    }

    /// Object identifier introduced by a protected-object transition.
    pub fn protected_object_id(&self) -> Option<Uuid> {
        match &self.kind {
            PrivacyTransitionKind::CreateProtectedObject { core, .. } => Some(core.object_id),
            _ => None,
        }
    }

    /// Protected object addressed by a grant or its revocation.
    pub fn privacy_object_id(&self) -> Option<Uuid> {
        match &self.kind {
            PrivacyTransitionKind::CreateProtectedObject { core, .. } => Some(core.object_id),
            PrivacyTransitionKind::GrantAccess { core, .. } => Some(core.object_id),
            PrivacyTransitionKind::RevokeGrant { core, .. } => Some(core.object_id),
            _ => None,
        }
    }

    /// Grant identifier carried by GrantAccess or RevokeGrant.
    pub fn privacy_grant_id(&self) -> Option<[u8; 32]> {
        match &self.kind {
            PrivacyTransitionKind::GrantAccess { grant_id, .. } => Some(*grant_id),
            PrivacyTransitionKind::RevokeGrant { core, .. } => Some(core.grant_id),
            _ => None,
        }
    }

    /// Strictly decode one complete lifecycle transition in the closed union.
    pub fn decode(bytes: &[u8]) -> Result<Self, PrivacyError> {
        if bytes.len() > MAX_CREATE_PROTECTED_OBJECT_BYTES {
            return Err(PrivacyError::Malformed(format!(
                "privacy transition exceeds {MAX_CREATE_PROTECTED_OBJECT_BYTES} bytes"
            )));
        }
        if bytes.len() < 6 || &bytes[..4] != RECORD_MAGIC {
            return Err(PrivacyError::Malformed(
                "invalid canonical privacy record magic".to_string(),
            ));
        }
        if bytes[4] != CREATE_OBJECT_COMPLETE_TAG && bytes.len() > MAX_RECORD_BYTES {
            return Err(PrivacyError::Malformed(format!(
                "non-create privacy transition exceeds {MAX_RECORD_BYTES} bytes"
            )));
        }
        match bytes[4] {
            REGISTER_COMPLETE_TAG => decode_registration(bytes),
            BIND_COMPLETE_TAG => decode_binding(bytes),
            REVOKE_COMPLETE_TAG => decode_revocation(bytes),
            CREATE_OBJECT_COMPLETE_TAG => decode_object_creation(bytes),
            GRANT_ACCESS_COMPLETE_TAG => decode_grant_access(bytes),
            REVOKE_GRANT_COMPLETE_TAG => decode_revoke_grant(bytes),
            tag => Err(PrivacyError::UnknownTransitionRecord(tag)),
        }
    }
}

fn decode_registration(bytes: &[u8]) -> Result<PrivacyControlTransition, PrivacyError> {
    let complete = decode_record(bytes, REGISTER_COMPLETE_TAG, 4)?;
    let core = decode_register_core(complete[0])?;
    let transition_id = fixed::<32>(complete[1], "registration transition identifier")?;
    let governance_signature = fixed::<64>(complete[2], "governance signature")?;
    let possession_signature = fixed::<64>(complete[3], "possession signature")?;
    let core_bytes = core.canonical_bytes();
    if core_bytes != complete[0] {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    let expected_transition_id = domain_hash(REGISTER_TRANSITION_ID_DOMAIN, &core_bytes);
    if transition_id != expected_transition_id {
        return Err(PrivacyError::Malformed(
            "registration transition identifier mismatch".to_string(),
        ));
    }
    let transition = PrivacyControlTransition {
        kind: PrivacyTransitionKind::RegisterPrincipal {
            core,
            transition_id,
            governance_signature,
            possession_signature,
        },
    };
    if transition.canonical_bytes() != bytes {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    Ok(transition)
}

fn decode_binding(bytes: &[u8]) -> Result<PrivacyControlTransition, PrivacyError> {
    let complete = decode_record(bytes, BIND_COMPLETE_TAG, 4)?;
    let core = decode_bind_core(complete[0])?;
    let transition_id = fixed::<32>(complete[1], "binding transition identifier")?;
    let authorization_signature = fixed::<64>(complete[2], "binding authorization signature")?;
    let possession_proof = fixed::<64>(complete[3], "binding possession proof")?;
    let core_bytes = core.canonical_bytes();
    if core_bytes != complete[0] {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    if transition_id != domain_hash(BIND_TRANSITION_ID_DOMAIN, &core_bytes) {
        return Err(PrivacyError::Malformed(
            "binding transition identifier mismatch".to_string(),
        ));
    }
    let transition = PrivacyControlTransition {
        kind: PrivacyTransitionKind::BindParticipantKey {
            core,
            transition_id,
            authorization_signature,
            possession_proof,
        },
    };
    if transition.canonical_bytes() != bytes {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    Ok(transition)
}

fn decode_revocation(bytes: &[u8]) -> Result<PrivacyControlTransition, PrivacyError> {
    let complete = decode_record(bytes, REVOKE_COMPLETE_TAG, 3)?;
    let core = decode_revoke_core(complete[0])?;
    let transition_id = fixed::<32>(complete[1], "revocation transition identifier")?;
    let authorization_signature = fixed::<64>(complete[2], "revocation authorization signature")?;
    let core_bytes = core.canonical_bytes();
    if core_bytes != complete[0] {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    if transition_id != domain_hash(REVOKE_TRANSITION_ID_DOMAIN, &core_bytes) {
        return Err(PrivacyError::Malformed(
            "revocation transition identifier mismatch".to_string(),
        ));
    }
    let transition = PrivacyControlTransition {
        kind: PrivacyTransitionKind::RevokeParticipantKey {
            core,
            transition_id,
            authorization_signature,
        },
    };
    if transition.canonical_bytes() != bytes {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    Ok(transition)
}

fn decode_object_creation(bytes: &[u8]) -> Result<PrivacyControlTransition, PrivacyError> {
    let complete = decode_record(bytes, CREATE_OBJECT_COMPLETE_TAG, 3)?;
    let core = decode_create_object_core(complete[0])?;
    let transition_id = fixed::<32>(complete[1], "object-creation transition identifier")?;
    let authorization_signature =
        fixed::<64>(complete[2], "object-creation authorization signature")?;
    let core_bytes = core.canonical_bytes();
    if core_bytes != complete[0] {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    if transition_id != domain_hash(CREATE_OBJECT_TRANSITION_ID_DOMAIN, &core_bytes) {
        return Err(PrivacyError::Malformed(
            "object-creation transition identifier mismatch".to_string(),
        ));
    }
    let transition = PrivacyControlTransition {
        kind: PrivacyTransitionKind::CreateProtectedObject {
            core: Box::new(core),
            transition_id,
            authorization_signature,
        },
    };
    if transition.canonical_bytes() != bytes {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    Ok(transition)
}

fn decode_grant_access(bytes: &[u8]) -> Result<PrivacyControlTransition, PrivacyError> {
    let complete = decode_record(bytes, GRANT_ACCESS_COMPLETE_TAG, 3)?;
    let core = decode_grant_access_core(complete[0])?;
    let grant_id = fixed::<32>(complete[1], "grant identifier")?;
    let authorization_signature = fixed::<64>(complete[2], "grant authorization signature")?;
    let core_bytes = core.canonical_bytes();
    if core_bytes != complete[0] {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    if grant_id != domain_hash(GRANT_ID_DOMAIN, &core_bytes) {
        return Err(PrivacyError::Malformed(
            "grant identifier mismatch".to_string(),
        ));
    }
    let transition = PrivacyControlTransition {
        kind: PrivacyTransitionKind::GrantAccess {
            core: Box::new(core),
            grant_id,
            authorization_signature,
        },
    };
    if transition.canonical_bytes() != bytes {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    Ok(transition)
}

fn decode_revoke_grant(bytes: &[u8]) -> Result<PrivacyControlTransition, PrivacyError> {
    let complete = decode_record(bytes, REVOKE_GRANT_COMPLETE_TAG, 3)?;
    let core = decode_revoke_grant_core(complete[0])?;
    let transition_id = fixed::<32>(complete[1], "grant revocation transition identifier")?;
    let authorization_signature =
        fixed::<64>(complete[2], "grant revocation authorization signature")?;
    let core_bytes = core.canonical_bytes();
    if core_bytes != complete[0] {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    if transition_id != domain_hash(REVOKE_GRANT_TRANSITION_ID_DOMAIN, &core_bytes) {
        return Err(PrivacyError::Malformed(
            "grant revocation transition identifier mismatch".to_string(),
        ));
    }
    let transition = PrivacyControlTransition {
        kind: PrivacyTransitionKind::RevokeGrant {
            core: Box::new(core),
            transition_id,
            authorization_signature,
        },
    };
    if transition.canonical_bytes() != bytes {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    Ok(transition)
}

fn decode_register_core(bytes: &[u8]) -> Result<RegisterPrincipalCore, PrivacyError> {
    let fields = decode_record(bytes, REGISTER_CORE_TAG, 14)?;
    validate_common_identifiers(
        fields[0],
        fields[1],
        fields[2],
        TransitionVariant::RegisterPrincipal,
    )?;
    let anchor = decode_anchor(&fields[3..10])?;
    let principal = uuid(fields[10], "new participant principal")?;
    let initial_authorization_key = decode_participant_key_binding(fields[11])?;
    if initial_authorization_key.reference.principal != principal
        || initial_authorization_key.reference.purpose
            != ParticipantKeyPurpose::PrivacyAuthorization
        || initial_authorization_key.reference.version != 1
    {
        return Err(PrivacyError::Malformed(
            "registration must bind authorization key version 1 to the new principal".to_string(),
        ));
    }
    one_byte(fields[12], 0x01, "bootstrap authorizer")?;
    let bootstrap_governance_key = decode_bootstrap_reference(fields[13])?;
    Ok(RegisterPrincipalCore {
        anchor,
        principal,
        initial_authorization_key,
        bootstrap_governance_key,
    })
}

fn decode_bind_core(bytes: &[u8]) -> Result<BindParticipantKeyCore, PrivacyError> {
    let fields = decode_record(bytes, BIND_CORE_TAG, 14)?;
    validate_common_identifiers(
        fields[0],
        fields[1],
        fields[2],
        TransitionVariant::BindParticipantKey,
    )?;
    let anchor = decode_anchor(&fields[3..10])?;
    let affected_principal = uuid(fields[10], "affected participant principal")?;
    let introduced_key = decode_participant_key_binding(fields[11])?;
    if introduced_key.reference.principal != affected_principal {
        return Err(PrivacyError::Malformed(
            "introduced key principal does not match the affected principal".to_string(),
        ));
    }
    one_byte(fields[12], 0x02, "participant authorizer")?;
    let authorizer_key = decode_participant_key_reference(fields[13])?;
    if authorizer_key.principal != affected_principal
        || authorizer_key.purpose != ParticipantKeyPurpose::PrivacyAuthorization
    {
        return Err(PrivacyError::Malformed(
            "binding authorizer must be the affected principal's authorization key".to_string(),
        ));
    }
    Ok(BindParticipantKeyCore {
        anchor,
        affected_principal,
        introduced_key,
        authorizer_key,
    })
}

fn decode_revoke_core(bytes: &[u8]) -> Result<RevokeParticipantKeyCore, PrivacyError> {
    let fields = decode_record(bytes, REVOKE_CORE_TAG, 16)?;
    validate_common_identifiers(
        fields[0],
        fields[1],
        fields[2],
        TransitionVariant::RevokeParticipantKey,
    )?;
    let anchor = decode_anchor(&fields[3..10])?;
    let affected_principal = uuid(fields[10], "affected participant principal")?;
    let target_principal = uuid(fields[11], "target participant principal")?;
    let target_purpose = decode_key_purpose(fields[12])?;
    let target_version = u32_field(fields[13], "target key version")?;
    one_byte(fields[14], 0x02, "participant authorizer")?;
    let authorizer_key = decode_participant_key_reference(fields[15])?;
    if affected_principal != target_principal
        || authorizer_key.principal != affected_principal
        || authorizer_key.purpose != ParticipantKeyPurpose::PrivacyAuthorization
    {
        return Err(PrivacyError::Malformed(
            "revocation target and authorizer must belong to the affected principal".to_string(),
        ));
    }
    Ok(RevokeParticipantKeyCore {
        anchor,
        affected_principal,
        target_key: ParticipantKeyLocator {
            principal: target_principal,
            purpose: target_purpose,
            version: target_version,
        },
        authorizer_key,
    })
}

fn decode_create_object_core(bytes: &[u8]) -> Result<CreateProtectedObjectCore, PrivacyError> {
    let fields = decode_record(bytes, CREATE_OBJECT_CORE_TAG, 19)?;
    validate_common_identifiers(
        fields[0],
        fields[1],
        fields[2],
        TransitionVariant::CreateProtectedObject,
    )?;
    let anchor = decode_anchor(&fields[3..10])?;
    let object_id = object_uuid(fields[10], "protected object identifier")?;
    let owner = uuid(fields[11], "protected object owner")?;
    let authorization_key = decode_participant_key_reference(fields[12])?;
    let wrapping_key = decode_participant_key_reference(fields[13])?;
    if authorization_key.principal != owner
        || authorization_key.purpose != ParticipantKeyPurpose::PrivacyAuthorization
        || wrapping_key.principal != owner
        || wrapping_key.purpose != ParticipantKeyPurpose::PrivacyKeyWrapping
    {
        return Err(PrivacyError::Malformed(
            "object keys must be the owner's purpose-separated key references".to_string(),
        ));
    }
    let object_context = fixed::<32>(fields[14], "object encryption context")?;
    let payload = ProtectedPayloadCiphertext::decode(fields[15])?;
    let encrypted_payload_commitment = fixed::<32>(fields[16], "encrypted payload commitment")?;
    let owner_envelope = OwnerDekEnvelope::decode(fields[17])?;
    one_byte(fields[18], 0x02, "participant authorizer")?;
    validate_protected_object_material(
        &anchor,
        object_id,
        owner,
        &wrapping_key,
        object_context,
        &payload,
        encrypted_payload_commitment,
        &owner_envelope,
    )?;
    let core = CreateProtectedObjectCore {
        anchor,
        object_id,
        owner,
        authorization_key,
        wrapping_key,
        object_context,
        payload,
        encrypted_payload_commitment,
        owner_envelope,
    };
    if core.canonical_bytes() != bytes {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    Ok(core)
}

fn decode_grant_access_core(bytes: &[u8]) -> Result<GrantAccessCore, PrivacyError> {
    let fields = decode_record(bytes, GRANT_ACCESS_CORE_TAG, 17)?;
    validate_common_identifiers(
        fields[0],
        fields[1],
        fields[2],
        TransitionVariant::GrantAccess,
    )?;
    let anchor = decode_anchor(&fields[3..10])?;
    let object_id = object_uuid(fields[10], "grant object identifier")?;
    let owner = uuid(fields[11], "grant owner principal")?;
    let grantee = uuid(fields[12], "grant grantee principal")?;
    let permission = PrivacyPermission::decode(fields[13])?;
    let delivery = GrantDekEnvelope::decode(fields[14])?;
    one_byte(fields[15], 0x02, "participant authorizer")?;
    let authorization_key = decode_participant_key_reference(fields[16])?;
    if owner == grantee
        || authorization_key.principal != owner
        || authorization_key.purpose != ParticipantKeyPurpose::PrivacyAuthorization
    {
        return Err(PrivacyError::Malformed(
            "grant owner, grantee, and authorizer do not satisfy the grant contract".to_string(),
        ));
    }
    validate_grant_delivery_material(
        &anchor, object_id, owner, grantee, permission, &delivery, None, None,
    )?;
    let core = GrantAccessCore {
        anchor,
        object_id,
        owner,
        grantee,
        permission,
        delivery,
        authorization_key,
    };
    if core.canonical_bytes() != bytes {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    Ok(core)
}

fn decode_revoke_grant_core(bytes: &[u8]) -> Result<RevokeGrantCore, PrivacyError> {
    let fields = decode_record(bytes, REVOKE_GRANT_CORE_TAG, 18)?;
    validate_common_identifiers(
        fields[0],
        fields[1],
        fields[2],
        TransitionVariant::RevokeGrant,
    )?;
    let anchor = decode_anchor(&fields[3..10])?;
    let grant_id = fixed::<32>(fields[10], "grant identifier")?;
    if grant_id == [0; 32] {
        return Err(PrivacyError::Malformed(
            "grant identifier cannot be empty".to_string(),
        ));
    }
    let object_id = object_uuid(fields[11], "revoked grant object identifier")?;
    let grantee = uuid(fields[12], "revoked grant grantee principal")?;
    let owner = uuid(fields[13], "revoked grant owner principal")?;
    let permission = PrivacyPermission::decode(fields[14])?;
    let creation_ledger_position = u64_field(fields[15], "grant creation ledger position")?;
    one_byte(fields[16], 0x02, "participant authorizer")?;
    let authorization_key = decode_participant_key_reference(fields[17])?;
    if owner == grantee
        || authorization_key.principal != owner
        || authorization_key.purpose != ParticipantKeyPurpose::PrivacyAuthorization
    {
        return Err(PrivacyError::Malformed(
            "grant revocation owner, grantee, and authorizer do not satisfy the contract"
                .to_string(),
        ));
    }
    let core = RevokeGrantCore {
        anchor,
        grant_id,
        object_id,
        grantee,
        owner,
        permission,
        creation_ledger_position,
        authorization_key,
    };
    if core.canonical_bytes() != bytes {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    Ok(core)
}

fn decode_anchor(fields: &[&[u8]]) -> Result<PrivacyAdmissionAnchor, PrivacyError> {
    if fields.len() != 7 {
        return Err(PrivacyError::Malformed(
            "privacy anchor field count mismatch".to_string(),
        ));
    }
    let network_id = ascii_identifier(fields[0], "network_id")?;
    let profile_id = ascii_identifier(fields[1], "profile_id")?;
    let profile_content_hash = fixed::<32>(fields[2], "profile content hash")?;
    let expected_ledger_position = u64_field(fields[3], "expected ledger position")?;
    let expected_privacy_revision = u64_field(fields[4], "expected privacy revision")?;
    let parent_ledger_prefix_hash = fixed::<32>(fields[5], "parent ledger prefix hash")?;
    let parent_envelope_hash = decode_parent_envelope_reference(fields[6])?;
    PrivacyAdmissionAnchor::new(
        network_id,
        profile_id,
        profile_content_hash,
        expected_ledger_position,
        expected_privacy_revision,
        parent_ledger_prefix_hash,
        parent_envelope_hash,
    )
}

fn decode_participant_key_binding(bytes: &[u8]) -> Result<ParticipantKeyBinding, PrivacyError> {
    let fields = decode_record(bytes, PARTICIPANT_KEY_BINDING_TAG, 8)?;
    let principal = uuid(fields[0], "participant key principal")?;
    let purpose = decode_key_purpose(fields[1])?;
    let version = u32_field(fields[2], "participant key version")?;
    let supplied_fingerprint = fixed::<32>(fields[5], "participant key fingerprint")?;
    let binding = match purpose {
        ParticipantKeyPurpose::PrivacyAuthorization => {
            one_byte(fields[3], KeyAlgorithm::Ed25519 as u8, "key algorithm")?;
            one_byte(
                fields[4],
                PublicKeyEncoding::Ed25519Raw32 as u8,
                "key encoding",
            )?;
            one_byte(
                fields[6],
                PossessionProofScheme::Ed25519Raw64 as u8,
                "possession proof scheme",
            )?;
            ParticipantKeyBinding::authorization(
                principal,
                version,
                fixed::<32>(fields[7], "participant Ed25519 public key")?,
            )?
        }
        ParticipantKeyPurpose::PrivacyKeyWrapping => {
            one_byte(fields[3], KeyAlgorithm::P256 as u8, "key algorithm")?;
            one_byte(
                fields[4],
                PublicKeyEncoding::Sec1UncompressedP256 as u8,
                "key encoding",
            )?;
            one_byte(
                fields[6],
                PossessionProofScheme::EcdsaP256Sha256RawLowS64 as u8,
                "possession proof scheme",
            )?;
            ParticipantKeyBinding::wrapping(
                principal,
                version,
                fixed::<65>(fields[7], "participant P-256 public key")?,
            )?
        }
    };
    if binding.reference.fingerprint != supplied_fingerprint {
        return Err(PrivacyError::Malformed(
            "participant key fingerprint mismatch".to_string(),
        ));
    }
    if binding.canonical_bytes() != bytes {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    Ok(binding)
}

fn decode_participant_key_reference(bytes: &[u8]) -> Result<ParticipantKeyReference, PrivacyError> {
    let fields = decode_record(bytes, PARTICIPANT_KEY_REFERENCE_TAG, 6)?;
    let principal = uuid(fields[0], "participant key principal")?;
    let purpose = decode_key_purpose(fields[1])?;
    let version = u32_field(fields[2], "participant key version")?;
    let algorithm = match fixed::<1>(fields[3], "key algorithm")?[0] {
        0x01 => KeyAlgorithm::Ed25519,
        0x02 => KeyAlgorithm::P256,
        value => {
            return Err(PrivacyError::Malformed(format!(
                "unsupported key algorithm value {value}"
            )));
        }
    };
    let encoding = match fixed::<1>(fields[4], "key encoding")?[0] {
        0x01 => PublicKeyEncoding::Ed25519Raw32,
        0x02 => PublicKeyEncoding::Sec1UncompressedP256,
        value => {
            return Err(PrivacyError::Malformed(format!(
                "unsupported key encoding value {value}"
            )));
        }
    };
    match purpose {
        ParticipantKeyPurpose::PrivacyAuthorization
            if algorithm == KeyAlgorithm::Ed25519
                && encoding == PublicKeyEncoding::Ed25519Raw32 => {}
        ParticipantKeyPurpose::PrivacyKeyWrapping
            if algorithm == KeyAlgorithm::P256
                && encoding == PublicKeyEncoding::Sec1UncompressedP256 => {}
        _ => {
            return Err(PrivacyError::Malformed(
                "key purpose, algorithm, and encoding are not interchangeable".to_string(),
            ));
        }
    }
    let reference = ParticipantKeyReference {
        principal,
        purpose,
        version,
        algorithm,
        encoding,
        fingerprint: fixed::<32>(fields[5], "participant key fingerprint")?,
    };
    if reference.canonical_bytes() != bytes {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    Ok(reference)
}

fn decode_key_purpose(bytes: &[u8]) -> Result<ParticipantKeyPurpose, PrivacyError> {
    match fixed::<1>(bytes, "participant key purpose")?[0] {
        0x01 => Ok(ParticipantKeyPurpose::PrivacyAuthorization),
        0x02 => Ok(ParticipantKeyPurpose::PrivacyKeyWrapping),
        value => Err(PrivacyError::Malformed(format!(
            "unsupported participant key purpose value {value}"
        ))),
    }
}

fn decode_bootstrap_reference(
    bytes: &[u8],
) -> Result<BootstrapGovernanceKeyReference, PrivacyError> {
    let fields = decode_record(bytes, BOOTSTRAP_GOVERNANCE_REFERENCE_TAG, 5)?;
    one_byte(fields[0], 0x01, "bootstrap profile role")?;
    let version = u32_field(fields[1], "bootstrap key version")?;
    if version != 1 {
        return Err(PrivacyError::Malformed(
            "bootstrap governance key reference must use version 1".to_string(),
        ));
    }
    one_byte(
        fields[2],
        KeyAlgorithm::Ed25519 as u8,
        "bootstrap key algorithm",
    )?;
    one_byte(
        fields[3],
        PublicKeyEncoding::Ed25519Raw32 as u8,
        "bootstrap key encoding",
    )?;
    let reference = BootstrapGovernanceKeyReference {
        version,
        fingerprint: fixed::<32>(fields[4], "bootstrap key fingerprint")?,
    };
    if reference.canonical_bytes() != bytes {
        return Err(PrivacyError::NonCanonicalRecord);
    }
    Ok(reference)
}

fn validate_common_identifiers(
    protocol: &[u8],
    schema: &[u8],
    variant: &[u8],
    expected_variant: TransitionVariant,
) -> Result<(), PrivacyError> {
    if protocol != PRIVACY_CONTROL_V1.as_bytes() {
        return Err(PrivacyError::Malformed(
            "wrong privacy-control protocol identifier".to_string(),
        ));
    }
    if schema != CANONICAL_PRIVACY_ENCODING_V1.as_bytes() {
        return Err(PrivacyError::Malformed(
            "wrong canonical privacy encoding identifier".to_string(),
        ));
    }
    one_byte(variant, expected_variant as u8, "transition variant")
}

fn key_fingerprint(
    scheme_identifier: &str,
    algorithm: KeyAlgorithm,
    encoding: PublicKeyEncoding,
    public_key: &[u8],
) -> [u8; 32] {
    let input = encode_record(
        KEY_FINGERPRINT_INPUT_TAG,
        vec![
            scheme_identifier.as_bytes().to_vec(),
            vec![algorithm as u8],
            vec![encoding as u8],
            public_key.to_vec(),
        ],
    );
    domain_hash(KEY_FINGERPRINT_DOMAIN, &input)
}

fn object_encryption_context_bytes(
    anchor: &PrivacyAdmissionAnchor,
    object_id: Uuid,
    owner: Uuid,
    wrapping_key: &ParticipantKeyReference,
    protected_content_commitment: [u8; 32],
) -> Vec<u8> {
    let mut fields = anchor.common_fields(TransitionVariant::CreateProtectedObject);
    fields.extend([
        object_id.as_bytes().to_vec(),
        owner.as_bytes().to_vec(),
        wrapping_key.canonical_bytes(),
        vec![PROTECTED_PAYLOAD_CIPHERTEXT_TAG],
        PROTECTED_DATA_SUITE_V1.as_bytes().to_vec(),
        vec![OWNER_DEK_ENVELOPE_TAG],
        PROTECTED_DATA_SUITE_V1.as_bytes().to_vec(),
        protected_content_commitment.to_vec(),
    ]);
    encode_record(OBJECT_ENCRYPTION_CONTEXT_TAG, fields)
}

pub(crate) fn protected_object_context(
    anchor: &PrivacyAdmissionAnchor,
    object_id: Uuid,
    owner: Uuid,
    wrapping_key: &ParticipantKeyReference,
    protected_content_commitment: [u8; 32],
) -> [u8; 32] {
    domain_hash(
        OBJECT_CONTEXT_DOMAIN,
        &object_encryption_context_bytes(
            anchor,
            object_id,
            owner,
            wrapping_key,
            protected_content_commitment,
        ),
    )
}

pub(crate) fn protected_payload(
    protected_content_commitment: [u8; 32],
    ciphertext_and_tag: Vec<u8>,
) -> Result<ProtectedPayloadCiphertext, PrivacyError> {
    ProtectedPayloadCiphertext::new(protected_content_commitment, ciphertext_and_tag)
}

pub(crate) fn encrypted_payload_commitment(payload: &ProtectedPayloadCiphertext) -> [u8; 32] {
    domain_hash(
        ENCRYPTED_PAYLOAD_COMMITMENT_DOMAIN,
        &payload.canonical_bytes,
    )
}

pub(crate) fn owner_envelope_header_and_info(
    anchor: &PrivacyAdmissionAnchor,
    object_id: Uuid,
    owner: Uuid,
    wrapping_key: ParticipantKeyReference,
    protected_content_commitment: [u8; 32],
    encrypted_payload_commitment: [u8; 32],
    object_context: [u8; 32],
) -> (OwnerDekHeader, [u8; 64]) {
    let header = OwnerDekHeader::from_parts(
        anchor,
        object_id,
        owner,
        wrapping_key,
        protected_content_commitment,
        encrypted_payload_commitment,
    );
    let header_hash = domain_hash(
        "provchain/protected-data/hpke-owner-header/v1",
        &header.canonical_bytes,
    );
    let mut info = [0_u8; 64];
    info[..32].copy_from_slice(&object_context);
    info[32..].copy_from_slice(&header_hash);
    (header, info)
}

pub(crate) fn owner_envelope(
    object_context: [u8; 32],
    header: OwnerDekHeader,
    encapsulated_key: [u8; 65],
    wrapped_dek_ciphertext: [u8; 48],
) -> Result<OwnerDekEnvelope, PrivacyError> {
    OwnerDekEnvelope::new(
        object_context,
        header,
        encapsulated_key,
        wrapped_dek_ciphertext,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn grant_envelope_header_and_context(
    anchor: &PrivacyAdmissionAnchor,
    object_id: Uuid,
    owner: Uuid,
    grantee: Uuid,
    permission: PrivacyPermission,
    wrapping_key: ParticipantKeyReference,
    protected_content_commitment: [u8; 32],
    encrypted_payload_commitment: [u8; 32],
) -> (GrantDekHeader, [u8; 32], [u8; 64]) {
    let header = GrantDekHeader::from_parts(
        anchor,
        object_id,
        owner,
        grantee,
        permission,
        wrapping_key,
        protected_content_commitment,
        encrypted_payload_commitment,
    );
    let context = domain_hash(
        GRANT_DELIVERY_CONTEXT_DOMAIN,
        &grant_delivery_context_bytes(
            anchor,
            object_id,
            owner,
            grantee,
            permission,
            &header.wrapping_key,
            protected_content_commitment,
            encrypted_payload_commitment,
        ),
    );
    let info = grant_hpke_info(context, &header);
    (header, context, info)
}

pub(crate) fn grant_envelope(
    context: [u8; 32],
    header: GrantDekHeader,
    encapsulated_key: [u8; 65],
    wrapped_dek_ciphertext: [u8; 48],
) -> Result<GrantDekEnvelope, PrivacyError> {
    GrantDekEnvelope::new(context, header, encapsulated_key, wrapped_dek_ciphertext)
}

#[allow(clippy::too_many_arguments)]
fn grant_delivery_context_bytes(
    anchor: &PrivacyAdmissionAnchor,
    object_id: Uuid,
    owner: Uuid,
    grantee: Uuid,
    permission: PrivacyPermission,
    wrapping_key: &ParticipantKeyReference,
    protected_content_commitment: [u8; 32],
    encrypted_payload_commitment: [u8; 32],
) -> Vec<u8> {
    let mut fields = anchor.common_fields(TransitionVariant::GrantAccess);
    fields.extend([
        object_id.as_bytes().to_vec(),
        owner.as_bytes().to_vec(),
        grantee.as_bytes().to_vec(),
        vec![permission as u8],
        PROTECTED_DATA_SUITE_V1.as_bytes().to_vec(),
        protected_content_commitment.to_vec(),
        encrypted_payload_commitment.to_vec(),
        wrapping_key.canonical_bytes(),
        vec![GRANT_DEK_ENVELOPE_TAG],
        PROTECTED_DATA_SUITE_V1.as_bytes().to_vec(),
    ]);
    encode_record(GRANT_DELIVERY_CONTEXT_TAG, fields)
}

fn grant_delivery_context_digest(header: &GrantDekHeader) -> Result<[u8; 32], PrivacyError> {
    let anchor = header.anchor()?;
    Ok(domain_hash(
        GRANT_DELIVERY_CONTEXT_DOMAIN,
        &grant_delivery_context_bytes(
            &anchor,
            header.object_id,
            header.owner,
            header.grantee,
            header.permission,
            &header.wrapping_key,
            header.protected_content_commitment,
            header.encrypted_payload_commitment,
        ),
    ))
}

fn grant_hpke_info(grant_delivery_context: [u8; 32], header: &GrantDekHeader) -> [u8; 64] {
    let header_hash = domain_hash(GRANT_HEADER_HASH_DOMAIN, &header.canonical_bytes);
    let mut info = [0_u8; 64];
    info[..32].copy_from_slice(&grant_delivery_context);
    info[32..].copy_from_slice(&header_hash);
    info
}

#[allow(clippy::too_many_arguments)]
fn validate_grant_delivery_material(
    anchor: &PrivacyAdmissionAnchor,
    object_id: Uuid,
    owner: Uuid,
    grantee: Uuid,
    permission: PrivacyPermission,
    delivery: &GrantDekEnvelope,
    protected_content_commitment: Option<[u8; 32]>,
    encrypted_payload_commitment: Option<[u8; 32]>,
) -> Result<(), PrivacyError> {
    let header = delivery.header();
    if header.anchor()? != *anchor
        || header.object_id != object_id
        || header.owner != owner
        || header.grantee != grantee
        || header.permission != permission
        || header.wrapping_key.purpose != ParticipantKeyPurpose::PrivacyKeyWrapping
        || header.wrapping_key.principal != grantee
    {
        return Err(PrivacyError::Malformed(
            "grant delivery header does not match the grant core".to_string(),
        ));
    }
    if let Some(expected) = protected_content_commitment {
        if header.protected_content_commitment != expected {
            return Err(PrivacyError::Malformed(
                "grant delivery protected-content commitment mismatch".to_string(),
            ));
        }
    }
    if let Some(expected) = encrypted_payload_commitment {
        if header.encrypted_payload_commitment != expected {
            return Err(PrivacyError::Malformed(
                "grant delivery encrypted-payload commitment mismatch".to_string(),
            ));
        }
    }
    if delivery.grant_delivery_context != grant_delivery_context_digest(header)? {
        return Err(PrivacyError::Malformed(
            "grant delivery context digest mismatch".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn protected_domain_hash(domain: &str, payload: &[u8]) -> [u8; 32] {
    domain_hash(domain, payload)
}

pub(crate) fn protected_encode_record(tag: u8, fields: Vec<Vec<u8>>) -> Vec<u8> {
    encode_record(tag, fields)
}

pub(crate) fn protected_encode_record_refs(tag: u8, fields: &[&[u8]]) -> Vec<u8> {
    encode_record_refs(tag, fields)
}

#[allow(clippy::too_many_arguments)]
fn validate_protected_object_material(
    anchor: &PrivacyAdmissionAnchor,
    object_id: Uuid,
    owner: Uuid,
    wrapping_key: &ParticipantKeyReference,
    object_context: [u8; 32],
    payload: &ProtectedPayloadCiphertext,
    encrypted_payload_commitment: [u8; 32],
    owner_envelope: &OwnerDekEnvelope,
) -> Result<(), PrivacyError> {
    let expected_context = domain_hash(
        OBJECT_CONTEXT_DOMAIN,
        &object_encryption_context_bytes(
            anchor,
            object_id,
            owner,
            wrapping_key,
            payload.protected_content_commitment,
        ),
    );
    if object_context != expected_context || owner_envelope.object_context != expected_context {
        return Err(PrivacyError::Malformed(
            "object encryption context mismatch".to_string(),
        ));
    }
    let expected_epc = domain_hash(
        ENCRYPTED_PAYLOAD_COMMITMENT_DOMAIN,
        &payload.canonical_bytes,
    );
    if encrypted_payload_commitment != expected_epc {
        return Err(PrivacyError::Malformed(
            "encrypted payload commitment mismatch".to_string(),
        ));
    }
    let expected_header = OwnerDekHeader::from_parts(
        anchor,
        object_id,
        owner,
        wrapping_key.clone(),
        payload.protected_content_commitment,
        encrypted_payload_commitment,
    );
    if owner_envelope.header != expected_header {
        return Err(PrivacyError::Malformed(
            "owner DEK envelope header does not match the object core".to_string(),
        ));
    }
    Ok(())
}

fn domain_hash(domain: &str, payload: &[u8]) -> [u8; 32] {
    let input = encode_record(
        DOMAIN_HASH_INPUT_TAG,
        vec![domain.as_bytes().to_vec(), payload.to_vec()],
    );
    Sha256::digest(input).into()
}

fn concat_digest_and_record(digest: [u8; 32], record: &[u8]) -> Vec<u8> {
    let mut payload = Vec::with_capacity(32 + record.len());
    payload.extend_from_slice(&digest);
    payload.extend_from_slice(record);
    payload
}

fn encode_record(tag: u8, fields: Vec<Vec<u8>>) -> Vec<u8> {
    let fields = fields.iter().map(Vec::as_slice).collect::<Vec<_>>();
    encode_record_refs(tag, &fields)
}

fn encode_record_refs(tag: u8, fields: &[&[u8]]) -> Vec<u8> {
    let field_count = u8::try_from(fields.len()).expect("closed record field count fits u8");
    let capacity = fields
        .iter()
        .map(|field| 5_usize.saturating_add(field.len()))
        .fold(6_usize, usize::saturating_add);
    let mut bytes = Vec::with_capacity(capacity);
    bytes.extend_from_slice(RECORD_MAGIC);
    bytes.push(tag);
    bytes.push(field_count);
    for (index, field) in fields.iter().enumerate() {
        bytes.push(u8::try_from(index + 1).expect("closed field tag fits u8"));
        bytes.extend_from_slice(
            &u32::try_from(field.len())
                .expect("bounded canonical field length fits u32")
                .to_be_bytes(),
        );
        bytes.extend_from_slice(field);
    }
    bytes
}

fn decode_record(
    bytes: &[u8],
    expected_tag: u8,
    expected_fields: u8,
) -> Result<Vec<&[u8]>, PrivacyError> {
    if bytes.len() < 6 || &bytes[..4] != RECORD_MAGIC {
        return Err(PrivacyError::Malformed(
            "invalid canonical privacy record magic".to_string(),
        ));
    }
    if bytes[4] != expected_tag {
        return Err(PrivacyError::Malformed(format!(
            "privacy record tag {}, expected {expected_tag}",
            bytes[4]
        )));
    }
    if bytes[5] != expected_fields {
        return Err(PrivacyError::Malformed(format!(
            "privacy record field count {}, expected {expected_fields}",
            bytes[5]
        )));
    }
    let mut cursor = 6_usize;
    let mut fields = Vec::with_capacity(expected_fields as usize);
    for expected_field_tag in 1..=expected_fields {
        let header_end = cursor.checked_add(5).ok_or_else(|| {
            PrivacyError::Malformed("privacy record field header overflow".to_string())
        })?;
        if header_end > bytes.len() {
            return Err(PrivacyError::Malformed(
                "truncated privacy record field header".to_string(),
            ));
        }
        if bytes[cursor] != expected_field_tag {
            return Err(PrivacyError::Malformed(format!(
                "privacy field tag {}, expected {expected_field_tag}",
                bytes[cursor]
            )));
        }
        let length = u32::from_be_bytes(
            bytes[cursor + 1..header_end]
                .try_into()
                .map_err(|_| PrivacyError::Malformed("invalid field length".to_string()))?,
        ) as usize;
        cursor = header_end;
        let field_end = cursor.checked_add(length).ok_or_else(|| {
            PrivacyError::Malformed("privacy record field length overflow".to_string())
        })?;
        if field_end > bytes.len() {
            return Err(PrivacyError::Malformed(
                "truncated privacy record field".to_string(),
            ));
        }
        if length == 0 {
            return Err(PrivacyError::Malformed(
                "empty privacy record field is forbidden".to_string(),
            ));
        }
        fields.push(&bytes[cursor..field_end]);
        cursor = field_end;
    }
    if cursor != bytes.len() {
        return Err(PrivacyError::Malformed(
            "trailing bytes after canonical privacy record".to_string(),
        ));
    }
    Ok(fields)
}

fn fixed<const N: usize>(bytes: &[u8], label: &str) -> Result<[u8; N], PrivacyError> {
    bytes.try_into().map_err(|_| {
        PrivacyError::Malformed(format!(
            "{label} must be exactly {N} bytes, got {}",
            bytes.len()
        ))
    })
}

fn one_byte(bytes: &[u8], expected: u8, label: &str) -> Result<(), PrivacyError> {
    let value = fixed::<1>(bytes, label)?[0];
    if value != expected {
        return Err(PrivacyError::Malformed(format!(
            "unsupported {label} value {value}"
        )));
    }
    Ok(())
}

fn u32_field(bytes: &[u8], label: &str) -> Result<u32, PrivacyError> {
    let value = u32::from_be_bytes(fixed::<4>(bytes, label)?);
    if value == 0 {
        return Err(PrivacyError::Malformed(format!("{label} must be positive")));
    }
    Ok(value)
}

fn u64_field(bytes: &[u8], label: &str) -> Result<u64, PrivacyError> {
    Ok(u64::from_be_bytes(fixed::<8>(bytes, label)?))
}

fn uuid(bytes: &[u8], label: &str) -> Result<Uuid, PrivacyError> {
    let value = Uuid::from_bytes(fixed::<16>(bytes, label)?);
    validate_principal(value)?;
    Ok(value)
}

fn object_uuid(bytes: &[u8], label: &str) -> Result<Uuid, PrivacyError> {
    let value = Uuid::from_bytes(fixed::<16>(bytes, label)?);
    validate_object_id(value)?;
    Ok(value)
}

fn validate_object_id(object_id: Uuid) -> Result<(), PrivacyError> {
    let bytes = object_id.as_bytes();
    if object_id.is_nil() || bytes[6] >> 4 != 0x04 || bytes[8] >> 6 != 0x02 {
        return Err(PrivacyError::Malformed(
            "protected object identifier must be a non-nil RFC 9562 UUIDv4".to_string(),
        ));
    }
    Ok(())
}

fn validate_principal(principal: Uuid) -> Result<(), PrivacyError> {
    if principal.is_nil() {
        return Err(PrivacyError::Malformed(
            "participant principal UUID must be non-nil".to_string(),
        ));
    }
    Ok(())
}

fn ascii_identifier(bytes: &[u8], label: &str) -> Result<String, PrivacyError> {
    let value = std::str::from_utf8(bytes)
        .map_err(|_| PrivacyError::Malformed(format!("{label} must be ASCII")))?;
    validate_identifier(value, label)?;
    Ok(value.to_string())
}

fn validate_identifier(value: &str, label: &str) -> Result<(), PrivacyError> {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_IDENTIFIER_BYTES || !value.is_ascii() {
        return Err(PrivacyError::Malformed(format!(
            "{label} must contain 1..={MAX_IDENTIFIER_BYTES} ASCII bytes"
        )));
    }
    let valid_first = bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit();
    let valid_rest = bytes.iter().all(|byte| {
        byte.is_ascii_lowercase()
            || byte.is_ascii_digit()
            || matches!(byte, b'.' | b'_' | b':' | b'-')
    });
    if !valid_first || !valid_rest {
        return Err(PrivacyError::Malformed(format!(
            "{label} does not use the canonical identifier alphabet"
        )));
    }
    Ok(())
}

fn encode_parent_envelope_reference(parent: Option<[u8; 32]>) -> [u8; 33] {
    let mut reference = [0; 33];
    if let Some(hash) = parent {
        reference[0] = 1;
        reference[1..].copy_from_slice(&hash);
    }
    reference
}

fn decode_parent_envelope_reference(bytes: &[u8]) -> Result<Option<[u8; 32]>, PrivacyError> {
    let reference = fixed::<33>(bytes, "parent envelope reference")?;
    match reference[0] {
        0 if reference[1..] == [0; 32] => Ok(None),
        0 => Err(PrivacyError::Malformed(
            "genesis parent reference carries a nonzero hash".to_string(),
        )),
        1 => {
            let hash: [u8; 32] = reference[1..]
                .try_into()
                .map_err(|_| PrivacyError::Malformed("invalid parent hash".to_string()))?;
            if hash == [0; 32] {
                return Err(PrivacyError::Malformed(
                    "non-genesis parent reference carries an empty hash".to_string(),
                ));
            }
            Ok(Some(hash))
        }
        tag => Err(PrivacyError::Malformed(format!(
            "unknown parent envelope reference tag {tag}"
        ))),
    }
}

/// Ledger-derived lifecycle status; callers never supply this value in a transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum KeyLifecycleStatus {
    /// The only status eligible for prospective authorization or wrapping.
    Active = 0x01,
    /// Terminal routine-rotation status, eligible only for exact historical release.
    Retired = 0x02,
    /// Terminal emergency status, never eligible for authorization, wrapping, or release.
    Revoked = 0x03,
}

/// Closed prospective/historical eligibility question for a participant key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacyKeyUse {
    /// Authorize a participant-controlled privacy transition.
    AuthorizeTransition,
    /// Receive a newly constructed protected-object or grant envelope.
    CreateEnvelope,
    /// Open only an immutable historical envelope that names this exact key reference.
    HistoricalRelease,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParticipantKeyRecord {
    reference: ParticipantKeyReference,
    possession_scheme: PossessionProofScheme,
    public_key: Vec<u8>,
    status: KeyLifecycleStatus,
    effective_ledger_position: u64,
}

/// Public ledger-derived view used by an isolated custody reconciliation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipantPublicKeyRecord {
    /// Immutable purpose/version reference.
    pub reference: ParticipantKeyReference,
    /// Lifecycle status derived from the verified journal prefix.
    pub status: KeyLifecycleStatus,
    /// Canonical public key bytes; never private material.
    pub public_key: Vec<u8>,
}

/// Immutable journal-derived public record for one protected object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedObjectRecord {
    object_id: Uuid,
    owner: Uuid,
    authorization_key: ParticipantKeyReference,
    wrapping_key: ParticipantKeyReference,
    object_context: [u8; 32],
    payload: ProtectedPayloadCiphertext,
    encrypted_payload_commitment: [u8; 32],
    owner_envelope: OwnerDekEnvelope,
    effective_ledger_position: u64,
}

impl ProtectedObjectRecord {
    /// Stable RFC 9562 UUIDv4 selected before encryption.
    pub fn object_id(&self) -> Uuid {
        self.object_id
    }

    /// Participant Principal that owns this immutable object.
    pub fn owner(&self) -> Uuid {
        self.owner
    }

    /// Exact immutable protected payload record.
    pub fn payload(&self) -> &ProtectedPayloadCiphertext {
        &self.payload
    }

    /// Exact public object encryption context used as protected-payload AAD.
    pub fn object_context(&self) -> [u8; 32] {
        self.object_context
    }

    /// Exact immutable owner-addressed DEK envelope.
    pub fn owner_envelope(&self) -> &OwnerDekEnvelope {
        &self.owner_envelope
    }

    /// Public encrypted-payload commitment recomputable from the payload record.
    pub fn encrypted_payload_commitment(&self) -> [u8; 32] {
        self.encrypted_payload_commitment
    }

    /// Exact wrapping-key version addressed by the owner envelope.
    pub fn wrapping_key(&self) -> &ParticipantKeyReference {
        &self.wrapping_key
    }

    /// Ledger position at which the object became effective.
    pub fn effective_ledger_position(&self) -> u64 {
        self.effective_ledger_position
    }
}

/// Immutable journal-derived record for one grant and its terminal status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivacyGrantRecord {
    grant_id: [u8; 32],
    object_id: Uuid,
    owner: Uuid,
    grantee: Uuid,
    permission: PrivacyPermission,
    delivery: GrantDekEnvelope,
    status: PrivacyGrantStatus,
    creation_ledger_position: u64,
    effective_ledger_position: u64,
}

impl PrivacyGrantRecord {
    /// Immutable grant identifier derived from the complete GrantAccess core.
    pub fn grant_id(&self) -> [u8; 32] {
        self.grant_id
    }

    /// Protected object addressed by the grant.
    pub fn object_id(&self) -> Uuid {
        self.object_id
    }

    /// Object owner authorized by the grant.
    pub fn owner(&self) -> Uuid {
        self.owner
    }

    /// Participant receiving the grant.
    pub fn grantee(&self) -> Uuid {
        self.grantee
    }

    /// Permission carried by the grant.
    pub fn permission(&self) -> PrivacyPermission {
        self.permission
    }

    /// Current ledger-derived terminal status.
    pub fn status(&self) -> PrivacyGrantStatus {
        self.status
    }

    /// Exact immutable grant delivery envelope.
    pub fn delivery(&self) -> &GrantDekEnvelope {
        &self.delivery
    }

    /// Ledger position where the grant became effective.
    pub fn creation_ledger_position(&self) -> u64 {
        self.creation_ledger_position
    }

    /// Ledger position of the latest status transition.
    pub fn effective_ledger_position(&self) -> u64 {
        self.effective_ledger_position
    }
}

/// Deterministic journal-derived participant and key-lifecycle projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectivePrivacyState {
    revision: u64,
    principals: BTreeSet<Uuid>,
    keys: BTreeMap<ParticipantKeyReference, ParticipantKeyRecord>,
    protected_objects: BTreeMap<Uuid, ProtectedObjectRecord>,
    privacy_grants: BTreeMap<[u8; 32], PrivacyGrantRecord>,
    active_grants_by_pair: BTreeMap<(Uuid, Uuid), [u8; 32]>,
    consumed_transition_ids: BTreeSet<[u8; 32]>,
}

impl Default for EffectivePrivacyState {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectivePrivacyState {
    /// Create the empty virtual-genesis privacy state.
    pub fn new() -> Self {
        Self {
            revision: 0,
            principals: BTreeSet::new(),
            keys: BTreeMap::new(),
            protected_objects: BTreeMap::new(),
            privacy_grants: BTreeMap::new(),
            active_grants_by_pair: BTreeMap::new(),
            consumed_transition_ids: BTreeSet::new(),
        }
    }

    /// Current committed privacy revision.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Whether a participant principal has been atomically registered.
    pub fn contains_principal(&self, principal: Uuid) -> bool {
        self.principals.contains(&principal)
    }

    /// Current reducer-derived status of one exact immutable key reference.
    pub fn key_status(&self, reference: &ParticipantKeyReference) -> Option<KeyLifecycleStatus> {
        self.keys.get(reference).map(|record| record.status)
    }

    /// Resolve the unique active key for a principal and purpose.
    pub fn active_key(
        &self,
        principal: Uuid,
        purpose: ParticipantKeyPurpose,
    ) -> Option<&ParticipantKeyReference> {
        self.keys
            .values()
            .find(|record| {
                record.reference.principal == principal
                    && record.reference.purpose == purpose
                    && record.status == KeyLifecycleStatus::Active
            })
            .map(|record| &record.reference)
    }

    /// Resolve one immutable protected object from the journal-derived projection.
    pub fn protected_object(&self, object_id: Uuid) -> Option<&ProtectedObjectRecord> {
        self.protected_objects.get(&object_id)
    }

    /// Resolve one immutable grant, including a terminal revoked history record.
    pub fn privacy_grant(&self, grant_id: &[u8; 32]) -> Option<&PrivacyGrantRecord> {
        self.privacy_grants.get(grant_id)
    }

    /// Resolve the one currently active grant for an object and grantee pair.
    pub fn active_privacy_grant(
        &self,
        object_id: Uuid,
        grantee: Uuid,
    ) -> Option<&PrivacyGrantRecord> {
        self.active_grants_by_pair
            .get(&(object_id, grantee))
            .and_then(|grant_id| self.privacy_grants.get(grant_id))
            .filter(|grant| grant.status == PrivacyGrantStatus::Active)
    }

    /// All immutable grant history records for one protected object in identifier order.
    pub fn privacy_grants_for_object(&self, object_id: Uuid) -> Vec<&PrivacyGrantRecord> {
        self.privacy_grants
            .values()
            .filter(|grant| grant.object_id == object_id)
            .collect()
    }

    /// Public key bytes for an exact ledger-derived participant key reference.
    pub fn participant_public_key(&self, reference: &ParticipantKeyReference) -> Option<&[u8]> {
        self.keys
            .get(reference)
            .map(|record| record.public_key.as_slice())
    }

    /// Complete public key set for one participant at this exact state revision.
    pub fn participant_keys(&self, principal: Uuid) -> Vec<ParticipantPublicKeyRecord> {
        self.keys
            .values()
            .filter(|record| record.reference.principal == principal)
            .map(|record| ParticipantPublicKeyRecord {
                reference: record.reference.clone(),
                status: record.status,
                public_key: record.public_key.clone(),
            })
            .collect()
    }

    /// Evaluate purpose and lifecycle eligibility for one exact key reference.
    pub fn key_is_eligible(
        &self,
        reference: &ParticipantKeyReference,
        requested_use: PrivacyKeyUse,
    ) -> bool {
        let Some(record) = self.keys.get(reference) else {
            return false;
        };
        match requested_use {
            PrivacyKeyUse::AuthorizeTransition => {
                record.reference.purpose == ParticipantKeyPurpose::PrivacyAuthorization
                    && record.status == KeyLifecycleStatus::Active
            }
            PrivacyKeyUse::CreateEnvelope => {
                record.reference.purpose == ParticipantKeyPurpose::PrivacyKeyWrapping
                    && record.status == KeyLifecycleStatus::Active
            }
            PrivacyKeyUse::HistoricalRelease => {
                record.reference.purpose == ParticipantKeyPurpose::PrivacyKeyWrapping
                    && matches!(
                        record.status,
                        KeyLifecycleStatus::Active | KeyLifecycleStatus::Retired
                    )
            }
        }
    }

    /// Canonical deterministic digest of the complete effective lifecycle projection.
    pub fn digest(&self) -> [u8; 32] {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"PROVCHAIN_EFFECTIVE_PRIVACY_STATE_V1");
        bytes.extend_from_slice(&self.revision.to_be_bytes());
        bytes.extend_from_slice(
            &u64::try_from(self.principals.len())
                .expect("bounded principal count fits u64")
                .to_be_bytes(),
        );
        for principal in &self.principals {
            bytes.extend_from_slice(principal.as_bytes());
        }
        bytes.extend_from_slice(
            &u64::try_from(self.keys.len())
                .expect("bounded participant key count fits u64")
                .to_be_bytes(),
        );
        for record in self.keys.values() {
            let reference = record.reference.canonical_bytes();
            bytes.extend_from_slice(
                &u32::try_from(reference.len())
                    .expect("bounded key reference fits u32")
                    .to_be_bytes(),
            );
            bytes.extend_from_slice(&reference);
            bytes.push(record.possession_scheme as u8);
            bytes.extend_from_slice(
                &u32::try_from(record.public_key.len())
                    .expect("bounded public key fits u32")
                    .to_be_bytes(),
            );
            bytes.extend_from_slice(&record.public_key);
            bytes.push(record.status as u8);
            bytes.extend_from_slice(&record.effective_ledger_position.to_be_bytes());
        }
        bytes.extend_from_slice(
            &u64::try_from(self.protected_objects.len())
                .expect("bounded protected object count fits u64")
                .to_be_bytes(),
        );
        for object in self.protected_objects.values() {
            bytes.extend_from_slice(object.object_id.as_bytes());
            bytes.extend_from_slice(object.owner.as_bytes());
            let authorization_reference = object.authorization_key.canonical_bytes();
            bytes.extend_from_slice(
                &u32::try_from(authorization_reference.len())
                    .expect("bounded key reference fits u32")
                    .to_be_bytes(),
            );
            bytes.extend_from_slice(&authorization_reference);
            let wrapping_reference = object.wrapping_key.canonical_bytes();
            bytes.extend_from_slice(
                &u32::try_from(wrapping_reference.len())
                    .expect("bounded key reference fits u32")
                    .to_be_bytes(),
            );
            bytes.extend_from_slice(&wrapping_reference);
            bytes.extend_from_slice(&object.object_context);
            let payload = object.payload.canonical_bytes();
            bytes.extend_from_slice(
                &u32::try_from(payload.len())
                    .expect("bounded payload fits u32")
                    .to_be_bytes(),
            );
            bytes.extend_from_slice(&payload);
            bytes.extend_from_slice(&object.encrypted_payload_commitment);
            let envelope = object.owner_envelope.canonical_bytes();
            bytes.extend_from_slice(
                &u32::try_from(envelope.len())
                    .expect("bounded owner envelope fits u32")
                    .to_be_bytes(),
            );
            bytes.extend_from_slice(&envelope);
            bytes.extend_from_slice(&object.effective_ledger_position.to_be_bytes());
        }
        bytes.extend_from_slice(
            &u64::try_from(self.privacy_grants.len())
                .expect("bounded privacy grant count fits u64")
                .to_be_bytes(),
        );
        for grant in self.privacy_grants.values() {
            bytes.extend_from_slice(&grant.grant_id);
            bytes.extend_from_slice(grant.object_id.as_bytes());
            bytes.extend_from_slice(grant.owner.as_bytes());
            bytes.extend_from_slice(grant.grantee.as_bytes());
            bytes.push(grant.permission as u8);
            let delivery = grant.delivery.canonical_bytes();
            bytes.extend_from_slice(
                &u32::try_from(delivery.len())
                    .expect("bounded grant envelope fits u32")
                    .to_be_bytes(),
            );
            bytes.extend_from_slice(&delivery);
            bytes.push(grant.status as u8);
            bytes.extend_from_slice(&grant.creation_ledger_position.to_be_bytes());
            bytes.extend_from_slice(&grant.effective_ledger_position.to_be_bytes());
        }
        bytes.extend_from_slice(
            &u64::try_from(self.consumed_transition_ids.len())
                .expect("bounded transition count fits u64")
                .to_be_bytes(),
        );
        for transition_id in &self.consumed_transition_ids {
            bytes.extend_from_slice(transition_id);
        }
        Sha256::digest(bytes).into()
    }

    /// Verify a complete transition against one exact parent and return its staged child state.
    ///
    /// The parent is never mutated. Callers may publish the returned projection only after the
    /// enclosing Admitted Block Envelope reaches the journal append-plus-`fsync` commit point.
    pub fn stage_transition(
        &self,
        profile: &PrivacyLifecycleProfile,
        expected_anchor: &PrivacyAdmissionAnchor,
        transition: &PrivacyControlTransition,
    ) -> Result<Self, PrivacyError> {
        profile.validate()?;
        expected_anchor.validate()?;
        match &transition.kind {
            PrivacyTransitionKind::RegisterPrincipal {
                core,
                transition_id,
                governance_signature,
                possession_signature,
            } => self.stage_registration(
                profile,
                expected_anchor,
                core,
                *transition_id,
                *governance_signature,
                *possession_signature,
            ),
            PrivacyTransitionKind::BindParticipantKey {
                core,
                transition_id,
                authorization_signature,
                possession_proof,
            } => self.stage_binding(
                profile,
                expected_anchor,
                core,
                *transition_id,
                *authorization_signature,
                *possession_proof,
            ),
            PrivacyTransitionKind::RevokeParticipantKey {
                core,
                transition_id,
                authorization_signature,
            } => self.stage_revocation(
                profile,
                expected_anchor,
                core,
                *transition_id,
                *authorization_signature,
            ),
            PrivacyTransitionKind::CreateProtectedObject {
                core,
                transition_id,
                authorization_signature,
            } => self.stage_object_creation(
                profile,
                expected_anchor,
                core,
                *transition_id,
                *authorization_signature,
            ),
            PrivacyTransitionKind::GrantAccess {
                core,
                grant_id,
                authorization_signature,
            } => self.stage_grant_access(
                profile,
                expected_anchor,
                core,
                *grant_id,
                *authorization_signature,
            ),
            PrivacyTransitionKind::RevokeGrant {
                core,
                transition_id,
                authorization_signature,
            } => self.stage_grant_revocation(
                profile,
                expected_anchor,
                core,
                *transition_id,
                *authorization_signature,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn stage_registration(
        &self,
        profile: &PrivacyLifecycleProfile,
        expected_anchor: &PrivacyAdmissionAnchor,
        core: &RegisterPrincipalCore,
        transition_id: [u8; 32],
        governance_signature: [u8; 64],
        possession_signature: [u8; 64],
    ) -> Result<Self, PrivacyError> {
        if &core.anchor != expected_anchor {
            return Err(PrivacyError::Rejected(
                "registration parent anchor does not match verified ledger state".to_string(),
            ));
        }
        if core.anchor.profile_content_hash != profile.network_profile_content_hash {
            return Err(PrivacyError::Rejected(
                "registration profile content hash mismatch".to_string(),
            ));
        }
        let expected_revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| PrivacyError::Rejected("privacy revision overflow".to_string()))?;
        if core.anchor.expected_privacy_revision != expected_revision {
            return Err(PrivacyError::Rejected(format!(
                "expected privacy revision {}, got {}",
                expected_revision, core.anchor.expected_privacy_revision
            )));
        }
        if core.bootstrap_governance_key != profile.bootstrap_reference() {
            return Err(PrivacyError::Rejected(
                "bootstrap governance key reference does not match the parent profile".to_string(),
            ));
        }
        if self.principals.contains(&core.principal) {
            return Err(PrivacyError::Rejected(
                "participant principal is already registered".to_string(),
            ));
        }
        if self.consumed_transition_ids.contains(&transition_id) {
            return Err(PrivacyError::Rejected(
                "privacy transition identifier was already consumed".to_string(),
            ));
        }
        let binding = &core.initial_authorization_key;
        if profile.key_reuses_profile_role(&binding.public_key) {
            return Err(PrivacyError::Rejected(
                "participant key reuses a Network Profile key role".to_string(),
            ));
        }
        if self.keys.values().any(|existing| {
            existing.public_key == binding.public_key
                || existing.reference.fingerprint == binding.reference.fingerprint
        }) {
            return Err(PrivacyError::Rejected(
                "participant public key or fingerprint is already bound".to_string(),
            ));
        }

        let core_bytes = core.canonical_bytes();
        let expected_transition_id = domain_hash(REGISTER_TRANSITION_ID_DOMAIN, &core_bytes);
        if transition_id != expected_transition_id {
            return Err(PrivacyError::Rejected(
                "registration transition identifier mismatch".to_string(),
            ));
        }
        let proof_payload = concat_digest_and_record(transition_id, &core_bytes);
        let authorization_digest = domain_hash(REGISTER_AUTHORIZATION_DOMAIN, &proof_payload);
        let possession_digest = domain_hash(REGISTER_POSSESSION_DOMAIN, &proof_payload);
        let governance_verifier = VerifyingKey::from_bytes(&profile.bootstrap_public_key())
            .map_err(|error| {
                PrivacyError::InvalidProfile(format!(
                    "invalid bootstrap governance key during admission: {error}"
                ))
            })?;
        governance_verifier
            .verify_strict(
                &authorization_digest,
                &Signature::from_bytes(&governance_signature),
            )
            .map_err(|_| {
                PrivacyError::Rejected(
                    "invalid bootstrap governance authorization proof".to_string(),
                )
            })?;
        let participant_public_key: [u8; 32] = binding
            .public_key
            .as_slice()
            .try_into()
            .map_err(|_| PrivacyError::Rejected("invalid possession key length".to_string()))?;
        let possession_verifier = VerifyingKey::from_bytes(&participant_public_key)
            .map_err(|error| PrivacyError::Rejected(format!("invalid possession key: {error}")))?;
        possession_verifier
            .verify_strict(
                &possession_digest,
                &Signature::from_bytes(&possession_signature),
            )
            .map_err(|_| {
                PrivacyError::Rejected("invalid participant key possession proof".to_string())
            })?;

        let mut staged = self.clone();
        staged.principals.insert(core.principal);
        staged.keys.insert(
            binding.reference.clone(),
            ParticipantKeyRecord {
                reference: binding.reference.clone(),
                possession_scheme: binding.possession_scheme,
                public_key: binding.public_key.clone(),
                status: KeyLifecycleStatus::Active,
                effective_ledger_position: core.anchor.expected_ledger_position,
            },
        );
        staged.consumed_transition_ids.insert(transition_id);
        staged.revision = expected_revision;
        Ok(staged)
    }

    #[allow(clippy::too_many_arguments)]
    fn stage_binding(
        &self,
        profile: &PrivacyLifecycleProfile,
        expected_anchor: &PrivacyAdmissionAnchor,
        core: &BindParticipantKeyCore,
        transition_id: [u8; 32],
        authorization_signature: [u8; 64],
        possession_proof: [u8; 64],
    ) -> Result<Self, PrivacyError> {
        let expected_revision = self.validate_common_transition_parent(
            profile,
            expected_anchor,
            &core.anchor,
            transition_id,
        )?;
        if !self.principals.contains(&core.affected_principal) {
            return Err(PrivacyError::Rejected(
                "affected participant principal is not registered".to_string(),
            ));
        }
        if core.introduced_key.reference.principal != core.affected_principal {
            return Err(PrivacyError::Rejected(
                "introduced key does not belong to the affected principal".to_string(),
            ));
        }
        let authorizer =
            self.active_authorization_record(core.affected_principal, &core.authorizer_key)?;
        let binding = &core.introduced_key;
        if profile.key_reuses_profile_role(&binding.public_key) {
            return Err(PrivacyError::Rejected(
                "participant key reuses a Network Profile key role".to_string(),
            ));
        }
        if self.keys.values().any(|existing| {
            existing.public_key == binding.public_key
                || existing.reference.fingerprint == binding.reference.fingerprint
        }) {
            return Err(PrivacyError::Rejected(
                "participant public key or fingerprint is already bound".to_string(),
            ));
        }

        let purpose = binding.reference.purpose;
        let mut purpose_records: Vec<&ParticipantKeyRecord> = self
            .keys
            .values()
            .filter(|record| {
                record.reference.principal == core.affected_principal
                    && record.reference.purpose == purpose
            })
            .collect();
        purpose_records.sort_by_key(|record| record.reference.version);
        let expected_version = match purpose_records.last() {
            Some(record) => record.reference.version.checked_add(1).ok_or_else(|| {
                PrivacyError::Rejected("participant key version overflow".to_string())
            })?,
            None if purpose == ParticipantKeyPurpose::PrivacyAuthorization => {
                return Err(PrivacyError::Rejected(
                    "authorization version 1 can only be created by RegisterPrincipal".to_string(),
                ));
            }
            None => 1,
        };
        if binding.reference.version != expected_version {
            return Err(PrivacyError::Rejected(format!(
                "participant key version must be contiguous: expected {expected_version}, got {}",
                binding.reference.version
            )));
        }
        let active_predecessor = purpose_records
            .iter()
            .find(|record| record.status == KeyLifecycleStatus::Active)
            .copied();
        if purpose == ParticipantKeyPurpose::PrivacyAuthorization && active_predecessor.is_none() {
            return Err(PrivacyError::Rejected(
                "participant-control freeze: no active privacy-authorization key".to_string(),
            ));
        }

        let core_bytes = core.canonical_bytes();
        if transition_id != domain_hash(BIND_TRANSITION_ID_DOMAIN, &core_bytes) {
            return Err(PrivacyError::Rejected(
                "binding transition identifier mismatch".to_string(),
            ));
        }
        let proof_payload = concat_digest_and_record(transition_id, &core_bytes);
        let authorization_digest = domain_hash(BIND_AUTHORIZATION_DOMAIN, &proof_payload);
        verify_ed25519_record(
            authorizer,
            &authorization_digest,
            authorization_signature,
            "participant authorization",
        )?;
        let possession_digest = domain_hash(BIND_POSSESSION_DOMAIN, &proof_payload);
        verify_possession_proof(binding, &possession_digest, possession_proof)?;

        let mut staged = self.clone();
        if let Some(predecessor) = active_predecessor {
            let predecessor = staged
                .keys
                .get_mut(&predecessor.reference)
                .ok_or_else(|| PrivacyError::Rejected("missing active predecessor".to_string()))?;
            predecessor.status = KeyLifecycleStatus::Retired;
        }
        staged.keys.insert(
            binding.reference.clone(),
            ParticipantKeyRecord {
                reference: binding.reference.clone(),
                possession_scheme: binding.possession_scheme,
                public_key: binding.public_key.clone(),
                status: KeyLifecycleStatus::Active,
                effective_ledger_position: core.anchor.expected_ledger_position,
            },
        );
        staged.consumed_transition_ids.insert(transition_id);
        staged.revision = expected_revision;
        Ok(staged)
    }

    fn stage_revocation(
        &self,
        profile: &PrivacyLifecycleProfile,
        expected_anchor: &PrivacyAdmissionAnchor,
        core: &RevokeParticipantKeyCore,
        transition_id: [u8; 32],
        authorization_signature: [u8; 64],
    ) -> Result<Self, PrivacyError> {
        let expected_revision = self.validate_common_transition_parent(
            profile,
            expected_anchor,
            &core.anchor,
            transition_id,
        )?;
        if core.target_key.principal != core.affected_principal {
            return Err(PrivacyError::Rejected(
                "revocation cannot target another participant principal".to_string(),
            ));
        }
        let authorizer =
            self.active_authorization_record(core.affected_principal, &core.authorizer_key)?;
        let target = self
            .keys
            .values()
            .find(|record| {
                record.reference.principal == core.target_key.principal
                    && record.reference.purpose == core.target_key.purpose
                    && record.reference.version == core.target_key.version
            })
            .ok_or_else(|| {
                PrivacyError::Rejected("revocation target key does not exist".to_string())
            })?;
        if target.status != KeyLifecycleStatus::Active {
            return Err(PrivacyError::Rejected(
                "revocation target must be the active key version".to_string(),
            ));
        }

        let core_bytes = core.canonical_bytes();
        if transition_id != domain_hash(REVOKE_TRANSITION_ID_DOMAIN, &core_bytes) {
            return Err(PrivacyError::Rejected(
                "revocation transition identifier mismatch".to_string(),
            ));
        }
        let proof_payload = concat_digest_and_record(transition_id, &core_bytes);
        let authorization_digest = domain_hash(REVOKE_AUTHORIZATION_DOMAIN, &proof_payload);
        verify_ed25519_record(
            authorizer,
            &authorization_digest,
            authorization_signature,
            "participant authorization",
        )?;

        let mut staged = self.clone();
        let target = staged
            .keys
            .get_mut(&target.reference)
            .ok_or_else(|| PrivacyError::Rejected("missing revocation target".to_string()))?;
        target.status = KeyLifecycleStatus::Revoked;
        staged.consumed_transition_ids.insert(transition_id);
        staged.revision = expected_revision;
        Ok(staged)
    }

    fn stage_object_creation(
        &self,
        profile: &PrivacyLifecycleProfile,
        expected_anchor: &PrivacyAdmissionAnchor,
        core: &CreateProtectedObjectCore,
        transition_id: [u8; 32],
        authorization_signature: [u8; 64],
    ) -> Result<Self, PrivacyError> {
        let expected_revision = self.validate_common_transition_parent(
            profile,
            expected_anchor,
            &core.anchor,
            transition_id,
        )?;
        if !self.principals.contains(&core.owner) {
            return Err(PrivacyError::Rejected(
                "protected-object owner is not a registered participant".to_string(),
            ));
        }
        if self.protected_objects.contains_key(&core.object_id) {
            return Err(PrivacyError::Rejected(
                "protected object identifier already exists at the parent".to_string(),
            ));
        }
        let authorizer = self.active_authorization_record(core.owner, &core.authorization_key)?;
        if !self.key_is_eligible(&core.wrapping_key, PrivacyKeyUse::CreateEnvelope) {
            return Err(PrivacyError::Rejected(
                "owner wrapping key is not Active at the verified parent".to_string(),
            ));
        }
        validate_protected_object_material(
            &core.anchor,
            core.object_id,
            core.owner,
            &core.wrapping_key,
            core.object_context,
            &core.payload,
            core.encrypted_payload_commitment,
            &core.owner_envelope,
        )
        .map_err(|error| PrivacyError::Rejected(error.to_string()))?;
        let core_bytes = core.canonical_bytes();
        if transition_id != domain_hash(CREATE_OBJECT_TRANSITION_ID_DOMAIN, &core_bytes) {
            return Err(PrivacyError::Rejected(
                "object-creation transition identifier mismatch".to_string(),
            ));
        }
        let proof_payload = concat_digest_and_record(transition_id, &core_bytes);
        let authorization_digest = domain_hash(CREATE_OBJECT_AUTHORIZATION_DOMAIN, &proof_payload);
        verify_ed25519_record(
            authorizer,
            &authorization_digest,
            authorization_signature,
            "protected-object owner authorization",
        )?;

        let mut staged = self.clone();
        staged.protected_objects.insert(
            core.object_id,
            ProtectedObjectRecord {
                object_id: core.object_id,
                owner: core.owner,
                authorization_key: core.authorization_key.clone(),
                wrapping_key: core.wrapping_key.clone(),
                object_context: core.object_context,
                payload: core.payload.clone(),
                encrypted_payload_commitment: core.encrypted_payload_commitment,
                owner_envelope: core.owner_envelope.clone(),
                effective_ledger_position: core.anchor.expected_ledger_position,
            },
        );
        staged.consumed_transition_ids.insert(transition_id);
        staged.revision = expected_revision;
        Ok(staged)
    }

    fn stage_grant_access(
        &self,
        profile: &PrivacyLifecycleProfile,
        expected_anchor: &PrivacyAdmissionAnchor,
        core: &GrantAccessCore,
        grant_id: [u8; 32],
        authorization_signature: [u8; 64],
    ) -> Result<Self, PrivacyError> {
        let expected_revision = self.validate_common_transition_parent(
            profile,
            expected_anchor,
            &core.anchor,
            grant_id,
        )?;
        let object = self
            .protected_objects
            .get(&core.object_id)
            .ok_or_else(|| PrivacyError::Rejected("protected object does not exist".to_string()))?;
        if core.owner != object.owner {
            return Err(PrivacyError::Rejected(
                "grant owner is not derived from the protected object".to_string(),
            ));
        }
        if core.owner == core.grantee || !self.principals.contains(&core.grantee) {
            return Err(PrivacyError::Rejected(
                "grant grantee must be a distinct registered participant".to_string(),
            ));
        }
        if self
            .active_grants_by_pair
            .contains_key(&(core.object_id, core.grantee))
        {
            return Err(PrivacyError::Rejected(
                "an active grant already exists for this object and grantee".to_string(),
            ));
        }
        if self.privacy_grants.contains_key(&grant_id) {
            return Err(PrivacyError::Rejected(
                "grant identifier already exists in immutable history".to_string(),
            ));
        }
        let authorizer = self.active_authorization_record(core.owner, &core.authorization_key)?;
        let wrapping_key = core.delivery.wrapping_key();
        if !self.key_is_eligible(wrapping_key, PrivacyKeyUse::CreateEnvelope)
            || self.participant_public_key(wrapping_key).is_none()
        {
            return Err(PrivacyError::Rejected(
                "grantee wrapping key is not Active at the verified parent".to_string(),
            ));
        }
        validate_grant_delivery_material(
            &core.anchor,
            core.object_id,
            core.owner,
            core.grantee,
            core.permission,
            &core.delivery,
            Some(object.payload.protected_content_commitment),
            Some(object.encrypted_payload_commitment),
        )
        .map_err(|error| PrivacyError::Rejected(error.to_string()))?;
        let core_bytes = core.canonical_bytes();
        if grant_id != domain_hash(GRANT_ID_DOMAIN, &core_bytes) {
            return Err(PrivacyError::Rejected(
                "grant identifier mismatch".to_string(),
            ));
        }
        let proof_payload = concat_digest_and_record(grant_id, &core_bytes);
        let authorization_digest = domain_hash(GRANT_AUTHORIZATION_DOMAIN, &proof_payload);
        verify_ed25519_record(
            authorizer,
            &authorization_digest,
            authorization_signature,
            "privacy grant owner authorization",
        )?;

        let mut staged = self.clone();
        staged.privacy_grants.insert(
            grant_id,
            PrivacyGrantRecord {
                grant_id,
                object_id: core.object_id,
                owner: core.owner,
                grantee: core.grantee,
                permission: core.permission,
                delivery: core.delivery.clone(),
                status: PrivacyGrantStatus::Active,
                creation_ledger_position: core.anchor.expected_ledger_position,
                effective_ledger_position: core.anchor.expected_ledger_position,
            },
        );
        staged
            .active_grants_by_pair
            .insert((core.object_id, core.grantee), grant_id);
        staged.consumed_transition_ids.insert(grant_id);
        staged.revision = expected_revision;
        Ok(staged)
    }

    fn stage_grant_revocation(
        &self,
        profile: &PrivacyLifecycleProfile,
        expected_anchor: &PrivacyAdmissionAnchor,
        core: &RevokeGrantCore,
        transition_id: [u8; 32],
        authorization_signature: [u8; 64],
    ) -> Result<Self, PrivacyError> {
        let expected_revision = self.validate_common_transition_parent(
            profile,
            expected_anchor,
            &core.anchor,
            transition_id,
        )?;
        let grant = self
            .privacy_grants
            .get(&core.grant_id)
            .ok_or_else(|| PrivacyError::Rejected("grant identifier does not exist".to_string()))?;
        if grant.status != PrivacyGrantStatus::Active {
            return Err(PrivacyError::Rejected(
                "grant revocation is terminal and cannot repeat".to_string(),
            ));
        }
        if grant.object_id != core.object_id
            || grant.grantee != core.grantee
            || grant.owner != core.owner
            || grant.permission != core.permission
            || grant.creation_ledger_position != core.creation_ledger_position
            || self
                .active_grants_by_pair
                .get(&(core.object_id, core.grantee))
                != Some(&core.grant_id)
        {
            return Err(PrivacyError::Rejected(
                "grant revocation fields do not match immutable grant history".to_string(),
            ));
        }
        let authorizer = self.active_authorization_record(core.owner, &core.authorization_key)?;
        let core_bytes = core.canonical_bytes();
        if transition_id != domain_hash(REVOKE_GRANT_TRANSITION_ID_DOMAIN, &core_bytes) {
            return Err(PrivacyError::Rejected(
                "grant revocation transition identifier mismatch".to_string(),
            ));
        }
        let proof_payload = concat_digest_and_record(transition_id, &core_bytes);
        let authorization_digest = domain_hash(REVOKE_GRANT_AUTHORIZATION_DOMAIN, &proof_payload);
        verify_ed25519_record(
            authorizer,
            &authorization_digest,
            authorization_signature,
            "privacy grant revocation owner authorization",
        )?;

        let mut staged = self.clone();
        let staged_grant = staged
            .privacy_grants
            .get_mut(&core.grant_id)
            .ok_or_else(|| PrivacyError::Rejected("missing grant history".to_string()))?;
        staged_grant.status = PrivacyGrantStatus::Revoked;
        staged_grant.effective_ledger_position = core.anchor.expected_ledger_position;
        staged
            .active_grants_by_pair
            .remove(&(core.object_id, core.grantee));
        staged.consumed_transition_ids.insert(transition_id);
        staged.revision = expected_revision;
        Ok(staged)
    }

    fn validate_common_transition_parent(
        &self,
        profile: &PrivacyLifecycleProfile,
        expected_anchor: &PrivacyAdmissionAnchor,
        actual_anchor: &PrivacyAdmissionAnchor,
        transition_id: [u8; 32],
    ) -> Result<u64, PrivacyError> {
        if actual_anchor != expected_anchor {
            return Err(PrivacyError::Rejected(
                "privacy transition parent anchor does not match verified ledger state".to_string(),
            ));
        }
        if actual_anchor.profile_content_hash != profile.network_profile_content_hash {
            return Err(PrivacyError::Rejected(
                "privacy transition profile content hash mismatch".to_string(),
            ));
        }
        if self.consumed_transition_ids.contains(&transition_id) {
            return Err(PrivacyError::Rejected(
                "privacy transition identifier was already consumed".to_string(),
            ));
        }
        let expected_revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| PrivacyError::Rejected("privacy revision overflow".to_string()))?;
        if actual_anchor.expected_privacy_revision != expected_revision {
            return Err(PrivacyError::Rejected(format!(
                "expected privacy revision {expected_revision}, got {}",
                actual_anchor.expected_privacy_revision
            )));
        }
        Ok(expected_revision)
    }

    fn active_authorization_record(
        &self,
        affected_principal: Uuid,
        supplied_reference: &ParticipantKeyReference,
    ) -> Result<&ParticipantKeyRecord, PrivacyError> {
        if supplied_reference.principal != affected_principal
            || supplied_reference.purpose != ParticipantKeyPurpose::PrivacyAuthorization
        {
            return Err(PrivacyError::Rejected(
                "authorizer is not the affected principal's privacy-authorization key".to_string(),
            ));
        }
        let authorizer = self.keys.get(supplied_reference).ok_or_else(|| {
            PrivacyError::Rejected(
                "authorizer key reference does not resolve in parent state".to_string(),
            )
        })?;
        if authorizer.status != KeyLifecycleStatus::Active {
            return Err(PrivacyError::Rejected(
                "participant-control freeze: no active privacy-authorization key".to_string(),
            ));
        }
        Ok(authorizer)
    }
}

fn verify_ed25519_record(
    record: &ParticipantKeyRecord,
    digest: &[u8; 32],
    signature: [u8; 64],
    proof_label: &str,
) -> Result<(), PrivacyError> {
    if record.reference.algorithm != KeyAlgorithm::Ed25519
        || record.reference.encoding != PublicKeyEncoding::Ed25519Raw32
        || record.possession_scheme != PossessionProofScheme::Ed25519Raw64
    {
        return Err(PrivacyError::Rejected(format!(
            "{proof_label} key uses the wrong purpose-specific scheme"
        )));
    }
    let public_key: [u8; 32] = record
        .public_key
        .as_slice()
        .try_into()
        .map_err(|_| PrivacyError::Rejected(format!("invalid {proof_label} key length")))?;
    let verifier = VerifyingKey::from_bytes(&public_key)
        .map_err(|error| PrivacyError::Rejected(format!("invalid {proof_label} key: {error}")))?;
    verifier
        .verify_strict(digest, &Signature::from_bytes(&signature))
        .map_err(|_| PrivacyError::Rejected(format!("invalid {proof_label} proof")))
}

fn verify_possession_proof(
    binding: &ParticipantKeyBinding,
    digest: &[u8; 32],
    proof: [u8; 64],
) -> Result<(), PrivacyError> {
    match binding.reference.purpose {
        ParticipantKeyPurpose::PrivacyAuthorization => verify_ed25519_record(
            &ParticipantKeyRecord {
                reference: binding.reference.clone(),
                possession_scheme: binding.possession_scheme,
                public_key: binding.public_key.clone(),
                status: KeyLifecycleStatus::Active,
                effective_ledger_position: 0,
            },
            digest,
            proof,
            "participant key possession",
        ),
        ParticipantKeyPurpose::PrivacyKeyWrapping => {
            if binding.reference.algorithm != KeyAlgorithm::P256
                || binding.reference.encoding != PublicKeyEncoding::Sec1UncompressedP256
                || binding.possession_scheme != PossessionProofScheme::EcdsaP256Sha256RawLowS64
                || binding.public_key.len() != 65
                || binding.public_key.first() != Some(&0x04)
            {
                return Err(PrivacyError::Rejected(
                    "wrapping possession key uses the wrong purpose-specific scheme".to_string(),
                ));
            }
            let signature = P256Signature::from_slice(&proof).map_err(|_| {
                PrivacyError::Rejected(
                    "invalid raw P-256 participant key possession proof".to_string(),
                )
            })?;
            if signature.normalize_s() != signature {
                return Err(PrivacyError::Rejected(
                    "high-S P-256 participant key possession proof is forbidden".to_string(),
                ));
            }
            let verifier =
                P256VerifyingKey::from_sec1_bytes(&binding.public_key).map_err(|error| {
                    PrivacyError::Rejected(format!(
                        "invalid P-256 participant possession key: {error}"
                    ))
                })?;
            verifier.verify(digest, &signature).map_err(|_| {
                PrivacyError::Rejected("invalid P-256 participant key possession proof".to_string())
            })
        }
    }
}

/// Canonical lifecycle/profile/transition failure.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PrivacyError {
    /// The profile cannot be activated safely.
    #[error("invalid privacy lifecycle profile: {0}")]
    InvalidProfile(String),
    /// Candidate bytes or a typed field violate the closed schema.
    #[error("malformed privacy control transition: {0}")]
    Malformed(String),
    /// The record decodes but is not the one canonical encoding of its values.
    #[error("noncanonical privacy control record")]
    NonCanonicalRecord,
    /// A transition variant unsupported by the selected lifecycle slice.
    #[error("privacy transition record tag {0:#04x} is not supported by this lifecycle slice")]
    UnsupportedTransitionVariant(u8),
    /// The complete record tag is outside the closed PrivacyControlV1 union.
    #[error("unknown privacy transition record tag {0:#04x}")]
    UnknownTransitionRecord(u8),
    /// A healthy node deterministically rejected a complete candidate.
    #[error("privacy control transition rejected: {0}")]
    Rejected(String),
}

#[cfg(test)]
mod issue10_tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use p256::ecdsa::SigningKey as P256SigningKey;

    const NETWORK_ID: &str = "provchain.issue10.unit";
    const PROFILE_ID: &str = "issue10.unit";

    fn p256_public(key: &P256SigningKey) -> [u8; 65] {
        key.verifying_key()
            .to_sec1_point(false)
            .as_bytes()
            .try_into()
            .expect("uncompressed P-256 public key")
    }

    #[allow(clippy::too_many_arguments)]
    fn grant_delivery(
        anchor: &PrivacyAdmissionAnchor,
        object_id: Uuid,
        owner: Uuid,
        grantee: Uuid,
        wrapping_key: ParticipantKeyReference,
        protected_content_commitment: [u8; 32],
        encrypted_payload_commitment: [u8; 32],
        ephemeral_key: &P256SigningKey,
    ) -> GrantDekEnvelope {
        let header = GrantDekHeader::from_parts(
            anchor,
            object_id,
            owner,
            grantee,
            PrivacyPermission::ReadProtectedObjectV1,
            wrapping_key.clone(),
            protected_content_commitment,
            encrypted_payload_commitment,
        );
        let context = domain_hash(
            GRANT_DELIVERY_CONTEXT_DOMAIN,
            &grant_delivery_context_bytes(
                anchor,
                object_id,
                owner,
                grantee,
                PrivacyPermission::ReadProtectedObjectV1,
                &wrapping_key,
                protected_content_commitment,
                encrypted_payload_commitment,
            ),
        );
        let info = grant_hpke_info(context, &header);
        assert_eq!(info.len(), 64);
        GrantDekEnvelope::new(context, header, p256_public(ephemeral_key), [0x51; 48])
            .expect("canonical grant envelope")
    }

    type Issue10State = (
        PrivacyLifecycleProfile,
        EffectivePrivacyState,
        PrivacyAdmissionAnchor,
        Uuid,
        Uuid,
        SigningKey,
        ParticipantKeyReference,
        ParticipantKeyReference,
        ParticipantKeyReference,
        ParticipantKeyReference,
        OwnerDekEnvelope,
        ProtectedPayloadCiphertext,
        [u8; 32],
        P256SigningKey,
    );

    fn state_with_object() -> Issue10State {
        let bootstrap_key = SigningKey::from_bytes(&[0x11; 32]);
        let owner_authorization = SigningKey::from_bytes(&[0x12; 32]);
        let owner_wrapping = P256SigningKey::from_slice(&[0x13; 32]).expect("owner scalar");
        let grantee_authorization = SigningKey::from_bytes(&[0x14; 32]);
        let grantee_wrapping = P256SigningKey::from_slice(&[0x15; 32]).expect("grantee scalar");
        let owner = Uuid::from_u128(0x10_01);
        let grantee = Uuid::from_u128(0x10_02);
        let owner_authorization_reference = ParticipantKeyReference::for_authorization(
            owner,
            1,
            owner_authorization.verifying_key().to_bytes(),
        )
        .expect("owner authorization reference");
        let owner_wrapping_reference =
            ParticipantKeyReference::for_wrapping(owner, 1, p256_public(&owner_wrapping))
                .expect("owner wrapping reference");
        let grantee_authorization_reference = ParticipantKeyReference::for_authorization(
            grantee,
            1,
            grantee_authorization.verifying_key().to_bytes(),
        )
        .expect("grantee authorization reference");
        let grantee_wrapping_reference =
            ParticipantKeyReference::for_wrapping(grantee, 1, p256_public(&grantee_wrapping))
                .expect("grantee wrapping reference");
        let profile_hash = [0xA1; 32];
        let profile =
            PrivacyLifecycleProfile::new(profile_hash, bootstrap_key.verifying_key().to_bytes())
                .expect("privacy profile");
        let anchor = PrivacyAdmissionAnchor::genesis(NETWORK_ID, PROFILE_ID, profile_hash, 0, 1)
            .expect("genesis anchor");
        let payload =
            ProtectedPayloadCiphertext::new([0x31; 32], vec![0x41; 17]).expect("opaque payload");
        let encrypted_payload_commitment = encrypted_payload_commitment(&payload);
        let object_id =
            Uuid::from_bytes([0x10, 0, 0, 0, 0, 0, 0x40, 0x03, 0x80, 0, 0, 0, 0, 0, 0, 3]);
        let object_context = [0x32; 32];
        let owner_header = OwnerDekHeader::from_parts(
            &anchor,
            object_id,
            owner,
            owner_wrapping_reference.clone(),
            payload.protected_content_commitment,
            encrypted_payload_commitment,
        );
        let owner_envelope = OwnerDekEnvelope::new(
            object_context,
            owner_header,
            p256_public(&owner_wrapping),
            [0x42; 48],
        )
        .expect("opaque owner envelope");

        let mut state = EffectivePrivacyState::new();
        state.principals.extend([owner, grantee]);
        state.keys.insert(
            owner_authorization_reference.clone(),
            ParticipantKeyRecord {
                reference: owner_authorization_reference.clone(),
                possession_scheme: PossessionProofScheme::Ed25519Raw64,
                public_key: owner_authorization.verifying_key().to_bytes().to_vec(),
                status: KeyLifecycleStatus::Active,
                effective_ledger_position: 0,
            },
        );
        state.keys.insert(
            owner_wrapping_reference.clone(),
            ParticipantKeyRecord {
                reference: owner_wrapping_reference.clone(),
                possession_scheme: PossessionProofScheme::EcdsaP256Sha256RawLowS64,
                public_key: p256_public(&owner_wrapping).to_vec(),
                status: KeyLifecycleStatus::Active,
                effective_ledger_position: 0,
            },
        );
        state.keys.insert(
            grantee_authorization_reference.clone(),
            ParticipantKeyRecord {
                reference: grantee_authorization_reference.clone(),
                possession_scheme: PossessionProofScheme::Ed25519Raw64,
                public_key: grantee_authorization.verifying_key().to_bytes().to_vec(),
                status: KeyLifecycleStatus::Active,
                effective_ledger_position: 0,
            },
        );
        state.keys.insert(
            grantee_wrapping_reference.clone(),
            ParticipantKeyRecord {
                reference: grantee_wrapping_reference.clone(),
                possession_scheme: PossessionProofScheme::EcdsaP256Sha256RawLowS64,
                public_key: p256_public(&grantee_wrapping).to_vec(),
                status: KeyLifecycleStatus::Active,
                effective_ledger_position: 0,
            },
        );
        state.protected_objects.insert(
            object_id,
            ProtectedObjectRecord {
                object_id,
                owner,
                authorization_key: owner_authorization_reference.clone(),
                wrapping_key: owner_wrapping_reference.clone(),
                object_context,
                payload: payload.clone(),
                encrypted_payload_commitment,
                owner_envelope: owner_envelope.clone(),
                effective_ledger_position: 0,
            },
        );
        (
            profile,
            state,
            anchor,
            object_id,
            grantee,
            owner_authorization,
            owner_authorization_reference,
            owner_wrapping_reference,
            grantee_authorization_reference,
            grantee_wrapping_reference,
            owner_envelope,
            payload,
            encrypted_payload_commitment,
            grantee_wrapping,
        )
    }

    #[test]
    fn grant_revoke_is_terminal_and_regrant_has_fresh_identity() {
        let (
            profile,
            state,
            anchor,
            object_id,
            grantee,
            owner_authorization,
            owner_authorization_reference,
            _owner_wrapping_reference,
            _grantee_authorization_reference,
            grantee_wrapping_reference,
            _owner_envelope,
            payload,
            encrypted_payload_commitment,
            grantee_wrapping,
        ) = state_with_object();
        let delivery = grant_delivery(
            &anchor,
            object_id,
            Uuid::from_u128(0x10_01),
            grantee,
            grantee_wrapping_reference.clone(),
            payload.protected_content_commitment,
            encrypted_payload_commitment,
            &grantee_wrapping,
        );
        let unsigned = UnsignedPrivacyTransition::grant_access(
            &profile,
            anchor.clone(),
            object_id,
            Uuid::from_u128(0x10_01),
            grantee,
            PrivacyPermission::ReadProtectedObjectV1,
            delivery.clone(),
            owner_authorization_reference.clone(),
        )
        .expect("grant core");
        let grant = unsigned
            .clone()
            .complete_grant_access(
                owner_authorization
                    .sign(&unsigned.authorization_digest())
                    .to_bytes(),
            )
            .expect("grant transition");
        assert_eq!(
            PrivacyControlTransition::decode(&grant.canonical_bytes()),
            Ok(grant.clone())
        );
        let granted = state
            .stage_transition(&profile, &anchor, &grant)
            .expect("stage grant");
        let grant_id = grant.privacy_grant_id().expect("grant id");
        assert_eq!(
            granted
                .active_privacy_grant(object_id, grantee)
                .unwrap()
                .grant_id(),
            grant_id
        );

        let duplicate_anchor = PrivacyAdmissionAnchor::new(
            NETWORK_ID,
            PROFILE_ID,
            profile.network_profile_content_hash(),
            1,
            2,
            [0x61; 32],
            Some([0x62; 32]),
        )
        .expect("duplicate anchor");
        let duplicate_delivery = grant_delivery(
            &duplicate_anchor,
            object_id,
            Uuid::from_u128(0x10_01),
            grantee,
            grantee_wrapping_reference.clone(),
            payload.protected_content_commitment,
            encrypted_payload_commitment,
            &grantee_wrapping,
        );
        let duplicate_unsigned = UnsignedPrivacyTransition::grant_access(
            &profile,
            duplicate_anchor.clone(),
            object_id,
            Uuid::from_u128(0x10_01),
            grantee,
            PrivacyPermission::ReadProtectedObjectV1,
            duplicate_delivery,
            owner_authorization_reference.clone(),
        )
        .expect("duplicate grant core");
        let duplicate = duplicate_unsigned
            .clone()
            .complete_grant_access(
                owner_authorization
                    .sign(&duplicate_unsigned.authorization_digest())
                    .to_bytes(),
            )
            .expect("duplicate grant transition");
        let before_duplicate = granted.digest();
        assert!(granted
            .stage_transition(&profile, &duplicate_anchor, &duplicate)
            .is_err());
        assert_eq!(granted.digest(), before_duplicate);

        let revoke_anchor = duplicate_anchor;
        let revoke_unsigned = UnsignedPrivacyTransition::revoke_grant(
            &profile,
            revoke_anchor.clone(),
            grant_id,
            object_id,
            grantee,
            Uuid::from_u128(0x10_01),
            PrivacyPermission::ReadProtectedObjectV1,
            anchor.expected_ledger_position(),
            owner_authorization_reference.clone(),
        )
        .expect("revoke core");
        let revoke = revoke_unsigned
            .clone()
            .complete_grant_revocation(
                owner_authorization
                    .sign(&revoke_unsigned.authorization_digest())
                    .to_bytes(),
            )
            .expect("revoke transition");
        let revoked = granted
            .stage_transition(&profile, &revoke_anchor, &revoke)
            .expect("stage terminal revoke");
        assert_eq!(
            revoked.privacy_grant(&grant_id).unwrap().status(),
            PrivacyGrantStatus::Revoked
        );
        assert!(revoked.active_privacy_grant(object_id, grantee).is_none());
        let before_double_revoke = revoked.digest();
        assert!(revoked
            .stage_transition(&profile, &revoke_anchor, &revoke)
            .is_err());
        assert_eq!(revoked.digest(), before_double_revoke);

        let regrant_anchor = PrivacyAdmissionAnchor::new(
            NETWORK_ID,
            PROFILE_ID,
            profile.network_profile_content_hash(),
            2,
            3,
            [0x71; 32],
            Some([0x72; 32]),
        )
        .expect("regrant anchor");
        let regrant_delivery = grant_delivery(
            &regrant_anchor,
            object_id,
            Uuid::from_u128(0x10_01),
            grantee,
            grantee_wrapping_reference,
            payload.protected_content_commitment,
            encrypted_payload_commitment,
            &grantee_wrapping,
        );
        let regrant_unsigned = UnsignedPrivacyTransition::grant_access(
            &profile,
            regrant_anchor.clone(),
            object_id,
            Uuid::from_u128(0x10_01),
            grantee,
            PrivacyPermission::ReadProtectedObjectV1,
            regrant_delivery,
            owner_authorization_reference,
        )
        .expect("fresh regrant core");
        let regrant = regrant_unsigned
            .clone()
            .complete_grant_access(
                owner_authorization
                    .sign(&regrant_unsigned.authorization_digest())
                    .to_bytes(),
            )
            .expect("fresh regrant transition");
        let regranted = revoked
            .stage_transition(&profile, &regrant_anchor, &regrant)
            .expect("stage fresh regrant");
        assert_ne!(regrant.privacy_grant_id(), Some(grant_id));
        assert_eq!(regranted.privacy_grants_for_object(object_id).len(), 2);
        assert_eq!(
            regranted
                .active_privacy_grant(object_id, grantee)
                .unwrap()
                .status(),
            PrivacyGrantStatus::Active
        );
    }

    #[test]
    fn live_release_response_is_opaque_and_canonical() {
        let (
            profile,
            _state,
            anchor,
            object_id,
            _grantee,
            _owner_authorization,
            _owner_authorization_reference,
            owner_wrapping_reference,
            _grantee_authorization_reference,
            _grantee_wrapping_reference,
            owner_envelope,
            payload,
            encrypted_payload_commitment,
            _grantee_wrapping,
        ) = state_with_object();
        let evidence = LivePrivacyReleaseEvidence::owner(
            NETWORK_ID,
            PROFILE_ID,
            profile.network_profile_content_hash(),
            7,
            [0x81; 32],
            object_id,
            Uuid::from_u128(0x10_01),
            owner_wrapping_reference,
        )
        .expect("owner release evidence");
        let response = LivePrivacyReleaseResponse::new(
            evidence,
            payload,
            PrivacyReleaseEnvelope::Owner(owner_envelope),
            encrypted_payload_commitment,
        )
        .expect("opaque release response");
        assert_eq!(
            LivePrivacyReleaseResponse::decode(&response.canonical_bytes())
                .expect("decode response"),
            response
        );
        assert!(!response
            .canonical_bytes()
            .windows(b"plaintext-that-must-not-escape".len())
            .any(|window| window == b"plaintext-that-must-not-escape"));
        let _ = anchor;
    }
}
