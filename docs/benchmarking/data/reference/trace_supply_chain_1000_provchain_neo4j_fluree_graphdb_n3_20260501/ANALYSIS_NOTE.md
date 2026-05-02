# GraphDB Trace Profile Analysis - 2026-05-01

## Evidence Scope

- Campaign: `20260501_trace_supply1000_provchain-neo4j-fluree-graphdb_n3_ttlfix`
- Status: `passed`, `3/3` epochs
- Dataset: `supply_chain_1000`
- Products: `ProvChain-Org`, `Neo4j`, `Fluree`, `GraphDB`
- Evidence role: `graphdb_comparator_reference_candidate`

This is profile evidence, not the final publication-facing `n30` GraphDB comparator campaign.

## Result Highlights

All systems reached `100.00%` success for the recorded rows.

Load means:

- ProvChain-Org Turtle RDF Import: `23.333 ms`
- Fluree JSON-LD Import: `522.667 ms`
- GraphDB Turtle RDF Import: `2719.667 ms`
- Neo4j Turtle to Cypher Import: `30383.667 ms`

Trace-query means:

- Simple Product Lookup: ProvChain-Org `0.619 ms`, Neo4j `22.689 ms`, Fluree `15.589 ms`, GraphDB `25.043 ms`
- Multi-hop Traceability: ProvChain-Org `0.567 ms`, Neo4j `35.400 ms`, Fluree `13.146 ms`, GraphDB `10.393 ms`
- Aggregation by Producer: ProvChain-Org `0.915 ms`, Neo4j `90.111 ms`, Fluree `187.463 ms`, GraphDB `24.898 ms`

## Interpretation

ProvChain-Org remains fastest in this profile for all three trace-query workloads and for bulk Turtle dataset admission. GraphDB is the strongest RDF/SPARQL comparator among the external systems in multi-hop and aggregation, but it remains substantially slower than ProvChain-Org on this slice.

The GraphDB import row is now valid because the adapter applies the shared benchmark Turtle normalizer before RDF4J ingest. This normalizer handles parser portability issues in the benchmark dataset, including slash-containing CURIE tokens such as `ex:Producer/Farm001`.

## Fairness Boundary

- GraphDB rows are labeled `native-rdf-path` and `native-comparable`.
- Neo4j rows remain a translated graph-model baseline.
- ProvChain-Org uses `bulk-turtle-single-block` for dataset admission in this campaign.
- This profile should not replace the primary `n30` evidence tables until `G007` completes.
