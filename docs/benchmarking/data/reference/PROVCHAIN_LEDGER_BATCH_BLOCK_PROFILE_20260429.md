# ProvChain Ledger Batch-Block Profile - 2026-04-29

This note records the R002 diagnostic campaign for submitting `100` RDF triples
as one blockchain block through the new benchmark endpoint:

- endpoint: `POST /api/blockchain/add-triples`
- campaign: `20260429_profile_ledger_supply1000_provchain-only_batchblock_n3`
- curated export: `docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_batchblock_n3_20260429/`
- status: `passed`, `3/3` epochs, `3` iterations per epoch
- evidence role: `reference-or-profiling-evidence`
- durability mode: `relaxed_batched_fsync`
- WAL sync interval: `100`
- chain-index sync interval: `100`

## Result Summary

| Metric | Samples | Mean | p95 | p99 |
|---|---:|---:|---:|---:|
| Existing write path: `100 tx` | 9 | `6434.444 ms` | `8094.600 ms` | `8141.320 ms` |
| Diagnostic batch-block path: `100 triples, 1 block` | 9 | `1646.265 ms` | `1678.490 ms` | `1687.068 ms` |
| Diagnostic throughput, including auth | 9 | `60.755 triples/s` | `61.925 triples/s` | `61.936 triples/s` |
| Diagnostic auth latency | 9 | `1232.097 ms` | `1246.784 ms` | `1255.096 ms` |
| Diagnostic client submit loop | 9 | `413.293 ms` | `448.728 ms` | `456.836 ms` |

## Server Stage Summary For Batch-Block Row

| Stage | Samples | Mean |
|---|---:|---:|
| `handler_total` | 9 | `407.085 ms/batch` |
| `request_validation` | 9 | `350.556 ms/batch` |
| `block_admission` | 9 | `56.402 ms/batch` |

## Interpretation

Within this diagnostic campaign, the batch-block path is `74.41%` faster than
the existing `100 tx` path in client-observed mean latency. This is expected
because it creates one block containing `100` triples instead of creating `100`
separate blocks.

The measurement is useful for locating per-block admission overhead and for
designing future batch-ingest APIs. It is not a replacement for the existing
ledger/write comparison metric, because the semantics differ:

- `Single-threaded Write (100 tx)` measures `100` blockchain transactions and
  `100` blocks.
- `Batched Write (100 triples, 1 block)` measures one blockchain transaction
  carrying `100` RDF triples.

## Claim Boundary

- This evidence can support a methods/limitations statement that ProvChain has
  substantial per-block overhead and benefits from batch-block semantics.
- This evidence cannot support a cross-system ledger/write win claim.
- This evidence cannot be compared directly with Fabric batch commit unless the
  paper explicitly compares equivalent batch semantics and finality boundaries.
- Because this campaign uses relaxed WAL/index fsync settings, it is not
  production durable-throughput evidence.

## Reproduction

```bash
PROVCHAIN_WAL_SYNC_INTERVAL=100 PROVCHAIN_CHAIN_INDEX_SYNC_INTERVAL=100 \
  ./benchmark-toolkit/scripts/provchain-fabric-campaign.sh run \
  --epochs 3 \
  --iterations 3 \
  --id 20260429_profile_ledger_supply1000_provchain-only_batchblock_n3 \
  --skip-fabric \
  --skip-preflight

./benchmark-toolkit/scripts/export-campaign-evidence.sh \
  20260429_profile_ledger_supply1000_provchain-only_batchblock_n3 \
  docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_batchblock_n3_20260429

python3 benchmark-toolkit/scripts/summarize-provchain-profile.py \
  benchmark-toolkit/results/campaigns/20260429_profile_ledger_supply1000_provchain-only_batchblock_n3 \
  docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_batchblock_n3_20260429
```
