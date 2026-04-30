#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "usage: $0 <run_id> [campaign_id]" >&2
  exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
RESULTS_ROOT="${REPO_ROOT}/benchmark-toolkit/results"
TRACE_RESULTS_DIR="${RESULTS_ROOT}/trace"
CAMPAIGNS_DIR="${RESULTS_ROOT}/campaigns"

RUN_ID="$1"
CAMPAIGN_ID="${2:-baseline_trace_supply1000_provchain-neo4j_${RUN_ID}_n1}"
SOURCE_DIR="${TRACE_RESULTS_DIR}/${RUN_ID}"
CAMPAIGN_DIR="${CAMPAIGNS_DIR}/${CAMPAIGN_ID}"
EPOCH_ID="epoch-001"
EPOCH_DIR="${CAMPAIGN_DIR}/epochs/${EPOCH_ID}"
RUN_DIR="${EPOCH_DIR}/runs/${RUN_ID}"

if [ ! -d "${SOURCE_DIR}" ]; then
  echo "trace run directory does not exist: ${SOURCE_DIR}" >&2
  exit 1
fi

mkdir -p "${RUN_DIR}" "${CAMPAIGN_DIR}/logs"

for file_name in environment_manifest.json benchmark_results.json benchmark_results.csv summary.json summary.md; do
  if [ -f "${SOURCE_DIR}/${file_name}" ]; then
    cp "${SOURCE_DIR}/${file_name}" "${RUN_DIR}/${file_name}"
  fi
done

created_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

cat > "${CAMPAIGN_DIR}/campaign_manifest.json" <<EOF_MANIFEST
{
  "campaign_id": "${CAMPAIGN_ID}",
  "created_at_utc": "${created_at}",
  "benchmark_family": "trace_query",
  "dataset_slice": "supply_chain_1000",
  "dataset_file": "supply_chain_1000.ttl",
  "products": ["provchain", "neo4j"],
  "epoch_count_target": 1,
  "iterations_per_epoch": null,
  "clean_volumes_per_epoch": null,
  "preflight_required": true,
  "preflight_skipped": null,
  "validity_gate": [
    "promoted historical run; inspect original run artifacts and summary before publication use"
  ],
  "notes": "This campaign wraps an existing single trace run for organization only. It is not a multi-epoch statistical campaign."
}
EOF_MANIFEST

cat > "${EPOCH_DIR}/epoch_manifest.json" <<EOF_EPOCH
{
  "epoch_id": "${EPOCH_ID}",
  "run_id": "${RUN_ID}",
  "started_at_utc": null,
  "completed_at_utc": "${created_at}",
  "status": "passed",
  "exclusion_reason": "",
  "benchmark_family": "trace_query",
  "dataset_slice": "supply_chain_1000",
  "iterations": null
}
EOF_EPOCH

cat > "${CAMPAIGN_DIR}/campaign_status.json" <<EOF_STATUS
{
  "campaign_id": "${CAMPAIGN_ID}",
  "completed_at_utc": "${created_at}",
  "epoch_count_target": 1,
  "passed_epochs": 1,
  "failed_epochs": 0,
  "status": "passed"
}
EOF_STATUS

cat > "${CAMPAIGN_DIR}/campaign_summary.md" <<EOF_SUMMARY
# Benchmark Campaign Summary

| Field | Value |
|---|---|
| Campaign ID | \`${CAMPAIGN_ID}\` |
| Benchmark family | \`trace_query\` |
| Dataset | \`supply_chain_1000.ttl\` |
| Products | \`provchain,neo4j\` |
| Epoch target | \`1\` |
| Source run | \`${RUN_ID}\` |

## Epochs

| Epoch | Run ID | Status | Notes |
|---|---|---|---|
| \`${EPOCH_ID}\` | \`${RUN_ID}\` | \`passed\` | promoted historical single run |
EOF_SUMMARY

python3 "${SCRIPT_DIR}/summarize-campaign.py" "${CAMPAIGN_DIR}"

echo "promoted trace run into campaign: ${CAMPAIGN_DIR}"
