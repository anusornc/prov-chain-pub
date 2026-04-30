# Campaign Aggregate Summary

- Campaign: `20260429_profile_semantic_supply1000_provchain-fluree_n3`
- Generated at: `2026-04-29T15:49:00Z`

| Family | Test | System | Path | Metric | Unit | Samples | Success Rate | Mean | p95 | p99 |
|---|---|---|---|---|---|---:|---:|---:|---:|---:|
| `semantic` | `Externalized TTL+JSON-LD Admission` | `Fluree` | `external-semantic-pipeline` | `validation-latency-ms` | `ms` | 3 | 100.00% | 872.691 | 917.199 | 922.219 |
| `semantic` | `Native RDF+SHACL Admission` | `ProvChain-Org` | `native-rdf-path` | `validation-latency-ms` | `ms` | 3 | 100.00% | 7109.333 | 7147.800 | 7151.960 |
| `semantic` | `Externalized JSON-LD Dataset Read` | `Fluree` | `external-semantic-pipeline` | `dataset-read-latency-ms` | `ms` | 3 | 100.00% | 0.116 | 0.124 | 0.125 |
| `semantic` | `Externalized JSON-LD Ledger Insert` | `Fluree` | `external-semantic-pipeline` | `load-latency-ms` | `ms` | 3 | 100.00% | 507.831 | 522.910 | 524.751 |
| `semantic` | `Externalized JSON-LD Parse` | `Fluree` | `external-semantic-pipeline` | `dataset-parse-latency-ms` | `ms` | 3 | 100.00% | 0.899 | 0.911 | 0.912 |
| `semantic` | `Externalized TTL-to-JSON-LD Translation` | `Fluree` | `external-semantic-pipeline` | `mapping-latency-ms` | `ms` | 3 | 100.00% | 204.667 | 247.600 | 253.520 |
| `semantic` | `Native Semantic Authentication` | `ProvChain-Org` | `native-rdf-path` | `authentication-latency-ms` | `ms` | 3 | 100.00% | 635.764 | 647.071 | 647.191 |
| `semantic` | `Native Semantic Dataset Read` | `ProvChain-Org` | `native-rdf-path` | `dataset-read-latency-ms` | `ms` | 3 | 100.00% | 0.093 | 0.110 | 0.112 |
| `semantic` | `Native Semantic HTTP Submit Loop` | `ProvChain-Org` | `native-rdf-path` | `client-submit-loop-latency-ms` | `ms` | 3 | 100.00% | 6461.478 | 6489.063 | 6492.749 |
| `semantic` | `Native Semantic Turtle Normalize` | `ProvChain-Org` | `native-rdf-path` | `dataset-normalize-latency-ms` | `ms` | 3 | 100.00% | 9.424 | 9.471 | 9.475 |
| `semantic` | `Native Semantic Turtle Parse` | `ProvChain-Org` | `native-rdf-path` | `dataset-parse-latency-ms` | `ms` | 3 | 100.00% | 3.055 | 3.066 | 3.067 |

## Semantic Capability Notes

| System | Native Semantic Support | External Semantic Stages | Explanation Support |
|---|---:|---|---:|
| `Fluree` | `false` | `ttl-to-jsonld-translation, jsonld-file-read, jsonld-parse, jsonld-ledger-insert` | `false` |
| `ProvChain-Org` | `true` | `` | `true` |
