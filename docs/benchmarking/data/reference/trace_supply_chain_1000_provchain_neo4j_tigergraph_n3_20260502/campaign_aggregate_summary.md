# Campaign Aggregate Summary

- Campaign: `20260502_trace_supply1000_provchain-neo4j-tigergraph_n3_buildonce`
- Generated at: `2026-05-02T11:30:20Z`

| Family | Test | System | Path | Metric | Unit | Samples | Success Rate | Mean | p95 | p99 |
|---|---|---|---|---|---|---:|---:|---:|---:|---:|
| `ledger-write` | `Turtle RDF Import` | `ProvChain-Org` | `native-rdf-path` | `load-latency-ms` | `ms` | 3 | 100.00% | 24.000 | 24.000 | 24.000 |
| `ledger-write` | `Turtle to Cypher Import` | `Neo4j` | `secondary-transactional-baseline` | `load-latency-ms` | `ms` | 3 | 100.00% | 31532.667 | 31950.200 | 31965.240 |
| `trace-query` | `Aggregation by Producer` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 9 | 100.00% | 85.889 | 264.400 | 265.680 |
| `trace-query` | `Aggregation by Producer` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 9 | 100.00% | 0.578 | 0.707 | 0.732 |
| `trace-query` | `Aggregation by Producer` | `TigerGraph` | `translated-property-graph-model` | `query-latency-ms` | `ms` | 9 | 100.00% | 3.338 | 3.716 | 3.719 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 45 | 100.00% | 39.222 | 209.800 | 263.120 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 45 | 100.00% | 0.560 | 0.922 | 1.024 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `TigerGraph` | `translated-property-graph-model` | `query-latency-ms` | `ms` | 45 | 100.00% | 4.165 | 4.847 | 6.213 |
| `trace-query` | `Simple Product Lookup` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 45 | 100.00% | 23.000 | 108.000 | 140.360 |
| `trace-query` | `Simple Product Lookup` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 45 | 100.00% | 0.539 | 1.132 | 1.315 |
| `trace-query` | `Simple Product Lookup` | `TigerGraph` | `translated-property-graph-model` | `query-latency-ms` | `ms` | 45 | 100.00% | 3.444 | 4.002 | 4.678 |
