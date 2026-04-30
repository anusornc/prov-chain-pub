# Benchmark Results Directory

This directory stores benchmark artifacts produced by the portable benchmark toolkit.

## Current Layout

```text
benchmark-toolkit/results/
├── trace/
│   ├── <run_id>/
│   └── latest
└── campaigns/
    └── <campaign_id>/
        ├── campaign_manifest.json
        ├── campaign_summary.md
        ├── campaign_status.json
        ├── logs/
        └── epochs/
            └── epoch-001/
                ├── epoch_manifest.json
                ├── benchmark-runner.log
                └── runs/
                    └── <run_id>/
```

`trace/<run_id>` is the current compatibility layout used by the trace benchmark runner.

`campaigns/<campaign_id>` is the planned layout for repeated benchmark campaigns with multiple epochs.

`campaigns/INDEX.md` classifies local campaign directories as primary paper
evidence, reference/superseded evidence, smoke/debug artifacts, failed runs,
partial runs, or incomplete directories.

## Run Artifact Rule

Every run directory should contain:

- `environment_manifest.json`
- `benchmark_results.json`
- `benchmark_results.csv`
- `summary.json`
- `summary.md`
- `raw_logs/` when available

## Evidence Rule

A run is evidence only if it has a manifest, a status label, and a valid benchmark family/fairness label. Debug runs without those fields should not be used for publication claims.

See `docs/benchmarking/BENCHMARK_RESULTS_ORGANIZATION_PLAN_2026-04-24.md` for the full organization policy.

## Paper Evidence Campaigns

Current paper-facing campaign IDs:

- `20260424_trace_supply1000_provchain-neo4j_n30`
- `20260428_trace_supply1000_provchain-neo4j-fluree_n30`
- `20260425_ledger_supply1000_provchain-fabric_managed_n30_fix3`
- `20260428_semantic_supply1000_provchain-fluree_n30`
- `20260428_policy_supply1000_fabric_pack_n30`
- `20260428_ledger_supply1000_provchain-geth_n30_fix1`

Use curated exports under `docs/benchmarking/data/` for paper tables and
figures. Treat raw campaign directories as audit backing, not manuscript inputs.

## Non-Paper Archive Rule

- failed, partial, and incomplete campaign directories are engineering debug artifacts
- passed `smoke_*` campaign directories are runtime/contract gates only
- superseded but valid evidence belongs under `docs/benchmarking/data/reference/`
- new paper evidence requires a passed campaign, curated export, and an entry in `campaigns/INDEX.md`

## Running A Trace Campaign

Run a short smoke campaign:

```bash
EPOCHS=3 ITERATIONS=10 ./benchmark-toolkit/scripts/run-trace-campaign.sh
```

Run a longer campaign:

```bash
EPOCHS=30 ITERATIONS=10 CAMPAIGN_ID=20260424_trace_supply1000_provchain-neo4j_n30 ./benchmark-toolkit/scripts/run-trace-campaign.sh
```

By default the script resets benchmark Docker volumes between epochs to avoid state carryover. Results are kept in this bind-mounted `results/` directory.

## Summarizing A Campaign

After a campaign has completed, aggregate all epoch result CSV files:

```bash
python3 benchmark-toolkit/scripts/summarize-campaign.py benchmark-toolkit/results/campaigns/<campaign_id>
```

The summarizer writes:

- `campaign_results.json`
- `campaign_results.csv`
- `campaign_aggregate_summary.md`

## Promoting An Existing Trace Run

Wrap a historical single run in the campaign layout:

```bash
./benchmark-toolkit/scripts/promote-trace-run-to-campaign.sh 20260422T154555Z
```

This is for organization only. A promoted single run is still not a multi-epoch statistical campaign.
