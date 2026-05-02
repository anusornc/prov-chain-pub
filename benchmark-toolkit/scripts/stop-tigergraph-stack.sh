#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOLKIT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
COMPOSE_FILE="${TOOLKIT_DIR}/docker-compose.tigergraph.yml"

TIGERGRAPH_COMPOSE_PROJECT="${TIGERGRAPH_COMPOSE_PROJECT:-provchain-tigergraph}"

docker compose -p "${TIGERGRAPH_COMPOSE_PROJECT}" -f "${COMPOSE_FILE}" down
