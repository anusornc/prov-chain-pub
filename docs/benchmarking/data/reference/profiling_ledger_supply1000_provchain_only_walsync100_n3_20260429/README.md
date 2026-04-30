# Curated Benchmark Evidence: 20260429_profile_ledger_supply1000_provchain-only_walsync100_n3

This directory is a compact curated export of the benchmark campaign:

- Source campaign: `benchmark-toolkit/results/campaigns/20260429_profile_ledger_supply1000_provchain-only_walsync100_n3`
- Export generated at: `2026-04-29T06:44:12Z`
- Benchmark family: `ledger_write`
- Dataset slice: `supply_chain_1000`
- Workload: `write`
- Products: `provchain`

## Evidence Boundary

This export is R002 remediation/profiling evidence after RDF snapshot flush
batching, state-root cache optimization, and WAL/index fsync batching. It uses
relaxed durability settings (`PROVCHAIN_WAL_SYNC_INTERVAL=100` and
`PROVCHAIN_CHAIN_INDEX_SYNC_INTERVAL=100`) while the default runtime remains
conservative sync-every-block. Do not use it as primary cross-system paper
comparison evidence, production durable-throughput evidence, or evidence for
other benchmark families, products, or runtime paths without a separate campaign
and validity gate.

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
