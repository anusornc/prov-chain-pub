#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TOOLKIT_DIR="${REPO_ROOT}/benchmark-toolkit"
RESULTS_ROOT="${TOOLKIT_DIR}/results"
WORKLOAD="${WORKLOAD:-write}"
FABRIC_RESULTS_DIR="${FABRIC_RESULTS_DIR:-${RESULTS_ROOT}/fabric-ledger}"
CAMPAIGNS_DIR="${RESULTS_ROOT}/campaigns"

EPOCHS="${EPOCHS:-3}"
ITERATIONS="${ITERATIONS:-10}"
FABRIC_BATCH_SIZE="${FABRIC_BATCH_SIZE:-100}"
FABRIC_GATEWAY_URL="${FABRIC_GATEWAY_URL:-http://localhost:18800}"
PROVCHAIN_URL="${PROVCHAIN_URL:-http://localhost:8080}"
DATASET_PATH="${DATASET_PATH:-${TOOLKIT_DIR}/datasets}"
DATASET_FILE="${DATASET_FILE:-supply_chain_1000.ttl}"
SKIP_LOAD="${SKIP_LOAD:-true}"
SKIP_PREFLIGHT="${SKIP_PREFLIGHT:-false}"
SKIP_PROVCHAIN="${SKIP_PROVCHAIN:-false}"
SKIP_FABRIC="${SKIP_FABRIC:-false}"
MANAGE_PROVCHAIN="${MANAGE_PROVCHAIN:-false}"
PROVCHAIN_MANAGED_PORT="${PROVCHAIN_MANAGED_PORT:-18080}"
JWT_SECRET="${JWT_SECRET:-benchmark-jwt-secret-minimum-32-characters}"
PROVCHAIN_BOOTSTRAP_TOKEN="${PROVCHAIN_BOOTSTRAP_TOKEN:-benchmark-bootstrap-token-20260424}"
PROVCHAIN_BENCHMARK_STAGE_TIMINGS="${PROVCHAIN_BENCHMARK_STAGE_TIMINGS:-true}"
PROVCHAIN_RDF_FLUSH_INTERVAL="${PROVCHAIN_RDF_FLUSH_INTERVAL:-100}"
PROVCHAIN_WAL_SYNC_INTERVAL="${PROVCHAIN_WAL_SYNC_INTERVAL:-100}"
PROVCHAIN_CHAIN_INDEX_SYNC_INTERVAL="${PROVCHAIN_CHAIN_INDEX_SYNC_INTERVAL:-${PROVCHAIN_WAL_SYNC_INTERVAL}}"
KEEP_PROVCHAIN_DATA="${KEEP_PROVCHAIN_DATA:-false}"
export JWT_SECRET PROVCHAIN_BOOTSTRAP_TOKEN PROVCHAIN_BENCHMARK_STAGE_TIMINGS PROVCHAIN_RDF_FLUSH_INTERVAL PROVCHAIN_WAL_SYNC_INTERVAL PROVCHAIN_CHAIN_INDEX_SYNC_INTERVAL
DATASET_SLICE="${DATASET_SLICE:-supply_chain_1000}"
if [ "${WORKLOAD}" = "policy" ]; then
  BENCHMARK_FAMILY="${BENCHMARK_FAMILY:-governance_policy}"
  DEFAULT_CAMPAIGN_KIND="policy"
  DEFAULT_NOTES="B014 policy campaign runner for real Fabric policy execution. This script does not start the local simulator."
elif [ "${WORKLOAD}" = "write" ]; then
  BENCHMARK_FAMILY="${BENCHMARK_FAMILY:-ledger_write}"
  DEFAULT_CAMPAIGN_KIND="ledger"
  if [ "${SKIP_FABRIC}" = "true" ]; then
    DEFAULT_NOTES="ProvChain-only ledger profiling campaign. Fabric gateway fields are retained as runner defaults and ignored because skip_fabric=true."
  else
    DEFAULT_NOTES="B013 ledger campaign runner for real ProvChain vs Fabric execution. This script does not start the local simulator."
  fi
else
  printf 'error: unsupported WORKLOAD=%s; expected write or policy\n' "${WORKLOAD}" >&2
  exit 1
fi
if [ -z "${PRODUCTS:-}" ]; then
  if [ "${SKIP_FABRIC}" = "true" ]; then
    PRODUCTS="provchain"
  else
    PRODUCTS="provchain,fabric"
  fi
fi
CAMPAIGN_ID="${CAMPAIGN_ID:-$(date -u +%Y%m%d)_${DEFAULT_CAMPAIGN_KIND}_${DATASET_SLICE}_provchain-fabric_n${EPOCHS}}"
CAMPAIGN_DIR="${CAMPAIGNS_DIR}/${CAMPAIGN_ID}"
PROVCHAIN_RUNTIME_DIR="${PROVCHAIN_RUNTIME_DIR:-${RESULTS_ROOT}/provchain-runtime/${CAMPAIGN_ID}}"
EPOCHS_DIR="${CAMPAIGN_DIR}/epochs"
LOGS_DIR="${CAMPAIGN_DIR}/logs"
STATUS_LOG="${CAMPAIGN_DIR}/campaign_status.log"

mkdir -p "${EPOCHS_DIR}" "${LOGS_DIR}" "${FABRIC_RESULTS_DIR}"

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

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
  local durability_mode="conservative_sync_every_block"
  local cold_load_phase_enabled="false"
  local provchain_append_phase="append_without_load_phase"
  if [ "${PROVCHAIN_WAL_SYNC_INTERVAL}" != "1" ] || [ "${PROVCHAIN_CHAIN_INDEX_SYNC_INTERVAL}" != "1" ]; then
    durability_mode="relaxed_batched_fsync"
  fi
  if [ "${SKIP_LOAD}" != "true" ]; then
    cold_load_phase_enabled="true"
    provchain_append_phase="steady_state_after_cold_load"
  fi
  products_json="$(write_json_array "${PRODUCTS}")"
  cat > "${CAMPAIGN_DIR}/campaign_manifest.json" <<EOF_MANIFEST
{
  "campaign_id": "${CAMPAIGN_ID}",
  "created_at_utc": "${created_at}",
  "benchmark_family": "${BENCHMARK_FAMILY}",
  "dataset_slice": "${DATASET_SLICE}",
  "dataset_path": "${DATASET_PATH}",
  "dataset_file": "${DATASET_FILE}",
  "products": ${products_json},
  "epoch_count_target": ${EPOCHS},
  "iterations_per_epoch": ${ITERATIONS},
  "fabric_batch_size": ${FABRIC_BATCH_SIZE},
  "fabric_gateway_url": "${FABRIC_GATEWAY_URL}",
  "provchain_url": "${PROVCHAIN_URL}",
  "skip_provchain": ${SKIP_PROVCHAIN},
  "skip_fabric": ${SKIP_FABRIC},
  "skip_load": ${SKIP_LOAD},
  "manage_provchain": ${MANAGE_PROVCHAIN},
  "provchain_managed_port": ${PROVCHAIN_MANAGED_PORT},
  "provchain_runtime_dir": "${PROVCHAIN_RUNTIME_DIR}",
  "provchain_benchmark_stage_timings": "${PROVCHAIN_BENCHMARK_STAGE_TIMINGS}",
  "provchain_rdf_flush_interval": "${PROVCHAIN_RDF_FLUSH_INTERVAL}",
  "provchain_wal_sync_interval": "${PROVCHAIN_WAL_SYNC_INTERVAL}",
  "provchain_chain_index_sync_interval": "${PROVCHAIN_CHAIN_INDEX_SYNC_INTERVAL}",
  "keep_managed_provchain_data": ${KEEP_PROVCHAIN_DATA},
  "provchain_persistence_durability_mode": "${durability_mode}",
  "cold_load_phase_enabled": ${cold_load_phase_enabled},
  "provchain_append_phase": "${provchain_append_phase}",
  "preflight_required": true,
  "preflight_skipped": ${SKIP_PREFLIGHT},
  "validity_gate": [
    "Fabric gateway must be a real runtime gateway, not the local contract simulator, unless skip_fabric is true",
    "Fabric gateway probe must pass unless skip_fabric is true",
    "local dataset file must exist",
    "ProvChain health check must pass unless skip_provchain is true",
    "managed ProvChain mode must use a fresh data directory per epoch",
    "managed ProvChain data is pruned after each epoch unless keep_managed_provchain_data is true",
    "if cold_load_phase_enabled is true, the Turtle RDF Import row measures cold-load cost and subsequent ProvChain write rows measure steady-state append after load",
    "run artifacts must include benchmark_results.json, benchmark_results.csv, summary.json, and summary.md",
    "comparative claims require successful rows for every claimed product"
  ],
  "workload": "${WORKLOAD}",
  "notes": "${DEFAULT_NOTES}"
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
  "iterations": ${ITERATIONS},
  "fabric_batch_size": ${FABRIC_BATCH_SIZE}
}
EOF_EPOCH
}

append_campaign_summary_header() {
  local summary_title="Fabric Campaign Summary"
  local evidence_role="runtime_campaign"
  local fabric_gateway_role="active"
  local durability_mode="conservative_sync_every_block"
  local cold_load_phase_enabled="false"
  local provchain_append_phase="append_without_load_phase"
  if [ "${PROVCHAIN_WAL_SYNC_INTERVAL}" != "1" ] || [ "${PROVCHAIN_CHAIN_INDEX_SYNC_INTERVAL}" != "1" ]; then
    durability_mode="relaxed_batched_fsync"
  fi
  if [ "${SKIP_LOAD}" != "true" ]; then
    cold_load_phase_enabled="true"
    provchain_append_phase="steady_state_after_cold_load"
  fi
  if [ "${SKIP_FABRIC}" = "true" ]; then
    summary_title="ProvChain-Only Ledger Profiling Campaign Summary"
    evidence_role="profiling_reference_not_primary_paper_comparison"
    fabric_gateway_role="ignored because skip_fabric=true"
  fi
  cat > "${CAMPAIGN_DIR}/campaign_summary.md" <<EOF_SUMMARY
# ${summary_title}

| Field | Value |
|---|---|
| Campaign ID | \`${CAMPAIGN_ID}\` |
| Evidence role | \`${evidence_role}\` |
| Benchmark family | \`${BENCHMARK_FAMILY}\` |
| Workload | \`${WORKLOAD}\` |
| Products | \`${PRODUCTS}\` |
| Epoch target | \`${EPOCHS}\` |
| Iterations per epoch | \`${ITERATIONS}\` |
| Batch size | \`${FABRIC_BATCH_SIZE}\` |
| Dataset path | \`${DATASET_PATH}\` |
| Dataset file | \`${DATASET_FILE}\` |
| Skip load rows | \`${SKIP_LOAD}\` |
| Skip Fabric | \`${SKIP_FABRIC}\` |
| Manage ProvChain | \`${MANAGE_PROVCHAIN}\` |
| Managed ProvChain port | \`${PROVCHAIN_MANAGED_PORT}\` |
| Managed ProvChain runtime dir | \`${PROVCHAIN_RUNTIME_DIR}\` |
| Managed ProvChain WAL sync interval | \`${PROVCHAIN_WAL_SYNC_INTERVAL}\` |
| Managed ProvChain chain-index sync interval | \`${PROVCHAIN_CHAIN_INDEX_SYNC_INTERVAL}\` |
| Keep managed ProvChain data | \`${KEEP_PROVCHAIN_DATA}\` |
| Managed ProvChain durability mode | \`${durability_mode}\` |
| Cold-load phase enabled | \`${cold_load_phase_enabled}\` |
| ProvChain append phase | \`${provchain_append_phase}\` |
| Fabric gateway URL | \`${FABRIC_GATEWAY_URL}\` |
| Fabric gateway role | \`${fabric_gateway_role}\` |
| ProvChain URL | \`${PROVCHAIN_URL}\` |
| Skip ProvChain | \`${SKIP_PROVCHAIN}\` |

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

run_preflight() {
  if [ ! -f "${DATASET_PATH}/${DATASET_FILE}" ]; then
    die "dataset file not found: ${DATASET_PATH}/${DATASET_FILE}"
  fi

  if [ "${SKIP_FABRIC}" != "true" ]; then
    cargo test --manifest-path "${TOOLKIT_DIR}/research-benchmarks/Cargo.toml" fabric -- --nocapture
    FABRIC_GATEWAY_URL="${FABRIC_GATEWAY_URL}" "${SCRIPT_DIR}/probe-fabric-gateway.sh"
  fi

  if [ "${SKIP_PROVCHAIN}" != "true" ] && [ "${MANAGE_PROVCHAIN}" != "true" ]; then
    curl -fsS --connect-timeout 2 --max-time 10 "${PROVCHAIN_URL%/}/health" >/dev/null \
      || die "ProvChain health check failed: ${PROVCHAIN_URL%/}/health. Start ProvChain, rerun with --manage-provchain, or rerun with --skip-provchain for Fabric-only runtime validation."
  fi
}

wait_for_url() {
  local url="$1"
  local timeout_seconds="$2"
  local started
  started="$(date +%s)"

  while true; do
    if curl -fsS --connect-timeout 2 --max-time 5 "${url}" >/dev/null 2>&1; then
      return 0
    fi

    if [ $(( $(date +%s) - started )) -ge "${timeout_seconds}" ]; then
      return 1
    fi

    sleep 2
  done
}

start_managed_provchain() {
  local epoch_dir="$1"
  local log_file="$2"
  local port="$3"
  local data_dir="${epoch_dir}/provchain-data"

  mkdir -p "${data_dir}"

  PROVCHAIN_DATA_DIR="${data_dir}" \
  PROVCHAIN_BENCHMARK_STAGE_TIMINGS="${PROVCHAIN_BENCHMARK_STAGE_TIMINGS}" \
  PROVCHAIN_RDF_FLUSH_INTERVAL="${PROVCHAIN_RDF_FLUSH_INTERVAL}" \
  PROVCHAIN_WAL_SYNC_INTERVAL="${PROVCHAIN_WAL_SYNC_INTERVAL}" \
  PROVCHAIN_CHAIN_INDEX_SYNC_INTERVAL="${PROVCHAIN_CHAIN_INDEX_SYNC_INTERVAL}" \
    cargo run -- examples web-server --port "${port}" >> "${log_file}" 2>&1 &

  local pid="$!"
  printf '%s\n' "${pid}" > "${epoch_dir}/provchain-server.pid"

  if ! wait_for_url "http://localhost:${port}/health" 120; then
    kill "${pid}" >/dev/null 2>&1 || true
    wait "${pid}" >/dev/null 2>&1 || true
    return 1
  fi

  return 0
}

stop_managed_provchain() {
  local epoch_dir="$1"
  local pid_file="${epoch_dir}/provchain-server.pid"

  if [ ! -f "${pid_file}" ]; then
    return
  fi

  local pid
  pid="$(cat "${pid_file}")"
  kill "${pid}" >/dev/null 2>&1 || true
  wait "${pid}" >/dev/null 2>&1 || true
}

prune_managed_provchain_data() {
  local epoch_dir="$1"
  if [ "${KEEP_PROVCHAIN_DATA}" = "true" ]; then
    return
  fi
  rm -rf "${epoch_dir}/provchain-data"
  rm -f "${epoch_dir}/provchain-server.pid"
}

run_epoch() {
  local run_id="$1"
  local results_path="${FABRIC_RESULTS_DIR}/${run_id}"
  local log_file="$2"
  local skip_provchain_args=()

  if [ "${SKIP_PROVCHAIN}" = "true" ]; then
    skip_provchain_args+=(--skip-provchain)
  fi

  local skip_fabric_args=()
  if [ "${SKIP_FABRIC}" = "true" ]; then
    skip_fabric_args+=(--skip-fabric)
  fi

  local skip_load_args=()
  if [ "${SKIP_LOAD}" = "true" ]; then
    skip_load_args+=(--skip-load)
  fi

  cargo run \
    --manifest-path "${TOOLKIT_DIR}/research-benchmarks/Cargo.toml" \
    -- \
    --"${WORKLOAD}" \
    --skip-neo4j \
    --skip-fluree \
    --skip-geth \
    "${skip_provchain_args[@]}" \
    "${skip_fabric_args[@]}" \
    "${skip_load_args[@]}" \
    --provchain-url "${PROVCHAIN_URL}" \
    --fabric-gateway-url "${FABRIC_GATEWAY_URL}" \
    --dataset-path "${DATASET_PATH}" \
    --dataset-file "${DATASET_FILE}" \
    --provchain-dataset-file "${DATASET_FILE}" \
    --iterations "${ITERATIONS}" \
    --fabric-batch-size "${FABRIC_BATCH_SIZE}" \
    --results-path "${results_path}" >> "${log_file}" 2>&1
}

main() {
  local created_at
  created_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  : > "${STATUS_LOG}"
  write_campaign_manifest "${created_at}"
  append_campaign_summary_header

  if [ "${SKIP_PREFLIGHT}" != "true" ]; then
    run_preflight | tee "${LOGS_DIR}/preflight.log"
  fi

  local passed_count=0
  local failed_count=0

  for epoch_num in $(seq 1 "${EPOCHS}"); do
    local epoch_id
    local run_id
    local started_at
    local completed_at
    local epoch_dir
    local run_dir
    local run_results_dir
    local log_file
    local status
    local reason

    epoch_id="$(printf 'epoch-%03d' "${epoch_num}")"
    run_id="$(date -u +%Y%m%dT%H%M%SZ)"
    started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    epoch_dir="${EPOCHS_DIR}/${epoch_id}"
    run_dir="${epoch_dir}/runs/${run_id}"
    run_results_dir="${FABRIC_RESULTS_DIR}/${run_id}"
    log_file="${LOGS_DIR}/${epoch_id}-${run_id}.log"
    mkdir -p "${epoch_dir}" "${run_dir}"

    log_campaign "[campaign] ${CAMPAIGN_ID} ${epoch_id}/${EPOCHS} run_id=${run_id}"

    local epoch_provchain_started="false"
    local original_provchain_url="${PROVCHAIN_URL}"
    if [ "${MANAGE_PROVCHAIN}" = "true" ] && [ "${SKIP_PROVCHAIN}" != "true" ]; then
      local epoch_port
      epoch_port=$((PROVCHAIN_MANAGED_PORT + epoch_num))
      PROVCHAIN_URL="http://localhost:${epoch_port}"
      if start_managed_provchain "${epoch_dir}" "${log_file}" "${epoch_port}"; then
        epoch_provchain_started="true"
      else
        status="failed"
        reason="managed ProvChain failed to start"
        failed_count=$((failed_count + 1))
        completed_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
        write_epoch_manifest "${epoch_dir}" "${epoch_id}" "${run_id}" "${started_at}" "${completed_at}" "${status}" "${reason}"
        append_epoch_summary "${epoch_id}" "${run_id}" "${status}" "${reason}"
        prune_managed_provchain_data "${epoch_dir}"
        PROVCHAIN_URL="${original_provchain_url}"
        continue
      fi
    fi

    if run_epoch "${run_id}" "${log_file}"; then
      status="passed"
      reason=""
      passed_count=$((passed_count + 1))
      copy_run_artifacts "${run_results_dir}" "${run_dir}"
    else
      status="failed"
      reason="benchmark runner exited non-zero"
      failed_count=$((failed_count + 1))
    fi

    if [ "${epoch_provchain_started}" = "true" ]; then
      stop_managed_provchain "${epoch_dir}"
      prune_managed_provchain_data "${epoch_dir}"
      PROVCHAIN_URL="${original_provchain_url}"
    fi

    completed_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    write_epoch_manifest "${epoch_dir}" "${epoch_id}" "${run_id}" "${started_at}" "${completed_at}" "${status}" "${reason}"
    append_epoch_summary "${epoch_id}" "${run_id}" "${status}" "${reason:-ok}"
  done

  local completed_at
  local final_status
  completed_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  if [ "${failed_count}" -eq 0 ]; then
    final_status="passed"
  else
    final_status="failed"
  fi

  cat > "${CAMPAIGN_DIR}/campaign_status.json" <<EOF_STATUS
{
  "campaign_id": "${CAMPAIGN_ID}",
  "completed_at_utc": "${completed_at}",
  "epoch_count_target": ${EPOCHS},
  "passed_epochs": ${passed_count},
  "failed_epochs": ${failed_count},
  "status": "${final_status}"
}
EOF_STATUS

  python3 "${SCRIPT_DIR}/summarize-campaign.py" "${CAMPAIGN_DIR}" | tee -a "${STATUS_LOG}"
  log_campaign "[campaign] complete: ${CAMPAIGN_DIR}"

  if [ "${failed_count}" -ne 0 ]; then
    exit 1
  fi
}

main "$@"
