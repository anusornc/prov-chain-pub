#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TOOLKIT_DIR="${REPO_ROOT}/benchmark-toolkit"
CAMPAIGNS_DIR="${TOOLKIT_DIR}/results/campaigns"
RUNNER="${SCRIPT_DIR}/run-trace-campaign.sh"

MODE="${1:-}"
if [ $# -gt 0 ]; then
  shift
fi

usage() {
  cat <<'EOF_USAGE'
Usage:
  benchmark-toolkit/scripts/provchain-neo4j-campaign.sh smoke [options]
  benchmark-toolkit/scripts/provchain-neo4j-campaign.sh full [options]
  benchmark-toolkit/scripts/provchain-neo4j-campaign.sh run [options]
  benchmark-toolkit/scripts/provchain-neo4j-campaign.sh status <campaign_id>

Modes:
  smoke   Run a 1 epoch / 1 iteration Docker smoke campaign.
  full    Run the standard 30 epoch / 10 iteration ProvChain vs Neo4j campaign.
  run     Run a custom campaign using --epochs, --iterations, and optional --id.
  status  Print campaign status and aggregate summary for an existing campaign.

Options for smoke/full/run:
  --id <campaign_id>       Override campaign id.
  --epochs <n>             Override epoch count.
  --iterations <n>         Override iterations per epoch.
  --dataset-file <file>    Dataset file under benchmark-toolkit/datasets.
  --dataset-slice <name>   Logical dataset slice name.
  --skip-preflight         Skip local preflight gate.
  --preflight              Force local preflight gate.
  --keep-volumes           Do not reset Docker volumes between epochs.
  --help                   Show this help.

Examples:
  benchmark-toolkit/scripts/provchain-neo4j-campaign.sh smoke
  benchmark-toolkit/scripts/provchain-neo4j-campaign.sh full
  benchmark-toolkit/scripts/provchain-neo4j-campaign.sh full --id 20260424_trace_supply1000_provchain-neo4j_n30
  benchmark-toolkit/scripts/provchain-neo4j-campaign.sh run --epochs 5 --iterations 3 --id dev_trace_n5
  benchmark-toolkit/scripts/provchain-neo4j-campaign.sh status 20260424_trace_supply1000_provchain-neo4j_n30
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
    sed -n '1,80p' "${campaign_dir}/campaign_aggregate_summary.md"
  else
    printf 'missing: %s\n' "${campaign_dir}/campaign_aggregate_summary.md"
  fi

  printf '\n[campaign] directory: %s\n' "${campaign_dir}"
}

run_campaign() {
  local campaign_id="$1"
  local epochs="$2"
  local iterations="$3"
  local dataset_file="$4"
  local dataset_slice="$5"
  local skip_preflight="$6"
  local clean_volumes="$7"

  check_docker

  printf '[campaign] starting %s\n' "${campaign_id}"
  (
    cd "${REPO_ROOT}"
    CAMPAIGN_ID="${campaign_id}" \
    EPOCHS="${epochs}" \
    ITERATIONS="${iterations}" \
    DATASET_FILE="${dataset_file}" \
    DATASET_SLICE="${dataset_slice}" \
    SKIP_PREFLIGHT="${skip_preflight}" \
    CLEAN_VOLUMES="${clean_volumes}" \
      "${RUNNER}"
  )
  print_campaign_report "${campaign_id}"
}

if [ -z "${MODE}" ] || [ "${MODE}" = "--help" ] || [ "${MODE}" = "-h" ]; then
  usage
  exit 0
fi

case "${MODE}" in
  smoke)
    EPOCHS_VALUE="1"
    ITERATIONS_VALUE="1"
    SKIP_PREFLIGHT_VALUE="true"
    DEFAULT_ID="smoke_trace_supply1000_provchain-neo4j_n1_$(date -u +%Y%m%dT%H%M%SZ)"
    ;;
  full)
    EPOCHS_VALUE="30"
    ITERATIONS_VALUE="10"
    SKIP_PREFLIGHT_VALUE="false"
    DEFAULT_ID="$(date -u +%Y%m%d)_trace_supply1000_provchain-neo4j_n30"
    ;;
  run)
    EPOCHS_VALUE="${EPOCHS:-3}"
    ITERATIONS_VALUE="${ITERATIONS:-10}"
    SKIP_PREFLIGHT_VALUE="${SKIP_PREFLIGHT:-false}"
    DEFAULT_ID="$(date -u +%Y%m%d)_trace_supply1000_provchain-neo4j_n${EPOCHS_VALUE}"
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
    --help|-h)
      usage
      exit 0
      ;;
    *)
      die "unknown option: $1"
      ;;
  esac
done

run_campaign \
  "${CAMPAIGN_ID_VALUE}" \
  "${EPOCHS_VALUE}" \
  "${ITERATIONS_VALUE}" \
  "${DATASET_FILE_VALUE}" \
  "${DATASET_SLICE_VALUE}" \
  "${SKIP_PREFLIGHT_VALUE}" \
  "${CLEAN_VOLUMES_VALUE}"

