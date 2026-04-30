# Benchmark Next Actions - 2026-04-23

## Immediate Priority

1. รักษา campaign `20260424_trace_supply1000_provchain-neo4j_n30` ไว้เป็น baseline หลักสำหรับ `Trace Query / Provenance`
2. ใช้ curated export ที่ `docs/benchmarking/data/trace_supply1000_provchain_neo4j_n30_20260424/` สำหรับ paper/report แทนการอ้าง raw result directory ทั้งก้อน
3. รักษา campaign `20260425_ledger_supply1000_provchain-fabric_managed_n30_fix3` ไว้เป็น baseline หลักสำหรับ `Ledger / Write Path`
4. ใช้ curated export ที่ `docs/benchmarking/data/ledger_supply1000_provchain_fabric_managed_n30_20260425/` สำหรับ paper/report แทนการอ้าง raw result directory ทั้งก้อน
5. รักษา campaign `20260425_policy_supply1000_fabric_n30` ไว้เป็น Fabric policy evidence สำหรับ `Governance / Policy`
6. ใช้ curated export ที่ `docs/benchmarking/data/reference/policy_supply1000_fabric_n30_20260425/`
7. แยก `Fluree` ออกจาก benchmark evidence จนกว่าจะปิดเรื่อง runtime contract verification
8. งานถัดไปตามแผนคือ `B015` นิยาม Geth workload

## Current Execution Order

### Track A - Stable baseline

1. ใช้ [BENCHMARK_STATUS_SUMMARY_2026-04-23.md](./BENCHMARK_STATUS_SUMMARY_2026-04-23.md) เป็นสถานะอ้างอิงหลัก
2. ใช้ [20260424_trace_supply1000_provchain-neo4j_n30](/home/cit/provchain-org/benchmark-toolkit/results/campaigns/20260424_trace_supply1000_provchain-neo4j_n30) เป็น baseline campaign อ้างอิง
3. ใช้ [trace_supply1000_provchain_neo4j_n30_20260424](./data/trace_supply1000_provchain_neo4j_n30_20260424/) เป็น publication-facing export
4. บังคับใช้ preflight gate ก่อน benchmark rerun ทุกครั้ง ยกเว้น smoke ที่ตั้งใจ `--skip-preflight`
5. ใช้ `benchmark-toolkit/scripts/provchain-neo4j-campaign.sh` สำหรับ smoke/full/status runs ครั้งถัดไป

### Track B - Fluree verification only

1. หยุดเพิ่ม benchmark claims สำหรับ `Fluree`
2. ยืนยัน `ledger create/query contract` จาก runtime ต้นทางที่เคยรันได้จริง
3. เมื่อ contract ชัดแล้ว ค่อยปรับ adapter และ compose อีกรอบเดียว
4. หลังจากนั้นจึงทำ `B008`

### Track C - Fabric as next comparator

1. `B010` กำหนด topology benchmark-facing ของ `Fabric` เสร็จแล้วใน `FABRIC_BENCHMARK_TOPOLOGY_CONTRACT_2026-04-24.md`
2. `B011` กำหนด logical record mapping เสร็จแล้วใน `FABRIC_CANONICAL_RECORD_MAPPING_2026-04-24.md`
3. `B012` local contract tests และ local smoke gate เสร็จแล้ว:
   - `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml fabric -- --nocapture`
   - `ITERATIONS=1 FABRIC_BATCH_SIZE=10 ./benchmark-toolkit/scripts/run-fabric-contract-smoke.sh`
4. `B013` เปิด `Family A. Ledger / Write Path` ด้วย Fabric runtime จริงแล้ว:
   - campaign `20260425_ledger_supply1000_provchain-fabric_managed_n30_fix3`
   - status `passed`, epochs `30/30`
   - export `docs/benchmarking/data/ledger_supply1000_provchain_fabric_managed_n30_20260425/`
5. `B014` เปิด `Family D. Governance / Policy` ด้วย Fabric runtime จริงแล้ว:
   - campaign `20260425_policy_supply1000_fabric_n30`
   - status `passed`, epochs `30/30`
   - export `docs/benchmarking/data/reference/policy_supply1000_fabric_n30_20260425/`
6. Codex process ปัจจุบันยังอาจเข้า Docker socket ไม่ได้ในบาง session; runtime campaign ที่ใช้ Docker ควรรันจาก shell ที่มี Docker access หรือ session ที่ inherited docker group

## Recommended Working Rule

- ห้ามใช้ `docker compose` เพื่อเดา contract ของ comparator ใหม่
- comparator ใหม่ทุกตัวต้องมี:
  - local contract tests
  - explicit image/runtime contract
  - documented caveat
- benchmark evidence ใช้ได้เฉพาะ comparator ที่ผ่าน 3 ข้อนี้แล้ว

## Decision

ถ้าต้องเลือกงานถัดไปเพียงงานเดียว:

- ให้ทำ `B015` ของ `Geth` ก่อน

เหตุผล:

- `ProvChain vs Neo4j` baseline พร้อมแล้ว
- n30 campaign ผ่านครบและถูก export เป็น publication evidence แล้ว
- `Fluree` ยังติด ambiguity ระดับ runtime contract
- `Fabric` ledger และ policy runtime evidence ผ่านแล้ว ดังนั้น public-chain baseline ถัดไปคือ `Geth`
