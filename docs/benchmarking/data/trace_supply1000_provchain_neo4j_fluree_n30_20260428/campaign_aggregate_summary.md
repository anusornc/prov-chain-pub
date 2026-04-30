# Campaign Aggregate Summary

- Campaign: `20260428_trace_supply1000_provchain-neo4j-fluree_n30`
- Generated at: `2026-04-28T17:07:54Z`

| Family | Test | System | Path | Metric | Unit | Samples | Success Rate | Mean | p95 | p99 |
|---|---|---|---|---|---|---:|---:|---:|---:|---:|
| `ledger-write` | `JSON-LD Import` | `Fluree` | `native-rdf-path` | `load-latency-ms` | `ms` | 30 | 100.00% | 469.767 | 570.300 | 580.810 |
| `ledger-write` | `Turtle RDF Import` | `ProvChain-Org` | `native-rdf-path` | `load-latency-ms` | `ms` | 30 | 100.00% | 12122.067 | 12339.900 | 12405.990 |
| `ledger-write` | `Turtle to Cypher Import` | `Neo4j` | `secondary-transactional-baseline` | `load-latency-ms` | `ms` | 30 | 100.00% | 30715.400 | 32151.950 | 32751.500 |
| `trace-query` | `Aggregation by Producer` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 300 | 100.00% | 127.179 | 238.204 | 297.764 |
| `trace-query` | `Aggregation by Producer` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 300 | 100.00% | 28.477 | 246.050 | 264.070 |
| `trace-query` | `Aggregation by Producer` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 300 | 100.00% | 0.566 | 0.755 | 1.204 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 1500 | 100.00% | 10.060 | 16.588 | 26.856 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 1500 | 100.00% | 14.515 | 64.000 | 253.010 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 1500 | 100.00% | 0.479 | 0.594 | 0.936 |
| `trace-query` | `Simple Product Lookup` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 1500 | 100.00% | 9.722 | 12.069 | 85.969 |
| `trace-query` | `Simple Product Lookup` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 1500 | 100.00% | 10.304 | 36.000 | 112.010 |
| `trace-query` | `Simple Product Lookup` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 1500 | 100.00% | 0.487 | 0.732 | 1.091 |
