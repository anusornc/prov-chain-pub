# GraphDB / TigerGraph Comparator Task List - 2026-04-30

## Purpose

เอกสารนี้เป็น task list สำหรับขยาย benchmark comparator หลัง benchmark round หลักปิดแล้ว

เป้าหมายคือเพิ่ม evidence ให้ตรงกับ thesis proposal ที่ระบุ graph database comparators เพิ่มเติม โดยไม่ทำให้หลักฐานเดิมที่ clean แล้วปนกับรอบทดลองใหม่

Primary existing evidence remains:

- `ProvChain vs Neo4j vs Fluree` สำหรับ `trace-query`
- `ProvChain vs Fluree` สำหรับ `semantic`
- `ProvChain vs Fabric` สำหรับ `ledger-write` และ `governance-policy`
- `ProvChain vs Geth` สำหรับ public-chain baseline
- `R002` สำหรับ bulk Turtle dataset admission

## Strategic Decision

เพิ่ม comparator แบบเป็นชั้น ไม่ rerun benchmark ทั้งหมด

1. `GraphDB` มาก่อน เพราะเป็น RDF/SPARQL semantic graph database และตรงกับ thesis ontology/RDF path มากกว่า
2. `TigerGraph` เป็น optional secondary comparator เพราะเป็น native property graph ต้องแปลง RDF เป็น property graph model เหมือนหรือมากกว่า Neo4j จึงมี fairness caveat สูง
3. Paper จะ update หลัง comparator campaign ผ่าน validity gate และ export evidence เข้า `docs/benchmarking/data/reference/` แล้วเท่านั้น

## External Runtime Notes

### GraphDB

- Product: Ontotext GraphDB
- Runtime type: RDF/SPARQL triplestore
- Docker image: `ontotext/graphdb`
- Current benchmark smoke default: `ontotext/graphdb:10.8.13`, selected as a pinned GraphDB 10.x runtime before requiring a local GraphDB 11 license-backed path.
- Important licensing note: GraphDB 11+ requires a valid license, including Free edition.
- License file placement note: GraphDB documentation and Docker guidance use a file named `graphdb.license` under the mounted GraphDB home, commonly `conf/graphdb.license`; the startup script also copies to `work/graphdb.license` and sets `-Dgraphdb.license.file=/opt/graphdb/home/conf/graphdb.license` when `GRAPHDB_LICENSE_FILE` is supplied.
- Sources:
  - https://www.ontotext.com/products/graphdb/
  - https://hub.docker.com/r/ontotext/graphdb/

### TigerGraph

- Product: TigerGraph DB
- Runtime type: native property graph / graph analytics platform
- Docker image family: `tigergraph/tigergraph`
- Important runtime note: Docker image is for personal or R&D use, not production; it requires higher resources than the current trace stack and has default credentials that must not be exposed.
- Sources:
  - https://docs.tigergraph.com/tigergraph-server/4.2/getting-started/
  - https://docs.tigergraph.com/tigergraph-server/3.11/getting-started/docker

## Claim Boundary

- GraphDB rows may be treated as `native-rdf-path` only if the campaign loads the same Turtle/RDF dataset into GraphDB and executes SPARQL equivalents of the benchmark queries.
- TigerGraph rows must be treated as `translated-property-graph-model` or `secondary-translated-graph-baseline`.
- TigerGraph must not be used as semantic/RDF-native evidence.
- Failed GraphDB/TigerGraph integration attempts must not enter primary paper tables.
- Existing Neo4j/Fluree/Fabric/Geth/R002 evidence remains valid and should not be rerun unless the comparator harness changes shared query semantics.

## Task List

### C001 - Register Comparator Expansion In Benchmark Strategy

- Priority: `P0`
- Status: `Done`
- Goal: ทำให้ repo มีแผนงานชัดเจนก่อนเริ่มแก้ benchmark runner
- Tasks:
  - [x] สร้าง task list เฉพาะ GraphDB/TigerGraph
  - [x] ผูก task list นี้กับ post-round remediation master list เป็น `R006`
  - [x] update continuity memory เพื่อให้ session ถัดไปเริ่มที่ `G001`
- Done when:
  - เอกสารนี้ถูก commit
  - master benchmark task list อ้างถึงเอกสารนี้

### G001 - GraphDB Runtime Feasibility Gate

- Priority: `P0`
- Status: `Done`
- Goal: ยืนยันว่า GraphDB รันใน Docker ได้แบบ repeatable และ license/runtime ไม่บล็อก
- Tasks:
  - [x] เลือก GraphDB image tag แบบ pinned version ไม่ใช้ `latest`
  - [x] ระบุ license file path ผ่าน env เช่น `GRAPHDB_LICENSE_FILE` โดยไม่ commit license
  - [x] เพิ่มหรือทดลอง `docker-compose.graphdb.yml`
  - [x] เพิ่ม scripted health check `http://localhost:7200/rest/repositories`
  - [x] เพิ่ม scripted repository create ผ่าน `POST /rest/repositories`
  - [x] เพิ่ม scripted minimal Turtle fixture load ผ่าน RDF4J statements endpoint
  - [x] เพิ่ม scripted minimal SPARQL `ASK` query ผ่าน HTTP
  - [x] run live Docker smoke ใน shell ที่เข้าถึง Docker daemon ได้
  - [x] rerun live Docker smoke with the default `ontotext/graphdb:10.8.13` free-mode path
  - [ ] rerun live Docker smoke with `GRAPHDB_LICENSE_FILE` only if GraphDB 11.x evidence is required
- Done when:
  - smoke script แสดง health/load/query ผ่าน
  - ถ้า license บล็อก ต้องบันทึกเป็น blocker พร้อมไม่แตะ paper claim
Current implementation evidence:
  - compose: `benchmark-toolkit/docker-compose.graphdb.yml`
  - pinned default image: `ontotext/graphdb:10.8.13`
  - fixture: `benchmark-toolkit/graphdb/minimal-trace-fixture.ttl`
  - repository config template: `benchmark-toolkit/graphdb/repository-config.ttl.template`
  - scripts:
    - `benchmark-toolkit/scripts/start-graphdb-stack.sh`
    - `benchmark-toolkit/scripts/probe-graphdb.sh`
  - local static validation:
    - `bash -n benchmark-toolkit/scripts/probe-graphdb.sh` passed
    - `bash -n benchmark-toolkit/scripts/start-graphdb-stack.sh` passed
    - `docker compose -f benchmark-toolkit/docker-compose.graphdb.yml config` passed
    - `git diff --check` passed
  - live smoke attempt from the agent shell failed before GraphDB startup because Docker socket access was denied:
    - `permission denied while trying to connect to the Docker daemon socket`
    - `sudo -n docker info` also failed because a password is required
    - this is an agent-shell access blocker, not a GraphDB runtime result
  - user-run live smoke without license:
    - command: `GRAPHDB_IMAGE=ontotext/graphdb:11.3.2 ./benchmark-toolkit/scripts/start-graphdb-stack.sh`
    - GraphDB container started successfully
    - `GET /rest/repositories` returned `200`
    - `DELETE /rest/repositories/provchain_smoke` returned `200`
    - `POST /rest/repositories` returned `201`
    - `POST /repositories/provchain_smoke/statements` returned `500` with body `No license was set`
    - conclusion: runtime path is viable, but GraphDB comparator work is blocked until a valid GraphDB license is supplied
  - next command in a Docker-enabled shell with license:
    - `GRAPHDB_LICENSE_FILE=/path/to/graphdb.license GRAPHDB_IMAGE=ontotext/graphdb:11.3.2 ./benchmark-toolkit/scripts/start-graphdb-stack.sh`
  - license placement fix:
    - `start-graphdb-stack.sh` now copies `GRAPHDB_LICENSE_FILE` to both `${GRAPHDB_HOME_DIR}/conf/graphdb.license` and `${GRAPHDB_HOME_DIR}/work/graphdb.license`
    - it also appends `-Dgraphdb.license.file=/opt/graphdb/home/conf/graphdb.license` to `GDB_JAVA_OPTS` if not already set
  - fallback smoke path without a local license file:
    - no GraphDB license file was found under `/home/cit` or `/tmp` during agent-side path search
    - command: `./benchmark-toolkit/scripts/stop-graphdb-stack.sh`
    - command: `GRAPHDB_IMAGE=ontotext/graphdb:10.8.13 ./benchmark-toolkit/scripts/start-graphdb-stack.sh`
    - user-run result: `GET /rest/repositories` returned `200`, `POST /rest/repositories` returned `201`, Turtle fixture ingest returned `204`, SPARQL `ASK` returned `200` with `"boolean": true`
    - conclusion: `GraphDB feasibility probe passed.`
  - cleanup command:
    - `./benchmark-toolkit/scripts/stop-graphdb-stack.sh`

### G002 - GraphDB Adapter Contract

- Priority: `P0`
- Status: `Done`
- Goal: เพิ่ม adapter ที่วัดแบบ fair กับ ProvChain trace-query path
- Tasks:
  - [x] เพิ่ม `GraphDbAdapter` ใน `benchmark-toolkit/research-benchmarks`
  - [x] เพิ่ม config: `GRAPHDB_URL`, `GRAPHDB_REPOSITORY`, timeout, credentials optional
  - [x] implement health check
  - [x] implement repository create/delete/reset
  - [x] implement Turtle load timing
  - [x] implement SPARQL query execution timing
  - [x] emit trace query rows as `System=GraphDB` with repository/graph metadata
  - [x] เพิ่ม unit tests ด้วย mock HTTP server
- Done when:
  - `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml graphdb -- --nocapture` ผ่าน
Current implementation evidence:
  - adapter: `benchmark-toolkit/research-benchmarks/src/adapters/graphdb.rs`
  - public exports: `GraphDbAdapter`, `GraphDbConfig`, `GraphDbTurtleLoadTiming`
  - validation: `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml graphdb -- --nocapture` passed with 6 GraphDB adapter tests
  - `Path=native-rdf-path` and `Fairness=native-comparable` labels will be attached when GraphDB rows are wired into the campaign runner in `G004`

### G003 - GraphDB Query Parity

- Priority: `P0`
- Status: `Done`
- Goal: ให้ GraphDB ใช้ query semantics เดียวกับ ProvChain/Fluree trace workload
- Tasks:
  - [x] reuse SPARQL query definitions จาก benchmark trace workload
  - [x] ตรวจ Simple Product Lookup
  - [x] ตรวจ Multi-hop Traceability
  - [x] ตรวจ Aggregation by Producer
  - [x] เพิ่ม contract test ว่า query คืน result shape ที่ runner คาดไว้
- Done when:
  - query contract tests ผ่านบน fixture เดียวกัน
Current implementation evidence:
  - GraphDB adapter calls shared `provchain_queries` SPARQL builders for the three trace-query scenarios.
  - Unit tests validate query shape and expected SPARQL JSON result counting for lookup, multi-hop, and aggregation.
  - GraphDB Turtle ingest uses an RDF4J-compatible N-Triples resource for named graph context, e.g. `<http://provchain.org/benchmark/graphdb/trace>`.

### G004 - GraphDB Campaign Wrapper

- Priority: `P0`
- Status: `Done`
- Goal: ทำให้รันง่ายแบบ one-command เหมือน campaign อื่น
- Tasks:
  - [x] เพิ่ม script เช่น `benchmark-toolkit/scripts/provchain-neo4j-fluree-graphdb-campaign.sh`
  - [x] รองรับ `smoke`, `profile`, `full`
  - [x] map GraphDB benchmark result rows เป็น `Path=native-rdf-path`, `Fairness=native-comparable`
  - [x] ใช้ isolated host ports และบันทึก port set ใน manifest
  - [x] เพิ่ม non-empty campaign directory guard
  - [x] export curated reference evidence อัตโนมัติเมื่อ run ผ่าน
  - [x] เพิ่ม validity gate ว่าทุก result row ต้อง `success=true`
- Done when:
  - `bash -n` ผ่าน
  - smoke command documented
Current implementation evidence:
  - runner integration: `benchmark-toolkit/research-benchmarks/src/main.rs`
  - Docker profile: `benchmark-toolkit/docker-compose.trace.yml` profile `graphdb`
  - one-command wrapper: `benchmark-toolkit/scripts/provchain-neo4j-fluree-graphdb-campaign.sh`
  - validation passed:
    - `cargo check --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml`
    - `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml -- --nocapture`
    - `bash -n benchmark-toolkit/scripts/run-trace-campaign.sh`
    - `bash -n benchmark-toolkit/scripts/provchain-neo4j-fluree-graphdb-campaign.sh`
    - `docker compose -f benchmark-toolkit/docker-compose.trace.yml --profile graphdb config`
  - smoke command:
    - `./benchmark-toolkit/scripts/provchain-neo4j-fluree-graphdb-campaign.sh smoke --id smoke_trace_supply1000_provchain-neo4j-fluree-graphdb_n1_20260430`

### G005 - GraphDB Smoke Campaign

- Priority: `P0`
- Status: `Done`
- Goal: ตรวจ end-to-end ก่อนรันเต็ม
- Tasks:
  - [x] run `smoke` ด้วย `EPOCHS=1`, `ITERATIONS=1` หรือผ่านด้วย profile gate ที่เข้มกว่า
  - [x] ตรวจ `campaign_status.json`
  - [x] ตรวจ aggregate summary ว่ามี GraphDB rows
  - [x] mark failed rows เป็น blocker ไม่ promote
- Done when:
  - smoke ผ่าน `1/1`
Current evidence:
  - user ran profile candidate `20260430_trace_supply1000_provchain-neo4j-fluree-graphdb_n3` before this smoke gate was closed
  - campaign status was `partial`, with 0 passed epochs and 3 failed epochs
  - failure was limited to `GraphDB / Data Loading / Turtle RDF Import`
  - root cause: RDF4J `context` parameter requires a legal N-Triples resource; the adapter sent `http://provchain.org/benchmark/graphdb/trace` instead of `<http://provchain.org/benchmark/graphdb/trace>`
  - fix: `GraphDbAdapter` now wraps plain graph IRIs as N-Triples resources before calling `/statements`
  - validation after fix:
    - `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml graphdb -- --nocapture` passed with 9 GraphDB adapter tests
    - `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml -- --nocapture` passed with 37 tests
  - user reran profile candidate `20260501_trace_supply1000_provchain-neo4j-fluree-graphdb_n3_contextfix`
  - campaign status was still non-publication evidence because GraphDB import failed in all 3 epochs
  - new root cause: GraphDB's strict Turtle parser rejected benchmark dataset slashed CURIE tokens such as `ex:Producer/Farm001`
  - fix: `GraphDbAdapter` now normalizes Turtle with the shared benchmark normalizer before posting to GraphDB, matching the parser-portability treatment already used by ProvChain/Neo4j paths
  - failure rows now retain `Path=native-rdf-path` and `Fairness=native-comparable` labels so failed aggregates do not misleadingly fall back to `Path=native`
  - validation after normalization fix:
    - `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml graphdb -- --nocapture` passed with 9 GraphDB adapter tests
    - `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml -- --nocapture` passed with 37 tests
  - rerun required with a new campaign id; do not reuse either failed candidate id
  - next command:
    - `./benchmark-toolkit/scripts/provchain-neo4j-fluree-graphdb-campaign.sh smoke --id smoke_trace_supply1000_provchain-neo4j-fluree-graphdb_n1_20260501_ttlfix`
  - profile rerun `20260501_trace_supply1000_provchain-neo4j-fluree-graphdb_n3_ttlfix` passed `3/3`; this supersedes the smoke gate for end-to-end readiness

### G006 - GraphDB Profile Campaign

- Priority: `P1`
- Status: `Done`
- Goal: ได้ profile evidence ก่อน full n30
- Tasks:
  - [x] run `profile` เช่น `EPOCHS=3`, `ITERATIONS=10`
  - [x] export evidence เข้า `docs/benchmarking/data/reference/`
  - [x] เขียน short analysis note เรื่อง load/query/fairness
- Done when:
  - profile ผ่าน `3/3`
  - analysis note ถูก commit
Current evidence:
  - campaign: `20260501_trace_supply1000_provchain-neo4j-fluree-graphdb_n3_ttlfix`
  - status: `passed`, `3/3`
  - curated export: `docs/benchmarking/data/reference/trace_supply_chain_1000_provchain_neo4j_fluree_graphdb_n3_20260501/`
  - analysis note: `docs/benchmarking/data/reference/trace_supply_chain_1000_provchain_neo4j_fluree_graphdb_n3_20260501/ANALYSIS_NOTE.md`
  - evidence role: `graphdb_comparator_reference_candidate`
  - caveat: profile evidence only; `G007` full n30 remains required before publication-facing GraphDB claims

### G007 - GraphDB Full Campaign

- Priority: `P1`
- Status: `Done`
- Goal: ได้ publication-facing comparator coverage
- Tasks:
  - [x] run `full` เช่น `EPOCHS=30`, `ITERATIONS=10`
  - [x] export curated reference evidence
  - [x] update publication benchmark report generator/source list
  - [x] update `PUBLICATION_BENCHMARK_REPORT_BUNDLE_2026-04-28.md`
- Done when:
  - full campaign ผ่าน `30/30`
  - report มี GraphDB trace-query rows พร้อม caveat
Current evidence:
  - campaign: `20260501_trace_supply1000_provchain-neo4j-fluree-graphdb_n30`
  - status: `passed`, `30/30`
  - curated export: `docs/benchmarking/data/reference/trace_supply_chain_1000_provchain_neo4j_fluree_graphdb_n30_20260501/`
  - analysis note: `docs/benchmarking/data/reference/trace_supply_chain_1000_provchain_neo4j_fluree_graphdb_n30_20260501/ANALYSIS_NOTE.md`
  - GraphDB is now available as publication-facing RDF/SPARQL-native comparator evidence for `supply_chain_1000` trace-query

### T001 - TigerGraph Feasibility Gate

- Priority: `P2`
- Status: `Done`
- Goal: ตัดสินใจด้วย evidence ว่าควรทำ TigerGraph ต่อหรือไม่
- Tasks:
  - [x] เลือก TigerGraph image tag แบบ pinned version
  - [x] ตรวจ Docker resource requirement บนเครื่อง benchmark
  - [x] start container แบบไม่ expose credentials เกินจำเป็น
  - [x] start TigerGraph services ภายใน container
  - [x] verify REST/GSQL access
  - [x] สร้าง graph schema แบบ minimal
  - [x] load fixture เล็ก
  - [x] query fixture เล็ก
  - [x] run live Docker smoke ใน shell ที่เข้าถึง Docker daemon ได้
- Done when:
  - ถ้าผ่าน: mark `T002` ready
  - ถ้าไม่ผ่าน: archive เป็น `deferred` พร้อมเหตุผลและไม่ใช้ใน paper
Current implementation evidence:
  - compose: `benchmark-toolkit/docker-compose.tigergraph.yml`
  - pinned default image: `tigergraph/community:4.2.2`
  - fixture files:
    - `benchmark-toolkit/tigergraph/minimal-products.csv`
    - `benchmark-toolkit/tigergraph/minimal-producers.csv`
    - `benchmark-toolkit/tigergraph/minimal-produced-by.csv`
    - `benchmark-toolkit/tigergraph/minimal-trace-smoke.gsql`
  - scripts:
    - `benchmark-toolkit/scripts/start-tigergraph-stack.sh`
    - `benchmark-toolkit/scripts/probe-tigergraph.sh`
    - `benchmark-toolkit/scripts/stop-tigergraph-stack.sh`
  - static validation:
    - `bash -n benchmark-toolkit/scripts/start-tigergraph-stack.sh benchmark-toolkit/scripts/probe-tigergraph.sh benchmark-toolkit/scripts/stop-tigergraph-stack.sh` passed
    - `docker compose -f benchmark-toolkit/docker-compose.tigergraph.yml config` passed
  - agent-shell live Docker blocker:
    - `docker info` failed with `permission denied while trying to connect to the docker API at unix:///var/run/docker.sock`
    - this is an agent-shell access blocker, not a TigerGraph runtime result
  - first user-run live smoke pulled `tigergraph/community:4.2.2` successfully but failed before TigerGraph service startup:
    - Docker returned `unable to upgrade to tcp, received 409` on the initial `docker exec`
    - root cause is treated as a container exec readiness race immediately after `compose up -d`, not a TigerGraph benchmark result
    - `start-tigergraph-stack.sh` now waits until the container is running and `gadmin` is exec-ready before running `gadmin start all`
  - second user-run live smoke failed because the compose file bind-mounted an empty host directory over `/home/tigergraph`
    - container logs showed `/home/tigergraph/entrypoint.sh: No such file or directory`
    - root cause is a tooling bug: the bind mount hid TigerGraph's image-provided runtime files
    - fix: remove the `/home/tigergraph` bind mount; the feasibility smoke now mounts only benchmark fixtures read-only
  - third user-run showed the container running and TigerGraph services starting, but the script still waited for exec-readiness
    - logs showed `Starting ZK ETCD DICT KAFKA ADMIN GSE NGINX GPE RESTPP ...`
    - root cause is likely the script overriding the image default runtime environment with `docker exec -u tigergraph`
    - fix: use the image default exec user/environment and check both `command -v gadmin` and the known app path
  - next exact command in a Docker-enabled shell:
    - `./benchmark-toolkit/scripts/stop-tigergraph-stack.sh`
    - `./benchmark-toolkit/scripts/start-tigergraph-stack.sh`
  - cleanup command:
    - `./benchmark-toolkit/scripts/stop-tigergraph-stack.sh`
  - claim boundary:
    - TigerGraph remains feasibility-only and must not be used in paper tables until a translated-model adapter and passed campaign evidence exist
Live smoke result:
  - container `tigergraph-benchmark` was running from `tigergraph/community:4.2.2`
  - RESTPP `/echo` returned `HTTP/1.1 200 OK` with `{"error":false, "message":"Hello GSQL"}`
  - `probe-tigergraph.sh` completed:
    - `gsql version`
    - minimal graph schema creation
    - CSV fixture load
    - `product_lookup("BATCH001")`
  - query result returned `Product` vertex `BATCH001`
  - script printed `TigerGraph feasibility probe passed.`
  - `T002` is now ready, but all future TigerGraph rows must be labeled as translated property-graph evidence, not RDF-native evidence

### T002 - TigerGraph Translated-Model Adapter

- Priority: `P2`
- Status: `Done`
- Goal: เพิ่ม property-graph comparator โดยไม่ปนกับ RDF-native claims
- Tasks:
  - [x] define vertex/edge schema จาก supply-chain RDF model
  - [x] implement RDF-to-TigerGraph translation
  - [x] implement generated CSV/GSQL data loader artifacts
  - [x] verify live data loader install against TigerGraph container
  - [x] implement installed query workload contracts
  - [x] preserve translated query parity caveat before campaign evidence is publication-facing
  - [x] label rows as `translated-property-graph-model`
  - [x] add fairness caveat in benchmark result metadata and capability labels
- Done when:
  - smoke fixture query ผ่าน
  - adapter tests ผ่าน
Current implementation evidence:
  - adapter module: `benchmark-toolkit/research-benchmarks/src/adapters/tigergraph.rs`
  - public exports: `TigerGraphAdapter`, `TigerGraphConfig`, `TigerGraphTranslatedDataset`, `translate_turtle_to_tigergraph`
  - capability path added: `translated-property-graph-model`
  - adapter contract:
    - health check via RESTPP `/echo`
    - installed query contract for `product_lookup`
    - installed query contract for `multi_hop_trace`
    - installed query contract for `aggregation_by_producer`
  - translator contract:
    - normalizes Turtle before parsing
    - maps Products, Actors, Transactions, Product-Actor edges, Product-Transaction edges, and Transaction-Party edges
  - validation:
    - `cargo check --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml` passed
    - `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml tigergraph -- --nocapture` passed with 5 tests
    - `bash -n benchmark-toolkit/scripts/install-tigergraph-trace-model.sh` passed
    - `cargo run --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml --bin tigergraph-translate -- --input benchmark-toolkit/datasets/supply_chain_1000.ttl --output-dir benchmark-toolkit/tigergraph/generated --graph ProvChainTrace` passed
  - generated artifact validation:
    - generated files are written under ignored path `benchmark-toolkit/tigergraph/generated/`
    - generated model counts for `supply_chain_1000`: products `42`, actors `9`, transactions `41`
    - generated files include CSV vertex/edge files plus `load-and-query.gsql`
    - generated GSQL currently uses conservative typed-neighbor traversal for `multi_hop_trace`; treat this as a live-install contract until parity is verified before `T003`
  - live install status:
    - user-run `./benchmark-toolkit/scripts/install-tigergraph-trace-model.sh` passed in a Docker-enabled shell
    - `product_lookup(BATCH001)` returned a `Product` vertex with `batch_id=BATCH001` and `product_type=Tomato`
    - script printed `TigerGraph translated model install passed.`
  - validation caveat:
    - full benchmark-runner unit test suite failed in the agent sandbox because `mockito` could not bind local mock servers (`Operation not permitted`), matching previous sandbox-only behavior for HTTP adapter tests
  - next task before T003:
    - run the TigerGraph translated-model campaign smoke via `provchain-neo4j-tigergraph-campaign.sh`

### T003 - TigerGraph Campaigns

- Priority: `P2`
- Status: `Done`
- Goal: เก็บ evidence เฉพาะถ้า feasibility ผ่านจริง
- Tasks:
  - [x] run smoke `n1`
  - [x] run clean profile `n3`
  - [x] decide whether full `n30` is worth running
  - [x] run full `n30`
  - [x] export evidence or archive blocker
- Done when:
  - มี either full evidence หรือ documented deferred decision
Current implementation evidence:
  - runner integration:
    - `benchmark-toolkit/research-benchmarks/src/main.rs` now supports `TIGERGRAPH_URL`, `TIGERGRAPH_GRAPH`, `TIGERGRAPH_TIMEOUT_SECONDS`, and `BENCHMARK_SKIP_TIGERGRAPH`
    - TigerGraph trace-query rows are labeled `Fairness=secondary-baseline` and `Path=translated-property-graph-model`
  - Docker compose integration:
    - `benchmark-toolkit/docker-compose.trace.yml` passes TigerGraph env vars to the benchmark runner
    - `host.docker.internal:host-gateway` is configured so the runner container can reach the host TigerGraph RESTPP port
  - one-command wrapper:
    - `benchmark-toolkit/scripts/provchain-neo4j-tigergraph-campaign.sh`
    - it installs the TigerGraph translated model first unless `--skip-install` is supplied
  - static validation passed:
    - `bash -n benchmark-toolkit/scripts/run-trace-campaign.sh benchmark-toolkit/scripts/provchain-neo4j-tigergraph-campaign.sh`
    - `docker compose -f benchmark-toolkit/docker-compose.trace.yml config`
    - `cargo check --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml`
    - `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml tigergraph -- --nocapture`
    - `git diff --check`
  - agent-shell smoke attempt:
    - `./benchmark-toolkit/scripts/provchain-neo4j-tigergraph-campaign.sh smoke --id smoke_trace_supply1000_provchain-neo4j-tigergraph_n1_20260502`
    - failed before running because Docker daemon is inaccessible from the agent shell
    - this is an agent-shell permission blocker, not a T003 runtime result
  - next command in a Docker-enabled shell:
    - `./benchmark-toolkit/scripts/provchain-neo4j-tigergraph-campaign.sh smoke --id smoke_trace_supply1000_provchain-neo4j-tigergraph_n1_20260502`
  - smoke result:
    - campaign: `smoke_trace_supply1000_provchain-neo4j-tigergraph_n1_20260502`
    - status: `passed`, `1/1`
    - completed at: `2026-05-02T04:25:52Z`
    - TigerGraph rows were present for Simple Product Lookup, Multi-hop Traceability, and Aggregation by Producer
    - TigerGraph path label: `translated-property-graph-model`
    - smoke means:
      - Simple Product Lookup: `3.946 ms`
      - Multi-hop Traceability (10 hops): `4.503 ms`
      - Aggregation by Producer: `5.927 ms`
  - next command in a Docker-enabled shell:
    - `./benchmark-toolkit/scripts/provchain-neo4j-tigergraph-campaign.sh profile --id 20260502_trace_supply1000_provchain-neo4j-tigergraph_n3`
  - first profile attempt:
    - campaign: `20260502_trace_supply1000_provchain-neo4j-tigergraph_n3`
    - status: `partial`, `2/3`
    - epoch-001 and epoch-003 passed
    - epoch-002 failed before benchmark execution during Docker image metadata resolution
    - Docker error included DNS timeout resolving Docker Hub metadata for `node:18-alpine`, Rust, and Debian base images
    - aggregate from passed epochs showed `100%` success for all emitted TigerGraph rows, but this campaign is not exportable because the campaign did not pass `3/3`
  - mitigation:
    - `run-trace-campaign.sh` now supports `BUILD_IMAGES=each|once|never`
    - `provchain-neo4j-tigergraph-campaign.sh` defaults to `BUILD_IMAGES=once`, so images are built before the campaign loop instead of before every epoch
  - rerun with a new campaign id:
    - `./benchmark-toolkit/scripts/provchain-neo4j-tigergraph-campaign.sh profile --id 20260502_trace_supply1000_provchain-neo4j-tigergraph_n3_buildonce`
  - clean profile result:
    - campaign: `20260502_trace_supply1000_provchain-neo4j-tigergraph_n3_buildonce`
    - status: `passed`, `3/3`
    - completed at: `2026-05-02T11:30:19Z`
    - curated export: `docs/benchmarking/data/reference/trace_supply_chain_1000_provchain_neo4j_tigergraph_n3_20260502/`
    - TigerGraph path label: `translated-property-graph-model`
    - TigerGraph means:
      - Simple Product Lookup: `3.444 ms`
      - Multi-hop Traceability (10 hops): `4.165 ms`
      - Aggregation by Producer: `3.338 ms`
    - interpretation: profile passed and justifies a full `n30` campaign if TigerGraph should be included as optional translated property-graph evidence
  - next command in a Docker-enabled shell:
    - `./benchmark-toolkit/scripts/provchain-neo4j-tigergraph-campaign.sh full --id 20260502_trace_supply1000_provchain-neo4j-tigergraph_n30`
  - full result:
    - campaign: `20260502_trace_supply1000_provchain-neo4j-tigergraph_n30`
    - status: `passed`, `30/30`
    - completed at: `2026-05-02T12:40:34Z`
    - curated export: `docs/benchmarking/data/reference/trace_supply_chain_1000_provchain_neo4j_tigergraph_n30_20260502/`
    - TigerGraph path label: `translated-property-graph-model`
    - TigerGraph means:
      - Simple Product Lookup: `3.222 ms`, p95 `3.740 ms`, p99 `5.311 ms`
      - Multi-hop Traceability (10 hops): `4.038 ms`, p95 `4.767 ms`, p99 `6.467 ms`
      - Aggregation by Producer: `3.276 ms`, p95 `3.686 ms`, p99 `5.557 ms`
    - interpretation: TigerGraph is now available as optional secondary translated property-graph trace-query comparator evidence; it remains excluded from RDF/SPARQL-native semantic claims

### P001 - Paper Update After Comparator Evidence

- Priority: `P1`
- Status: `Done`
- Goal: update paper หลังมี evidence เท่านั้น
- Tasks:
  - [x] update benchmark report bundle
  - [x] update paper table provenance map
  - [x] update manuscript benchmark table or appendix with GraphDB n30 evidence
  - [x] update manuscript benchmark table or appendix with TigerGraph n30 evidence
  - [x] update thesis/paper wording: GraphDB as RDF/SPARQL comparator; TigerGraph as optional translated property-graph comparator
  - [x] preserve caveats for all non-native paths
- Done when:
  - paper text references only passed curated evidence
  - no failed/smoke-only campaign is used as paper evidence
Current implementation evidence:
  - manuscript copies updated:
    - `docs/paper_submission/main.tex`
    - `docs/paper_submission/overleaf_upload/main.tex`
  - provenance map updated:
    - `docs/reviews/PAPER_TABLE_PROVENANCE_MAP_2026-04-14.md`
  - report bundle already includes GraphDB addendum and GraphDB rows:
    - `docs/benchmarking/PUBLICATION_BENCHMARK_REPORT_BUNDLE_2026-04-28.md`
  - TigerGraph full `n30` evidence is integrated as a translated property-graph comparator, not as RDF/SPARQL-native evidence

### H001 - Evidence Hygiene And Public Release Safety

- Priority: `P0`
- Status: `In progress - TigerGraph n30 evidence exported; public release pending after paper update`
- Goal: กันผล benchmark ปนกับ public/sensitive files และ output จำนวนมาก
- Tasks:
  - [x] keep raw runtime output out of git unless curated
  - [x] export only summary CSV/JSON/README/manifest/aggregate files
  - [ ] archive failed campaigns outside publication reference path
  - [x] update `.gitignore` if new generated runtime directories appear
  - [ ] run public release safety script only after all doc/paper updates are finalized
- Done when:
  - `git status --short` ไม่มี target/log/generated dump ที่ไม่ควร commit
Current implementation evidence:
  - `benchmark-toolkit/results/` remains ignored
  - `benchmark-toolkit/datasets/translated/` remains ignored
  - new TigerGraph generated artifacts are ignored via `benchmark-toolkit/tigergraph/generated/`
  - after the GraphDB manuscript integration commit, `git status --short --branch` showed no uncommitted tracked or untracked files
  - public release export/push is intentionally deferred until TigerGraph is either passed or documented as deferred

## Execution Order

1. C001
2. G001
3. G002
4. G003
5. G004
6. G005
7. G006
8. G007
9. T001
10. T002 only if T001 passes
11. T003 only if T002 passes
12. P001
13. H001

## Current Status Summary

- GraphDB: full `n30` evidence passed and is publication-facing as an RDF/SPARQL-native comparator
- TigerGraph: feasibility, translated-model live install, smoke/profile, and full `n30` campaign evidence passed
- Paper update: GraphDB rows are integrated as RDF/SPARQL-native evidence; TigerGraph rows are integrated as secondary translated property-graph evidence
- Existing benchmark evidence: remains valid and must not be rerun by default
