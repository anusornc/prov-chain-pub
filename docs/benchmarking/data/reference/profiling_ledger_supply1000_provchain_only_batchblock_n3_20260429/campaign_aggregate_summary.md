# Campaign Aggregate Summary

- Campaign: `20260429_profile_ledger_supply1000_provchain-only_batchblock_n3`
- Generated at: `2026-04-29T06:57:02Z`

| Family | Test | System | Path | Metric | Unit | Samples | Success Rate | Mean | p95 | p99 |
|---|---|---|---|---|---|---:|---:|---:|---:|---:|
| `ledger-write` | `Single-threaded Write (100 tx)` | `ProvChain-Org` | `native-rdf-path` | `submit-latency-ms` | `ms` | 9 | 100.00% | 6434.444 | 8094.600 | 8141.320 |
| `ledger-write` | `Batched Write (100 triples, 1 block)` | `ProvChain-Org` | `native-rdf-path` | `submit-latency-ms` | `ms` | 9 | 100.00% | 1646.265 | 1678.490 | 1687.068 |
| `ledger-write` | `Write Batch Authentication` | `ProvChain-Org` | `native-rdf-path` | `authentication-latency-ms` | `ms` | 9 | 100.00% | 1679.475 | 2514.290 | 2516.887 |
| `ledger-write` | `Write Batch HTTP Submit Loop` | `ProvChain-Org` | `native-rdf-path` | `client-submit-loop-latency-ms` | `ms` | 9 | 100.00% | 4755.490 | 6844.681 | 6908.552 |
