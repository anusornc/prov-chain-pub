#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOLKIT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

GRAPHDB_URL="${GRAPHDB_URL:-http://localhost:${GRAPHDB_HTTP_PORT:-17200}}"
GRAPHDB_REPOSITORY="${GRAPHDB_REPOSITORY:-provchain_smoke}"
GRAPHDB_RESET_REPOSITORY="${GRAPHDB_RESET_REPOSITORY:-true}"
FIXTURE_FILE="${GRAPHDB_FIXTURE_FILE:-${TOOLKIT_DIR}/graphdb/minimal-trace-fixture.ttl}"
CONFIG_TEMPLATE="${GRAPHDB_REPO_CONFIG_TEMPLATE:-${TOOLKIT_DIR}/graphdb/repository-config.ttl.template}"

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

http_request() {
  local method="$1"
  local url="$2"
  local body_file="$3"
  shift 3

  local response_file
  response_file="$(mktemp)"

  local status
  if [ -n "${body_file}" ]; then
    status="$(curl -sS -o "${response_file}" -w "%{http_code}" -X "${method}" "$@" --data-binary @"${body_file}" "${url}")"
  else
    status="$(curl -sS -o "${response_file}" -w "%{http_code}" -X "${method}" "$@" "${url}")"
  fi

  printf '%s %s\n' "${status}" "${response_file}"
}

print_response() {
  local label="$1"
  local status="$2"
  local response_file="$3"

  printf -- '--- %s\n' "${label}"
  printf 'status: %s\n' "${status}"
  sed -n '1,30p' "${response_file}"
  printf '\n'
}

expect_2xx() {
  local label="$1"
  local status="$2"
  local response_file="$3"
  print_response "${label}" "${status}" "${response_file}"
  if grep -qi "No license was set" "${response_file}"; then
    cat >&2 <<'EOF_LICENSE'

GraphDB responded with "No license was set".
Set GRAPHDB_LICENSE_FILE to a valid GraphDB license file and rerun, for example:

  GRAPHDB_LICENSE_FILE=/path/to/graphdb.license ./benchmark-toolkit/scripts/start-graphdb-stack.sh

Do not commit the license file.
If the comparison does not require GraphDB 11, rerun the default free-mode image first:

  ./benchmark-toolkit/scripts/stop-graphdb-stack.sh
  GRAPHDB_IMAGE=ontotext/graphdb:10.8.13 ./benchmark-toolkit/scripts/start-graphdb-stack.sh
EOF_LICENSE
    die "${label} failed because GraphDB is missing a license"
  fi
  case "${status}" in
    2*) ;;
    *) die "${label} failed with HTTP ${status}" ;;
  esac
}

if [ ! -f "${FIXTURE_FILE}" ]; then
  die "fixture file not found: ${FIXTURE_FILE}"
fi

if [ ! -f "${CONFIG_TEMPLATE}" ]; then
  die "repository config template not found: ${CONFIG_TEMPLATE}"
fi

printf 'GraphDB probe target: %s\n' "${GRAPHDB_URL}"
printf 'Repository: %s\n' "${GRAPHDB_REPOSITORY}"
printf 'Fixture: %s\n\n' "${FIXTURE_FILE}"

read -r status response_file < <(http_request "GET" "${GRAPHDB_URL}/rest/repositories" "" -H "Accept: application/json")
expect_2xx "GET /rest/repositories" "${status}" "${response_file}"
rm -f "${response_file}"

if [ "${GRAPHDB_RESET_REPOSITORY}" = "true" ]; then
  case "${GRAPHDB_REPOSITORY}" in
    provchain_*|provchain-*) ;;
    *) die "refusing to reset non-provchain repository: ${GRAPHDB_REPOSITORY}" ;;
  esac

  read -r status response_file < <(http_request "DELETE" "${GRAPHDB_URL}/rest/repositories/${GRAPHDB_REPOSITORY}" "")
  print_response "DELETE /rest/repositories/${GRAPHDB_REPOSITORY}" "${status}" "${response_file}"
  case "${status}" in
    2*|404) ;;
    *) die "repository reset failed with HTTP ${status}" ;;
  esac
  rm -f "${response_file}"
fi

repo_config="$(mktemp)"
sed "s/__REPOSITORY_ID__/${GRAPHDB_REPOSITORY}/g" "${CONFIG_TEMPLATE}" > "${repo_config}"

response_file="$(mktemp)"
status="$(
  curl -sS -o "${response_file}" -w "%{http_code}" \
    -X POST \
    -H "Content-Type: multipart/form-data" \
    -F "config=@${repo_config}" \
    "${GRAPHDB_URL}/rest/repositories"
)"
expect_2xx "POST /rest/repositories" "${status}" "${response_file}"
rm -f "${response_file}" "${repo_config}"

read -r status response_file < <(
  http_request "POST" "${GRAPHDB_URL}/repositories/${GRAPHDB_REPOSITORY}/statements" "${FIXTURE_FILE}" \
    -H "Content-Type: text/turtle"
)
expect_2xx "POST /repositories/${GRAPHDB_REPOSITORY}/statements" "${status}" "${response_file}"
rm -f "${response_file}"

query='ASK WHERE { <http://example.org/batch/BATCH001> <http://example.org/supplychain/batchId> "BATCH001" . }'
response_file="$(mktemp)"
status="$(
  curl -sS -o "${response_file}" -w "%{http_code}" \
    -G \
    -H "Accept: application/sparql-results+json" \
    --data-urlencode "query=${query}" \
    "${GRAPHDB_URL}/repositories/${GRAPHDB_REPOSITORY}"
)"
expect_2xx "GET /repositories/${GRAPHDB_REPOSITORY}?query=ASK" "${status}" "${response_file}"

if ! grep -Eq '"boolean"[[:space:]]*:[[:space:]]*true' "${response_file}"; then
  die "GraphDB ASK query did not return true"
fi
rm -f "${response_file}"

printf 'GraphDB feasibility probe passed.\n'
