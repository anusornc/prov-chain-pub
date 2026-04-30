# Multi-Product Benchmark One-Page Summary - 2026-04-22

## Goal

สร้าง benchmark ที่ป้องกันการเปรียบเทียบผิดประเภทระหว่าง:

- `ProvChain`
- `Neo4j`
- `Fluree`
- `Hyperledger Fabric`
- `Go Ethereum (Geth)`

หลักคิดคือ:

- ไม่ใช้ตารางผู้ชนะรวมเพียงตารางเดียว
- เทียบกันตาม benchmark family
- ระบุ caveat ทุกครั้งเมื่อความสามารถของระบบไม่เหมือนกันโดยตรง

## Benchmark Families

| Family | วัดอะไร | Product หลัก |
|---|---|---|
| `A. Ledger / Write Path` | การเขียนข้อมูล, commit, finality, throughput | ProvChain, Fabric, Geth, Fluree |
| `B. Trace Query / Provenance` | การค้นย้อนเส้นทาง, multi-hop trace, aggregation | ProvChain, Neo4j, Fluree |
| `C. Semantic / Standards Admission` | RDF/JSON-LD ingest, SHACL, semantic validation | ProvChain, Fluree |
| `D. Governance / Policy` | permission, policy checks, authorized/unauthorized access | ProvChain, Fabric |
| `E. Interchange / Bridge` | export, transform, validate, import ระหว่างระบบ | ProvChain + target-specific peers |

## Product Roles

| Product | บทบาทหลัก | ไม่ควรใช้เป็น baseline สำหรับ |
|---|---|---|
| `ProvChain` | ระบบอ้างอิงหลัก | - |
| `Neo4j` | graph query baseline | ledger finality |
| `Fluree` | semantic ledger baseline | public-chain baseline |
| `Fabric` | permissioned ledger baseline | native semantic/RDF baseline |
| `Geth` | public-chain baseline | permissioned ledger baseline |

## What Can Be Compared Directly

### Direct and strong comparisons

- `ProvChain vs Neo4j` ใน `Trace Query`
- `ProvChain vs Fluree` ใน `Trace Query`
- `ProvChain vs Fluree` ใน `Semantic Admission`
- `ProvChain vs Fabric` ใน `Ledger / Write Path`
- `ProvChain vs Fabric` ใน `Governance / Policy`

### Comparisons that require explicit caveat

- `ProvChain vs Geth` ใน `Ledger / Write Path`
  - ต้องแยก `submit` ออกจาก `confirmation`
- `ProvChain vs Neo4j` ใน write path
  - ใช้ได้เพียงเป็น `secondary baseline`
- `Fabric` หรือ `Geth` ใน `Trace Query`
  - ใช้ได้ก็ต่อเมื่อมี documented query/index layer และต้องนับ layer นั้นเป็นส่วนหนึ่งของ stack

### Comparisons that should not be presented as like-for-like

- `Neo4j` เทียบกับ `Fabric` หรือ `Geth` ด้วย finality metric
- `Neo4j` เทียบกับ `ProvChain` ด้วย semantic-native claims
- `Geth` เทียบกับ `Fabric` เหมือนเป็น permissioned ledger ชนิดเดียวกัน

## Canonical Workloads

| Workload | Description |
|---|---|
| `W1` | single record insert |
| `W2` | batch insert (`10/100/1000`) |
| `W3` | single-entity lookup |
| `W4` | one-hop trace |
| `W5` | three-hop trace |
| `W6` | full provenance reconstruction |
| `W7` | semantic admission |
| `W8` | policy-enforced write |
| `W9` | policy-enforced read |
| `W10` | cross-system export/import |

## Canonical Dataset Slices

- `UHT case-study`
- `Hybrid GS1/EPCIS-UHT`
- `Healthcare-device`
- `Pharmaceutical-storage`

ทุกระบบต้องเริ่มจาก logical dataset เดียวกันก่อน แล้วค่อยแปลเป็น format ของระบบนั้น

## Report Labels

ทุกผล benchmark ต้องติดป้ายหนึ่งในนี้:

- `native-comparable`
- `secondary-baseline`
- `externalized-semantic-pipeline`
- `indexed-query-stack`
- `not-comparable`

## Current Execution Order

1. `ProvChain vs Neo4j`
   - Family `B`
   - baseline ที่พร้อมใช้งานแล้ว
2. `ProvChain vs Fluree`
   - Family `B` และ `C`
3. `ProvChain vs Fabric`
   - Family `A` และ `D`
4. `ProvChain vs Geth`
   - Family `A`
5. รวมผลหลาย family ใน final report

## Non-Negotiable Rules

- local contract tests ต้องผ่านก่อน Docker benchmark
- ถ้า `success_rate = 0` ห้ามใช้ latency ประกาศผู้ชนะ
- ห้ามทำ single global winner table
- ทุก claim ต้องระบุ:
  - benchmark family
  - dataset slice
  - compared products
  - fairness label

## Immediate Takeaway

ถ้าจะ benchmark อย่างถูกหลัก:

- อย่าเริ่มจากคำถามว่า `ProvChain` จะชนะใคร
- ให้เริ่มจากคำถามว่า `กำลังวัดความสามารถชั้นใด`
- แล้วจึงเลือกคู่เทียบที่เหมาะกับชั้นความสามารถนั้น
