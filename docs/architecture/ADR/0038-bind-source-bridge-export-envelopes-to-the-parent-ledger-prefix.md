# ADR 0038: Bind Source Bridge Export Envelopes to the Parent Ledger Prefix

**Status:** Accepted
**Date:** 2026-09-05
**Context:** Non-genesis source-prefix verification for `ProvChainBridgeSuiteV1`

---

## Decision

Every newly created source bridge export uses a **Prefix-Bound Bridge Export Envelope** whose
Proposal Body and Admitted Block Envelope contain the exact `previous_ledger_prefix_hash32`.
The field is covered by the Proposal Digest, Scheduled Authority signature, and Envelope Hash.
Source Final Admission verifies it against the current journal-derived parent prefix before the
sole append-plus-`fsync` commit point, and Verified Journal Replay verifies it sequentially.

The canonical parent-anchor order is:

```text
previous_envelope_hash32
previous_ledger_prefix_hash32
previous_state_commitment32
```

At source position zero, `previous_ledger_prefix_hash32` is the fixed virtual-genesis Ledger
Prefix Hash; it is not a zero sentinel. The strict codec rejects a missing, malformed, incorrectly
positioned, or noncanonical value.

The Prefix-Bound Bridge Export Envelope uses envelope codec version `6`. Version `4` retains its
legacy bridge-export meaning, version `5` retains its target bridge-import meaning, and existing
versions remain replayable. New source bridge-export construction emits version `6`, including at
source position zero.

`ProvChainBridgeSuiteV1`, `BridgeExportCoreV1`, the ten-field Bridge Export Declaration, the Bridge
Transfer ID derivation, and the five-field `BridgeProofBundleV1` framing do not change. The exact
source envelope already occupies field four of the proof, so the target obtains the authenticated
parent prefix without a relayer field or a second proof format.

For a prefix-bound proof, the target computes the existing ledger-prefix transition over the
authenticated parent prefix, source position, and exact source envelope bytes, then requires the
result to equal the Ledger Prefix Hash signed by all three pinned source receipt nodes. The trust
claim remains Source Convergence Evidence under the static reference trust model: at least one
pinned receipt signer must remain honest and issue no receipt before conforming Source Final
Admission. This is not full-history verification, SPV, BFT finality, or trustless finality.

The original version-4 genesis proof remains valid because its parent is the fixed virtual-genesis
prefix and can be reconstructed without an encoded parent field. A version-4 proof at any later
source position fails closed. Its committed source envelope remains valid history, but it is not
bridge-exportable; the payload must be declared again in a newly committed prefix-bound export at
a later position, producing the Transfer ID derived for that new position. Parent-prefix evidence
must never be retrofitted after source commitment.

All three source validators must support version `6` before the network creates a prefix-bound
export. This corrective extension does not claim safe rolling operation across mixed validator
versions. Existing committed envelope versions remain readable and replayable.

## Identity and conflict behavior

The parent Ledger Prefix is evidence for the source history context, not a new component of the
Bridge Export Declaration or Transfer ID. If conflicting proof bytes claim the same ledger
instance, source position, target, and payload under different parent-prefix evidence, they retain
the same Transfer ID but different proof hashes. The target returns `ReplayConflict` and treats the
condition as a safety incident; it does not create a second transfer identity or choose a fork.

## Conformance evidence

The published legacy genesis vector remains unchanged. A separate independently generated vector
must commit a prefix-bound export after at least one earlier source envelope and demonstrate:

1. source proposal signing and Final Admission bind the exact parent Ledger Prefix;
2. source replay reconstructs and verifies the same parent-to-result transition;
3. all three source receipts sign the identical resulting prefix;
4. target proof verification independently recomputes that result from the proof-carried envelope;
5. malformed or mismatched parent-prefix evidence creates no target candidate or transfer state;
6. version-4 non-genesis evidence fails closed; and
7. target import, restart, exact retry, conflict, and projection recovery retain ADR 0037 behavior.

## Scope boundary

This decision supplies the missing authenticated parent-prefix input for bridge-export envelopes
only. It does not redesign ordinary, privacy-control, or bridge-import envelope versions, create a
generic light-client proof, add source RPC or external history lookup, change receipt authority, or
broaden the bridge beyond ADR 0037. A generic realization of parent Ledger Prefix binding for other
Admission Kinds is separate architecture work.

## Related decisions

- [ADR 0016](./0016-make-the-ledger-journal-the-sole-commit-authority.md)
- [ADR 0017](./0017-use-one-final-admission-boundary-for-every-ledger-write.md)
- [ADR 0023](./0023-replicate-exact-committed-envelopes-before-declaring-convergence.md)
- [ADR 0024](./0024-commit-canonical-post-block-public-provenance-state.md)
- [ADR 0037](./0037-bound-provchain-bridge-to-converged-source-evidence-and-final-admission.md)
