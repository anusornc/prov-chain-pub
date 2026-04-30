#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
CAMPAIGNS_DIR="${REPO_ROOT}/benchmark-toolkit/results/campaigns"

CAMPAIGN_ID="${1:-}"
OUTPUT_DIR="${2:-}"

usage() {
  cat <<'EOF_USAGE'
Usage:
  benchmark-toolkit/scripts/export-campaign-evidence.sh <campaign_id> [output_dir]

Copies a compact, publication-facing evidence bundle from a benchmark campaign.
The export includes campaign-level aggregate files, campaign manifests, epoch
manifests, and per-run environment manifests. It intentionally excludes raw
Docker logs and per-iteration JSON result files.

Example:
  benchmark-toolkit/scripts/export-campaign-evidence.sh \
    20260424_trace_supply1000_provchain-neo4j_n30 \
    docs/benchmarking/data/trace_supply1000_provchain_neo4j_n30_20260424
EOF_USAGE
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

if [ -z "${CAMPAIGN_ID}" ] || [ "${CAMPAIGN_ID}" = "--help" ] || [ "${CAMPAIGN_ID}" = "-h" ]; then
  usage
  exit 0
fi

CAMPAIGN_DIR="${CAMPAIGNS_DIR}/${CAMPAIGN_ID}"
if [ ! -d "${CAMPAIGN_DIR}" ]; then
  die "campaign directory does not exist: ${CAMPAIGN_DIR}"
fi

if [ -z "${OUTPUT_DIR}" ]; then
  OUTPUT_DIR="${REPO_ROOT}/docs/benchmarking/data/${CAMPAIGN_ID}"
elif [[ "${OUTPUT_DIR}" != /* ]]; then
  OUTPUT_DIR="${REPO_ROOT}/${OUTPUT_DIR}"
fi

if [ -d "${OUTPUT_DIR}" ] && [ -n "$(find "${OUTPUT_DIR}" -mindepth 1 -print -quit)" ]; then
  die "output directory already exists and is not empty: ${OUTPUT_DIR}"
fi

mkdir -p \
  "${OUTPUT_DIR}/epoch_manifests" \
  "${OUTPUT_DIR}/environment_manifests" \
  "${OUTPUT_DIR}/run_summaries"

copy_required() {
  local file_name="$1"
  if [ ! -f "${CAMPAIGN_DIR}/${file_name}" ]; then
    die "required campaign file is missing: ${file_name}"
  fi
  cp "${CAMPAIGN_DIR}/${file_name}" "${OUTPUT_DIR}/${file_name}"
}

copy_optional() {
  local file_name="$1"
  if [ -f "${CAMPAIGN_DIR}/${file_name}" ]; then
    cp "${CAMPAIGN_DIR}/${file_name}" "${OUTPUT_DIR}/${file_name}"
  fi
}

copy_required campaign_manifest.json
copy_required campaign_status.json
copy_required campaign_results.csv
copy_required campaign_results.json
copy_required campaign_aggregate_summary.md
copy_optional campaign_summary.md

MANIFEST_INDEX="${OUTPUT_DIR}/manifest_index.csv"
printf 'epoch_id,run_id,epoch_manifest,environment_manifest,summary_json,summary_md\n' > "${MANIFEST_INDEX}"

while IFS= read -r epoch_manifest; do
  epoch_id="$(basename "$(dirname "${epoch_manifest}")")"
  run_count="$(find "$(dirname "${epoch_manifest}")/runs" -mindepth 1 -maxdepth 1 -type d | wc -l | tr -d '[:space:]')"
  if [ "${run_count}" -ne 1 ]; then
    die "expected exactly one run directory for ${epoch_id}, found ${run_count}; campaign may have been reused"
  fi

  run_dir="$(find "$(dirname "${epoch_manifest}")/runs" -mindepth 1 -maxdepth 1 -type d | sort | head -n 1)"
  run_id="$(basename "${run_dir}")"
  cp "${epoch_manifest}" "${OUTPUT_DIR}/epoch_manifests/${epoch_id}.json"

  env_target=""
  summary_json_target=""
  summary_md_target=""

  if [ -f "${run_dir}/environment_manifest.json" ]; then
    env_target="environment_manifests/${run_id}.json"
    cp "${run_dir}/environment_manifest.json" "${OUTPUT_DIR}/${env_target}"
  fi
  if [ -f "${run_dir}/summary.json" ]; then
    summary_json_target="run_summaries/${run_id}.json"
    cp "${run_dir}/summary.json" "${OUTPUT_DIR}/${summary_json_target}"
  fi
  if [ -f "${run_dir}/summary.md" ]; then
    summary_md_target="run_summaries/${run_id}.md"
    cp "${run_dir}/summary.md" "${OUTPUT_DIR}/${summary_md_target}"
  fi

  printf '%s,%s,%s,%s,%s,%s\n' \
    "${epoch_id}" \
    "${run_id}" \
    "epoch_manifests/${epoch_id}.json" \
    "${env_target}" \
    "${summary_json_target}" \
    "${summary_md_target}" \
    >> "${MANIFEST_INDEX}"
done < <(find "${CAMPAIGN_DIR}/epochs" -mindepth 2 -maxdepth 2 -name epoch_manifest.json | sort)

MANIFEST_SUMMARY="$(
  python3 - "${CAMPAIGN_DIR}/campaign_manifest.json" <<'PY_SUMMARY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    manifest = json.load(handle)

products = ", ".join(f"`{product}`" for product in manifest.get("products", []))
print(manifest.get("benchmark_family", "unknown"))
print(manifest.get("dataset_slice", "unknown"))
print(products or "`unknown`")
print(manifest.get("workload", manifest.get("benchmark_family", "unknown")))
PY_SUMMARY
)"
BENCHMARK_FAMILY="$(printf '%s\n' "${MANIFEST_SUMMARY}" | sed -n '1p')"
DATASET_SLICE="$(printf '%s\n' "${MANIFEST_SUMMARY}" | sed -n '2p')"
PRODUCTS_MARKDOWN="$(printf '%s\n' "${MANIFEST_SUMMARY}" | sed -n '3p')"
WORKLOAD="$(printf '%s\n' "${MANIFEST_SUMMARY}" | sed -n '4p')"

cat > "${OUTPUT_DIR}/README.md" <<EOF_README
# Curated Benchmark Evidence: ${CAMPAIGN_ID}

This directory is a compact publication-facing export of the benchmark campaign:

- Source campaign: \`benchmark-toolkit/results/campaigns/${CAMPAIGN_ID}\`
- Export generated at: \`$(date -u +%Y-%m-%dT%H:%M:%SZ)\`
- Benchmark family: \`${BENCHMARK_FAMILY}\`
- Dataset slice: \`${DATASET_SLICE}\`
- Workload: \`${WORKLOAD}\`
- Products: ${PRODUCTS_MARKDOWN}

## Evidence Boundary

This export supports only the benchmark family, workload, products, and dataset
slice listed above. Do not use it as evidence for other benchmark families,
products, or runtime paths without a separate campaign and validity gate.

## Files

- \`campaign_manifest.json\` - campaign configuration and validity gate
- \`campaign_status.json\` - pass/fail campaign status
- \`campaign_results.csv\` - aggregate metric table for analysis
- \`campaign_results.json\` - aggregate metric table with metadata
- \`campaign_aggregate_summary.md\` - human-readable aggregate summary
- \`manifest_index.csv\` - mapping from epoch/run ids to copied manifests
- \`epoch_manifests/\` - one manifest per epoch
- \`environment_manifests/\` - one environment manifest per run
- \`run_summaries/\` - per-run summary files for audit spot checks

Raw Docker logs and per-iteration JSON files are intentionally not copied here.
They remain in the source campaign directory if deeper forensic review is needed.
EOF_README

printf '[export] wrote evidence bundle: %s\n' "${OUTPUT_DIR}"
printf '[export] raw campaign artifacts can be removed after audit with:\n'
printf '  %s --apply\n' "${REPO_ROOT}/benchmark-toolkit/scripts/cleanup-benchmark-artifacts.sh"
