# ProvChain-Only Ledger Profiling Campaign Summary

| Field | Value |
|---|---|
| Campaign ID | `20260429_profile_ledger_supply1000_provchain-only_batchblock_n3` |
| Evidence role | `profiling_reference_not_primary_paper_comparison` |
| Benchmark family | `ledger_write` |
| Workload | `write` |
| Products | `provchain` |
| Epoch target | `3` |
| Iterations per epoch | `3` |
| Batch size | `100` |
| Dataset path | `/home/cit/provchain-org/benchmark-toolkit/datasets` |
| Dataset file | `supply_chain_1000.ttl` |
| Skip load rows | `true` |
| Skip Fabric | `true` |
| Manage ProvChain | `true` |
| Managed ProvChain port | `18080` |
| Managed ProvChain runtime dir | `/home/cit/provchain-org/benchmark-toolkit/results/provchain-runtime/20260429_profile_ledger_supply1000_provchain-only_batchblock_n3` |
| Managed ProvChain WAL sync interval | `100` |
| Managed ProvChain chain-index sync interval | `100` |
| Managed ProvChain durability mode | `relaxed_batched_fsync` |
| Fabric gateway URL | `http://localhost:18800` |
| Fabric gateway role | `ignored because skip_fabric=true` |
| ProvChain URL | `http://localhost:8080` |
| Skip ProvChain | `false` |

## Epochs

| Epoch | Run ID | Status | Notes |
|---|---|---|---|
| `epoch-001` | `20260429T065527Z` | `passed` | ok |
| `epoch-002` | `20260429T065559Z` | `passed` | ok |
| `epoch-003` | `20260429T065630Z` | `passed` | ok |
