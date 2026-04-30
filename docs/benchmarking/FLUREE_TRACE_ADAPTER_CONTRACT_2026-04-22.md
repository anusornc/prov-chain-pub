# Fluree Trace Adapter Contract - 2026-04-22

## Purpose

เอกสารนี้ตรึง contract ขั้นต่ำสำหรับการนำ `Fluree` กลับเข้ามาใน trace benchmark อย่างเป็นระบบ

เป้าหมายคือหยุด workflow เดิมที่อาศัย:

- image แบบ `latest`
- การเดา endpoint family ไปเรื่อยๆ
- แล้วค่อยดูความผิดพลาดจาก `docker compose` ทีหลัง

## Current Decision

### 1. `latest` is no longer acceptable for benchmark evidence

ตั้งแต่รอบนี้เป็นต้นไป:

- ห้ามใช้ `fluree/server:latest` เป็นค่าอ้างอิงของ benchmark path
- ให้ย้ายไป image family `fluree/ledger`
- การเปิด `Fluree` ต้องระบุ `FLUREE_IMAGE` แบบ explicit image pin ก่อน

สิ่งที่เปลี่ยนใน repo:

- [docker-compose.trace.yml](/home/cit/provchain-org/benchmark-toolkit/docker-compose.trace.yml) เลิกใช้ `latest` เป็นค่าเริ่มต้นแล้ว
- service `fluree` ถูกวางไว้หลัง profile `fluree`
- มีตัวอย่าง env ที่ [`.env.trace.example`](/home/cit/provchain-org/benchmark-toolkit/.env.trace.example)

### 2. Fluree remains deferred until image pin + contract verification exist together

ก่อน re-enable ต้องมีครบ:

- explicit image pin
- verified API contract
- local adapter tests

## Repository Evidence So Far

จาก [CODEX_CONTINUITY_MEMORY.md](/home/cit/provchain-org/docs/reviews/CODEX_CONTINUITY_MEMORY.md) และ benchmark script ที่ผู้ใช้ให้มา:

- image family ที่ใช้ benchmark ได้จริงคือ `fluree/ledger`
- port หลักคือ `8090`
- health path คือ `/fdb/health`
- transaction/query path อยู่ใต้ `/fdb/<network>/<db>/...`
- `FDB_GROUP_PRIVATE_KEY` ต้องเป็น `secp256k1` private key จริงในรูปแบบ hex 64 ตัวอักษร

adapter ตอนนี้รองรับ fallback order ดังนี้:

- `/fdb/<network>/<db>/{endpoint}`
- `/v1/fluree/{endpoint}`
- `/fluree/{endpoint}`
- `/{endpoint}`

ข้อสรุป:

- ปัญหาไม่ได้อยู่แค่ query syntax
- ปัญหาอยู่ที่ image/API contract ambiguity

## Adapter Contract We Can Verify Locally

ไฟล์ adapter:

- [fluree.rs](/home/cit/provchain-org/benchmark-toolkit/research-benchmarks/src/adapters/fluree.rs)

### Verified local behaviors

1. `health_check()` ใช้ `GET /fdb/health` เป็นหลัก
   - ถ้า contract นั้นไม่พร้อม จึง fallback ไปที่ root `/`
   - root redirect ยังคงถือว่า healthy ได้

2. endpoint fallback order ถูกตรึงไว้
   - `/v1/fluree/{endpoint}`
   - `/fluree/{endpoint}`
   - `/{endpoint}`

3. `ensure_ledger_exists()` สามารถ fallback จาก family แรกไป family ที่สองได้

4. query errors ต้องเก็บ response body กลับมา
   - เพื่อให้ benchmark log ใช้วินิจฉัยได้จริง

## Local Adapter Tests

local contract tests ที่เพิ่มแล้ว:

- `health_check_accepts_root_redirect`
- `ensure_ledger_exists_falls_back_to_second_api_family`
- `execute_query_includes_response_body_in_error`

ความหมาย:

- logic ของ adapter เองถูกคุมใน local แล้ว
- ถ้ายัง fail หลังจากนี้ จะมีแนวโน้มสูงว่าเป็นปัญหาของ image pin หรือ API contract จริง

## Re-Enable Criteria

จะถือว่า `Fluree` พร้อมกลับเข้า trace family เมื่อ:

1. `FLUREE_IMAGE` ถูก pin แบบ explicit
2. local adapter tests ผ่าน
3. endpoint probe กับ image ที่ pin ไว้ยืนยัน contract เดียวกับที่ adapter ใช้
4. compose trace stack รัน `ProvChain + Neo4j + Fluree` ได้จริง
5. query scenarios ผ่านโดยไม่เกิด `404` จาก ledger/query path

## Required Operator Workflow

เมื่อจะเปิด `Fluree` อีกครั้ง:

1. ตั้ง `FLUREE_IMAGE` เป็น explicit tag
2. รัน local tests ก่อน
3. probe endpoint family ของ image ที่ pin ไว้
4. เปิด profile `fluree` พร้อมปลด `BENCHMARK_SKIP_FLUREE`
5. ค่อย rerun Docker benchmark

ตัวอย่างคำสั่ง:

```bash
export FLUREE_IMAGE=fluree/ledger:<explicit-tag>
export FLUREE_URL=http://localhost:18090
export FLUREE_LEDGER=provchain/benchmark
export FLUREE_GROUP_PRIVATE_KEY=<64-hex-secp256k1-private-key>

cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml fluree -- --nocapture
./benchmark-toolkit/scripts/probe-fluree-ledger.sh

BENCHMARK_SKIP_FLUREE=false \
docker compose -f benchmark-toolkit/docker-compose.trace.yml --profile fluree up --build
```

## Non-Negotiable Rule

ห้ามอ้างผล benchmark ที่มี `Fluree` ถ้า:

- image ยังไม่ได้ pin
- หรือ endpoint contract ยังไม่ได้ยืนยัน

เพราะจะทำให้ผล benchmark กลับไปมี ambiguity แบบเดิมอีก
