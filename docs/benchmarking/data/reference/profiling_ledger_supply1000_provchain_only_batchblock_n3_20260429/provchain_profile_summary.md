# ProvChain Ledger Write Profiling Summary

- Campaign: `20260429_profile_ledger_supply1000_provchain-only_batchblock_n3`
- Generated at: `2026-04-29T06:57:27Z`
- Samples: `9`
- Transactions per sample: `100`
- Evidence role: `profiling_reference_not_primary_paper_comparison`

## Client-observed Batch Metrics

| Metric | Samples | Mean ms | p95 ms | p99 ms | Min ms | Max ms |
|---|---:|---:|---:|---:|---:|---:|
| Batch total | 9 | 6434.444 | 8094.600 | 8141.320 | 4949.000 | 8153.000 |
| Authentication | 9 | 1679.475 | 2514.290 | 2516.887 | 1227.684 | 2517.536 |
| Client submit loop | 9 | 4755.490 | 6844.681 | 6908.552 | 2440.514 | 6924.520 |

## Server Handler Stage Averages

| Stage | Samples | Mean ms/tx | p95 ms/tx | p99 ms/tx | Min ms/tx | Max ms/tx |
|---|---:|---:|---:|---:|---:|---:|
| `handler_total` | 9 | 46.069 | 66.964 | 67.597 | 22.931 | 67.755 |
| `block_admission` | 9 | 41.960 | 63.099 | 63.675 | 19.018 | 63.820 |
| `request_validation` | 9 | 4.074 | 4.665 | 4.723 | 3.525 | 4.738 |
| `blockchain_lock_wait` | 9 | 0.004 | 0.005 | 0.005 | 0.004 | 0.005 |
| `turtle_materialization` | 9 | 0.002 | 0.003 | 0.003 | 0.002 | 0.003 |

## Interpretation

- Dominant measured server stage: `block_admission` at `91.08%` of mean handler time.
- This is profiling/remediation evidence, not a primary cross-system comparison table.
