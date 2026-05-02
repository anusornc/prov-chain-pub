# Benchmark Campaign Summary

| Field | Value |
|---|---|
| Campaign ID | `20260502_trace_supply1000_provchain-neo4j-tigergraph_n3_buildonce` |
| Benchmark family | `trace_query` |
| Dataset | `supply_chain_1000.ttl` |
| Dataset slice | `supply_chain_1000` |
| Evidence role | `tigergraph_translated_property_graph_candidate` |
| Test batch IDs | `BATCH001,BATCH010,BATCH017,BATCH025,BATCH050` |
| Products | `provchain,neo4j,tigergraph` |
| Epoch target | `3` |
| Iterations per epoch | `3` |
| Clean volumes per epoch | `true` |
| Host ports | `provchain=18580, metrics=19590, neo4j_http=18874, neo4j_bolt=19087, fluree=18090, graphdb=17200` |
| Neo4j runtime | `heap_initial=512m, heap_max=1G, pagecache=512m, load_batch_size=100` |

## Epochs

| Epoch | Run ID | Status | Notes |
|---|---|---|---|
| `epoch-001` | `20260502T112625Z` | `passed` | artifacts copied |
| `epoch-002` | `20260502T112744Z` | `passed` | artifacts copied |
| `epoch-003` | `20260502T112901Z` | `passed` | artifacts copied |
