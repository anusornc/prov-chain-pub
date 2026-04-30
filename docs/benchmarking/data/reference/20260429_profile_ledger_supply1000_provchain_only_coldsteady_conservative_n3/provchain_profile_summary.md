# ProvChain Ledger Write Profiling Summary

- Campaign: `20260429_profile_ledger_supply1000_provchain-only_coldsteady_conservative_n3`
- Generated at: `2026-04-29T10:27:07Z`
- Append samples: `9`
- Cold-load samples: `3`
- Transactions per sample: `100`
- Evidence role: `profiling_reference_not_primary_paper_comparison`
- Append test names observed: `Steady-state Append After Cold Load (100 tx)`

## Cold-Load Metrics

| Metric | Samples | Mean ms | p95 ms | p99 ms | Min ms | Max ms |
|---|---:|---:|---:|---:|---:|---:|
| Cold Turtle RDF import total | 3 | 50736.000 | 51061.600 | 51107.520 | 50544.000 | 51119.000 |
| Dataset read | 3 | 0.038 | 0.039 | 0.039 | 0.036 | 0.039 |
| Turtle normalize | 3 | 51.398 | 51.430 | 51.431 | 51.339 | 51.431 |
| Turtle parse | 3 | 6.536 | 6.586 | 6.591 | 6.487 | 6.592 |
| Authentication | 3 | 2515.807 | 2516.259 | 2516.306 | 2515.376 | 2516.318 |
| Client submit loop | 3 | 48162.729 | 48488.438 | 48534.370 | 47970.625 | 48545.853 |

## Cold-Load Server Handler Stage Averages

| Stage | Samples | Mean ms/tx | p95 ms/tx | p99 ms/tx | Min ms/tx | Max ms/tx |
|---|---:|---:|---:|---:|---:|---:|
| `handler_total` | 3 | 74.080 | 74.556 | 74.621 | 73.775 | 74.637 |
| `block_admission` | 3 | 69.342 | 69.740 | 69.796 | 69.105 | 69.810 |
| `request_validation` | 3 | 4.703 | 4.780 | 4.789 | 4.634 | 4.791 |
| `blockchain_lock_wait` | 3 | 0.004 | 0.005 | 0.005 | 0.004 | 0.005 |
| `turtle_materialization` | 3 | 0.002 | 0.002 | 0.002 | 0.002 | 0.002 |

## Steady-State Append Metrics

| Metric | Samples | Mean ms | p95 ms | p99 ms | Min ms | Max ms |
|---|---:|---:|---:|---:|---:|---:|
| Batch total | 9 | 17972.444 | 20141.800 | 20193.960 | 15690.000 | 20207.000 |
| Authentication | 9 | 1259.958 | 1286.191 | 1288.440 | 1231.168 | 1289.002 |
| Client submit loop | 9 | 16713.070 | 18873.838 | 18917.555 | 14459.161 | 18928.485 |

## Steady-State Append Server Handler Stage Averages

| Stage | Samples | Mean ms/tx | p95 ms/tx | p99 ms/tx | Min ms/tx | Max ms/tx |
|---|---:|---:|---:|---:|---:|---:|
| `handler_total` | 9 | 164.899 | 186.451 | 186.849 | 142.392 | 186.949 |
| `block_admission` | 9 | 161.195 | 182.765 | 183.153 | 138.740 | 183.250 |
| `request_validation` | 9 | 3.666 | 3.755 | 3.764 | 3.613 | 3.766 |
| `blockchain_lock_wait` | 9 | 0.005 | 0.005 | 0.005 | 0.004 | 0.005 |
| `turtle_materialization` | 9 | 0.002 | 0.002 | 0.002 | 0.002 | 0.002 |

## Interpretation

- Dominant measured server stage: `block_admission` at `97.75%` of mean handler time.
- This is profiling/remediation evidence, not a primary cross-system comparison table.
