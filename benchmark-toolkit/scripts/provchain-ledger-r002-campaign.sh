#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
CAMPAIGNS_DIR="${REPO_ROOT}/benchmark-toolkit/results/campaigns"
RUNNER="${SCRIPT_DIR}/provchain-fabric-campaign.sh"
EXPORTER="${SCRIPT_DIR}/export-campaign-evidence.sh"
PROFILE_SUMMARIZER="${SCRIPT_DIR}/summarize-provchain-profile.py"

MODE="${1:-}"
if [ $# -gt 0 ]; then
  shift
fi

usage() {
  cat <<'EOF_USAGE'
Usage:
  benchmark-toolkit/scripts/provchain-ledger-r002-campaign.sh smoke [options]
  benchmark-toolkit/scripts/provchain-ledger-r002-campaign.sh profile [options]
  benchmark-toolkit/scripts/provchain-ledger-r002-campaign.sh full [options]
  benchmark-toolkit/scripts/provchain-ledger-r002-campaign.sh status <campaign_id>

Purpose:
  Run the R002 ProvChain-only ledger profiling campaign with cold-load and
  steady-state append phases separated in the benchmark artifacts.

Modes:
  smoke    1 epoch / 1 append iteration, for fast validity checking.
  profile  3 epochs / 3 append iterations, for engineering profiling evidence.
  full     30 epochs / 10 append iterations, for the final R002 rerun.
  status   Print campaign status and aggregate summary.

Options:
  --id <campaign_id>             Override campaign id.
  --epochs <n>                   Override epoch count.
  --iterations <n>               Override append iterations per epoch.
  --durability conservative      WAL/index fsync every block.
  --durability relaxed           WAL/index fsync every 100 blocks.
  --rdf-flush-interval <n>       RDF snapshot flush interval; default 100.
  --export-dir <path>            Curated export directory.
  --no-export                    Do not export/summarize curated evidence.
  --skip-preflight               Skip preflight.
  --preflight                    Force preflight.
  --help                         Show this help.

Examples:
  ./benchmark-toolkit/scripts/provchain-ledger-r002-campaign.sh smoke --durability conservative
  ./benchmark-toolkit/scripts/provchain-ledger-r002-campaign.sh profile --durability conservative
  ./benchmark-toolkit/scripts/provchain-ledger-r002-campaign.sh full --durability conservative
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

safe_slug() {
  printf '%s' "$1" | tr '-' '_' | tr -c 'A-Za-z0-9_' '_'
}

print_campaign_report() {
  "${RUNNER}" status "$1"
}

if [ -z "${MODE}" ] || [ "${MODE}" = "--help" ] || [ "${MODE}" = "-h" ]; then
  usage
  exit 0
fi

case "${MODE}" in
  smoke)
    EPOCHS_VALUE="1"
    ITERATIONS_VALUE="1"
    DEFAULT_ID="smoke_profile_ledger_supply1000_provchain-only_coldsteady_n1_$(date -u +%Y%m%dT%H%M%SZ)"
    ;;
  profile)
    EPOCHS_VALUE="3"
    ITERATIONS_VALUE="3"
    DEFAULT_ID="$(date -u +%Y%m%d)_profile_ledger_supply1000_provchain-only_coldsteady_n3"
    ;;
  full)
    EPOCHS_VALUE="30"
    ITERATIONS_VALUE="10"
    DEFAULT_ID="$(date -u +%Y%m%d)_profile_ledger_supply1000_provchain-only_coldsteady_n30"
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
DURABILITY_VALUE="${R002_DURABILITY:-conservative}"
RDF_FLUSH_INTERVAL_VALUE="${PROVCHAIN_RDF_FLUSH_INTERVAL:-100}"
SKIP_PREFLIGHT_VALUE="${SKIP_PREFLIGHT:-false}"
EXPORT_ENABLED="true"
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
    --durability)
      require_value "$1" "${2:-}"
      DURABILITY_VALUE="$2"
      shift 2
      ;;
    --rdf-flush-interval)
      require_value "$1" "${2:-}"
      RDF_FLUSH_INTERVAL_VALUE="$2"
      shift 2
      ;;
    --export-dir)
      require_value "$1" "${2:-}"
      EXPORT_DIR_VALUE="$2"
      shift 2
      ;;
    --no-export)
      EXPORT_ENABLED="false"
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

case "${DURABILITY_VALUE}" in
  conservative)
    WAL_SYNC_INTERVAL_VALUE="1"
    CHAIN_INDEX_SYNC_INTERVAL_VALUE="1"
    DURABILITY_SUFFIX="conservative"
    ;;
  relaxed)
    WAL_SYNC_INTERVAL_VALUE="100"
    CHAIN_INDEX_SYNC_INTERVAL_VALUE="100"
    DURABILITY_SUFFIX="relaxed"
    ;;
  *)
    die "--durability must be conservative or relaxed"
    ;;
esac

if [ -z "${EXPORT_DIR_VALUE}" ]; then
  EXPORT_DIR_VALUE="${REPO_ROOT}/docs/benchmarking/data/reference/$(safe_slug "${CAMPAIGN_ID_VALUE}")"
fi

printf '[r002] starting %s durability=%s epochs=%s iterations=%s\n' \
  "${CAMPAIGN_ID_VALUE}" "${DURABILITY_SUFFIX}" "${EPOCHS_VALUE}" "${ITERATIONS_VALUE}"

(
  cd "${REPO_ROOT}"
  PROVCHAIN_RDF_FLUSH_INTERVAL="${RDF_FLUSH_INTERVAL_VALUE}" \
  PROVCHAIN_WAL_SYNC_INTERVAL="${WAL_SYNC_INTERVAL_VALUE}" \
  PROVCHAIN_CHAIN_INDEX_SYNC_INTERVAL="${CHAIN_INDEX_SYNC_INTERVAL_VALUE}" \
    "${RUNNER}" run \
      --id "${CAMPAIGN_ID_VALUE}" \
      --epochs "${EPOCHS_VALUE}" \
      --iterations "${ITERATIONS_VALUE}" \
      --skip-fabric \
      --include-load \
      --manage-provchain \
      $(if [ "${SKIP_PREFLIGHT_VALUE}" = "true" ]; then printf '%s' "--skip-preflight"; else printf '%s' "--preflight"; fi)
)

if [ "${EXPORT_ENABLED}" = "true" ]; then
  printf '[r002] exporting curated evidence to %s\n' "${EXPORT_DIR_VALUE}"
  "${EXPORTER}" "${CAMPAIGN_ID_VALUE}" "${EXPORT_DIR_VALUE}"
  python3 "${PROFILE_SUMMARIZER}" "${CAMPAIGNS_DIR}/${CAMPAIGN_ID_VALUE}" "${EXPORT_DIR_VALUE}"
fi

printf '[r002] complete: %s\n' "${CAMPAIGN_ID_VALUE}"
