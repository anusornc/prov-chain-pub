#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="${FABRIC_INSTALL_DIR:-${HOME}}"
FABRIC_INSTALL_SCRIPT_URL="${FABRIC_INSTALL_SCRIPT_URL:-https://raw.githubusercontent.com/hyperledger/fabric/main/scripts/install-fabric.sh}"
FABRIC_VERSION="${FABRIC_VERSION:-}"
FABRIC_CA_VERSION="${FABRIC_CA_VERSION:-}"
FABRIC_NODEENV_VERSION="${FABRIC_NODEENV_VERSION:-2.5}"

mkdir -p "${INSTALL_DIR}"
cd "${INSTALL_DIR}"

if [ ! -f install-fabric.sh ]; then
  curl -sSLO "${FABRIC_INSTALL_SCRIPT_URL}"
  chmod +x install-fabric.sh
fi

args=()
if [ -n "${FABRIC_VERSION}" ]; then
  args+=(--fabric-version "${FABRIC_VERSION}")
fi
if [ -n "${FABRIC_CA_VERSION}" ]; then
  args+=(--ca-version "${FABRIC_CA_VERSION}")
fi
args+=(docker binary samples)

./install-fabric.sh "${args[@]}"

docker pull "hyperledger/fabric-nodeenv:${FABRIC_NODEENV_VERSION}"
docker tag "hyperledger/fabric-nodeenv:${FABRIC_NODEENV_VERSION}" "hyperledger/fabric-nodeenv:latest"

cat <<EOF_DONE

Fabric samples installed.

Use:
  FABRIC_SAMPLES_DIR=${INSTALL_DIR}/fabric-samples ./benchmark-toolkit/scripts/start-fabric-stack.sh

EOF_DONE
