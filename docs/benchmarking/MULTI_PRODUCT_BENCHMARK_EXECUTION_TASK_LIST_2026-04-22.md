# Multi-Product Benchmark Execution Task List - 2026-04-22

## Purpose

เอกสารนี้แปลง methodology benchmark หลายผลิตภัณฑ์ให้เป็น task list เชิงปฏิบัติ โดยยึดตาม:

- [MULTI_PRODUCT_BENCHMARK_METHODOLOGY_PLAN_2026-04-22.md](./MULTI_PRODUCT_BENCHMARK_METHODOLOGY_PLAN_2026-04-22.md)

เป้าหมายคือให้ทีมลงมือทำ benchmark ทีละ family แบบเป็นระบบ โดยไม่ย้อนกลับไปใช้ workflow แบบแก้ปัญหาเฉพาะหน้าแล้วรัน Docker วนซ้ำโดยไม่มี local guard

## Phase 0. Shared Ground Rules

### Task 0.1

ตรึง benchmark families ใน code และ report schema ให้ explicit:

- `ledger`
- `trace_query`
- `semantic`
- `policy`
- `interchange`

### Task 0.2

เพิ่ม fairness labels ใน result schema:

- `native-comparable`
- `secondary-baseline`
- `externalized-semantic-pipeline`
- `indexed-query-stack`
- `not-comparable`

### Task 0.3

เพิ่ม environment manifest ต่อ run:

- CPU
- RAM
- storage
- OS
- Docker image tags
- dataset slice
- workload IDs
- run ID

### Task 0.4

บังคับ pre-run local gate:

- product-specific correctness tests
- benchmark query contract tests
- adapter unit tests

## Phase 1. Harden `ProvChain vs Neo4j`

เป้าหมาย:

- ทำให้ `Family B. Trace Query` เป็น artifact-backed baseline ที่นิ่งและทำซ้ำได้

### Task 1.1

รักษา local contract suite สำหรับ benchmark queries ให้เป็น gate ถาวร

สถานะ:

- เสร็จแล้ว

### Task 1.2

เพิ่ม trace workloads ให้ครบ:

- one-hop trace
- three-hop trace
- full provenance reconstruction
- aggregation
- result-cardinality scaling

### Task 1.3

ทำ result report ให้แสดง:

- `success_rate`
- `p50/p95/p99`
- workload IDs
- family label
- fairness label

### Task 1.4

แยก `ProvChain vs Neo4j` write-path report ออกเป็น `secondary-baseline` ชัดเจน

เหตุผล:

- Neo4j ไม่ใช่ permissioned ledger baseline

## Phase 2. Recover `Fluree`

เป้าหมาย:

- นำ `Fluree` กลับเข้ามาเป็น comparator ที่ถูก family

### Task 2.1

Status: `Done`

pin Fluree version ใน Docker และ docs ให้ชัด

Current pinned runtime:

- `fluree/server@sha256:e241fe44cabcfbfef4010fdc0d54301e5c91e3cbea8d6420e52ed795bdf0f15e`

### Task 2.2

Status: `Done`

ยืนยัน API contract ของ Fluree แบบ probe-driven

สิ่งที่ต้องล็อก:

- health endpoint
- create/open ledger endpoint
- transaction endpoint
- query endpoint

Verified runtime contract:

- health probe: `/index.html`
- ledger create: `POST /fluree/create`
- transaction: `POST /fluree/transact`
- query: `POST /fluree/query`

### Task 2.3

Status: `Done`

สร้าง local adapter contract tests ของ Fluree ก่อน Docker rerun

Validation:

- `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml fluree -- --nocapture`
- `benchmark-toolkit/scripts/probe-fluree-ledger.sh`

### Task 2.4

Status: `Done`

เปิด `ProvChain vs Fluree` ใน `Family B`

Evidence:

- campaign: `20260428_trace_supply1000_provchain-neo4j-fluree_n30`
- status: `passed`, epochs `30/30`
- curated export: [trace_supply1000_provchain_neo4j_fluree_n30_20260428](./data/trace_supply1000_provchain_neo4j_fluree_n30_20260428/)
- Simple Product Lookup mean: ProvChain `0.487 ms`, Fluree `9.722 ms`, Neo4j `10.304 ms`
- Multi-hop Traceability mean: ProvChain `0.479 ms`, Fluree `10.060 ms`, Neo4j `14.515 ms`
- Aggregation by Producer mean: ProvChain `0.566 ms`, Neo4j `28.477 ms`, Fluree `127.179 ms`

### Task 2.5

Status: `Done`

เปิด `ProvChain vs Fluree` ใน `Family C`

หมายเหตุ:

- ถ้า semantic pipeline ของ Fluree ต้องใช้ translation layer เพิ่ม ต้องนับเวลา translation ด้วย

Evidence:

- campaign: `20260428_semantic_supply1000_provchain-fluree_n30`
- status: `passed`, epochs `30/30`
- curated export: [semantic_supply1000_provchain_fluree_n30_20260428](./data/semantic_supply1000_provchain_fluree_n30_20260428/)
- ProvChain path: `native-rdf-path`, native semantic support `true`, explanation support `true`
- Fluree path: `external-semantic-pipeline`, native semantic support `false`, external stages `ttl-to-jsonld-translation, jsonld-ledger-insert`

## Phase 3. Implement `Fabric`

เป้าหมาย:

- ทำ `Fabric` ให้เป็น permissioned ledger baseline จริง ไม่ใช่แค่ health check

### Task 3.1

Status: `Done`

กำหนด benchmark-facing Fabric entrypoint:

- gateway service
- fixed chaincode
- reproducible test network topology

### Task 3.2

Status: `Done`

นิยาม logical record to chaincode mapping

### Task 3.3

Status: `Done`

สร้าง local contract tests สำหรับ:

- single write
- batch write
- commit confirmation
- optional policy-enforced operations

Validation:

- `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml fabric -- --nocapture`
- `ITERATIONS=1 FABRIC_BATCH_SIZE=10 ./benchmark-toolkit/scripts/run-fabric-contract-smoke.sh`

### Task 3.4

Status: `Done`

เปิด `ProvChain vs Fabric` ใน `Family A`

Runner:

- `benchmark-toolkit/scripts/run-fabric-ledger-campaign.sh`
- `benchmark-toolkit/scripts/provchain-fabric-campaign.sh`

Runtime assets:

- `benchmark-toolkit/docker-compose.fabric.yml`
- `benchmark-toolkit/fabric/chaincode/traceability/`
- `benchmark-toolkit/fabric/gateway/`
- `benchmark-toolkit/scripts/start-fabric-stack.sh`
- `benchmark-toolkit/scripts/stop-fabric-stack.sh`

Runtime requirement:

- shell ต้องมี Docker access
- `FABRIC_SAMPLES_DIR` ต้องชี้ไปที่ Fabric samples ที่มี `test-network/network.sh`
- `FABRIC_GATEWAY_URL` ต้องชี้ไปที่ real Fabric benchmark gateway
- `PROVCHAIN_URL` ต้องชี้ไปที่ ProvChain API ที่รันอยู่
- runner ไม่ start local simulator และไม่ถือ simulator เป็น evidence
- smoke validation passed against real Fabric gateway:
  - campaign: `smoke_ledger_supply1000_provchain-fabric_n1_20260424T164747Z`
  - `GET /health`, single write, batch write, and policy check all returned `200`
  - epoch `001/1` passed and produced campaign artifacts
- full n30 attempt `20260424_ledger_supply1000_provchain-fabric_n30` is excluded from evidence:
  - `passed_epochs: 0`, `failed_epochs: 30`
  - Fabric gateway was healthy
  - ProvChain API at `http://localhost:8080` was not reachable
  - local dataset path default was corrected for native runner execution
- full n30 attempt `20260424_ledger_supply1000_provchain-fabric_n30_fix1` must also be excluded:
  - by epoch `006/30`, epoch duration had drifted from minutes to much longer
  - cause: external ProvChain server reused persistent state across epochs
  - corrected runner mode is managed ProvChain per epoch with `skip_load=true`
- managed full n30 campaign passed:
  - campaign: `20260425_ledger_supply1000_provchain-fabric_managed_n30_fix3`
  - status: `passed`, epochs `30/30`
  - curated export: [ledger_supply1000_provchain_fabric_managed_n30_20260425](./data/ledger_supply1000_provchain_fabric_managed_n30_20260425/)
  - Fabric single record submit mean: `12.809 ms`
  - Fabric single record commit mean: `2022.808 ms`
  - Fabric batch submit mean: `139.526 ms`
  - Fabric batch commit mean: `2165.363 ms`
  - ProvChain single-threaded write workload mean: `29352.300 ms`

### Task 3.5

Status: `Done`

เปิด `Fabric` ใน `Family D`

Validation:

- local simulator campaign smoke passed:
  - campaign: `smoke_policy_supply1000_fabric_local_contract_n1`
  - status: `passed`, epochs `1/1`
- real Fabric smoke passed:
  - campaign: `smoke_policy_supply1000_fabric_real_n1_20260425`
  - status: `passed`, epochs `1/1`
- real Fabric full n30 campaign passed:
  - campaign: `20260425_policy_supply1000_fabric_n30`
  - status: `passed`, epochs `30/30`
  - curated export: [policy_supply1000_fabric_n30_20260425](./data/reference/policy_supply1000_fabric_n30_20260425/)
  - authorized-read mean: `5.523 ms`
  - auditor-read mean: `5.291 ms`
  - unauthorized-read mean: `5.301 ms`
  - success rate: `100.00%` for all three scenarios

Boundary:

- this completes real Fabric policy evidence
- it does not yet complete comparative `ProvChain vs Fabric` policy evidence

### Task 3.6

Status: `Pending query/index-layer decision`

ถ้ามี query layer:

- ต้องระบุชัดว่าเป็น `indexed-query-stack`
- ห้ามรายงานเป็น native trace-query path

## Phase 4. Implement `Geth`

เป้าหมาย:

- ใช้ `Geth` เป็น public-chain baseline ที่ honest

### Task 4.1

Status: `Done`

กำหนด smart-contract workload ขั้นต่ำ

Evidence:

- [GETH_BENCHMARK_WORKLOAD_CONTRACT_2026-04-25.md](./GETH_BENCHMARK_WORKLOAD_CONTRACT_2026-04-25.md)

### Task 4.2

Status: `Done`

แยกเมทริก:

- submit latency
- mined/confirmed latency
- gas / fee metadata

### Task 4.3

Status: `Done`

สร้าง local contract tests สำหรับ:

- contract deployment
- single transaction submit
- batch transaction submit
- confirmation timing

Validation:

- `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml geth -- --nocapture`
- `cargo check --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml`

Implemented local contract coverage:

- JSON-RPC client version health check
- contract address code validation via `eth_getCode`
- raw transaction submit via `eth_sendRawTransaction`
- receipt polling via `eth_getTransactionReceipt`
- confirmation latency capture
- failed receipt status capture
- gas metadata decode

### Task 4.4

Status: `Done`

เปิด `ProvChain vs Geth` ใน `Family A`

ข้อห้าม:

- ห้ามใช้ `Geth` เป็น permissioned enterprise baseline

Evidence:

- campaign: `20260428_ledger_supply1000_provchain-geth_n30_fix1`
- status: `passed`, epochs `30/30`
- curated export: [ledger_supply1000_provchain_geth_n30_20260428](./data/ledger_supply1000_provchain_geth_n30_20260428/)
- Geth submit mean: `2.208 ms`
- Geth confirmation mean: `256.571 ms`
- Geth gas mean: `46248.600 gas`
- ProvChain single-threaded write mean: `29361.670 ms`
- boundary: Geth rows use `public-chain-baseline` and `public-chain-smart-contract`

## Phase 5. Semantic Family Hardening

เป้าหมาย:

- ทำ `Family C` ให้เป็น benchmark family ที่ป้องกันการอ้างเกินจริง

### Task 5.1

Status: `Done`

ตรึง canonical semantic workloads:

- JSON-LD ingest
- RDF ingest
- SHACL validation
- validation failure explanation
- end-to-end semantic admission

### Task 5.2

Status: `Done`

บังคับให้ทุก external semantic pipeline ถูกวัดครบทั้ง pipeline

### Task 5.3

Status: `Done`

เพิ่ม report fields:

- native semantic support
- externalized semantic stages
- explanation support

Validation:

- semantic smoke `smoke_semantic_supply1000_provchain-fluree_n1_20260428_fix1` passed
- semantic full campaign `20260428_semantic_supply1000_provchain-fluree_n30` passed `30/30`
- aggregate summary includes `Semantic Capability Notes`

## Phase 6. Policy Family Hardening

เป้าหมาย:

- ทำ `Family D` สำหรับ `ProvChain` และ `Fabric` ให้เป็น comparative benchmark ที่ใช้ได้จริง

### Task 6.1

Status: `Done`

กำหนด policy scenarios กลาง:

- authorized read
- unauthorized read rejection
- authorized write
- rejected write

Evidence:

- campaign: `20260428_policy_supply1000_fabric_pack_n30`
- status: `passed`, epochs `30/30`
- curated export: [policy_supply1000_fabric_pack_n30_20260428](./data/reference/policy_supply1000_fabric_pack_n30_20260428/)
- Fabric scenario pack covers `authorized-read`, `auditor-read`, `unauthorized-read`, `authorized-write`, and `rejected-write`
- Superseded for paper tables by comparative campaign `20260429_policy_supply1000_provchain-fabric_e2e_n30`

### Task 6.2

Status: `Done`

สร้าง expected outcomes กลาง:

- allow
- reject
- action metadata
- reject explanation is not claimed for the current Fabric-only pack

Validation:

- allowed outcomes are measured as successful authorization checks
- rejected outcomes are measured as successful policy rejections, not benchmark failures
- each scenario records the policy action in metadata
- the current Fabric gateway does not emit explanation payloads, so this evidence is not a policy-explanation benchmark

### Task 6.3

Status: `Done`

เพิ่ม metrics:

- policy overhead
- rejection latency
- throughput under policy remains derivable from latency samples rather than a standalone row in this export
- authorized write latency
- rejected write latency

Evidence:

- authorized-read mean: `5.646 ms`
- auditor-read mean: `5.282 ms`
- unauthorized-read mean: `5.241 ms`
- authorized-write mean: `5.200 ms`
- rejected-write mean: `5.195 ms`

Boundary:

- this completes the shared Fabric policy workload pack
- comparative ProvChain policy evidence still requires a matching ProvChain policy adapter/workload path

## Phase 7. Final Publication Bundle

เป้าหมาย:

- รวมหลาย family เข้าใน final benchmark package โดยไม่เสียความเป็นธรรม

### Task 7.1

Status: `Done`

สร้าง report template ที่แยกผลตาม family

Evidence:

- [PUBLICATION_BENCHMARK_REPORT_BUNDLE_2026-04-28.md](./PUBLICATION_BENCHMARK_REPORT_BUNDLE_2026-04-28.md)
- [generate-publication-benchmark-report.py](/home/cit/provchain-org/benchmark-toolkit/scripts/generate-publication-benchmark-report.py)

### Task 7.2

Status: `Done`

สร้าง table template แยก:

- trace-query winners
- ledger winners
- semantic cost comparisons
- policy overhead comparisons

Evidence:

- report sections are grouped by `governance-policy`, `ledger-write`, `semantic`, and `trace-query`
- semantic section includes native/externalized semantic capability fields
- policy section includes Fabric read/write allow/reject workload-pack rows

### Task 7.3

Status: `Done`

ห้ามสร้าง single global winner table

Validation:

- report header states no single global winner table is generated
- generated output contains only family-scoped tables

### Task 7.4

Status: `Done`

เพิ่ม section `fairness and limitations` เป็นมาตรฐานในทุกผลลัพธ์

Evidence:

- report bundle includes `Fairness And Limitations`
- evidence-source table lists campaign status, family, dataset, workload, and products for each source export

## Admission Criteria Before Any New Product Comparison

ก่อนเพิ่ม product ใหม่เข้า benchmark family ใด ต้องผ่านครบ:

1. version pinning
2. health-check contract ชัดเจน
3. local adapter contract tests
4. target-specific translation documentation
5. fairness label assignment

## Recommended Near-Term Order

1. `B001`-`B020` เสร็จครบใน benchmark round นี้
2. งานใหม่หลังจากนี้ต้องเปิดเป็น backlog รอบใหม่ ไม่ใช่เพิ่มทางเลือกในรอบเดิม

## Definition of Done

จะถือว่า multi-product benchmark framework พร้อมระดับใช้งานจริงเมื่อ:

- ทุก product อยู่ใน benchmark family ที่เหมาะกับมัน
- ทุก comparative claim มี fairness label
- ทุก Docker benchmark มี local gate ก่อนรัน
- ทุก report แยกผลตาม family
- ไม่มี global winner table ที่หลอกตา
