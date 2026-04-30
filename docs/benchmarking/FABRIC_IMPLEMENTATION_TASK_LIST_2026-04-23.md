# Fabric Implementation Task List - 2026-04-23

## Purpose

เอกสารนี้แตก phase `Fabric` ของ benchmark program ให้เป็นรายการงานระดับ implementation โดยอิงจากสถานะจริงของ repo ปัจจุบัน

จุดตั้งต้นที่ยืนยันได้ตอนนี้:

- [fabric.rs](/home/cit/provchain-org/benchmark-toolkit/research-benchmarks/src/adapters/fabric.rs) มีเพียง `health_check()`
- ยังไม่มี write path, commit/finality path, policy path, หรือ local contract tests สำหรับ `Fabric`
- ดังนั้น `Fabric` ยังไม่ใช่ benchmark comparator ที่ใช้งานได้จริง

## Current State

### What exists

- `FabricAdapter` scaffold
- config fields:
  - `gateway_url`
  - `channel`
  - `chaincode`
- gateway health check

### What is missing

- benchmark-facing topology definition
- canonical record mapping to chaincode input
- write submission path
- commit/finality measurement path
- policy / authorization scenarios
- local adapter tests
- Docker ledger stack for `Fabric`

### F1 topology contract

เอกสาร topology contract ถูกเพิ่มแล้ว:

- [FABRIC_BENCHMARK_TOPOLOGY_CONTRACT_2026-04-24.md](./FABRIC_BENCHMARK_TOPOLOGY_CONTRACT_2026-04-24.md)

สถานะ:

- `F1` เสร็จในระดับ architecture contract
- ขั้นถัดไปคือ `F2` canonical record mapping

### F2 canonical record mapping

เอกสาร mapping contract ถูกเพิ่มแล้ว:

- [FABRIC_CANONICAL_RECORD_MAPPING_2026-04-24.md](./FABRIC_CANONICAL_RECORD_MAPPING_2026-04-24.md)

สถานะ:

- `F2` เสร็จในระดับ mapping contract
- ขั้นถัดไปคือ `F3/F5` adapter method และ local contract tests

## Implementation Sequence

### Phase F1 - Topology Contract

เป้าหมาย:

- ตรึง topology ขั้นต่ำของ `Fabric` ที่ benchmark จะถือเป็น reference environment

สถานะ:

- เสร็จใน [FABRIC_BENCHMARK_TOPOLOGY_CONTRACT_2026-04-24.md](./FABRIC_BENCHMARK_TOPOLOGY_CONTRACT_2026-04-24.md)

งาน:

1. กำหนด test network shape
   - จำนวน org
   - จำนวน peer
   - จำนวน orderer
   - ใช้ channel เดียวหรือหลาย channel
2. ตรึง gateway entrypoint
   - benchmark runner จะคุยกับอะไร
   - REST gateway หรือ custom shim
3. ตรึง chaincode scope
   - chaincode จะรับ logical record แบบไหน
   - จะ encode record เป็น JSON หรือ field-based struct
4. ตรึง finality rule
   - วัดเวลาถึง submit สำเร็จ
   - หรือวัดถึง commit event

ผลลัพธ์ที่ต้องได้:

- architecture note สำหรับ `Fabric` benchmark topology
- ค่าเริ่มต้นของ `gateway_url`, `channel`, `chaincode`

Definition of Done:

- ตัดสินใจ topology ชัดเจนหนึ่งแบบ
- ไม่เหลือ ambiguity ว่า benchmark จะคุยกับ Fabric อย่างไร

### Phase F2 - Canonical Record Mapping

เป้าหมาย:

- แปลง canonical benchmark records ไปเป็น input ที่ chaincode ใช้งานได้จริง

สถานะ:

- เสร็จใน [FABRIC_CANONICAL_RECORD_MAPPING_2026-04-24.md](./FABRIC_CANONICAL_RECORD_MAPPING_2026-04-24.md)

งาน:

1. นิยาม canonical ledger record schema
   - id
   - entity type
   - event type
   - timestamps
   - producer / processor / location references
   - visibility / policy metadata
2. นิยาม mapping จาก canonical schema ไปเป็น Fabric payload
3. ตัดสินใจ field ที่อยู่ on-chain และ field ที่ถือเป็น externalized semantics
4. สร้าง fixtures ของ record ตัวอย่าง

ผลลัพธ์ที่ต้องได้:

- mapping spec
- fixture files

Definition of Done:

- มีตัวอย่าง input/output ที่ benchmark adapter ใช้ได้จริง

### Phase F3 - Adapter Write Path

เป้าหมาย:

- ให้ `FabricAdapter` เขียนข้อมูลได้จริงใน benchmark family `ledger`

สถานะ:

- เสร็จสำหรับ benchmark-facing gateway contract path ใน [fabric.rs](/home/cit/provchain-org/benchmark-toolkit/research-benchmarks/src/adapters/fabric.rs)
- มี contract methods สำหรับ `submit_record`, `submit_batch`, และ structured response parsing
- ผูกเข้า benchmark runner แล้วผ่าน `--write`
- runner แยกผล `submit_latency_ms` และ `commit_latency_ms` เป็น result rows คนละ metric
- ยังไม่ถือเป็น comparative evidence จนกว่าจะมี `fabric-gateway` runtime จริง แทน simulator

งาน:

1. เพิ่ม submit API ใน `FabricAdapter`
2. เพิ่ม batch submit path
3. เพิ่ม structured response parsing
4. แยก metric:
   - submit latency
   - commit/finality latency
   - success/failure

ผลลัพธ์ที่ต้องได้:

- `FabricAdapter` ไม่ใช่แค่ health check
- มี write-path contract ชัดเจน

Definition of Done:

- benchmark runner เรียก write path ของ `Fabric` ผ่าน gateway contract ได้จริง
- comparative evidence ยังรอ `B013` real runtime/gateway stack

### Phase F4 - Adapter Finality Path

เป้าหมาย:

- วัด commit/finality สำหรับ `Fabric` อย่าง explicit

งาน:

1. เลือก signal ของ finality
   - commit event
   - block commit ack
   - gateway confirmation
2. implement measurement path
3. แยก metric นี้ออกจาก submit latency

ผลลัพธ์ที่ต้องได้:

- finality metric ที่ป้องกันการเปรียบเทียบผิดกับ `Neo4j`

Definition of Done:

- result schema เก็บ finality ของ `Fabric` ได้จริง

### Phase F5 - Local Contract Tests

เป้าหมาย:

- ปิด workflow แบบต้องเดา root cause หลัง `docker compose`

สถานะ:

- เริ่มแล้วใน [fabric.rs](/home/cit/provchain-org/benchmark-toolkit/research-benchmarks/src/adapters/fabric.rs)
- มี mock gateway tests สำหรับ health, submit success, submit failure, batch submit, และ policy check
- มี gateway probe script ที่ [probe-fabric-gateway.sh](/home/cit/provchain-org/benchmark-toolkit/scripts/probe-fabric-gateway.sh)
- Fabric contract tests ถูกเพิ่มเข้า preflight gate แล้วที่ [preflight-trace-benchmark.sh](/home/cit/provchain-org/benchmark-toolkit/scripts/preflight-trace-benchmark.sh)
- มี local-only contract simulator ที่ [fabric-gateway-contract-sim.py](/home/cit/provchain-org/benchmark-toolkit/scripts/fabric-gateway-contract-sim.py)
- มี local smoke wrapper ที่ [run-fabric-contract-smoke.sh](/home/cit/provchain-org/benchmark-toolkit/scripts/run-fabric-contract-smoke.sh)
- validation ล่าสุด:
  - `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml fabric -- --nocapture` ผ่าน `5` tests
  - `ITERATIONS=1 FABRIC_BATCH_SIZE=10 ./benchmark-toolkit/scripts/run-fabric-contract-smoke.sh` ผ่านและสร้าง `7` result rows
  - smoke artifact ล่าสุดอยู่ที่ `benchmark-toolkit/results/fabric-contract/20260424T102338Z/`

งาน:

1. เพิ่ม unit tests สำหรับ request/response handling
2. เพิ่ม local contract tests สำหรับ:
   - submit success
   - submit failure
   - batch submit
   - commit acknowledgment
3. เพิ่ม regression tests สำหรับ policy-related rejections

ผลลัพธ์ที่ต้องได้:

- local gate ของ `Fabric` ก่อนเปิด Docker stack

Definition of Done:

- `Fabric` local adapter suite ผ่านครบ
- local simulator smoke ผ่านครบ

### Phase F6 - Ledger Family Enablement

เป้าหมาย:

- เปิด `ProvChain vs Fabric` ใน family `ledger`

สถานะ:

- campaign runner พร้อมแล้ว:
  - [run-fabric-ledger-campaign.sh](/home/cit/provchain-org/benchmark-toolkit/scripts/run-fabric-ledger-campaign.sh)
  - [provchain-fabric-campaign.sh](/home/cit/provchain-org/benchmark-toolkit/scripts/provchain-fabric-campaign.sh)
- runtime assets พร้อมแล้ว:
  - [docker-compose.fabric.yml](/home/cit/provchain-org/benchmark-toolkit/docker-compose.fabric.yml)
  - [fabric/chaincode/traceability](/home/cit/provchain-org/benchmark-toolkit/fabric/chaincode/traceability/chaincode.go)
  - [fabric/gateway](/home/cit/provchain-org/benchmark-toolkit/fabric/gateway/main.go)
  - [start-fabric-stack.sh](/home/cit/provchain-org/benchmark-toolkit/scripts/start-fabric-stack.sh)
  - [stop-fabric-stack.sh](/home/cit/provchain-org/benchmark-toolkit/scripts/stop-fabric-stack.sh)
- runner ใช้ `FABRIC_GATEWAY_URL` กับ `PROVCHAIN_URL` ที่รันอยู่แล้ว
- runner ไม่ start local simulator และไม่ถือ simulator เป็น evidence
- ยังต้อง execute Docker stack จริงจาก shell ที่มี Docker access และ Fabric samples

งาน:

1. สร้าง compose/service topology สำหรับ `Fabric` เสร็จแล้วในระดับ repo assets
2. ต่อ benchmark runner เข้ากับ `FabricAdapter` เสร็จแล้ว
3. เพิ่ม result labeling เสร็จแล้ว:
   - family
   - fairness label
   - capability path
4. รัน artifact-backed benchmark ครั้งแรกกับ real Fabric runtime

Definition of Done:

- มี run artifact จริงของ `ProvChain vs Fabric` ใน family `ledger`

### Phase F7 - Policy Family Enablement

เป้าหมาย:

- เปิด family `policy` สำหรับ `ProvChain vs Fabric`

งาน:

1. นิยาม authorized workload
2. นิยาม unauthorized workload
3. นิยาม policy-check overhead metric
4. เพิ่ม policy benchmark scenarios ใน runner

Definition of Done:

- มี policy benchmark artifact จริง

## Minimal First Sprint

ชุด sprint แรกปิดแล้ว:

1. `F1` Topology Contract
2. `F2` Canonical Record Mapping
3. `F5` Local Contract Tests skeleton

ผลลัพธ์:

- contract docs พร้อม
- adapter write/policy path พร้อมสำหรับ gateway contract
- local tests/smoke ผ่าน
- ขั้นถัดไปคือ `F6/B013` real Fabric ledger stack
- และจะกลับไปปัญหาเดิมแบบ `Fluree` คือ compose มาก่อน contract

## File-Level Ownership Suggestion

### Benchmark toolkit

- [fabric.rs](/home/cit/provchain-org/benchmark-toolkit/research-benchmarks/src/adapters/fabric.rs)
- [main.rs](/home/cit/provchain-org/benchmark-toolkit/research-benchmarks/src/main.rs)
- `benchmark-toolkit/docker-compose.ledger.yml` ในอนาคต

### Documentation

- `docs/benchmarking/*Fabric*`

### Tests

- `benchmark-toolkit/research-benchmarks/tests/*fabric*`

## Recommended Next Step

ขั้นถัดไปที่ถูกต้องที่สุดคือ:

1. เขียน architecture note สำหรับ `Fabric` benchmark topology
2. ตามด้วย mapping spec
3. แล้วค่อยลงมือ adapter implementation

ถ้าข้ามสองข้อแรกไป จะได้ implementation ที่ยังไม่ตรึง contract และมีความเสี่ยงต้องรื้ออีก
