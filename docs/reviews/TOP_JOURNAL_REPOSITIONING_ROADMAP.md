# Top-Journal Repositioning Roadmap for ProvChain

**Date:** 2026-03-08
**Scope:** Research repositioning, code roadmap, and experiment plan for a top-tier journal submission
**Status:** Working document based on current codebase state

## Executive Summary

The current codebase is publishable only after the research claim is narrowed and the implementation path is unified. The strongest route is not to market ProvChain as a generic "blockchain + EPCIS + ontology" platform. That framing is too broad, easy to challenge, and already weakly supported by the current artifact.

The strongest route is to reposition ProvChain as a:

**Domain-general semantic blockchain framework with incremental semantic validation and ontology-driven domain adaptation for traceability workloads.**

This shifts the novelty from "integration" to a concrete method that can be implemented, benchmarked, ablated, and defended.

## Current Publication Blockers

### 1. Claim-to-evidence mismatch

The manuscript currently overstates several capabilities relative to the codebase:

- Full GS1 EPCIS 2.0 compliance
- Full CBV vocabulary coverage
- Production-grade OWL2 reasoning across the stack
- Multi-domain generalization already validated
- Fault-tolerance scenarios with stronger evidence than the test suite currently shows

This is the primary rejection risk.

### 2. Split semantic implementation paths

There are two semantic directions in the repository:

- Legacy semantic modules in `src/semantic/*`
- SPACL-backed reasoning in `owl2-reasoner` git dependency and `src/ontology/*`

For publication, experimental claims must come from one authoritative implementation path only.

### 3. Weak empirical story for "generalization"

The domain plugin architecture exists, but current adapters mainly prove configurability, not semantic generalization. The journal-level claim requires:

- domain-specific ontology loading,
- domain-specific SHACL constraints,
- domain-specific validation behavior,
- and cross-domain experimental evidence.

### 4. Reproducibility package is not submission-ready

The current publication package still contains placeholders, incomplete artifact references, and benchmark naming mismatches.

## Recommended Research Thesis

### Recommended paper position

Use the project as a systems paper centered on one main idea and one supporting idea:

1. **Main contribution:** Incremental semantic validation for blockchain traceability
2. **Supporting contribution:** Ontology-driven domain adaptation with reusable semantic contracts

### Recommended title direction

Use a title in this family:

`ProvChain: A Domain-General Semantic Blockchain with Incremental Validation for Multi-Domain Traceability`

Avoid title directions that imply:

- full standards conformance unless formally verified,
- fully general production deployment,
- or complete OWL2 reasoning across all execution paths.

## Novelty Package That Is Strong Enough for a Top Journal

The paper should defend a compact novelty package instead of many loosely connected features.

### Contribution A: Incremental semantic validation pipeline

Design and implement a pipeline where a new event triggers:

- delta extraction,
- affected-entity detection,
- selective SHACL validation,
- selective reasoning over the impacted subgraph,
- cached result reuse,
- and explicit validation provenance.

This is scientifically stronger than simply saying the system uses SHACL and OWL.

### Contribution B: Domain-general semantic contracts

Replace adapter-level manual validation with domain packages that include:

- ontology,
- SHACL shapes,
- domain event mapping,
- domain-specific constraints,
- and benchmark fixtures.

This turns the "generalization" claim into something testable.

### Contribution C: Explainable semantic provenance

Each validation or inference result should be able to report:

- triggering event,
- impacted entities,
- violated shape or satisfied shape,
- reasoning rule or ontology axiom used,
- and whether the result was reused from cache or computed fresh.

This is useful both academically and practically, and it materially strengthens the contribution story.

## What Should Be Removed or Downgraded in the Current Narrative

Until formal evidence exists, the paper should not claim:

- full GS1 EPCIS 2.0 compliance,
- full CBV coverage,
- full official conformance-suite passage,
- complete OWL2 reasoning across the whole system,
- or validated multi-domain generalization.

Replace those with evidence-backed claims such as:

- typed EPCIS-inspired event support for selected event classes,
- partial CBV support for demonstrated workflows,
- ontology-assisted validation and inference,
- WAL-backed durability for tested scenarios,
- and domain-adaptable architecture evaluated on multiple case-study domains.

## Required Code Direction

## 1. Unify the semantic execution path

**Goal:** make `src/ontology/*` plus the SPACL dependency the only publication path for reasoning and semantic validation.

### Actions

- Keep SPACL-backed `SimpleReasoner` as the authoritative reasoner path
- Deprecate legacy publication use of:
  - `src/semantic/owl_reasoner.rs`
  - `src/semantic/owl2_traceability.rs`
  - any placeholder-driven reasoning path
- Move all benchmarked semantic workflows behind a single service boundary
- Ensure the CLI and demos use the same publication path

### Deliverable

A single semantic pipeline that reviewers can trace from:

- event input
- to RDF materialization
- to SHACL validation
- to reasoning
- to blockchain commit

## 2. Build a typed event model instead of stringly typed builders

**Goal:** make event semantics explicit, testable, and extensible.

### Actions

- Replace the generic property map in `src/semantic/gs1_epcis.rs` with typed structs
- Add explicit support for the event classes actually used in experiments
- Separate:
  - internal canonical event model,
  - RDF/Turtle serializer,
  - JSON-LD serializer,
  - and domain extension mapping
- Add round-trip tests:
  - typed event -> RDF/JSON-LD -> parsed model

### Deliverable

A semantically explicit event layer that can support standards alignment without overclaiming conformance.

## 3. Implement incremental SHACL and reasoning

**Goal:** make semantic processing itself the novel algorithmic contribution.

### Actions

- Add a dependency graph between events, entities, and semantic constraints
- Track which nodes and constraints are affected by each new event
- Validate only the impacted shapes when possible
- Cache prior validation and inference results keyed by:
  - ontology version,
  - shape set hash,
  - entity or subgraph hash
- Add invalidation logic for cache changes after updates
- Emit an explanation object for each semantic decision

### Suggested module split

- `src/ontology/incremental.rs`
- `src/ontology/semantic_cache.rs`
- `src/ontology/explanations.rs`
- `src/ontology/event_projection.rs`

### Deliverable

A measurable incremental semantic engine with ablation-friendly architecture.

## 4. Replace manual domain adapters with semantic contracts

**Goal:** make cross-domain support real.

### Actions

- Redesign domain adapters to require a domain bundle:
  - ontology
  - SHACL shapes
  - event schema mapping
  - example dataset
  - benchmark fixture
- Remove adapter success criteria based only on required-property checks
- Ensure each domain has at least:
  - one valid workflow,
  - one invalid workflow,
  - and one domain-specific inference scenario

### Target domains

- Food/UHT as the anchor case
- Pharmaceutical as second domain
- Healthcare or manufacturing as third domain

### Deliverable

A domain package model that proves adaptation is semantic, not just structural.

## 5. Add failure-injection and durability validation

**Goal:** make durability claims evidence-based.

### Actions

- Add deterministic crash tests around WAL write and recovery boundaries
- Add disk pressure tests
- Add validator failure tests during semantic commit
- Add network disruption tests only if the consensus path being claimed is actually under evaluation

### Deliverable

A bounded and defensible reliability section with real fault-injection evidence.

## Experiment Design Required for a Top-Tier Submission

## A. Core research questions

The evaluation should explicitly answer:

1. Does incremental semantic validation reduce latency and recomputation cost?
2. Does the semantic contract model generalize across domains without rewriting core logic?
3. What semantic quality is gained relative to a non-semantic or non-incremental baseline?
4. What is the durability and recovery behavior under realistic failure conditions?

## B. Required baselines

At minimum compare:

1. Blockchain pipeline without SHACL or reasoning
2. Full semantic validation without incrementality
3. Incremental SHACL only
4. Incremental SHACL plus reasoning

If possible, also compare against:

5. A rule-based domain validator without ontologies

## C. Required ablations

Measure the impact of:

- cache on vs cache off
- delta validation on vs full-graph validation
- reasoning on vs off
- single-domain vs multi-domain configuration
- explanation generation on vs off

## D. Required datasets

Use three categories:

1. Synthetic controlled datasets for scale testing
2. Realistic domain datasets for ecological validity
3. Cross-domain datasets showing adaptation cost and semantic reuse

Minimum expectation:

- UHT production trace
- pharmaceutical batch trace
- healthcare equipment or clinical asset trace

## E. Metrics that matter

Primary metrics:

- event commit latency
- semantic validation latency
- incremental recomputation ratio
- cache hit rate
- invalid event detection precision and recall
- recovery time objective after forced interruption

Secondary metrics:

- memory usage
- RDF store growth
- explanation generation overhead
- domain onboarding effort

## F. Statistical quality bar

For publishable results:

- run repeated trials,
- report confidence intervals,
- report effect sizes where relevant,
- document hardware and environment precisely,
- and release raw benchmark data.

## Proposed Implementation Phases

## Phase 1: Submission rescue

**Objective:** stop overclaiming and establish a defensible artifact.

### Tasks

- align manuscript claims with current code reality
- choose the authoritative semantic path
- remove or downgrade unsupported standards claims
- clean reproducibility package placeholders

### Outcome

A defensible but narrower paper.

## Phase 2: Novelty implementation

**Objective:** create the actual research contribution.

### Tasks

- incremental validation engine
- semantic cache
- explanation model
- typed event model
- domain contract redesign

### Outcome

A system with clear method-level novelty.

## Phase 3: Empirical strengthening

**Objective:** produce reviewer-resistant evidence.

### Tasks

- multi-domain experiments
- ablation suite
- baseline comparisons
- fault-injection suite
- end-to-end reproducibility package

### Outcome

A top-journal-ready empirical section.

## Concrete File-Level Roadmap

## High priority refactors

- `src/semantic/gs1_epcis.rs`
  - replace loose property map with typed models
- `src/ontology/domain_manager.rs`
  - make this the main semantic orchestration path
- `src/ontology/shacl_validator.rs`
  - extend into incremental validator entry point
- `src/domain/adapters/healthcare.rs`
  - convert from manual checks to semantic contract bundle
- `src/domain/adapters/pharmaceutical.rs`
  - convert from manual checks to semantic contract bundle
- `src/storage/persistence.rs`
  - add failure-injection hooks and durability test seams

## Low priority or publication-deprecated paths

- `src/semantic/owl_reasoner.rs`
- `src/semantic/owl2_traceability.rs`

These may remain for backward compatibility or experimentation, but they should not be the basis of published claims unless fully reworked and experimentally validated.

## Paper Strategy

## Abstract should claim only what is evidenced

The abstract should say that ProvChain:

- introduces an incremental semantic validation architecture,
- supports ontology-driven domain adaptation,
- demonstrates the method across selected traceability domains,
- and quantifies the performance and quality tradeoffs through controlled evaluation.

It should not say:

- full EPCIS compliance,
- universal multi-domain readiness,
- or production-grade reasoning across all OWL2 features,

unless those are formally demonstrated.

## Discussion section should state the real limitation

The real limitation is not only throughput. It is that semantic richness usually increases execution cost, and the research contribution is the method used to reduce that cost while preserving semantic quality.

That is a much stronger top-journal framing.

## Acceptance Checklist

Before targeting a top journal, the project should satisfy all of the following:

- one authoritative semantic execution path
- no placeholder-driven reasoning in the published path
- typed event model for the evaluated workflow
- three-domain semantic contract evaluation
- incremental-vs-non-incremental ablation
- real fault-injection evidence for all reliability claims
- reproducibility package with valid URLs, commit SHA, scripts, and raw results
- manuscript text fully aligned with the actual artifact

## Immediate Next Steps

1. Reposition the manuscript around incremental semantic validation and domain contracts
2. Freeze a single authoritative reasoning and validation path for all future experiments
3. Implement a minimal incremental validation slice for one domain first
4. Add the first ablation benchmark comparing full vs incremental validation
5. Extend the same mechanism to two additional domains

## Recommended First Engineering Sprint

If only one sprint is available, do this:

1. Refactor `src/semantic/gs1_epcis.rs` into a typed internal event model
2. Route semantic validation through `src/ontology/domain_manager.rs`
3. Add an incremental validation cache layer in `src/ontology/`
4. Implement one UHT benchmark comparing:
   - no semantics
   - full validation
   - incremental validation
5. Rewrite the paper title, abstract, and evaluation section to match this new thesis

This is the shortest realistic path from the current repository to a submission that has a defensible top-journal story.
