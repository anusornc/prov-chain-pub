#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOLKIT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
COMPOSE_FILE="${TOOLKIT_DIR}/docker-compose.geth.yml"
GETH_COMPOSE_PROJECT="${GETH_COMPOSE_PROJECT:-provchain-geth}"

GETH_RPC_PORT="${GETH_RPC_PORT:-18545}"
GETH_RPC_URL="${GETH_RPC_URL:-http://localhost:${GETH_RPC_PORT}}"
export GETH_RPC_PORT GETH_RPC_URL

docker compose -p "${GETH_COMPOSE_PROJECT}" -f "${COMPOSE_FILE}" up -d

echo "Geth dev chain starting on ${GETH_RPC_URL} (compose project: ${GETH_COMPOSE_PROJECT})"

for _ in $(seq 1 60); do
  if GETH_RPC_URL="${GETH_RPC_URL}" "${SCRIPT_DIR}/probe-geth-rpc.sh" >/dev/null 2>&1; then
    GETH_RPC_URL="${GETH_RPC_URL}" "${SCRIPT_DIR}/probe-geth-rpc.sh"
    exit 0
  fi
  sleep 2
done

echo "error: Geth RPC probe failed after startup wait" >&2
docker compose -p "${GETH_COMPOSE_PROJECT}" -f "${COMPOSE_FILE}" logs --tail=80 geth >&2 || true
exit 1
