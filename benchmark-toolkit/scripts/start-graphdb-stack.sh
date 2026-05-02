#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOLKIT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
COMPOSE_FILE="${TOOLKIT_DIR}/docker-compose.graphdb.yml"

GRAPHDB_COMPOSE_PROJECT="${GRAPHDB_COMPOSE_PROJECT:-provchain-graphdb}"
GRAPHDB_HTTP_PORT="${GRAPHDB_HTTP_PORT:-17200}"
GRAPHDB_URL="${GRAPHDB_URL:-http://localhost:${GRAPHDB_HTTP_PORT}}"
GRAPHDB_IMAGE="${GRAPHDB_IMAGE:-ontotext/graphdb:10.8.13}"
GRAPHDB_HOME_DIR="${GRAPHDB_HOME_DIR:-/tmp/provchain-graphdb-home-10.8}"
GRAPHDB_REPOSITORY="${GRAPHDB_REPOSITORY:-provchain_smoke}"
GDB_JAVA_OPTS="${GDB_JAVA_OPTS:--Xms1g -Xmx2g}"

export GRAPHDB_HTTP_PORT GRAPHDB_URL GRAPHDB_IMAGE GRAPHDB_HOME_DIR GRAPHDB_REPOSITORY GDB_JAVA_OPTS

mkdir -p "${GRAPHDB_HOME_DIR}/conf" "${GRAPHDB_HOME_DIR}/work"

if [ -n "${GRAPHDB_LICENSE_FILE:-}" ]; then
  if [ ! -f "${GRAPHDB_LICENSE_FILE}" ]; then
    printf 'error: GRAPHDB_LICENSE_FILE does not exist: %s\n' "${GRAPHDB_LICENSE_FILE}" >&2
    exit 1
  fi
  cp "${GRAPHDB_LICENSE_FILE}" "${GRAPHDB_HOME_DIR}/conf/graphdb.license"
  cp "${GRAPHDB_LICENSE_FILE}" "${GRAPHDB_HOME_DIR}/work/graphdb.license"
  chmod 600 "${GRAPHDB_HOME_DIR}/conf/graphdb.license" || true
  chmod 600 "${GRAPHDB_HOME_DIR}/work/graphdb.license" || true
  case " ${GDB_JAVA_OPTS} " in
    *" -Dgraphdb.license.file="*) ;;
    *) GDB_JAVA_OPTS="${GDB_JAVA_OPTS} -Dgraphdb.license.file=/opt/graphdb/home/conf/graphdb.license" ;;
  esac
  export GDB_JAVA_OPTS
  printf 'GraphDB license copied into runtime home: %s/conf/graphdb.license and %s/work/graphdb.license\n' \
    "${GRAPHDB_HOME_DIR}" "${GRAPHDB_HOME_DIR}"
else
  printf 'warning: GRAPHDB_LICENSE_FILE is not set; GraphDB will run in free/limited mode when supported by the image.\n' >&2
fi

docker compose -p "${GRAPHDB_COMPOSE_PROJECT}" -f "${COMPOSE_FILE}" up -d

printf 'GraphDB starting on %s (image: %s, compose project: %s)\n' \
  "${GRAPHDB_URL}" "${GRAPHDB_IMAGE}" "${GRAPHDB_COMPOSE_PROJECT}"

for _ in $(seq 1 90); do
  if curl -fsS "${GRAPHDB_URL}/rest/repositories" >/dev/null 2>&1; then
    GRAPHDB_URL="${GRAPHDB_URL}" GRAPHDB_REPOSITORY="${GRAPHDB_REPOSITORY}" "${SCRIPT_DIR}/probe-graphdb.sh"
    exit 0
  fi
  sleep 2
done

printf 'error: GraphDB probe failed after startup wait\n' >&2
docker compose -p "${GRAPHDB_COMPOSE_PROJECT}" -f "${COMPOSE_FILE}" logs --tail=120 graphdb >&2 || true
exit 1
