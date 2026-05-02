# Benchmark Campaign Summary

| Field | Value |
|---|---|
| Campaign ID | `20260501_trace_supply1000_provchain-neo4j-fluree-graphdb_n3_ttlfix` |
| Benchmark family | `trace_query` |
| Dataset | `supply_chain_1000.ttl` |
| Dataset slice | `supply_chain_1000` |
| Evidence role | `graphdb_comparator_reference_candidate` |
| Test batch IDs | `BATCH001,BATCH010,BATCH017,BATCH025,BATCH050` |
| Products | `provchain,neo4j,fluree,graphdb` |
| Epoch target | `3` |
| Iterations per epoch | `3` |
| Clean volumes per epoch | `true` |
| Host ports | `provchain=18480, metrics=19490, neo4j_http=18774, neo4j_bolt=18987, fluree=18490, graphdb=18500` |
| Neo4j runtime | `heap_initial=512m, heap_max=1G, pagecache=512m, load_batch_size=100` |

## Epochs

| Epoch | Run ID | Status | Notes |
|---|---|---|---|
| `epoch-001` | `20260501T020606Z` | `passed` | artifacts copied |
| `epoch-002` | `20260501T020803Z` | `passed` | artifacts copied |
| `epoch-003` | `20260501T020957Z` | `passed` | artifacts copied |
