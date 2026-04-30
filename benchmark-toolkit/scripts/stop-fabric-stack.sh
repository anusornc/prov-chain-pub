#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TOOLKIT_DIR="${REPO_ROOT}/benchmark-toolkit"
COMPOSE_FILE="${TOOLKIT_DIR}/docker-compose.fabric.yml"

FABRIC_SAMPLES_DIR="${FABRIC_SAMPLES_DIR:-${HOME}/fabric-samples}"
FABRIC_TEST_NETWORK_DIR="${FABRIC_TEST_NETWORK_DIR:-${FABRIC_SAMPLES_DIR}/test-network}"
FABRIC_DOCKER_NETWORK="${FABRIC_DOCKER_NETWORK:-fabric_test}"

FABRIC_TEST_NETWORK_DIR="${FABRIC_TEST_NETWORK_DIR}" \
FABRIC_DOCKER_NETWORK="${FABRIC_DOCKER_NETWORK}" \
  docker compose -f "${COMPOSE_FILE}" down --remove-orphans

if [ -x "${FABRIC_TEST_NETWORK_DIR}/network.sh" ]; then
  (
    cd "${FABRIC_TEST_NETWORK_DIR}"
    ./network.sh down
  )
fi
