# ProvChain Bulk Import R002 Plan - 2026-04-30

## Purpose

This document defines the experimental branch plan for fixing the current
ProvChain data-load/import weakness before any merge or new publication
benchmark claim.

Branch under test:

- `exp-provchain-bulk-import-r002`

Primary implementation artifacts on this branch:

- `POST /api/datasets/import-turtle`
- `PROVCHAIN_IMPORT_MODE=bulk-turtle-single-block`
- `benchmark-toolkit/scripts/provchain-bulk-import-r002-campaign.sh`

## Current Problem

Current benchmark data-load results show that ProvChain is much slower than
Fluree for `Turtle RDF Import` / `JSON-LD Import` rows.

The current ProvChain benchmark path is not a real bulk import path. It is a
per-triple blockchain admission loop:

1. benchmark client reads and normalizes the Turtle dataset
2. benchmark client parses Turtle into triples
3. benchmark client submits each triple to `POST /api/blockchain/add-triple`
4. the web handler turns each triple back into a Turtle statement
5. each triple becomes one blockchain block
6. each block repeats ontology/SHACL validation, state-root calculation, block
   construction, signature, RDF insert, metadata insert, WAL/index persistence,
   and RDF snapshot flush checks

Fluree is measured differently:

1. benchmark client reads and parses JSON-LD
2. benchmark client submits one `insert` payload to Fluree `transact`
3. Fluree handles the import as a database transaction rather than one
   blockchain block per RDF statement

The observed gap is therefore primarily a workload-shape gap: ProvChain is
paying blockchain admission cost per triple, while Fluree is using a bulk
transaction path.

## Current Algorithmic Costs

The current hot path repeats these operations per submitted triple:

- HTTP request/response overhead
- authenticated handler dispatch
- blockchain write lock acquisition
- Turtle string materialization
- ontology/SHACL validation over a one-line Turtle block
- state-root calculation over the RDF store
- block hash construction and Ed25519 signature
- signature verification
- temporary Oxigraph store creation
- Turtle parse into the temporary store
- quad remapping into a named graph
- Oxigraph insertion into the main store
- blockchain metadata RDF insertion
- block push into the in-memory chain
- durable block file + WAL + chain-index persistence
- RDF snapshot flush check or flush depending on configured interval

Prior R002 profiling already showed that `block_admission` dominates the write
path. The batch-block diagnostic also showed that admitting many RDF statements
as one block can reduce client-observed latency materially, but that diagnostic
did not yet provide a clean production-facing import endpoint.

## Experimental Fix

Add a native bulk Turtle import endpoint:

- `POST /api/datasets/import-turtle`

The first experimental version will:

- accept normalized Turtle data in one request
- validate request size and non-empty content
- create one blockchain block for the full Turtle document
- preserve native RDF+SHACL/blockchain semantics
- return stage timings under `timings_ms` when
  `PROVCHAIN_BENCHMARK_STAGE_TIMINGS=true`
- return dataset byte count, block count, and logical import mode metadata

This intentionally separates bulk dataset admission from:

- `POST /api/blockchain/add-triple` single-triple transaction admission
- `POST /api/blockchain/add-triples` diagnostic batch-block writes
- the `Single-threaded Write (100 tx)` ledger/write metric

## Benchmark Harness Changes

The ProvChain benchmark adapter will support explicit import modes:

- `bulk-turtle-single-block` - default for dataset load/import rows on this branch
- `legacy-per-triple` - fallback diagnostic mode for reproducing old results

The benchmark row must record metadata:

- `provchain_import_mode`
- `block_count`
- `triple_count`
- dataset read/normalize/parse/client-submit timings
- auth/bootstrap timing as separate metadata, excluded from `load-latency-ms`
- server timing totals and averages

The paper/report boundary must remain explicit:

- bulk import evidence is `data-load` / import evidence
- it must not replace the `Single-threaded Write (100 tx)` ledger/write metric
- if a single bulk block is used, it is not directly comparable to per-record
  commit/finality metrics unless reported with a clear capability/fairness label

## Test Plan

Required local gates before any benchmark:

1. `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml provchain -- --nocapture`
2. `cargo check --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml`
3. `cargo check --bin provchain-org`
4. focused web/model tests for the bulk import request contract
5. shell syntax checks for touched campaign scripts if any are changed

Runtime smoke gate after local tests:

1. start a Docker-enabled trace shell
2. run a ProvChain-focused smoke campaign first
3. inspect `campaign_status.json`, `campaign_results.csv`, and server timing metadata
4. only then run comparative `ProvChain+Neo4j+Fluree` smoke
5. only then run profile/full campaign if smoke passes

## Benchmark Plan

Initial branch-only campaign names should include the branch/experiment marker:

- smoke:
  - `smoke_import_supply1000_provchain_bulk_r002_n1_20260430`
- profile:
  - `20260430_import_supply1000_provchain_bulk_r002_n3`
- comparative smoke:
  - `smoke_trace_supply1000_provchain-neo4j-fluree_bulkimport_r002_n1_20260430`

Do not overwrite existing campaign ids.

Use the R002 wrapper instead of long environment-variable command lines:

```bash
./benchmark-toolkit/scripts/provchain-bulk-import-r002-campaign.sh smoke --id smoke_import_supply1000_provchain-bulk-r002_n1_20260430
```

After the smoke passes, run the profile gate:

```bash
./benchmark-toolkit/scripts/provchain-bulk-import-r002-campaign.sh profile --id 20260430_import_supply1000_provchain-bulk-r002_n3
```

Only after the profile gate passes should a full campaign be run:

```bash
./benchmark-toolkit/scripts/provchain-bulk-import-r002-campaign.sh full --id 20260430_import_supply1000_provchain-bulk-r002_n30
```

If a direct A/B check against the old import path is needed on the same
dataset/runtime stack, use:

```bash
./benchmark-toolkit/scripts/provchain-bulk-import-r002-campaign.sh profile --legacy-provchain-import --id 20260430_import_supply1000_provchain-legacy-r002_n3
```

The wrapper records `provchain_import_mode` in `campaign_manifest.json` and
exports passing profile/full evidence to `docs/benchmarking/data/reference/`.

## Local Validation Completed

The branch implementation has passed these local gates:

- `cargo check --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml`
- `cargo check --bin provchain-org`
- `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml provchain -- --nocapture`
- `cargo test turtle_import_payload -- --nocapture`
- `bash -n benchmark-toolkit/scripts/run-trace-campaign.sh`
- `bash -n benchmark-toolkit/scripts/provchain-bulk-import-r002-campaign.sh`
- `docker compose -f benchmark-toolkit/docker-compose.trace.yml config`
- `git diff --check`

The first benchmark-runner test attempt failed inside the sandbox because the
mock HTTP server could not bind a local port. The same test passed when rerun
with local mock-server permissions.

## First Profile Result And Harness Correction

First R002 Docker profile:

- campaign: `20260430_import_supply1000_provchain-bulk-r002_n3`
- status: `passed`, `3/3` epochs
- evidence export:
  `docs/benchmarking/data/reference/r002_import_supply_chain_1000_provchain_bulk-r002_n3_20260430/`
- manifest confirmed `provchain_import_mode=bulk-turtle-single-block`
- ProvChain bulk import row: mean `661.333 ms`
- Fluree JSON-LD import row: mean `486.000 ms`
- Neo4j Turtle-to-Cypher import row: mean `11119.667 ms`

Important interpretation:

- the first profile proves the algorithmic fix works and reduced ProvChain
  import from the previous same-slice baseline of `12124.033 ms` to
  `661.333 ms`
- however, raw metadata showed the ProvChain row still included auth/bootstrap
  latency of about `634`-`646 ms`
- server-side import handling itself was about `10 ms`, with one block for
  `632` triples

Harness correction:

- ProvChain dataset-load timing now authenticates before starting
  `load-latency-ms`
- `auth_latency_ms` remains in metadata for audit
- this keeps load/import rows focused on dataset read, normalization, parsing,
  submit, and server admission rather than benchmark client login setup
- local validation after this correction passed:
  - `cargo check --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml`
  - `cargo check --bin provchain-org`
  - `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml provchain -- --nocapture`
  - `cargo test turtle_import_payload -- --nocapture`
  - `bash -n benchmark-toolkit/scripts/run-trace-campaign.sh`
  - `bash -n benchmark-toolkit/scripts/provchain-bulk-import-r002-campaign.sh`
- a new R002 smoke/profile rerun is required before merge

## Auth-Excluded Profile Result

Auth-excluded R002 Docker profile:

- campaign: `20260430_import_supply1000_provchain-bulk-r002_authfix_n3`
- status: `passed`, `3/3` epochs
- evidence export:
  `docs/benchmarking/data/reference/r002_import_supply_chain_1000_provchain_bulk-r002_authfix_n3_20260430/`
- manifest confirmed `provchain_import_mode=bulk-turtle-single-block`
- ProvChain bulk Turtle import row: mean `23.000 ms`
- Fluree JSON-LD import row: mean `514.333 ms`
- Neo4j Turtle-to-Cypher import row: mean `11594.667 ms`
- ProvChain metadata still records auth separately:
  - `auth_latency_ms` about `610`-`637 ms`
  - `client_submit_loop_latency_ms` about `11 ms`
  - server `handler_total` about `10 ms`
  - `block_count=1`
  - `triple_count=632`

Interpretation:

- the post-correction metric now reflects dataset read, normalization, parse,
  submit, and server admission, excluding benchmark client login/bootstrap
- against the old same-slice baseline of `12124.033 ms`, the auth-excluded
  bulk import profile is about `527.13x` faster and `99.81%` lower latency
- this result is strong enough to merge the R002 import path into `main`, but
  it remains a bulk dataset-admission result rather than a replacement for
  per-transaction ledger/write metrics

Export wrapper hardening:

- `provchain-bulk-import-r002-campaign.sh` now defaults the curated export
  directory to `docs/benchmarking/data/reference/<campaign_id>` so reruns with
  explicit ids do not collide with earlier date/dataset defaults

## Final Full Result On Main

Final R002 full campaign:

- campaign: `20260430_import_supply1000_provchain-bulk-r002_final_n30`
- status: `passed`, `30/30` epochs
- evidence export:
  `docs/benchmarking/data/reference/20260430_import_supply1000_provchain-bulk-r002_final_n30/`
- manifest confirmed:
  - `provchain_import_mode=bulk-turtle-single-block`
  - `products=provchain,neo4j,fluree`
  - `iterations_per_epoch=10`
- ProvChain bulk Turtle import: mean `24.333 ms`, p95 `35.150 ms`,
  p99 `41.000 ms`
- Fluree JSON-LD import: mean `478.467 ms`, p95 `560.650 ms`,
  p99 `575.970 ms`
- Neo4j Turtle-to-Cypher import: mean `11431.367 ms`, p95 `11855.800 ms`,
  p99 `11935.670 ms`

Interpretation:

- compared with the previous same-slice legacy ProvChain baseline
  `20260424_trace_supply1000_provchain-neo4j_n30` (`12124.033 ms`), final
  R002 bulk import is about `498.26x` faster and `99.80%` lower mean latency
- compared with Fluree in the same final campaign, ProvChain bulk Turtle import
  is about `19.66x` faster on the load row
- query rankings remain stable: ProvChain remains fastest across simple
  lookup, multi-hop traceability, and aggregation by producer
- claim boundary remains unchanged: use this as bulk dataset-admission evidence,
  not as a per-transaction ledger/write or finality claim

## Merge Criteria

Do not merge this branch until:

- local Rust checks pass
- benchmark-runner adapter tests pass
- the bulk endpoint is authenticated and size-limited
- metadata clearly labels the import mode
- old per-triple behavior remains available as a diagnostic fallback
- a smoke benchmark proves that the endpoint works inside the Docker trace stack
- a profile benchmark shows whether `Turtle RDF Import` improves enough to
  justify merging the import path into `main`
- the post-auth-exclusion smoke/profile rerun passes
- evidence documents explicitly prevent using bulk import as a replacement for
  per-transaction ledger/write claims
