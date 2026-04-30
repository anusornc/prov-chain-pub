# Competitive Benchmark Fairness Matrix - 2026-04-17

## Purpose

This document defines how ProvChain should be benchmarked against currently relevant market products without producing misleading or academically weak comparisons.

It exists because ProvChain combines:

- a permissioned blockchain runtime,
- RDF and provenance-oriented storage,
- ontology-backed semantic admission,
- and standards-facing GS1/EPCIS-oriented event handling.

No single external product matches all of these layers. A fair comparison therefore requires multiple benchmark families instead of a single "winner" table.

## Current Decision

The benchmark comparison set for the next competitive evaluation round is:

- **ProvChain**
- **Hyperledger Fabric**
- **Go Ethereum (Geth)**
- **Neo4j**
- **Fluree**

`PostgreSQL` is intentionally excluded from the current competitive plan because the user requested an Ethereum baseline instead.

## Why These Products

### 1. Hyperledger Fabric

Role in the comparison:

- permissioned enterprise blockchain baseline

Why it belongs:

- closest widely recognized comparison target for permissioned ledger throughput and finality
- official documentation explicitly discusses performance tradeoffs for peers, orderers, channels, block cutting, and state database choices

Official source:

- Hyperledger Fabric performance considerations:
  - https://hyperledger-fabric.readthedocs.io/en/latest/performance.html

### 2. Go Ethereum (Geth)

Role in the comparison:

- public-chain execution baseline

Why it belongs:

- gives a real-world public blockchain reference point
- helps answer how ProvChain differs from a mainstream public-chain client in submission latency, confirmation latency, and operational cost

Important fairness caveat:

- Geth is not a permissioned enterprise ledger and should not be treated as a like-for-like substitute for Fabric or ProvChain
- it should be used as a public-chain baseline, not as the primary direct competitor for all metrics

Official source:

- Geth JSON-RPC documentation:
  - https://geth.ethereum.org/docs/interacting-with-geth/rpc

### 3. Neo4j

Role in the comparison:

- graph query and trace-reconstruction baseline

Why it belongs:

- strong comparison point for multi-hop provenance traversal and graph-style trace queries
- official docs expose explicit transaction and performance considerations

Official sources:

- Neo4j documentation:
  - https://neo4j.com/docs/
- Neo4j Query API transactions:
  - https://neo4j.com/docs/query-api/current/transactions/
- Neo4j Go driver performance recommendations:
  - https://neo4j.com/docs/go-manual/current/performance/

### 4. Fluree

Role in the comparison:

- RDF / JSON-LD / ledger-style semantic data baseline

Why it belongs:

- closer than Neo4j to ProvChain's RDF and JSON-LD data model
- supports transaction and query semantics that can be adapted to provenance-oriented workloads

Official sources:

- Fluree transaction syntax:
  - https://developers.flur.ee/docs/reference/transaction-syntax/
- Fluree transactions API:
  - https://next.developers.flur.ee/docs/reference/http-api/transactions/
- Fluree query foundations:
  - https://developers.flur.ee/docs/learn/foundations/querying/

## Fairness Principle

**Do not compare all products with a single raw TPS number and treat that as the result.**

That would be scientifically weak because:

- Fabric and ProvChain are permissioned blockchain systems,
- Geth is a public-chain client,
- Neo4j is a graph database,
- Fluree is a ledgered semantic database.

They solve overlapping but not identical problems.

## Required Benchmark Families

### Family A. Ledger / Write Path

Goal:

- compare ingestion and commit behavior for systems that accept transaction-like writes

Products:

- ProvChain
- Hyperledger Fabric
- Geth
- Fluree

Primary metrics:

- write throughput (`records/sec` or `tx/sec`)
- submit latency (`p50`, `p95`, `p99`)
- commit / confirmation latency
- batch-ingest behavior
- crash/recovery behavior where applicable

Do not include:

- Neo4j as a primary ledger baseline here

Reason:

- Neo4j is ACID and transactional, but it is not a ledger or consensus system

### Family B. Trace Query / Provenance Reconstruction

Goal:

- compare graph-style traceability and multi-hop provenance retrieval

Products:

- ProvChain
- Neo4j
- Fluree

Optional comparative note:

- Fabric and Geth may appear only if an off-chain index or query layer is explicitly added and measured as part of their stack

Primary metrics:

- single-entity lookup latency
- one-hop trace latency
- three-hop trace latency
- full provenance reconstruction latency
- aggregation latency over batches / lots
- result cardinality scaling behavior

### Family C. Semantic and Standards Layer

Goal:

- compare the cost of standards-facing data mapping and semantic validation

Products:

- ProvChain
- Fluree

Secondary / externalized baselines:

- Hyperledger Fabric with external validator pipeline
- Geth with external validator pipeline
- Neo4j with external validator pipeline

Primary metrics:

- JSON-LD / RDF ingest overhead
- SHACL validation overhead
- standards-facing EPCIS payload mapping cost
- explanation / validation-failure reporting cost

Critical fairness rule:

- if a product does not natively support SHACL or RDF semantics, that cost must be counted as an externalized pipeline stage, not ignored

## Common Dataset and Workload Rules

All benchmark families must use the same logical workload definitions.

### Dataset rules

- one canonical traceability dataset family
- one canonical field mapping
- one canonical event set
- one canonical trace-query set

Recommended workload slices:

1. **UHT case-study slice**
   - direct continuity with the current paper
2. **Hybrid GS1/EPCIS-UHT slice**
   - standards-facing event layer
3. **Healthcare-device slice**
   - non-food reference package
4. **Pharmaceutical-storage slice**
   - non-food reference package

### Input normalization rules

- generate comparable logical records first
- then adapt those records into each system's native write format
- keep a documented transform layer per target system

This prevents hidden data-shape bias from becoming a benchmark artifact.

## Metric Matrix

| Metric | ProvChain | Fabric | Geth | Neo4j | Fluree | Fair Use |
|---|---|---|---|---|---|---|
| Write throughput | yes | yes | yes | yes, secondary | yes | compare within benchmark family context only |
| Commit/finality latency | yes | yes | yes | no consensus finality | ledger-style commit only | do not flatten into one cross-family "winner" metric |
| Trace query latency | yes | external/indexed only | external/indexed only | yes | yes | core metric for query family |
| RDF/JSON-LD native ingest | yes | no | no | partial via import pipeline | yes | count external transform cost where not native |
| SHACL validation | yes | external only | external only | external only | external only unless implemented | must not be omitted when comparing semantic workloads |
| Standards-facing EPCIS mapping | yes | external mapping | external mapping | external mapping | yes, via JSON-LD/RDF path | compare as full pipeline cost |
| Storage footprint | yes | yes | yes | yes | yes | normalize per logical record |
| Operational complexity | yes | yes | yes | yes | yes | describe qualitatively plus service count / deployment count |

## Rules for a Scientifically Defensible Comparison

1. **Same hardware**

- same CPU class
- same RAM limit
- same storage class
- same local / network deployment assumptions

2. **Same workload**

- same logical records
- same batch sizes
- same query templates

3. **Explicit consistency model**

- state clearly whether a metric reflects:
  - accepted write
  - committed write
  - block finality
  - public-chain confirmation depth

4. **Externalized semantic costs must be counted**

If Fabric, Geth, or Neo4j need external RDF conversion or SHACL validation, benchmark that full path explicitly.

5. **No vendor-marketing numbers**

Do not compare ProvChain measurements to unverified vendor TPS claims.

6. **No simulated competitor numbers in paper evidence**

The existing simulated competitor utilities under:

- `tests/utils/enhanced_competitive_benchmarks.rs`

must not be used as paper evidence.

## Recommended Benchmark Outputs

Each benchmark family should produce:

1. machine-readable CSV
2. raw run logs
3. environment manifest
4. exact commands used
5. one Markdown artifact summarizing:
   - hardware
   - software versions
   - workload definition
   - limitations

## Execution Plan

### Phase 1. Fairness Harness Design

Create:

- a benchmark spec document
- target-product adapters
- dataset-to-target transformation definitions

Deliverables:

- benchmark family definitions
- input schemas
- system-adapter contracts

### Phase 2. Trace Query Baseline

Implement first:

- ProvChain
- Neo4j
- Fluree

Reason:

- this is the cleanest near-term comparison because the query problem is better aligned than ledger finality across products

### Phase 3. Ledger Baseline

Implement next:

- ProvChain
- Hyperledger Fabric
- Geth
- Fluree

Reason:

- write-path comparison is useful, but only after commit/finality semantics are explicitly normalized

### Phase 4. Semantic Pipeline Costing

Measure:

- standards-facing event mapping
- RDF / JSON-LD conversion
- SHACL validation
- explanation generation

Reason:

- this is where ProvChain's differentiator is strongest
- it must be measured honestly rather than hand-waved

## Recommendation for the Paper

For the current manuscript:

- do **not** claim direct benchmark superiority over Fabric, Geth, Neo4j, or Fluree
- keep current external-system discussion as literature or positioning context only

For the next paper or major revision:

- use this fairness matrix to produce a new artifact-backed competitive benchmark suite

## Immediate Next Step

The next implementation artifact should be:

- a `benchmark specification` document that freezes:
  - products,
  - benchmark families,
  - metrics,
  - dataset slices,
  - and fairness rules.

Only after that should execution harness code be written.
