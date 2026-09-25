# ADR 0015: Bound Remediation to a Thesis-Defensible End-to-End Reference System

**Status:** Accepted
**Date:** 2026-08-29
**Context:** Thesis-to-code remediation scope and evidence boundaries

---

## Decision

The current remediation target is a **thesis-defensible end-to-end reference system**, not an operational production pilot.

Completion of this milestone requires implemented behavior and reproducible evidence for all of the following:

- three-node PoA convergence,
- atomic fail-closed admission,
- a durable privacy lifecycle,
- authenticated network membership,
- ontology-package-declared SHACL enforcement,
- a bounded ProvChain-to-ProvChain bridge, and
- corrected reproducible thesis evidence.

The following remain future work or later milestones:

- PBFT completion and distributed transport,
- heterogeneous or SPV-style bridges,
- a human usability study,
- operational deployment, and
- production-pilot controls.

This decision accepts the scope boundary; it does not assert that the in-scope capabilities are already implemented or evidenced.

## Rationale

This boundary closes the central thesis-to-code gaps with a finite, testable stop condition. Expanding the same milestone to production operations, heterogeneous chains, PBFT, or human-subject evaluation would introduce distinct research questions and external operational dependencies without being necessary to demonstrate the reference system's core contribution.

## Consequences

- specifications, tickets, tests, and evidence must map explicitly to the seven in-scope outcomes;
- a capability is not complete merely because an isolated unit or prototype exists;
- thesis and publication wording must distinguish reproduced evidence from planned or historical claims; and
- out-of-scope capabilities must be labelled experimental, future work, or the next milestone rather than implied as completed.

## Related Decisions

- [ADR 0006](./0006-dual-consensus-protocol.md) remains a long-term consensus direction; PBFT is outside this remediation milestone.
- [ADR 0014](./0014-use-shared-ontology-packages-and-spacl-production-path.md) remains the semantic architecture boundary for package-declared validation.
- [ADR 0037](./0037-bound-provchain-bridge-to-converged-source-evidence-and-final-admission.md) fixes the bounded bridge as exact public ProvChain-to-ProvChain import and keeps heterogeneous/SPV, mapping, private-data, and asset bridges outside this milestone.

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-08-29
