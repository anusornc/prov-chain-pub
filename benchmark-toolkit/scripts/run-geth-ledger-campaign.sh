#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TOOLKIT_DIR="${REPO_ROOT}/benchmark-toolkit"
RESULTS_ROOT="${TOOLKIT_DIR}/results"
GETH_RESULTS_DIR="${GETH_RESULTS_DIR:-${RESULTS_ROOT}/geth-ledger}"
CAMPAIGNS_DIR="${RESULTS_ROOT}/campaigns"

EPOCHS="${EPOCHS:-3}"
ITERATIONS="${ITERATIONS:-10}"
GETH_RPC_URL="${GETH_RPC_URL:-http://localhost:18545}"
GETH_CONTRACT_ADDRESS="${GETH_CONTRACT_ADDRESS:-}"
GETH_SENDER_ADDRESS="${GETH_SENDER_ADDRESS:-}"
GETH_TX_GAS="${GETH_TX_GAS:-0x100000}"
GETH_MINING_MODE="${GETH_MINING_MODE:-dev-auto}"
GETH_CONFIRMATION_TIMEOUT_SECONDS="${GETH_CONFIRMATION_TIMEOUT_SECONDS:-60}"
GETH_CONFIRMATION_POLL_MS="${GETH_CONFIRMATION_POLL_MS:-250}"
PROVCHAIN_URL="${PROVCHAIN_URL:-http://localhost:8080}"
DATASET_PATH="${DATASET_PATH:-${TOOLKIT_DIR}/datasets}"
DATASET_FILE="${DATASET_FILE:-supply_chain_1000.ttl}"
SKIP_LOAD="${SKIP_LOAD:-true}"
MANAGE_PROVCHAIN="${MANAGE_PROVCHAIN:-true}"
PROVCHAIN_MANAGED_PORT="${PROVCHAIN_MANAGED_PORT:-18180}"
JWT_SECRET="${JWT_SECRET:-benchmark-jwt-secret-minimum-32-characters}"
PROVCHAIN_BOOTSTRAP_TOKEN="${PROVCHAIN_BOOTSTRAP_TOKEN:-benchmark-bootstrap-token-20260424}"
PROVCHAIN_BENCHMARK_STAGE_TIMINGS="${PROVCHAIN_BENCHMARK_STAGE_TIMINGS:-true}"
KEEP_PROVCHAIN_DATA="${KEEP_PROVCHAIN_DATA:-false}"
export JWT_SECRET PROVCHAIN_BOOTSTRAP_TOKEN PROVCHAIN_BENCHMARK_STAGE_TIMINGS

DATASET_SLICE="${DATASET_SLICE:-supply_chain_1000}"
BENCHMARK_FAMILY="${BENCHMARK_FAMILY:-ledger_write}"
PRODUCTS="${PRODUCTS:-provchain,geth}"
SKIP_PREFLIGHT="${SKIP_PREFLIGHT:-false}"
SKIP_PROVCHAIN="${SKIP_PROVCHAIN:-false}"
CAMPAIGN_ID="${CAMPAIGN_ID:-$(date -u +%Y%m%d)_ledger_${DATASET_SLICE}_provchain-geth_n${EPOCHS}}"
CAMPAIGN_DIR="${CAMPAIGNS_DIR}/${CAMPAIGN_ID}"
PROVCHAIN_RUNTIME_DIR="${PROVCHAIN_RUNTIME_DIR:-${RESULTS_ROOT}/provchain-runtime/${CAMPAIGN_ID}}"
EPOCHS_DIR="${CAMPAIGN_DIR}/epochs"
LOGS_DIR="${CAMPAIGN_DIR}/logs"
STATUS_LOG="${CAMPAIGN_DIR}/campaign_status.log"

mkdir -p "${EPOCHS_DIR}" "${LOGS_DIR}" "${GETH_RESULTS_DIR}"

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
  "geth_rpc_url": "${GETH_RPC_URL}",
  "geth_contract_address": "${GETH_CONTRACT_ADDRESS}",
  "geth_sender_address": "${GETH_SENDER_ADDRESS}",
  "geth_tx_gas": "${GETH_TX_GAS}",
  "geth_mining_mode": "${GETH_MINING_MODE}",
  "provchain_url": "${PROVCHAIN_URL}",
  "skip_provchain": ${SKIP_PROVCHAIN},
  "skip_load": ${SKIP_LOAD},
  "manage_provchain": ${MANAGE_PROVCHAIN},
  "provchain_managed_port": ${PROVCHAIN_MANAGED_PORT},
  "provchain_runtime_dir": "${PROVCHAIN_RUNTIME_DIR}",
  "provchain_benchmark_stage_timings": "${PROVCHAIN_BENCHMARK_STAGE_TIMINGS}",
  "keep_managed_provchain_data": ${KEEP_PROVCHAIN_DATA},
  "preflight_required": true,
  "preflight_skipped": ${SKIP_PREFLIGHT},
  "fairness_label": "public-chain-baseline",
  "capability_path": "public-chain-smart-contract",
  "validity_gate": [
    "Geth RPC must be a real local development chain, not a mock server",
    "Geth RPC probe must pass and return at least one unlocked sender account",
    "local Geth adapter contract tests must pass before campaign execution",
    "runner must separate submit latency and receipt confirmation latency",
    "gas metadata must be recorded for confirmed Geth transactions",
    "managed ProvChain mode must use a fresh data directory per epoch",
    "managed ProvChain data is pruned after each epoch unless keep_managed_provchain_data is true",
    "comparative claims must identify Geth as a public-chain baseline"
  ],
  "workload": "write",
  "notes": "B017 ledger campaign runner for ProvChain vs Geth. Geth is a public-chain smart-contract baseline, not a permissioned-ledger parity target."
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
# Geth Campaign Summary

| Field | Value |
|---|---|
| Campaign ID | \`${CAMPAIGN_ID}\` |
| Benchmark family | \`${BENCHMARK_FAMILY}\` |
| Workload | \`write\` |
| Products | \`${PRODUCTS}\` |
| Epoch target | \`${EPOCHS}\` |
| Iterations per epoch | \`${ITERATIONS}\` |
| Dataset path | \`${DATASET_PATH}\` |
| Dataset file | \`${DATASET_FILE}\` |
| Skip load rows | \`${SKIP_LOAD}\` |
| Manage ProvChain | \`${MANAGE_PROVCHAIN}\` |
| Managed ProvChain port | \`${PROVCHAIN_MANAGED_PORT}\` |
| Managed ProvChain runtime dir | \`${PROVCHAIN_RUNTIME_DIR}\` |
| Keep managed ProvChain data | \`${KEEP_PROVCHAIN_DATA}\` |
| Geth RPC URL | \`${GETH_RPC_URL}\` |
| Geth contract address | \`${GETH_CONTRACT_ADDRESS:-runner-deployed-per-run}\` |
| Geth mining mode | \`${GETH_MINING_MODE}\` |
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

  cargo test --manifest-path "${TOOLKIT_DIR}/research-benchmarks/Cargo.toml" geth -- --nocapture
  GETH_RPC_URL="${GETH_RPC_URL}" "${SCRIPT_DIR}/probe-geth-rpc.sh"

  if [ "${SKIP_PROVCHAIN}" != "true" ] && [ "${MANAGE_PROVCHAIN}" != "true" ]; then
    curl -fsS --connect-timeout 2 --max-time 10 "${PROVCHAIN_URL%/}/health" >/dev/null \
      || die "ProvChain health check failed: ${PROVCHAIN_URL%/}/health. Start ProvChain or rerun with SKIP_PROVCHAIN=true for Geth-only runtime validation."
  fi
}

probe_geth_or_fail() {
  GETH_RPC_URL="${GETH_RPC_URL}" "${SCRIPT_DIR}/probe-geth-rpc.sh" >/dev/null 2>&1
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
  local results_path="${GETH_RESULTS_DIR}/${run_id}"
  local log_file="$2"
  local skip_provchain_args=()
  local geth_contract_args=()
  local geth_sender_args=()

  if [ "${SKIP_PROVCHAIN}" = "true" ]; then
    skip_provchain_args+=(--skip-provchain)
  fi

  local skip_load_args=()
  if [ "${SKIP_LOAD}" = "true" ]; then
    skip_load_args+=(--skip-load)
  fi

  if [ -n "${GETH_CONTRACT_ADDRESS}" ]; then
    geth_contract_args+=(--geth-contract-address "${GETH_CONTRACT_ADDRESS}")
  fi

  if [ -n "${GETH_SENDER_ADDRESS}" ]; then
    geth_sender_args+=(--geth-sender-address "${GETH_SENDER_ADDRESS}")
  fi

  cargo run \
    --manifest-path "${TOOLKIT_DIR}/research-benchmarks/Cargo.toml" \
    -- \
    --write \
    --skip-neo4j \
    --skip-fluree \
    --skip-fabric \
    "${skip_provchain_args[@]}" \
    "${skip_load_args[@]}" \
    "${geth_contract_args[@]}" \
    "${geth_sender_args[@]}" \
    --provchain-url "${PROVCHAIN_URL}" \
    --geth-rpc-url "${GETH_RPC_URL}" \
    --geth-tx-gas "${GETH_TX_GAS}" \
    --geth-mining-mode "${GETH_MINING_MODE}" \
    --geth-confirmation-timeout-seconds "${GETH_CONFIRMATION_TIMEOUT_SECONDS}" \
    --geth-confirmation-poll-ms "${GETH_CONFIRMATION_POLL_MS}" \
    --dataset-path "${DATASET_PATH}" \
    --dataset-file "${DATASET_FILE}" \
    --provchain-dataset-file "${DATASET_FILE}" \
    --iterations "${ITERATIONS}" \
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
    run_results_dir="${GETH_RESULTS_DIR}/${run_id}"
    log_file="${LOGS_DIR}/${epoch_id}-${run_id}.log"
    mkdir -p "${epoch_dir}" "${run_dir}"

    log_campaign "[campaign] ${CAMPAIGN_ID} ${epoch_id}/${EPOCHS} run_id=${run_id}"

    if ! probe_geth_or_fail; then
      status="failed"
      reason="Geth RPC probe failed before epoch"
      failed_count=$((failed_count + 1))
      completed_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
      write_epoch_manifest "${epoch_dir}" "${epoch_id}" "${run_id}" "${started_at}" "${completed_at}" "${status}" "${reason}"
      append_epoch_summary "${epoch_id}" "${run_id}" "${status}" "${reason}"
      continue
    fi

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
