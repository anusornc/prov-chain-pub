# Fabric Benchmark Runtime

This directory contains the real Fabric runtime assets for B013.

It uses the official Fabric `test-network` as the peer/orderer/channel runtime,
deploys the `traceability` Go chaincode, and starts a benchmark-facing REST
gateway on host port `18800`.

## Start

Install or provide Fabric samples first:

```bash
FABRIC_INSTALL_DIR=$HOME ./benchmark-toolkit/scripts/install-fabric-samples.sh
```

Then run from the repository root:

```bash
FABRIC_SAMPLES_DIR=$HOME/fabric-samples \
./benchmark-toolkit/scripts/start-fabric-stack.sh
```

This starts the official Fabric test-network, creates channel `provchain`,
deploys chaincode `traceability`, and starts the REST gateway container from:

- `benchmark-toolkit/docker-compose.fabric.yml`
- `benchmark-toolkit/fabric/chaincode/traceability-javascript/`
- `benchmark-toolkit/fabric/gateway/`

The script expects:

- `$FABRIC_SAMPLES_DIR/test-network/network.sh`
- Docker access from the current shell
- Fabric test-network Docker images available or pullable

Stop the stack with:

```bash
FABRIC_SAMPLES_DIR=$HOME/fabric-samples \
./benchmark-toolkit/scripts/stop-fabric-stack.sh
```

## Contract

The REST gateway must satisfy:

- `GET /health`
- `POST /ledger/records`
- `POST /ledger/records/batch`
- `POST /policy/check`

Probe it with:

```bash
FABRIC_GATEWAY_URL=http://localhost:18800 ./benchmark-toolkit/scripts/probe-fabric-gateway.sh
```

Then run B013 smoke:

```bash
FABRIC_GATEWAY_URL=http://localhost:18800 \
PROVCHAIN_URL=http://localhost:8080 \
./benchmark-toolkit/scripts/provchain-fabric-campaign.sh smoke
```

The local simulator under `scripts/fabric-gateway-contract-sim.py` remains a
contract-only helper and must not be used as comparative Fabric evidence.
