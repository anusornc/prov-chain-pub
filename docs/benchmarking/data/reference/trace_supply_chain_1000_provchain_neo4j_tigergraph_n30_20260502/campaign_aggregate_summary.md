# Campaign Aggregate Summary

- Campaign: `20260502_trace_supply1000_provchain-neo4j-tigergraph_n30`
- Generated at: `2026-05-02T12:40:34Z`

| Family | Test | System | Path | Metric | Unit | Samples | Success Rate | Mean | p95 | p99 |
|---|---|---|---|---|---|---:|---:|---:|---:|---:|
| `ledger-write` | `Turtle RDF Import` | `ProvChain-Org` | `native-rdf-path` | `load-latency-ms` | `ms` | 30 | 100.00% | 23.200 | 24.000 | 24.000 |
| `ledger-write` | `Turtle to Cypher Import` | `Neo4j` | `secondary-transactional-baseline` | `load-latency-ms` | `ms` | 30 | 100.00% | 31592.100 | 33048.100 | 34265.980 |
| `trace-query` | `Aggregation by Producer` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 300 | 100.00% | 29.817 | 255.000 | 296.070 |
| `trace-query` | `Aggregation by Producer` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 300 | 100.00% | 0.574 | 1.062 | 1.220 |
| `trace-query` | `Aggregation by Producer` | `TigerGraph` | `translated-property-graph-model` | `query-latency-ms` | `ms` | 300 | 100.00% | 3.276 | 3.686 | 5.557 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 1500 | 100.00% | 14.824 | 66.000 | 258.000 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 1500 | 100.00% | 0.479 | 0.708 | 0.985 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `TigerGraph` | `translated-property-graph-model` | `query-latency-ms` | `ms` | 1500 | 100.00% | 4.038 | 4.767 | 6.467 |
| `trace-query` | `Simple Product Lookup` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 1500 | 100.00% | 10.885 | 38.000 | 131.020 |
| `trace-query` | `Simple Product Lookup` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 1500 | 100.00% | 0.490 | 0.858 | 1.185 |
| `trace-query` | `Simple Product Lookup` | `TigerGraph` | `translated-property-graph-model` | `query-latency-ms` | `ms` | 1500 | 100.00% | 3.222 | 3.740 | 5.311 |
