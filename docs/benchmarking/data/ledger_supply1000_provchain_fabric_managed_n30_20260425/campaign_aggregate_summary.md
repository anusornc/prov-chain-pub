# Campaign Aggregate Summary

- Campaign: `20260425_ledger_supply1000_provchain-fabric_managed_n30_fix3`
- Generated at: `2026-04-25T08:40:45Z`

| Family | Test | System | Samples | Success Rate | Mean ms | p95 ms | p99 ms |
|---|---|---|---:|---:|---:|---:|---:|
| `ledger-write` | `Batch Commit (100 records)` | `Hyperledger Fabric` | 300 | 100.00% | 2165.363 | 2208.168 | 2248.422 |
| `ledger-write` | `Batch Submit (100 records)` | `Hyperledger Fabric` | 300 | 100.00% | 139.526 | 179.506 | 223.464 |
| `ledger-write` | `Single Record Commit` | `Hyperledger Fabric` | 300 | 100.00% | 2022.808 | 2025.666 | 2026.863 |
| `ledger-write` | `Single Record Submit` | `Hyperledger Fabric` | 300 | 100.00% | 12.809 | 15.194 | 16.436 |
| `ledger-write` | `Single-threaded Write (100 tx)` | `ProvChain-Org` | 300 | 100.00% | 29352.300 | 53346.500 | 55913.880 |
