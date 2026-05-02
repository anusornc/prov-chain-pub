#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TOOLKIT_DIR="${REPO_ROOT}/benchmark-toolkit"
CAMPAIGNS_DIR="${TOOLKIT_DIR}/results/campaigns"
RUNNER="${SCRIPT_DIR}/run-trace-campaign.sh"
EXPORTER="${SCRIPT_DIR}/export-campaign-evidence.sh"
INSTALLER="${SCRIPT_DIR}/install-tigergraph-trace-model.sh"

MODE="${1:-}"
if [ $# -gt 0 ]; then
  shift
fi

usage() {
  cat <<'EOF_USAGE'
Usage:
  benchmark-toolkit/scripts/provchain-neo4j-tigergraph-campaign.sh smoke [options]
  benchmark-toolkit/scripts/provchain-neo4j-tigergraph-campaign.sh profile [options]
  benchmark-toolkit/scripts/provchain-neo4j-tigergraph-campaign.sh full [options]
  benchmark-toolkit/scripts/provchain-neo4j-tigergraph-campaign.sh run [options]
  benchmark-toolkit/scripts/provchain-neo4j-tigergraph-campaign.sh status <campaign_id>

Modes:
  smoke    Run a 1 epoch / 1 iteration confidence campaign.
  profile  Run a 3 epoch / 3 iteration reference candidate campaign.
  full     Run a 30 epoch / 10 iteration reference candidate campaign.
  run      Run a custom campaign using --epochs, --iterations, and optional --id.
  status   Print campaign status and aggregate summary for an existing campaign.

Options for smoke/profile/full/run:
  --id <campaign_id>          Override campaign id.
  --epochs <n>                Override epoch count.
  --iterations <n>            Override iterations per epoch.
  --dataset-slice <name>      Logical dataset slice. Defaults to supply_chain_1000.
  --dataset-file <file>       Turtle dataset file under benchmark-toolkit/datasets.
  --test-batch-ids <ids>      Comma-separated deterministic batch ids.
  --port-base <n>             Set isolated host port block. Defaults to 18580.
  --skip-install              Do not reinstall the TigerGraph translated model first.
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
    sed -n '1,140p' "${campaign_dir}/campaign_aggregate_summary.md"
  else
    printf 'missing: %s\n' "${campaign_dir}/campaign_aggregate_summary.md"
  fi

  printf '\n[campaign] directory: %s\n' "${campaign_dir}"
}

campaign_passed() {
  local campaign_id="$1"
  local status_file="${CAMPAIGNS_DIR}/${campaign_id}/campaign_status.json"
  [ -f "${status_file}" ] && grep -q '"status": "passed"' "${status_file}"
}

run_campaign() {
  local campaign_id="$1"
  local epochs="$2"
  local iterations="$3"
  local dataset_file="$4"
  local dataset_slice="$5"
  local test_batch_ids="$6"
  local skip_preflight="$7"
  local clean_volumes="$8"
  local port_base="$9"
  local export_enabled="${10}"
  local export_dir="${11}"
  local install_first="${12}"

  local provchain_http_port="${PROVCHAIN_TRACE_HTTP_PORT:-${port_base}}"
  local provchain_metrics_port="${PROVCHAIN_TRACE_METRICS_PORT:-$((port_base + 1010))}"
  local neo4j_http_port="${NEO4J_TRACE_HTTP_PORT:-$((port_base + 294))}"
  local neo4j_bolt_port="${NEO4J_TRACE_BOLT_PORT:-$((port_base + 507))}"

  validate_campaign_id "${campaign_id}"
  check_docker

  if [ "${install_first}" = "true" ]; then
    TIGERGRAPH_DATASET_FILE="${TOOLKIT_DIR}/datasets/${dataset_file}" \
    TIGERGRAPH_GRAPH="${TIGERGRAPH_GRAPH:-ProvChainTrace}" \
      "${INSTALLER}"
  fi

  printf '[campaign] starting %s\n' "${campaign_id}"
  printf '[campaign] tigergraph translated-model ports: provchain=%s metrics=%s neo4j_http=%s neo4j_bolt=%s tigergraph=%s\n' \
    "${provchain_http_port}" \
    "${provchain_metrics_port}" \
    "${neo4j_http_port}" \
    "${neo4j_bolt_port}" \
    "${TIGERGRAPH_URL:-http://host.docker.internal:19000}"

  (
    cd "${REPO_ROOT}"
    CAMPAIGN_ID="${campaign_id}" \
    EPOCHS="${epochs}" \
    ITERATIONS="${iterations}" \
    DATASET_FILE="${dataset_file}" \
    DATASET_SLICE="${dataset_slice}" \
    TEST_BATCH_IDS="${test_batch_ids}" \
    EVIDENCE_ROLE="tigergraph_translated_property_graph_candidate" \
    BENCHMARK_FAMILY="trace_query" \
    PRODUCTS="provchain,neo4j,tigergraph" \
    RUNNER_MODE_ARGS="--query" \
    SKIP_NEO4J=false \
    SKIP_FLUREE=true \
    SKIP_GRAPHDB=true \
    SKIP_TIGERGRAPH=false \
    SKIP_FABRIC=true \
    SKIP_GETH=true \
    SKIP_PREFLIGHT="${skip_preflight}" \
    CLEAN_VOLUMES="${clean_volumes}" \
    BUILD_IMAGES="${BUILD_IMAGES:-once}" \
    PROVCHAIN_TRACE_HTTP_PORT="${provchain_http_port}" \
    PROVCHAIN_TRACE_METRICS_PORT="${provchain_metrics_port}" \
    NEO4J_TRACE_HTTP_PORT="${neo4j_http_port}" \
    NEO4J_TRACE_BOLT_PORT="${neo4j_bolt_port}" \
    TIGERGRAPH_URL="${TIGERGRAPH_URL:-http://host.docker.internal:19000}" \
    TIGERGRAPH_GRAPH="${TIGERGRAPH_GRAPH:-ProvChainTrace}" \
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
    DEFAULT_ID="smoke_trace_supply1000_provchain-neo4j-tigergraph_n1_${DATE_UTC}"
    ;;
  profile)
    EPOCHS_VALUE="3"
    ITERATIONS_VALUE="3"
    SKIP_PREFLIGHT_VALUE="${SKIP_PREFLIGHT:-false}"
    EXPORT_ENABLED_VALUE="${EXPORT_EVIDENCE:-true}"
    DEFAULT_ID="${DATE_UTC}_trace_supply1000_provchain-neo4j-tigergraph_n3"
    ;;
  full)
    EPOCHS_VALUE="30"
    ITERATIONS_VALUE="10"
    SKIP_PREFLIGHT_VALUE="${SKIP_PREFLIGHT:-false}"
    EXPORT_ENABLED_VALUE="${EXPORT_EVIDENCE:-true}"
    DEFAULT_ID="${DATE_UTC}_trace_supply1000_provchain-neo4j-tigergraph_n30"
    ;;
  run)
    EPOCHS_VALUE="${EPOCHS:-3}"
    ITERATIONS_VALUE="${ITERATIONS:-3}"
    SKIP_PREFLIGHT_VALUE="${SKIP_PREFLIGHT:-false}"
    EXPORT_ENABLED_VALUE="${EXPORT_EVIDENCE:-false}"
    DEFAULT_ID="${DATE_UTC}_trace_supply1000_provchain-neo4j-tigergraph_n${EPOCHS_VALUE}"
    ;;
  status)
    campaign_id="${1:-}"
    require_value "status" "${campaign_id}"
    print_campaign_report "${campaign_id}"
    exit 0
    ;;
  *)
    usage >&2
    die "unknown mode: ${MODE}"
    ;;
esac

CAMPAIGN_ID_VALUE="${CAMPAIGN_ID:-${DEFAULT_ID}}"
DATASET_SLICE_VALUE="${DATASET_SLICE:-supply_chain_1000}"
DATASET_FILE_VALUE="${DATASET_FILE:-${DATASET_SLICE_VALUE}.ttl}"
TEST_BATCH_IDS_VALUE="${TEST_BATCH_IDS:-BATCH001,BATCH010,BATCH017,BATCH025,BATCH050}"
CLEAN_VOLUMES_VALUE="${CLEAN_VOLUMES:-true}"
PORT_BASE_VALUE="${PORT_BASE:-18580}"
INSTALL_FIRST_VALUE="${INSTALL_TIGERGRAPH_MODEL:-true}"
EXPORT_DIR_VALUE=""

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
      shift 2
      ;;
    --dataset-file)
      require_value "$1" "${2:-}"
      DATASET_FILE_VALUE="$2"
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
    --skip-install)
      INSTALL_FIRST_VALUE="false"
      shift
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

if [ -z "${EXPORT_DIR_VALUE}" ]; then
  EXPORT_DIR_VALUE="docs/benchmarking/data/reference/trace_${DATASET_SLICE_VALUE}_provchain_neo4j_tigergraph_n${EPOCHS_VALUE}_${DATE_UTC}"
fi

run_campaign \
  "${CAMPAIGN_ID_VALUE}" \
  "${EPOCHS_VALUE}" \
  "${ITERATIONS_VALUE}" \
  "${DATASET_FILE_VALUE}" \
  "${DATASET_SLICE_VALUE}" \
  "${TEST_BATCH_IDS_VALUE}" \
  "${SKIP_PREFLIGHT_VALUE}" \
  "${CLEAN_VOLUMES_VALUE}" \
  "${PORT_BASE_VALUE}" \
  "${EXPORT_ENABLED_VALUE}" \
  "${EXPORT_DIR_VALUE}" \
  "${INSTALL_FIRST_VALUE}"
