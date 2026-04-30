# Geth Benchmark Workload Contract - 2026-04-25

## Purpose

This document defines the minimum benchmark-facing workload for adding
`Go Ethereum (Geth)` to the multi-product benchmark program.

The goal is to make `Geth` an honest public-chain execution baseline for
`Family A. Ledger / Write Path`. It must not be presented as a permissioned
enterprise ledger, semantic graph store, or native trace-query system.

## Scope

`Geth` participates only in these benchmark families until a later ADR changes
the contract:

- `ledger_write`: primary target
- `governance_policy`: conditional future target if policy is implemented in a
  smart contract and measured end to end

Out of scope for this phase:

- native RDF storage
- native SPARQL / trace-query workloads
- native SHACL or OWL reasoning
- permissioned endorsement semantics
- private data collection semantics

## Runtime Assumption

The benchmark target is a local deterministic Geth development chain.

Minimum runtime contract:

- JSON-RPC endpoint: `GETH_RPC_URL`, default `http://localhost:8545`
- chain id: explicit in the runtime manifest
- mining mode: deterministic local mining or dev auto-mining
- funded sender account: explicit in the runtime manifest
- benchmark contract address: explicit in the run manifest
- transaction receipts must be retrievable by JSON-RPC

The benchmark must record whether the runtime uses auto-mining or interval
mining because it changes confirmation latency interpretation.

## Smart Contract Scope

The minimum contract stores one logical supply-chain event per call.

Canonical event fields:

| Field | Type | Notes |
|---|---|---|
| `recordId` | `string` or `bytes32` | canonical benchmark record id |
| `entityId` | `string` or `bytes32` | supply-chain entity id |
| `eventType` | `string` or `bytes32` | e.g. `Produced`, `Transferred` |
| `actorId` | `string` or `bytes32` | actor identifier |
| `timestamp` | `uint64` or `string` | benchmark timestamp |
| `payloadHash` | `bytes32` | hash of canonical off-chain payload |

Required contract methods:

- `submitRecord(...)`
  - emits `RecordSubmitted(recordId, entityId, payloadHash)`
  - persists enough state to verify the record exists
- `getRecord(recordId)`
  - returns stored record metadata or a deterministic existence flag

Optional later methods:

- `submitBatch(...)`
- `checkPolicy(...)`

## Workloads

### W1. Single Record Submit

Submit one logical event as one Ethereum transaction.

Metrics:

- `submit_latency_ms`: wall-clock time from JSON-RPC submission start to tx hash
  availability
- `confirmation_latency_ms`: wall-clock time from JSON-RPC submission start to
  receipt availability with success status
- `gas_used`
- `effective_gas_price_wei`, if available
- `tx_hash`
- `block_number`
- `receipt_status`

### W2. Sequential Record Submit

Submit `N` logical events sequentially, one transaction per record.

Default `N` values:

- smoke: `1`
- full campaign: `10` iterations per epoch, one record per iteration
- optional stress run: `100`

Metrics:

- per-record metrics from `W1`
- throughput in confirmed records per second
- failure rate

### W3. Batch-Like Submit

This is optional for B017 unless the contract implements `submitBatch`.

If implemented, one Ethereum transaction carries multiple logical records.
The benchmark must report this separately from single-record transactions.

Metrics:

- `submit_latency_ms`
- `confirmation_latency_ms`
- `records_per_transaction`
- `gas_used`
- `gas_used_per_record`
- confirmed records per second

## Submit vs Confirmation Model

The benchmark must separate:

- submit latency: JSON-RPC tx submission and tx hash return
- confirmation latency: transaction receipt availability and success status

Do not compare Geth submit latency directly against Fabric commit latency or
ProvChain write latency without naming the metric. Cross-system ledger claims
must use family-specific tables and fairness labels.

## Result Schema Requirements

Every Geth result row must include:

- `family = ledger-write`
- `system = Go Ethereum (Geth)`
- `fairness_label = public-chain-baseline`
- `capability_path = public-chain-smart-contract`
- metric type:
  - `submit-latency-ms`
  - `confirmation-latency-ms`
  - `gas-used`
  - optional `gas-used-per-record`
- metadata:
  - `chain_id`
  - `contract_address`
  - `sender_address`
  - `tx_hash`
  - `block_number`
  - `receipt_status`
  - `gas_used`
  - `effective_gas_price_wei`
  - `mining_mode`

If the current result enum does not yet include `confirmation-latency-ms`,
`gas-used`, or `public-chain-baseline`, B016 must add them before local tests
are considered complete.

## Fairness Boundary

Allowed claims:

- Geth provides a public-chain smart-contract ledger baseline.
- Geth confirmation latency is measured separately from submit latency.
- gas metadata is part of the evidence.

Disallowed claims:

- Geth is a permissioned ledger comparable to Fabric.
- Geth has native RDF, SPARQL, SHACL, OWL, or provenance graph-query support.
- Geth trace-query results exist unless an explicit external indexer is added
  and measured as part of the stack.

## B016 Acceptance Criteria

B016 is done when local contract tests prove:

- health check decodes JSON-RPC client version
- deploy or configured contract address is validated
- single submit returns a tx hash
- receipt polling records confirmation latency
- failed receipts are surfaced as failed benchmark rows
- gas metadata is decoded into result metadata

## B017 Acceptance Criteria

B017 is done when:

- a smoke campaign runs against a real local Geth runtime
- a full campaign produces campaign layout artifacts
- campaign summary separates submit and confirmation latency
- curated export is stored under `docs/benchmarking/data/`
- docs mark the evidence as `public-chain-baseline`, not permissioned ledger
  parity
