//! Fresh mutual Node Identity proof for reference-node peer sessions.
//!
//! The bounded protocol authenticates membership-key possession at session
//! establishment. It deliberately does not claim transport confidentiality or
//! per-frame integrity after the handshake.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use rand::RngCore;
use thiserror::Error;
use uuid::Uuid;

use crate::network::canonical::{hash_domain_separated_parts, write_len_prefixed_text};
use crate::network::membership::{ActiveMembership, MemberRole, MemberStatus, NetworkMember};
use crate::network::profile::NetworkProfile;

/// Bounded lifetime of one challenge-response exchange.
pub const PEER_HANDSHAKE_TIMEOUT_MILLIS: u64 = 10_000;

/// Global live-handshake cap for one activated reference node.
pub const MAX_PENDING_PEER_HANDSHAKES: usize = 128;

/// Global replay-window cap for one activated reference node.
pub const MAX_PEER_REPLAY_RECORDS: usize = 1_024;

const PEER_SESSION_PROTOCOL_VERSION: u16 = 1;
const HANDSHAKE_CONTEXT_MAGIC: &[u8] = b"PROVCHAIN_PEER_SESSION_CONTEXT_V1";
const HANDSHAKE_HELLO_MAGIC: &[u8] = b"PROVCHAIN_PEER_SESSION_HELLO_V1";
const HELLO_DIGEST_DOMAIN: &[u8] = b"provchain/peer-session-hello/v1";
const TRANSCRIPT_DIGEST_DOMAIN: &[u8] = b"provchain/peer-session-transcript/v1";

/// Unforgeable local binding for one accepted or opened transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PeerTransportId([u8; 32]);

impl PeerTransportId {
    fn random() -> Self {
        let mut value = [0u8; 32];
        OsRng.fill_bytes(&mut value);
        Self(value)
    }

    fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Exact active contract bound into both peers' identity proofs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerSessionContext {
    /// Closed handshake protocol version.
    pub protocol_version: u16,
    /// Permissioned network identity.
    pub network_id: String,
    /// Exact active Network Profile identity.
    pub network_profile_id: String,
    /// Stable Membership Manifest identity.
    pub manifest_id: String,
    /// Active Membership Manifest version.
    pub manifest_version: u64,
    /// Canonical digest of the exact active manifest content.
    pub manifest_digest: [u8; 32],
}

/// Initiator contribution to one fresh peer-session transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerHandshakeHello {
    /// Exact contract expected by both endpoints.
    pub context: PeerSessionContext,
    /// Manifest-bound initiator identity.
    pub initiator_node_id: Uuid,
    /// Intended manifest-bound responder identity.
    pub responder_node_id: Uuid,
    /// Initiator-local transport capability bound into both proofs.
    pub initiator_transport_id: PeerTransportId,
    /// Trusted local creation time in Unix milliseconds.
    pub issued_at_millis: u64,
    /// Fresh initiator CSPRNG nonce.
    pub initiator_nonce: [u8; 32],
}

/// Responder nonce and proof over the mutually bound transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerHandshakeChallenge {
    /// Exact hello accepted by the responder.
    pub hello: PeerHandshakeHello,
    /// Responder-local transport capability bound into both proofs.
    pub responder_transport_id: PeerTransportId,
    /// Fresh responder CSPRNG nonce.
    pub responder_nonce: [u8; 32],
    /// Responder Node Identity signature over the transcript digest.
    pub responder_proof: [u8; 64],
}

/// Initiator proof returned after authenticating the responder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerHandshakeResponse {
    /// Digest of the exact transcript both endpoints signed.
    pub transcript_digest: [u8; 32],
    /// Initiator Node Identity signature over the transcript digest.
    pub initiator_proof: [u8; 64],
}

/// A local association bound to one verified remote manifest identity.
#[derive(Debug, PartialEq, Eq)]
pub struct AuthenticatedPeerSession {
    local_node_id: Uuid,
    authenticated_peer_id: Uuid,
    local_transport_id: PeerTransportId,
    context: PeerSessionContext,
    transcript_digest: [u8; 32],
    established_at_millis: u64,
}

/// Frame class used by the pre-authentication quarantine boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerMessageClass {
    /// One of the bounded peer-session handshake frames.
    Handshake,
    /// A protocol error frame that cannot mutate ledger state.
    Error,
    /// A connection-close frame.
    Close,
    /// A consensus message reserved for an authenticated session.
    Consensus,
    /// A ledger-admission message reserved for an authenticated session.
    Admission,
    /// A ledger-synchronization message reserved for an authenticated session.
    Synchronization,
}

/// Authentication state of one reference-node transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerTransportState {
    /// Only handshake, error, and close behavior is eligible.
    Quarantined,
    /// A fresh manifest-bound peer session has been attached.
    Authenticated,
    /// The transport is terminal and no further frame is eligible.
    Closed,
}

/// Fail-closed message gate for a newly accepted transport.
pub struct PeerTransport {
    transport_id: PeerTransportId,
    local_node_id: Uuid,
    manifest_digest: [u8; 32],
    state: PeerTransportState,
    authenticated_peer_id: Option<Uuid>,
    registry: PeerTransportRegistry,
}

/// Node-local registry preventing one peer identity from owning two live transports.
#[derive(Clone, Default)]
pub(crate) struct PeerTransportRegistry {
    active_bindings: Arc<Mutex<BTreeMap<Uuid, PeerTransportId>>>,
}

impl PeerTransportRegistry {
    fn claim(&self, peer_id: Uuid, transport_id: PeerTransportId) -> bool {
        let mut bindings = self
            .active_bindings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if bindings.contains_key(&peer_id) {
            return false;
        }
        bindings.insert(peer_id, transport_id);
        true
    }

    fn release(&self, peer_id: Uuid, transport_id: PeerTransportId) {
        let mut bindings = self
            .active_bindings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if bindings.get(&peer_id) == Some(&transport_id) {
            bindings.remove(&peer_id);
        }
    }
}

impl PeerTransport {
    /// Create a unique transport capability in the fail-closed quarantine state.
    pub(crate) fn quarantined(
        local_node_id: Uuid,
        manifest_digest: [u8; 32],
        registry: PeerTransportRegistry,
    ) -> Self {
        Self {
            transport_id: PeerTransportId::random(),
            local_node_id,
            manifest_digest,
            state: PeerTransportState::Quarantined,
            authenticated_peer_id: None,
            registry,
        }
    }

    /// Attach the only capability that can leave quarantine: a verified session.
    pub fn bind_authenticated_session(
        &mut self,
        session: AuthenticatedPeerSession,
    ) -> Result<(), PeerSessionError> {
        if self.state == PeerTransportState::Closed {
            return Err(PeerSessionError::TransportClosed);
        }
        if self.state == PeerTransportState::Authenticated {
            return Err(PeerSessionError::SessionAlreadyBound);
        }
        if session.local_node_id != self.local_node_id {
            self.close();
            return Err(PeerSessionError::SessionLocalIdentityMismatch);
        }
        if session.manifest_digest() != self.manifest_digest {
            self.close();
            return Err(PeerSessionError::SessionManifestMismatch);
        }
        if session.local_transport_id != self.transport_id {
            self.close();
            return Err(PeerSessionError::SessionTransportMismatch);
        }
        let peer_id = session.authenticated_peer_id();
        if !self.registry.claim(peer_id, self.transport_id) {
            self.close();
            return Err(PeerSessionError::DuplicatePeerIdentity(peer_id));
        }
        self.authenticated_peer_id = Some(peer_id);
        self.state = PeerTransportState::Authenticated;
        Ok(())
    }

    /// Authorize one frame class and terminate on ordinary pre-auth traffic.
    pub fn authorize_message(&mut self, message: PeerMessageClass) -> Result<(), PeerSessionError> {
        match self.state {
            PeerTransportState::Closed => Err(PeerSessionError::TransportClosed),
            PeerTransportState::Quarantined => match message {
                PeerMessageClass::Handshake | PeerMessageClass::Error => Ok(()),
                PeerMessageClass::Close => {
                    self.close();
                    Ok(())
                }
                ordinary => {
                    self.close();
                    Err(PeerSessionError::PreAuthenticationMessage(ordinary))
                }
            },
            PeerTransportState::Authenticated => {
                if message == PeerMessageClass::Close {
                    self.close();
                }
                Ok(())
            }
        }
    }

    /// Current terminal or authentication state.
    pub fn state(&self) -> PeerTransportState {
        self.state
    }

    /// Verified logical peer identity, available only after binding a session.
    pub fn authenticated_peer_id(&self) -> Option<Uuid> {
        self.authenticated_peer_id
    }

    /// Return the unique local binding included in this transport's handshake.
    pub fn transport_id(&self) -> PeerTransportId {
        self.transport_id
    }

    /// Validate that this quarantined transport belongs to the active local contract.
    pub(crate) fn handshake_binding(
        &self,
        local_node_id: Uuid,
        manifest_digest: [u8; 32],
    ) -> Result<PeerTransportId, PeerSessionError> {
        match self.state {
            PeerTransportState::Closed => return Err(PeerSessionError::TransportClosed),
            PeerTransportState::Authenticated => return Err(PeerSessionError::SessionAlreadyBound),
            PeerTransportState::Quarantined => {}
        }
        if self.local_node_id != local_node_id {
            return Err(PeerSessionError::SessionLocalIdentityMismatch);
        }
        if self.manifest_digest != manifest_digest {
            return Err(PeerSessionError::SessionManifestMismatch);
        }
        Ok(self.transport_id)
    }

    /// Return the contract-bound subject only for an authenticated transport.
    pub(crate) fn authenticated_binding(&self) -> Option<(Uuid, Uuid, [u8; 32])> {
        if self.state != PeerTransportState::Authenticated {
            return None;
        }
        self.authenticated_peer_id
            .map(|peer_id| (self.local_node_id, peer_id, self.manifest_digest))
    }

    fn close(&mut self) {
        if let Some(peer_id) = self.authenticated_peer_id.take() {
            self.registry.release(peer_id, self.transport_id);
        }
        self.state = PeerTransportState::Closed;
    }
}

impl Drop for PeerTransport {
    fn drop(&mut self) {
        if let Some(peer_id) = self.authenticated_peer_id {
            self.registry.release(peer_id, self.transport_id);
        }
    }
}

impl AuthenticatedPeerSession {
    /// Local endpoint of this association.
    pub fn local_node_id(&self) -> Uuid {
        self.local_node_id
    }

    /// Logical peer identity verified from the active manifest and proof.
    pub fn authenticated_peer_id(&self) -> Uuid {
        self.authenticated_peer_id
    }

    /// Digest of the exact mutually signed handshake transcript.
    pub fn transcript_digest(&self) -> [u8; 32] {
        self.transcript_digest
    }

    /// Digest of the manifest under which this session was authenticated.
    pub fn manifest_digest(&self) -> [u8; 32] {
        self.context.manifest_digest
    }

    /// Trusted local completion time in Unix milliseconds.
    pub fn established_at_millis(&self) -> u64 {
        self.established_at_millis
    }
}

#[derive(Debug, Clone)]
struct PendingInitiator {
    hello: PeerHandshakeHello,
}

#[derive(Debug, Clone)]
struct PendingResponder {
    initiator_node_id: Uuid,
    local_transport_id: PeerTransportId,
    issued_at_millis: u64,
}

/// Stateful challenge-response engine owned by one activated reference node.
pub(crate) struct PeerSessionAuthenticator {
    local_node_id: Uuid,
    context: PeerSessionContext,
    identity_key: SigningKey,
    members: BTreeMap<Uuid, NetworkMember>,
    pending_initiator: HashMap<[u8; 32], PendingInitiator>,
    pending_responder: HashMap<[u8; 32], PendingResponder>,
    pending_transports: BTreeSet<PeerTransportId>,
    seen_remote_nonces: HashMap<(Uuid, [u8; 32]), u64>,
    completed_transcripts: HashMap<[u8; 32], u64>,
}

impl PeerSessionAuthenticator {
    /// Create a bounded handshake engine for one activated membership contract.
    pub(crate) fn new(profile: &NetworkProfile, membership: &ActiveMembership) -> Self {
        let manifest = &membership.signed_manifest().manifest;
        Self {
            local_node_id: membership.local_node_id(),
            context: PeerSessionContext {
                protocol_version: PEER_SESSION_PROTOCOL_VERSION,
                network_id: profile.network_id.clone(),
                network_profile_id: profile.profile_id.clone(),
                manifest_id: manifest.manifest_id.clone(),
                manifest_version: manifest.version,
                manifest_digest: membership.manifest_digest(),
            },
            identity_key: membership.identity_key().clone(),
            members: manifest
                .members
                .iter()
                .cloned()
                .map(|member| (member.node_id, member))
                .collect(),
            pending_initiator: HashMap::new(),
            pending_responder: HashMap::new(),
            pending_transports: BTreeSet::new(),
            seen_remote_nonces: HashMap::new(),
            completed_transcripts: HashMap::new(),
        }
    }

    /// Start one outbound handshake bound to the supplied quarantined transport.
    pub(crate) fn begin_at(
        &mut self,
        local_transport_id: PeerTransportId,
        responder_node_id: Uuid,
        now_millis: u64,
    ) -> Result<PeerHandshakeHello, PeerSessionError> {
        self.prune_expired(now_millis);
        if responder_node_id == self.local_node_id {
            return Err(PeerSessionError::SelfHandshake);
        }
        self.active_peer(responder_node_id)?;
        self.reserve_pending_transport(local_transport_id)?;

        let mut initiator_nonce = [0u8; 32];
        loop {
            OsRng.fill_bytes(&mut initiator_nonce);
            if !self.pending_initiator.contains_key(&initiator_nonce) {
                break;
            }
        }
        let hello = PeerHandshakeHello {
            context: self.context.clone(),
            initiator_node_id: self.local_node_id,
            responder_node_id,
            initiator_transport_id: local_transport_id,
            issued_at_millis: now_millis,
            initiator_nonce,
        };
        self.pending_initiator.insert(
            initiator_nonce,
            PendingInitiator {
                hello: hello.clone(),
            },
        );
        Ok(hello)
    }

    /// Accept one inbound hello on the exact quarantined responder transport.
    pub(crate) fn accept_at(
        &mut self,
        local_transport_id: PeerTransportId,
        hello: PeerHandshakeHello,
        now_millis: u64,
    ) -> Result<PeerHandshakeChallenge, PeerSessionError> {
        self.prune_expired(now_millis);
        self.validate_context(&hello.context)?;
        if hello.responder_node_id != self.local_node_id {
            return Err(PeerSessionError::WrongResponder);
        }
        if hello.initiator_node_id == self.local_node_id {
            return Err(PeerSessionError::SelfHandshake);
        }
        self.validate_freshness(hello.issued_at_millis, now_millis)?;
        self.active_peer(hello.initiator_node_id)?;
        let remote_nonce = (hello.initiator_node_id, hello.initiator_nonce);
        if self.seen_remote_nonces.contains_key(&remote_nonce) {
            return Err(PeerSessionError::ReplayedNonce);
        }
        self.reserve_pending_transport(local_transport_id)?;
        if let Err(error) = self.record_remote_nonce(remote_nonce, hello.issued_at_millis) {
            self.pending_transports.remove(&local_transport_id);
            return Err(error);
        }

        let mut responder_nonce = [0u8; 32];
        let transcript_digest = loop {
            OsRng.fill_bytes(&mut responder_nonce);
            let digest = transcript_digest(&hello, &local_transport_id, &responder_nonce);
            if !self.pending_responder.contains_key(&digest)
                && !self.completed_transcripts.contains_key(&digest)
            {
                break digest;
            }
        };
        let responder_proof = self.identity_key.sign(&transcript_digest).to_bytes();
        self.pending_responder.insert(
            transcript_digest,
            PendingResponder {
                initiator_node_id: hello.initiator_node_id,
                local_transport_id,
                issued_at_millis: hello.issued_at_millis,
            },
        );
        Ok(PeerHandshakeChallenge {
            hello,
            responder_transport_id: local_transport_id,
            responder_nonce,
            responder_proof,
        })
    }

    /// Verify a responder proof on the initiating transport and issue one-use session evidence.
    pub(crate) fn answer_at(
        &mut self,
        local_transport_id: PeerTransportId,
        challenge: PeerHandshakeChallenge,
        now_millis: u64,
    ) -> Result<(PeerHandshakeResponse, AuthenticatedPeerSession), PeerSessionError> {
        self.prune_expired(now_millis);
        let transcript_digest = transcript_digest(
            &challenge.hello,
            &challenge.responder_transport_id,
            &challenge.responder_nonce,
        );
        let Some(pending) = self
            .pending_initiator
            .remove(&challenge.hello.initiator_nonce)
        else {
            return if self.completed_transcripts.contains_key(&transcript_digest) {
                Err(PeerSessionError::ReplayedProof)
            } else {
                Err(PeerSessionError::UnknownHandshake)
            };
        };
        self.pending_transports
            .remove(&pending.hello.initiator_transport_id);
        if pending.hello.initiator_transport_id != local_transport_id {
            return Err(PeerSessionError::SessionTransportMismatch);
        }
        if pending.hello != challenge.hello {
            return Err(PeerSessionError::WrongTranscript);
        }
        self.validate_context(&challenge.hello.context)?;
        self.validate_freshness(challenge.hello.issued_at_millis, now_millis)?;
        let responder = self.active_peer(challenge.hello.responder_node_id)?;
        verify_identity_proof(responder, &transcript_digest, &challenge.responder_proof)?;
        let initiator_proof = self.identity_key.sign(&transcript_digest).to_bytes();
        self.record_completed_transcript(transcript_digest, now_millis)?;
        Ok((
            PeerHandshakeResponse {
                transcript_digest,
                initiator_proof,
            },
            AuthenticatedPeerSession {
                local_node_id: self.local_node_id,
                authenticated_peer_id: challenge.hello.responder_node_id,
                local_transport_id,
                context: self.context.clone(),
                transcript_digest,
                established_at_millis: now_millis,
            },
        ))
    }

    /// Verify an initiator proof on the accepting transport and issue one-use session evidence.
    pub(crate) fn finish_at(
        &mut self,
        local_transport_id: PeerTransportId,
        response: PeerHandshakeResponse,
        now_millis: u64,
    ) -> Result<AuthenticatedPeerSession, PeerSessionError> {
        self.prune_expired(now_millis);
        let Some(pending) = self.pending_responder.remove(&response.transcript_digest) else {
            return if self
                .completed_transcripts
                .contains_key(&response.transcript_digest)
            {
                Err(PeerSessionError::ReplayedProof)
            } else {
                Err(PeerSessionError::UnknownHandshake)
            };
        };
        self.pending_transports.remove(&pending.local_transport_id);
        if pending.local_transport_id != local_transport_id {
            return Err(PeerSessionError::SessionTransportMismatch);
        }
        self.validate_freshness(pending.issued_at_millis, now_millis)?;
        let initiator = self.active_peer(pending.initiator_node_id)?;
        verify_identity_proof(
            initiator,
            &response.transcript_digest,
            &response.initiator_proof,
        )?;
        self.record_completed_transcript(response.transcript_digest, now_millis)?;

        Ok(AuthenticatedPeerSession {
            local_node_id: self.local_node_id,
            authenticated_peer_id: pending.initiator_node_id,
            local_transport_id,
            context: self.context.clone(),
            transcript_digest: response.transcript_digest,
            established_at_millis: now_millis,
        })
    }

    fn reserve_pending_transport(
        &mut self,
        local_transport_id: PeerTransportId,
    ) -> Result<(), PeerSessionError> {
        if self.pending_transports.contains(&local_transport_id) {
            return Err(PeerSessionError::TransportHandshakeInProgress);
        }
        if self.pending_initiator.len() + self.pending_responder.len()
            >= MAX_PENDING_PEER_HANDSHAKES
        {
            return Err(PeerSessionError::HandshakeCapacityReached);
        }
        self.pending_transports.insert(local_transport_id);
        Ok(())
    }

    fn record_remote_nonce(
        &mut self,
        remote_nonce: (Uuid, [u8; 32]),
        issued_at_millis: u64,
    ) -> Result<(), PeerSessionError> {
        if self.seen_remote_nonces.len() >= MAX_PEER_REPLAY_RECORDS {
            return Err(PeerSessionError::HandshakeCapacityReached);
        }
        self.seen_remote_nonces.insert(
            remote_nonce,
            issued_at_millis.saturating_add(PEER_HANDSHAKE_TIMEOUT_MILLIS),
        );
        Ok(())
    }

    fn record_completed_transcript(
        &mut self,
        transcript_digest: [u8; 32],
        now_millis: u64,
    ) -> Result<(), PeerSessionError> {
        if self.completed_transcripts.len() >= MAX_PEER_REPLAY_RECORDS {
            return Err(PeerSessionError::HandshakeCapacityReached);
        }
        self.completed_transcripts.insert(
            transcript_digest,
            now_millis.saturating_add(PEER_HANDSHAKE_TIMEOUT_MILLIS),
        );
        Ok(())
    }

    fn prune_expired(&mut self, now_millis: u64) {
        self.pending_initiator
            .retain(|_, pending| !handshake_expired(pending.hello.issued_at_millis, now_millis));
        self.pending_responder
            .retain(|_, pending| !handshake_expired(pending.issued_at_millis, now_millis));
        self.pending_transports.clear();
        self.pending_transports.extend(
            self.pending_initiator
                .values()
                .map(|pending| pending.hello.initiator_transport_id),
        );
        self.pending_transports.extend(
            self.pending_responder
                .values()
                .map(|pending| pending.local_transport_id),
        );
        self.seen_remote_nonces
            .retain(|_, expires_at| now_millis <= *expires_at);
        self.completed_transcripts
            .retain(|_, expires_at| now_millis <= *expires_at);
    }

    fn validate_context(&self, context: &PeerSessionContext) -> Result<(), PeerSessionError> {
        if context != &self.context {
            return Err(PeerSessionError::WrongSessionContext);
        }
        Ok(())
    }

    fn validate_freshness(
        &self,
        issued_at_millis: u64,
        now_millis: u64,
    ) -> Result<(), PeerSessionError> {
        let age = now_millis
            .checked_sub(issued_at_millis)
            .ok_or(PeerSessionError::HandshakeFromFuture)?;
        if age > PEER_HANDSHAKE_TIMEOUT_MILLIS {
            return Err(PeerSessionError::HandshakeExpired);
        }
        Ok(())
    }

    fn active_peer(&self, node_id: Uuid) -> Result<&NetworkMember, PeerSessionError> {
        let member = self
            .members
            .get(&node_id)
            .ok_or(PeerSessionError::UnknownPeer(node_id))?;
        if member.status != MemberStatus::Active {
            return Err(PeerSessionError::RemovedPeer(node_id));
        }
        if !member.roles.contains(&MemberRole::Peer) {
            return Err(PeerSessionError::PeerRoleInactive(node_id));
        }
        Ok(member)
    }
}

fn verify_identity_proof(
    member: &NetworkMember,
    transcript_digest: &[u8; 32],
    proof: &[u8; 64],
) -> Result<(), PeerSessionError> {
    let key = VerifyingKey::from_bytes(&member.identity_public_key)
        .map_err(|_| PeerSessionError::InvalidIdentityProof(member.node_id))?;
    let signature = Signature::from_bytes(proof);
    key.verify_strict(transcript_digest, &signature)
        .map_err(|_| PeerSessionError::InvalidIdentityProof(member.node_id))
}

fn handshake_expired(issued_at_millis: u64, now_millis: u64) -> bool {
    now_millis
        .checked_sub(issued_at_millis)
        .is_some_and(|age| age > PEER_HANDSHAKE_TIMEOUT_MILLIS)
}

fn context_bytes(context: &PeerSessionContext) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(HANDSHAKE_CONTEXT_MAGIC);
    bytes.extend_from_slice(&context.protocol_version.to_be_bytes());
    write_len_prefixed_text(&mut bytes, &context.network_id);
    write_len_prefixed_text(&mut bytes, &context.network_profile_id);
    write_len_prefixed_text(&mut bytes, &context.manifest_id);
    bytes.extend_from_slice(&context.manifest_version.to_be_bytes());
    bytes.extend_from_slice(&context.manifest_digest);
    bytes
}

fn hello_bytes(hello: &PeerHandshakeHello) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(HANDSHAKE_HELLO_MAGIC);
    let context = context_bytes(&hello.context);
    bytes.extend_from_slice(&(context.len() as u64).to_be_bytes());
    bytes.extend_from_slice(&context);
    bytes.extend_from_slice(hello.initiator_node_id.as_bytes());
    bytes.extend_from_slice(hello.responder_node_id.as_bytes());
    bytes.extend_from_slice(hello.initiator_transport_id.as_bytes());
    bytes.extend_from_slice(&hello.issued_at_millis.to_be_bytes());
    bytes.extend_from_slice(&hello.initiator_nonce);
    bytes
}

fn transcript_digest(
    hello: &PeerHandshakeHello,
    responder_transport_id: &PeerTransportId,
    responder_nonce: &[u8; 32],
) -> [u8; 32] {
    hash_domain_separated_parts(
        TRANSCRIPT_DIGEST_DOMAIN,
        &[
            &hello_digest(hello),
            responder_transport_id.as_bytes(),
            responder_nonce,
        ],
    )
}

fn hello_digest(hello: &PeerHandshakeHello) -> [u8; 32] {
    hash_domain_separated_parts(HELLO_DIGEST_DOMAIN, &[&hello_bytes(hello)])
}

/// Fail-closed errors emitted while establishing an authenticated peer session.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PeerSessionError {
    /// The requested peer is not declared by the active manifest.
    #[error("peer {0} is unknown to the active Membership Manifest")]
    UnknownPeer(Uuid),
    /// The requested peer is removed in the active manifest version.
    #[error("peer {0} is removed in the active Membership Manifest")]
    RemovedPeer(Uuid),
    /// The member lacks the active peer role.
    #[error("peer role is inactive for member {0}")]
    PeerRoleInactive(Uuid),
    /// A node cannot authenticate a session to itself.
    #[error("a node cannot establish a peer session with itself")]
    SelfHandshake,
    /// The message was addressed to a different responder identity.
    #[error("peer handshake targets a different responder")]
    WrongResponder,
    /// Network, profile, manifest, or protocol binding differs.
    #[error("peer handshake context does not match the active membership contract")]
    WrongSessionContext,
    /// A received transcript differs from the locally initiated hello.
    #[error("peer handshake transcript differs from the initiated transcript")]
    WrongTranscript,
    /// The caller supplied a time earlier than the handshake creation time.
    #[error("peer handshake creation time is in the future")]
    HandshakeFromFuture,
    /// The bounded handshake deadline elapsed.
    #[error("peer handshake expired")]
    HandshakeExpired,
    /// This transport already owns a live handshake state.
    #[error("peer transport already has a handshake in progress")]
    TransportHandshakeInProgress,
    /// Pending or replay-window state reached its fail-closed global cap.
    #[error("peer handshake state reached its bounded capacity")]
    HandshakeCapacityReached,
    /// The remote reused an initiator nonce already seen by this node.
    #[error("peer handshake nonce was replayed")]
    ReplayedNonce,
    /// A proof or completed transcript was submitted more than once.
    #[error("peer identity proof was replayed")]
    ReplayedProof,
    /// No live handshake state matches the supplied message.
    #[error("peer handshake is unknown or already closed")]
    UnknownHandshake,
    /// The manifest-listed Node Identity Key did not verify the proof.
    #[error("peer {0} supplied an invalid Node Identity proof")]
    InvalidIdentityProof(Uuid),
    /// An ordinary frame arrived before mutual peer authentication completed.
    #[error("pre-authenticated transport attempted {0:?} behavior")]
    PreAuthenticationMessage(PeerMessageClass),
    /// A terminal transport cannot receive or bind any more frames.
    #[error("peer transport is closed")]
    TransportClosed,
    /// A transport may bind at most one authenticated session.
    #[error("peer transport already has an authenticated session")]
    SessionAlreadyBound,
    /// The supplied session was established by a different local node.
    #[error("authenticated session belongs to a different local node")]
    SessionLocalIdentityMismatch,
    /// The supplied session was established under a different manifest.
    #[error("authenticated session belongs to a different Membership Manifest")]
    SessionManifestMismatch,
    /// The supplied session belongs to a different transport capability.
    #[error("authenticated session belongs to a different peer transport")]
    SessionTransportMismatch,
    /// One live transport is already bound to this logical peer identity.
    #[error("peer identity {0} already owns an authenticated transport")]
    DuplicatePeerIdentity(Uuid),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replay_key(index: usize) -> [u8; 32] {
        let mut value = [0u8; 32];
        value[..8].copy_from_slice(&(index as u64).to_be_bytes());
        value
    }

    fn empty_authenticator() -> PeerSessionAuthenticator {
        PeerSessionAuthenticator {
            local_node_id: Uuid::from_u128(1),
            context: PeerSessionContext {
                protocol_version: PEER_SESSION_PROTOCOL_VERSION,
                network_id: "test-network".to_string(),
                network_profile_id: "test-profile".to_string(),
                manifest_id: "test-manifest".to_string(),
                manifest_version: 1,
                manifest_digest: [1u8; 32],
            },
            identity_key: SigningKey::from_bytes(&[2u8; 32]),
            members: BTreeMap::new(),
            pending_initiator: HashMap::new(),
            pending_responder: HashMap::new(),
            pending_transports: BTreeSet::new(),
            seen_remote_nonces: HashMap::new(),
            completed_transcripts: HashMap::new(),
        }
    }

    #[test]
    fn replay_windows_fail_closed_at_capacity_and_reopen_only_after_ttl() {
        let mut authenticator = empty_authenticator();
        let peer_id = Uuid::from_u128(2);
        let recorded_at = 1_000_000;

        for index in 0..MAX_PEER_REPLAY_RECORDS {
            let key = replay_key(index);
            authenticator
                .record_remote_nonce((peer_id, key), recorded_at)
                .expect("remote nonce below replay-window cap");
            authenticator
                .record_completed_transcript(key, recorded_at)
                .expect("completed transcript below replay-window cap");
        }
        let overflow_key = replay_key(MAX_PEER_REPLAY_RECORDS);
        assert_eq!(
            authenticator
                .record_remote_nonce((peer_id, overflow_key), recorded_at)
                .expect_err("remote nonce replay window must be bounded"),
            PeerSessionError::HandshakeCapacityReached
        );
        assert_eq!(
            authenticator
                .record_completed_transcript(overflow_key, recorded_at)
                .expect_err("completed transcript replay window must be bounded"),
            PeerSessionError::HandshakeCapacityReached
        );

        authenticator.prune_expired(recorded_at + PEER_HANDSHAKE_TIMEOUT_MILLIS);
        assert_eq!(
            authenticator.seen_remote_nonces.len(),
            MAX_PEER_REPLAY_RECORDS
        );
        assert_eq!(
            authenticator.completed_transcripts.len(),
            MAX_PEER_REPLAY_RECORDS
        );

        authenticator.prune_expired(recorded_at + PEER_HANDSHAKE_TIMEOUT_MILLIS + 1);
        assert!(authenticator.seen_remote_nonces.is_empty());
        assert!(authenticator.completed_transcripts.is_empty());
        authenticator
            .record_remote_nonce((peer_id, overflow_key), recorded_at)
            .expect("expired remote nonces must release replay-window capacity");
        authenticator
            .record_completed_transcript(overflow_key, recorded_at)
            .expect("expired transcripts must release replay-window capacity");
    }
}
