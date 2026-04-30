# Benchmark Status Summary - 2026-04-23

## Purpose

เอกสารนี้สรุปสถานะ benchmark ปัจจุบันแบบสั้น เพื่อบอกว่า:

- อะไรพร้อมใช้งานแล้ว
- อะไรยังถูก block
- อะไรคือ baseline ที่เชื่อถือได้ตอนนี้
- ถัดไปควรทำอะไร

## Current Valid Baseline

baseline ที่พร้อมใช้งานจริงตอนนี้มี 4 track แยก benchmark family และ capability path กัน:

- `ProvChain vs Neo4j`
- benchmark family: `trace_query`
- `ProvChain vs Fabric`
- benchmark family: `ledger_write`
- `Fabric`
- benchmark family: `governance_policy`
- `ProvChain vs Geth`
- benchmark family: `ledger_write`
- boundary: Geth เป็น `public-chain-baseline` ไม่ใช่ permissioned ledger baseline

สถานะนี้ถือว่าใช้ได้จริง และตอนนี้มีทั้ง single-run baseline เดิมกับ multi-epoch campaign ใหม่ เพราะ:

- local correctness gate ผ่านแล้ว
- local benchmark-query contract tests ผ่านแล้ว
- Docker trace stack รันจบได้จริง
- ผล benchmark มี artifact เก็บครบ
- campaign 30 epoch ผ่านครบและ aggregate artifacts ถูกสร้างแล้ว

run ที่ใช้อ้างอิงล่าสุด:

- [20260422T154555Z](/home/cit/provchain-org/benchmark-toolkit/results/trace/20260422T154555Z)

baseline นี้ถูกจัดเข้า campaign layout แล้วเพื่อให้ผลเดิมอยู่ในรูปแบบเดียวกับ campaign หลาย epoch ในอนาคต:

- [baseline_trace_supply1000_provchain-neo4j_20260422T154555Z_n1](/home/cit/provchain-org/benchmark-toolkit/results/campaigns/baseline_trace_supply1000_provchain-neo4j_20260422T154555Z_n1)

multi-epoch campaign ล่าสุด:

- [20260424_trace_supply1000_provchain-neo4j_n30](/home/cit/provchain-org/benchmark-toolkit/results/campaigns/20260424_trace_supply1000_provchain-neo4j_n30)
- status: `passed`
- epochs: `30/30`
- iterations per epoch: `10`
- aggregate files:
  - `campaign_results.json`
  - `campaign_results.csv`
  - `campaign_aggregate_summary.md`

ข้อจำกัด:

- ผล trace-query ยังเป็น `ProvChain vs Neo4j` เท่านั้น
- ผล trace-query ใหม่มี `ProvChain vs Neo4j vs Fluree` แล้ว แต่ควรรายงานเป็น campaign แยกจาก baseline 2026-04-24
- ผล ledger-write มีทั้ง `ProvChain vs Fabric` สำหรับ permissioned-ledger evidence และ `ProvChain vs Geth` สำหรับ public-chain baseline evidence แต่ต้องรายงานเป็นคนละ campaign/capability path
- ผล governance-policy ยังเป็น `Fabric` เท่านั้น
- ยังไม่ใช่ single global multi-product winner table และไม่ควรรวมข้าม family/capability path
- write/load rows ควรตีความตาม fairness label และไม่ควรใช้แทน ledger-finality comparison

ledger-write campaign ล่าสุด:

- [20260425_ledger_supply1000_provchain-fabric_managed_n30_fix3](/home/cit/provchain-org/benchmark-toolkit/results/campaigns/20260425_ledger_supply1000_provchain-fabric_managed_n30_fix3)
- status: `passed`
- epochs: `30/30`
- iterations per epoch: `10`
- curated export: [ledger_supply1000_provchain_fabric_managed_n30_20260425](./data/ledger_supply1000_provchain_fabric_managed_n30_20260425/)

governance-policy campaign ล่าสุด:

- [20260425_policy_supply1000_fabric_n30](/home/cit/provchain-org/benchmark-toolkit/results/campaigns/20260425_policy_supply1000_fabric_n30)
- status: `passed`
- epochs: `30/30`
- iterations per epoch: `10`
- curated export: [policy_supply1000_fabric_n30_20260425](./data/reference/policy_supply1000_fabric_n30_20260425/)

comparative governance-policy campaign ล่าสุด:

- [20260429_policy_supply1000_provchain-fabric_e2e_n30](/home/cit/provchain-org/benchmark-toolkit/results/campaigns/20260429_policy_supply1000_provchain-fabric_e2e_n30)
- status: `passed`
- epochs: `30/30`
- iterations per epoch: `10`
- curated export: [policy_supply1000_provchain_fabric_e2e_n30_20260429](./data/policy_supply1000_provchain_fabric_e2e_n30_20260429/)
- scenarios: `authorized-read`, `auditor-read`, `unauthorized-read`, `authorized-write`, `rejected-write`
- aggregate client-observed policy API round-trip means:
  - Fabric: authorized-read `6.831 ms`, auditor-read `6.552 ms`, unauthorized-read `6.366 ms`, authorized-write `6.317 ms`, rejected-write `6.231 ms`
  - ProvChain: authorized-read `2.249 ms`, auditor-read `1.947 ms`, unauthorized-read `1.923 ms`, authorized-write `1.870 ms`, rejected-write `1.864 ms`
- aggregate server-reported decision means:
  - Fabric: `5.167`-`5.636 ms`
  - ProvChain: `0.001`-`0.002 ms`
- caveat: ProvChain rows are `cross-model-with-caveat`; Fabric reads policy from ledger state while the current ProvChain parity endpoint receives policy context in the request

reference policy workload pack campaign:

- [20260428_policy_supply1000_fabric_pack_n30](/home/cit/provchain-org/benchmark-toolkit/results/campaigns/20260428_policy_supply1000_fabric_pack_n30)
- status: `passed`
- epochs: `30/30`
- iterations per epoch: `10`
- curated export: [policy_supply1000_fabric_pack_n30_20260428](./data/reference/policy_supply1000_fabric_pack_n30_20260428/)
- scenarios: `authorized-read`, `auditor-read`, `unauthorized-read`, `authorized-write`, `rejected-write`
- aggregate means: authorized-read `5.646 ms`, auditor-read `5.282 ms`, unauthorized-read `5.241 ms`, authorized-write `5.200 ms`, rejected-write `5.195 ms`

trace-query campaign with Fluree ล่าสุด:

- [20260428_trace_supply1000_provchain-neo4j-fluree_n30](/home/cit/provchain-org/benchmark-toolkit/results/campaigns/20260428_trace_supply1000_provchain-neo4j-fluree_n30)
- status: `passed`
- epochs: `30/30`
- iterations per epoch: `10`
- curated export: [trace_supply1000_provchain_neo4j_fluree_n30_20260428](./data/trace_supply1000_provchain_neo4j_fluree_n30_20260428/)

semantic campaign with Fluree ล่าสุด:

- [20260428_semantic_supply1000_provchain-fluree_n30](/home/cit/provchain-org/benchmark-toolkit/results/campaigns/20260428_semantic_supply1000_provchain-fluree_n30)
- status: `passed`
- epochs: `30/30`
- iterations per epoch: `1`
- curated export: [semantic_supply1000_provchain_fluree_n30_20260428](./data/semantic_supply1000_provchain_fluree_n30_20260428/)
- boundary: ProvChain uses native RDF+SHACL admission; Fluree uses externalized JSON-LD admission pipeline

ledger-write campaign with Geth ล่าสุด:

- [20260428_ledger_supply1000_provchain-geth_n30_fix1](/home/cit/provchain-org/benchmark-toolkit/results/campaigns/20260428_ledger_supply1000_provchain-geth_n30_fix1)
- status: `passed`
- epochs: `30/30`
- iterations per epoch: `10`
- curated export: [ledger_supply1000_provchain_geth_n30_20260428](./data/ledger_supply1000_provchain_geth_n30_20260428/)
- boundary: Geth uses public-chain smart-contract execution; do not report it as permissioned-ledger parity evidence

publication report bundle ล่าสุด:

- [PUBLICATION_BENCHMARK_REPORT_BUNDLE_2026-04-28.md](./PUBLICATION_BENCHMARK_REPORT_BUNDLE_2026-04-28.md)
- generated from curated trace, ledger, semantic, policy, and public-chain baseline exports
- includes evidence-source table, family-specific result tables, semantic capability notes, and fairness/limitations
- does not generate a single global winner table

post-round remediation task list:

- [BENCHMARK_POST_ROUND_REMEDIATION_TASK_LIST_2026-04-29.md](./BENCHMARK_POST_ROUND_REMEDIATION_TASK_LIST_2026-04-29.md)
- required order: report semantics cleanup, ProvChain write/admission profiling, semantic cost profiling, ProvChain policy parity, then scale-up campaigns
- latest R002 batch-block diagnostic:
  - campaign: [20260429_profile_ledger_supply1000_provchain-only_batchblock_n3](/home/cit/provchain-org/benchmark-toolkit/results/campaigns/20260429_profile_ledger_supply1000_provchain-only_batchblock_n3)
  - curated export: [profiling_ledger_supply1000_provchain_only_batchblock_n3_20260429](./data/reference/profiling_ledger_supply1000_provchain_only_batchblock_n3_20260429/)
  - diagnostic note: [PROVCHAIN_LEDGER_BATCH_BLOCK_PROFILE_20260429.md](./data/reference/PROVCHAIN_LEDGER_BATCH_BLOCK_PROFILE_20260429.md)
  - result: same-campaign `100 tx` mean `6434.444 ms`; diagnostic `100 triples, 1 block` mean `1646.265 ms`
  - caveat: this is relaxed-durability, ProvChain-only, batch-semantics profiling evidence, not primary paper comparison evidence
- latest R002 cold-load vs steady-state profile:
  - campaign: [20260429_profile_ledger_supply1000_provchain-only_coldsteady_conservative_n3](/home/cit/provchain-org/benchmark-toolkit/results/campaigns/20260429_profile_ledger_supply1000_provchain-only_coldsteady_conservative_n3)
  - curated export: [20260429_profile_ledger_supply1000_provchain_only_coldsteady_conservative_n3](./data/reference/20260429_profile_ledger_supply1000_provchain_only_coldsteady_conservative_n3/)
  - diagnostic note: [PROVCHAIN_LEDGER_COLD_STEADY_PROFILE_20260429.md](./data/reference/PROVCHAIN_LEDGER_COLD_STEADY_PROFILE_20260429.md)
  - status: `passed`, `3/3` epochs, conservative WAL/index sync every block
  - result: cold `Turtle RDF Import` mean `50736.000 ms`; post-load `Steady-state Append After Cold Load (100 tx)` mean `17972.444 ms`
  - caveat: this is ProvChain-only profiling/reference evidence; it separates phases but does not support a cross-system ledger/write win claim
- R003 semantic admission profiling status:
  - instrumentation has been added for ProvChain semantic client stages and Fluree externalized translation/read/parse/insert stages
  - ProvChain server-side semantic admission timing now splits ontology validation, SHACL data-store create, SHACL RDF parse/load, SHACL shape validation, explanation summary, state-root/hash/sign, RDF store insert/cache, metadata insert, chain push, and persistence when `PROVCHAIN_BENCHMARK_STAGE_TIMINGS=true`
  - semantic Docker startup has been corrected to use `config/ontology_package.toml` and `--skip-demo-data` for benchmark runs
  - latest semantic smoke `smoke_semantic_supply1000_provchain-fluree_profile_n1_20260429` produced no benchmark rows because Docker could not bind host port `18080`; the semantic wrapper now defaults to an isolated host port block starting at `18180`, records the port set in campaign manifests, and exports passing `full`/`run` campaigns automatically
  - follow-up smoke `smoke_semantic_supply1000_provchain-` passed `1/1` with the isolated port block; use it only as a smoke gate because the id was truncated by shell line wrapping after `provchain-`
  - smoke result: ProvChain native semantic admission `7172.000 ms`; Fluree externalized admission `906.389 ms`; ProvChain client submit loop was `6513.138 ms`, while read/normalize/parse was only `12.542 ms` combined
  - campaign wrappers now reject suspicious ids that end with punctuation to prevent truncated-id evidence
  - profile campaign `20260429_profile_semantic_supply1000_provchain-fluree_n3` passed `3/3` and was exported to [profiling_semantic_supply1000_provchain_fluree_n3_20260429](./data/reference/profiling_semantic_supply1000_provchain_fluree_n3_20260429/)
  - semantic profile note: [PROVCHAIN_SEMANTIC_ADMISSION_PROFILE_20260429.md](./data/reference/PROVCHAIN_SEMANTIC_ADMISSION_PROFILE_20260429.md)
  - profile result: ProvChain native RDF+SHACL admission mean `7109.333 ms`; Fluree externalized TTL+JSON-LD admission mean `872.691 ms`
  - ProvChain server-side breakdown: handler mean `9.796 ms/record`, block admission `9.358 ms/record`, state root `4.396 ms/record`, persistence `4.469 ms/record`, ontology+SHACL validation about `0.620 ms/record`
  - interpretation: semantic validation itself is not the dominant cost; current ProvChain admission latency is dominated by per-record API/block admission, state root, and persistence
- R005 scale-up confidence status:
  - deterministic larger trace-query input slices now exist for `supply_chain_5000` and `supply_chain_10000`
  - generated slice manifests record generator, logical batch count, byte size, checksum, evidence role, and test batch IDs
  - trace campaign manifests now record dataset checksum/bytes/evidence role and pass explicit `TEST_BATCH_IDS` through Docker
  - scale-up campaigns now have a one-command wrapper: `benchmark-toolkit/scripts/provchain-neo4j-fluree-scale-campaign.sh`
  - the wrapper defaults to `supply_chain_5000`, `ProvChain+Neo4j+Fluree`, reference-only evidence role, curated reference export, and host ports starting at `18280`
  - first scale smoke `smoke_trace_supply5000_provchain-neo4j-fluree_n1_20260429` is rejected as evidence even though the status file said `passed`, because Neo4j `Turtle to Cypher Import` had `success=false`
  - root cause: Neo4j `OutOfMemoryError` during load with default `1G` heap
  - remediation: scale wrapper now defaults Neo4j to `heap_initial=1G`, `heap_max=2G`, `pagecache=1G`, and `NEO4J_LOAD_BATCH_SIZE=25`; trace campaigns now fail the epoch if any result row has `success=false`
  - fixed scale smoke `smoke_trace_supply5000_provchain-neo4j-fluree_n1_20260429_fix1` passed `1/1`; all load and trace-query rows have `100%` success
  - fixed smoke load means: ProvChain `372474.000 ms`, Neo4j `102980.000 ms`, Fluree `1162.000 ms`
  - fixed smoke trace-query means confirm ProvChain remains fastest on `supply_chain_5000`, but this is still a smoke gate; profile `n3` is required before reporting scaling trend
  - clean scale profile `20260429_scale_trace_supply5000_provchain-neo4j-fluree_n3` passed `3/3` and is exported to [scale_trace_supply_chain_5000_provchain_neo4j_fluree_n3_20260429](./data/reference/scale_trace_supply_chain_5000_provchain_neo4j_fluree_n3_20260429/)
  - scale-up analysis note: [TRACE_SCALE_UP_CONFIDENCE_20260430.md](./data/reference/TRACE_SCALE_UP_CONFIDENCE_20260430.md)
  - clean profile trace-query means: ProvChain Simple `0.660 ms`, Multi-hop `0.767 ms`, Aggregation `1.497 ms`
  - clean profile scale ratios versus `supply_chain_1000`: ProvChain Simple `1.35x`, Multi-hop `1.60x`, Aggregation `2.65x`
  - evidence hygiene note: a later rerun reused the same campaign id and mixed old/new run rows, so that duplicate aggregate is not used; scripts now reject non-empty campaign directories and multi-run epoch exports
  - scale-up results remain reference/confidence evidence, not primary paper evidence, unless a later publication decision promotes larger-slice campaigns explicitly

## What We Can Say Now

ภายใต้ benchmark harness ปัจจุบัน:

- `ProvChain` ผ่านทุก trace-query scenario
- `Neo4j` ผ่านทุก trace-query scenario
- ใน campaign `20260424_trace_supply1000_provchain-neo4j_n30` ทั้งสองระบบมี `success_rate = 100%` ใน trace-query scenarios
- `ProvChain` เร็วกว่า `Neo4j` ใน 3 query scenarios ที่วัด
- data loading / ingestion path ต้องแยกตีความจาก trace-query และดู fairness label ประกอบ

ข้อสรุปที่ปลอดภัย:

- baseline `ProvChain vs Neo4j` ใช้เป็นหลักฐาน multi-epoch สำหรับ `Trace Query / Provenance` ได้
- baseline `ProvChain vs Fabric` ใช้เป็นหลักฐาน multi-epoch สำหรับ `Ledger / Write Path` ได้
- baseline `ProvChain vs Geth` ใช้เป็นหลักฐาน multi-epoch สำหรับ `Ledger / Write Path` ได้เฉพาะฐานะ public-chain baseline
- baseline `Fabric` ใช้เป็นหลักฐาน multi-epoch สำหรับ `Governance / Policy` ได้เฉพาะ Fabric policy path
- Fabric policy workload pack ใช้เป็น scenario pack กลางสำหรับ `Governance / Policy` ได้ โดยมี read/write allow/reject ครบ
- comparative `ProvChain vs Fabric` policy parity path ถูก implement แล้ว แต่ยังต้อง rerun smoke/full campaign ก่อนนำไปใช้เป็น evidence
- ยังไม่ควรอ้างเป็น global winner

## Current Blockers

### Fluree

สถานะ:

- image ถูก pin ด้วย digest แล้ว
- local adapter tests ผ่าน
- runtime boot ผ่านและ health ผ่าน
- runtime contract ถูกยืนยันกับ current API family แล้ว:
  - `/index.html`
  - `/fluree/create`
  - `/fluree/transact`
  - `/fluree/query`
- trace-query campaign `ProvChain + Neo4j + Fluree` ผ่านครบ `30/30`

ข้อสรุป:

- `Fluree` ไม่ block สำหรับ trace-query family แล้ว
- semantic family evidence is now available with native/externalized capability fields

### Fabric

สถานะ:

- topology contract และ canonical record mapping ถูกนิยามแล้ว
- adapter มี write, batch, commit-metric response parsing, และ policy-check contract methods
- local mock gateway tests ผ่านแล้ว
- local contract smoke ผ่านด้วย simulator และสร้าง artifact ได้จริง
- real Fabric runtime/gateway ledger campaign ผ่านแล้วสำหรับ `B013`
- real Fabric runtime/gateway policy campaign ผ่านแล้วสำหรับ `B014`
- Fabric policy workload pack campaign ผ่านแล้วสำหรับ `B019` พร้อม read/write allow/reject scenarios

ข้อสรุป:

- `Fabric` อยู่สถานะ `ledger and policy evidence ready`
- ใช้เป็น benchmark evidence เทียบกับ `ProvChain` ได้เฉพาะ family `ledger_write`
- family `policy` มี comparative runner path แล้ว แต่ campaign ใหม่ยังไม่ถูก validate/export

### Geth

สถานะ:

- workload contract, adapter contract tests, local dev-chain orchestration, and campaign wrapper are implemented
- Geth dev chain runs in its own Compose project (`provchain-geth`) to avoid trace-stack orphan cleanup
- full local Geth campaign `20260428_ledger_supply1000_provchain-geth_n30_fix1` passed `30/30`
- aggregate rows separate submit latency, confirmation latency, and gas used

ข้อสรุป:

- `Geth` is ready only as a `public-chain-baseline` for ledger/write-path evidence
- `Geth` must not be used as a permissioned enterprise ledger, trace-query, or semantic comparator

## Benchmark Program Status

### Ready

- `ProvChain vs Neo4j`
- trace-query family
- timestamped result directories
- environment manifest per run
- local preflight gate
- fairness/capability labels in result schema
- campaign directory layout
- campaign aggregation helper
- historical single-run promotion helper
- one-command ProvChain vs Neo4j campaign wrapper
- completed 30-epoch statistical campaign for `ProvChain vs Neo4j` trace-query baseline
- Fabric topology/mapping/local contract gate
- completed 30-epoch statistical campaign for `ProvChain vs Fabric` ledger-write baseline
- completed 30-epoch statistical campaign for `Fabric` governance-policy baseline
- completed 30-epoch statistical campaign for `ProvChain vs Neo4j vs Fluree` trace-query evidence
- completed 30-epoch statistical campaign for `ProvChain vs Fluree` semantic admission evidence
- completed 30-epoch statistical campaign for `ProvChain vs Geth` ledger-write public-chain baseline evidence
- completed 30-epoch statistical campaign for the Fabric governance-policy workload pack
- publication-ready multi-family report bundle for the completed benchmark round
- post-round remediation task list for paper correctness and ProvChain performance follow-up
- publication report semantics cleanup for load/import/reference metric separation
- ProvChain ledger write profiling artifact for R002 bottleneck analysis
- ProvChain-only R002 remediation profiling after RDF snapshot flush batching, state-root cache optimization, relaxed WAL/index fsync batching, batch-block diagnostics, and conservative cold-load/steady-state phase separation
- comparative `ProvChain vs Fabric` governance-policy evidence with client-observed policy API round-trip rows
- R003 semantic admission cost profile showing semantic validation is not the dominant ProvChain cost
- R005 reference scale-up confidence profile for `supply_chain_5000` trace-query ranking stability

### Partially Ready

- ProvChain write/admission performance claim: evidence exists and profiling identifies `block_admission` as the current bottleneck; RDF snapshot flush batching plus state-root cache optimization improved the ProvChain-only conservative profile by about `49.77%` on batch total and `60.92%` on server `block_admission`; relaxed WAL/index fsync profiling reaches `50.48%` and `61.51%` cumulative improvement, and the conservative cold-load/steady-state profile shows post-load append mean `17972.444 ms` per `100 tx`; comparative/full reruns are still required before making competitiveness claims
- semantic admission latency claim: cost breakdown exists and supports a tradeoff explanation, but current results do not support a ProvChain latency superiority claim
- scale-up primary publication claim: reference evidence exists for `supply_chain_5000`, but primary paper tables still use the completed `supply_chain_1000` n30 campaign unless larger-slice campaigns are explicitly promoted

### Not Ready

- optimized ProvChain write/admission campaign after R002 remediation
- ledger/write scale-up evidence after R002 final durability/batching mode selection

## Recommended Next Actions

1. `R001` report semantics cleanup เสร็จแล้ว
2. ทำ `R002` ProvChain write/admission optimization ต่อจาก profiling artifacts ที่ระบุ `block_admission` เป็นคอขวดหลัก; cold-load/steady-state phase separation เสร็จใน conservative `n3` profile แล้ว และ next target คือ full/comparative rerun เมื่อ fix durability/batching modes ชัดเจน
3. `R003` semantic admission cost breakdown เสร็จแล้ว
4. `R004` ProvChain policy parity workload เสร็จแล้วด้วย campaign `20260429_policy_supply1000_provchain-fabric_e2e_n30`
5. `R005` trace-query scale-up confidence เสร็จแล้วด้วย campaign `20260429_scale_trace_supply5000_provchain-neo4j-fluree_n3`; ledger/write scale-up ยังรอ R002 final mode

## Decision Summary

ถ้าต้องใช้ benchmark ตอนนี้:

- ใช้ `ProvChain vs Neo4j`
- ใช้ `ProvChain vs Neo4j vs Fluree` ได้เฉพาะ trace-query family campaign `20260428_trace_supply1000_provchain-neo4j-fluree_n30`
- ใช้ R005 scale-up profile `20260429_scale_trace_supply5000_provchain-neo4j-fluree_n3` ได้เป็น reference/confidence evidence ว่า trace-query ranking ยัง stable บน `supply_chain_5000`
- ใช้ `ProvChain vs Fluree` semantic evidence ได้เฉพาะ semantic admission campaign `20260428_semantic_supply1000_provchain-fluree_n30`
- ใช้ `ProvChain vs Fabric` ได้เฉพาะ `ledger_write`
- ใช้ `ProvChain vs Geth` ได้เฉพาะ `ledger_write` ในฐานะ public-chain baseline campaign `20260428_ledger_supply1000_provchain-geth_n30_fix1`
- ใช้ `ProvChain vs Fabric` governance-policy evidence ได้จาก campaign `20260429_policy_supply1000_provchain-fabric_e2e_n30` โดยต้องระบุ `cross-model-with-caveat` สำหรับ ProvChain rows
- ใช้ [PUBLICATION_BENCHMARK_REPORT_BUNDLE_2026-04-28.md](./PUBLICATION_BENCHMARK_REPORT_BUNDLE_2026-04-28.md) เป็นรายงานรวมระดับ publication สำหรับ benchmark round นี้
- ระบุชัดว่า ProvChain policy comparison เปิดแล้วเฉพาะ scenario-level parity และไม่ใช่ native ledger-state policy equivalence แบบไม่มี caveat

ถ้าจะกลับไปพัฒนา benchmark program ต่อ:

- ให้ใช้ [MULTI_PRODUCT_BENCHMARK_BACKLOG_2026-04-22.md](./MULTI_PRODUCT_BENCHMARK_BACKLOG_2026-04-22.md)
- และ [FLUREE_TRACE_ADAPTER_CONTRACT_2026-04-22.md](./FLUREE_TRACE_ADAPTER_CONTRACT_2026-04-22.md) เป็นจุดอ้างอิงหลัก
