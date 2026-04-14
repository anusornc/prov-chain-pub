# Shared Ontology Network Use Cases

**Date:** 2026-03-09
**Purpose:** Canonical use-case reference for the ProvChain shared-ontology network model
**Status:** Active reference

## Why this document exists

This document records the agreed operational view of ProvChain so future code, experiments, and publication claims stay aligned.

It complements:

- `ADR 0014`, which records the production semantic-path decision,
- `SHARED_ONTOLOGY_NETWORK_WORKING_PLAN.md`, which records the current codebase truth and phased implementation plan.

This document answers four practical questions:

1. who the system actors are,
2. how a network is expected to operate over time,
3. what the end-to-end use cases look like,
4. and where the boundary lies between the core platform and demo domain packages.

## Canonical system view

ProvChain is a **permissioned traceability network** in which participating organizations share a common ontology package as the semantic contract for provenance capture, validation, and traceability queries.

The intended semantic stack is:

- `PROV-O` as the foundational provenance layer,
- network-specific ontology extensions for business semantics,
- optional mappings to external standards such as GS1/EPCIS where needed,
- SPACL-backed validation and reasoning through `src/ontology/*` as the production path.

The intended network stack is:

- a network-wide consensus and semantic contract shared by all members,
- node-local runtime configuration for deployment-specific settings,
- deterministic validation behavior across all nodes in the same network.

## Core actors

### 1. Network authority or consortium administrator

Responsible for establishing and governing a permissioned network.

Typical responsibilities:

- define the network profile,
- choose the consensus protocol,
- approve validator membership,
- approve the ontology package used by the network,
- coordinate upgrades to network or semantic policy.

### 2. Validator organization

An organization that runs a validator node and participates in consensus.

Typical responsibilities:

- run node infrastructure,
- validate incoming transactions and semantic constraints,
- participate in block production and agreement,
- reject peers or data that do not match the network contract.

### 3. Participant organization

An organization that submits traceability events and queries provenance data.

Typical responsibilities:

- produce domain events,
- map local business data to the shared ontology package,
- query traceability and audit history,
- consume proofs or query results for compliance and operations.

### 4. Ontology package maintainer

Responsible for the semantic contract used by the network.

Typical responsibilities:

- maintain the ontology package manifest,
- evolve ontology and SHACL shapes,
- manage compatibility rules and versioning,
- provide fixtures, test data, and migration notes.

### 5. Auditor, regulator, or external reviewer

Consumes evidence from the network.

Typical responsibilities:

- verify provenance history,
- inspect rule conformance,
- review semantic and operational consistency,
- validate cross-organization traceability claims.

## Shared artifacts

The system depends on the following shared artifacts.

### 1. Network profile

This is the network-wide contract that all nodes in the same permissioned network must match.

Expected contents:

- `network_id`,
- consensus type and parameters,
- validator or authority set,
- block-production timing rules,
- semantic package identity and version,
- semantic package hashes or compatibility identifiers,
- policy flags that affect validation behavior.

### 2. Ontology package

This is the semantic contract shared across organizations in the network.

Expected contents:

- core provenance ontology reference,
- network-specific ontology extensions,
- SHACL shapes,
- optional mappings to standards such as GS1/EPCIS,
- package identifier,
- package version,
- content hash or manifest hash,
- example data and conformance fixtures.

### 3. Node-local configuration

This is local deployment configuration and must not be confused with the network-wide contract.

Typical contents:

- node identity,
- listen address and port,
- local storage paths,
- local key paths,
- local logging and runtime settings.

## Lifecycle of a shared-ontology network

### Phase 1. Network bootstrap

A consortium or authority group creates a new permissioned network.

They agree on:

- the consensus mode, such as PoA or PBFT,
- the validator membership model,
- the initial ontology package,
- and the initial network profile.

At this point, the ontology package defines the semantic meaning of traceability events for that network.

### Phase 2. Node provisioning

Each organization configures and starts its node.

Before joining, a node is expected to verify that:

- its local runtime config is valid,
- its network profile matches the intended network,
- its ontology package identity and hash match the network contract,
- and its consensus settings are compatible with the other nodes.

### Phase 3. Event onboarding

Organizations submit traceability events to the network.

Those events are expected to be expressed in terms that can be interpreted through:

- the shared provenance core,
- the network ontology extension,
- and any network-approved mappings to external standards.

### Phase 4. Semantic validation and block admission

Before acceptance into the chain, an event is validated against the network's semantic contract.

Expected validation scope:

- structural conformance,
- relation and class consistency,
- required provenance links,
- domain-specific shape constraints,
- rule-based semantic checks supported by the production semantic path.

Only valid events should proceed to consensus and become committed blocks.

### Phase 5. Traceability query and audit

Committed events form a provenance graph that can be queried and audited.

Users should be able to:

- trace an entity, batch, or process across organizations,
- reconstruct event history,
- inspect conformance violations or exceptions,
- and produce audit evidence from immutable provenance records.

### Phase 6. Semantic evolution

The network may eventually need to evolve its ontology package.

Examples:

- new event classes,
- new constraints,
- updated mappings to industry standards,
- compatibility fixes across participating organizations.

These changes must be governed as network-level changes, not ad hoc local node changes.

## End-to-end reference use cases

### Use case 1. Bootstrap a new traceability network

**Goal:** create a new permissioned network that all organizations can interpret consistently.

**Flow:**

1. The consortium defines a network profile.
2. The consortium approves an ontology package.
3. Validator organizations receive the approved profile and ontology package.
4. Each validator provisions keys and local node config.
5. Each validator starts a node and checks compatibility with the network profile.
6. The network begins operation only after compatible validators join.

### Use case 2. Onboard a new organization into an existing network

**Goal:** let a new organization participate without breaking semantic interoperability.

**Flow:**

1. The organization receives the current network profile and ontology package.
2. The organization maps its local data model to the shared ontology package.
3. The organization configures and starts its node or client integration.
4. The node verifies network and semantic compatibility before joining.
5. The organization begins submitting traceability events under the shared contract.

### Use case 3. Submit a traceability event

**Goal:** commit a business event as a semantically valid provenance record.

**Flow:**

1. A participant generates a business event from its local system.
2. The event is transformed into the network's shared semantic model.
3. The node validates the event against the ontology package and SHACL constraints.
4. If validation passes, the event enters block admission and consensus.
5. A committed block stores immutable provenance for the event.
6. Other participants can now query or audit the event.

### Use case 4. Query end-to-end provenance

**Goal:** reconstruct the history of an asset, batch, or process across organizations.

**Flow:**

1. A user requests a traceability query.
2. The query resolves over blockchain-backed provenance data.
3. The platform follows semantic links defined by the shared ontology package.
4. The user receives a multi-step provenance history with supporting evidence.

### Use case 5. Support a domain-specific deployment

**Goal:** apply the same core platform to a new domain without hardcoding the core system for that domain.

**Flow:**

1. A domain team prepares a network ontology package that extends the PROV-O core.
2. The package adds domain classes, relations, shapes, and optional external-standard mappings.
3. The network adopts that package through governance.
4. The same blockchain core and consensus stack are reused.
5. Only the semantic package and domain mapping layer change.

This is the intended generalization story for UHT, healthcare, pharmaceutical, and future domains.

### Use case 6. Upgrade a network ontology package

**Goal:** evolve semantics without fragmenting the network.

**Flow:**

1. The ontology package maintainer prepares a new package version.
2. Compatibility and migration rules are reviewed.
3. The consortium approves the upgrade.
4. Nodes adopt the new semantic package under the agreed rollout policy.
5. Nodes that do not match the required semantic contract should be rejected or isolated.

## Reference examples

### Example A. UHT supply-chain network

The UHT use case is a reference deployment in which:

- producers,
- processors,
- logistics providers,
- distributors,
- and retailers

share one ontology package for milk-batch provenance.

This demonstrates cross-organization traceability, not a UHT-only platform.

### Example B. Pharmaceutical network

The same core platform can support medicine-batch traceability if the network adopts a pharmaceutical ontology package that extends the provenance core and adds the required domain constraints.

### Example C. Healthcare network

The same platform can support healthcare traceability if the network adopts a healthcare ontology package with appropriate patient-safety, custody, or process semantics.

## Boundary of the core platform

The core platform should remain domain-general.

### The core platform is responsible for

- block creation and persistence,
- consensus and network participation,
- shared-ontology-aware validation path,
- provenance storage and query support,
- network and semantic contract enforcement,
- interoperability hooks for domain packages.

### Domain or demo packages are responsible for

- domain-specific ontology extensions,
- domain-specific SHACL shapes,
- mappings from local data models to the shared semantic model,
- sample datasets and demos,
- domain fixtures used for tests and demonstrations.

### What must not happen

The platform must not require hardcoded UHT-, healthcare-, or pharmaceutical-specific logic in order to be considered correct.

Those domains are reference demonstrations of the platform model, not the architectural boundary of the product.

## Boundary of PROV-O in the architecture

`PROV-O` should be treated as the foundational provenance layer, not as the full domain model.

It should provide the generic semantics needed to express:

- entities,
- activities,
- agents,
- generation,
- usage,
- derivation,
- attribution,
- and temporal relationships.

Network-specific ontologies then extend that foundation with domain vocabulary and constraints.

This means:

- `PROV-O` is the semantic backbone,
- network extensions provide business meaning,
- and external standards such as GS1 are optional mapping layers when required by a given network.

## Operational implications for implementation

The agreed use-case model implies the following implementation priorities:

1. production startup must use the ontology-aware path,
2. network participation must verify semantic contract compatibility,
3. network and node configuration must be separated cleanly,
4. ontology packages must become first-class deployable artifacts,
5. demo domains must remain outside the core-platform correctness boundary.

## Non-goals

The agreed model does not imply:

- a single hardcoded ontology for all possible industries,
- full dependence on GS1 for every deployment,
- domain-specific hardcoding in the blockchain core,
- or continued reliance on legacy semantic modules as the production path.

## Relationship to current code reality

This document describes the intended operating model.

The structural gaps between this intended model and the current implementation are tracked separately in:

- `SHARED_ONTOLOGY_NETWORK_WORKING_PLAN.md`
- `docs/reviews/STRUCTURAL_AUDIT_NODE_CONFIG_ONTOLOGY_2026-03-09.md`

Use this document as the conceptual reference and those documents as the implementation-gap reference.
