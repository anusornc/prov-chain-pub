# Docker Competitive Benchmark Architecture Plan - 2026-04-18

## Purpose

This document defines the Docker-based execution architecture for the next real competitive benchmark suite.

It exists to answer a concrete operational question:

**How should ProvChain, Neo4j, Fluree, Hyperledger Fabric, Geth, and the benchmark runner be orchestrated so the benchmark is reproducible, fair, and publication-defensible?**

This document does **not** claim that the full Docker benchmark stack already exists.

It is a deployment and orchestration plan that should guide future `docker compose` implementation.

## Inputs

This plan builds directly on:

- `docs/benchmarking/COMPETITIVE_BENCHMARK_FAIRNESS_MATRIX_2026-04-17.md`
- `docs/benchmarking/COMPETITIVE_BENCHMARK_SPEC_2026-04-17.md`
- `docs/benchmarking/COMPETITIVE_BENCHMARK_ADAPTER_IMPLEMENTATION_PLAN_2026-04-17.md`

## Decision

**The competitive benchmark suite should use Docker Compose as the primary orchestration mechanism.**

This is the right default because it provides:

- reproducible dependency setup,
- product version pinning,
- explicit service topology,
- portable benchmark execution on new machines,
- and cleaner artifact capture for publication and review.

## Scope

The Docker plan covers:

- systems under test,
- benchmark runner placement,
- shared datasets and result volumes,
- health-check and readiness behavior,
- resource controls,
- and staged rollout strategy.

It does **not** replace the benchmark specification or fairness matrix.

## Architecture Principle

Do **not** put every benchmark target into one monolithic container image.

Use:

1. one service per system under test,
2. one service for the benchmark runner,
3. shared mounted volumes for datasets and artifacts,
4. one Docker network per benchmark stack.

This keeps the topology explicit and makes service-specific failures visible.

## Benchmark Stack Layers

### Layer 1. Systems Under Test

Services:

- `provchain`
- `neo4j`
- `fluree`
- `fabric-gateway` or equivalent Fabric entry service
- `geth`

These services expose the actual APIs or RPC endpoints consumed by the benchmark adapters.

### Layer 2. Benchmark Runner

Service:

- `benchmark-runner`

Responsibilities:

- wait for service readiness,
- execute selected benchmark family workloads,
- collect timing data,
- export machine-readable results,
- export raw logs and summaries.

The runner must remain logically separate from the systems under test.

### Layer 3. Shared Artifacts

Shared volumes:

- `datasets/`
- `results/`
- `raw_logs/`
- `manifests/`

These should be mounted read-only where appropriate for inputs and writable only for output directories.

## Staged Compose Strategy

The benchmark suite should be implemented in **three staged compose files**, not as one all-in-one stack from the start.

### Stage A. Trace Stack

Recommended file:

- `benchmark-toolkit/docker-compose.trace.yml`

Services:

- `provchain`
- `neo4j`
- `fluree`
- `benchmark-runner`

Purpose:

- prove the multi-target trace-query family first
- reuse the strongest current adapter path
- keep orchestration complexity low while the result schema and workload structure stabilize

Benchmark families supported:

- trace query / provenance reconstruction
- partial semantic-family support where Fluree JSON-LD ingest is needed

### Stage B. Ledger Stack

Recommended file:

- `benchmark-toolkit/docker-compose.ledger.yml`

Services:

- `provchain`
- `fluree`
- `fabric-gateway`
- `geth`
- `benchmark-runner`

Purpose:

- isolate write-path and commit/finality comparisons
- avoid forcing Neo4j into a ledger family it is not designed to represent

Benchmark families supported:

- ledger / write path
- optional semantic-family steps where applicable

### Stage C. Full Stack

Recommended file:

- `benchmark-toolkit/docker-compose.full.yml`

Services:

- `provchain`
- `neo4j`
- `fluree`
- `fabric-gateway`
- `geth`
- `benchmark-runner`

Purpose:

- provide a single integration stack after Stage A and Stage B are already stable
- support final report generation from one orchestrated environment

This stack should be implemented **last**, not first.

## Service Topology

### 1. ProvChain Service

Expected role:

- reference implementation under test

Expected interfaces:

- health endpoint
- trace-query endpoint(s)
- transaction / import endpoint(s)

Expected mounts:

- config
- ontology package fixtures
- benchmark datasets
- runtime data volume

### 2. Neo4j Service

Expected role:

- graph-query baseline for the trace family

Expected interfaces:

- Bolt
- optional HTTP endpoint

Expected mounts:

- persistent graph data
- import data if needed

### 3. Fluree Service

Expected role:

- JSON-LD / ledger-style semantic baseline

Expected interfaces:

- health endpoint
- transaction endpoint
- query endpoint

Expected mounts:

- JSON-LD benchmark inputs
- persistent ledger volume

### 4. Fabric Service

Expected role:

- permissioned ledger baseline

Recommended topology:

- keep Fabric behind one benchmark-facing entry service such as a gateway container or a controlled test network entrypoint

Reason:

- the benchmark runner should not need to know internal peer/orderer topology details during the first implementation round

Expected interfaces:

- health endpoint for the gateway layer
- invoke/query path for benchmark chaincode

Expected mounts:

- network crypto material
- channel artifacts
- benchmark chaincode source or package

### 5. Geth Service

Expected role:

- public-chain baseline

Expected interfaces:

- JSON-RPC

Expected mounts:

- chain data
- pre-funded dev-network configuration

Important constraint:

- the dev chain configuration must be fixed and documented so receipt and confirmation timing are reproducible

### 6. Benchmark Runner Service

Expected role:

- orchestrated executor only

It should not persist benchmark truth inside the container image itself.

Expected mounts:

- datasets read-only
- output/results writable
- raw logs writable
- environment manifest writable

## Health Checks and Readiness

Each benchmark stack must define explicit health checks.

### Required behavior

1. service health checks run first,
2. benchmark runner waits until all required services are healthy,
3. warmup phase runs before measured iterations,
4. measured phase starts only after the stack is stable.

### Required health checks by target

- ProvChain:
  - REST health endpoint
- Neo4j:
  - Bolt/driver health check
- Fluree:
  - HTTP health endpoint
- Fabric:
  - gateway health endpoint or equivalent invocation check
- Geth:
  - JSON-RPC `web3_clientVersion` or equivalent lightweight RPC probe

## Dataset and Translation Strategy

Docker should not hide benchmark translation logic.

### Required rule

Each benchmark run must start from the same logical slice and then apply target-specific translation explicitly.

### Input layout

Recommended shared volume structure:

```text
benchmark-toolkit/
  datasets/
    logical/
      uht/
      hybrid-epcis-uht/
      healthcare-device/
      pharmaceutical-storage/
    translated/
      provchain/
      neo4j/
      fluree/
      fabric/
      geth/
```

### Required translation rule

- logical records are source of truth
- target-specific payloads are derived artifacts
- translated payload generation must be documented and rerunnable

## Output Artifact Strategy

Every benchmark stack must produce the same core output set.

Required artifacts:

- `environment_manifest.json`
- `benchmark_results.json`
- `benchmark_results.csv`
- `raw_logs/`
- `summary.md`

Recommended layout:

```text
benchmark-toolkit/results/
  trace/
  ledger/
  semantic/
```

Each run should also record:

- compose file used,
- git commit,
- benchmark target list,
- dataset slice,
- host machine metadata,
- container image tags.

## Resource Controls

Docker alone does not make the benchmark fair.

The compose stack must also specify:

- CPU limits,
- memory limits,
- storage assumptions,
- network mode assumptions,
- and service count.

### Required fairness rule

The benchmark report must distinguish:

- single-node service execution,
- dev-network execution,
- and multi-service clustered execution.

Do not report them as if they were equivalent.

## Phase-by-Phase Execution Plan

### Phase 1. Docker Trace Stack

Goal:

- bring up ProvChain + Neo4j + Fluree + benchmark-runner reliably

Success criteria:

- health checks stable
- benchmark runner can execute trace workloads against all three systems
- shared result schema exports cleanly

### Phase 2. Docker Ledger Stack

Goal:

- bring up ProvChain + Fluree + Fabric + Geth + benchmark-runner

Success criteria:

- Fabric and Geth move beyond health-check-only status
- ledger family metrics can be produced with fair labeling of submit vs commit vs confirmation

### Phase 3. Docker Full Stack

Goal:

- unify final publication-ready benchmark orchestration

Success criteria:

- all approved systems can be orchestrated from a documented set of compose files
- artifacts are exported in one consistent directory structure

## Immediate Next Implementation Step

The next implementation step should be:

1. create the Docker plan for the **trace stack first**
2. define exact service names, ports, volumes, and health-check commands
3. only then implement `docker-compose.trace.yml`

Do **not** start with the full stack.

## Non-Goals for the First Docker Milestone

The first Docker milestone should **not** attempt to:

- optimize absolute performance,
- benchmark every system family at once,
- claim publication-ready head-to-head results,
- or hide service-specific deployment differences.

The first milestone is about **controlled orchestration and reproducibility**.
