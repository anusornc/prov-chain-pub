# ProvChain Ledger Write Profiling Summary

- Campaign: `20260429_profile_ledger_supply1000_provchain-only_walsync100_n3`
- Generated at: `2026-04-29T06:44:12Z`
- Samples: `9`
- Transactions per sample: `100`
- Evidence role: `profiling_reference_not_primary_paper_comparison`

## Client-observed Batch Metrics

| Metric | Samples | Mean ms | p95 ms | p99 ms | Min ms | Max ms |
|---|---:|---:|---:|---:|---:|---:|
| Batch total | 9 | 5926.444 | 7654.400 | 7817.280 | 4689.000 | 7858.000 |
| Authentication | 9 | 1647.734 | 2497.162 | 2510.465 | 1229.624 | 2513.791 |
| Client submit loop | 9 | 4279.226 | 6424.598 | 6587.141 | 2180.833 | 6627.777 |

## Server Handler Stage Averages

| Stage | Samples | Mean ms/tx | p95 ms/tx | p99 ms/tx | Min ms/tx | Max ms/tx |
|---|---:|---:|---:|---:|---:|---:|
| `handler_total` | 9 | 41.349 | 62.781 | 64.378 | 20.381 | 64.777 |
| `block_admission` | 9 | 37.618 | 58.928 | 60.399 | 16.807 | 60.766 |
| `request_validation` | 9 | 3.699 | 3.981 | 3.984 | 3.526 | 3.984 |
| `blockchain_lock_wait` | 9 | 0.004 | 0.004 | 0.004 | 0.004 | 0.004 |
| `turtle_materialization` | 9 | 0.002 | 0.002 | 0.002 | 0.002 | 0.002 |

## Interpretation

- Dominant measured server stage: `block_admission` at `90.98%` of mean handler time.
- This is profiling/remediation evidence, not a primary cross-system comparison table.
