# Curated Benchmark Evidence: 20260429_profile_semantic_supply1000_provchain-fluree_n3

This directory is a compact publication-facing export of the benchmark campaign:

- Source campaign: `benchmark-toolkit/results/campaigns/20260429_profile_semantic_supply1000_provchain-fluree_n3`
- Export generated at: `2026-04-29T15:49:00Z`
- Benchmark family: `semantic`
- Dataset slice: `supply_chain_1000`
- Workload: `semantic`
- Products: `provchain`, `fluree`

## Evidence Boundary

This export supports only the benchmark family, workload, products, and dataset
slice listed above. Do not use it as evidence for other benchmark families,
products, or runtime paths without a separate campaign and validity gate.

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
