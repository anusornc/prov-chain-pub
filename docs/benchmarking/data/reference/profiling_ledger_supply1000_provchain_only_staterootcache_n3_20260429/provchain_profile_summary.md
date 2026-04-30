# ProvChain Ledger Write Profiling Summary

- Campaign: `20260429_profile_ledger_supply1000_provchain-only_staterootcache_n3`
- Generated at: `2026-04-29T06:18:20Z`
- Samples: `9`
- Transactions per sample: `100`
- Evidence role: `profiling_reference_not_primary_paper_comparison`

## Client-observed Batch Metrics

| Metric | Samples | Mean ms | p95 ms | p99 ms | Min ms | Max ms |
|---|---:|---:|---:|---:|---:|---:|
| Batch total | 9 | 6011.111 | 7698.600 | 7794.920 | 4736.000 | 7819.000 |
| Authentication | 9 | 1667.347 | 2545.319 | 2562.483 | 1229.800 | 2566.775 |
| Client submit loop | 9 | 4344.425 | 6457.648 | 6547.969 | 2229.319 | 6570.549 |

## Server Handler Stage Averages

| Stage | Samples | Mean ms/tx | p95 ms/tx | p99 ms/tx | Min ms/tx | Max ms/tx |
|---|---:|---:|---:|---:|---:|---:|
| `handler_total` | 9 | 41.992 | 63.074 | 63.942 | 20.914 | 64.159 |
| `block_admission` | 9 | 38.197 | 59.145 | 59.915 | 17.329 | 60.108 |
| `request_validation` | 9 | 3.758 | 4.171 | 4.257 | 3.549 | 4.278 |
| `blockchain_lock_wait` | 9 | 0.004 | 0.004 | 0.004 | 0.004 | 0.004 |
| `turtle_materialization` | 9 | 0.002 | 0.002 | 0.002 | 0.002 | 0.002 |

## Interpretation

- Dominant measured server stage: `block_admission` at `90.96%` of mean handler time.
- This is profiling/remediation evidence, not a primary cross-system comparison table.
