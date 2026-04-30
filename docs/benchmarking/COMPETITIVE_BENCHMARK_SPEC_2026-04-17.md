# Competitive Benchmark Specification - 2026-04-17

## Purpose

This document freezes the benchmark design for the next competitive evaluation round.

It translates the fairness rules from:

- `docs/benchmarking/COMPETITIVE_BENCHMARK_FAIRNESS_MATRIX_2026-04-17.md`

into an executable specification for future harness and adapter work.

## Current Reality

The repository currently has:

- artifact-backed benchmark suites for ProvChain itself
- a public benchmark pipeline for UHT, hybrid GS1/EPCIS-UHT, healthcare-device, and pharmaceutical-storage package profiles
- a portable `benchmark-toolkit` that currently centers on ProvChain vs Neo4j
- simulated competitor utilities under `tests/utils/enhanced_competitive_benchmarks.rs`, which are **not valid paper evidence**

The repository does **not** currently have artifact-backed real benchmark adapters for:

- Hyperledger Fabric
- Geth
- Fluree

Therefore this document is a design freeze, not a claim that the new cross-product benchmark suite already exists.

## Benchmark Targets

The approved target set is:

1. `ProvChain`
2. `Hyperledger Fabric`
3. `Go Ethereum (Geth)`
4. `Neo4j`
5. `Fluree`

## Benchmark Families

The benchmark suite is divided into three families.

### Family A. Ledger / Write Path

Products:

- ProvChain
- Hyperledger Fabric
- Geth
- Fluree

Goal:

- compare write-path cost and commit behavior for transaction-bearing systems

Primary metrics:

- write throughput (`records/sec`, `tx/sec`)
- submit latency (`p50`, `p95`, `p99`)
- commit latency / confirmation latency
- batch commit efficiency
- crash / restart recovery time where measurable

### Family B. Trace Query / Provenance Reconstruction

Products:

- ProvChain
- Neo4j
- Fluree

Optional:

- Fabric and Geth only if a documented external query/index layer is explicitly benchmarked as part of the target stack

Goal:

- compare graph-oriented provenance reconstruction and traceability queries

Primary metrics:

- single-batch lookup latency
- one-hop trace latency
- three-hop trace latency
- full provenance reconstruction latency
- aggregate query latency
- query throughput under repeated workloads

### Family C. Semantic / Standards Layer

Products:

- ProvChain
- Fluree

Externalized baselines:

- Fabric + external semantic pipeline
- Geth + external semantic pipeline
- Neo4j + external semantic pipeline

Goal:

- measure the cost of standards-facing event mapping and semantic validation honestly

Primary metrics:

- payload transformation time
- JSON-LD / RDF ingestion time
- SHACL validation time
- explanation-generation time
- end-to-end semantic admission time

## Dataset Slices

The benchmark suite will use four logical slices derived from the current reference-package strategy.

### Slice 1. UHT Case Study

Use:

- continuity with the current paper
- direct package fit with the strongest current artifact chain

Representative records:

- UHT processing batches
- packaging
- storage and shipment

### Slice 2. Hybrid GS1/EPCIS-UHT

Use:

- standards-facing event modeling benchmark slice

Representative records:

- EPCIS-style object-event payloads
- CBV-aligned business steps and dispositions

### Slice 3. Healthcare Device

Use:

- non-food traceability package

Representative records:

- device identity
- handling / movement events
- audit-relevant provenance lookups

### Slice 4. Pharmaceutical Storage

Use:

- non-food, storage-sensitive package

Representative records:

- storage condition events
- transfer / custody changes
- package-level provenance reconstruction

## Canonical Workloads

Each product must be tested against equivalent logical workloads, even if physical query or transaction syntax differs.

### Workload W1. Single Record Insert

Description:

- insert one logical record / event

Applies to:

- all families where writes exist

Metrics:

- `submit_latency_ms`
- `commit_latency_ms`

### Workload W2. Batch Insert

Description:

- insert logical batches of size `10`, `100`, `1000`

Metrics:

- `throughput_records_per_sec`
- `throughput_tx_per_sec`
- `batch_commit_latency_ms`

### Workload W3. Single-Entity Lookup

Description:

- retrieve one known entity by identifier

Metrics:

- `query_p50_ms`
- `query_p95_ms`

### Workload W4. One-Hop Trace

Description:

- reconstruct direct predecessor/successor information

Metrics:

- `query_p50_ms`
- `query_p95_ms`
- `result_count`

### Workload W5. Three-Hop Trace

Description:

- reconstruct multi-hop chain from a known batch/device/package

Metrics:

- `query_p50_ms`
- `query_p95_ms`
- `result_count`

### Workload W6. Full Provenance Reconstruction

Description:

- reconstruct all current known lineage edges for one entity or batch

Metrics:

- `query_p50_ms`
- `query_p95_ms`
- `records_scanned`

### Workload W7. Semantic Admission

Description:

- map a standards-facing logical record into target representation and validate it

Metrics:

- `mapping_latency_ms`
- `semantic_validation_latency_ms`
- `explanation_latency_ms`
- `end_to_end_admission_latency_ms`

## Fairness Constraints

### 1. Hardware Lock

Each benchmark run must record:

- CPU model
- core count
- RAM
- storage type
- OS and kernel
- container vs native execution

No result is comparable without this manifest.

### 2. Dataset Lock

Each run must record:

- dataset slice
- record count
- serialization format
- transformation path

### 3. Consistency Lock

Each write metric must declare whether it measures:

- accepted request
- durable local commit
- permissioned finality
- public-chain confirmation depth

### 4. Semantic Cost Lock

If a product lacks native RDF / SHACL support:

- the external mapping/validation stage must be benchmarked and counted

### 5. No Vendor Numbers

Only artifact-backed measurements from our harness are allowed in future competitive evidence.

## Output Artifacts

Each benchmark run must generate:

1. `environment_manifest.json`
2. `benchmark_results.json`
3. `benchmark_results.csv`
4. `raw_logs/`
5. `summary.md`

Optional:

6. `plots/`
7. `notebook/` or `analysis script`

## Adapter Requirements

### ProvChain Adapter

Must support:

- current ontology-package ingestion path
- current trace query path
- current semantic admission path

### Hyperledger Fabric Adapter

Must support:

- permissioned write transactions
- query pattern matching through chaincode/query endpoint or documented off-chain query layer

Important note:

- Go chaincode should be preferred for performance-sensitive measurements according to official Fabric guidance

### Geth Adapter

Must support:

- JSON-RPC write submission
- transaction receipt polling / confirmation timing

Important note:

- public-chain confirmation semantics must be explicitly separated from permissioned finality semantics

### Neo4j Adapter

Must support:

- transactional writes
- Cypher-based query workloads

Important note:

- query runs should specify the target database explicitly and avoid driver-induced overhead ambiguity

### Fluree Adapter

Must support:

- transaction endpoint writes
- query endpoint trace workloads
- JSON-LD-aware ingest path

## Implementation Sequence

### Step 1. Extend Current Toolkit

Current known state:

- `benchmark-toolkit` already contains practical ProvChain and Neo4j comparison machinery
- it should be treated as the base layer for the next expansion

Immediate engineering target:

- audit toolkit code to isolate reusable harness interfaces

### Step 2. Add Common Workload Schema

Create:

- logical record schema
- query scenario schema
- per-target adapter contract

### Step 3. Add Neo4j + ProvChain Trace Family First

Reason:

- lowest ambiguity
- strongest near-term fairness
- easiest extension of current toolkit

### Step 4. Add Fluree

Reason:

- closest semantic/RDF comparison target

### Step 5. Add Fabric

Reason:

- strongest permissioned-ledger competitor
- requires explicit deployment and chaincode workload design

### Step 6. Add Geth

Reason:

- useful public-chain baseline
- should be added only after confirmation/finality semantics are frozen in the harness

## What Can Be Used In the Next Paper

Only the following may be used as competitive evidence:

- artifact-backed results produced under this specification

The following may **not** be used:

- simulated competitor utilities
- vendor marketing TPS numbers
- literature numbers presented as if they were measured by our harness

## Immediate Next Engineering Task

Create a technical implementation plan for the adapter layer with:

- input schema
- adapter interface
- result schema
- and runner orchestration strategy

This should be the next document after this specification.
