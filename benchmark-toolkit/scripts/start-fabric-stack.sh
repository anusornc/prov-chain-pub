#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TOOLKIT_DIR="${REPO_ROOT}/benchmark-toolkit"
COMPOSE_FILE="${TOOLKIT_DIR}/docker-compose.fabric.yml"

FABRIC_SAMPLES_DIR="${FABRIC_SAMPLES_DIR:-${HOME}/fabric-samples}"
FABRIC_TEST_NETWORK_DIR="${FABRIC_TEST_NETWORK_DIR:-${FABRIC_SAMPLES_DIR}/test-network}"
FABRIC_CHANNEL="${FABRIC_CHANNEL:-provchain}"
FABRIC_CHAINCODE="${FABRIC_CHAINCODE:-traceability}"
FABRIC_CHAINCODE_PATH="${FABRIC_CHAINCODE_PATH:-${TOOLKIT_DIR}/fabric/chaincode/traceability-javascript}"
FABRIC_CHAINCODE_LANGUAGE="${FABRIC_CHAINCODE_LANGUAGE:-javascript}"
FABRIC_NODEENV_VERSION="${FABRIC_NODEENV_VERSION:-2.5}"
FABRIC_DOCKER_NETWORK="${FABRIC_DOCKER_NETWORK:-fabric_test}"
FABRIC_GATEWAY_HOST_PORT="${FABRIC_GATEWAY_HOST_PORT:-18800}"
RESET_FABRIC_NETWORK="${RESET_FABRIC_NETWORK:-true}"

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

check_docker() {
  docker info >/dev/null 2>&1 || die "Docker is not accessible from this shell"
}

check_fabric_samples() {
  [ -d "${FABRIC_TEST_NETWORK_DIR}" ] || die "Fabric test-network not found: ${FABRIC_TEST_NETWORK_DIR}. Set FABRIC_SAMPLES_DIR or FABRIC_TEST_NETWORK_DIR."
  [ -x "${FABRIC_TEST_NETWORK_DIR}/network.sh" ] || die "network.sh not executable: ${FABRIC_TEST_NETWORK_DIR}/network.sh"
}

ensure_nodeenv_image() {
  if [ "${FABRIC_CHAINCODE_LANGUAGE}" != "javascript" ] && [ "${FABRIC_CHAINCODE_LANGUAGE}" != "node" ]; then
    return
  fi
  if docker image inspect "hyperledger/fabric-nodeenv:${FABRIC_NODEENV_VERSION}" >/dev/null 2>&1; then
    return
  fi
  docker pull "hyperledger/fabric-nodeenv:${FABRIC_NODEENV_VERSION}"
}

stop_gateway() {
  FABRIC_TEST_NETWORK_DIR="${FABRIC_TEST_NETWORK_DIR}" \
  FABRIC_CHANNEL="${FABRIC_CHANNEL}" \
  FABRIC_CHAINCODE="${FABRIC_CHAINCODE}" \
  FABRIC_DOCKER_NETWORK="${FABRIC_DOCKER_NETWORK}" \
  FABRIC_GATEWAY_HOST_PORT="${FABRIC_GATEWAY_HOST_PORT}" \
    docker compose -f "${COMPOSE_FILE}" down --remove-orphans >/dev/null 2>&1 || true
}

start_test_network() {
  (
    cd "${FABRIC_TEST_NETWORK_DIR}"
    if [ "${RESET_FABRIC_NETWORK}" = "true" ]; then
      ./network.sh down
    fi
    ./network.sh up createChannel -c "${FABRIC_CHANNEL}" -ca
    ./network.sh deployCC -c "${FABRIC_CHANNEL}" -ccn "${FABRIC_CHAINCODE}" -ccp "${FABRIC_CHAINCODE_PATH}" -ccl "${FABRIC_CHAINCODE_LANGUAGE}"
  )
}

start_gateway() {
  FABRIC_TEST_NETWORK_DIR="${FABRIC_TEST_NETWORK_DIR}" \
  FABRIC_CHANNEL="${FABRIC_CHANNEL}" \
  FABRIC_CHAINCODE="${FABRIC_CHAINCODE}" \
  FABRIC_DOCKER_NETWORK="${FABRIC_DOCKER_NETWORK}" \
  FABRIC_GATEWAY_HOST_PORT="${FABRIC_GATEWAY_HOST_PORT}" \
    docker compose -f "${COMPOSE_FILE}" up --build -d fabric-gateway
}

show_gateway_logs() {
  FABRIC_TEST_NETWORK_DIR="${FABRIC_TEST_NETWORK_DIR}" \
  FABRIC_CHANNEL="${FABRIC_CHANNEL}" \
  FABRIC_CHAINCODE="${FABRIC_CHAINCODE}" \
  FABRIC_DOCKER_NETWORK="${FABRIC_DOCKER_NETWORK}" \
  FABRIC_GATEWAY_HOST_PORT="${FABRIC_GATEWAY_HOST_PORT}" \
    docker compose -f "${COMPOSE_FILE}" logs --no-color --tail=120 fabric-gateway || true
}

main() {
  check_docker
  check_fabric_samples
  ensure_nodeenv_image
  stop_gateway
  start_test_network
  stop_gateway
  start_gateway

  printf 'Fabric gateway starting on http://localhost:%s\n' "${FABRIC_GATEWAY_HOST_PORT}"
  if ! FABRIC_GATEWAY_URL="http://localhost:${FABRIC_GATEWAY_HOST_PORT}" "${SCRIPT_DIR}/probe-fabric-gateway.sh"; then
    printf '\nFabric gateway probe failed. Recent gateway logs:\n' >&2
    show_gateway_logs >&2
    exit 1
  fi
}

main "$@"
