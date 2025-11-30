# Codebase Reorganization Plan (Draft)

Status: Draft for review/decision

Owner: Core maintainers (Reasoner + Parser)

Last updated: YYYY-MM-DD

---

## 1) Context & Goals

The current `owl2-reasoner` crate contains core types, parsers, reasoning engines, EPCIS functionality, benchmarks, and examples. This breadth increases cognitive load, compile times, and coupling. We propose reorganizing the workspace into focused crates with clear boundaries while maintaining backward compatibility and protecting downstream consumers (tests, benches, and external benchmarking tools in the workspace root).

Goals:
- Improve navigability and maintainability (smaller, focused modules and crates).
- Reduce build times for common edit/compile loops.
- Better dependency isolation (parsers and heavy deps feature-gated, not always pulled).
- Preserve public API stability or provide smooth migration paths.
- Keep benchmarks, tests, and external tools functional throughout the transition.

Non-goals (for this reorg):
- No major algorithmic changes.
- No performance regressions (we will measure, but not attempt large optimizations here).

---

## 2) Proposed Target Architecture

Workspace crates (new/existing):

- `owl2-core` (new)
  - error, iri, entities, axioms, ontology, cache, memory
  - minimal dependencies (e.g., `hashbrown`, `smallvec`, `once_cell`, `thiserror`)
  - stable public types and prelude

- `owl2-parser` (new)
  - parsers: Turtle, RDF/XML, OWL/XML, Functional syntax; shared `common`, `arena`
  - feature-gated backends: `rio-xml`, `turtle`, `owl-xml`, `sophia`
  - depends on `owl2-core`

- `owl2-reasoner` (current, slimmed)
  - reasoning engines (tableaux, classification, rules), profiles
  - depends on `owl2-core`; optional feature `parsers` to pull `owl2-parser`

- `owl2-epcis` (optional, new)
  - EPCIS types, parsers, generators, test-data
  - depends on `owl2-core` (+ optionally `owl2-parser`)

- `owl2-cli` (optional, new)
  - CLI wrapper for examples and tools

- `owl2-benches` (optional, new)
  - all Criterion benches decoupled from core crates

- `owl2-reasoner-test-suite` (existing, unchanged path)
  - stays a separate crate; adapt imports to new structure as needed

Module structure (within crates) — split large files:

- ontology/: `mod.rs`, `core.rs`, `indexes.rs`, `ops.rs`, `validate.rs`
- iri/: `mod.rs`, `cache.rs`, `validate.rs`
- cache/: `mod.rs`, `config.rs`, `strategies.rs`, `bounded.rs`, `metrics.rs`
- reasoning/: `tableaux/`, `classification/`, `rules/`
- parser/: `common.rs`, `turtle.rs`, `rdf_xml.rs`, `owl_xml.rs`, `owl_functional.rs`, `arena.rs`

Public API exposure:
- `owl2-core::prelude::*` for common types
- `owl2-reasoner` exposes high-level reasoning API (`SimpleReasoner`, etc.)
- `owl2-parser` exposes `OntologyParser` and concrete parsers
- Keep curated re-exports; avoid wildcard flooding at root

---

## 3) Impact Analysis

Downstream code in this workspace:
- Examples (in `examples/`): will need minor import path updates if/when crates split.
- Internal benches (in `benches/`): can be moved to `owl2-benches` or adjusted to new imports.
- External benchmarking framework (in `benchmarking/`): mostly invokes binaries/scripts; ensure example runners/CLI entry points remain (or migrate to `owl2-cli`).
- Test suites (in `tests/` and `owl2-reasoner-test-suite/`): import adjustments required; functionality unchanged.

Binary & script compatibility:
- Maintain existing example binary names/paths; if moving to `owl2-cli`, create shims or document new commands.

Dependency footprint:
- Parser-heavy deps (rio, sophia, xml) move to `owl2-parser` and are feature-gated.
- `owl2-core` remains light — improves compile times for core edits.

Performance & size:
- No expected runtime regressions. We will benchmark before/after reorg.

Docs & CI:
- README and mdBook sections will be updated with new crate usage.
- CI will run clippy/fmt/test/doc for all crates.

---

## 4) Branching, Release, and Rollback Strategy

Branching:
- Create a long-lived feature branch: `feature/reorg-workspace`.
- Phase work into small PRs targeting this branch (see Phases below).

Release & versioning:
- Publish internal crates only when the workspace builds and tests pass.
- If publishing to crates.io later: use semver; keep `owl2-reasoner` as a meta crate depending on subcrates or re-exporting types to preserve user experience.

Rollback plan:
- Each phase is reversible via git revert if blockers arise.
- Keep master/main unchanged until acceptance criteria are met.

Merge plan:
- After final verification (benches/tests/docs/bench framework), create a PR from `feature/reorg-workspace` to `main`.

---

## 5) Phased Plan & Tasks

Phase 0 — Validation & Guardrails
- [ ] Capture current baseline: `cargo build`, `cargo test`, `cargo bench --no-run` timings
- [ ] Ensure CI has clippy/fmt/test across workspace
- [ ] Define acceptance criteria (see Section 8)

Phase 1 — In-crate Modularization (No public API break)
- [ ] Split `src/ontology.rs` into `src/ontology/{mod,core,indexes,ops,validate}.rs`
- [ ] Split `src/iri.rs` into `src/iri/{mod,cache,validate}.rs`
- [ ] Split `src/cache.rs` into `src/cache/{mod,config,strategies,bounded,metrics}.rs`
- [ ] Update `lib.rs` to use modules; keep public API surface identical (re-export where needed)
- [ ] Run full tests/benches to verify no behavior change

Phase 2 — Introduce `owl2-core`
- [ ] Create `owl2-core` crate and migrate: error, iri, entities, axioms, ontology, cache, memory
- [ ] Update `owl2-reasoner` to depend on `owl2-core`; fix imports
- [ ] Update examples/tests/benches imports; ensure build passes

Phase 3 — Introduce `owl2-parser`
- [ ] Create `owl2-parser`; move parser modules; add features: `rio-xml`, `turtle`, `owl-xml`
- [ ] Update feature flags in workspace; move heavy deps to `owl2-parser`
- [ ] Update `owl2-reasoner` feature `parsers` to pull `owl2-parser`
- [ ] Update examples/tests/benches that use parsing

Phase 4 — Optional crates
- [ ] Create `owl2-epcis` and move EPCIS modules + examples
- [ ] Create `owl2-cli` (optional) consolidating CLI/example runners
- [ ] Create `owl2-benches` to house Criterion benches; reduce warnings and separate deps

Phase 5 — Docs & CI updates
- [ ] README + mdBook: new crate structure, usage examples, bench commands
- [ ] scripts: ensure `run_benchmarks.sh` and `update_docs.sh` run across new crates
- [ ] CI: clippy, fmt, test, doc for all crates; benchmarks compile-check with `--no-run`

---

## 6) Testing & Benchmarking Strategy

Internal tests:
- Run `cargo test --workspace` after each phase.
- Add smoke tests for cross-crate imports.

Benches:
- Compile benches with `cargo bench --no-run` after each move.
- Run selected Criterion benches on representative datasets to confirm no regressions.

External benchmarking (in `benchmarking/`):
- Validate scripts continue to run after import path updates.
- Verify generated reports and summary scripts.

Examples:
- `cargo run --example ...` paths adapted.
- Optionally migrate to `owl2-cli` with parity commands.

---

## 7) Risk Assessment & Mitigations

Risks:
- Breakages in import paths across workspace tools/benches.
- Hidden internals becoming public by accident when refactoring modules.
- Temporary increases in compile time during transitions.

Mitigations:
- Keep re-export shims where helpful (e.g., `owl2-reasoner` re-exports common types from `owl2-core`).
- Small PRs; frequent integration tests.
- Feature flags to isolate optional functionality.

Rollback:
- Revert individual PRs on `feature/reorg-workspace` if needed.

---

## 8) Acceptance Criteria (for merging to main)

- Build: `cargo build --workspace` passes (dev + release)
- Tests: `cargo test --workspace` passes (including doctests)
- Benches: `cargo bench --no-run` succeeds
- External benchmarking scripts run end-to-end
- Docs: `cargo doc --no-deps` for crates + `mdbook build docs` succeed or are documented if optional
- No public API removals without deprecation or documented migration notes
- README and docs updated with new structure and usage

---

## 9) Communication & Coordination

- Announce plan to contributors; request feedback on crate boundaries.
- Document any blocking issues in this file (Decision Log).
- Provide migration notes for downstream users (if published externally).

---

## 10) Timeline (estimate)

- Phase 0: 0.5–1 day
- Phase 1: 1–2 days
- Phase 2: 2–3 days
- Phase 3: 2–3 days
- Phase 4: 1–3 days (optional)
- Phase 5: 1–2 days

Total: 7–13 working days depending on optional scope and review cadence.

---

## 11) Task Checklist (roll-up)

- [ ] Create branch `feature/reorg-workspace`
- [ ] Capture baselines and enable CI checks
- [ ] Phase 1: split large modules in-place
- [ ] Phase 2: create `owl2-core` and migrate core modules
- [ ] Phase 3: create `owl2-parser` and migrate parsers (+ features)
- [ ] Phase 4: optional `owl2-epcis`, `owl2-cli`, `owl2-benches`
- [ ] Phase 5: docs + CI updates
- [ ] Run full validation (build/test/benches/docs/external bench)
- [ ] Merge to `main`

---

## 12) Decision Log & Notes

- [ ] 2025-..-..: Draft plan created
- [ ] 2025-..-..: Review feedback incorporated
- [ ] 2025-..-..: Go/No-Go decision recorded

