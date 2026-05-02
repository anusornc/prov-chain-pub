# Campaign Aggregate Summary

- Campaign: `20260501_trace_supply1000_provchain-neo4j-fluree-graphdb_n30`
- Generated at: `2026-05-01T03:46:11Z`

| Family | Test | System | Path | Metric | Unit | Samples | Success Rate | Mean | p95 | p99 |
|---|---|---|---|---|---|---:|---:|---:|---:|---:|
| `ledger-write` | `JSON-LD Import` | `Fluree` | `native-rdf-path` | `load-latency-ms` | `ms` | 30 | 100.00% | 509.133 | 570.600 | 576.000 |
| `ledger-write` | `Turtle RDF Import` | `GraphDB` | `native-rdf-path` | `load-latency-ms` | `ms` | 30 | 100.00% | 2707.967 | 2923.100 | 2930.390 |
| `ledger-write` | `Turtle RDF Import` | `ProvChain-Org` | `native-rdf-path` | `load-latency-ms` | `ms` | 30 | 100.00% | 24.133 | 33.350 | 41.710 |
| `ledger-write` | `Turtle to Cypher Import` | `Neo4j` | `secondary-transactional-baseline` | `load-latency-ms` | `ms` | 30 | 100.00% | 31224.833 | 32667.500 | 32896.400 |
| `trace-query` | `Aggregation by Producer` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 300 | 100.00% | 133.887 | 267.464 | 311.512 |
| `trace-query` | `Aggregation by Producer` | `GraphDB` | `native-rdf-path` | `query-latency-ms` | `ms` | 300 | 100.00% | 13.733 | 56.765 | 62.838 |
| `trace-query` | `Aggregation by Producer` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 300 | 100.00% | 30.000 | 254.000 | 271.140 |
| `trace-query` | `Aggregation by Producer` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 300 | 100.00% | 0.587 | 0.962 | 1.201 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 1500 | 100.00% | 10.418 | 17.186 | 28.950 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `GraphDB` | `native-rdf-path` | `query-latency-ms` | `ms` | 1500 | 100.00% | 9.618 | 12.872 | 19.724 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 1500 | 100.00% | 15.163 | 65.000 | 263.010 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 1500 | 100.00% | 0.492 | 0.735 | 0.951 |
| `trace-query` | `Simple Product Lookup` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 1500 | 100.00% | 10.226 | 13.014 | 96.942 |
| `trace-query` | `Simple Product Lookup` | `GraphDB` | `native-rdf-path` | `query-latency-ms` | `ms` | 1500 | 100.00% | 14.567 | 16.336 | 230.547 |
| `trace-query` | `Simple Product Lookup` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 1500 | 100.00% | 11.011 | 38.000 | 129.000 |
| `trace-query` | `Simple Product Lookup` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 1500 | 100.00% | 0.510 | 0.858 | 1.080 |
