# Campaign Aggregate Summary

- Campaign: `20260430_import_supply1000_provchain-bulk-r002_final_n30`
- Generated at: `2026-04-30T05:18:05Z`

| Family | Test | System | Path | Metric | Unit | Samples | Success Rate | Mean | p95 | p99 |
|---|---|---|---|---|---|---:|---:|---:|---:|---:|
| `ledger-write` | `JSON-LD Import` | `Fluree` | `native-rdf-path` | `load-latency-ms` | `ms` | 30 | 100.00% | 478.467 | 560.650 | 575.970 |
| `ledger-write` | `Turtle RDF Import` | `ProvChain-Org` | `native-rdf-path` | `load-latency-ms` | `ms` | 30 | 100.00% | 24.333 | 35.150 | 41.000 |
| `ledger-write` | `Turtle to Cypher Import` | `Neo4j` | `secondary-transactional-baseline` | `load-latency-ms` | `ms` | 30 | 100.00% | 11431.367 | 11855.800 | 11935.670 |
| `trace-query` | `Aggregation by Producer` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 300 | 100.00% | 127.191 | 256.258 | 300.051 |
| `trace-query` | `Aggregation by Producer` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 300 | 100.00% | 29.573 | 251.100 | 268.020 |
| `trace-query` | `Aggregation by Producer` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 300 | 100.00% | 0.548 | 0.818 | 1.000 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 1500 | 100.00% | 9.768 | 15.736 | 23.964 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 1500 | 100.00% | 14.390 | 62.000 | 266.030 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 1500 | 100.00% | 0.450 | 0.571 | 0.802 |
| `trace-query` | `Simple Product Lookup` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 1500 | 100.00% | 9.757 | 13.060 | 89.678 |
| `trace-query` | `Simple Product Lookup` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 1500 | 100.00% | 10.323 | 36.000 | 123.020 |
| `trace-query` | `Simple Product Lookup` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 1500 | 100.00% | 0.456 | 0.676 | 1.028 |
