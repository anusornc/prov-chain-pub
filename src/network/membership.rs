//! Governance-authenticated membership for the bounded reference-node path.
//!
//! A signed [`MembershipManifest`] is the sole source of peer membership and
//! validator role authorization. Bootstrap addresses, semantic compatibility,
//! and an authenticated transport do not grant a ledger-signing role.

use std::collections::BTreeSet;

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::network::canonical::{hash_domain_separated_parts, write_len_prefixed_text};
use crate::network::profile::{MembershipProfile, NetworkProfile};

const MANIFEST_FORMAT_VERSION: u16 = 1;
const MAX_MANIFEST_TEXT_BYTES: usize = 4_096;
const MAX_MEMBERS: usize = 1_024;
const MANIFEST_MAGIC: &[u8] = b"PROVCHAIN_MEMBERSHIP_MANIFEST_V1";
const SIGNED_MANIFEST_MAGIC: &[u8] = b"PROVCHAIN_SIGNED_MEMBERSHIP_MANIFEST_V1";
const MANIFEST_DIGEST_DOMAIN: &[u8] = b"provchain/membership-manifest-digest/v1";
const MANIFEST_SIGNATURE_DOMAIN: &[u8] = b"provchain/membership-manifest-signature/v1";

/// A role granted to one manifest member.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
pub enum MemberRole {
    /// Permission to establish an authenticated peer session.
    Peer = 1,
    /// Permission to be considered for validator Signer Authorization.
    Validator = 2,
}

/// Prospective membership status in one manifest version.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
pub enum MemberStatus {
    /// The member may exercise the roles declared in this manifest version.
    Active = 1,
    /// The member remains auditable but cannot start or authenticate a session.
    Removed = 2,
}

/// One logical node and its role-separated verification keys.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkMember {
    /// Stable logical node identity.
    pub node_id: Uuid,
    /// Dedicated Node Identity Key used for peer authentication and node-level
    /// commit receipts, never for PoA proposal signing.
    pub identity_public_key: [u8; 32],
    /// Roles granted by this manifest version.
    pub roles: Vec<MemberRole>,
    /// Whether the role grants are active for prospective participation.
    pub status: MemberStatus,
    /// Separate PoA proposal verification key, required for validators.
    pub validator_public_key: Option<[u8; 32]>,
}

/// Canonical governance-controlled membership content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MembershipManifest {
    /// Stable manifest lineage identifier.
    pub manifest_id: String,
    /// Selected manifest version.
    pub version: u64,
    /// Network to which these grants apply.
    pub network_id: String,
    /// Exact Network Profile identity to which these grants apply.
    pub network_profile_id: String,
    /// Member records; canonical encoding sorts them by logical node identity.
    pub members: Vec<NetworkMember>,
}

/// A manifest plus its governance-root signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedMembershipManifest {
    /// Exact canonical membership content.
    pub manifest: MembershipManifest,
    /// Ed25519 signature by the locally trusted governance root.
    pub governance_signature: [u8; 64],
}

/// Local identity material presented while activating one reference node.
pub struct NodeIdentity {
    node_id: Uuid,
    expected_role: MemberRole,
    signing_key: SigningKey,
}

impl NodeIdentity {
    /// Bind a local Node Identity Key to its expected manifest identity and role.
    /// This key authenticates peer sessions and node-level commit receipts; its
    /// role remains separate from the validator proposal key.
    pub fn new(node_id: Uuid, expected_role: MemberRole, signing_key: SigningKey) -> Self {
        Self {
            node_id,
            expected_role,
            signing_key,
        }
    }
}

/// A verified manifest and local identity, ready to authenticate peers.
pub struct ActiveMembership {
    signed_manifest: SignedMembershipManifest,
    local_member: NetworkMember,
    local_role: MemberRole,
    identity_key: SigningKey,
    authorized_validator_keys: BTreeSet<[u8; 32]>,
}

/// Evidence that a separately authenticated PoA key has an active validator role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignerAuthorization {
    node_id: Uuid,
    validator_public_key: [u8; 32],
    manifest_digest: [u8; 32],
}

impl SignerAuthorization {
    /// Manifest-bound logical validator identity.
    pub fn node_id(&self) -> Uuid {
        self.node_id
    }

    /// Separate proposal key authorized for the validator role.
    pub fn validator_public_key(&self) -> [u8; 32] {
        self.validator_public_key
    }

    /// Exact manifest under which this role was authorized.
    pub fn manifest_digest(&self) -> [u8; 32] {
        self.manifest_digest
    }
}

impl MembershipManifest {
    /// Validate and sign this exact manifest with the Governance Trust Root.
    pub fn sign(
        self,
        governance_key: &SigningKey,
    ) -> Result<SignedMembershipManifest, MembershipError> {
        self.validate()?;
        let digest = manifest_signature_digest(&self);
        Ok(SignedMembershipManifest {
            manifest: self,
            governance_signature: governance_key.sign(&digest).to_bytes(),
        })
    }

    fn validate(&self) -> Result<(), MembershipError> {
        validate_text(&self.manifest_id, "manifest_id")?;
        validate_text(&self.network_id, "network_id")?;
        validate_text(&self.network_profile_id, "network_profile_id")?;
        if self.version == 0 {
            return Err(MembershipError::InvalidManifest(
                "manifest version must be greater than 0".to_string(),
            ));
        }
        if self.members.is_empty() || self.members.len() > MAX_MEMBERS {
            return Err(MembershipError::InvalidManifest(format!(
                "manifest must contain between 1 and {MAX_MEMBERS} members"
            )));
        }

        let mut node_ids = BTreeSet::new();
        let mut identity_keys = BTreeSet::new();
        let mut validator_keys = BTreeSet::new();
        for member in &self.members {
            if !node_ids.insert(member.node_id) {
                return Err(MembershipError::InvalidManifest(format!(
                    "duplicate node identity {}",
                    member.node_id
                )));
            }
            VerifyingKey::from_bytes(&member.identity_public_key).map_err(|_| {
                MembershipError::InvalidManifest(format!(
                    "node {} has an invalid identity verification key",
                    member.node_id
                ))
            })?;
            if !identity_keys.insert(member.identity_public_key) {
                return Err(MembershipError::InvalidManifest(
                    "Node Identity Keys must be unique".to_string(),
                ));
            }

            let roles: BTreeSet<_> = member.roles.iter().copied().collect();
            if roles.len() != member.roles.len() || !roles.contains(&MemberRole::Peer) {
                return Err(MembershipError::InvalidManifest(format!(
                    "node {} must have unique roles including peer",
                    member.node_id
                )));
            }
            if roles.contains(&MemberRole::Validator) {
                let validator_key = member.validator_public_key.ok_or_else(|| {
                    MembershipError::InvalidManifest(format!(
                        "validator {} is missing its separate proposal key",
                        member.node_id
                    ))
                })?;
                VerifyingKey::from_bytes(&validator_key).map_err(|_| {
                    MembershipError::InvalidManifest(format!(
                        "validator {} has an invalid proposal key",
                        member.node_id
                    ))
                })?;
                if validator_key == member.identity_public_key {
                    return Err(MembershipError::InvalidManifest(format!(
                        "node {} reuses its Node Identity Key as validator key",
                        member.node_id
                    )));
                }
                if !validator_keys.insert(validator_key) {
                    return Err(MembershipError::InvalidManifest(
                        "validator proposal keys must be unique".to_string(),
                    ));
                }
            } else if member.validator_public_key.is_some() {
                return Err(MembershipError::InvalidManifest(format!(
                    "non-validator {} carries an ungranted validator key",
                    member.node_id
                )));
            }
        }
        if !identity_keys.is_disjoint(&validator_keys) {
            return Err(MembershipError::InvalidManifest(
                "Node Identity Keys and validator proposal keys must be role-separated".to_string(),
            ));
        }
        Ok(())
    }

    fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MANIFEST_MAGIC);
        bytes.extend_from_slice(&MANIFEST_FORMAT_VERSION.to_be_bytes());
        write_len_prefixed_text(&mut bytes, &self.manifest_id);
        bytes.extend_from_slice(&self.version.to_be_bytes());
        write_len_prefixed_text(&mut bytes, &self.network_id);
        write_len_prefixed_text(&mut bytes, &self.network_profile_id);

        let mut members: Vec<_> = self.members.iter().collect();
        members.sort_by_key(|member| *member.node_id.as_bytes());
        bytes.extend_from_slice(&(members.len() as u64).to_be_bytes());
        for member in members {
            bytes.extend_from_slice(member.node_id.as_bytes());
            bytes.extend_from_slice(&member.identity_public_key);
            bytes.push(member.status as u8);
            let mut roles = member.roles.clone();
            roles.sort_unstable();
            bytes.extend_from_slice(&(roles.len() as u64).to_be_bytes());
            for role in roles {
                bytes.push(role as u8);
            }
            match member.validator_public_key {
                Some(key) => {
                    bytes.push(1);
                    bytes.extend_from_slice(&key);
                }
                None => bytes.push(0),
            }
        }
        bytes
    }
}

impl SignedMembershipManifest {
    /// Encode the exact canonical signed manifest artifact carried by bridge proofs.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let manifest = self.manifest.canonical_bytes();
        let mut bytes = Vec::with_capacity(
            SIGNED_MANIFEST_MAGIC.len() + 4 + manifest.len() + self.governance_signature.len(),
        );
        bytes.extend_from_slice(SIGNED_MANIFEST_MAGIC);
        bytes.extend_from_slice(&(manifest.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&manifest);
        bytes.extend_from_slice(&self.governance_signature);
        bytes
    }

    /// Return the profile binding for this exact canonical manifest.
    pub fn binding(&self) -> MembershipProfile {
        MembershipProfile {
            manifest_id: self.manifest.manifest_id.clone(),
            manifest_version: self.manifest.version,
            manifest_digest: hex::encode(self.digest()),
        }
    }

    /// Return the SHA-256 identity of the exact canonical manifest content.
    pub fn digest(&self) -> [u8; 32] {
        manifest_digest(&self.manifest)
    }

    /// Validate canonical manifest structure without treating its signature as trusted.
    pub(crate) fn validate_structure(&self) -> Result<(), MembershipError> {
        self.manifest.validate()
    }

    pub(crate) fn verify(&self, governance_root: &VerifyingKey) -> Result<(), MembershipError> {
        self.manifest.validate()?;
        let signature = Signature::from_bytes(&self.governance_signature);
        governance_root
            .verify_strict(&manifest_signature_digest(&self.manifest), &signature)
            .map_err(|_| MembershipError::InvalidGovernanceSignature)
    }
}

impl ActiveMembership {
    /// Authenticate and activate one local identity under the profile-bound manifest.
    pub fn activate(
        profile: &NetworkProfile,
        signed_manifest: SignedMembershipManifest,
        governance_root: VerifyingKey,
        identity: NodeIdentity,
    ) -> Result<Self, MembershipError> {
        profile
            .validate()
            .map_err(|error| MembershipError::InvalidProfile(error.to_string()))?;
        signed_manifest.verify(&governance_root)?;

        let governance_public_key = governance_root.to_bytes();
        if let Some(member) = signed_manifest.manifest.members.iter().find(|member| {
            member.identity_public_key == governance_public_key
                || member.validator_public_key == Some(governance_public_key)
        }) {
            return Err(MembershipError::GovernanceKeyReused(member.node_id));
        }

        let binding = profile
            .membership
            .as_ref()
            .ok_or(MembershipError::MissingManifestBinding)?;
        let manifest = &signed_manifest.manifest;
        if manifest.network_id != profile.network_id {
            return Err(MembershipError::WrongNetwork);
        }
        if manifest.network_profile_id != profile.profile_id {
            return Err(MembershipError::WrongNetworkProfile);
        }
        if manifest.manifest_id != binding.manifest_id {
            return Err(MembershipError::WrongManifestIdentity);
        }
        if manifest.version != binding.manifest_version {
            return Err(MembershipError::WrongManifestVersion);
        }
        if signed_manifest.digest_hex() != binding.manifest_digest {
            return Err(MembershipError::WrongManifestDigest);
        }

        let local_member = manifest
            .members
            .iter()
            .find(|member| member.node_id == identity.node_id)
            .cloned()
            .ok_or(MembershipError::UnknownLocalIdentity(identity.node_id))?;
        if local_member.status != MemberStatus::Active {
            return Err(MembershipError::RemovedLocalIdentity(identity.node_id));
        }
        if !local_member.roles.contains(&MemberRole::Peer)
            || !local_member.roles.contains(&identity.expected_role)
        {
            return Err(MembershipError::InactiveLocalRole {
                node_id: identity.node_id,
                role: identity.expected_role,
            });
        }
        if local_member.identity_public_key != identity.signing_key.verifying_key().to_bytes() {
            return Err(MembershipError::LocalIdentityKeyMismatch(identity.node_id));
        }
        if identity.expected_role == MemberRole::Validator {
            let validator_key =
                local_member
                    .validator_public_key
                    .ok_or(MembershipError::InactiveLocalRole {
                        node_id: identity.node_id,
                        role: MemberRole::Validator,
                    })?;
            let validator_hex = hex::encode(validator_key);
            if !profile.consensus.authority_keys.contains(&validator_hex) {
                return Err(MembershipError::InactiveLocalRole {
                    node_id: identity.node_id,
                    role: MemberRole::Validator,
                });
            }
        }
        let authorized_validator_keys = validate_profile_authorities(profile, manifest)?;

        Ok(Self {
            signed_manifest,
            local_member,
            local_role: identity.expected_role,
            identity_key: identity.signing_key,
            authorized_validator_keys,
        })
    }

    /// Local logical node identity authenticated during startup.
    pub fn local_node_id(&self) -> Uuid {
        self.local_member.node_id
    }

    /// Local role selected and validated during startup.
    pub fn local_role(&self) -> MemberRole {
        self.local_role
    }

    /// Digest of the profile-bound active manifest.
    pub fn manifest_digest(&self) -> [u8; 32] {
        self.signed_manifest.digest()
    }

    /// Exact governance-verified manifest backing peer authentication.
    pub(crate) fn signed_manifest(&self) -> &SignedMembershipManifest {
        &self.signed_manifest
    }

    /// Dedicated local Node Identity Key used by the handshake engine.
    pub(crate) fn identity_key(&self) -> &SigningKey {
        &self.identity_key
    }

    /// Resolve a peer's separately presented PoA key under the active role grants.
    pub(crate) fn authorize_validator_signer(
        &self,
        node_id: Uuid,
        presented_key: [u8; 32],
    ) -> Result<SignerAuthorization, SignerAuthorizationError> {
        let member = self
            .signed_manifest
            .manifest
            .members
            .iter()
            .find(|member| member.node_id == node_id)
            .ok_or(SignerAuthorizationError::UnknownMember(node_id))?;
        if member.status != MemberStatus::Active {
            return Err(SignerAuthorizationError::RemovedMember(node_id));
        }
        if !member.roles.contains(&MemberRole::Validator) {
            return Err(SignerAuthorizationError::ValidatorRoleInactive(node_id));
        }
        let expected_key = member
            .validator_public_key
            .ok_or(SignerAuthorizationError::ValidatorRoleInactive(node_id))?;
        if expected_key != presented_key {
            return Err(SignerAuthorizationError::ValidatorKeyMismatch(node_id));
        }
        if !self.authorized_validator_keys.contains(&presented_key) {
            return Err(SignerAuthorizationError::ValidatorNotInProfile(node_id));
        }
        Ok(SignerAuthorization {
            node_id,
            validator_public_key: presented_key,
            manifest_digest: self.manifest_digest(),
        })
    }
}

impl SignedMembershipManifest {
    fn digest_hex(&self) -> String {
        hex::encode(self.digest())
    }
}

pub(crate) fn validate_profile_authorities(
    profile: &NetworkProfile,
    manifest: &MembershipManifest,
) -> Result<BTreeSet<[u8; 32]>, MembershipError> {
    let active_validator_keys: BTreeSet<_> = manifest
        .members
        .iter()
        .filter(|member| {
            member.status == MemberStatus::Active && member.roles.contains(&MemberRole::Validator)
        })
        .filter_map(|member| member.validator_public_key)
        .collect();

    let mut configured_keys = BTreeSet::new();
    for encoded_key in &profile.consensus.authority_keys {
        let decoded = hex::decode(encoded_key).map_err(|_| {
            MembershipError::InvalidProfile("authority key is not valid hex".to_string())
        })?;
        let key: [u8; 32] = decoded.try_into().map_err(|_| {
            MembershipError::InvalidProfile("authority key must contain 32 bytes".to_string())
        })?;
        if !active_validator_keys.contains(&key) {
            return Err(MembershipError::UnresolvedValidatorKey(encoded_key.clone()));
        }
        if !configured_keys.insert(key) {
            return Err(MembershipError::InvalidProfile(
                "authority keys must be unique".to_string(),
            ));
        }
    }
    Ok(configured_keys)
}

fn validate_text(value: &str, field: &str) -> Result<(), MembershipError> {
    if value.trim().is_empty() || value.len() > MAX_MANIFEST_TEXT_BYTES {
        return Err(MembershipError::InvalidManifest(format!(
            "{field} must contain between 1 and {MAX_MANIFEST_TEXT_BYTES} bytes"
        )));
    }
    Ok(())
}

fn manifest_digest(manifest: &MembershipManifest) -> [u8; 32] {
    hash_domain_separated_parts(MANIFEST_DIGEST_DOMAIN, &[&manifest.canonical_bytes()])
}

fn manifest_signature_digest(manifest: &MembershipManifest) -> [u8; 32] {
    hash_domain_separated_parts(MANIFEST_SIGNATURE_DOMAIN, &[&manifest_digest(manifest)])
}

/// Fail-closed membership activation errors safe to expose to node operators.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MembershipError {
    /// The Network Profile is malformed or unsupported.
    #[error("invalid Network Profile: {0}")]
    InvalidProfile(String),
    /// The canonical manifest is malformed or internally inconsistent.
    #[error("invalid Membership Manifest: {0}")]
    InvalidManifest(String),
    /// The governance signature does not verify under the local trust root.
    #[error("Membership Manifest governance signature is invalid")]
    InvalidGovernanceSignature,
    /// Reference startup requires an exact profile binding.
    #[error("Network Profile has no Membership Manifest binding")]
    MissingManifestBinding,
    /// The manifest targets a different network.
    #[error("Membership Manifest targets a different network")]
    WrongNetwork,
    /// The manifest targets a different Network Profile.
    #[error("Membership Manifest targets a different Network Profile")]
    WrongNetworkProfile,
    /// The bound stable manifest identity differs.
    #[error("Membership Manifest identity does not match the Network Profile")]
    WrongManifestIdentity,
    /// The bound manifest version differs.
    #[error("Membership Manifest version does not match the Network Profile")]
    WrongManifestVersion,
    /// The exact canonical manifest content differs from the profile binding.
    #[error("Membership Manifest digest does not match the Network Profile")]
    WrongManifestDigest,
    /// The local logical node is absent from the active manifest.
    #[error("local node {0} is unknown to the Membership Manifest")]
    UnknownLocalIdentity(Uuid),
    /// The local logical node has been removed in this version.
    #[error("local node {0} is removed in the Membership Manifest")]
    RemovedLocalIdentity(Uuid),
    /// The dedicated local Node Identity Key does not match the manifest.
    #[error("local node {0} has the wrong Node Identity Key")]
    LocalIdentityKeyMismatch(Uuid),
    /// Governance trust and node/validator identities are separate key roles.
    #[error("member {0} reuses the Governance Trust Root for another key role")]
    GovernanceKeyReused(Uuid),
    /// The requested local role is absent or inactive.
    #[error("local node {node_id} is not active for role {role:?}")]
    InactiveLocalRole {
        /// Logical node whose role was checked.
        node_id: Uuid,
        /// Requested role that was not active.
        role: MemberRole,
    },
    /// A configured authority does not resolve to an active manifest validator.
    #[error("validator key {0} does not resolve to an active manifest validator")]
    UnresolvedValidatorKey(String),
}

/// Role checks intentionally separate from peer-session identity proof.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SignerAuthorizationError {
    /// Signer checks require an already authenticated transport.
    #[error("peer transport has no authenticated session")]
    UnauthenticatedTransport,
    /// The supplied transport belongs to another local node.
    #[error("authenticated transport belongs to a different local node")]
    TransportLocalIdentityMismatch,
    /// The supplied transport was authenticated under another manifest.
    #[error("authenticated transport belongs to a different Membership Manifest")]
    TransportManifestMismatch,
    /// The session peer is absent from the active manifest.
    #[error("member {0} is unknown to the active Membership Manifest")]
    UnknownMember(Uuid),
    /// The session peer is removed in the active manifest.
    #[error("member {0} is removed in the active Membership Manifest")]
    RemovedMember(Uuid),
    /// Peer authentication succeeded, but the validator role is not active.
    #[error("validator role is inactive for member {0}")]
    ValidatorRoleInactive(Uuid),
    /// The presented proposal key differs from the member's separate PoA key.
    #[error("validator proposal key does not match member {0}")]
    ValidatorKeyMismatch(Uuid),
    /// The validator is not selected by the Network Profile authority set.
    #[error("validator {0} is not in the Network Profile authority set")]
    ValidatorNotInProfile(Uuid),
}
