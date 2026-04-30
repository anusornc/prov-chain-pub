#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RESULTS_ROOT="${RESULTS_ROOT:-$ROOT_DIR/benchmark-toolkit/results/fabric-contract}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
RESULTS_PATH="$RESULTS_ROOT/$RUN_ID"
FABRIC_GATEWAY_URL="${FABRIC_GATEWAY_URL:-http://127.0.0.1:18800}"
ITERATIONS="${ITERATIONS:-1}"
FABRIC_BATCH_SIZE="${FABRIC_BATCH_SIZE:-10}"
SIM_LOG="$RESULTS_PATH/fabric-gateway-contract-sim.log"

mkdir -p "$RESULTS_PATH"

cleanup() {
  if [[ -n "${SIM_PID:-}" ]]; then
    kill "$SIM_PID" >/dev/null 2>&1 || true
    wait "$SIM_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

python3 "$ROOT_DIR/benchmark-toolkit/scripts/fabric-gateway-contract-sim.py" >"$SIM_LOG" 2>&1 &
SIM_PID="$!"

for _ in $(seq 1 40); do
  if curl -fsS "$FABRIC_GATEWAY_URL/health" >/dev/null; then
    break
  fi
  sleep 0.25
done

curl -fsS "$FABRIC_GATEWAY_URL/health" >/dev/null

cargo run \
  --manifest-path "$ROOT_DIR/benchmark-toolkit/research-benchmarks/Cargo.toml" \
  -- \
  --write \
  --policy \
  --skip-provchain \
  --skip-neo4j \
  --skip-fluree \
  --skip-geth \
  --fabric-gateway-url "$FABRIC_GATEWAY_URL" \
  --iterations "$ITERATIONS" \
  --fabric-batch-size "$FABRIC_BATCH_SIZE" \
  --results-path "$RESULTS_PATH"

echo "fabric contract smoke complete: $RESULTS_PATH"
