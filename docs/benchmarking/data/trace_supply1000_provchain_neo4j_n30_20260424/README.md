# Curated Benchmark Evidence: 20260424_trace_supply1000_provchain-neo4j_n30

This directory is a compact publication-facing export of the benchmark campaign:

- Source campaign: `benchmark-toolkit/results/campaigns/20260424_trace_supply1000_provchain-neo4j_n30`
- Export generated at: `2026-04-24T09:36:46Z`
- Benchmark family: `trace_query`
- Dataset slice: `supply_chain_1000`
- Compared systems: `ProvChain-Org`, `Neo4j`

## Evidence Boundary

This export supports the `ProvChain vs Neo4j` trace-query/provenance baseline.
It does not support claims against Fluree, Hyperledger Fabric, or Geth.
Ledger/write rows are retained for transparency but must be interpreted separately
from trace-query rows and with their fairness labels.

## Files

- `campaign_manifest.json` - campaign configuration and validity gate
- `campaign_status.json` - pass/fail campaign status
- `campaign_results.csv` - aggregate metric table for analysis
- `campaign_results.json` - aggregate metric table with metadata
- `campaign_aggregate_summary.md` - human-readable aggregate summary
- `manifest_index.csv` - mapping from epoch/run ids to copied manifests
- `epoch_manifests/` - one manifest per epoch
- `environment_manifests/` - one environment manifest per run
- `run_summaries/` - per-run summary files for audit spot checks

Raw Docker logs and per-iteration JSON files are intentionally not copied here.
They remain in the source campaign directory if deeper forensic review is needed.
