# Multi-Product Benchmark Methodology Plan - 2026-04-22

## Purpose

เอกสารนี้กำหนดวิธีวางแผน benchmark สำหรับ `ProvChain`, `Neo4j`, `Fluree`, `Hyperledger Fabric`, และ `Go Ethereum (Geth)` ให้มีความเป็นวิชาการมากพอสำหรับใช้เป็นหลักอ้างอิงภายในโครงการ และต่อยอดไปสู่ผลลัพธ์ที่ป้องกันการตีความเกินจริงได้

เป้าหมายของเอกสารนี้ไม่ใช่การประกาศว่า benchmark ทุกกลุ่มพร้อมใช้งานแล้ว แต่เพื่อกำหนดว่า:

- ควรเทียบ `ProvChain` กับผลิตภัณฑ์ใดในบริบทใด
- ห้ามเทียบอะไรกับอะไรแบบตรงๆ
- เมทริกใดจึงถือว่าเป็นธรรมในแต่ละกลุ่ม
- ต้องมีเงื่อนไขอะไรบ้างก่อนนับว่าผล benchmark ใช้ได้

เอกสารนี้ต่อยอดจาก:

- [COMPETITIVE_BENCHMARK_FAIRNESS_MATRIX_2026-04-17.md](./COMPETITIVE_BENCHMARK_FAIRNESS_MATRIX_2026-04-17.md)
- [COMPETITIVE_BENCHMARK_SPEC_2026-04-17.md](./COMPETITIVE_BENCHMARK_SPEC_2026-04-17.md)
- [COMPETITIVE_BENCHMARK_ADAPTER_IMPLEMENTATION_PLAN_2026-04-17.md](./COMPETITIVE_BENCHMARK_ADAPTER_IMPLEMENTATION_PLAN_2026-04-17.md)
- [DOCKER_COMPETITIVE_BENCHMARK_ARCHITECTURE_PLAN_2026-04-18.md](./DOCKER_COMPETITIVE_BENCHMARK_ARCHITECTURE_PLAN_2026-04-18.md)

## Core Problem

`ProvChain` ไม่ได้เป็นเพียง blockchain หรือ graph database อย่างใดอย่างหนึ่ง แต่รวมหลายชั้นความสามารถไว้พร้อมกัน:

- permissioned ledger runtime
- provenance-oriented RDF storage
- ontology-backed semantic admission
- traceability queries
- standards-facing payload handling

ดังนั้น benchmark ที่ยุติธรรมต้องไม่พยายามสรุปผลด้วยตาราง "ใครชนะทุกอย่าง" เพียงตารางเดียว

## Benchmarking Principle

### Rule 1. Compare products by benchmark family, not by brand name alone

ให้เริ่มจากคำถามว่า "กำลังวัดความสามารถอะไร" ก่อนเสมอ ไม่ใช่เริ่มจากคำถามว่า "อยากชนะผลิตภัณฑ์ใด"

### Rule 2. Do not force non-equivalent systems into one fake metric

ห้ามรวม:

- blockchain finality
- database commit
- graph traversal latency
- semantic validation overhead

เข้าเป็นตัวเลขเดียวแล้วประกาศผู้ชนะ

### Rule 3. Report native capability vs externalized pipeline separately

ถ้าระบบใดไม่มี RDF, SHACL, หรือ semantic validation แบบ native:

- ต้องบันทึกว่าใช้ external pipeline
- ต้องนับเวลา external pipeline เป็นส่วนหนึ่งของ benchmark family ที่เกี่ยวข้อง
- ห้ามตัดออกแล้วทำเหมือนระบบนั้นมีความสามารถเทียบเท่า `ProvChain` หรือ `Fluree`

### Rule 4. A benchmark run is valid only if all compared systems succeed

สำหรับแต่ละ scenario:

- ถ้าระบบใด `success_rate = 0`
- ห้ามใช้ latency ของระบบนั้นประกาศผู้ชนะ
- ต้องถือว่า scenario นั้นยังไม่พร้อมสำหรับ comparative interpretation

### Rule 5. Local contract tests must pass before Docker benchmark execution

ก่อนรัน benchmark ใน Docker:

- local correctness gate ของระบบหลักต้องผ่าน
- benchmark query contract tests ต้องผ่าน
- ห้ามใช้ workflow แบบ "run compose แล้วค่อยเดา root cause" เป็น default อีก

## Product Role Matrix

ตารางนี้ใช้กำหนด "บทบาทที่ถูกต้อง" ของแต่ละผลิตภัณฑ์ในงาน benchmark

| Product | Primary role | Secondary role | Things it should not be used for |
|---|---|---|---|
| `ProvChain` | reference system under test | ledger, trace query, semantic admission | none |
| `Neo4j` | graph query and provenance reconstruction baseline | limited write-path baseline with explicit caveat | permissioned-ledger finality baseline |
| `Fluree` | semantic ledger and RDF/JSON-LD baseline | trace query baseline | public-chain or enterprise permissioned-chain baseline |
| `Hyperledger Fabric` | permissioned enterprise ledger baseline | optional query baseline only if documented index/query layer exists | native semantic/RDF baseline |
| `Geth` | public-chain execution baseline | optional query baseline only if documented external index layer exists | permissioned enterprise ledger baseline |

## Benchmark Family Matrix

### Family A. Ledger / Write Path

คำถามวิจัย:

- ระบบใดรับข้อมูลเชิงธุรกรรมได้เร็วและเสถียรที่สุดภายใต้รูปแบบ workload เดียวกัน
- commit หรือ confirmation behavior ต่างกันอย่างไร

ผู้เข้าร่วมหลัก:

- `ProvChain`
- `Hyperledger Fabric`
- `Geth`
- `Fluree`

ผู้เข้าร่วมรอง:

- `Neo4j` ในฐานะ transactional write baseline เท่านั้น และต้องติดป้ายว่าเป็น `secondary baseline`

เมทริกหลัก:

- `submit_latency_ms`
- `commit_latency_ms`
- `confirmation_latency_ms`
- `throughput_records_per_sec`
- `throughput_tx_per_sec`
- `batch_commit_latency_ms`
- `recovery_time_ms`

กฎความเป็นธรรม:

- ห้ามเอา `Neo4j` มาเป็นตัวแทน ledger finality
- ต้องแยก `submit`, `durable commit`, และ `finality/confirmation` ออกจากกัน
- ถ้าระบบใดไม่มี finality แบบ chain-level ให้รายงานว่า `not applicable`

### Family B. Trace Query / Provenance Reconstruction

คำถามวิจัย:

- ระบบใดตอบ traceability และ provenance reconstruction ได้เร็วและถูกต้องที่สุด

ผู้เข้าร่วมหลัก:

- `ProvChain`
- `Neo4j`
- `Fluree`

ผู้เข้าร่วมแบบมีเงื่อนไข:

- `Hyperledger Fabric` ถ้ามี query/index layer ที่นิยามชัดและถูกนับเป็นส่วนหนึ่งของ stack
- `Geth` ถ้ามี query/index layer ที่นิยามชัดและถูกนับเป็นส่วนหนึ่งของ stack

เมทริกหลัก:

- `query_p50_ms`
- `query_p95_ms`
- `query_p99_ms`
- `success_rate`
- `result_count`
- `records_scanned`
- `query_throughput_qps`

สถานการณ์ query ขั้นต่ำ:

- single-entity lookup
- one-hop trace
- three-hop trace
- multi-hop trace
- full provenance reconstruction
- aggregate-by-producer / batch analytics

กฎความเป็นธรรม:

- query syntax ไม่จำเป็นต้อง identical แต่ต้อง semantic-equivalent
- ต้องใช้ canonical logical scenario เดียวกันก่อนแปลเป็น syntax ของแต่ละระบบ
- ถ้าระบบใช้ external index layer ต้องรายงานให้ชัดว่าเป็น `stacked query path`

### Family C. Semantic / Standards Admission

คำถามวิจัย:

- ระบบใดรับ payload แบบ standards-facing และ semantic-rich ได้ดีที่สุดเมื่อคิดต้นทุน semantic honestly

ผู้เข้าร่วมหลัก:

- `ProvChain`
- `Fluree`

ผู้เข้าร่วมภายนอกแบบมีเงื่อนไข:

- `Hyperledger Fabric + external semantic pipeline`
- `Geth + external semantic pipeline`
- `Neo4j + external semantic pipeline`

เมทริกหลัก:

- `mapping_latency_ms`
- `rdf_or_jsonld_ingest_latency_ms`
- `semantic_validation_latency_ms`
- `explanation_latency_ms`
- `end_to_end_admission_latency_ms`
- `validation_failure_reporting_latency_ms`

กฎความเป็นธรรม:

- ถ้าความสามารถ semantic ไม่ได้เป็น native capability ต้องนับเวลา external pipeline ทุกขั้น
- ห้ามเปรียบเทียบ semantic admission โดยตัด SHACL/RDF stage ของบางระบบออก

### Family D. Governance / Permission / Policy

คำถามวิจัย:

- ระบบใดมีต้นทุนเพิ่มขึ้นเท่าใดเมื่อบังคับใช้ permission, access control, และ policy checks

ผู้เข้าร่วมหลัก:

- `ProvChain`
- `Hyperledger Fabric`

ผู้เข้าร่วมรอง:

- `Geth` ถ้าออกแบบ smart-contract policy workload ได้อย่างชัดเจน

ผู้เข้าร่วมที่มักไม่ตรง family:

- `Neo4j`
- `Fluree`

เมทริกหลัก:

- `policy_check_overhead_ms`
- `authorized_read_latency_ms`
- `unauthorized_request_rejection_latency_ms`
- `write_with_policy_latency_ms`
- `throughput_under_policy_controls`

กฎความเป็นธรรม:

- เปรียบเทียบเฉพาะระบบที่มี policy enforcement อยู่ใน stack จริง
- ห้ามอ้างว่า product ที่ไม่มี native permissioned ledger behavior "เทียบเท่า" ใน family นี้

### Family E. Cross-System / Interchange / Bridge

คำถามวิจัย:

- `ProvChain` ส่งผ่าน provenance หรือ semantic payload ไปยังระบบอื่นได้อย่างไร และต้นทุนเท่าใด

ผู้เข้าร่วม:

- `ProvChain` เป็นระบบหลัก
- product อื่นเข้าร่วมตาม bridge path ที่มีจริง

เมทริกหลัก:

- `export_latency_ms`
- `transform_latency_ms`
- `validation_latency_ms`
- `import_latency_ms`
- `round_trip_consistency_rate`

กฎความเป็นธรรม:

- family นี้ไม่ใช่การแข่งขันแบบ winner-takes-all
- เป็น interoperability benchmark มากกว่า competitive benchmark

## Canonical Dataset Policy

ทุก benchmark family ต้องเริ่มจาก logical dataset เดียวกันก่อนเสมอ แล้วค่อยแปลเป็น target-specific representation

### Approved dataset slices

1. `UHT case-study`
2. `Hybrid GS1/EPCIS-UHT`
3. `Healthcare-device`
4. `Pharmaceutical-storage`

### Dataset rules

- ต้องมี canonical logical records กลาง
- ต้องมี canonical query scenarios กลาง
- ต้องมี documented transform layer ต่อ product
- ห้ามแก้ dataset เฉพาะระบบใดระบบหนึ่งแบบเงียบๆ

### Translation rules

- `ProvChain`: RDF / ontology-backed ingest path
- `Neo4j`: translated graph model with documented mapping
- `Fluree`: JSON-LD or RDF-compatible path with documented translation
- `Fabric`: chaincode-facing logical record translation
- `Geth`: contract-call / event translation

## Workload Matrix

| Workload | Description | Families | Eligible products |
|---|---|---|---|
| `W1` single record insert | insert one logical event | A, C | ProvChain, Fabric, Geth, Fluree, Neo4j secondary |
| `W2` batch insert | insert `10/100/1000` logical records | A | ProvChain, Fabric, Geth, Fluree, Neo4j secondary |
| `W3` single-entity lookup | lookup by batch/device/package ID | B | ProvChain, Neo4j, Fluree |
| `W4` one-hop trace | direct predecessor/successor lookup | B | ProvChain, Neo4j, Fluree |
| `W5` three-hop trace | fixed-depth provenance walk | B | ProvChain, Neo4j, Fluree |
| `W6` full provenance reconstruction | return all reachable lineage edges | B | ProvChain, Neo4j, Fluree |
| `W7` semantic admission | map + validate one semantic record | C | ProvChain, Fluree, others only with external pipeline |
| `W8` policy-enforced write | write under access/policy checks | D | ProvChain, Fabric, Geth conditional |
| `W9` policy-enforced read | query under access/policy checks | D | ProvChain, Fabric, Geth conditional |
| `W10` cross-system export/import | bridge or interchange path | E | ProvChain plus target-specific peers |

## Product-to-Family Admission Matrix

| Product | Family A | Family B | Family C | Family D | Family E |
|---|---|---|---|---|---|
| `ProvChain` | primary | primary | primary | primary | primary |
| `Neo4j` | secondary only | primary | external pipeline only | normally excluded | optional target |
| `Fluree` | primary | primary | primary | usually excluded | optional target |
| `Hyperledger Fabric` | primary | conditional | external pipeline only | primary | optional target |
| `Geth` | primary | conditional | external pipeline only | conditional | optional target |

## Fairness Labels Required in Every Report

ทุกผล benchmark ต้องติดป้ายหนึ่งในกลุ่มนี้:

- `native-comparable`
- `secondary-baseline`
- `externalized-semantic-pipeline`
- `indexed-query-stack`
- `not-comparable`

ตัวอย่าง:

- `ProvChain vs Neo4j` ใน trace query = `native-comparable` สำหรับ query family
- `ProvChain vs Neo4j` ใน ledger finality = `not-comparable`
- `ProvChain vs Fabric` ใน permissioned write path = `native-comparable`
- `ProvChain vs Geth` ใน finality = `cross-model with caveat`
- `ProvChain vs Fluree` ใน semantic admission = `native-comparable` มากกว่า Neo4j/Fabric/Geth

## Statistical Rules

เพื่อให้ benchmark ป้องกันการตีความผิด:

- ทุก scenario ต้องมี warm-up phase
- รายงานอย่างน้อย `p50`, `p95`, `p99`, `mean`, `stddev`
- ต้องบันทึก `success_rate`
- ต้องบันทึก environment manifest:
  - CPU
  - RAM
  - storage
  - Docker image tags
  - dataset slice
  - run ID
- ห้ามใช้ latency ของ failed requests เป็น evidence เชิง comparative

## Minimum Validity Criteria

จะถือว่าผล benchmark หนึ่ง scenario ใช้ได้เมื่อ:

1. เทียบกันภายใน benchmark family ที่ถูกต้อง
2. ใช้ logical dataset และ query scenario เดียวกัน
3. local contract tests ผ่านก่อนรัน
4. compared systems ทุกตัวมี `success_rate > 0`
5. report ระบุ caveat เรื่อง native vs externalized path ชัดเจน

## Evidence Classification For Paper Use

หลังจบ benchmark round `B001`-`B020` ให้จัด evidence เป็นสามชั้นก่อนนำไปใช้ใน paper:

| Evidence class | Location | Use |
|---|---|---|
| `primary-paper-evidence` | `docs/benchmarking/data/<campaign>/` | ใช้ใน result tables ได้เมื่อ family/dataset/product ตรงกับ claim |
| `reference-or-profiling-evidence` | `docs/benchmarking/data/reference/<campaign>/` | ใช้ประกอบ methods, limitations, หรือ remediation analysis เท่านั้น |
| `raw-campaign-archive` | `benchmark-toolkit/results/campaigns/<campaign>/` | ใช้ audit/debug; ห้ามใช้ตรงใน paper ถ้ายังไม่มี curated export |

Primary paper evidence ชุดปัจจุบัน:

- `docs/benchmarking/data/trace_supply1000_provchain_neo4j_n30_20260424/`
- `docs/benchmarking/data/trace_supply1000_provchain_neo4j_fluree_n30_20260428/`
- `docs/benchmarking/data/ledger_supply1000_provchain_fabric_managed_n30_20260425/`
- `docs/benchmarking/data/semantic_supply1000_provchain_fluree_n30_20260428/`
- `docs/benchmarking/data/policy_supply1000_provchain_fabric_e2e_n30_20260429/`
- `docs/benchmarking/data/ledger_supply1000_provchain_geth_n30_20260428/`

Reference/profiling evidence ชุดปัจจุบัน:

- `docs/benchmarking/data/reference/policy_supply1000_fabric_pack_n30_20260428/`
- `docs/benchmarking/data/reference/policy_supply1000_provchain_fabric_n30_20260429/`
- `docs/benchmarking/data/reference/policy_supply1000_fabric_n30_20260425/`
- `docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_n3_20260429/`
- `docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_flush100_n3_20260429/`
- `docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_staterootcache_n3_20260429/`
- `docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_walsync100_n3_20260429/`
- `docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_batchblock_n3_20260429/`
- `docs/benchmarking/data/reference/20260429_profile_ledger_supply1000_provchain_only_coldsteady_conservative_n3/`
- `docs/benchmarking/data/reference/PROVCHAIN_LEDGER_PROFILE_COMPARISON_20260429.md`
- `docs/benchmarking/data/reference/PROVCHAIN_LEDGER_BATCH_BLOCK_PROFILE_20260429.md`
- `docs/benchmarking/data/reference/PROVCHAIN_LEDGER_COLD_STEADY_PROFILE_20260429.md`

กฎสำคัญ:

- publication report bundle ดึงได้เฉพาะ `primary-paper-evidence`
- profiling evidence เช่น `profiling_ledger_supply1000_provchain_only_n3_20260429` ใช้ตอบคำถามว่า "คอขวดอยู่ตรงไหน" ไม่ใช่ "ระบบใดชนะ"
- relaxed-durability profiling evidence เช่น `profiling_ledger_supply1000_provchain_only_walsync100_n3_20260429` ต้องระบุ WAL/index fsync interval และห้ามใช้แทน production durable-throughput evidence
- batch-semantics profiling evidence เช่น `profiling_ledger_supply1000_provchain_only_batchblock_n3_20260429` ต้องระบุว่า `100` triples ถูกบันทึกเป็น `1` block และห้ามใช้แทน metric `100 tx` เดิม
- cold/steady profiling evidence เช่น `20260429_profile_ledger_supply1000_provchain_only_coldsteady_conservative_n3` ต้องแยก `Turtle RDF Import` ออกจาก `Steady-state Append After Cold Load (100 tx)` และห้ามผสมสอง phase ใน paper table เดียว
- failed, smoke, partial, หรือ incomplete campaign ต้องอยู่ใน archive/debug layer จนกว่าจะ rerun ผ่าน validity gate และ export ใหม่

## Current Recommended Execution Roadmap

### Phase 1. Stable baseline

เป้าหมาย:

- `ProvChain vs Neo4j`
- เฉพาะ `Family B. Trace Query`

สถานะ:

- ใช้งานได้แล้ว
- เป็น engineering baseline ที่ใช้ได้

### Phase 2. Semantic-semantic comparison

เป้าหมาย:

- `ProvChain vs Fluree`
- `Family B` และ `Family C`

เงื่อนไขก่อนเริ่ม:

- ต้อง pin Fluree version ให้ชัด
- ต้องยืนยัน adapter/API contract

### Phase 3. Permissioned ledger comparison

เป้าหมาย:

- `ProvChain vs Hyperledger Fabric`
- `Family A` และ `Family D`

เงื่อนไขก่อนเริ่ม:

- Fabric ต้องมี real write path
- ถ้ามี query layer ต้องประกาศว่าเป็น on-chain หรือ indexed query stack

### Phase 4. Public-chain baseline

เป้าหมาย:

- `ProvChain vs Geth`
- เน้น `Family A`

เงื่อนไขก่อนเริ่ม:

- ต้องนิยาม smart-contract workload ที่เทียบได้จริง
- ต้องแยก `submit` กับ `confirmation` ให้ชัด

### Phase 5. Full publication bundle

เป้าหมาย:

- รวมหลาย family ในรายงานเดียว
- แต่ยังต้องแยกตารางผลตาม family

ข้อห้าม:

- ห้ามทำ single global winner table

## Immediate Task List

1. ปรับ `benchmark-toolkit` ให้รองรับ family labels อย่าง explicit
2. เพิ่ม environment manifest สำหรับทุก run
3. แยกผลลัพธ์ตาม benchmark family ชัดเจนใน report schema
4. รักษา local benchmark-query contract suite เป็น precondition ก่อน Docker rerun
5. ฟื้น `Fluree` โดยเริ่มจาก version pinning และ API contract verification
6. สร้าง real adapter path สำหรับ `Fabric`
7. สร้าง minimal but honest adapter path สำหรับ `Geth`

## Decision Summary

ข้อสรุปที่ต้องยึดต่อจากนี้คือ:

- เราไม่ได้ benchmark `ProvChain` กับทุก product ด้วยกติกาเดียว
- เรา benchmark เป็นหลาย benchmark families
- แต่ละ family มีผู้เข้าร่วมหลักไม่เท่ากัน
- ทุก comparative claim ต้องระบุ family, dataset slice, และ fairness label
- ถ้าไม่ทำตามนี้ benchmark จะเสี่ยงกลายเป็น marketing comparison มากกว่างานวิชาการ
