# ADR 0021: Use a Governance-Signed Membership Manifest

**Status:** Accepted
**Date:** 2026-08-29
**Context:** Authenticated consortium membership for the bounded thesis reference system
**Clarified by:** [ADR 0030](./0030-bootstrap-the-principal-and-initial-authorization-key-atomically.md) — the Privacy Bootstrap Governance Key is a distinct Network Profile binding and cannot reuse or substitute the Membership Manifest Governance Trust Root.

---

## Decision

The thesis reference system uses one static, versioned, governance-signed **Membership Manifest** as the sole source of permissioned-network membership. The active Network Profile binds the manifest by stable identity, version, and content hash.

The manifest binds its target network and Network Profile identity and declares each **Network Member** identity verification key and its authorized roles, including whether it may act as a PoA validator. Every node verifies the manifest signature against a locally provisioned **Governance Trust Root** before activating the Network Profile. A missing, invalid, mismatched, or untrusted manifest fails closed.

Peer lists, bootstrap addresses, matching `network_id` values, matching semantic contract hashes, node-local configuration, and the current `authority_keys` field do not independently grant membership. Authority ordering in the Network Profile must resolve only to active validator members in the bound manifest; an unknown or removed identity is ineligible for peer authentication and consensus participation.

Membership changes occur by distributing and activating a new governance-signed manifest version. Dynamic self-registration, an online join workflow, replicated membership governance, and multi-party governance approval are future work outside the current milestone.

This decision establishes the trusted membership registry. ADR 0022 defines how peers prove possession of manifest-listed identity keys and keeps those keys separate from PoA block-signing keys.

## Rationale

A permissioned network cannot derive identity from peer-supplied UUIDs or from semantic compatibility alone. A static signed manifest gives all three reference nodes a deterministic, reproducible membership answer without expanding the thesis milestone into a membership-governance protocol. Pinning the governance trust root out of band keeps bootstrap trust explicit and testable.

## Consequences

- the Network Profile format must bind a specific signed manifest and reject substitution or downgrade;
- startup must validate the local Network Profile and authenticate its bound manifest before networking or block admission is enabled;
- peer authentication and Signer Authorization must resolve identities and roles through the active manifest;
- bootstrap peers remain connection hints only and discovered peers remain untrusted until authenticated;
- removed members lose prospective connection and consensus eligibility when a newer manifest is activated, without rewriting committed ledger history;
- fixtures must include governance keys, signed manifests, member roles, and invalid-signature, unknown-member, removed-member, wrong-network, and stale-version rejection cases; and
- the reference-system evidence must identify the exact manifest version and hash used for every three-node run.

## Related Decisions

- [ADR 0014](./0014-use-shared-ontology-packages-and-spacl-production-path.md) defines the semantic contract that is compatible with, but distinct from, authenticated membership.
- [ADR 0015](./0015-bound-remediation-to-thesis-defensible-reference-system.md) excludes operational membership governance from the current milestone.
- [ADR 0019](./0019-require-the-scheduled-authority-for-poa-acceptance.md) consumes authenticated validator membership as one prerequisite for PoA eligibility.
- [ADR 0020](./0020-stall-poa-on-a-missed-turn.md) derives the PoA schedule from the profile's ordered active validator set.
- [ADR 0037](./0037-bound-provchain-bridge-to-converged-source-evidence-and-final-admission.md) requires a target-profile-pinned Bridge Source Trust Binding to exact foreign profile/manifest artifacts; ordinary target membership or a proof-supplied key cannot grant foreign bridge trust.
- [ADR 0022](./0022-authenticate-peer-sessions-with-mutual-ed25519-challenge-response.md) defines peer proof of possession for manifest-listed identities.
- [ADR 0030](./0030-bootstrap-the-principal-and-initial-authorization-key-atomically.md) defines the separate governance key that authorizes participant bootstrap, not network membership.

## Implementation Status

Accepted architecture; implementation is pending. The current Network Profile contains consensus `authority_keys` but no member records or governance-signed manifest binding. Peer discovery accepts self-asserted node identity metadata after network and semantic compatibility checks, and current WebSocket connections do not prove possession of a manifest-listed identity key.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-29
