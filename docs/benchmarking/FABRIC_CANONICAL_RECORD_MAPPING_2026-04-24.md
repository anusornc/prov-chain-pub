# Fabric Canonical Record Mapping - 2026-04-24

## Purpose

เอกสารนี้นิยาม mapping ระหว่าง canonical benchmark record กับ payload ที่ `Fabric` gateway และ chaincode ต้องรับ

เอกสารนี้เป็น `F2` ต่อจาก:

- [FABRIC_BENCHMARK_TOPOLOGY_CONTRACT_2026-04-24.md](./FABRIC_BENCHMARK_TOPOLOGY_CONTRACT_2026-04-24.md)

## Scope

mapping นี้ใช้สำหรับ benchmark family:

- `Ledger / Write Path`
- `Governance / Policy`

mapping นี้ยังไม่ใช่ semantic admission benchmark และยังไม่กล่าวอ้างว่า `Fabric` มี RDF หรือ SHACL native capability

## Canonical Logical Record

canonical record คือ logical shape กลางก่อนแปลงเข้า product-specific adapter

field ขั้นต่ำ:

| Field | Type | Required | Meaning |
|---|---|---:|---|
| `record_id` | string | yes | benchmark-level unique record ID |
| `entity_id` | string | yes | traceable entity เช่น batch หรือ shipment |
| `entity_type` | string | yes | logical entity class เช่น `ProductBatch` |
| `event_type` | string | yes | logical event เช่น `Produced`, `Processed`, `Transferred` |
| `timestamp` | string | yes | RFC3339 timestamp |
| `actor_id` | string | yes | producer, processor, carrier, auditor, หรือ organization actor |
| `location_id` | string | no | site หรือ location reference |
| `previous_record_ids` | array<string> | no | provenance links ย้อนกลับ |
| `attributes` | object | no | non-indexed benchmark attributes |
| `visibility` | string | yes | `public`, `restricted`, หรือ `private` |
| `owner_org` | string | yes | Fabric MSP owner เช่น `Org1MSP` |

## Fabric Gateway Payload

gateway payload สำหรับ single submit:

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
    "previous_record_ids": [],
    "attributes": {
      "quantity": 100
    }
  },
  "policy": {
    "visibility": "public",
    "owner_org": "Org1MSP"
  }
}
```

## Chaincode Storage Shape

chaincode ควรเก็บ object เดียวต่อ `record_id`

storage key:

```text
trace-record:<record_id>
```

storage value:

```json
{
  "record_id": "record-001",
  "entity_id": "BATCH001",
  "entity_type": "ProductBatch",
  "event_type": "Produced",
  "timestamp": "2026-04-24T00:00:00Z",
  "actor_id": "producer-001",
  "location_id": "site-001",
  "previous_record_ids": [],
  "attributes": {
    "quantity": 100
  },
  "visibility": "public",
  "owner_org": "Org1MSP"
}
```

## Chaincode Function Mapping

| Gateway endpoint | Chaincode function | Input |
|---|---|---|
| `POST /ledger/records` | `PutTraceRecord(recordJson)` | gateway single payload |
| `POST /ledger/records/batch` | `PutTraceBatch(recordsJson)` | array of single payloads |
| `GET /ledger/records/{record_id}` | `GetTraceRecord(recordId)` | record ID |
| `POST /policy/check` | `CheckPolicy(recordId, actorOrg, action)` | policy check payload |

## Policy Mapping

phase แรกใช้ policy แบบง่าย:

| Visibility | Authorized rule |
|---|---|
| `public` | ทุก org อ่านได้ |
| `restricted` | owner org และ auditor org อ่านได้ |
| `private` | owner org อ่านได้เท่านั้น |

ค่า org เริ่มต้น:

- owner org: `Org1MSP`
- peer org อื่น: `Org2MSP`
- auditor org: `AuditorMSP`

## Metrics Generated From This Mapping

single submit:

- `submit_latency_ms`
- `commit_latency_ms`
- `success`
- `tx_id`
- `block_number`

batch submit:

- `submit_latency_ms`
- `commit_latency_ms`
- `submitted`
- `committed`
- `success_rate`

policy check:

- `policy_latency_ms`
- `authorized`
- `success`

## Fairness Label

สำหรับ `Fabric` ใน family `Ledger / Write Path`:

- fairness label: `native-comparable`
- capability path: `native`

สำหรับ `Fabric` ใน family `Governance / Policy`:

- fairness label: `native-comparable` เฉพาะ workload ที่ทั้ง `ProvChain` และ `Fabric` enforce policy จริง
- capability path: `native`

## Out of Scope

phase นี้ยังไม่รวม:

- RDF serialization
- SHACL validation
- ontology package validation
- CouchDB rich query
- private data collection benchmark
- semantic explanation generation

ถ้าต้อง benchmark semantic capability ของ `Fabric` ภายหลัง ต้องถือเป็น external semantic pipeline และนับเวลานั้นทั้งหมด

## Definition of Done for F2

ถือว่า `F2` เสร็จเมื่อ:

- canonical record shape ถูกนิยาม
- gateway payload ถูกนิยาม
- chaincode storage shape ถูกนิยาม
- chaincode function mapping ถูกนิยาม
- policy mapping ขั้นต่ำถูกนิยาม
- metric mapping ถูกนิยาม

สถานะปัจจุบัน:

- `F2` เสร็จในระดับ mapping contract
- ขั้นถัดไปคือ `F3/F5` adapter method และ local contract tests
