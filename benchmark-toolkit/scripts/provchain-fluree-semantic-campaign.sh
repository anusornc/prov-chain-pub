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
  benchmark-toolkit/scripts/provchain-fluree-semantic-campaign.sh smoke [options]
  benchmark-toolkit/scripts/provchain-fluree-semantic-campaign.sh full [options]
  benchmark-toolkit/scripts/provchain-fluree-semantic-campaign.sh run [options]
  benchmark-toolkit/scripts/provchain-fluree-semantic-campaign.sh status <campaign_id>

Modes:
  smoke   Run a 1 epoch / 1 iteration semantic-admission smoke campaign.
  full    Run the standard 30 epoch / 1 iteration ProvChain vs Fluree semantic campaign.
  run     Run a custom semantic campaign using --epochs, --iterations, and optional --id.
  status  Print campaign status and aggregate summary for an existing campaign.

Options for smoke/full/run:
  --id <campaign_id>       Override campaign id.
  --epochs <n>             Override epoch count.
  --iterations <n>         Override iterations per epoch.
  --dataset-file <file>    RDF/Turtle dataset file under benchmark-toolkit/datasets.
  --dataset-slice <name>   Logical dataset slice name.
  --port-base <n>          Set semantic host port block. Defaults to 18180.
  --skip-preflight         Skip local preflight gate.
  --preflight              Force local preflight gate.
  --keep-volumes           Do not reset Docker volumes between epochs.
  --export-dir <dir>       Export curated evidence to a custom directory.
  --no-export              Do not export after a passing full/run campaign.
  --help                   Show this help.
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
    sed -n '1,100p' "${campaign_dir}/campaign_aggregate_summary.md"
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
  local skip_preflight="$6"
  local clean_volumes="$7"
  local port_base="$8"
  local export_enabled="$9"
  local export_dir="${10}"

  local provchain_http_port="${PROVCHAIN_TRACE_HTTP_PORT:-${port_base}}"
  local provchain_metrics_port="${PROVCHAIN_TRACE_METRICS_PORT:-$((port_base + 1010))}"
  local neo4j_http_port="${NEO4J_TRACE_HTTP_PORT:-$((port_base + 294))}"
  local neo4j_bolt_port="${NEO4J_TRACE_BOLT_PORT:-$((port_base + 507))}"
  local fluree_http_port="${FLUREE_TRACE_HTTP_PORT:-$((port_base + 10))}"

  validate_campaign_id "${campaign_id}"
  check_docker

  printf '[campaign] starting %s\n' "${campaign_id}"
  printf '[campaign] semantic host ports: provchain=%s metrics=%s neo4j_http=%s neo4j_bolt=%s fluree=%s\n' \
    "${provchain_http_port}" \
    "${provchain_metrics_port}" \
    "${neo4j_http_port}" \
    "${neo4j_bolt_port}" \
    "${fluree_http_port}"
  (
    cd "${REPO_ROOT}"
    CAMPAIGN_ID="${campaign_id}" \
    EPOCHS="${epochs}" \
    ITERATIONS="${iterations}" \
    DATASET_FILE="${dataset_file}" \
    DATASET_SLICE="${dataset_slice}" \
    BENCHMARK_FAMILY="semantic" \
    PRODUCTS="provchain,fluree" \
    PROVCHAIN_ONTOLOGY_PACKAGE="${PROVCHAIN_ONTOLOGY_PACKAGE:-config/ontology_package.toml}" \
    PROVCHAIN_SKIP_DEMO_DATA="${PROVCHAIN_SKIP_DEMO_DATA:-true}" \
    RUNNER_MODE_ARGS="--semantic --skip-load" \
    SKIP_NEO4J=true \
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
    DEFAULT_ID="smoke_semantic_supply1000_provchain-fluree_n1_$(date -u +%Y%m%dT%H%M%SZ)"
    ;;
  full)
    EPOCHS_VALUE="30"
    ITERATIONS_VALUE="1"
    SKIP_PREFLIGHT_VALUE="false"
    EXPORT_ENABLED_VALUE="${EXPORT_EVIDENCE:-true}"
    DEFAULT_ID="${DATE_UTC}_semantic_supply1000_provchain-fluree_n30"
    ;;
  run)
    EPOCHS_VALUE="${EPOCHS:-3}"
    ITERATIONS_VALUE="${ITERATIONS:-1}"
    SKIP_PREFLIGHT_VALUE="${SKIP_PREFLIGHT:-false}"
    EXPORT_ENABLED_VALUE="${EXPORT_EVIDENCE:-true}"
    DEFAULT_ID="${DATE_UTC}_semantic_supply1000_provchain-fluree_n${EPOCHS_VALUE}"
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

CAMPAIGN_ID_VALUE="${CAMPAIGN_ID:-${DEFAULT_ID}}"
DATASET_FILE_VALUE="${DATASET_FILE:-supply_chain_1000.ttl}"
DATASET_SLICE_VALUE="${DATASET_SLICE:-supply_chain_1000}"
CLEAN_VOLUMES_VALUE="${CLEAN_VOLUMES:-true}"
PORT_BASE_VALUE="${SEMANTIC_PORT_BASE:-18180}"
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
    --dataset-file)
      require_value "$1" "${2:-}"
      DATASET_FILE_VALUE="$2"
      shift 2
      ;;
    --dataset-slice)
      require_value "$1" "${2:-}"
      DATASET_SLICE_VALUE="$2"
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

if [ -z "${EXPORT_DIR_VALUE}" ]; then
  EXPORT_DIR_VALUE="docs/benchmarking/data/reference/profiling_semantic_${CAMPAIGN_DATASET_LABEL}_provchain_fluree_n${EPOCHS_VALUE}_${DATE_UTC}"
fi

run_campaign \
  "${CAMPAIGN_ID_VALUE}" \
  "${EPOCHS_VALUE}" \
  "${ITERATIONS_VALUE}" \
  "${DATASET_FILE_VALUE}" \
  "${DATASET_SLICE_VALUE}" \
  "${SKIP_PREFLIGHT_VALUE}" \
  "${CLEAN_VOLUMES_VALUE}" \
  "${PORT_BASE_VALUE}" \
  "${EXPORT_ENABLED_VALUE}" \
  "${EXPORT_DIR_VALUE}"
