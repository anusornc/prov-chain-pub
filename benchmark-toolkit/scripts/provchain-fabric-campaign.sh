#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TOOLKIT_DIR="${REPO_ROOT}/benchmark-toolkit"
CAMPAIGNS_DIR="${TOOLKIT_DIR}/results/campaigns"
RUNNER="${SCRIPT_DIR}/run-fabric-ledger-campaign.sh"

MODE="${1:-}"
if [ $# -gt 0 ]; then
  shift
fi

usage() {
  cat <<'EOF_USAGE'
Usage:
  benchmark-toolkit/scripts/provchain-fabric-campaign.sh smoke [options]
  benchmark-toolkit/scripts/provchain-fabric-campaign.sh full [options]
  benchmark-toolkit/scripts/provchain-fabric-campaign.sh run [options]
  benchmark-toolkit/scripts/provchain-fabric-campaign.sh status <campaign_id>

Modes:
  smoke   Run 1 epoch / 1 iteration against a real Fabric gateway.
  full    Run the standard 30 epoch / 10 iteration ProvChain vs Fabric ledger campaign.
  run     Run a custom campaign using --epochs, --iterations, and optional --id.
  status  Print status and aggregate summary for an existing campaign.

Required runtime:
  - FABRIC_GATEWAY_URL must point at a real Fabric benchmark gateway.
  - PROVCHAIN_URL must point at a running ProvChain API unless --manage-provchain or --skip-provchain is used.
  - This wrapper does not start Fabric and does not start the local simulator.

Options for smoke/full/run:
  --id <campaign_id>       Override campaign id.
  --epochs <n>             Override epoch count.
  --iterations <n>         Override iterations per epoch.
  --fabric-batch-size <n>  Override Fabric batch size.
  --dataset-path <path>    Override local dataset directory.
  --dataset-file <file>    Override dataset file name.
  --fabric-gateway-url <u> Override Fabric gateway URL.
  --provchain-url <u>      Override ProvChain API URL.
  --manage-provchain       Start a fresh ProvChain server per epoch.
  --external-provchain     Use the caller-provided ProvChain API.
  --skip-load              Skip data-loading rows.
  --include-load           Include data-loading rows.
  --skip-provchain         Run Fabric-only contract/runtime rows.
  --skip-fabric            Run ProvChain-only profiling rows.
  --skip-preflight         Skip local test/probe preflight.
  --preflight              Force local test/probe preflight.
  --help                   Show this help.

Examples:
  FABRIC_GATEWAY_URL=http://localhost:18800 PROVCHAIN_URL=http://localhost:8080 benchmark-toolkit/scripts/provchain-fabric-campaign.sh smoke
  benchmark-toolkit/scripts/provchain-fabric-campaign.sh full --id 20260424_ledger_supply1000_provchain-fabric_n30
  benchmark-toolkit/scripts/provchain-fabric-campaign.sh run --epochs 5 --iterations 3 --fabric-batch-size 100
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
  local fabric_batch_size="$4"
  local dataset_path="$5"
  local dataset_file="$6"
  local fabric_gateway_url="$7"
  local provchain_url="$8"
  local manage_provchain="$9"
  local skip_load="${10}"
  local skip_provchain="${11}"
  local skip_preflight="${12}"
  local skip_fabric="${13}"

  printf '[campaign] starting %s\n' "${campaign_id}"
  (
    cd "${REPO_ROOT}"
    CAMPAIGN_ID="${campaign_id}" \
    EPOCHS="${epochs}" \
    ITERATIONS="${iterations}" \
    FABRIC_BATCH_SIZE="${fabric_batch_size}" \
    DATASET_PATH="${dataset_path}" \
    DATASET_FILE="${dataset_file}" \
    FABRIC_GATEWAY_URL="${fabric_gateway_url}" \
    PROVCHAIN_URL="${provchain_url}" \
    MANAGE_PROVCHAIN="${manage_provchain}" \
    SKIP_LOAD="${skip_load}" \
    SKIP_PROVCHAIN="${skip_provchain}" \
    SKIP_FABRIC="${skip_fabric}" \
    SKIP_PREFLIGHT="${skip_preflight}" \
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
    SKIP_PREFLIGHT_VALUE="false"
    DEFAULT_ID="smoke_ledger_supply1000_provchain-fabric_n1_$(date -u +%Y%m%dT%H%M%SZ)"
    ;;
  full)
    EPOCHS_VALUE="30"
    ITERATIONS_VALUE="10"
    SKIP_PREFLIGHT_VALUE="false"
    DEFAULT_ID="$(date -u +%Y%m%d)_ledger_supply1000_provchain-fabric_n30"
    ;;
  run)
    EPOCHS_VALUE="${EPOCHS:-3}"
    ITERATIONS_VALUE="${ITERATIONS:-10}"
    SKIP_PREFLIGHT_VALUE="${SKIP_PREFLIGHT:-false}"
    DEFAULT_ID="$(date -u +%Y%m%d)_ledger_supply1000_provchain-fabric_n${EPOCHS_VALUE}"
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
FABRIC_BATCH_SIZE_VALUE="${FABRIC_BATCH_SIZE:-100}"
DATASET_PATH_VALUE="${DATASET_PATH:-${TOOLKIT_DIR}/datasets}"
DATASET_FILE_VALUE="${DATASET_FILE:-supply_chain_1000.ttl}"
FABRIC_GATEWAY_URL_VALUE="${FABRIC_GATEWAY_URL:-http://localhost:18800}"
PROVCHAIN_URL_VALUE="${PROVCHAIN_URL:-http://localhost:8080}"
MANAGE_PROVCHAIN_VALUE="${MANAGE_PROVCHAIN:-true}"
SKIP_LOAD_VALUE="${SKIP_LOAD:-true}"
SKIP_PROVCHAIN_VALUE="${SKIP_PROVCHAIN:-false}"
SKIP_FABRIC_VALUE="${SKIP_FABRIC:-false}"

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
    --fabric-batch-size)
      require_value "$1" "${2:-}"
      FABRIC_BATCH_SIZE_VALUE="$2"
      shift 2
      ;;
    --dataset-path)
      require_value "$1" "${2:-}"
      DATASET_PATH_VALUE="$2"
      shift 2
      ;;
    --dataset-file)
      require_value "$1" "${2:-}"
      DATASET_FILE_VALUE="$2"
      shift 2
      ;;
    --fabric-gateway-url)
      require_value "$1" "${2:-}"
      FABRIC_GATEWAY_URL_VALUE="$2"
      shift 2
      ;;
    --provchain-url)
      require_value "$1" "${2:-}"
      PROVCHAIN_URL_VALUE="$2"
      shift 2
      ;;
    --manage-provchain)
      MANAGE_PROVCHAIN_VALUE="true"
      shift
      ;;
    --external-provchain)
      MANAGE_PROVCHAIN_VALUE="false"
      shift
      ;;
    --skip-load)
      SKIP_LOAD_VALUE="true"
      shift
      ;;
    --include-load)
      SKIP_LOAD_VALUE="false"
      shift
      ;;
    --skip-provchain)
      SKIP_PROVCHAIN_VALUE="true"
      shift
      ;;
    --skip-fabric)
      SKIP_FABRIC_VALUE="true"
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
  "${FABRIC_BATCH_SIZE_VALUE}" \
  "${DATASET_PATH_VALUE}" \
  "${DATASET_FILE_VALUE}" \
  "${FABRIC_GATEWAY_URL_VALUE}" \
  "${PROVCHAIN_URL_VALUE}" \
  "${MANAGE_PROVCHAIN_VALUE}" \
  "${SKIP_LOAD_VALUE}" \
  "${SKIP_PROVCHAIN_VALUE}" \
  "${SKIP_PREFLIGHT_VALUE}" \
  "${SKIP_FABRIC_VALUE}"
