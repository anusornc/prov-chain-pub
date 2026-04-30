# Campaign Aggregate Summary

- Campaign: `20260429_scale_trace_supply5000_provchain-neo4j-fluree_n3`
- Generated at: `2026-04-29T17:26:41Z`

| Family | Test | System | Path | Metric | Unit | Samples | Success Rate | Mean | p95 | p99 |
|---|---|---|---|---|---|---:|---:|---:|---:|---:|
| `ledger-write` | `JSON-LD Import` | `Fluree` | `native-rdf-path` | `load-latency-ms` | `ms` | 3 | 100.00% | 1011.000 | 1039.500 | 1043.100 |
| `ledger-write` | `Turtle RDF Import` | `ProvChain-Org` | `native-rdf-path` | `load-latency-ms` | `ms` | 3 | 100.00% | 367152.333 | 372765.300 | 373325.860 |
| `ledger-write` | `Turtle to Cypher Import` | `Neo4j` | `secondary-transactional-baseline` | `load-latency-ms` | `ms` | 3 | 100.00% | 104467.333 | 106656.000 | 106919.200 |
| `trace-query` | `Aggregation by Producer` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 9 | 100.00% | 4575.558 | 4921.156 | 4944.836 |
| `trace-query` | `Aggregation by Producer` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 9 | 100.00% | 90.222 | 264.800 | 265.760 |
| `trace-query` | `Aggregation by Producer` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 9 | 100.00% | 1.497 | 1.586 | 1.591 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 45 | 100.00% | 18.860 | 39.930 | 80.179 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 45 | 100.00% | 33.822 | 171.600 | 260.920 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 45 | 100.00% | 0.767 | 0.867 | 1.150 |
| `trace-query` | `Simple Product Lookup` | `Fluree` | `native-rdf-path` | `query-latency-ms` | `ms` | 45 | 100.00% | 18.494 | 82.027 | 163.748 |
| `trace-query` | `Simple Product Lookup` | `Neo4j` | `translated-graph-model` | `query-latency-ms` | `ms` | 45 | 100.00% | 18.756 | 72.800 | 119.160 |
| `trace-query` | `Simple Product Lookup` | `ProvChain-Org` | `native-rdf-path` | `query-latency-ms` | `ms` | 45 | 100.00% | 0.660 | 1.189 | 1.331 |
