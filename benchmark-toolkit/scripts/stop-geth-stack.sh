#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOLKIT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
COMPOSE_FILE="${TOOLKIT_DIR}/docker-compose.geth.yml"
GETH_COMPOSE_PROJECT="${GETH_COMPOSE_PROJECT:-provchain-geth}"

docker compose -p "${GETH_COMPOSE_PROJECT}" -f "${COMPOSE_FILE}" down
