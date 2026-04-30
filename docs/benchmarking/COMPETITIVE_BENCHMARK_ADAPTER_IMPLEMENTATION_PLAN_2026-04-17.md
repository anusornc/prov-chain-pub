# Competitive Benchmark Adapter Implementation Plan - 2026-04-17

## Purpose

This document translates the benchmark specification into an implementation plan for the adapter layer and runner architecture.

It is the next step after:

- `docs/benchmarking/COMPETITIVE_BENCHMARK_FAIRNESS_MATRIX_2026-04-17.md`
- `docs/benchmarking/COMPETITIVE_BENCHMARK_SPEC_2026-04-17.md`

## Current Toolkit Reality

The current `benchmark-toolkit` is useful, but not yet organized as a general competitive benchmark platform.

### What already exists

- a benchmark runner under `benchmark-toolkit/research-benchmarks/src/main.rs`
- a concrete `Neo4j` client under:
  - `benchmark-toolkit/research-benchmarks/src/neo4j_client.rs`
- Docker and orchestration assets oriented around ProvChain + Neo4j

Legacy note:

- `benchmark-toolkit/research-benchmarks/src/jena_client.rs` still exists as a legacy engineering baseline
- it is no longer part of the active competitive benchmark runner path

### What does not yet exist

- no dedicated `provchain_client.rs` adapter module in `benchmark-toolkit/research-benchmarks/src/`
- no adapter modules for:
  - Hyperledger Fabric
  - Geth
  - Fluree
- no shared adapter trait for all systems
- no shared workload schema split by benchmark family
- no result schema that distinguishes:
  - write submit
  - durable commit
  - finality / confirmation
  - trace query
  - semantic admission

## Design Goal

Refactor the benchmark layer so that all target systems implement the same adapter contract for the workloads they actually support.

This must prevent:

- forcing unlike systems into one fake metric
- hiding external semantic costs
- mixing query, ledger, and semantic workloads without explicit labels

## Proposed Architecture

### 1. Shared Core Layer

Create a shared benchmark core responsible for:

- loading workload definitions
- loading dataset slices
- dispatching workloads to target adapters
- timing execution
- collecting metadata
- writing machine-readable outputs

Recommended location:

- `benchmark-toolkit/research-benchmarks/src/core/`

Proposed files:

- `adapter.rs`
- `workload.rs`
- `dataset.rs`
- `result.rs`
- `runner.rs`
- `environment.rs`

### 2. Target Adapter Layer

Each product should implement one or more adapters depending on supported benchmark families.

Recommended location:

- `benchmark-toolkit/research-benchmarks/src/adapters/`

Proposed files:

- `provchain.rs`
- `neo4j.rs`
- `fabric.rs`
- `geth.rs`
- `fluree.rs`

### 3. Workload Family Modules

Benchmark families should be explicit in code.

Recommended location:

- `benchmark-toolkit/research-benchmarks/src/workloads/`

Proposed files:

- `ledger.rs`
- `trace_query.rs`
- `semantic.rs`

## Adapter Contract

Define explicit capabilities instead of pretending every system supports every workload natively.

### Proposed trait shape

```rust
pub trait BenchmarkAdapter {
    fn system_name(&self) -> &'static str;
    fn capabilities(&self) -> AdapterCapabilities;
}
```

Capability structure:

- `supports_ledger_write`
- `supports_trace_query`
- `supports_semantic_pipeline`
- `supports_native_rdf`
- `supports_native_jsonld`
- `supports_native_shacl`
- `supports_finality_measurement`

Then use narrower traits per family:

```rust
pub trait LedgerWriteAdapter {
    async fn submit_one(&mut self, record: LogicalRecord) -> Result<WriteResult>;
    async fn submit_batch(&mut self, batch: Vec<LogicalRecord>) -> Result<BatchWriteResult>;
}

pub trait TraceQueryAdapter {
    async fn entity_lookup(&mut self, id: &str) -> Result<QueryResult>;
    async fn trace_one_hop(&mut self, id: &str) -> Result<QueryResult>;
    async fn trace_three_hop(&mut self, id: &str) -> Result<QueryResult>;
    async fn full_provenance(&mut self, id: &str) -> Result<QueryResult>;
}

pub trait SemanticPipelineAdapter {
    async fn semantic_admission(&mut self, record: LogicalRecord) -> Result<SemanticResult>;
}
```

## Shared Data Model

### LogicalRecord

This must be the canonical workload record before target-specific translation.

Required fields:

- `record_id`
- `slice_id`
- `entity_type`
- `timestamp`
- `source_payload_kind`
- `payload`
- `semantic_profile`

### QueryScenario

Required fields:

- `scenario_id`
- `family`
- `slice_id`
- `lookup_key`
- `expected_min_results`

### Result Types

Must distinguish:

- `submit_latency_ms`
- `commit_latency_ms`
- `confirmation_latency_ms`
- `query_latency_ms`
- `validation_latency_ms`
- `mapping_latency_ms`
- `explanation_latency_ms`

Do not collapse them into a single `duration_ms` field without context.

## Product-Specific Adapter Plans

### ProvChain Adapter

Status:

- now exists as `benchmark-toolkit/research-benchmarks/src/adapters/provchain.rs`
- active for health check, data loading, write batch, and trace-query paths

Required implementation:

1. complete the remaining extraction of current ProvChain request logic from `main.rs`
2. extend the adapter for all benchmark families we keep
3. support:
   - ledger writes
   - trace queries
   - semantic admission

Priority:

- highest

### Neo4j Adapter

Status:

- adapter now exists as `benchmark-toolkit/research-benchmarks/src/adapters/neo4j.rs`
- active for data-loading and trace-query paths

Required implementation:

1. complete the refactor boundary around the existing `neo4j_client.rs`
2. expose:
   - entity lookup
   - one-hop trace
   - three-hop trace
   - full provenance reconstruction
   - optional write-path baseline

Priority:

- highest

### Hyperledger Fabric Adapter

Status:

- initial adapter scaffold now exists at:
  - `benchmark-toolkit/research-benchmarks/src/adapters/fabric.rs`
- currently limited to capability declaration plus health-check foothold
- not yet wired into write/query benchmark execution

Required implementation:

1. define deployment strategy:
   - Docker Compose or external network
2. choose client approach:
   - Fabric Gateway client
3. choose chaincode language:
   - Go
4. define minimal ledger data model for benchmark workloads
5. expose:
   - single write
   - batch write
   - optional query endpoints or documented off-chain query layer

Important design constraint:

- benchmark must record whether query behavior is on-chain, chaincode-mediated, or off-chain indexed

Priority:

- medium-high

### Geth Adapter

Status:

- initial adapter scaffold now exists at:
  - `benchmark-toolkit/research-benchmarks/src/adapters/geth.rs`
- currently limited to capability declaration plus RPC health-check foothold
- not yet wired into write/query benchmark execution

Required implementation:

1. define network mode:
   - local dev chain
   - fixed mining / sealing configuration
2. define write submission:
   - JSON-RPC transaction submission
3. define confirmation measurement:
   - transaction receipt + confirmation depth
4. define external query/index strategy for provenance workloads if included

Important design constraint:

- public-chain confirmation must never be mislabeled as permissioned finality

Priority:

- medium

### Fluree Adapter

Status:

- initial adapter scaffold now exists at:
  - `benchmark-toolkit/research-benchmarks/src/adapters/fluree.rs`
- not yet wired into the main runner or result exports

Required implementation:

1. define transaction endpoint integration
2. define query endpoint integration
3. define JSON-LD ingest path
4. define trace workload query mappings
5. define semantic-family role:
   - native JSON-LD ingest
   - query family
   - optional semantic-family baseline

Priority:

- medium-high

## Implementation Order

### Phase 1. Internal Refactor

Goal:

- extract current toolkit into reusable core + adapters

Tasks:

1. create `core/` module
2. create `adapters/` module
3. move existing Neo4j logic behind trait boundaries
4. extract ProvChain logic from runner into adapter module

Status:

- completed for the first trace/data-load/write milestone
- shared adapter core exists
- ProvChain and Neo4j adapters exist
- Jena has been removed from the active runner path

### Phase 2. First Executable Family

Goal:

- artifact-backed `ProvChain vs Neo4j` trace-query suite

Reason:

- lowest ambiguity
- highest reuse from current toolkit

Deliverables:

- runnable trace-query workloads
- output schema
- updated result exports

### Phase 3. Fluree Integration

Goal:

- add semantic-friendly comparative baseline

Reason:

- closest target to ProvChain's RDF/JSON-LD semantics

Status:

- partially landed
- Fluree adapter exists and is now wired into:
  - health checks
  - trace-query execution path
  - conditional JSON-LD data-loading path

### Phase 4. Fabric Integration

Goal:

- add permissioned-ledger competitor

Reason:

- strongest market-relevant ledger comparison

### Phase 5. Geth Integration

Goal:

- add public-chain baseline

Reason:

- useful contrast, but less like-for-like than Fabric

## Orchestration Plan

The runner should stop using ad hoc flags tied to one comparison pair.

Instead, it should accept:

- benchmark family
- dataset slice
- target system list
- iteration count
- environment manifest path

Proposed CLI examples:

```bash
benchmark-runner family trace-query --slice uht --targets provchain,neo4j
benchmark-runner family ledger --slice hybrid-epcis-uht --targets provchain,fabric,geth,fluree
benchmark-runner family semantic --slice hybrid-epcis-uht --targets provchain,fluree
```

Current implementation status:

- CLI/system wiring now includes:
  - ProvChain
  - Neo4j
  - Fluree
  - Fabric health-check scaffold
  - Geth health-check scaffold
- report generation is now generic across multiple systems instead of hardcoding only ProvChain vs Neo4j

## Result Schema Plan

Current result records are too generic for fair multi-family comparison.

Status:

- now partially implemented in `benchmark-toolkit/research-benchmarks/src/core/result.rs`
- shared result records now carry:
  - `family`
  - `metric_type`
  - `unit`
- `main.rs` now consumes the shared result schema instead of declaring its own local result/summary structs

Create a schema with:

- `family`
- `slice`
- `system`
- `scenario`
- `metric_type`
- `value`
- `unit`
- `iteration`
- `success`
- `metadata`

This avoids mixing query latency and commit latency in one undifferentiated result field.

## Risks

### Risk 1. False apples-to-apples comparison

Mitigation:

- preserve family-specific reporting

### Risk 2. Hidden external semantic cost

Mitigation:

- benchmark semantic stages explicitly

### Risk 3. Public-chain confirmation ambiguity

Mitigation:

- separate `submit`, `receipt`, and `confirmation depth` timings

### Risk 4. Benchmark-toolkit scope creep

Mitigation:

- land `ProvChain + Neo4j` trace family first before adding more systems

## Immediate Next Coding Task

The next coding milestone should be:

1. harden the first artifact-backed multi-target trace family around `ProvChain + Neo4j + Fluree`
2. add real ledger-execution paths for Fabric and Geth beyond health-check scaffolds
3. extend dataset translation so Fluree/Fabric/Geth can ingest the same logical slices fairly

Only after that should Fabric and Geth move from health-check scaffolds into real ledger benchmark execution.
