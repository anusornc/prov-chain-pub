# Benchmark Campaign Index

This index classifies local campaign directories by publication use.

Raw campaign directories are audit artifacts. Paper-facing claims should use curated
exports under `docs/benchmarking/data/` unless this index explicitly says otherwise.

## Primary Paper Evidence

| Campaign | Curated Export | Use |
|---|---|---|
| `20260424_trace_supply1000_provchain-neo4j_n30` | `docs/benchmarking/data/trace_supply1000_provchain_neo4j_n30_20260424/` | Trace-query baseline |
| `20260428_trace_supply1000_provchain-neo4j-fluree_n30` | `docs/benchmarking/data/trace_supply1000_provchain_neo4j_fluree_n30_20260428/` | Trace-query Fluree evidence |
| `20260425_ledger_supply1000_provchain-fabric_managed_n30_fix3` | `docs/benchmarking/data/ledger_supply1000_provchain_fabric_managed_n30_20260425/` | Permissioned ledger/write evidence |
| `20260428_semantic_supply1000_provchain-fluree_n30` | `docs/benchmarking/data/semantic_supply1000_provchain_fluree_n30_20260428/` | Semantic admission evidence |
| `20260428_policy_supply1000_fabric_pack_n30` | `docs/benchmarking/data/policy_supply1000_fabric_pack_n30_20260428/` | Fabric policy workload pack |
| `20260428_ledger_supply1000_provchain-geth_n30_fix1` | `docs/benchmarking/data/ledger_supply1000_provchain_geth_n30_20260428/` | Public-chain ledger baseline |

## Reference Or Superseded

| Campaign | Status | Reason |
|---|---|---|
| `baseline_trace_supply1000_provchain-neo4j_20260422T154555Z_n1` | reference | Historical single-run baseline; superseded by `20260424_trace_supply1000_provchain-neo4j_n30` for statistical evidence. |
| `20260425_policy_supply1000_fabric_n30` | reference | Valid B014 Fabric-only policy evidence; superseded by the richer B019 policy workload pack for paper use. |
| `20260429_profile_ledger_supply1000_provchain-only_n3_fix1` | profiling reference | R002 ProvChain-only write-path profiling evidence; use for bottleneck analysis, not paper comparison tables. |
| `20260429_profile_ledger_supply1000_provchain-only_flush100_n3` | profiling reference | R002 post-flush-batching profiling evidence; use for before/after remediation analysis, not paper comparison tables. |
| `20260429_profile_ledger_supply1000_provchain-only_staterootcache_n3` | profiling reference | R002 post-flush-batching plus state-root cache profiling evidence; use for remediation analysis, not paper comparison tables. |
| `20260429_profile_ledger_supply1000_provchain-only_walsync100_n3` | profiling reference | R002 relaxed WAL/index fsync profiling evidence; use for remediation analysis, not paper comparison tables or production durable-throughput claims. |
| `20260429_profile_ledger_supply1000_provchain-only_batchblock_n3` | profiling reference | R002 batch-block semantics diagnostic; use for per-block overhead analysis only, not as a replacement for `100 tx` ledger/write comparison rows. |

## Non-Paper Archive Categories

| Category | Campaigns | Rule |
|---|---|---|
| `failed` | `20260424_ledger_supply1000_provchain-fabric_n30`, failed `smoke_ledger_*`, failed `smoke_ledger_supply1000_provchain-geth_*`, `20260429_profile_ledger_supply1000_provchain-only_n3` | Keep for engineering forensics only. Do not use for paper tables. |
| `partial` | partial smoke trace campaigns | Keep for debugging harness evolution only. Do not use for paper claims. |
| `smoke` | passed `smoke_*` campaigns | Use as contract/runtime gates only. Do not use as statistical evidence. |
| `incomplete-no-status` | directories without `campaign_status.json` | Treat as debug output until classified by a manifest and status file. |

## Paper Rule

Only `Primary Paper Evidence` campaigns may feed the publication report bundle.
Reference campaigns may be cited in methods/history if clearly labeled.
Failed, partial, smoke, and incomplete campaigns must stay out of paper result tables.
