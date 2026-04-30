# Trace Scale-Up Confidence Profile - 2026-04-30

## Evidence Status

- Clean campaign: `20260429_scale_trace_supply5000_provchain-neo4j-fluree_n3`
- Curated export: `docs/benchmarking/data/reference/scale_trace_supply_chain_5000_provchain_neo4j_fluree_n3_20260429/`
- Status: `passed`, `3/3` epochs
- Completed at: `2026-04-29T17:26:41Z`
- Dataset slice: `supply_chain_5000`
- Dataset SHA-256: `311e0d6b6b89af99039392fd1f3906da5a72f18da20ff5f61ef231c4fee32fb3`
- Iterations per epoch: `3`
- Test batch IDs: `BATCH001,BATCH010,BATCH017,BATCH025,BATCH050`
- Evidence role: `scale_up_confidence_not_primary_paper_evidence`

Runtime notes:

- Neo4j heap initial: `1G`
- Neo4j heap max: `2G`
- Neo4j page cache: `1G`
- Neo4j load batch size: `25`
- Docker volumes were reset between epochs.

## Evidence Hygiene Note

A later rerun reused the same campaign id and completed at `2026-04-30T02:02:34Z`.
That rerun is not used as curated evidence because the campaign directory already
contained the earlier run directories. The resulting aggregate combined old and
new rows, with `runs_observed=6` instead of the expected `3`.

The benchmark scripts now guard this case:

- `benchmark-toolkit/scripts/run-trace-campaign.sh` refuses to run into a non-empty campaign directory.
- `benchmark-toolkit/scripts/export-campaign-evidence.sh` refuses to export an epoch with more than one run directory.

## Scale-Up Results

Load/import rows are included for audit context, not as trace-query evidence.

| Family | Test | System | Samples | Success Rate | Mean ms | p95 ms | p99 ms |
|---|---|---|---:|---:|---:|---:|---:|
| `ledger-write` | `JSON-LD Import` | `Fluree` | 3 | 100.00% | 1011.000 | 1039.500 | 1043.100 |
| `ledger-write` | `Turtle RDF Import` | `ProvChain-Org` | 3 | 100.00% | 367152.333 | 372765.300 | 373325.860 |
| `ledger-write` | `Turtle to Cypher Import` | `Neo4j` | 3 | 100.00% | 104467.333 | 106656.000 | 106919.200 |
| `trace-query` | `Aggregation by Producer` | `Fluree` | 9 | 100.00% | 4575.558 | 4921.156 | 4944.836 |
| `trace-query` | `Aggregation by Producer` | `Neo4j` | 9 | 100.00% | 90.222 | 264.800 | 265.760 |
| `trace-query` | `Aggregation by Producer` | `ProvChain-Org` | 9 | 100.00% | 1.497 | 1.586 | 1.591 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `Fluree` | 45 | 100.00% | 18.860 | 39.930 | 80.179 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `Neo4j` | 45 | 100.00% | 33.822 | 171.600 | 260.920 |
| `trace-query` | `Multi-hop Traceability (10 hops)` | `ProvChain-Org` | 45 | 100.00% | 0.767 | 0.867 | 1.150 |
| `trace-query` | `Simple Product Lookup` | `Fluree` | 45 | 100.00% | 18.494 | 82.027 | 163.748 |
| `trace-query` | `Simple Product Lookup` | `Neo4j` | 45 | 100.00% | 18.756 | 72.800 | 119.160 |
| `trace-query` | `Simple Product Lookup` | `ProvChain-Org` | 45 | 100.00% | 0.660 | 1.189 | 1.331 |

## Scaling Trend Versus `supply_chain_1000`

The primary `supply_chain_1000` trace campaign is
`20260428_trace_supply1000_provchain-neo4j-fluree_n30`.

| Test | System | `supply_chain_1000` Mean ms | `supply_chain_5000` Mean ms | Ratio |
|---|---|---:|---:|---:|
| Simple Product Lookup | ProvChain-Org | 0.487 | 0.660 | 1.35x |
| Simple Product Lookup | Neo4j | 10.304 | 18.756 | 1.82x |
| Simple Product Lookup | Fluree | 9.722 | 18.494 | 1.90x |
| Multi-hop Traceability (10 hops) | ProvChain-Org | 0.479 | 0.767 | 1.60x |
| Multi-hop Traceability (10 hops) | Neo4j | 14.515 | 33.822 | 2.33x |
| Multi-hop Traceability (10 hops) | Fluree | 10.060 | 18.860 | 1.87x |
| Aggregation by Producer | ProvChain-Org | 0.566 | 1.497 | 2.65x |
| Aggregation by Producer | Neo4j | 28.477 | 90.222 | 3.17x |
| Aggregation by Producer | Fluree | 127.179 | 4575.558 | 35.98x |

## Interpretation

The larger-slice trace profile preserves the trace-query ranking observed in the
primary `supply_chain_1000` evidence. ProvChain remains fastest on all three
trace-query scenarios, including the aggregation query where scale sensitivity is
most visible.

The result supports a conservative scale-up confidence claim for trace-query
workloads only. It does not support a ledger/write or data-load superiority
claim. The load/import rows show the opposite pattern for the current native
ProvChain import path: ProvChain remains much slower than Neo4j and Fluree for
bulk dataset admission on this slice.

## Paper Boundary

Use this artifact as reference evidence for ranking stability and scale-up
confidence. Do not promote it into the primary paper tables unless a later
decision explicitly promotes larger-slice campaigns into the publication bundle.
