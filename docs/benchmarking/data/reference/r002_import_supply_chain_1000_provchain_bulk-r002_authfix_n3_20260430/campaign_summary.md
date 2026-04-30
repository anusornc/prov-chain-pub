# Benchmark Campaign Summary

| Field | Value |
|---|---|
| Campaign ID | `20260430_import_supply1000_provchain-bulk-r002_authfix_n3` |
| Benchmark family | `trace_query` |
| Dataset | `supply_chain_1000.ttl` |
| Dataset slice | `supply_chain_1000` |
| Evidence role | `r002_bulk_turtle_import_experiment_not_primary_paper_evidence` |
| Test batch IDs | `BATCH001,BATCH010,BATCH017,BATCH025,BATCH050` |
| Products | `provchain,neo4j,fluree` |
| Epoch target | `3` |
| Iterations per epoch | `3` |
| Clean volumes per epoch | `true` |
| Host ports | `provchain=18380, metrics=19390, neo4j_http=18674, neo4j_bolt=18887, fluree=18390` |
| Neo4j runtime | `heap_initial=1G, heap_max=2G, pagecache=1G, load_batch_size=25` |

## Epochs

| Epoch | Run ID | Status | Notes |
|---|---|---|---|
| `epoch-001` | `20260430T041008Z` | `passed` | artifacts copied |
| `epoch-002` | `20260430T041124Z` | `passed` | artifacts copied |
| `epoch-003` | `20260430T041240Z` | `passed` | artifacts copied |
