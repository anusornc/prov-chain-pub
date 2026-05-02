#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOLKIT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
COMPOSE_FILE="${TOOLKIT_DIR}/docker-compose.tigergraph.yml"

TIGERGRAPH_COMPOSE_PROJECT="${TIGERGRAPH_COMPOSE_PROJECT:-provchain-tigergraph}"
TIGERGRAPH_CONTAINER_NAME="${TIGERGRAPH_CONTAINER_NAME:-tigergraph-benchmark}"
TIGERGRAPH_IMAGE="${TIGERGRAPH_IMAGE:-tigergraph/community:4.2.2}"
TIGERGRAPH_RESTPP_PORT="${TIGERGRAPH_RESTPP_PORT:-19000}"
TIGERGRAPH_GRAPHSTUDIO_PORT="${TIGERGRAPH_GRAPHSTUDIO_PORT:-19240}"
TIGERGRAPH_URL="${TIGERGRAPH_URL:-http://localhost:${TIGERGRAPH_RESTPP_PORT}}"
TIGERGRAPH_EXEC="${TIGERGRAPH_EXEC:-docker exec ${TIGERGRAPH_CONTAINER_NAME} bash -lc}"

export TIGERGRAPH_COMPOSE_PROJECT
export TIGERGRAPH_CONTAINER_NAME
export TIGERGRAPH_IMAGE
export TIGERGRAPH_RESTPP_PORT
export TIGERGRAPH_GRAPHSTUDIO_PORT
export TIGERGRAPH_URL

mem_total_mb="$(awk '/MemTotal/ { printf "%d", $2 / 1024 }' /proc/meminfo 2>/dev/null || printf '0')"
cpu_count="$(getconf _NPROCESSORS_ONLN 2>/dev/null || printf '0')"
printf 'TigerGraph resource check: cpus=%s memory_mb=%s\n' "${cpu_count}" "${mem_total_mb}"
if [ "${mem_total_mb}" -gt 0 ] && [ "${mem_total_mb}" -lt 8192 ]; then
  printf 'warning: TigerGraph Docker guidance recommends at least 8GB memory for small local use.\n' >&2
fi

docker compose -p "${TIGERGRAPH_COMPOSE_PROJECT}" -f "${COMPOSE_FILE}" up -d

printf 'TigerGraph container starting (image: %s, compose project: %s)\n' \
  "${TIGERGRAPH_IMAGE}" "${TIGERGRAPH_COMPOSE_PROJECT}"
printf 'RESTPP: %s\nGraphStudio: http://localhost:%s\n' \
  "${TIGERGRAPH_URL}" "${TIGERGRAPH_GRAPHSTUDIO_PORT}"

printf 'Waiting for TigerGraph container to accept docker exec...\n'
gadmin_probe='for dir in /home/tigergraph/tigergraph/app/cmd /home/tigergraph/tigergraph/app/bin /home/tigergraph/tigergraph/bin /home/tigergraph/app/cmd /home/tigergraph/app/bin /usr/local/bin /usr/bin; do test -x "$dir/gadmin" && exit 0; done; command -v gadmin >/dev/null 2>&1'
for _ in $(seq 1 120); do
  state="$(docker inspect -f '{{.State.Status}}' "${TIGERGRAPH_CONTAINER_NAME}" 2>/dev/null || true)"
  if [ "${state}" = "running" ] && ${TIGERGRAPH_EXEC} "${gadmin_probe}" >/dev/null 2>&1; then
    break
  fi
  if [ "${state}" = "exited" ] || [ "${state}" = "dead" ]; then
    printf 'error: TigerGraph container entered state: %s\n' "${state}" >&2
    docker compose -p "${TIGERGRAPH_COMPOSE_PROJECT}" -f "${COMPOSE_FILE}" logs --tail=160 tigergraph >&2 || true
    exit 1
  fi
  sleep 2
done

if ! ${TIGERGRAPH_EXEC} "${gadmin_probe}" >/dev/null 2>&1; then
  printf 'error: TigerGraph container did not become exec-ready\n' >&2
  printf 'diagnostic: files under /home/tigergraph:\n' >&2
  ${TIGERGRAPH_EXEC} 'find /home/tigergraph -maxdepth 4 -type f \( -name gadmin -o -name gsql \) -print 2>/dev/null | sort | head -40' >&2 || true
  docker compose -p "${TIGERGRAPH_COMPOSE_PROJECT}" -f "${COMPOSE_FILE}" logs --tail=160 tigergraph >&2 || true
  exit 1
fi

printf 'Starting TigerGraph services inside the container. This can take several minutes.\n'
${TIGERGRAPH_EXEC} 'export PATH="/home/tigergraph/tigergraph/app/cmd:/home/tigergraph/tigergraph/app/bin:/home/tigergraph/tigergraph/bin:/home/tigergraph/app/cmd:/home/tigergraph/app/bin:$PATH"; gadmin start all'

for _ in $(seq 1 150); do
  if curl -fsS "${TIGERGRAPH_URL}/echo" >/dev/null 2>&1; then
    TIGERGRAPH_URL="${TIGERGRAPH_URL}" \
      TIGERGRAPH_CONTAINER_NAME="${TIGERGRAPH_CONTAINER_NAME}" \
      "${SCRIPT_DIR}/probe-tigergraph.sh"
    exit 0
  fi
  sleep 2
done

printf 'error: TigerGraph RESTPP did not become ready after startup wait\n' >&2
docker compose -p "${TIGERGRAPH_COMPOSE_PROJECT}" -f "${COMPOSE_FILE}" logs --tail=160 tigergraph >&2 || true
exit 1
