# Campaign Aggregate Summary

- Campaign: `20260424_trace_supply1000_provchain-neo4j_n30`
- Generated at: `2026-04-24T07:41:26Z`

| Family | Test | System | Samples | Success Rate | Mean ms | p95 ms | p99 ms |
|---|---|---|---:|---:|---:|---:|---:|
| `ledger-write` | `Turtle RDF Import` | `ProvChain-Org` | 30 | 100.00% | 12124.033 | 12310.000 | 12351.660 |
| `ledger-write` | `Turtle to Cypher Import` | `Neo4j` | 30 | 100.00% | 31250.767 | 33106.250 | 33776.470 |
| `trace-query` | `Aggregation by Producer` | `Neo4j` | 300 | 100.00% | 29.413 | 257.100 | 281.020 |
| `trace-query` | `Aggregation by Producer` | `ProvChain-Org` | 300 | 100.00% | 0.482 | 0.632 | 0.970 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `Neo4j` | 1500 | 100.00% | 13.695 | 63.000 | 260.040 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `ProvChain-Org` | 1500 | 100.00% | 0.363 | 0.538 | 0.843 |
| `trace-query` | `Simple Product Lookup` | `Neo4j` | 1500 | 100.00% | 9.389 | 37.000 | 122.060 |
| `trace-query` | `Simple Product Lookup` | `ProvChain-Org` | 1500 | 100.00% | 0.358 | 0.603 | 1.089 |
