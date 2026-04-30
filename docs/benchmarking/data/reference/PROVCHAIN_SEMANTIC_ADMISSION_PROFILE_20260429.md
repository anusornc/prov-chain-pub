# ProvChain Semantic Admission Profile - 2026-04-29

## Evidence

- Campaign: `20260429_profile_semantic_supply1000_provchain-fluree_n3`
- Curated export: `docs/benchmarking/data/reference/profiling_semantic_supply1000_provchain_fluree_n3_20260429/`
- Status: `passed`, `3/3` epochs
- Dataset: `supply_chain_1000.ttl`
- Products: `ProvChain-Org`, `Fluree`
- Evidence role: reference profiling evidence for R003 semantic admission analysis

This profile was run after the semantic campaign wrapper was corrected to use:

- ontology package startup: `config/ontology_package.toml`
- demo preload disabled
- isolated host ports starting at `18180`
- curated reference export after passing `run` campaigns

## Aggregate Results

| System | Path | Metric | Samples | Success Rate | Mean ms | p95 ms | p99 ms |
|---|---|---|---:|---:|---:|---:|---:|
| `ProvChain-Org` | `native-rdf-path` | `validation-latency-ms` | 3 | 100.00% | 7109.333 | 7147.800 | 7151.960 |
| `Fluree` | `external-semantic-pipeline` | `validation-latency-ms` | 3 | 100.00% | 872.691 | 917.199 | 922.219 |

Diagnostic component means:

| System | Component | Mean ms |
|---|---|---:|
| `ProvChain-Org` | Authentication | 635.764 |
| `ProvChain-Org` | Dataset read | 0.093 |
| `ProvChain-Org` | Turtle normalize | 9.424 |
| `ProvChain-Org` | Turtle parse | 3.055 |
| `ProvChain-Org` | HTTP submit loop | 6461.478 |
| `Fluree` | TTL-to-JSON-LD translation | 204.667 |
| `Fluree` | JSON-LD dataset read | 0.116 |
| `Fluree` | JSON-LD parse | 0.899 |
| `Fluree` | JSON-LD ledger insert | 507.831 |

## ProvChain Server-Side Breakdown

The ProvChain semantic admission path produced `632` server timing samples per epoch.
Across the three epochs, the per-record server timing means were:

| Stage | Mean ms per record |
|---|---:|
| Handler total | 9.796 |
| Request validation | 0.428 |
| Block admission total | 9.358 |
| Create block proposal | 4.776 |
| Submit signed block | 4.580 |
| State root | 4.396 |
| Persistence | 4.469 |
| Ontology validation total | 0.333 |
| SHACL shape validation total | 0.287 |
| SHACL core shape validation | 0.158 |
| SHACL domain shape validation | 0.129 |
| RDF store add | 0.044 |
| Metadata insert | 0.059 |

## Interpretation

This profile confirms that ProvChain's semantic admission cost is dominated by
per-record HTTP/block admission work, not Turtle parsing or SHACL validation.

The main current bottlenecks are:

- per-record state-root calculation
- per-record persistence
- one HTTP/API admission per parsed RDF triple

The native semantic checks themselves are comparatively small in this profile:
ontology validation plus SHACL shape validation average about `0.620 ms` per
record, while total handler cost is about `9.796 ms` per record.

## Claim Boundary

This evidence supports the paper statement that ProvChain provides native
RDF+SHACL semantic admission with explanation support, but its current admission
latency is higher than Fluree's externalized pipeline on this workload.

Do not use this profile to claim semantic latency superiority. Use it to explain
the tradeoff between native semantic/blockchain admission and an externalized
translation plus ledger-insert pipeline.
