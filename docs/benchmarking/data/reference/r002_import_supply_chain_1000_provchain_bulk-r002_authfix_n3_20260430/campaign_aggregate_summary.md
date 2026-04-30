# Campaign Aggregate Summary

- Campaign: `20260430_import_supply1000_provchain-bulk-r002_authfix_n3`
- Generated at: `2026-04-30T04:13:57Z`

| Family | Test | System | Path | Metric | Unit | Samples | Success Rate | Mean | p95 | p99 |
|---|---|---|---|---|---|---:|---:|---:|---:|---:|
| `ledger-write` | `JSON-LD Import` | `Fluree` | `native-rdf-path` | `load-latency-ms` | `ms` | 3 | 100.00% | 514.333 | 554.500 | 555.700 |
| `ledger-write` | `Turtle RDF Import` | `ProvChain-Org` | `native-rdf-path` | `load-latency-ms` | `ms` | 3 | 100.00% | 23.000 | 23.000 | 23.000 |
| `ledger-write` | `Turtle to Cypher Import` | `Neo4j` | `secondary-transactional-baseline` | `load-latency-ms` | `ms` | 3 | 100.00% | 11594.667 | 11715.900 | 11731.980 |
| `trace-query` | `Aggregation by Producer` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 9 | 100.00% | 198.229 | 290.621 | 303.780 |
| `trace-query` | `Aggregation by Producer` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 9 | 100.00% | 88.889 | 267.600 | 282.320 |
| `trace-query` | `Aggregation by Producer` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 9 | 100.00% | 0.492 | 0.574 | 0.606 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 45 | 100.00% | 13.350 | 22.132 | 26.606 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 45 | 100.00% | 38.289 | 216.400 | 268.440 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 45 | 100.00% | 0.429 | 0.659 | 0.908 |
| `trace-query` | `Simple Product Lookup` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 45 | 100.00% | 15.176 | 58.407 | 97.511 |
| `trace-query` | `Simple Product Lookup` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 45 | 100.00% | 21.467 | 88.800 | 128.080 |
| `trace-query` | `Simple Product Lookup` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 45 | 100.00% | 0.482 | 0.854 | 1.251 |
