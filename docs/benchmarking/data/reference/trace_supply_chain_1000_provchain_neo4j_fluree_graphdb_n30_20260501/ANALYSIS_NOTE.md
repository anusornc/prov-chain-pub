# GraphDB Trace Full Campaign Analysis - 2026-05-01

## Evidence Scope

- Campaign: `20260501_trace_supply1000_provchain-neo4j-fluree-graphdb_n30`
- Status: `passed`, `30/30` epochs
- Dataset: `supply_chain_1000`
- Products: `ProvChain-Org`, `Neo4j`, `Fluree`, `GraphDB`
- Workload: trace-query plus load/import setup rows

This is publication-facing GraphDB comparator evidence for the `supply_chain_1000` trace-query benchmark family.

## Result Highlights

All recorded rows reached `100.00%` success.

Load/import means:

- ProvChain-Org Turtle RDF Import: `24.133 ms`
- Fluree JSON-LD Import: `509.133 ms`
- GraphDB Turtle RDF Import: `2707.967 ms`
- Neo4j Turtle to Cypher Import: `31224.833 ms`

Trace-query means:

- Simple Product Lookup: ProvChain-Org `0.510 ms`, Neo4j `11.011 ms`, Fluree `10.226 ms`, GraphDB `14.567 ms`
- Multi-hop Traceability: ProvChain-Org `0.492 ms`, Neo4j `15.163 ms`, Fluree `10.418 ms`, GraphDB `9.618 ms`
- Aggregation by Producer: ProvChain-Org `0.587 ms`, Neo4j `30.000 ms`, Fluree `133.887 ms`, GraphDB `13.733 ms`

## Interpretation

ProvChain-Org remains fastest for all three trace-query workloads on this dataset slice. GraphDB is a valid RDF/SPARQL-native comparator and is the strongest external RDF-native system for multi-hop and aggregation in this run, but it remains substantially slower than ProvChain-Org for the measured trace-query paths.

GraphDB import is now valid because the adapter applies the shared Turtle normalizer before RDF4J ingest. The normalizer is a parser-portability step for the benchmark dataset and handles slash-containing CURIE tokens such as `ex:Producer/Farm001`.

## Fairness Boundary

- GraphDB rows are `native-rdf-path` and `native-comparable`.
- Fluree rows are `native-rdf-path` for the benchmark's JSON-LD/RDF path.
- Neo4j rows are `translated-graph-model` and should remain a secondary property-graph baseline.
- Load/import rows remain setup-path evidence, not per-transaction ledger/write evidence.
- ProvChain-Org uses `bulk-turtle-single-block` for dataset admission in this campaign.
