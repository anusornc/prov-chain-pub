# ADR 0014: Use Shared Ontology Packages and SPACL as the Production Semantic Path

**Status:** Accepted
**Date:** 2026-03-09
**Context:** Semantic interoperability and traceability validation in permissioned networks

---

## Decision

**ProvChain will use SPACL-backed reasoning and `src/ontology/*` as the sole production semantic path.**

**Shared ontology packages are the semantic contract for interoperability across organizations in a permissioned network.**

This means:

- participating organizations in the same network are expected to use the same ontology package,
- semantic validation and reasoning for production workloads must flow through `src/ontology/*`,
- and demo-specific semantic modules in `src/semantic/*` are not part of the production claim unless explicitly reworked and promoted later.

---

## Rationale

### 1. The real system goal is general traceability, not domain-specific hardcoding

ProvChain is intended to support general traceability for permissioned networks. UHT, healthcare, and pharmaceutical flows are reference use cases that demonstrate how a network can adopt the platform using a shared ontology package.

### 2. Interoperability requires a shared semantic contract

The meaningful unit of interoperability is not a hardcoded application branch. It is the ontology package shared by network participants, including:

- ontology files,
- SHACL shapes,
- semantic validation rules,
- and version/hash information used for consistency checks.

### 3. Production guidance must use one semantic execution path

The repository contains older semantic experiments in `src/semantic/*`. Keeping those in the production narrative creates ambiguity and weakens both engineering and publication claims. SPACL plus `src/ontology/*` is the path that best matches current system intent.

---

## Consequences

### Positive

- clearer production architecture,
- clearer publication narrative,
- better interoperability story for consortium deployment,
- and cleaner future work around ontology packages and incremental semantic validation.

### Negative

- legacy modules remain in the repository but must be clearly marked as experimental,
- some CLI demos may still rely on non-production semantic code until they are migrated,
- and documentation must be kept consistent with this decision.

---

## Implementation Guidance

### Production semantic path

- `src/ontology/mod.rs`
- `src/ontology/domain_manager.rs`
- `src/ontology/shacl_validator.rs`
- `Cargo.toml` SPACL `owl2-reasoner` dependency

### Non-production semantic path

- `src/semantic/owl_reasoner.rs`
- `src/semantic/owl2_traceability.rs`
- `src/semantic/owl2_enhanced_reasoner.rs`
- related demos that depend on those modules

These modules may be retained for experimentation, migration support, or historical reference, but they are not the basis for production or publication claims.

### Shared ontology package model

Each deployable traceability domain should be represented as an ontology package containing:

- core ontology import or reference,
- domain ontology,
- SHACL shapes,
- package identifier,
- package version,
- package hash for network consistency,
- and example data or fixtures for validation and demos.

---

## Follow-up Work

1. migrate production documentation to the ontology-package framing
2. mark legacy semantic modules as experimental or legacy
3. align CLI help and demos with the production semantic path
4. build incremental semantic validation on top of the ontology-package model

---

**Authors:** Anusorn Chaikaew, Codex collaboration record
**Approval Date:** 2026-03-09
