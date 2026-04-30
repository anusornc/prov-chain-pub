# Docker Trace Stack Service Specification - 2026-04-18

## Purpose

This document locks the exact service-level specification for the first Docker-based competitive benchmark stack:

- `ProvChain`
- `Neo4j`
- `Fluree`
- `benchmark-runner`

It is the implementation companion to:

- `docs/benchmarking/DOCKER_COMPETITIVE_BENCHMARK_ARCHITECTURE_PLAN_2026-04-18.md`

and the concrete compose file:

- `benchmark-toolkit/docker-compose.trace.yml`

## Scope

This stack is for the **trace-query family first**.

It is not yet the final publication-ready full benchmark stack.

## Service Inventory

### 1. `provchain`

Container name:

- `provchain-trace`

Role:

- reference system under test

Build:

- `context: ..`
- `dockerfile: deploy/Dockerfile.production`

Container port:

- `8080` HTTP API / health
- `9090` metrics

Host port mapping:

- `18080:8080`
- `19090:9090`

Health check:

- `curl -f http://localhost:8080/health`

Mounted volumes:

- `provchain_trace_data:/app/data`
- `../config:/app/config:ro`
- `./datasets:/benchmark/datasets:ro`
- `./results:/benchmark/results`

### 2. `neo4j`

Container name:

- `neo4j-trace`

Role:

- trace-query baseline

Image:

- `neo4j:5.15-community`

Container ports:

- `7474` HTTP
- `7687` Bolt

Host port mapping:

- `17474:7474`
- `17687:7687`

Health check:

- `cypher-shell -u neo4j -p benchmark "RETURN 1"`

Mounted volumes:

- `neo4j_trace_data:/data`
- `neo4j_trace_logs:/logs`
- `./datasets:/benchmark/datasets:ro`

### 3. `fluree`

Container name:

- `fluree-trace`

Role:

- JSON-LD / ledger-style semantic baseline for trace-query family

Image:

- `${FLUREE_IMAGE:-fluree/ledger:pin-required}`

Profile:

- `fluree`

Container port:

- `8090`

Host port mapping:

- `18090:8090`

Health check:

- `curl -f http://localhost:8090/fdb/health`
- `BENCHMARK_SKIP_FLUREE=false` เมื่อต้องการเปิด comparator นี้จริง

Mounted volumes:

- `fluree_trace_data:/var/lib/fluree`
- `./datasets:/benchmark/datasets:ro`

## Benchmark Runner

Container name:

- `benchmark-runner-trace`

Role:

- benchmark orchestrator

Build:

- `context: ./research-benchmarks`
- `dockerfile: Dockerfile`

Execution mode:

- query family first

Current command:

```bash
python3 /benchmark/scripts/translate_trace_dataset_to_jsonld.py \
  --input /benchmark/datasets/${PROVCHAIN_DATASET_FILE:-supply_chain_1000.ttl} \
  --output /benchmark/datasets/${FLUREE_DATASET_FILE:-translated/fluree/supply_chain_1000.jsonld} \
  && /benchmark/benchmark-runner --query
```

Required environment:

- `PROVCHAIN_URL=http://provchain:8080`
- `NEO4J_URI=bolt://neo4j:7687`
- `NEO4J_USER=neo4j`
- `NEO4J_PASSWORD=benchmark`
- `FLUREE_URL=http://fluree:8090`
- `FLUREE_LEDGER=provchain/benchmark`
- `DATASET_PATH=/benchmark/datasets`
- `RESULTS_PATH=/benchmark/results/trace`
- `PROVCHAIN_DATASET_FILE=supply_chain_1000.ttl`
- `NEO4J_DATASET_FILE=supply_chain_1000.ttl`
- `FLUREE_DATASET_FILE=translated/fluree/supply_chain_1000.jsonld`

Mounted volumes:

- `./datasets:/benchmark/datasets`
- `./results:/benchmark/results`
- `./logs:/benchmark/logs`

Depends on:

- `provchain` healthy
- `neo4j` healthy

Current baseline note:

- the stable baseline currently runs with `--skip-fluree`
- `Fluree` is deferred behind a profile until an explicit image pin and verified API contract are available

## Network

Network name:

- `trace_benchmark_net`

Driver:

- `bridge`

## Named Volumes

- `provchain_trace_data`
- `neo4j_trace_data`
- `neo4j_trace_logs`
- `fluree_trace_data`

## Current Data-Format Rule

This trace stack now assumes **per-target dataset files**:

- ProvChain uses Turtle
- Neo4j uses Turtle
- Fluree uses JSON-LD

This is intentional and necessary for fairness.

## Current Implementation Constraint

The previous manual blocker for Fluree dataset preparation has now been reduced:

- the runner image now carries a dedicated Turtle-to-JSON-LD translation script
- the trace compose stack now runs that translation step automatically before executing the query benchmark family
- the datasets mount for `benchmark-runner` is intentionally writable so the translated JSON-LD artifact can be emitted under:
  - `benchmark-toolkit/datasets/translated/fluree/supply_chain_1000.jsonld`

So the current status is now:

- service topology is defined,
- runner environment is defined,
- per-target dataset wiring exists in code,
- Fluree dataset preparation is automated in the trace stack,
- Fluree is no longer allowed to rely on `latest` as benchmark evidence,
- the stable baseline remains `ProvChain + Neo4j`,
- Fluree re-entry now requires:
  - explicit `FLUREE_IMAGE` pin,
  - local adapter contract tests,
  - verified API contract.

## Immediate Next Step

The next implementation step after this service spec is:

1. set an explicit `FLUREE_IMAGE` pin
2. validate the pinned image against the documented adapter contract
3. re-enable the `fluree` profile in the trace stack
4. verify that the JSON-LD artifact is emitted and consumed successfully by Fluree
5. then capture the first artifact-backed multi-target trace run
