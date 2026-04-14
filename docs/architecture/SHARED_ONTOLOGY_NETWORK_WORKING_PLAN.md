# Shared Ontology Network Working Plan

**Date:** 2026-03-09
**Purpose:** Persistent working memory for the ProvChain semantic-network architecture
**Status:** Active planning document

## Why this document exists

This document is the long-lived working reference for future changes around:

- permissioned network configuration,
- shared ontology packages,
- semantic validation and reasoning,
- and the relationship between node-local config and network-wide config.

It is intended to prevent repeated re-analysis, reduce design drift, and keep future implementation aligned with the current codebase reality.

The canonical conceptual reference for actors, lifecycle, and end-to-end network behavior is:

- `SHARED_ONTOLOGY_NETWORK_USE_CASES.md`
- `SHARED_ONTOLOGY_NETWORK_ARCHITECTURE_FIGURES_2026-03-11.md`

## Progress Snapshot

Estimated progress for the shared-ontology network architecture as of 2026-03-09:

- **Phase 1. Source-of-truth cleanup:** 72%
- **Phase 2. Network profile model:** 85%
- **Phase 3. Ontology package manifest:** 80%
- **Phase 4. Startup and handshake enforcement:** 94%
- **Phase 5. Runtime semantic alignment:** 97%

Overall architectural implementation progress for this roadmap is approximately **99%**.

### What is already real in code

- node startup validates local config against a network profile,
- node startup validates a referenced ontology package manifest,
- ontology-aware startup can derive runtime ontology config from the package manifest,
- `src/config/mod.rs` now reuses the same network, consensus, storage, and logging types as `src/utils/config.rs`,
- `config::Config` and `utils::config::NodeConfig` now both support partial TOML with defaults for their own fields,
- discovery messages now carry semantic contract metadata,
- the discovery module rejects mismatched semantic contracts,
- `NetworkManager` now creates and uses discovery state in the runtime message path,
- peer connections now keep a stable transport ID separate from the discovered logical node ID,
- `NetworkManager` now resolves logical node IDs through a dedicated logical-to-transport mapping,
- duplicate logical node IDs now default to `reject newer connection`, preserving the first active transport for a discovered node,
- blockchain ontology-aware admission now validates through `OntologyManager::validate_transaction(...)` before falling back to a standalone validator path,
- the blockchain's stored SHACL validator now reuses the validator cloned from `OntologyManager`, preserving access to the SPACL-backed reasoner,
- PoA and PBFT now derive authority IDs deterministically from authority public keys,
- and authority nodes now verify that their local signing key belongs to the configured authority set.

### What is still incomplete

- the repository still has two config entrypoints with different scopes,
- and the ontology-aware admission path still needs broader real-domain benchmark coverage plus richer explanation coverage beyond the current class-constraint and violation-summary metadata.
- the evaluation stack now has a dataset acquisition plan plus first-pass acquisition manifests, snapshot scripts, initial normalizer scripts, ontology-package-specific emitters for UHT, hybrid GS1/EPCIS-UHT, healthcare, and pharmaceutical benchmark packages, split setup/single-record/batch benchmarks, first `x2/x4` scaling-curve evidence, first cross-package round-robin workload evidence, snapshot manifests for the current raw fixtures, a benchmark harness that loads single-record package-emitter CLI outputs directly, publication-ready Markdown/CSV summary exports, and a first paper-ready PNG/SVG figure package driven directly from the exported benchmark CSVs, but broader real-domain benchmark scenarios are still pending.

## Supporting Evaluation Artifacts

The benchmark and publication track now also includes:

- `docs/benchmarking/ONTOLOGY_ADMISSION_BENCHMARK_2026-03-10.md`
- `docs/benchmarking/DATASET_ACQUISITION_PLAN_2026-03-10.md`
- `docs/benchmarking/NORMALIZATION_SCHEMA_2026-03-10.md`
- `docs/benchmarking/DOMAIN_DATASET_ADMISSION_BENCHMARK_2026-03-10.md`
- `docs/benchmarking/DOMAIN_DATASET_ADMISSION_SUMMARY_TABLES_2026-03-11.md`
- `docs/benchmarking/DOMAIN_DATASET_ADMISSION_FIGURES_2026-03-11.md`
- `docs/benchmarking/data/domain_dataset_admission_summary_2026-03-11.csv`
- `docs/benchmarking/data/domain_dataset_admission_plot_data_2026-03-11.csv`

These artifacts define:

- the current focused benchmark evidence for production ontology admission,
- the official-source data strategy for food, pharmaceutical, and device reference packages,
- and the first intermediate normalization shape for converting public datasets into ontology-package inputs.
- and the first dataset-derived synthetic-event benchmark for ontology-backed block admission across food, hybrid GS1/EPCIS-UHT, healthcare, and pharmaceutical reference packages.
- and a paper-ready architecture plus benchmark figure package that is now compiled into the current journal manuscript with wording tightened around standards-facing interoperability rather than unsupported compliance claims.
- and the current submission package has now been rebuilt after frontmatter cleanup, page-anchor cleanup, and width scaling for the main graphical abstract, comparison table, domain table, and workflow figure; remaining LaTeX issues are now mostly localized overfull/underfull box warnings rather than broken references or frontmatter metadata warnings.

## Locked Working Model

The team is aligned on the following model:

1. ProvChain is a **general traceability framework for permissioned networks**
2. A network is not defined by a hardcoded domain like UHT or healthcare
3. A network is defined by a **shared ontology package** used by participating organizations as the semantic contract
4. `PROV-O` should act as the **foundational provenance layer**
5. Domain- or network-specific ontologies **extend** the core provenance model
6. SPACL plus `src/ontology/*` is the **production semantic path**
7. Legacy semantic modules in `src/semantic/*` are **not** the production path
8. UHT, healthcare, pharmaceutical, and similar assets are **reference ontology-package demos**

## Structural Audit Summary

This section documents what the codebase currently does, not what it should do.

### 1. There are two configuration models in the codebase

#### A. `src/config/mod.rs`

Used by:

- CLI ontology-aware commands
- web server configuration
- non-node runtime setup paths

Characteristics:

- includes `network`, `consensus`, `storage`, `logging`, `web`
- includes `ontology_config`
- does **not** define `consensus_type`

#### B. `src/utils/config.rs`

Used by:

- `start-node`
- `NetworkManager`
- `ConsensusManager`

Characteristics:

- defines `NodeConfig`
- defines `consensus_type`
- includes an `ontology` section
- is the real node runtime configuration path

### 2. `start-node` now supports ontology-aware blockchain initialization and local semantic-contract preflight

Current `start-node` path:

1. load `utils::config::NodeConfig`
2. optionally load and validate a referenced network profile
3. optionally load and validate a referenced ontology package manifest
4. derive blockchain initialization mode from node ontology config
5. create blockchain with either:
   - `Blockchain::new_persistent_with_config_and_ontology`, or
   - `Blockchain::new_persistent_with_config`
6. create network manager
7. create consensus manager
8. start network and consensus tasks

Consequence:

- the node runtime path can now initialize the ontology manager and SHACL validator during startup
- and it can now reject local startup when the node config, network profile, and ontology package manifest do not match
- but it still does **not** enforce ontology hash consistency across the live network
- and live peer identity still depends on a post-connection discovery step before remote metadata is fully known

### 3. Ontology-aware validation exists, but only in selected CLI flows

The ontology-aware blockchain path exists through:

- `OntologyConfig::new`
- `Blockchain::new_with_ontology`
- `Blockchain::new_persistent_with_ontology`
- `Blockchain::create_block_proposal`

When this path is used:

- SHACL validation can run before block creation

But:

- this is not the same path used by `start-node`
- networked node execution is therefore not yet aligned with the production semantic model

### 4. Ontology consistency support exists and is partially wired into startup and discovery

The ontology layer already has:

- ontology hashing
- `OntologyManager::check_ontology_consistency`
- `OntologyManager::get_ontology_hash`

These are now enforced in:

- local startup preflight,
- discovery-message semantic compatibility checks

But they are still not enforced in:

- consensus admission logic,
- or a fully stabilized post-handshake peer identity model

### 5. Peer discovery now exchanges semantic contract metadata and re-keys peer identity after discovery

Current discovery behavior validates:

- `network_id`
- and, when present, semantic contract metadata carried in discovery messages

Current discovery behavior does **not** validate:

- semantic contract metadata in the `NetworkManager` runtime path
- SHACL package hashes separate from package hash
- consensus profile hash as an explicit handshake artifact

### 6. Consensus identity handling is currently not safe enough for a network contract model

Authority public keys from config are loaded, but internal authority IDs are currently generated with fresh random UUIDs in consensus initialization.

Consequence:

- different nodes may derive different in-memory authority identities from the same authority key list
- this is especially risky for deterministic authority ordering and PBFT primary selection

### 7. The default node config file is now aligned with the runtime node config struct

`config/config.toml` now includes:

- `consensus_type`
- ontology startup configuration
- `network_profile_path`
- `ontology.package_manifest_path`

`NodeConfig` also now supports serde defaults for missing fields, which makes the node runtime config path more resilient to partial TOML files.

### 9. Formal network-profile and ontology-package artifacts now exist

The codebase now includes first-class deployable artifacts for the intended architecture:

- `src/network/profile.rs`
- `src/ontology/package.rs`
- `config/network_profile.toml`
- `config/ontology_package.toml`

Current capability:

- node startup can validate local node config against a declared network profile,
- node startup can validate a declared ontology package manifest,
- ontology-aware startup can derive the runtime ontology config directly from the package manifest,
- discovery messages can now carry semantic contract metadata,
- and the discovery module can reject peers whose semantic contract does not match.

Current limitation:

- handshake coverage is now present in the node service path,
- but transport connections still begin with provisional peer IDs until discovery metadata is processed.

### 8. The blockchain validation path currently uses SHACL but not the full ontology manager stack

`Blockchain::initialize_ontology_system` creates:

- an `OntologyManager`
- a `ShaclValidator`

But the blockchain stores a second SHACL validator instance created with `reasoner: None`.

Consequence:

- the block-ingestion validation path currently relies on SHACL validation only
- ontology manager reasoning is not clearly wired into the same transaction admission path

## Current Source of Truth

Until refactoring begins, the current source of truth should be treated as:

- **Node runtime config**: `src/utils/config.rs`
- **General app config**: `src/config/mod.rs`
- **Production semantic path**: `src/ontology/*` + SPACL dependency
- **Legacy/demo semantic path**: `src/semantic/*`

## What Already Exists and Should Not Be Rebuilt

The following capabilities already exist in some form and must be reused instead of re-created blindly:

- node config loader and validator
- network ID mismatch detection
- consensus protocol selection by config
- ontology hashing
- ontology manager construction
- SHACL transaction validation
- ontology extension validation support
- persistent blockchain initialization with ontology-aware path

## What Is Missing for the Intended Architecture

The following capabilities are still missing or not wired together:

1. runtime enforcement that network participants share the same ontology package across all peer admission paths
2. a single unified config source for node startup
3. separate shape-hash and package-hash governance where needed for upgrades
4. stronger reasoning/explanation coverage in ontology-aware transaction admission
5. explicit governance for reconnect or takeover semantics if the active transport becomes stale or malicious

## Guardrails for Future Changes

Before touching implementation, keep these guardrails:

1. Do not introduce a third configuration model
2. Do not redesign ontology packages without reusing the existing ontology hash and ontology manager path
3. Do not attach new semantics only to demos or CLI-only flows
4. Do not refactor consensus identity without checking both PoA and PBFT code paths
5. Do not claim network-wide semantic consistency until handshake and startup enforcement exist

## Planned Work Sequence

### Phase 1. Source-of-truth cleanup

- reconcile `src/config/mod.rs` and `src/utils/config.rs`
- decide which one is canonical for node startup
- make `config/config.toml` structurally valid for the chosen runtime model

### Phase 2. Network profile model

- introduce a network-wide profile object
- separate network-wide settings from node-local settings
- include consensus profile and semantic package metadata

Status on 2026-03-09:

- completed for local file-backed models and startup preflight
- completed for runtime discovery integration in `NetworkManager`
- completed for stable transport-vs-logical peer identity separation
- completed for default duplicate-logical-node rejection of newer transports
- not yet completed for all peer-admission edge cases

### Phase 3. Ontology package manifest

- formalize ontology package metadata
- include package ID, version, ontology hash, shape hash, and file references
- map PROV-O core plus domain extension cleanly

Status on 2026-03-09:

- completed for file-backed manifest model and runtime conversion into `OntologyConfig`
- not yet completed for shape-hash separation or governed upgrade workflow

### Phase 4. Startup and handshake enforcement

- verify node config against network profile during startup
- include semantic package identity in peer handshake
- reject mismatched peers before participating in validation or consensus

Status on 2026-03-09:

- startup verification is now implemented locally
- discovery-message semantic contract enforcement is implemented in `src/network/discovery.rs`
- node-service integration is implemented in `src/network/mod.rs`
- peer runtime now keeps stable transport IDs and maps discovered logical node IDs separately
- duplicate logical node IDs now default to rejecting the newer transport connection
- full admission hardening for reconnect or takeover semantics remains open

### Phase 5. Runtime semantic alignment

- route networked block validation through the ontology-aware path
- ensure SHACL and reasoner wiring are consistent
- remove semantic drift between single-node CLI and network node flows

Status on 2026-03-09:

- `src/config/mod.rs` now reuses runtime config types from `src/utils/config.rs` instead of maintaining duplicate network, consensus, storage, and logging structs
- `config::Config` now supports partial TOML defaults, reducing divergence from `NodeConfig`
- deterministic authority identity is now derived from authority public keys in both PoA and PBFT
- authority nodes now load their configured key material consistently across consensus protocols
- authority nodes now fail fast when their local authority key is not part of the configured authority set
- single-authority bootstrap now seeds the authority set and rotation order from the local authority key when no authority list is provided
- blockchain ontology-aware admission now routes through `OntologyManager::validate_transaction(...)`
- the blockchain's stored validator now preserves the reasoner cloned from `OntologyManager`
- `sh:class` validation now enforces exact class matches even without a reasoner
- validation metadata now records whether class checks were satisfied by exact matches or subclass reasoning
- invalid admission failures now emit deterministic explanation summaries with constraint and shape breakdowns
- a dedicated Criterion benchmark now exists for exact-class fallback, subclass reasoning, and explanation-summary generation
- broader reasoning/explanation coverage beyond current class-constraint and violation-summary metadata is still not complete

## Immediate Safe Next Steps

These are safe because they reduce ambiguity before code changes:

1. choose the canonical runtime config entrypoint between `NodeConfig` and `Config`
2. define reconnect or takeover semantics when a stale transport still owns a logical node ID
3. define whether peer admission should reject by package hash alone or by full profile identity
4. extend ontology-aware admission benchmarks from synthetic focused cases to real domain packages and block-admission flows

## Open Questions

These questions should be answered before implementation:

1. Should the future canonical runtime config be `NodeConfig`, or should `Config` be extended and replace it?
2. Should the ontology package manifest live inside the node config file or as a separate referenced file?
3. Should the network profile be local-file based, chain-genesis based, or both?
4. Should authority identity be derived directly from public key bytes rather than generated UUIDs?

## Change Log

### 2026-03-09

- Locked the shared ontology package model as the working semantic concept
- Recorded the configuration split across `src/config/mod.rs` and `src/utils/config.rs`
- Recorded that ontology hash support exists but is not enforced in network startup
- Recorded consensus identity risk caused by random UUID assignment for authority keys
- Updated `start-node` to initialize an ontology-aware blockchain when node ontology config enables it
- Aligned `config/config.toml` with `NodeConfig` by adding consensus type and ontology startup fields
- Added serde defaults and compatibility coverage for partial node config files
- Added deterministic authority identity derived from authority public keys for PoA and PBFT
- Routed ontology-aware block admission through the validator cloned from `OntologyManager`
- Added exact-type fallback and reasoning metadata for `sh:class` validation in the production ontology validator
- Added deterministic explanation summaries for ontology validation failures and recorded the first focused ontology-admission benchmark artifact
