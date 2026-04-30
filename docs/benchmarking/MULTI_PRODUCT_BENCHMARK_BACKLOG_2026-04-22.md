# Multi-Product Benchmark Backlog - 2026-04-22

## Purpose

เอกสารนี้แปลง benchmark methodology และ execution task list ให้เป็น backlog แบบใช้งานจริง

เอกสารอ้างอิงหลัก:

- [MULTI_PRODUCT_BENCHMARK_METHODOLOGY_PLAN_2026-04-22.md](./MULTI_PRODUCT_BENCHMARK_METHODOLOGY_PLAN_2026-04-22.md)
- [MULTI_PRODUCT_BENCHMARK_EXECUTION_TASK_LIST_2026-04-22.md](./MULTI_PRODUCT_BENCHMARK_EXECUTION_TASK_LIST_2026-04-22.md)

หลักการจัดลำดับ:

- งานที่ทำให้ benchmark ปัจจุบันน่าเชื่อถือขึ้นก่อน
- งานที่ลด contract ambiguity ก่อน
- งานที่เพิ่ม comparator ใหม่ทีละตัว
- หลีกเลี่ยงการเปิดหลาย target พร้อมกันโดยไม่มี local gate

## Current Baseline

สถานะที่ยืนยันได้แล้ว:

- `ProvChain vs Neo4j`
- family `trace_query`
- มี local gate สำหรับ benchmark queries
- มี artifact-backed benchmark run จริง

ดังนั้น backlog หลังจากนี้ต้องไม่ทำลาย baseline นี้

## Priority Legend

- `P0` ต้องทำก่อนงาน benchmark รอบถัดไป
- `P1` งานลำดับถัดไปที่มีผลต่อ comparator ใหม่โดยตรง
- `P2` งานสำคัญแต่รอได้
- `P3` งานเพิ่มความสมบูรณ์เชิง publication

## Backlog

### Completed in current round

- `B001` เสร็จแล้ว
- `B002` เสร็จแล้ว
- `B003` เสร็จแล้ว
- `B004` เสร็จแล้ว
- `B005` เสร็จแล้วในระดับ orchestration rule และ documentation
- `B006` เสร็จแล้วในระดับ adapter contract documentation
- `B007` เสร็จแล้วในระดับ local adapter tests
- `B010` เสร็จแล้วในระดับ topology contract documentation
- `B011` เสร็จแล้วในระดับ canonical record mapping documentation
- `B012` เสร็จแล้วในระดับ local adapter tests และ local simulator smoke
- `B013` เสร็จแล้วด้วย real Fabric runtime/gateway ledger campaign `30/30`
- `B014` เสร็จแล้วด้วย real Fabric runtime/gateway policy campaign `30/30`
- `B015` เสร็จแล้วในระดับ Geth workload contract documentation
- `B016` เสร็จแล้วในระดับ Geth JSON-RPC adapter contract tests
- `B008` เสร็จแล้วด้วย real Docker trace campaign `ProvChain + Neo4j + Fluree` `30/30`
- `B009` เสร็จแล้วด้วย semantic admission campaign `ProvChain + Fluree` `30/30`
- `B018` เสร็จแล้วด้วย semantic capability fields ใน aggregate report
- `B017` เสร็จแล้วด้วย real local Geth development-chain campaign `ProvChain + Geth` `30/30`
- `B019` เสร็จแล้วด้วย Fabric policy workload pack campaign `30/30`
- `B020` เสร็จแล้วด้วย publication-ready report bundle

### Current next task

- ไม่มีงานค้างใน backlog รอบ `B001`-`B020`

### B001 - Freeze result schema by benchmark family

- Priority: `P0`
- Goal: ทำให้ผลลัพธ์ทุก run ระบุ family, fairness label, และ capability path อย่าง explicit
- Why:
  - ป้องกันการอ่านผลผิดว่าเป็น cross-family winner
  - ทำให้รายงานและ CSV ใช้งานเชิงวิชาการได้
- Deliverables:
  - result schema รองรับ `benchmark_family`
  - result schema รองรับ `fairness_label`
  - result schema รองรับ `capability_path`
- Done when:
  - `benchmark_results.json` มี field ใหม่ครบ
  - `summary.md` แยก family ได้

### B002 - Add environment manifest for every run

- Priority: `P0`
- Goal: ทำให้ทุก benchmark run มี manifest ที่บอก environment จริง
- Why:
  - ไม่มี manifest จะเทียบข้ามรอบไม่ได้อย่างน่าเชื่อถือ
- Deliverables:
  - `environment.json` ต่อ run
  - บันทึก CPU, RAM, storage, image tags, dataset slice, run ID
- Done when:
  - ทุกโฟลเดอร์ผลลัพธ์มี manifest

### B003 - Lock pre-run benchmark gate

- Priority: `P0`
- Goal: ห้าม benchmark run ผ่าน Docker ถ้า local benchmark-query contract suite ยังไม่ผ่าน
- Why:
  - ปิดปัญหา workflow แบบรัน compose แล้วค่อยเดา root cause
- Deliverables:
  - script หรือ documented command set สำหรับ pre-run gate
  - runner docs ระบุชัดว่า local gate ต้องผ่านก่อน
- Done when:
  - มีคำสั่งเดียวหรือชุดคำสั่งมาตรฐานก่อน benchmark

### B004 - Normalize trace-query report semantics

- Priority: `P0`
- Goal: ทำให้รายงาน trace-query แสดง `success_rate`, `p50`, `p95`, `p99`, และ caveat อย่างถูกต้อง
- Why:
  - ลดความเสี่ยงจากการสรุป winner ผิดประเภท
- Deliverables:
  - summary format ใหม่
  - markdown summary ที่แยก by scenario และ by family
- Done when:
  - report ไม่สามารถประกาศ winner จาก failed scenario ได้

### B005 - Fluree version pinning

- Priority: `P1`
- Goal: เลิกใช้ `latest` และตรึง version ของ `Fluree`
- Why:
  - ต้นตอปัญหาเดิมคือ API contract ไม่ชัด
- Deliverables:
  - image tag ที่ pin แล้ว
  - note อธิบายว่าใช้ API generation ใด
- Done when:
  - compose และ docs ใช้ version เดียวกัน

### B006 - Fluree API contract verification

- Priority: `P1`
- Goal: ยืนยัน endpoint และ request/response contract ของ `Fluree` แบบมีหลักฐาน
- Why:
  - ถ้าไม่ปิด ambiguity นี้ จะกลับไปวนกับ `404` อีก
- Deliverables:
  - verified endpoints
  - documented request examples
  - documented health criteria
- Done when:
  - มี probe log หรือ contract tests ยืนยัน endpoint ที่ใช้จริง

### B007 - Fluree local adapter contract tests

- Priority: `P1`
- Goal: เพิ่ม local tests สำหรับ load/query path ของ `Fluree`
- Why:
  - ต้องมี gate ก่อนเปิด comparator กลับเข้า Docker trace stack
- Deliverables:
  - adapter contract tests
  - load/query smoke tests
- Done when:
  - local tests ผ่านก่อน compose rerun

### B008 - Re-enable Fluree in trace family

- Priority: `P1`
- Status: `Done`
- Goal: เปิด `ProvChain vs Neo4j vs Fluree` ใน family `trace_query`
- Dependencies:
  - `B005`
  - `B006`
  - `B007`
- Done when:
  - trace stack รันจบ
  - query scenarios ผ่านทุกระบบที่อยู่ใน family นี้
  - ใช้ `FLUREE_IMAGE` แบบ explicit pin
  - ใช้ `--profile fluree` และ `BENCHMARK_SKIP_FLUREE=false`
- Evidence:
  - campaign: `20260428_trace_supply1000_provchain-neo4j-fluree_n30`
  - status: `passed`, epochs `30/30`
  - curated export: [trace_supply1000_provchain_neo4j_fluree_n30_20260428](./data/trace_supply1000_provchain_neo4j_fluree_n30_20260428/)
  - Fluree image: `fluree/server@sha256:e241fe44cabcfbfef4010fdc0d54301e5c91e3cbea8d6420e52ed795bdf0f15e`
  - Simple Product Lookup mean: ProvChain `0.487 ms`, Fluree `9.722 ms`, Neo4j `10.304 ms`
  - Multi-hop Traceability mean: ProvChain `0.479 ms`, Fluree `10.060 ms`, Neo4j `14.515 ms`
  - Aggregation by Producer mean: ProvChain `0.566 ms`, Neo4j `28.477 ms`, Fluree `127.179 ms`

### B009 - Re-enable Fluree in semantic family

- Priority: `P1`
- Status: `Done`
- Goal: เปิด `ProvChain vs Fluree` ใน family `semantic`
- Dependencies:
  - `B005`
  - `B006`
  - `B007`
- Done when:
  - semantic admission workloads มี artifact-backed results
- Evidence:
  - campaign: `20260428_semantic_supply1000_provchain-fluree_n30`
  - status: `passed`, epochs `30/30`
  - curated export: [semantic_supply1000_provchain_fluree_n30_20260428](./data/semantic_supply1000_provchain_fluree_n30_20260428/)
  - ProvChain native RDF+SHACL admission mean: `12121.933 ms`
  - Fluree externalized JSON-LD admission mean: `514.667 ms`
  - semantic capability report fields distinguish native support, external stages, and explanation support

### B010 - Fabric benchmark-facing topology definition

- Priority: `P1`
- Status: `Done`
- Goal: ตัดสินใจ topology ขั้นต่ำของ `Fabric` สำหรับ benchmark
- Why:
  - ตอนนี้ยังมีแค่ health-check scaffold
- Deliverables:
  - gateway entrypoint
  - fixed test network shape
  - benchmark chaincode scope
- Done when:
  - มี architecture note และ compose-facing contract ชัด

### B011 - Fabric logical record mapping

- Priority: `P1`
- Status: `Done`
- Goal: กำหนดว่าจะแปลง canonical logical records ไปเป็น chaincode input อย่างไร
- Dependencies:
  - `B010`
- Done when:
  - mapping spec ชัด
  - มี test fixtures รองรับ

### B012 - Fabric local contract tests

- Priority: `P1`
- Status: `Done`
- Goal: มี local tests สำหรับ write/commit/policy path ของ `Fabric`
- Dependencies:
  - `B010`
  - `B011`
- Done when:
  - single write, batch write, commit acknowledgment ใช้งานได้
  - local simulator smoke ผ่านและสร้าง artifact ได้
  - validation ล่าสุด:
    - `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml fabric -- --nocapture`
    - `ITERATIONS=1 FABRIC_BATCH_SIZE=10 ./benchmark-toolkit/scripts/run-fabric-contract-smoke.sh`

### B013 - Enable Fabric ledger family

- Priority: `P1`
- Status: `Done`
- Goal: เปิด `ProvChain vs Fabric` ใน family `ledger`
- Dependencies:
  - `B010`
  - `B011`
  - `B012`
- Runner:
  - [run-fabric-ledger-campaign.sh](/home/cit/provchain-org/benchmark-toolkit/scripts/run-fabric-ledger-campaign.sh)
  - [provchain-fabric-campaign.sh](/home/cit/provchain-org/benchmark-toolkit/scripts/provchain-fabric-campaign.sh)
- Runtime assets:
  - [docker-compose.fabric.yml](/home/cit/provchain-org/benchmark-toolkit/docker-compose.fabric.yml)
  - [fabric/chaincode/traceability](/home/cit/provchain-org/benchmark-toolkit/fabric/chaincode/traceability/chaincode.go)
  - [fabric/gateway](/home/cit/provchain-org/benchmark-toolkit/fabric/gateway/main.go)
  - [start-fabric-stack.sh](/home/cit/provchain-org/benchmark-toolkit/scripts/start-fabric-stack.sh)
  - [stop-fabric-stack.sh](/home/cit/provchain-org/benchmark-toolkit/scripts/stop-fabric-stack.sh)
- Done when:
  - ledger stack รันได้พร้อม artifact จริง
  - ใช้ real `fabric-gateway`/peer/orderer/chaincode ไม่ใช่ local simulator
  - full n30 campaign ผ่านและ export curated evidence แล้ว
- Evidence:
  - campaign: `20260425_ledger_supply1000_provchain-fabric_managed_n30_fix3`
  - status: `passed`, epochs `30/30`
  - curated export: [ledger_supply1000_provchain_fabric_managed_n30_20260425](./data/ledger_supply1000_provchain_fabric_managed_n30_20260425/)

### B014 - Enable Fabric policy family

- Priority: `P1`
- Status: `Done`
- Goal: เปิด `Fabric` ใน family `policy`
- Dependencies:
  - `B012`
  - `B013`
  - policy scenarios definition
- Done when:
  - authorized/unauthorized workloads มีผลจริง
- Evidence:
  - campaign: `20260425_policy_supply1000_fabric_n30`
  - status: `passed`, epochs `30/30`
  - curated export: [policy_supply1000_fabric_n30_20260425](./data/reference/policy_supply1000_fabric_n30_20260425/)
  - boundary: Fabric policy evidence only; ProvChain policy comparison remains future work

### B015 - Geth workload definition

- Priority: `P2`
- Status: `Done`
- Goal: นิยาม smart-contract workload ขั้นต่ำสำหรับ `Geth`
- Why:
  - ถ้าไม่มี workload ที่ชัด จะเปรียบเทียบกับ `ProvChain` ไม่ได้
- Deliverables:
  - contract scope
  - submit vs confirmation model
  - gas metadata fields
- Done when:
  - workload spec ชัดและ implementable
- Evidence:
  - [GETH_BENCHMARK_WORKLOAD_CONTRACT_2026-04-25.md](./GETH_BENCHMARK_WORKLOAD_CONTRACT_2026-04-25.md)

### B016 - Geth local contract tests

- Priority: `P2`
- Status: `Done`
- Goal: มี local tests สำหรับ deploy/submit/confirm ของ `Geth`
- Dependencies:
  - `B015`
- Done when:
  - local tests ผ่านครบตาม path ขั้นต่ำ
- Evidence:
  - `GethAdapter` supports client version, chain id, contract-code validation, raw transaction submit, receipt polling, confirmation latency, failed receipts, and gas metadata decoding
  - validation: `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml geth -- --nocapture`

### B017 - Enable Geth ledger family

- Priority: `P2`
- Status: `Done`
- Goal: เปิด `ProvChain vs Geth` ใน family `ledger`
- Dependencies:
  - `B015`
  - `B016`
- Done when:
  - report แยก `submit_latency` กับ `confirmation_latency` ชัดเจน
- Evidence:
  - campaign: `20260428_ledger_supply1000_provchain-geth_n30_fix1`
  - status: `passed`, epochs `30/30`
  - curated export: [ledger_supply1000_provchain_geth_n30_20260428](./data/ledger_supply1000_provchain_geth_n30_20260428/)
  - Geth `Single Record Submit` mean: `2.208 ms`
  - Geth `Single Record Confirmation` mean: `256.571 ms`
  - Geth `Single Record Gas Used` mean: `46248.600 gas`
  - ProvChain `Single-threaded Write (100 tx)` mean: `29361.670 ms`
  - boundary: Geth is `public-chain-baseline` / `public-chain-smart-contract`, not a permissioned enterprise ledger baseline

### B018 - Semantic-family result enrichment

- Priority: `P2`
- Status: `Done`
- Goal: เพิ่มข้อมูลว่า semantic path ไหนเป็น native และ path ไหนเป็น externalized
- Deliverables:
  - `native_semantic_support`
  - `external_semantic_stages`
  - `explanation_support`
- Done when:
  - semantic reports อ่านแล้วไม่ทำให้เข้าใจผิดว่า capability เท่ากัน
- Evidence:
  - `campaign_aggregate_summary.md` now includes `Path`, `Metric`, and `Unit`
  - semantic reports include `Semantic Capability Notes`
  - `campaign_results.csv/json` include `native_semantic_support`, `external_semantic_stages`, and `explanation_support`

### B019 - Policy-family workload pack

- Priority: `P2`
- Status: `Done`
- Goal: สร้าง workload ชุดกลางสำหรับ permission/policy family
- Deliverables:
  - authorized read
  - unauthorized read
  - authorized write
  - rejected write
- Done when:
  - scenario pack พร้อมสำหรับ ProvChain/Fabric
- Evidence:
  - campaign: `20260428_policy_supply1000_fabric_pack_n30`
  - status: `passed`, epochs `30/30`
  - curated export: [policy_supply1000_fabric_pack_n30_20260428](./data/reference/policy_supply1000_fabric_pack_n30_20260428/)
  - superseded for paper tables by comparative campaign `20260429_policy_supply1000_provchain-fabric_e2e_n30`
  - authorized-read mean: `5.646 ms`
  - auditor-read mean: `5.282 ms`
  - unauthorized-read mean: `5.241 ms`
  - authorized-write mean: `5.200 ms`
  - rejected-write mean: `5.195 ms`
  - boundary: current evidence is Fabric-only policy workload pack; ProvChain policy parity remains future comparative work

### B020 - Publication-ready report bundle

- Priority: `P3`
- Status: `Done`
- Goal: สร้าง report template ที่แยก by family และมี fairness/limitations เป็นมาตรฐาน
- Deliverables:
  - family-specific tables
  - family-specific figures
  - limitations section template
- Done when:
  - ไม่มี single global winner table
- Evidence:
  - generator: [generate-publication-benchmark-report.py](/home/cit/provchain-org/benchmark-toolkit/scripts/generate-publication-benchmark-report.py)
  - report bundle: [PUBLICATION_BENCHMARK_REPORT_BUNDLE_2026-04-28.md](./PUBLICATION_BENCHMARK_REPORT_BUNDLE_2026-04-28.md)
  - source evidence: trace Neo4j, trace Neo4j+Fluree, ledger Fabric, semantic Fluree, policy Fabric pack, and ledger Geth curated exports
  - report includes family-specific tables, evidence-source table, semantic capability fields, and fairness/limitations

## Recommended Next Order

ลำดับที่ควรทำต่อจริงจากสถานะวันนี้:

1. `B001`
2. `B002`
3. `B003`
4. `B004`
5. `B005`
6. `B006`
7. `B007`
8. `B008`
9. `B009`
10. `B010`
11. `B011`
12. `B012`
13. `B013`
14. `B014`
15. `B015`
16. `B016`
17. `B017`
18. `B019`
19. `B020`

เหตุผล:

- 4 งานแรกทำให้ benchmark framework ปัจจุบันแข็งแรงขึ้นทันที
- จากนั้นค่อยฟื้น `Fluree`
- แล้วค่อยทำ `Fabric` ledger และ policy ตามลำดับ
- `Geth` ควรตามมาหลังจากนั้น

## Recommended Immediate Execution Slice

ถ้าจะเริ่มลงมือรอบถัดไปทันทีโดยไม่เปิดหลายเรื่องพร้อมกัน:

- Sprint 1:
  - `B001`
  - `B002`
  - `B003`
  - `B004`
- Sprint 2:
  - `B005`
  - `B006`
  - `B007`
- Sprint 3:
  - `B008`
  - `B009`
- Sprint 4:
  - `B010`
  - `B011`
  - `B012`
  - `B013`
- Sprint 5:
  - `B014`

## Definition of Ready for Implementation

ก่อนเริ่มงานใดใน backlog นี้ ต้องมี:

- benchmark family ชัด
- compared products ชัด
- fairness label ที่คาดว่าจะใช้ชัด
- dataset slice ชัด
- success criteria ชัด

## Definition of Done for the Program

จะถือว่า multi-product benchmark program พร้อมระดับใช้งานจริงเมื่อ:

- trace family มี `ProvChain + Neo4j + Fluree`
- ledger family มี `ProvChain + Fabric + Geth` อย่างน้อย
- semantic family มี `ProvChain + Fluree`
- ทุก family มี report แยกชัด
- ทุก run มี environment manifest
- ทุก comparative claim มี fairness label
