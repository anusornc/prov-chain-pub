# Benchmark Campaign Summary

| Field | Value |
|---|---|
| Campaign ID | `20260429_scale_trace_supply5000_provchain-neo4j-fluree_n3` |
| Benchmark family | `trace_query` |
| Dataset | `supply_chain_5000.ttl` |
| Dataset slice | `supply_chain_5000` |
| Evidence role | `scale_up_confidence_not_primary_paper_evidence` |
| Test batch IDs | `BATCH001,BATCH010,BATCH017,BATCH025,BATCH050` |
| Products | `provchain,neo4j,fluree` |
| Epoch target | `3` |
| Iterations per epoch | `3` |
| Clean volumes per epoch | `true` |
| Host ports | `provchain=18280, metrics=19290, neo4j_http=18574, neo4j_bolt=18787, fluree=18290` |
| Neo4j runtime | `heap_initial=1G, heap_max=2G, pagecache=1G, load_batch_size=25` |

## Epochs

| Epoch | Run ID | Status | Notes |
|---|---|---|---|
| `epoch-001` | `20260429T165910Z` | `passed` | artifacts copied |
| `epoch-002` | `20260429T170826Z` | `passed` | artifacts copied |
| `epoch-003` | `20260429T171734Z` | `passed` | artifacts copied |
