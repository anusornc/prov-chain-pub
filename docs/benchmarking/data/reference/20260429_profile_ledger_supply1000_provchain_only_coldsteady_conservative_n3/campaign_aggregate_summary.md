# Campaign Aggregate Summary

- Campaign: `20260429_profile_ledger_supply1000_provchain-only_coldsteady_conservative_n3`
- Generated at: `2026-04-29T10:27:07Z`

| Family | Test | System | Path | Metric | Unit | Samples | Success Rate | Mean | p95 | p99 |
|---|---|---|---|---|---|---:|---:|---:|---:|---:|
| `ledger-write` | `Turtle RDF Import` | `ProvChain-Org` | `native-rdf-path` | `load-latency-ms` | `ms` | 3 | 100.00% | 50736.000 | 51061.600 | 51107.520 |
| `ledger-write` | `Batched Write (100 triples, 1 block)` | `ProvChain-Org` | `native-rdf-path` | `submit-latency-ms` | `ms` | 9 | 100.00% | 1766.770 | 1803.113 | 1808.423 |
| `ledger-write` | `Write Batch Authentication` | `ProvChain-Org` | `native-rdf-path` | `authentication-latency-ms` | `ms` | 9 | 100.00% | 1259.958 | 1286.191 | 1288.440 |
| `ledger-write` | `Write Batch HTTP Submit Loop` | `ProvChain-Org` | `native-rdf-path` | `client-submit-loop-latency-ms` | `ms` | 9 | 100.00% | 16713.070 | 18873.838 | 18917.555 |
| `ledger-write` | `Steady-state Append After Cold Load (100 tx)` | `ProvChain-Org` | `native-rdf-path` | `submit-latency-ms` | `ms` | 9 | 100.00% | 17972.444 | 20141.800 | 20193.960 |
