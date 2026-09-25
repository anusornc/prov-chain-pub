# ADR 0022: Authenticate Peer Sessions with Mutual Ed25519 Challenge-Response

**Status:** Accepted
**Date:** 2026-08-30
**Context:** Bounded proof of permissioned-peer identity over the existing WebSocket transport

---

## Decision

Every inbound and outbound P2P WebSocket in the thesis reference system must complete mutual, domain-separated Ed25519 nonce challenge-response against the active Membership Manifest before it becomes an **Authenticated Peer Session**.

Each manifest entry binds its logical node identity to a dedicated **Node Identity Key**, role, and status. This key is separate from the Governance Trust Root, wallet keys, and PoA block-signing keys. Validator nodes therefore hold both a node identity key for peer authentication and a PoA key for block proposals; non-validator members require only the identity key for P2P membership.

Before authentication succeeds, a connection may carry only handshake, error, and close messages. Ordinary discovery, synchronization, consensus, and application messages received before mutual authentication must fail closed and terminate the connection.

Each endpoint contributes a fresh cryptographically random nonce and signs the same canonical transcript. The transcript must be domain-separated and bind at least the protocol version, network and Network Profile identities, Membership Manifest identity/version/hash, canonical initiator and responder node identities, and both nonces. Each endpoint verifies the remote signature with the identity key and active status resolved from the manifest. A mismatch, invalid signature, replay, duplicate identity conflict, or handshake timeout terminates the connection.

Only after both proofs succeed may the transport be bound to the verified logical node identity, semantic-contract compatibility be evaluated, and ordinary P2P handlers receive messages.

This milestone does not add mTLS, transport confidentiality, or a signed envelope for every later WebSocket frame. The architecture therefore claims fresh membership-key possession at session establishment, not cryptographic integrity or confidentiality of the whole transport. Every artifact capable of changing authoritative ledger history must still carry its own end-to-end admission evidence and pass Final Admission. mTLS and per-message authenticated transport envelopes remain future hardening work.

## Rationale

The current P2P path accepts self-asserted node IDs over plaintext WebSockets and dispatches messages without an authentication gate. Mutual signed nonces are the smallest mechanism that demonstrates possession of manifest-listed keys, rejects replay, and preserves the existing WebSocket/JSON protocol for the bounded reference system.

mTLS would protect the complete channel but would also introduce certificate issuance, certificate-to-manifest binding, rotation, revocation, TLS endpoint configuration, and deployment evidence beyond the current milestone. Signing every P2P frame would require a canonical message envelope, sequence and replay-window rules, and changes across every message path. Both remain valid later milestones; neither is implied by this decision.

Separating identity and PoA keys preserves the distinction between peer authentication and Consensus Acceptance, permits non-validator membership, limits compromise scope, and allows membership-key rotation without silently changing the validator schedule.

## Consequences

- node configuration must reference a local node identity private key whose public key matches the active manifest;
- node startup must fail before opening P2P traffic when the configured identity is absent, removed, mismatched, or cannot sign;
- the P2P state machine must quarantine new transports until mutual authentication completes;
- a logical node ID comes from verified manifest binding, never from an unsigned discovery field;
- authentication transcripts require canonical encoding, explicit domain/version separation, fresh nonces, bounded timeout, and replay tests;
- three-node tests must cover successful mutual authentication plus unknown, removed, wrong-key, wrong-network, wrong-manifest, modified-transcript, replayed-proof, pre-auth-message, and timeout rejection;
- subsequent semantic compatibility remains a distinct gate and cannot substitute for identity proof; and
- publication wording must preserve the boundary between authenticated session establishment and a fully secure transport channel.

## Related Decisions

- [ADR 0004](./0004-use-ed25519-signatures.md) selects Ed25519 while this ADR assigns a separate peer-identity key role.
- [ADR 0007](./0007-websocket-p2p-protocol.md) remains the transport choice for the bounded reference system.
- [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md) keeps ledger mutation dependent on end-to-end admission evidence rather than transport trust.
- [ADR 0021](./0021-use-a-governance-signed-membership-manifest.md) defines the registry against which identity proofs are verified.
- [ADR 0023](./0023-replicate-exact-committed-envelopes-before-declaring-convergence.md) requires independently verifiable envelopes and signed Commit Receipts after session establishment.
- [ADR 0037](./0037-bound-provchain-bridge-to-converged-source-evidence-and-final-admission.md) treats transport as delivery only; an Authenticated Peer Session never substitutes for the exact end-to-end Bridge Proof Bundle.

## Implementation Status

Accepted architecture; implementation is pending. Current discovery messages contain unsigned node, network, and semantic metadata; incoming and outgoing WebSockets use temporary random identities; ordinary messages reach handlers without an authenticated-session gate; and no node identity key exists in network configuration.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-30
