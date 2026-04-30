# ProvChain Ledger Write Profiling Summary

- Campaign: `20260429_profile_ledger_supply1000_provchain-only_flush100_n3`
- Generated at: `2026-04-29T02:33:15Z`
- Samples: `9`
- Transactions per sample: `100`
- Evidence role: `profiling_reference_not_primary_paper_comparison`

## Client-observed Batch Metrics

| Metric | Samples | Mean ms | p95 ms | p99 ms | Min ms | Max ms |
|---|---:|---:|---:|---:|---:|---:|
| Batch total | 9 | 10695.444 | 14902.200 | 14976.440 | 6860.000 | 14995.000 |
| Authentication | 9 | 1651.974 | 2506.955 | 2507.271 | 1218.832 | 2507.350 |
| Client submit loop | 9 | 9044.055 | 13678.848 | 13750.210 | 4359.056 | 13768.051 |

## Server Handler Stage Averages

| Stage | Samples | Mean ms/tx | p95 ms/tx | p99 ms/tx | Min ms/tx | Max ms/tx |
|---|---:|---:|---:|---:|---:|---:|
| `handler_total` | 9 | 88.971 | 135.337 | 135.965 | 42.148 | 136.121 |
| `block_admission` | 9 | 85.207 | 131.597 | 132.129 | 38.430 | 132.262 |
| `request_validation` | 9 | 3.730 | 4.049 | 4.130 | 3.530 | 4.150 |
| `blockchain_lock_wait` | 9 | 0.004 | 0.005 | 0.005 | 0.004 | 0.005 |
| `turtle_materialization` | 9 | 0.002 | 0.003 | 0.003 | 0.002 | 0.003 |

## Interpretation

- Dominant measured server stage: `block_admission` at `95.77%` of mean handler time.
- This is profiling/remediation evidence, not a primary cross-system comparison table.
