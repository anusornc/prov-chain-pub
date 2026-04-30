# Benchmark Results Organization Plan - 2026-04-24

## Purpose

เอกสารนี้กำหนดวิธีจัดเก็บผล benchmark ให้รองรับการรันหลายรอบ หลาย epoch และหลาย campaign โดยไม่ปนกัน

เป้าหมายคือ:

- หา run เดิมได้ง่าย
- รู้ว่า run ไหนเป็นหลักฐานที่ใช้ตีความได้
- แยกผลทดลองจริงออกจากเอกสารแผน
- ลดจำนวนเอกสารที่ต้องเปิดอ่านเมื่อทำงานประจำวัน

## Current Problem

ตอนนี้เอกสาร benchmark มีหลายไฟล์ เพราะมีทั้ง:

- แผนวิธีวัด
- แผนสร้าง adapter
- สถานะปัจจุบัน
- สรุปสำหรับผู้บริหาร
- รายละเอียดเฉพาะ `Fluree`
- รายละเอียดเฉพาะ `Fabric`
- ผล benchmark จริง

ปัญหาไม่ใช่ว่ามีเอกสารเยอะอย่างเดียว แต่ยังไม่มีระดับการจัดหมวดที่ชัดว่าไฟล์ใดคือ:

- เอกสารใช้งานประจำ
- เอกสารอ้างอิงระยะยาว
- เอกสารประวัติการตัดสินใจ
- raw result
- campaign summary

## Decision

ให้แยก benchmark program เป็น 3 ชั้น:

| Layer | Purpose | Location |
|---|---|---|
| `Operational docs` | ใช้ตัดสินใจและรันงานประจำวัน | `docs/benchmarking/` |
| `Raw artifacts` | ผลลัพธ์จริงจากเครื่องมือ benchmark | `benchmark-toolkit/results/` |
| `Publication exports` | ชุดผลที่คัดแล้วสำหรับ paper/report | `docs/benchmarking/data/` หรือ artifact bundle ที่ระบุชัด |

## Active Documentation Set

เอกสารที่ควรเปิดอ่านก่อนสำหรับงาน benchmark ปัจจุบันมีเพียง 4 ไฟล์:

| Priority | File | Use |
|---|---|---|
| 1 | `BENCHMARK_STATUS_SUMMARY_2026-04-23.md` | ดูว่าสถานะล่าสุดคืออะไร อะไรพร้อม อะไร block |
| 2 | `BENCHMARK_METRICS_STRATEGY_2026-04-24.md` | ดูว่าจะวัดเมทริกอะไร และตีความอย่างไร |
| 3 | `BENCHMARK_RESULTS_ORGANIZATION_PLAN_2026-04-24.md` | ดูว่าจะเก็บผลหลาย run และหลาย epoch อย่างไร |
| 4 | `BENCHMARK_NEXT_ACTIONS_2026-04-23.md` | ดูงานถัดไป |

เอกสารอื่นให้ถือเป็น reference หรือ historical planning ไม่ใช่ไฟล์ที่ต้องอ่านทุกครั้ง

## Results Directory Model

โครงสร้างใหม่สำหรับผล benchmark:

```text
benchmark-toolkit/results/
├── README.md
├── campaigns/
│   └── <campaign_id>/
│       ├── campaign_manifest.json
│       ├── campaign_summary.md
│       ├── campaign_results.csv
│       ├── campaign_results.json
│       └── epochs/
│           └── epoch-001/
│               ├── epoch_manifest.json
│               └── runs/
│                   └── <run_id>/
│                       ├── environment_manifest.json
│                       ├── benchmark_results.json
│                       ├── benchmark_results.csv
│                       ├── summary.json
│                       ├── summary.md
│                       └── raw_logs/
└── trace/
    ├── <run_id>/
    └── latest
```

`trace/<run_id>` ยังเก็บไว้เพื่อ compatibility กับ harness ปัจจุบัน

เมื่อเริ่มทำ campaign หลาย epoch ให้ copy หรือ generate ผลเข้า `campaigns/<campaign_id>/epochs/...` ด้วย เพื่อให้ผลรันระยะยาวไม่กระจาย

## Naming Rules

### Campaign ID

รูปแบบ:

```text
YYYYMMDD_<family>_<dataset>_<products>_n<epoch_count>
```

ตัวอย่าง:

```text
20260424_trace_supply1000_provchain-neo4j_n30
20260425_ledger_supply1000_provchain-fabric_n30
20260426_semantic_uht_provchain-fluree_n20
```

### Run ID

ใช้ UTC timestamp:

```text
YYYYMMDDTHHMMSSZ
```

ตัวอย่าง:

```text
20260422T154555Z
```

### Epoch ID

ใช้เลขสามหลัก:

```text
epoch-001
epoch-002
epoch-003
```

## Required Files

### Campaign Manifest

`campaign_manifest.json` ต้องบอกว่า campaign นี้คืออะไร

ขั้นต่ำควรมี:

| Field | Meaning |
|---|---|
| `campaign_id` | รหัส campaign |
| `created_at_utc` | เวลาสร้าง |
| `benchmark_family` | family ที่วัด |
| `dataset_slice` | dataset ที่ใช้ |
| `products` | product ที่เข้าร่วม |
| `epoch_count_target` | จำนวน epoch ที่ตั้งใจรัน |
| `iterations_per_epoch` | จำนวน iteration ต่อ epoch |
| `validity_gate` | เงื่อนไขที่ต้องผ่านก่อนนับผล |
| `notes` | caveat สำคัญ |

### Epoch Manifest

`epoch_manifest.json` ต้องบอกว่า epoch นั้นรันภายใต้เงื่อนไขใด

ขั้นต่ำควรมี:

| Field | Meaning |
|---|---|
| `epoch_id` | เช่น `epoch-001` |
| `run_id` | run ที่ผูกกับ epoch |
| `started_at_utc` | เวลาเริ่ม |
| `completed_at_utc` | เวลาจบ |
| `status` | `passed`, `failed`, `partial`, `excluded` |
| `exclusion_reason` | เหตุผลถ้าไม่นับผล |

### Run Artifacts

ทุก run ต้องมีไฟล์เหล่านี้:

| File | Required | Meaning |
|---|---|---|
| `environment_manifest.json` | yes | เครื่อง, Docker image, config, dataset, commit |
| `benchmark_results.json` | yes | raw structured results |
| `benchmark_results.csv` | yes | ใช้วิเคราะห์ด้วย spreadsheet หรือ script |
| `summary.json` | yes | สรุป machine-readable |
| `summary.md` | yes | สรุปสำหรับอ่านเร็ว |
| `raw_logs/` | recommended | log จาก container หรือ runner |

## Result Status Labels

ทุก run และ epoch ต้องมีสถานะ:

| Status | Meaning |
|---|---|
| `passed` | ใช้ตีความได้ |
| `partial` | บางระบบหรือบาง scenario ผ่าน ต้องระบุ caveat |
| `failed` | รันไม่จบหรือผลใช้ไม่ได้ |
| `excluded` | รันจบแต่ไม่ควรใช้ เช่น config ผิด หรือระบบหนึ่ง `success_rate = 0` |
| `blocked` | adapter หรือ runtime contract ยังไม่พร้อม |

## What Counts As Evidence

ผลจะนับเป็นหลักฐานได้เมื่อ:

1. อยู่ใน campaign หรือ run directory ที่มี manifest ครบ
2. local preflight ผ่านก่อนรัน
3. ทุกระบบที่ถูกเปรียบเทียบใน scenario มี `success_rate > 0`
4. มี `environment_manifest.json`
5. มี `fairness_label`
6. มีการระบุว่าเป็น family ใด

ผลที่ยังไม่มี manifest หรือไม่มี success gate ให้ถือเป็น engineering debug artifact เท่านั้น

## Documentation Organization

เอกสารใน `docs/benchmarking/` ให้แบ่งบทบาทแบบนี้:

| Group | Files | Rule |
|---|---|---|
| Active control docs | status, metrics strategy, results organization, next actions | ใช้อ่านประจำ |
| Methodology references | fairness matrix, competitive spec, methodology plan | ใช้เมื่อแก้หลักการ benchmark |
| Product contracts | Fluree contract, Fabric topology, Fabric mapping | ใช้เมื่อทำ product นั้น |
| Historical artifacts | old summaries, old plans | ห้ามลบทันที แต่ไม่ใช่ entry point |
| Publication data | CSV, figures, curated outputs | ต้อง reproducible จาก raw artifacts |

## Proposed README Policy

หน้า `docs/benchmarking/README.md` ควรมีส่วนบนสุดเป็น `Start Here`:

1. current status
2. metrics strategy
3. results organization
4. next actions

หลังจากนั้นค่อยมี reference list ยาว

## Migration Plan

### Phase 1. No-risk organization

- เพิ่มเอกสาร organization plan
- เพิ่ม `benchmark-toolkit/results/README.md`
- เพิ่มลิงก์เข้า `docs/benchmarking/README.md` และ `docs/INDEX.md`
- ยังไม่ย้าย raw result เก่า

Historical single-run promotion helper:

- `benchmark-toolkit/scripts/promote-trace-run-to-campaign.sh`

Example:

```bash
./benchmark-toolkit/scripts/promote-trace-run-to-campaign.sh 20260422T154555Z
```

ข้อสำคัญ:

- การ promote เป็นเพียงการจัดระเบียบผลเก่า
- ไม่ทำให้ single run กลายเป็นหลักฐานเชิงสถิติหลาย epoch

### Phase 2. Campaign index

- สร้าง `benchmark-toolkit/results/campaigns/`
- สร้าง campaign แรกจาก baseline `ProvChain vs Neo4j`
- ทำ `campaign_manifest.json`
- ทำ `campaign_summary.md`

### Phase 3. Epoch runner

- เพิ่ม script สำหรับรันหลาย epoch
- ให้ script สร้าง `epoch_manifest.json` อัตโนมัติ
- ให้ script mark `failed`, `partial`, หรือ `excluded` โดยอิงจาก success gate

Current implementation:

- `benchmark-toolkit/scripts/run-trace-campaign.sh`
- default target: `ProvChain vs Neo4j`
- default family: `trace_query`
- default behavior: reset benchmark Docker volumes between epochs and preserve results under `benchmark-toolkit/results/campaigns/<campaign_id>/`

Example:

```bash
EPOCHS=30 ITERATIONS=10 CAMPAIGN_ID=20260424_trace_supply1000_provchain-neo4j_n30 ./benchmark-toolkit/scripts/run-trace-campaign.sh
```

### Phase 4. Publication export

- สร้าง script รวม campaign เป็น CSV เดียว
- export เฉพาะ campaign ที่ผ่าน validity gate
- เก็บ curated output ไว้ใน `docs/benchmarking/data/`

Current aggregation helper:

- `benchmark-toolkit/scripts/summarize-campaign.py`

Example:

```bash
python3 benchmark-toolkit/scripts/summarize-campaign.py benchmark-toolkit/results/campaigns/20260424_trace_supply1000_provchain-neo4j_n30
```

Output:

- `campaign_results.json`
- `campaign_results.csv`
- `campaign_aggregate_summary.md`

## Immediate Rule From Now On

ห้ามรัน benchmark campaign ยาวโดยเก็บผลรวมกันใน directory เดียวแบบไม่ระบุ campaign

ทุก campaign ต้องมี:

- `campaign_id`
- manifest
- epoch directory
- run artifacts
- status label

run เดี่ยวเพื่อ debug ยังทำได้ แต่ต้องติดป้ายว่าเป็น debug artifact ไม่ใช่ publication evidence
