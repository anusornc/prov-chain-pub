#!/usr/bin/env bash
set -euo pipefail

TIGERGRAPH_COMPOSE_PROJECT="${TIGERGRAPH_COMPOSE_PROJECT:-provchain-tigergraph}"
TIGERGRAPH_CONTAINER_NAME="${TIGERGRAPH_CONTAINER_NAME:-tigergraph-benchmark}"
TIGERGRAPH_RESTPP_PORT="${TIGERGRAPH_RESTPP_PORT:-19000}"
TIGERGRAPH_URL="${TIGERGRAPH_URL:-http://localhost:${TIGERGRAPH_RESTPP_PORT}}"
TIGERGRAPH_EXEC="${TIGERGRAPH_EXEC:-docker exec ${TIGERGRAPH_CONTAINER_NAME} bash -lc}"
TIGERGRAPH_PATH_PREFIX='export PATH="/home/tigergraph/tigergraph/app/cmd:/home/tigergraph/tigergraph/app/bin:/home/tigergraph/tigergraph/bin:/home/tigergraph/app/cmd:/home/tigergraph/app/bin:$PATH";'

printf 'TigerGraph probe target: %s\n' "${TIGERGRAPH_URL}"

printf '\n--- RESTPP /echo\n'
if curl -fsS "${TIGERGRAPH_URL}/echo"; then
  printf '\n'
else
  printf 'error: TigerGraph RESTPP /echo failed\n' >&2
  exit 1
fi

printf '\n--- gsql version\n'
${TIGERGRAPH_EXEC} "${TIGERGRAPH_PATH_PREFIX} gsql version"

printf '\n--- minimal GSQL fixture\n'
${TIGERGRAPH_EXEC} "${TIGERGRAPH_PATH_PREFIX} gsql \"DROP ALL\"" >/dev/null 2>&1 || true
${TIGERGRAPH_EXEC} "${TIGERGRAPH_PATH_PREFIX} gsql /benchmark/tigergraph/minimal-trace-smoke.gsql"

printf '\nTigerGraph feasibility probe passed.\n'
