# Curated Benchmark Evidence: 20260425_policy_supply1000_fabric_n30

This directory is a compact publication-facing export of the benchmark campaign:

- Source campaign: `benchmark-toolkit/results/campaigns/20260425_policy_supply1000_fabric_n30`
- Export generated at: `2026-04-25T16:05:32Z`
- Benchmark family: `governance_policy`
- Dataset slice: `supply_chain_1000`
- Workload: `policy`
- Products: `fabric`
- Epochs: `30/30` passed
- Iterations per epoch: `10`
- Runtime: real Fabric peer/orderer/gateway/chaincode stack

## Evidence Boundary

This export supports the `Fabric` governance/policy evidence for benchmark task
`B014`. It is based on a real Fabric runtime stack, not the local contract
simulator.

It supports only Fabric policy checks for the `restricted` visibility scenario:
owner-org read, auditor read, and unauthorized org rejection. It does not support
ProvChain policy comparison, trace-query, semantic-reasoning, ledger-write,
Fluree, or Geth claims.

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
