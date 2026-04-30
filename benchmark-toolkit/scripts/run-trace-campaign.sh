#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TOOLKIT_DIR="${REPO_ROOT}/benchmark-toolkit"
COMPOSE_FILE="${TOOLKIT_DIR}/docker-compose.trace.yml"
RESULTS_ROOT="${TOOLKIT_DIR}/results"
TRACE_RESULTS_DIR="${RESULTS_ROOT}/trace"
CAMPAIGNS_DIR="${RESULTS_ROOT}/campaigns"

EPOCHS="${EPOCHS:-3}"
ITERATIONS="${ITERATIONS:-10}"
DATASET_FILE="${DATASET_FILE:-supply_chain_1000.ttl}"
DATASET_SLICE="${DATASET_SLICE:-supply_chain_1000}"
FLUREE_DATASET_FILE="${FLUREE_DATASET_FILE:-translated/fluree/${DATASET_SLICE}.jsonld}"
TEST_BATCH_IDS="${TEST_BATCH_IDS:-BATCH001,BATCH010,BATCH017,BATCH025,BATCH050}"
EVIDENCE_ROLE="${EVIDENCE_ROLE:-primary_or_reference_per_campaign_index}"
BENCHMARK_FAMILY="${BENCHMARK_FAMILY:-trace_query}"
PRODUCTS="${PRODUCTS:-provchain,neo4j}"
RUNNER_MODE_ARGS="${RUNNER_MODE_ARGS:---query}"
SKIP_NEO4J="${SKIP_NEO4J:-${BENCHMARK_SKIP_NEO4J:-false}}"
SKIP_FLUREE="${SKIP_FLUREE:-${BENCHMARK_SKIP_FLUREE:-true}}"
SKIP_FABRIC="${SKIP_FABRIC:-${BENCHMARK_SKIP_FABRIC:-true}}"
SKIP_GETH="${SKIP_GETH:-${BENCHMARK_SKIP_GETH:-true}}"
SKIP_PREFLIGHT="${SKIP_PREFLIGHT:-false}"
CLEAN_VOLUMES="${CLEAN_VOLUMES:-true}"
PROVCHAIN_TRACE_HTTP_PORT="${PROVCHAIN_TRACE_HTTP_PORT:-18080}"
PROVCHAIN_TRACE_METRICS_PORT="${PROVCHAIN_TRACE_METRICS_PORT:-19090}"
NEO4J_TRACE_HTTP_PORT="${NEO4J_TRACE_HTTP_PORT:-17474}"
NEO4J_TRACE_BOLT_PORT="${NEO4J_TRACE_BOLT_PORT:-17687}"
FLUREE_TRACE_HTTP_PORT="${FLUREE_TRACE_HTTP_PORT:-18090}"
PROVCHAIN_IMPORT_MODE="${PROVCHAIN_IMPORT_MODE:-bulk-turtle-single-block}"
NEO4J_HEAP_INITIAL="${NEO4J_HEAP_INITIAL:-512m}"
NEO4J_HEAP_MAX="${NEO4J_HEAP_MAX:-1G}"
NEO4J_PAGECACHE="${NEO4J_PAGECACHE:-512m}"
NEO4J_LOAD_BATCH_SIZE="${NEO4J_LOAD_BATCH_SIZE:-100}"
CAMPAIGN_ID="${CAMPAIGN_ID:-$(date -u +%Y%m%d)_trace_${DATASET_SLICE}_provchain-neo4j_n${EPOCHS}}"
CAMPAIGN_DIR="${CAMPAIGNS_DIR}/${CAMPAIGN_ID}"
EPOCHS_DIR="${CAMPAIGN_DIR}/epochs"
LOGS_DIR="${CAMPAIGN_DIR}/logs"
STATUS_LOG="${CAMPAIGN_DIR}/campaign_status.log"

validate_campaign_id() {
  local campaign_id="$1"

  if [[ ! "${campaign_id}" =~ ^[A-Za-z0-9._-]+$ ]]; then
    printf 'error: invalid CAMPAIGN_ID: %s\n' "${campaign_id}" >&2
    printf 'hint: use only letters, numbers, dot, underscore, and hyphen.\n' >&2
    exit 1
  fi

  if [[ "${campaign_id}" =~ [-_.]$ ]]; then
    printf 'error: suspicious CAMPAIGN_ID ends with punctuation: %s\n' "${campaign_id}" >&2
    printf 'hint: the command may have been split across lines after a hyphen; rerun as one line or use a trailing backslash.\n' >&2
    exit 1
  fi
}

validate_campaign_id "${CAMPAIGN_ID}"

if [ -d "${CAMPAIGN_DIR}" ] && [ -n "$(find "${CAMPAIGN_DIR}" -mindepth 1 -print -quit)" ]; then
  printf 'error: campaign directory already exists and is not empty: %s\n' "${CAMPAIGN_DIR}" >&2
  printf 'hint: use a new CAMPAIGN_ID; reusing an id can mix old and new run artifacts.\n' >&2
  exit 1
fi

export PROVCHAIN_TRACE_HTTP_PORT PROVCHAIN_TRACE_METRICS_PORT
export NEO4J_TRACE_HTTP_PORT NEO4J_TRACE_BOLT_PORT FLUREE_TRACE_HTTP_PORT
export PROVCHAIN_IMPORT_MODE
export NEO4J_HEAP_INITIAL NEO4J_HEAP_MAX NEO4J_PAGECACHE NEO4J_LOAD_BATCH_SIZE

mkdir -p "${EPOCHS_DIR}" "${LOGS_DIR}" "${TRACE_RESULTS_DIR}"

log_campaign() {
  printf '%s\n' "$*" | tee -a "${STATUS_LOG}"
}

write_json_array() {
  local comma_list="$1"
  local first="true"
  printf '['
  IFS=',' read -ra items <<< "${comma_list}"
  for item in "${items[@]}"; do
    item="$(printf '%s' "${item}" | xargs)"
    if [ -z "${item}" ]; then
      continue
    fi
    if [ "${first}" = "true" ]; then
      first="false"
    else
      printf ', '
    fi
    printf '"%s"' "${item}"
  done
  printf ']'
}

write_campaign_manifest() {
  local created_at="$1"
  local products_json
  local dataset_path="${TOOLKIT_DIR}/datasets/${DATASET_FILE}"
  local dataset_sha256=""
  local dataset_bytes="0"
  if [ -f "${dataset_path}" ]; then
    dataset_sha256="$(sha256sum "${dataset_path}" | awk '{print $1}')"
    dataset_bytes="$(wc -c < "${dataset_path}" | tr -d '[:space:]')"
  fi
  products_json="$(write_json_array "${PRODUCTS}")"
  cat > "${CAMPAIGN_DIR}/campaign_manifest.json" <<EOF_MANIFEST
{
  "campaign_id": "${CAMPAIGN_ID}",
  "created_at_utc": "${created_at}",
  "benchmark_family": "${BENCHMARK_FAMILY}",
  "dataset_slice": "${DATASET_SLICE}",
  "dataset_file": "${DATASET_FILE}",
  "fluree_dataset_file": "${FLUREE_DATASET_FILE}",
  "dataset_sha256": "${dataset_sha256}",
  "dataset_bytes": ${dataset_bytes},
  "test_batch_ids": "${TEST_BATCH_IDS}",
  "evidence_role": "${EVIDENCE_ROLE}",
  "products": ${products_json},
  "provchain_import_mode": "${PROVCHAIN_IMPORT_MODE}",
  "epoch_count_target": ${EPOCHS},
  "iterations_per_epoch": ${ITERATIONS},
  "clean_volumes_per_epoch": ${CLEAN_VOLUMES},
  "host_ports": {
    "provchain_http": ${PROVCHAIN_TRACE_HTTP_PORT},
    "provchain_metrics": ${PROVCHAIN_TRACE_METRICS_PORT},
    "neo4j_http": ${NEO4J_TRACE_HTTP_PORT},
    "neo4j_bolt": ${NEO4J_TRACE_BOLT_PORT},
    "fluree_http": ${FLUREE_TRACE_HTTP_PORT}
  },
  "neo4j_runtime": {
    "heap_initial": "${NEO4J_HEAP_INITIAL}",
    "heap_max": "${NEO4J_HEAP_MAX}",
    "pagecache": "${NEO4J_PAGECACHE}",
    "load_batch_size": ${NEO4J_LOAD_BATCH_SIZE}
  },
  "preflight_required": true,
  "preflight_skipped": ${SKIP_PREFLIGHT},
  "validity_gate": [
    "local preflight must pass unless explicitly skipped",
    "each compared system must have success_rate greater than zero",
    "run artifacts must include benchmark_results.json, benchmark_results.csv, summary.json, and summary.md"
  ],
  "notes": "Campaign runner args: ${RUNNER_MODE_ARGS}. Skip flags: Neo4j=${SKIP_NEO4J}, Fluree=${SKIP_FLUREE}, Fabric=${SKIP_FABRIC}, Geth=${SKIP_GETH}."
}
EOF_MANIFEST
}

write_epoch_manifest() {
  local epoch_dir="$1"
  local epoch_id="$2"
  local run_id="$3"
  local started_at="$4"
  local completed_at="$5"
  local status="$6"
  local reason="$7"
  cat > "${epoch_dir}/epoch_manifest.json" <<EOF_EPOCH
{
  "epoch_id": "${epoch_id}",
  "run_id": "${run_id}",
  "started_at_utc": "${started_at}",
  "completed_at_utc": "${completed_at}",
  "status": "${status}",
  "exclusion_reason": "${reason}",
  "benchmark_family": "${BENCHMARK_FAMILY}",
  "dataset_slice": "${DATASET_SLICE}",
  "iterations": ${ITERATIONS}
}
EOF_EPOCH
}

append_campaign_summary_header() {
  cat > "${CAMPAIGN_DIR}/campaign_summary.md" <<EOF_SUMMARY
# Benchmark Campaign Summary

| Field | Value |
|---|---|
| Campaign ID | \`${CAMPAIGN_ID}\` |
| Benchmark family | \`${BENCHMARK_FAMILY}\` |
| Dataset | \`${DATASET_FILE}\` |
| Dataset slice | \`${DATASET_SLICE}\` |
| Evidence role | \`${EVIDENCE_ROLE}\` |
| Test batch IDs | \`${TEST_BATCH_IDS}\` |
| Products | \`${PRODUCTS}\` |
| Epoch target | \`${EPOCHS}\` |
| Iterations per epoch | \`${ITERATIONS}\` |
| Clean volumes per epoch | \`${CLEAN_VOLUMES}\` |
| Host ports | \`provchain=${PROVCHAIN_TRACE_HTTP_PORT}, metrics=${PROVCHAIN_TRACE_METRICS_PORT}, neo4j_http=${NEO4J_TRACE_HTTP_PORT}, neo4j_bolt=${NEO4J_TRACE_BOLT_PORT}, fluree=${FLUREE_TRACE_HTTP_PORT}\` |
| Neo4j runtime | \`heap_initial=${NEO4J_HEAP_INITIAL}, heap_max=${NEO4J_HEAP_MAX}, pagecache=${NEO4J_PAGECACHE}, load_batch_size=${NEO4J_LOAD_BATCH_SIZE}\` |

## Epochs

| Epoch | Run ID | Status | Notes |
|---|---|---|---|
EOF_SUMMARY
}

append_epoch_summary() {
  local epoch_id="$1"
  local run_id="$2"
  local status="$3"
  local notes="$4"
  printf '| `%s` | `%s` | `%s` | %s |\n' "${epoch_id}" "${run_id}" "${status}" "${notes}" >> "${CAMPAIGN_DIR}/campaign_summary.md"
}

copy_run_artifacts() {
  local source_dir="$1"
  local target_dir="$2"
  mkdir -p "${target_dir}"
  for file_name in environment_manifest.json benchmark_results.json benchmark_results.csv summary.json summary.md; do
    if [ -f "${source_dir}/${file_name}" ]; then
      cp "${source_dir}/${file_name}" "${target_dir}/${file_name}"
    fi
  done
}

validate_run_results() {
  local results_file="$1"

  python3 - "${results_file}" <<'PY_VALIDATE'
import json
import sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as handle:
    rows = json.load(handle)

failures = [row for row in rows if row.get("success") is False]
if not failures:
    raise SystemExit(0)

print(f"benchmark_results.json contains {len(failures)} failed result row(s):")
for row in failures[:10]:
    system = row.get("system", "unknown")
    family = row.get("family", "unknown")
    test = row.get("test_name", "unknown")
    error = row.get("error_message") or ""
    print(f"- {family} / {test} / {system}: {error}")
if len(failures) > 10:
    print(f"- ... {len(failures) - 10} more")
raise SystemExit(1)
PY_VALIDATE
}

check_docker_access() {
  if ! docker info >/dev/null 2>&1; then
    cat >&2 <<'EOF_DOCKER'
[campaign] Docker daemon is not accessible from this shell.
[campaign] Fix Docker permissions first, or run this script from a shell that can execute `docker info`.
EOF_DOCKER
    return 1
  fi
}

run_compose_epoch() {
  local epoch_id="$1"
  local run_id="$2"
  local epoch_dir="$3"
  local log_file="${LOGS_DIR}/${epoch_id}-${run_id}.log"
  local compose_status=0
  local compose_cmd=(docker compose -f "${COMPOSE_FILE}")

  if [ "${SKIP_FLUREE}" != "true" ]; then
    compose_cmd+=(--profile fluree)
  fi

  if [ "${CLEAN_VOLUMES}" = "true" ]; then
    "${compose_cmd[@]}" down -v --remove-orphans >> "${log_file}" 2>&1 || true
  else
    "${compose_cmd[@]}" down --remove-orphans >> "${log_file}" 2>&1 || true
  fi

  if [ "${SKIP_FLUREE}" != "true" ]; then
    "${compose_cmd[@]}" up -d fluree >> "${log_file}" 2>&1 || compose_status=$?
    if [ "${compose_status}" -eq 0 ]; then
      local fluree_health=""
      for _ in $(seq 1 40); do
        fluree_health="$(docker inspect -f '{{if .State.Health}}{{.State.Health.Status}}{{else}}unknown{{end}}' fluree-trace 2>/dev/null || true)"
        if [ "${fluree_health}" = "healthy" ]; then
          break
        fi
        sleep 3
      done
      if [ "${fluree_health}" != "healthy" ]; then
        printf 'Fluree did not become healthy, last health state: %s\n' "${fluree_health}" >> "${log_file}"
        compose_status=1
      fi
    fi
  fi

  if [ "${compose_status}" -eq 0 ]; then
    BENCHMARK_RUN_ID="${run_id}" \
    ITERATIONS="${ITERATIONS}" \
    PROVCHAIN_DATASET_FILE="${DATASET_FILE}" \
    NEO4J_DATASET_FILE="${DATASET_FILE}" \
    FLUREE_DATASET_FILE="${FLUREE_DATASET_FILE}" \
    RUNNER_MODE_ARGS="${RUNNER_MODE_ARGS}" \
    TEST_BATCH_IDS="${TEST_BATCH_IDS}" \
    BENCHMARK_SKIP_NEO4J="${SKIP_NEO4J}" \
    BENCHMARK_SKIP_FLUREE="${SKIP_FLUREE}" \
    BENCHMARK_SKIP_FABRIC="${SKIP_FABRIC}" \
    BENCHMARK_SKIP_GETH="${SKIP_GETH}" \
    PROVCHAIN_IMPORT_MODE="${PROVCHAIN_IMPORT_MODE}" \
      "${compose_cmd[@]}" up --build --abort-on-container-exit --exit-code-from benchmark-runner >> "${log_file}" 2>&1 || compose_status=$?
  fi

  "${compose_cmd[@]}" logs --no-color benchmark-runner > "${epoch_dir}/benchmark-runner.log" 2>&1 || true

  if [ "${CLEAN_VOLUMES}" = "true" ]; then
    "${compose_cmd[@]}" down -v --remove-orphans >> "${log_file}" 2>&1 || true
  else
    "${compose_cmd[@]}" down --remove-orphans >> "${log_file}" 2>&1 || true
  fi

  return "${compose_status}"
}

main() {
  local created_at
  created_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  : > "${STATUS_LOG}"
  write_campaign_manifest "${created_at}"
  append_campaign_summary_header

  if [ "${SKIP_PREFLIGHT}" != "true" ]; then
    "${SCRIPT_DIR}/preflight-trace-benchmark.sh" | tee "${LOGS_DIR}/preflight.log"
  fi

  check_docker_access 2>&1 | tee "${LOGS_DIR}/docker-access.log"

  local passed_count=0
  local failed_count=0

  for epoch_num in $(seq 1 "${EPOCHS}"); do
    local epoch_id
    local run_id
    local started_at
    local completed_at
    local epoch_dir
    local run_dir
    local trace_run_dir
    local status
    local reason

    epoch_id="$(printf 'epoch-%03d' "${epoch_num}")"
    run_id="$(date -u +%Y%m%dT%H%M%SZ)"
    started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    epoch_dir="${EPOCHS_DIR}/${epoch_id}"
    run_dir="${epoch_dir}/runs/${run_id}"
    trace_run_dir="${TRACE_RESULTS_DIR}/${run_id}"
    mkdir -p "${epoch_dir}" "${run_dir}" "${epoch_dir}/runs"

    log_campaign "[campaign] ${CAMPAIGN_ID} ${epoch_id}/${EPOCHS} run_id=${run_id}"

    status="passed"
    reason=""
    if ! run_compose_epoch "${epoch_id}" "${run_id}" "${epoch_dir}"; then
      status="failed"
      reason="docker compose benchmark execution failed"
    fi

    copy_run_artifacts "${trace_run_dir}" "${run_dir}"

    if [ "${status}" = "passed" ] && [ ! -f "${run_dir}/benchmark_results.json" ]; then
      status="failed"
      reason="benchmark_results.json was not produced"
    fi

    if [ "${status}" = "passed" ] && ! validate_run_results "${run_dir}/benchmark_results.json" > "${epoch_dir}/result-validation.log" 2>&1; then
      status="failed"
      reason="benchmark_results.json contained failed result rows"
    fi

    completed_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    write_epoch_manifest "${epoch_dir}" "${epoch_id}" "${run_id}" "${started_at}" "${completed_at}" "${status}" "${reason}"

    if [ "${status}" = "passed" ]; then
      passed_count=$((passed_count + 1))
      append_epoch_summary "${epoch_id}" "${run_id}" "${status}" "artifacts copied"
    else
      failed_count=$((failed_count + 1))
      append_epoch_summary "${epoch_id}" "${run_id}" "${status}" "${reason}"
    fi

    sleep 2
  done

  cat > "${CAMPAIGN_DIR}/campaign_status.json" <<EOF_STATUS
{
  "campaign_id": "${CAMPAIGN_ID}",
  "completed_at_utc": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "epoch_count_target": ${EPOCHS},
  "passed_epochs": ${passed_count},
  "failed_epochs": ${failed_count},
  "status": "$([ "${failed_count}" -eq 0 ] && printf 'passed' || printf 'partial')"
}
EOF_STATUS

  if ! "${SCRIPT_DIR}/summarize-campaign.py" "${CAMPAIGN_DIR}" >> "${STATUS_LOG}" 2>&1; then
    log_campaign "[campaign] warning: aggregate summary generation failed"
  fi

  log_campaign "[campaign] complete: ${CAMPAIGN_DIR}"
}

main "$@"
