# Fabric Benchmark Topology Contract - 2026-04-24

## Purpose

เอกสารนี้ตรึง topology ขั้นต่ำของ `Hyperledger Fabric` สำหรับ benchmark program ของ `ProvChain`

เป้าหมายคือทำให้ `Fabric` กลายเป็น comparator ที่ตรวจสอบได้จริงใน benchmark family:

- `Ledger / Write Path`
- `Governance / Policy`

เอกสารนี้ยังไม่เปิด `Fabric` benchmark ทันที แต่เป็น contract ที่ต้องใช้ก่อนทำ adapter, local tests, และ Docker stack

## Current Repository State

สถานะจริงตอนนี้:

- [fabric.rs](/home/cit/provchain-org/benchmark-toolkit/research-benchmarks/src/adapters/fabric.rs) มีเพียง `health_check()`
- ยังไม่มี write path
- ยังไม่มี commit/finality measurement
- ยังไม่มี local contract tests
- ยังไม่มี Docker ledger stack สำหรับ `Fabric`

ดังนั้น `Fabric` ยังไม่ใช่ comparator ที่พร้อมใช้งานใน benchmark evidence

## Benchmark Role

`Fabric` จะถูกใช้เป็น:

- permissioned ledger baseline
- policy/governance baseline

`Fabric` จะไม่ถูกใช้เป็น:

- native RDF baseline
- native semantic validation baseline
- trace-query baseline เว้นแต่มี query/index layer ที่ถูกนิยามและนับเป็นส่วนหนึ่งของ stack

## Reference Topology

topology ขั้นต่ำที่เลือกใช้:

- 2 organizations
- 1 peer ต่อ organization
- 1 ordering service
- 1 channel
- 1 benchmark chaincode
- 1 benchmark-facing gateway service

ค่าชื่อเริ่มต้น:

- channel: `provchain`
- chaincode: `traceability`
- gateway URL: `http://fabric-gateway:8800`
- host port สำหรับ gateway: `18800`

เหตุผล:

- 2 organizations เพียงพอสำหรับ policy และ endorsement scenarios
- 1 peer ต่อ organization ลดต้นทุน orchestration แต่ยังรักษา permissioned-network semantics
- 1 ordering service เพียงพอสำหรับ benchmark baseline ระยะแรก
- benchmark runner ไม่ควรคุยกับ peer/orderer โดยตรง แต่ควรคุยผ่าน gateway ที่มี contract ชัดเจน

## Gateway Contract

benchmark runner จะคุยกับ `Fabric` ผ่าน REST gateway เท่านั้น

### Health

```http
GET /health
```

response ขั้นต่ำ:

```json
{
  "status": "ok",
  "channel": "provchain",
  "chaincode": "traceability"
}
```

### Submit single record

```http
POST /ledger/records
Content-Type: application/json
```

request body:

```json
{
  "record_id": "record-001",
  "payload": {
    "entity_id": "BATCH001",
    "entity_type": "ProductBatch",
    "event_type": "Produced",
    "timestamp": "2026-04-24T00:00:00Z",
    "actor_id": "producer-001",
    "location_id": "site-001",
    "attributes": {}
  },
  "policy": {
    "visibility": "public",
    "owner_org": "Org1MSP"
  }
}
```

response body:

```json
{
  "success": true,
  "tx_id": "fabric-tx-id",
  "submit_latency_ms": 12.3,
  "commit_latency_ms": 45.6,
  "block_number": 7
}
```

### Submit batch

```http
POST /ledger/records/batch
Content-Type: application/json
```

request body:

```json
{
  "records": []
}
```

response body:

```json
{
  "success": true,
  "submitted": 100,
  "committed": 100,
  "submit_latency_ms": 120.0,
  "commit_latency_ms": 480.0
}
```

### Policy check

```http
POST /policy/check
Content-Type: application/json
```

request body:

```json
{
  "record_id": "record-001",
  "actor_org": "Org1MSP",
  "action": "read"
}
```

response body:

```json
{
  "authorized": true,
  "policy_latency_ms": 1.2
}
```

## Chaincode Scope

benchmark chaincode ต้องรองรับ function ขั้นต่ำ:

- `PutTraceRecord(recordJson)`
- `PutTraceBatch(recordsJson)`
- `GetTraceRecord(recordId)`
- `CheckPolicy(recordId, actorOrg, action)`

chaincode ไม่ต้องทำ semantic validation ใน phase แรก

ถ้าจะเพิ่ม semantic validation ภายหลัง ต้องถือเป็น external semantic pipeline และต้องนับเวลาใน benchmark family `Semantic / Standards Admission`

## Finality Rule

สำหรับ `Fabric` ต้องแยกสองค่า:

- `submit_latency_ms`: เวลาถึง gateway ส่ง transaction ได้สำเร็จ
- `commit_latency_ms`: เวลาถึง transaction ถูก commit และได้รับ commit event หรือ equivalent confirmation

ค่า `commit_latency_ms` คือ metric หลักสำหรับ ledger finality

ห้ามใช้ submit latency เพียงอย่างเดียวเป็นตัวแทน finality

## Benchmark Families

### Ledger / Write Path

ผู้เข้าร่วม:

- `ProvChain`
- `Fabric`

metric หลัก:

- `submit_latency_ms`
- `commit_latency_ms`
- `throughput_records_per_sec`
- `success_rate`
- `block_or_commit_height`

fairness label:

- `native-comparable`

### Governance / Policy

ผู้เข้าร่วม:

- `ProvChain`
- `Fabric`

metric หลัก:

- `authorized_read_latency_ms`
- `unauthorized_rejection_latency_ms`
- `policy_check_overhead_ms`
- `write_with_policy_latency_ms`
- `success_rate`

fairness label:

- `native-comparable` เฉพาะ policy workloads ที่ทั้งสองระบบ enforce จริง

## Docker Service Contract

future Docker ledger stack ควรใช้ service names:

- `fabric-orderer`
- `fabric-peer-org1`
- `fabric-peer-org2`
- `fabric-gateway`
- `ledger-benchmark-runner`

benchmark runner env ขั้นต่ำ:

```bash
FABRIC_GATEWAY_URL=http://fabric-gateway:8800
FABRIC_CHANNEL=provchain
FABRIC_CHAINCODE=traceability
BENCHMARK_SKIP_FABRIC=false
```

## Local Tests Required Before Docker

ต้องมี local contract tests ก่อนเปิด Docker benchmark:

- health success
- health failure
- single submit success
- single submit failure with response body
- batch submit success
- commit/finality response parsing
- authorized policy check
- unauthorized policy rejection

เมื่อมี gateway runtime จริงแล้ว ให้ probe contract ก่อน:

```bash
export FABRIC_GATEWAY_URL=http://localhost:18800
./benchmark-toolkit/scripts/probe-fabric-gateway.sh
```

ก่อนมี gateway จริง สามารถตรวจ probe กับ contract simulator ได้:

```bash
python3 benchmark-toolkit/scripts/fabric-gateway-contract-sim.py
```

simulator นี้ใช้ตรวจ REST contract เท่านั้น ห้ามใช้เป็น benchmark evidence

## Out of Scope for First Fabric Pass

ยังไม่ทำใน phase แรก:

- multi-orderer production topology
- multiple peers per org
- private data collection benchmark
- CouchDB rich-query benchmark
- semantic validation inside chaincode
- cross-chain bridge workload

เหตุผล:

- phase แรกต้องพิสูจน์ ledger write/finality และ policy path ก่อน
- งานที่ใหญ่กว่านี้ควรทำหลัง local contract และ first artifact-backed run ผ่านแล้ว

## Definition of Done for F1

ถือว่า `F1` เสร็จเมื่อ:

- topology หนึ่งแบบถูกเลือกชัดเจน
- gateway contract ถูกนิยาม
- chaincode scope ถูกนิยาม
- finality rule ถูกนิยาม
- benchmark family และ fairness label ถูกระบุ
- local test requirements ถูกระบุ

สถานะปัจจุบัน:

- `F1` เสร็จในระดับ architecture contract
- ขั้นถัดไปคือ `F2` canonical record mapping
