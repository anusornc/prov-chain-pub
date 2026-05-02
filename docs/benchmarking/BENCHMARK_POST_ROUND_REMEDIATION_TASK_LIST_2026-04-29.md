# Benchmark Post-Round Remediation Task List - 2026-04-29

## Purpose

เอกสารนี้เป็น task list หลังจบ benchmark round `B001`-`B020`

สถานะสำคัญ:

- benchmark execution backlog เดิมเสร็จครบแล้ว
- evidence bundle และ publication report bundle พร้อมใช้อ้างอิง
- งานต่อจากนี้ไม่ใช่การเปิด comparator เพิ่ม แต่เป็นการแก้จุดอ่อนที่เห็นจากผล benchmark และลดความเสี่ยงการตีความผลผิดใน paper

Primary evidence:

- [PUBLICATION_BENCHMARK_REPORT_BUNDLE_2026-04-28.md](./PUBLICATION_BENCHMARK_REPORT_BUNDLE_2026-04-28.md)
- [BENCHMARK_STATUS_SUMMARY_2026-04-23.md](./BENCHMARK_STATUS_SUMMARY_2026-04-23.md)

## Result Assessment

### Strong Evidence

- `ProvChain` ทำ trace-query ได้ดีมากเมื่อเทียบกับ `Neo4j` และ `Fluree`
  - Simple Product Lookup: ProvChain `0.487 ms`, Fluree `9.722 ms`, Neo4j `10.304 ms`
  - Multi-hop Traceability: ProvChain `0.479 ms`, Fluree `10.060 ms`, Neo4j `14.515 ms`
  - Aggregation by Producer: ProvChain `0.566 ms`, Neo4j `28.477 ms`, Fluree `127.179 ms`
- ProvChain/Fabric policy parity workload ผ่านครบแบบมี caveat
  - campaign `20260429_policy_supply1000_provchain-fabric_e2e_n30` ผ่าน `30/30`
  - ทั้ง 5 scenarios มี `300` samples และ success rate `100%`
  - client-observed round-trip: Fabric mean `6.231`-`6.831 ms`, ProvChain mean `1.864`-`2.249 ms`
  - server-reported decision latency: Fabric mean `5.167`-`5.636 ms`, ProvChain mean `0.001`-`0.002 ms`
  - ProvChain rows เป็น `cross-model-with-caveat` เพราะ endpoint รับ policy context ใน request ขณะที่ Fabric อ่านจาก ledger state
- Geth evidence ใช้ได้ในฐานะ public-chain baseline เท่านั้น
  - submit mean `2.208 ms`
  - confirmation mean `256.571 ms`
  - gas mean `46248.600 gas`

### Weak Or Risky Evidence

- ProvChain legacy per-transaction write/admission path ช้ากว่า comparator ชัดเจนใน workloads ที่วัด
  - ProvChain single-threaded write workload: mean `29352.300`-`29361.670 ms` ต่อ `100 tx`
  - Fabric batch commit `100 records`: mean `2165.363 ms`
  - Geth local confirmation: mean `256.571 ms` ต่อ single record, แต่เป็น public-chain dev baseline ไม่ใช่ permissioned enterprise baseline
  - R002 bulk Turtle import แก้เฉพาะ dataset-admission/import-row path แล้ว ไม่ใช่ proof ว่า per-transaction ledger/write throughput ชนะ comparator
- Semantic admission ของ ProvChain ช้ากว่า Fluree externalized pipeline ใน current measurement
  - ProvChain native RDF+SHACL admission: mean `12121.933 ms`
  - Fluree externalized JSON-LD admission: mean `514.667 ms`
  - ต้องรักษา caveat ว่า capability path ต่างกัน เพราะ ProvChain เป็น native semantic validation พร้อม explanation support
- publication report ยังมี legacy rows ที่ `fairness_label` และ `capability_path` เป็น `legacy-not-recorded`
  - ใช้อ้างอิงได้ แต่ควร cleanup ก่อนนำไปใส่ paper final table
- comparative `ProvChain vs Fabric` policy parity evidence มีแล้ว แต่ต้องรักษา caveat เรื่อง endpoint/model semantics

## Remediation Backlog

### R001 - Clean publication report semantics

- Priority: `P0`
- Status: `Done`
- Goal: ทำให้ publication report ไม่ทำให้ผู้อ่านเข้าใจผิดเรื่อง family และ metric
- Tasks:
  - แยก load/import rows ออกจาก `ledger-write` หรือ label เป็น `data-load`
  - backfill หรือ annotate legacy rows ที่ยังเป็น `legacy-not-recorded`
  - เพิ่ม rule ใน generator ว่า table ใดเป็น primary evidence และ table ใดเป็น legacy/reference evidence
- Done when:
  - report ไม่มี ledger-write table ที่ปน trace import/load rows แบบไม่อธิบาย
  - legacy rows มี caveat ชัดเจน
- Completion evidence:
  - `benchmark-toolkit/scripts/generate-publication-benchmark-report.py` now separates primary metrics from load/import/reference metrics
  - `docs/benchmarking/PUBLICATION_BENCHMARK_REPORT_BUNDLE_2026-04-28.md` includes caveat columns and no global winner table

### R002 - Profile and optimize ProvChain write/admission path

- Priority: `P0`
- Status: `Done for R002 bulk import path - merged to main and final full campaign passed`
- Status note: first R002 bulk profile passed but included auth/bootstrap latency; the auth-excluded rerun passed and is the current R002 evidence for the import-row claim boundary
- Goal: ลด latency ของ ProvChain write/admission workload ก่อน claim ledger/write competitiveness
- Tasks:
  - [x] profile `Single-threaded Write (100 tx)` เพื่อแยก RDF parse, SHACL validation, block commit, Oxigraph write, hashing/signature
  - [x] เพิ่ม per-stage timing ใน benchmark artifacts
  - [x] ทดลองลด repeated RDF snapshot flush cost ใน benchmark-managed append path
  - [x] ทดลองลด repeated state-root hashing/sorting cost ด้วย cache ใน RDF store
  - [x] ทดลองลด per-block WAL/index fsync cost ใน benchmark-managed relaxed-durability profiling path
  - [x] ทดลอง batch RDF insert หรือ transaction batching ใน native path
  - [x] แยก cold-load cost ออกจาก steady-state append cost ใน conservative ProvChain-only profile
  - [x] สร้าง branch ทดลอง `exp-provchain-bulk-import-r002` เพื่อทดสอบ import algorithm แยกจาก `main`
  - [x] เพิ่ม authenticated bulk Turtle endpoint `POST /api/datasets/import-turtle`
  - [x] เพิ่ม benchmark adapter mode `PROVCHAIN_IMPORT_MODE=bulk-turtle-single-block` พร้อม fallback `legacy-per-triple`
  - [x] เพิ่ม one-command wrapper `benchmark-toolkit/scripts/provchain-bulk-import-r002-campaign.sh`
  - [x] run first R002 Docker smoke campaign บน branch ทดลอง
  - [x] run first R002 Docker profile campaign และ export curated reference evidence
  - [x] แก้ harness ให้ `load-latency-ms` ไม่รวม auth/bootstrap แต่ยังบันทึก `auth_latency_ms`
  - [x] rerun R002 Docker smoke campaign หลัง auth-exclusion fix
  - [x] rerun R002 Docker profile campaign หลัง auth-exclusion fix และ export curated reference evidence
  - [x] แก้ R002 wrapper default export dir ให้ผูกกับ campaign id เพื่อไม่ชนกับ export เดิม
  - [x] document legacy A/B decision: not required for the current paper because same-slice legacy baseline and R002 final campaign already define the import-row claim boundary; rerun only if reviewers require same-day A/B evidence
  - [x] document conservative cold/steady decision: existing conservative profile remains reference evidence; no additional full campaign is required for the R002 bulk-import claim
  - [x] document comparative decision: R002 final comparative campaign passed for bulk dataset admission; relaxed-durability/batching diagnostics remain excluded from primary ledger/write claims
  - [x] merge R002 branch กลับ `main` หลัง smoke/profile ผ่านและ claim boundary ถูกอัปเดตแล้ว
  - [x] run R002 full campaign บน `main`
- Done when:
  - มี per-stage timing artifact
  - มี optimized write-path campaign อย่างน้อย smoke `n1` และ full `n30` หลัง optimization หลัก
  - report ระบุได้ว่าคอขวดหลักอยู่ที่ stage ใด
- Current evidence:
  - profiling campaign: `20260429_profile_ledger_supply1000_provchain-only_n3_fix1`
  - curated export: `docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_n3_20260429/`
  - result: `3/3` epochs passed, `9` samples, `100 tx` per sample
  - stage summary: `docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_n3_20260429/provchain_profile_summary.md`
  - current bottleneck: server-side `block_admission` averages `97.731 ms/tx`, `96.24%` of mean handler time
- Scoped optimization evidence:
  - change: benchmark-managed ProvChain now uses `PROVCHAIN_RDF_FLUSH_INTERVAL=100` to avoid dumping the RDF snapshot on every block while preserving WAL-per-block behavior
  - post-change campaign: `20260429_profile_ledger_supply1000_provchain-only_flush100_n3`
  - curated export: `docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_flush100_n3_20260429/`
  - comparison note: `docs/benchmarking/data/reference/PROVCHAIN_LEDGER_PROFILE_COMPARISON_20260429.md`
  - effect: batch total mean improved from `11967.556 ms` to `10695.444 ms` per `100 tx` (`10.63%` faster)
  - effect: server `block_admission` mean improved from `97.731 ms/tx` to `85.207 ms/tx` (`12.82%` faster)
  - remaining bottleneck: `block_admission` is still `95.77%` of mean handler time
- State-root cache optimization evidence:
  - change: RDF store now caches sorted per-quad hashes for state-root calculation and updates the cache incrementally on append paths
  - commit: `fa4bfcf perf(storage): cache rdf state root hashes`
  - post-change campaign: `20260429_profile_ledger_supply1000_provchain-only_staterootcache_n3`
  - curated export: `docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_staterootcache_n3_20260429/`
  - comparison note: `docs/benchmarking/data/reference/PROVCHAIN_LEDGER_PROFILE_COMPARISON_20260429.md`
  - effect: batch total mean improved from `11967.556 ms` to `6011.111 ms` per `100 tx` (`49.77%` faster cumulatively)
  - effect: server `block_admission` mean improved from `97.731 ms/tx` to `38.197 ms/tx` (`60.92%` faster cumulatively)
  - remaining bottleneck: `block_admission` is still `90.96%` of mean handler time
- WAL/index fsync batching profiling evidence:
  - change: persistent storage now supports configurable WAL and chain-index fsync intervals; default remains conservative sync-every-block
  - post-change campaign: `20260429_profile_ledger_supply1000_provchain-only_walsync100_n3`
  - curated export: `docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_walsync100_n3_20260429/`
  - durability mode: relaxed batched fsync with WAL/index sync intervals set to `100`
  - effect: batch total mean improved from `11967.556 ms` to `5926.444 ms` per `100 tx` (`50.48%` faster cumulatively)
  - effect: server `block_admission` mean improved from `97.731 ms/tx` to `37.618 ms/tx` (`61.51%` faster cumulatively)
  - remaining bottleneck: `block_admission` is still `90.98%` of mean handler time
  - caveat: this evidence cannot be used as production durable-throughput or cross-system comparison evidence
- Batch-block diagnostic evidence:
  - change: added `POST /api/blockchain/add-triples` and benchmark row `Batched Write (100 triples, 1 block)`
  - smoke campaign: `smoke_ledger_supply1000_provchain-only_batchblock_n1_20260429`
  - profiling campaign: `20260429_profile_ledger_supply1000_provchain-only_batchblock_n3`
  - curated export: `docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_batchblock_n3_20260429/`
  - diagnostic note: `docs/benchmarking/data/reference/PROVCHAIN_LEDGER_BATCH_BLOCK_PROFILE_20260429.md`
  - result: existing `100 tx` path mean `6434.444 ms`; diagnostic `100 triples, 1 block` mean `1646.265 ms`
  - effect: `74.41%` lower client-observed mean latency in the same profiling campaign
  - caveat: this is a semantics change and relaxed-durability profiling evidence, not replacement evidence for the `100 tx` ledger/write comparison metric
  - next required task: rerun full/comparative campaigns with clearly separated durability and batching modes
- Cold-load vs steady-state append evidence:
  - change: added `benchmark-toolkit/scripts/provchain-ledger-r002-campaign.sh` to run a managed ProvChain-only R002 campaign with `--include-load`, explicit durability mode, curated export, and profile summary generation
  - campaign: `20260429_profile_ledger_supply1000_provchain-only_coldsteady_conservative_n3`
  - curated export: `docs/benchmarking/data/reference/20260429_profile_ledger_supply1000_provchain_only_coldsteady_conservative_n3/`
  - note: `docs/benchmarking/data/reference/PROVCHAIN_LEDGER_COLD_STEADY_PROFILE_20260429.md`
  - status: `passed`, `3/3` epochs
  - durability mode: conservative WAL/index sync every block (`wal=1`, `chain_index=1`)
  - result: cold `Turtle RDF Import` mean `50736.000 ms` for `632` triples
  - result: post-load `Steady-state Append After Cold Load (100 tx)` mean `17972.444 ms`
  - result: post-load server `block_admission` mean `161.195 ms/tx`, `97.75%` of mean handler time
  - interpretation: state size materially increases per-block admission cost; cold-load and append rows must not be mixed in paper tables
- Bulk Turtle import branch implementation:
  - branch: `exp-provchain-bulk-import-r002`
  - plan: `docs/benchmarking/PROVCHAIN_BULK_IMPORT_R002_PLAN_2026-04-30.md`
  - endpoint: authenticated `POST /api/datasets/import-turtle`
  - algorithm: validate the Turtle document once, count quads, then admit the full normalized Turtle dataset as one blockchain block instead of submitting each triple as a separate block
  - benchmark mode: default `PROVCHAIN_IMPORT_MODE=bulk-turtle-single-block`; diagnostic fallback `legacy-per-triple`
  - campaign wrapper: `benchmark-toolkit/scripts/provchain-bulk-import-r002-campaign.sh`
  - compose/manifest wiring: `PROVCHAIN_IMPORT_MODE` is passed into the benchmark-runner container and recorded in `campaign_manifest.json`
  - local validation:
    - `cargo check --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml` passed with existing warning only
    - `cargo check --bin provchain-org` passed with existing warnings only
    - `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml provchain -- --nocapture` passed after rerun with local mock-server permissions
    - `cargo test turtle_import_payload -- --nocapture` passed
    - `bash -n benchmark-toolkit/scripts/run-trace-campaign.sh` passed
    - `bash -n benchmark-toolkit/scripts/provchain-bulk-import-r002-campaign.sh` passed
    - `docker compose -f benchmark-toolkit/docker-compose.trace.yml config` passed
    - `git diff --check` passed
  - completed smoke command used during R002 validation:
    - `./benchmark-toolkit/scripts/provchain-bulk-import-r002-campaign.sh smoke --id smoke_import_supply1000_provchain-bulk-r002_authfix_n1_20260430`
  - interpretation rule: even if the import row improves materially, this evidence is a bulk dataset-admission result and must not replace the per-transaction `Single-threaded Write (100 tx)` ledger/write metric
- First R002 Docker profile result:
  - smoke: `smoke_import_supply1000_provchain-bulk-r002_n1_20260430`, status `passed`, `1/1`
  - profile: `20260430_import_supply1000_provchain-bulk-r002_n3`, status `passed`, `3/3`
  - curated export: `docs/benchmarking/data/reference/r002_import_supply_chain_1000_provchain_bulk-r002_n3_20260430/`
  - result: ProvChain `Turtle RDF Import` mean `661.333 ms`, Fluree `JSON-LD Import` mean `486.000 ms`, Neo4j `Turtle to Cypher Import` mean `11119.667 ms`
  - improvement versus same-slice legacy baseline `20260424_trace_supply1000_provchain-neo4j_n30`: `12124.033 ms` to `661.333 ms`, about `18.34x` faster and `94.55%` lower mean latency
  - audit finding: ProvChain metadata showed `auth_latency_ms` about `634`-`646 ms`, while server-side handler was about `10 ms`; therefore the first profile proves the algorithmic direction but is superseded for final R002 import-row reporting
  - fix: `benchmark-toolkit/research-benchmarks/src/adapters/provchain.rs` now authenticates before starting dataset-load timing and keeps auth as separate metadata
- Auth-excluded R002 profile result:
  - profile: `20260430_import_supply1000_provchain-bulk-r002_authfix_n3`, status `passed`, `3/3`
  - curated export: `docs/benchmarking/data/reference/r002_import_supply_chain_1000_provchain_bulk-r002_authfix_n3_20260430/`
  - result: ProvChain `Turtle RDF Import` mean `23.000 ms`, Fluree `JSON-LD Import` mean `514.333 ms`, Neo4j `Turtle to Cypher Import` mean `11594.667 ms`
  - improvement versus same-slice legacy baseline `20260424_trace_supply1000_provchain-neo4j_n30`: `12124.033 ms` to `23.000 ms`, about `527.13x` faster and `99.81%` lower mean latency
  - audit metadata: auth remains recorded separately at about `610`-`637 ms`; client submit loop is about `11 ms`; server handler total is about `10 ms`; `block_count=1`, `triple_count=632`
  - wrapper fix: future default exports use `docs/benchmarking/data/reference/<campaign_id>` to avoid collisions
  - merge: `exp-provchain-bulk-import-r002` fast-forward merged into `main`
  - required full command on `main`:
    - `./benchmark-toolkit/scripts/provchain-bulk-import-r002-campaign.sh full --id 20260430_import_supply1000_provchain-bulk-r002_final_n30`
- Final R002 full campaign:
  - campaign: `20260430_import_supply1000_provchain-bulk-r002_final_n30`
  - status: `passed`, `30/30`
  - curated export: `docs/benchmarking/data/reference/20260430_import_supply1000_provchain-bulk-r002_final_n30/`
  - manifest: `provchain_import_mode=bulk-turtle-single-block`, `iterations_per_epoch=10`, `products=provchain,neo4j,fluree`
  - result: ProvChain `Turtle RDF Import` mean `24.333 ms`, p95 `35.150 ms`, p99 `41.000 ms`
  - result: Fluree `JSON-LD Import` mean `478.467 ms`, p95 `560.650 ms`, p99 `575.970 ms`
  - result: Neo4j `Turtle to Cypher Import` mean `11431.367 ms`, p95 `11855.800 ms`, p99 `11935.670 ms`
  - final improvement versus same-slice legacy baseline `20260424_trace_supply1000_provchain-neo4j_n30`: `12124.033 ms` to `24.333 ms`, about `498.26x` faster and `99.80%` lower mean latency
  - final caveat: this is a bulk dataset-admission result, not a replacement for per-transaction ledger/write or finality metrics

### R003 - Profile semantic admission cost

- Priority: `P1`
- Status: `Done - semantic profile campaign passed and curated reference evidence exported`
- Goal: ทำให้ semantic-family claim แข็งแรงกว่าแค่ end-to-end latency
- Tasks:
  - [x] แยกเวลา client-side dataset read, TTL normalize, TTL parse, auth, submit loop สำหรับ ProvChain semantic admission
  - [x] เพิ่ม diagnostic rows แบบ `not-comparable` สำหรับ semantic stage timings เพื่อไม่ปน primary metric
  - [x] แก้ Fluree externalized path ให้ semantic row นับ TTL-to-JSON-LD translation cost และ JSON-LD read/parse/ledger insert metadata
  - [x] แก้ benchmark startup ให้ semantic campaign ใช้ ontology package manifest และ skip demo preload ที่ไม่ใช่ benchmark dataset
  - [x] เพิ่ม server-side timing ที่แยก ontology validation, SHACL parse/load, shape validation/reasoning, explanation summary, state-root/hash/sign, RDF store insert, metadata insert, persistence
  - [x] ตรวจ smoke failure ล่าสุดและแยกสาเหตุว่าเป็น host port collision ไม่ใช่ metric failure
  - [x] ปรับ semantic campaign wrapper ให้ใช้ isolated host port block, บันทึก port set ลง manifest, และ export curated evidence อัตโนมัติเมื่อ full/run ผ่าน
  - [x] เพิ่ม campaign id guard เพื่อกันคำสั่งที่ถูก line-wrap หลัง hyphen แล้วได้ id ไม่สมบูรณ์
  - [x] รัน semantic profile campaign ใหม่ใน shell ที่ `docker info` ผ่าน แล้ว export curated evidence
- Done when:
  - semantic report แสดง native capability cost breakdown
  - paper สามารถอธิบาย tradeoff ระหว่าง latency กับ native semantic support ได้
  - campaign ใหม่ผ่าน validity gate และถูก export เข้า `docs/benchmarking/data/reference/` หรือ primary evidence ตาม claim boundary
  - publication report generator ไม่ดึง diagnostic `not-comparable` rows เข้า primary tables
Current implementation evidence:
  - code paths updated:
    - `benchmark-toolkit/research-benchmarks/src/main.rs`
    - `benchmark-toolkit/research-benchmarks/src/adapters/fluree.rs`
    - `benchmark-toolkit/docker-compose.trace.yml`
    - `benchmark-toolkit/scripts/provchain-fluree-semantic-campaign.sh`
    - `src/main.rs`
    - `src/core/blockchain.rs`
    - `src/ontology/shacl_validator.rs`
    - `src/storage/rdf_store.rs`
    - `src/web/handlers/transaction.rs`
  - server-side timing now includes env-gated `timings_ms` fields for ontology validation, SHACL data-store create, SHACL RDF parse/load, SHACL core/domain shape validation, explanation summary, state-root, block hash construction, signature creation/verification, RDF store parse/insert/cache, metadata insert, chain push, and persistence
  - local validation:
    - `cargo check --bin provchain-org` passed with existing warnings only
    - `cargo check --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml` passed with existing warning only
    - local endpoint smoke passed: bootstrap admin, login, and authenticated `POST /api/policy/check` returned `authorized=true`
    - `cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml fluree -- --nocapture` passed `5/5`
    - `docker compose -f benchmark-toolkit/docker-compose.trace.yml config` passed
    - `PROVCHAIN_DATA_DIR=/tmp/provchain-semantic-package-smoke2 timeout 35s cargo run -- examples web-server --port 18183 --ontology-package config/ontology_package.toml --skip-demo-data` reached web server startup; timeout exit `124` was expected
  - latest failed smoke diagnosis:
    - campaign: `smoke_semantic_supply1000_provchain-fluree_profile_n1_20260429`
    - status: `partial`, `0/1` passed, no benchmark rows produced
    - root cause: Docker could not bind host port `18080` because it was already in use
    - remediation: `benchmark-toolkit/scripts/provchain-fluree-semantic-campaign.sh` now defaults to semantic host ports `18180`, `19190`, `18474`, `18687`, and `18190`, with `--port-base` override support and automatic curated export for passing `full`/`run` campaigns
  - latest smoke gate:
    - campaign: `smoke_semantic_supply1000_provchain-`
    - status: `passed`, `1/1` epoch
    - caveat: id was truncated by shell line wrapping after `provchain-`; use only as smoke gate, not curated paper evidence
    - result: ProvChain native semantic admission `7172.000 ms`; Fluree externalized admission `906.389 ms`
    - main ProvChain client-side cost: HTTP submit loop `6513.138 ms`; dataset read/normalize/parse was small (`12.542 ms` combined)
  - semantic profile evidence:
    - campaign: `20260429_profile_semantic_supply1000_provchain-fluree_n3`
    - status: `passed`, `3/3` epochs
    - curated export: `docs/benchmarking/data/reference/profiling_semantic_supply1000_provchain_fluree_n3_20260429/`
    - note: `docs/benchmarking/data/reference/PROVCHAIN_SEMANTIC_ADMISSION_PROFILE_20260429.md`
    - result: ProvChain native RDF+SHACL admission mean `7109.333 ms`; Fluree externalized TTL+JSON-LD admission mean `872.691 ms`
    - result: ProvChain HTTP submit loop mean `6461.478 ms`; auth mean `635.764 ms`; dataset read/normalize/parse combined mean `12.571 ms`
    - server-side result: ProvChain handler mean `9.796 ms/record`, block admission `9.358 ms/record`, state root `4.396 ms/record`, persistence `4.469 ms/record`, ontology+SHACL validation about `0.620 ms/record`
    - interpretation: semantic validation itself is not the dominant cost; current admission latency is dominated by per-record API/block admission, state root, and persistence

### R004 - Implement ProvChain policy parity workload

- Priority: `P1`
- Status: `Done`
- Goal: ปิดช่องว่าง comparative policy-family evidence
- Tasks:
  - [x] นิยาม ProvChain policy adapter/workload ให้ match Fabric scenarios
  - [x] ใช้ scenarios เดียวกับ Fabric pack: authorized-read, auditor-read, unauthorized-read, authorized-write, rejected-write
  - [x] เพิ่ม expected rejection semantics และ metadata ให้เหมือน Fabric
  - [x] แก้ Fabric campaign runner ให้รายงาน preflight failure ถูกต้องเมื่อ ProvChain API ยังไม่ขึ้น
  - [x] รัน comparative smoke/full campaign ใหม่ใน environment ที่มีทั้ง ProvChain API และ Fabric gateway
- Done when:
  - มี `ProvChain vs Fabric` policy campaign หรือมีเหตุผลชัดว่าทำ comparative policy ไม่ได้ใน architecture ปัจจุบัน
Current implementation evidence:
  - `src/web/handlers/transaction.rs` exposes authenticated `POST /api/policy/check`
  - `benchmark-toolkit/research-benchmarks/src/adapters/provchain.rs` can call the ProvChain policy endpoint
  - `benchmark-toolkit/research-benchmarks/src/main.rs` now emits both ProvChain and Fabric rows for the same five policy scenarios when neither product is skipped
  - ProvChain policy rows are labeled `cross-model-with-caveat` because Fabric reads policy from ledger state while the first ProvChain parity endpoint receives policy context in the request
  - `benchmark-toolkit/scripts/run-fabric-ledger-campaign.sh` now defines the missing `die()` helper and points failed ProvChain health checks at `--manage-provchain`
  - latest user-run policy smoke reached real Fabric gateway successfully, then stopped because `http://localhost:8080/health` was not serving ProvChain
  - comparative full campaign:
    - campaign: `20260429_policy_supply1000_provchain-fabric_e2e_n30`
    - status: `passed`
    - epochs: `30/30`
    - iterations per epoch: `10`
    - completed at: `2026-04-29T09:55:59Z`
    - curated export: `docs/benchmarking/data/policy_supply1000_provchain_fabric_e2e_n30_20260429/`
    - publication report now includes `fairness_label=cross-model-with-caveat` for ProvChain policy rows and separates server-reported decision latency from client-observed API round-trip latency
    - superseded Fabric-only workload pack moved to `docs/benchmarking/data/reference/policy_supply1000_fabric_pack_n30_20260428/`
    - superseded decision-core-only comparative export moved to `docs/benchmarking/data/reference/policy_supply1000_provchain_fabric_n30_20260429/`
  - local validation:
    - `cargo check --bin provchain-org` passed with existing warnings only
    - `cargo check --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml` passed with existing warning only
    - `bash -n benchmark-toolkit/scripts/run-fabric-ledger-campaign.sh` passed
    - `bash -n benchmark-toolkit/scripts/provchain-fabric-campaign.sh` passed
    - `bash -n benchmark-toolkit/scripts/provchain-fabric-policy-campaign.sh` passed

Post-result interpretation:
  - ProvChain policy mean around `0.001 ms` is server-side decision-function latency, not full HTTP/API round-trip latency
  - The value is low because the current parity endpoint performs deterministic string/visibility checks in the handler and does not touch RDF parsing, SHACL validation, state-root computation, block creation, WAL/fsync, or ledger lookup
  - Fabric policy mean around `6.264`-`6.359 ms` measures gateway-side `EvaluateTransaction("CheckPolicy", ...)`, including chaincode execution and ledger-state read of the policy record
  - This is good evidence for low-cost policy decision checks, but not proof that ProvChain and Fabric policy paths have identical ledger-state semantics

Post-R004 measurement hardening:
  - [x] Add client-observed `policy-check-latency-ms` rows for both systems in future policy campaigns
  - [x] Reuse one ProvChain benchmark token per policy campaign run so the client-observed policy row measures the authenticated API request, not repeated login
  - [x] Rerun comparative policy smoke/full campaign to collect the added end-to-end policy rows

### R005 - Add scale-up confidence campaigns

- Priority: `P2`
- Status: `Done for trace-query scale-up confidence - larger-slice profile passed and curated reference evidence exported; larger-slice bulk-import scale-up is optional future evidence, not required for current paper claims`
- Goal: ตรวจว่า ranking และ bottleneck ไม่เปลี่ยนเมื่อ dataset ใหญ่ขึ้น
- Tasks:
  - [x] เพิ่ม deterministic larger dataset slices สำหรับ `supply_chain_5000` และ `supply_chain_10000`
  - [x] เพิ่ม manifest/checksum/evidence-role metadata สำหรับ scale-up inputs
  - [x] ปรับ trace campaign manifest ให้บันทึก dataset checksum, bytes, evidence role, test batch IDs, และ Fluree translated output path ต่อ slice
  - [x] เพิ่ม one-command scale-up campaign wrapper พร้อม curated export และ isolated host port block
  - [x] เพิ่ม validity gate ให้ fail ถ้า `benchmark_results.json` มี result row ที่ `success=false`
  - [x] เพิ่ม Neo4j scale runtime tuning (`heap_max`, `pagecache`, `load_batch_size`) เพื่อแก้ scale smoke OOM
  - [x] rerun trace-query ที่ dataset มากกว่า `supply_chain_1000`
  - [x] document write/admission scale-up decision after R002 final mode selection: not required for current paper because R002 final is scoped to `supply_chain_1000` import-row remediation; run larger-slice bulk-import only if a later scaling claim is added
  - [x] report scaling trend แยกตาม family ที่มี validity gate ผ่าน
- Done when:
  - มีอย่างน้อยหนึ่ง larger-slice campaign ที่ผ่าน validity gate
Current implementation evidence:
  - generator: `benchmark-toolkit/scripts/generate-scale-dataset.py`
  - dataset docs: `benchmark-toolkit/datasets/README.md`
  - generated inputs:
    - `benchmark-toolkit/datasets/supply_chain_5000.ttl`
    - `benchmark-toolkit/datasets/supply_chain_5000.manifest.json`
    - `benchmark-toolkit/datasets/supply_chain_10000.ttl`
    - `benchmark-toolkit/datasets/supply_chain_10000.manifest.json`
  - `benchmark-toolkit/scripts/run-trace-campaign.sh` now records scale-up evidence metadata and passes `TEST_BATCH_IDS`
  - `benchmark-toolkit/scripts/provchain-neo4j-fluree-scale-campaign.sh` now wraps smoke/profile/full scale-up campaigns with `scale_up_confidence_not_primary_paper_evidence`, default `supply_chain_5000`, automatic reference export, and default host port block starting at `18280`
  - first scale smoke:
    - campaign: `smoke_trace_supply5000_provchain-neo4j-fluree_n1_20260429`
    - status file reported `passed`, but it is rejected as evidence because Neo4j load row had `success=false`
    - root cause: Neo4j `OutOfMemoryError` during `Turtle to Cypher Import` with the default `1G` heap
    - code remediation: future scale runs default Neo4j to `heap_initial=1G`, `heap_max=2G`, `pagecache=1G`, and `NEO4J_LOAD_BATCH_SIZE=25`; campaign manifests record these values
    - gate remediation: future trace campaigns mark an epoch failed if any result row has `success=false`
  - fixed scale smoke:
    - campaign: `smoke_trace_supply5000_provchain-neo4j-fluree_n1_20260429_fix1`
    - status: `passed`, `1/1` epoch
    - Neo4j runtime: `heap_initial=1G`, `heap_max=2G`, `pagecache=1G`, `load_batch_size=25`
    - load rows: ProvChain `372474.000 ms`, Neo4j `102980.000 ms`, Fluree `1162.000 ms`; all `100%` success
    - trace-query rows: ProvChain remains fastest in all three reported query scenarios on `supply_chain_5000`
    - evidence status: smoke gate only; profile `n3` still required before writing scaling trend
  - clean scale profile:
    - campaign: `20260429_scale_trace_supply5000_provchain-neo4j-fluree_n3`
    - status: `passed`, `3/3` epochs
    - completed at: `2026-04-29T17:26:41Z`
    - curated export: `docs/benchmarking/data/reference/scale_trace_supply_chain_5000_provchain_neo4j_fluree_n3_20260429/`
    - analysis note: `docs/benchmarking/data/reference/TRACE_SCALE_UP_CONFIDENCE_20260430.md`
    - result: ProvChain remains fastest on all trace-query scenarios at `supply_chain_5000`
    - means: Simple Product Lookup `0.660 ms`, Multi-hop Traceability `0.767 ms`, Aggregation by Producer `1.497 ms`
    - scale ratios versus `supply_chain_1000`: Simple `1.35x`, Multi-hop `1.60x`, Aggregation `2.65x`
    - load/import caveat: ProvChain `Turtle RDF Import` mean `367152.333 ms`, so this evidence must not be used as a ledger/write or bulk-load superiority claim
  - evidence hygiene follow-up:
    - a later rerun reused the same campaign id and produced a mixed aggregate with old and new run directories
    - that duplicate aggregate is not used as curated evidence
    - `run-trace-campaign.sh` now refuses non-empty campaign directories, and `export-campaign-evidence.sh` rejects epochs with more than one run directory
  - `benchmark-toolkit/docker-compose.trace.yml` now passes `--test-batch-ids` into the benchmark runner
  - local validation:
    - `python3 -m py_compile benchmark-toolkit/scripts/generate-scale-dataset.py` passed
    - scale slices were generated successfully
    - JSON-LD translation validation could not run in the agent host because `rdflib` is not installed outside the benchmark-runner container

### R006 - Add GraphDB / TigerGraph comparator coverage

- Priority: `P1` for GraphDB, `P2` for TigerGraph
- Status: `Done`
- Plan: [GRAPHDB_TIGERGRAPH_COMPARATOR_TASK_LIST_2026-04-30.md](./GRAPHDB_TIGERGRAPH_COMPARATOR_TASK_LIST_2026-04-30.md)
- Goal: ขยาย graph database comparator coverage ให้ตรงกับ thesis proposal โดยไม่ rerun benchmark หลักทั้งหมด
- Tasks:
  - [x] สร้าง task list เฉพาะ GraphDB/TigerGraph
  - [x] เพิ่ม GraphDB runtime feasibility gate tooling
  - [x] run GraphDB live feasibility smoke ใน shell ที่เข้าถึง Docker daemon ได้
  - [x] rerun GraphDB live feasibility smoke with default `ontotext/graphdb:10.8.13`
  - [ ] rerun GraphDB live feasibility smoke with `GRAPHDB_LICENSE_FILE` only if GraphDB 11.x evidence is required
  - [x] เพิ่ม GraphDB adapter contract
  - [x] เพิ่ม GraphDB trace-query parity contract
  - [x] เพิ่ม one-command GraphDB-inclusive campaign wrapper
  - [x] run GraphDB profile campaign พร้อม curated reference evidence
  - [x] run GraphDB full campaign พร้อม curated publication-facing evidence
  - [x] เพิ่ม TigerGraph feasibility gate tooling
  - [x] run TigerGraph live feasibility smoke ก่อนตัดสินใจ implement adapter
  - [x] เพิ่ม TigerGraph translated-model adapter contract
  - [x] เพิ่ม TigerGraph generated CSV/GSQL loader artifacts
  - [x] verify TigerGraph live translated-model install ใน Docker-enabled shell
  - [x] run TigerGraph translated-model smoke campaign
  - [x] run TigerGraph translated-model profile campaign
  - [x] run TigerGraph translated-model full campaign
  - [x] update paper ด้วย GraphDB passed curated evidence
  - [x] update paper ด้วย TigerGraph only if translated-model campaign evidence passes
- Done when:
  - GraphDB full campaign ผ่าน validity gate และ export เข้า `docs/benchmarking/data/reference/`
  - TigerGraph มี either passed campaign evidence หรือ documented deferred decision
  - publication benchmark report และ paper provenance map ระบุ GraphDB/TigerGraph claim boundary ชัดเจน

## Required Next Order

1. `R001` ก่อน เพราะเป็น paper/report correctness - `Done`
2. `R002` ต่อทันที เพราะเป็นจุดอ่อนหลักของ ProvChain จากผล benchmark - `Done for bulk dataset-admission/import-row claim; per-transaction ledger/write remains caveated`
3. `R003` เพื่อทำ semantic claim ให้ defensible
4. `R004` เพื่อปิด policy parity gap - `Done`
5. `R005` trace-query scale-up confidence - `Done`; larger-slice bulk-import scale-up remains optional future evidence
6. `R006` GraphDB ก่อน TigerGraph เพื่อเพิ่ม comparator coverage ตาม thesis proposal; paper update หลัง GraphDB/TigerGraph evidence ผ่านแล้วเท่านั้น

## Current Claim Boundary

- claim ได้ว่า ProvChain แข็งแรงมากใน trace-query/provenance workloads
- claim ได้ว่า trace-query ranking ยัง stable ใน reference scale-up profile `supply_chain_5000`
- claim ได้ว่า benchmark harness หลาย product พร้อมระดับ artifact-backed
- claim ได้ว่า ProvChain/Fabric policy scenarios ผ่านครบใน comparative governance-policy campaign โดยต้องระบุ `cross-model-with-caveat`
- claim ได้ว่า R002 bulk Turtle dataset admission แก้ปัญหา import-row เดิมอย่างมีหลักฐาน: final campaign `20260430_import_supply1000_provchain-bulk-r002_final_n30` ผ่าน `30/30`, ProvChain import mean `24.333 ms` เทียบ legacy same-slice baseline `12124.033 ms` (`~498.26x` faster, `99.80%` lower)
- ยังไม่ควร claim ว่า ProvChain ชนะด้าน ledger/write throughput หรือ semantic admission latency
- ยังไม่ควรใช้ R002 bulk import เป็นหลักฐานแทน per-transaction ledger/write throughput หรือ finality
- ยังไม่ควร claim ว่า ProvChain policy path เป็น native-comparable กับ Fabric ledger-state policy path แบบไม่มี caveat
- claim ได้ว่า GraphDB เป็น RDF/SPARQL-native comparator สำหรับ trace-query จาก campaign `20260501_trace_supply1000_provchain-neo4j-fluree-graphdb_n30` ที่ผ่าน `30/30`
- claim ได้ว่า TigerGraph เป็น optional translated property-graph comparator สำหรับ trace-query จาก campaign `20260502_trace_supply1000_provchain-neo4j-tigergraph_n30` ที่ผ่าน `30/30`
- ยังไม่ควร claim TigerGraph เป็น RDF-native, semantic-validation, ledger-write, หรือ ontology comparator
