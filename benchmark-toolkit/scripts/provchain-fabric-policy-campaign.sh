#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
  cat <<'EOF_USAGE'
Usage:
  benchmark-toolkit/scripts/provchain-fabric-policy-campaign.sh smoke [options]
  benchmark-toolkit/scripts/provchain-fabric-policy-campaign.sh full [options]
  benchmark-toolkit/scripts/provchain-fabric-policy-campaign.sh run [options]
  benchmark-toolkit/scripts/provchain-fabric-policy-campaign.sh status <campaign_id>

Modes:
  smoke   Run 1 epoch / 1 iteration ProvChain+Fabric policy campaign.
  full    Run the standard 30 epoch / 10 iteration ProvChain+Fabric policy campaign.
  run     Run a custom policy campaign using --epochs, --iterations, and optional --id.
  status  Print status and aggregate summary for an existing campaign.

Required runtime:
  - FABRIC_GATEWAY_URL must point at a real Fabric benchmark gateway.
  - PROVCHAIN_URL must point at a running ProvChain API unless --manage-provchain is used.
  - This wrapper does not start Fabric. It can start ProvChain only when --manage-provchain is passed.

Defaults:
  - WORKLOAD=policy
  - PRODUCTS=provchain,fabric
  - SKIP_PROVCHAIN=false
  - SKIP_LOAD=true
  - MANAGE_PROVCHAIN=false

Examples:
  FABRIC_GATEWAY_URL=http://localhost:18800 benchmark-toolkit/scripts/provchain-fabric-policy-campaign.sh smoke --manage-provchain
  benchmark-toolkit/scripts/provchain-fabric-policy-campaign.sh full --id 20260425_policy_supply1000_fabric_n30
EOF_USAGE
}

export WORKLOAD="${WORKLOAD:-policy}"
export FABRIC_RESULTS_DIR="${FABRIC_RESULTS_DIR:-${SCRIPT_DIR}/../results/fabric-policy}"
export BENCHMARK_FAMILY="${BENCHMARK_FAMILY:-governance_policy}"
export PRODUCTS="${PRODUCTS:-provchain,fabric}"
export SKIP_PROVCHAIN="${SKIP_PROVCHAIN:-false}"
export MANAGE_PROVCHAIN="${MANAGE_PROVCHAIN:-false}"
export SKIP_LOAD="${SKIP_LOAD:-true}"

MODE="${1:-}"
if [ $# -gt 0 ]; then
  shift
fi

case "${MODE}" in
  smoke)
    DEFAULT_ID="smoke_policy_supply1000_provchain-fabric_n1_$(date -u +%Y%m%dT%H%M%SZ)"
    ;;
  full)
    DEFAULT_ID="$(date -u +%Y%m%d)_policy_supply1000_provchain-fabric_n30"
    ;;
  run)
    DEFAULT_ID="$(date -u +%Y%m%d)_policy_supply1000_provchain-fabric_n${EPOCHS:-3}"
    ;;
  status)
    exec "${SCRIPT_DIR}/provchain-fabric-campaign.sh" status "$@"
    ;;
  --help|-h|"")
    usage
    exit 0
    ;;
  *)
    printf 'error: unknown mode: %s\n' "${MODE}" >&2
    exit 1
    ;;
esac

CAMPAIGN_ID="${CAMPAIGN_ID:-${DEFAULT_ID}}" \
  "${SCRIPT_DIR}/provchain-fabric-campaign.sh" "${MODE}" "$@"
