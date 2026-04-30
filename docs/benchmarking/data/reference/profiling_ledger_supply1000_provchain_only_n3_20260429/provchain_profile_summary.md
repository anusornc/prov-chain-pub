# ProvChain Ledger Write Profiling Summary

- Campaign: `20260429_profile_ledger_supply1000_provchain-only_n3_fix1`
- Generated at: `2026-04-29T02:22:43Z`
- Samples: `9`
- Transactions per sample: `100`
- Evidence role: `profiling_reference_not_primary_paper_comparison`

## Client-observed Batch Metrics

| Metric | Samples | Mean ms | p95 ms | p99 ms | Min ms | Max ms |
|---|---:|---:|---:|---:|---:|---:|
| Batch total | 9 | 11967.556 | 16893.200 | 17020.240 | 7419.000 | 17052.000 |
| Authentication | 9 | 1661.287 | 2521.564 | 2524.083 | 1231.923 | 2524.713 |
| Client submit loop | 9 | 10306.832 | 15660.227 | 15786.763 | 4895.152 | 15818.397 |

## Server Handler Stage Averages

| Stage | Samples | Mean ms/tx | p95 ms/tx | p99 ms/tx | Min ms/tx | Max ms/tx |
|---|---:|---:|---:|---:|---:|---:|
| `handler_total` | 9 | 101.547 | 155.065 | 156.315 | 47.473 | 156.628 |
| `block_admission` | 9 | 97.731 | 151.354 | 152.638 | 43.704 | 152.959 |
| `request_validation` | 9 | 3.779 | 4.067 | 4.142 | 3.604 | 4.161 |
| `blockchain_lock_wait` | 9 | 0.004 | 0.005 | 0.005 | 0.004 | 0.005 |
| `turtle_materialization` | 9 | 0.002 | 0.003 | 0.003 | 0.002 | 0.003 |

## Interpretation

- Dominant measured server stage: `block_admission` at `96.24%` of mean handler time.
- This is profiling/remediation evidence, not a primary cross-system comparison table.
