# Campaign Aggregate Summary

- Campaign: `20260501_trace_supply1000_provchain-neo4j-fluree-graphdb_n3_ttlfix`
- Generated at: `2026-05-01T02:11:51Z`

| Family | Test | System | Path | Metric | Unit | Samples | Success Rate | Mean | p95 | p99 |
|---|---|---|---|---|---|---:|---:|---:|---:|---:|
| `ledger-write` | `JSON-LD Import` | `Fluree` | `native-rdf-path` | `load-latency-ms` | `ms` | 3 | 100.00% | 522.667 | 561.600 | 565.120 |
| `ledger-write` | `Turtle RDF Import` | `GraphDB` | `native-rdf-path` | `load-latency-ms` | `ms` | 3 | 100.00% | 2719.667 | 2734.500 | 2736.500 |
| `ledger-write` | `Turtle RDF Import` | `ProvChain-Org` | `native-rdf-path` | `load-latency-ms` | `ms` | 3 | 100.00% | 23.333 | 23.900 | 23.980 |
| `ledger-write` | `Turtle to Cypher Import` | `Neo4j` | `secondary-transactional-baseline` | `load-latency-ms` | `ms` | 3 | 100.00% | 30383.667 | 31553.600 | 31709.920 |
| `trace-query` | `Aggregation by Producer` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 9 | 100.00% | 187.463 | 290.420 | 313.083 |
| `trace-query` | `Aggregation by Producer` | `GraphDB` | `native-rdf-path` | `query-latency-ms` | `ms` | 9 | 100.00% | 24.898 | 56.128 | 59.225 |
| `trace-query` | `Aggregation by Producer` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 9 | 100.00% | 90.111 | 261.000 | 262.600 |
| `trace-query` | `Aggregation by Producer` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 9 | 100.00% | 0.915 | 1.470 | 1.486 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 45 | 100.00% | 13.146 | 21.884 | 28.378 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `GraphDB` | `native-rdf-path` | `query-latency-ms` | `ms` | 45 | 100.00% | 10.393 | 15.994 | 16.354 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 45 | 100.00% | 35.400 | 172.800 | 253.040 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 45 | 100.00% | 0.567 | 0.844 | 0.933 |
| `trace-query` | `Simple Product Lookup` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 45 | 100.00% | 15.589 | 76.228 | 94.086 |
| `trace-query` | `Simple Product Lookup` | `GraphDB` | `native-rdf-path` | `query-latency-ms` | `ms` | 45 | 100.00% | 25.043 | 164.793 | 223.018 |
| `trace-query` | `Simple Product Lookup` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 45 | 100.00% | 22.689 | 100.800 | 133.920 |
| `trace-query` | `Simple Product Lookup` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 45 | 100.00% | 0.619 | 1.181 | 1.270 |
