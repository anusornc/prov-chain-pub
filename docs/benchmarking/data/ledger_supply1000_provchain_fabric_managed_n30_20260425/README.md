# Curated Benchmark Evidence: 20260425_ledger_supply1000_provchain-fabric_managed_n30_fix3

This directory is a compact publication-facing export of the benchmark campaign:

- Source campaign: `benchmark-toolkit/results/campaigns/20260425_ledger_supply1000_provchain-fabric_managed_n30_fix3`
- Export generated at: `2026-04-25T14:30:17Z`
- Benchmark family: `ledger_write`
- Dataset slice: `supply_chain_1000`
- Compared systems: `ProvChain-Org`, `Hyperledger Fabric`
- Epochs: `30/30` passed
- Iterations per epoch: `10`
- Managed runtime: fresh ProvChain data directory per epoch, Fabric real peer/orderer/gateway/chaincode stack

## Evidence Boundary

This export supports the `ProvChain vs Fabric` ledger/write-path evidence for
benchmark task `B013`. It is based on a real Fabric runtime stack, not the local
contract simulator.

It supports only `ledger-write` claims for this dataset slice. It does not
support trace-query, semantic-reasoning, governance-policy, Fluree, or Geth
claims. The Fabric rows separate submit latency from commit/finality latency;
ProvChain rows report the single-threaded API write workload used by this
campaign.

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
