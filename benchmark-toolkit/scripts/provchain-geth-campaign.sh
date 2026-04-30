#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TOOLKIT_DIR="${REPO_ROOT}/benchmark-toolkit"
CAMPAIGNS_DIR="${TOOLKIT_DIR}/results/campaigns"
RUNNER="${SCRIPT_DIR}/run-geth-ledger-campaign.sh"

MODE="${1:-}"
if [ $# -gt 0 ]; then
  shift
fi

usage() {
  cat <<'EOF_USAGE'
Usage:
  benchmark-toolkit/scripts/provchain-geth-campaign.sh smoke [options]
  benchmark-toolkit/scripts/provchain-geth-campaign.sh full [options]
  benchmark-toolkit/scripts/provchain-geth-campaign.sh run [options]
  benchmark-toolkit/scripts/provchain-geth-campaign.sh status <campaign_id>

Modes:
  smoke   Run 1 epoch / 1 iteration against a real local Geth RPC.
  full    Run the standard 30 epoch / 10 iteration ProvChain vs Geth ledger campaign.
  run     Run a custom campaign using --epochs, --iterations, and optional --id.
  status  Print status and aggregate summary for an existing campaign.

Required runtime:
  - GETH_RPC_URL must point at a real local Geth development chain.
  - Use benchmark-toolkit/scripts/start-geth-stack.sh to start the provided dev stack.
  - This wrapper does not use a mock Geth server.

Options for smoke/full/run:
  --id <campaign_id>       Override campaign id.
  --epochs <n>             Override epoch count.
  --iterations <n>         Override iterations per epoch.
  --dataset-path <path>    Override local dataset directory.
  --dataset-file <file>    Override dataset file name.
  --geth-rpc-url <url>     Override Geth RPC URL.
  --geth-contract-address <addr>
                            Use a pre-deployed benchmark contract.
  --geth-sender-address <addr>
                            Use an explicit unlocked sender account.
  --provchain-url <url>    Override ProvChain API URL.
  --manage-provchain       Start a fresh ProvChain server per epoch.
  --external-provchain     Use the caller-provided ProvChain API.
  --skip-load              Skip data-loading rows.
  --include-load           Include data-loading rows.
  --skip-provchain         Run Geth-only runtime rows.
  --skip-preflight         Skip local test/probe preflight.
  --preflight              Force local test/probe preflight.
  --help                   Show this help.

Examples:
  benchmark-toolkit/scripts/start-geth-stack.sh
  benchmark-toolkit/scripts/provchain-geth-campaign.sh smoke
  benchmark-toolkit/scripts/provchain-geth-campaign.sh full --id 20260428_ledger_supply1000_provchain-geth_n30
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
    sed -n '1,100p' "${campaign_dir}/campaign_aggregate_summary.md"
  else
    printf 'missing: %s\n' "${campaign_dir}/campaign_aggregate_summary.md"
  fi

  printf '\n[campaign] directory: %s\n' "${campaign_dir}"
}

run_campaign() {
  local campaign_id="$1"
  local epochs="$2"
  local iterations="$3"
  local dataset_path="$4"
  local dataset_file="$5"
  local geth_rpc_url="$6"
  local geth_contract_address="$7"
  local geth_sender_address="$8"
  local provchain_url="$9"
  local manage_provchain="${10}"
  local skip_load="${11}"
  local skip_provchain="${12}"
  local skip_preflight="${13}"

  printf '[campaign] starting %s\n' "${campaign_id}"
  (
    cd "${REPO_ROOT}"
    CAMPAIGN_ID="${campaign_id}" \
    EPOCHS="${epochs}" \
    ITERATIONS="${iterations}" \
    DATASET_PATH="${dataset_path}" \
    DATASET_FILE="${dataset_file}" \
    GETH_RPC_URL="${geth_rpc_url}" \
    GETH_CONTRACT_ADDRESS="${geth_contract_address}" \
    GETH_SENDER_ADDRESS="${geth_sender_address}" \
    PROVCHAIN_URL="${provchain_url}" \
    MANAGE_PROVCHAIN="${manage_provchain}" \
    SKIP_LOAD="${skip_load}" \
    SKIP_PROVCHAIN="${skip_provchain}" \
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
    DEFAULT_ID="smoke_ledger_supply1000_provchain-geth_n1_$(date -u +%Y%m%dT%H%M%SZ)"
    ;;
  full)
    EPOCHS_VALUE="30"
    ITERATIONS_VALUE="10"
    SKIP_PREFLIGHT_VALUE="false"
    DEFAULT_ID="$(date -u +%Y%m%d)_ledger_supply1000_provchain-geth_n30"
    ;;
  run)
    EPOCHS_VALUE="${EPOCHS:-3}"
    ITERATIONS_VALUE="${ITERATIONS:-10}"
    SKIP_PREFLIGHT_VALUE="${SKIP_PREFLIGHT:-false}"
    DEFAULT_ID="$(date -u +%Y%m%d)_ledger_supply1000_provchain-geth_n${EPOCHS_VALUE}"
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
DATASET_PATH_VALUE="${DATASET_PATH:-${TOOLKIT_DIR}/datasets}"
DATASET_FILE_VALUE="${DATASET_FILE:-supply_chain_1000.ttl}"
GETH_RPC_URL_VALUE="${GETH_RPC_URL:-http://localhost:18545}"
GETH_CONTRACT_ADDRESS_VALUE="${GETH_CONTRACT_ADDRESS:-}"
GETH_SENDER_ADDRESS_VALUE="${GETH_SENDER_ADDRESS:-}"
PROVCHAIN_URL_VALUE="${PROVCHAIN_URL:-http://localhost:8080}"
MANAGE_PROVCHAIN_VALUE="${MANAGE_PROVCHAIN:-true}"
SKIP_LOAD_VALUE="${SKIP_LOAD:-true}"
SKIP_PROVCHAIN_VALUE="${SKIP_PROVCHAIN:-false}"

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
    --geth-rpc-url)
      require_value "$1" "${2:-}"
      GETH_RPC_URL_VALUE="$2"
      shift 2
      ;;
    --geth-contract-address)
      require_value "$1" "${2:-}"
      GETH_CONTRACT_ADDRESS_VALUE="$2"
      shift 2
      ;;
    --geth-sender-address)
      require_value "$1" "${2:-}"
      GETH_SENDER_ADDRESS_VALUE="$2"
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
  "${DATASET_PATH_VALUE}" \
  "${DATASET_FILE_VALUE}" \
  "${GETH_RPC_URL_VALUE}" \
  "${GETH_CONTRACT_ADDRESS_VALUE}" \
  "${GETH_SENDER_ADDRESS_VALUE}" \
  "${PROVCHAIN_URL_VALUE}" \
  "${MANAGE_PROVCHAIN_VALUE}" \
  "${SKIP_LOAD_VALUE}" \
  "${SKIP_PROVCHAIN_VALUE}" \
  "${SKIP_PREFLIGHT_VALUE}"
