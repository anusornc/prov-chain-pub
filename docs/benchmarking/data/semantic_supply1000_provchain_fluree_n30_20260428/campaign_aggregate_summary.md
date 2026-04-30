# Campaign Aggregate Summary

- Campaign: `20260428_semantic_supply1000_provchain-fluree_n30`
- Generated at: `2026-04-28T17:46:39Z`

| Family | Test | System | Path | Metric | Unit | Samples | Success Rate | Mean | p95 | p99 |
|---|---|---|---|---|---|---:|---:|---:|---:|---:|
| `semantic` | `Externalized JSON-LD Admission` | `Fluree` | `external-semantic-pipeline` | `validation-latency-ms` | `ms` | 30 | 100.00% | 514.667 | 566.000 | 570.970 |
| `semantic` | `Native RDF+SHACL Admission` | `ProvChain-Org` | `native-rdf-path` | `validation-latency-ms` | `ms` | 30 | 100.00% | 12121.933 | 12344.250 | 12834.530 |

## Semantic Capability Notes

| System | Native Semantic Support | External Semantic Stages | Explanation Support |
|---|---:|---|---:|
| `Fluree` | `false` | `ttl-to-jsonld-translation, jsonld-ledger-insert` | `false` |
| `ProvChain-Org` | `true` | `` | `true` |
