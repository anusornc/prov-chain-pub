#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOLKIT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

TIGERGRAPH_CONTAINER_NAME="${TIGERGRAPH_CONTAINER_NAME:-tigergraph-benchmark}"
TIGERGRAPH_GRAPH="${TIGERGRAPH_GRAPH:-ProvChainTrace}"
TIGERGRAPH_DATASET_FILE="${TIGERGRAPH_DATASET_FILE:-${TOOLKIT_DIR}/datasets/supply_chain_1000.ttl}"
TIGERGRAPH_GENERATED_DIR="${TIGERGRAPH_GENERATED_DIR:-${TOOLKIT_DIR}/tigergraph/generated}"
TIGERGRAPH_EXEC="${TIGERGRAPH_EXEC:-docker exec ${TIGERGRAPH_CONTAINER_NAME} bash -lc}"
TIGERGRAPH_PATH_PREFIX='export PATH="/home/tigergraph/tigergraph/app/cmd:/home/tigergraph/tigergraph/app/bin:/home/tigergraph/tigergraph/bin:/home/tigergraph/app/cmd:/home/tigergraph/app/bin:$PATH";'

printf 'TigerGraph translated model install\n'
printf 'dataset: %s\n' "${TIGERGRAPH_DATASET_FILE}"
printf 'generated dir: %s\n' "${TIGERGRAPH_GENERATED_DIR}"
printf 'graph: %s\n' "${TIGERGRAPH_GRAPH}"

cargo run \
  --manifest-path "${TOOLKIT_DIR}/research-benchmarks/Cargo.toml" \
  --bin tigergraph-translate \
  -- \
  --input "${TIGERGRAPH_DATASET_FILE}" \
  --output-dir "${TIGERGRAPH_GENERATED_DIR}" \
  --graph "${TIGERGRAPH_GRAPH}"

printf '\n--- installing translated TigerGraph model\n'
${TIGERGRAPH_EXEC} "${TIGERGRAPH_PATH_PREFIX} gsql /benchmark/tigergraph/generated/load-and-query.gsql"

printf '\n--- smoke query product_lookup(BATCH001)\n'
curl -fsS "http://localhost:${TIGERGRAPH_RESTPP_PORT:-19000}/query/${TIGERGRAPH_GRAPH}/product_lookup?product_id=BATCH001"
printf '\nTigerGraph translated model install passed.\n'
