#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TOOLKIT_DIR="${REPO_ROOT}/benchmark-toolkit"
CAMPAIGNS_DIR="${TOOLKIT_DIR}/results/campaigns"
RUNNER="${SCRIPT_DIR}/run-trace-campaign.sh"
EXPORTER="${SCRIPT_DIR}/export-campaign-evidence.sh"

MODE="${1:-}"
if [ $# -gt 0 ]; then
  shift
fi

usage() {
  cat <<'EOF_USAGE'
Usage:
  benchmark-toolkit/scripts/provchain-neo4j-fluree-scale-campaign.sh smoke [options]
  benchmark-toolkit/scripts/provchain-neo4j-fluree-scale-campaign.sh profile [options]
  benchmark-toolkit/scripts/provchain-neo4j-fluree-scale-campaign.sh full [options]
  benchmark-toolkit/scripts/provchain-neo4j-fluree-scale-campaign.sh run [options]
  benchmark-toolkit/scripts/provchain-neo4j-fluree-scale-campaign.sh status <campaign_id>

Modes:
  smoke    Run a 1 epoch / 1 iteration scale-up confidence smoke campaign.
  profile  Run the standard 3 epoch / 3 iteration scale-up confidence campaign.
  full     Run a 30 epoch / 10 iteration scale-up confidence campaign.
  run      Run a custom scale-up campaign using --epochs, --iterations, and optional --id.
  status   Print campaign status and aggregate summary for an existing campaign.

Options for smoke/profile/full/run:
  --id <campaign_id>          Override campaign id.
  --epochs <n>                Override epoch count.
  --iterations <n>            Override iterations per epoch.
  --dataset-slice <name>      Logical dataset slice. Defaults to supply_chain_5000.
  --dataset-file <file>       RDF/Turtle dataset file under benchmark-toolkit/datasets.
  --fluree-dataset-file <f>   JSON-LD output path under benchmark-toolkit/datasets.
  --test-batch-ids <ids>      Comma-separated deterministic batch ids.
  --port-base <n>             Set scale-up host port block. Defaults to 18280.
  --skip-preflight            Skip local preflight gate.
  --preflight                 Force local preflight gate.
  --keep-volumes              Do not reset Docker volumes between epochs.
  --export-dir <dir>          Export curated evidence to a custom directory.
  --no-export                 Do not export after a passing profile/full/run.
  --help                      Show this help.
EOF_USAGE
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

require_value() {
  local option="$1"
  local value="${2:-}"
  if [ -z "${value}" ]; then
    die "${option} requires a value"
  fi
}

validate_campaign_id() {
  local campaign_id="$1"

  if [[ ! "${campaign_id}" =~ ^[A-Za-z0-9._-]+$ ]]; then
    die "invalid campaign id: ${campaign_id}. Use only letters, numbers, dot, underscore, and hyphen."
  fi

  if [[ "${campaign_id}" =~ [-_.]$ ]]; then
    die "suspicious campaign id ends with punctuation: ${campaign_id}. Rerun as one line or use a trailing backslash."
  fi
}

check_docker() {
  if ! docker info >/dev/null 2>&1; then
    cat >&2 <<'EOF_DOCKER'
error: Docker daemon is not accessible from this shell.
hint: run `newgrp docker` or use a shell where `docker info` works, then rerun this script.
EOF_DOCKER
    exit 1
  fi
}

print_campaign_report() {
  local campaign_id="$1"
  local campaign_dir="${CAMPAIGNS_DIR}/${campaign_id}"

  if [ ! -d "${campaign_dir}" ]; then
    die "campaign directory does not exist: ${campaign_dir}"
  fi

  printf '\n[campaign] status\n'
  if [ -f "${campaign_dir}/campaign_status.json" ]; then
    cat "${campaign_dir}/campaign_status.json"
  else
    printf 'missing: %s\n' "${campaign_dir}/campaign_status.json"
  fi

  printf '\n[campaign] recent log\n'
  if [ -f "${campaign_dir}/campaign_status.log" ]; then
    tail -n 40 "${campaign_dir}/campaign_status.log"
  else
    printf 'missing: %s\n' "${campaign_dir}/campaign_status.log"
  fi

  printf '\n[campaign] aggregate summary\n'
  if [ -f "${campaign_dir}/campaign_aggregate_summary.md" ]; then
    sed -n '1,120p' "${campaign_dir}/campaign_aggregate_summary.md"
  else
    printf 'missing: %s\n' "${campaign_dir}/campaign_aggregate_summary.md"
  fi

  printf '\n[campaign] directory: %s\n' "${campaign_dir}"
}

campaign_passed() {
  local campaign_id="$1"
  local status_file="${CAMPAIGNS_DIR}/${campaign_id}/campaign_status.json"

  if [ ! -f "${status_file}" ]; then
    return 1
  fi

  grep -q '"status": "passed"' "${status_file}"
}

run_campaign() {
  local campaign_id="$1"
  local epochs="$2"
  local iterations="$3"
  local dataset_file="$4"
  local dataset_slice="$5"
  local fluree_dataset_file="$6"
  local test_batch_ids="$7"
  local skip_preflight="$8"
  local clean_volumes="$9"
  local port_base="${10}"
  local export_enabled="${11}"
  local export_dir="${12}"

  local provchain_http_port="${PROVCHAIN_TRACE_HTTP_PORT:-${port_base}}"
  local provchain_metrics_port="${PROVCHAIN_TRACE_METRICS_PORT:-$((port_base + 1010))}"
  local neo4j_http_port="${NEO4J_TRACE_HTTP_PORT:-$((port_base + 294))}"
  local neo4j_bolt_port="${NEO4J_TRACE_BOLT_PORT:-$((port_base + 507))}"
  local fluree_http_port="${FLUREE_TRACE_HTTP_PORT:-$((port_base + 10))}"
  local neo4j_heap_initial="${NEO4J_HEAP_INITIAL:-1G}"
  local neo4j_heap_max="${NEO4J_HEAP_MAX:-2G}"
  local neo4j_pagecache="${NEO4J_PAGECACHE:-1G}"
  local neo4j_load_batch_size="${NEO4J_LOAD_BATCH_SIZE:-25}"

  validate_campaign_id "${campaign_id}"
  check_docker

  printf '[campaign] starting %s\n' "${campaign_id}"
  printf '[campaign] scale host ports: provchain=%s metrics=%s neo4j_http=%s neo4j_bolt=%s fluree=%s\n' \
    "${provchain_http_port}" \
    "${provchain_metrics_port}" \
    "${neo4j_http_port}" \
    "${neo4j_bolt_port}" \
    "${fluree_http_port}"
  printf '[campaign] scale Neo4j runtime: heap_initial=%s heap_max=%s pagecache=%s load_batch_size=%s\n' \
    "${neo4j_heap_initial}" \
    "${neo4j_heap_max}" \
    "${neo4j_pagecache}" \
    "${neo4j_load_batch_size}"

  (
    cd "${REPO_ROOT}"
    CAMPAIGN_ID="${campaign_id}" \
    EPOCHS="${epochs}" \
    ITERATIONS="${iterations}" \
    DATASET_FILE="${dataset_file}" \
    DATASET_SLICE="${dataset_slice}" \
    FLUREE_DATASET_FILE="${fluree_dataset_file}" \
    TEST_BATCH_IDS="${test_batch_ids}" \
    EVIDENCE_ROLE="scale_up_confidence_not_primary_paper_evidence" \
    BENCHMARK_FAMILY="trace_query" \
    PRODUCTS="provchain,neo4j,fluree" \
    RUNNER_MODE_ARGS="--query" \
    SKIP_NEO4J=false \
    SKIP_FLUREE=false \
    SKIP_FABRIC=true \
    SKIP_GETH=true \
    SKIP_PREFLIGHT="${skip_preflight}" \
    CLEAN_VOLUMES="${clean_volumes}" \
    PROVCHAIN_TRACE_HTTP_PORT="${provchain_http_port}" \
    PROVCHAIN_TRACE_METRICS_PORT="${provchain_metrics_port}" \
    NEO4J_TRACE_HTTP_PORT="${neo4j_http_port}" \
    NEO4J_TRACE_BOLT_PORT="${neo4j_bolt_port}" \
    FLUREE_TRACE_HTTP_PORT="${fluree_http_port}" \
    NEO4J_HEAP_INITIAL="${neo4j_heap_initial}" \
    NEO4J_HEAP_MAX="${neo4j_heap_max}" \
    NEO4J_PAGECACHE="${neo4j_pagecache}" \
    NEO4J_LOAD_BATCH_SIZE="${neo4j_load_batch_size}" \
      "${RUNNER}"
  )

  print_campaign_report "${campaign_id}"

  if [ "${export_enabled}" = "true" ]; then
    if campaign_passed "${campaign_id}"; then
      "${EXPORTER}" "${campaign_id}" "${export_dir}"
    else
      die "campaign did not pass; not exporting evidence"
    fi
  fi
}

if [ -z "${MODE}" ] || [ "${MODE}" = "--help" ] || [ "${MODE}" = "-h" ]; then
  usage
  exit 0
fi

DATE_UTC="$(date -u +%Y%m%d)"

case "${MODE}" in
  smoke)
    EPOCHS_VALUE="1"
    ITERATIONS_VALUE="1"
    SKIP_PREFLIGHT_VALUE="true"
    EXPORT_ENABLED_VALUE="false"
    DEFAULT_ID="smoke_trace_supply5000_provchain-neo4j-fluree_n1_${DATE_UTC}"
    ;;
  profile)
    EPOCHS_VALUE="3"
    ITERATIONS_VALUE="3"
    SKIP_PREFLIGHT_VALUE="${SKIP_PREFLIGHT:-false}"
    EXPORT_ENABLED_VALUE="true"
    DEFAULT_ID="${DATE_UTC}_scale_trace_supply5000_provchain-neo4j-fluree_n3"
    ;;
  full)
    EPOCHS_VALUE="30"
    ITERATIONS_VALUE="10"
    SKIP_PREFLIGHT_VALUE="${SKIP_PREFLIGHT:-false}"
    EXPORT_ENABLED_VALUE="true"
    DEFAULT_ID="${DATE_UTC}_scale_trace_supply5000_provchain-neo4j-fluree_n30"
    ;;
  run)
    EPOCHS_VALUE="${EPOCHS:-3}"
    ITERATIONS_VALUE="${ITERATIONS:-3}"
    SKIP_PREFLIGHT_VALUE="${SKIP_PREFLIGHT:-false}"
    EXPORT_ENABLED_VALUE="${EXPORT_EVIDENCE:-true}"
    DEFAULT_ID="${DATE_UTC}_scale_trace_supply5000_provchain-neo4j-fluree_n${EPOCHS_VALUE}"
    ;;
  status)
    CAMPAIGN_ID_VALUE="${1:-}"
    require_value "status" "${CAMPAIGN_ID_VALUE}"
    print_campaign_report "${CAMPAIGN_ID_VALUE}"
    exit 0
    ;;
  *)
    usage >&2
    die "unknown mode: ${MODE}"
    ;;
esac

CAMPAIGN_ID_VALUE="${CAMPAIGN_ID:-}"
DATASET_SLICE_VALUE="${DATASET_SLICE:-supply_chain_5000}"
DATASET_FILE_VALUE="${DATASET_FILE:-${DATASET_SLICE_VALUE}.ttl}"
FLUREE_DATASET_FILE_VALUE="${FLUREE_DATASET_FILE:-translated/fluree/${DATASET_SLICE_VALUE}.jsonld}"
TEST_BATCH_IDS_VALUE="${TEST_BATCH_IDS:-BATCH001,BATCH010,BATCH017,BATCH025,BATCH050}"
CLEAN_VOLUMES_VALUE="${CLEAN_VOLUMES:-true}"
PORT_BASE_VALUE="${SCALE_PORT_BASE:-18280}"
EXPORT_DIR_VALUE="${EXPORT_DIR:-}"

while [ $# -gt 0 ]; do
  case "$1" in
    --id)
      require_value "$1" "${2:-}"
      CAMPAIGN_ID_VALUE="$2"
      shift 2
      ;;
    --epochs)
      require_value "$1" "${2:-}"
      EPOCHS_VALUE="$2"
      shift 2
      ;;
    --iterations)
      require_value "$1" "${2:-}"
      ITERATIONS_VALUE="$2"
      shift 2
      ;;
    --dataset-slice)
      require_value "$1" "${2:-}"
      DATASET_SLICE_VALUE="$2"
      DATASET_FILE_VALUE="${DATASET_FILE:-${DATASET_SLICE_VALUE}.ttl}"
      FLUREE_DATASET_FILE_VALUE="${FLUREE_DATASET_FILE:-translated/fluree/${DATASET_SLICE_VALUE}.jsonld}"
      shift 2
      ;;
    --dataset-file)
      require_value "$1" "${2:-}"
      DATASET_FILE_VALUE="$2"
      shift 2
      ;;
    --fluree-dataset-file)
      require_value "$1" "${2:-}"
      FLUREE_DATASET_FILE_VALUE="$2"
      shift 2
      ;;
    --test-batch-ids)
      require_value "$1" "${2:-}"
      TEST_BATCH_IDS_VALUE="$2"
      shift 2
      ;;
    --port-base)
      require_value "$1" "${2:-}"
      PORT_BASE_VALUE="$2"
      shift 2
      ;;
    --skip-preflight)
      SKIP_PREFLIGHT_VALUE="true"
      shift
      ;;
    --preflight)
      SKIP_PREFLIGHT_VALUE="false"
      shift
      ;;
    --keep-volumes)
      CLEAN_VOLUMES_VALUE="false"
      shift
      ;;
    --export-dir)
      require_value "$1" "${2:-}"
      EXPORT_DIR_VALUE="$2"
      shift 2
      ;;
    --no-export)
      EXPORT_ENABLED_VALUE="false"
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      die "unknown option: $1"
      ;;
  esac
done

CAMPAIGN_DATASET_LABEL="${DATASET_SLICE_VALUE#supply_chain_}"
if [ "${CAMPAIGN_DATASET_LABEL}" != "${DATASET_SLICE_VALUE}" ]; then
  CAMPAIGN_DATASET_LABEL="supply${CAMPAIGN_DATASET_LABEL}"
else
  CAMPAIGN_DATASET_LABEL="${DATASET_SLICE_VALUE//_/-}"
fi

if [ -z "${CAMPAIGN_ID_VALUE}" ]; then
  case "${MODE}" in
    smoke)
      CAMPAIGN_ID_VALUE="smoke_trace_${CAMPAIGN_DATASET_LABEL}_provchain-neo4j-fluree_n1_${DATE_UTC}"
      ;;
    profile|run)
      CAMPAIGN_ID_VALUE="${DATE_UTC}_scale_trace_${CAMPAIGN_DATASET_LABEL}_provchain-neo4j-fluree_n${EPOCHS_VALUE}"
      ;;
    full)
      CAMPAIGN_ID_VALUE="${DATE_UTC}_scale_trace_${CAMPAIGN_DATASET_LABEL}_provchain-neo4j-fluree_n30"
      ;;
  esac
fi

if [ -z "${EXPORT_DIR_VALUE}" ]; then
  EXPORT_DIR_VALUE="docs/benchmarking/data/reference/scale_trace_${DATASET_SLICE_VALUE}_provchain_neo4j_fluree_n${EPOCHS_VALUE}_${DATE_UTC}"
fi

run_campaign \
  "${CAMPAIGN_ID_VALUE}" \
  "${EPOCHS_VALUE}" \
  "${ITERATIONS_VALUE}" \
  "${DATASET_FILE_VALUE}" \
  "${DATASET_SLICE_VALUE}" \
  "${FLUREE_DATASET_FILE_VALUE}" \
  "${TEST_BATCH_IDS_VALUE}" \
  "${SKIP_PREFLIGHT_VALUE}" \
  "${CLEAN_VOLUMES_VALUE}" \
  "${PORT_BASE_VALUE}" \
  "${EXPORT_ENABLED_VALUE}" \
  "${EXPORT_DIR_VALUE}"
