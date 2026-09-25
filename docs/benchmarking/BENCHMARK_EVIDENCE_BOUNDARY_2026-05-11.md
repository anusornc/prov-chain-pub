# Benchmark Evidence Boundary - 2026-05-11

## Purpose

This document defines which benchmark artifacts may be used for thesis, paper, or public-release claims.

The repository contains active benchmark infrastructure, legacy benchmark code, raw local outputs, and historical planning documents. Only the curated evidence chain described here should be used for publication-facing claims.

## Valid Paper / Thesis Evidence

Benchmark evidence is valid for paper or thesis claims only when all of the following are true:

1. The result is produced by the active benchmark harness or a documented Criterion benchmark.
2. The result has a curated export under `docs/benchmarking/data/reference/` or a documented CSV/figure export under `docs/benchmarking/data/`.
3. The result is linked from the benchmark report, paper evidence index, or paper table provenance map.
4. The claim is scoped to the benchmark family recorded in the artifact.

Current valid evidence families include:

- ontology-admission micro-benchmarks from `benches/ontology_admission_benchmarks.rs`
- domain dataset admission Criterion exports under `docs/benchmarking/data/`
- curated trace-query campaigns under `docs/benchmarking/data/reference/`
- curated semantic/admission, policy, ledger, and diagnostic profiles when their caveats are preserved

## Active Benchmark Harness

The active Docker/adaptor benchmark harness is:

- `benchmark-toolkit/research-benchmarks/`

The active trace-stack orchestration and campaign wrappers are:

- `benchmark-toolkit/docker-compose.trace.yml`
- `benchmark-toolkit/scripts/run-trace-campaign.sh`
- product/family-specific campaign wrappers under `benchmark-toolkit/scripts/`

## Non-Evidence Sources

The following must not be used as paper, thesis, or public benchmark evidence:

- `benchmark-toolkit/src/main.rs`
- output from the legacy portable runner under `benchmark-toolkit/src/`
- raw local files under `benchmark-toolkit/results/` unless intentionally curated and exported
- simulated competitor utilities under `tests/utils/`
- placeholder summaries with zero/TODO comparator values
- one-off terminal output that has not been exported into a documented reference artifact

## Legacy Portable Runner Status

`benchmark-toolkit/src/main.rs` is retained only as historical/legacy code. It contains stale route assumptions such as `/api/rdf/import` and `/api/transactions`, plus placeholder Neo4j summary behavior.

It is intentionally not part of the evidence chain. If it is run for archaeological comparison, the output must be labeled `legacy-non-evidence` and excluded from paper tables, figures, and thesis claims.

## Claim Scoping Rules

Use benchmark results only inside their recorded family:

- `trace_query`: provenance reconstruction and trace-query latency only
- `semantic_admission`: semantic/standards admission behavior only
- `ledger_write`: ledger/write path behavior only
- `policy`: governance or access-policy behavior only
- `diagnostic` or `profile`: bottleneck analysis only, not primary cross-system superiority evidence

Do not generalize trace-query results into ledger finality, write throughput, production deployment, or all-purpose database superiority claims.

## Review Checklist

Before adding a benchmark number to a manuscript, report, or README:

- confirm the artifact path is curated under `docs/benchmarking/data/` or `docs/benchmarking/data/reference/`
- confirm the campaign status is `passed`
- confirm the benchmark family matches the claim
- confirm caveats and fairness labels are preserved
- confirm the source is not the legacy portable runner or a raw local result
