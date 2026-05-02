#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOLKIT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
COMPOSE_FILE="${TOOLKIT_DIR}/docker-compose.graphdb.yml"
GRAPHDB_COMPOSE_PROJECT="${GRAPHDB_COMPOSE_PROJECT:-provchain-graphdb}"

docker compose -p "${GRAPHDB_COMPOSE_PROJECT}" -f "${COMPOSE_FILE}" down
