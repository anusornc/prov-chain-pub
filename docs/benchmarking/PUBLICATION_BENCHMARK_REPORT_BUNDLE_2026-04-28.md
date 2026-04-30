# Publication Benchmark Report Bundle

- Generated at: `2026-04-29T09:57:54Z`
- Scope: family-specific benchmark evidence only
- Rule: no single global winner table is generated

This report separates primary benchmark metrics from load, import, and reference metrics. Load/import rows describe setup or data-ingestion paths and must not be used as primary ledger-write evidence.

## Evidence Sources

| Evidence Directory | Campaign | Status | Family | Dataset | Workload | Products |
|---|---|---|---|---|---|---|
| `docs/benchmarking/data/trace_supply1000_provchain_neo4j_n30_20260424` | `20260424_trace_supply1000_provchain-neo4j_n30` | `passed` | `trace_query` | `supply_chain_1000` | `` | `provchain, neo4j` |
| `docs/benchmarking/data/trace_supply1000_provchain_neo4j_fluree_n30_20260428` | `20260428_trace_supply1000_provchain-neo4j-fluree_n30` | `passed` | `trace_query` | `supply_chain_1000` | `` | `provchain, neo4j, fluree` |
| `docs/benchmarking/data/ledger_supply1000_provchain_fabric_managed_n30_20260425` | `20260425_ledger_supply1000_provchain-fabric_managed_n30_fix3` | `passed` | `ledger_write` | `supply_chain_1000` | `` | `provchain, fabric` |
| `docs/benchmarking/data/semantic_supply1000_provchain_fluree_n30_20260428` | `20260428_semantic_supply1000_provchain-fluree_n30` | `passed` | `semantic` | `supply_chain_1000` | `` | `provchain, fluree` |
| `docs/benchmarking/data/policy_supply1000_provchain_fabric_e2e_n30_20260429` | `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `passed` | `governance_policy` | `supply_chain_1000` | `policy` | `provchain, fabric` |
| `docs/benchmarking/data/ledger_supply1000_provchain_geth_n30_20260428` | `20260428_ledger_supply1000_provchain-geth_n30_fix1` | `passed` | `ledger_write` | `supply_chain_1000` | `write` | `provchain, geth` |

## governance-policy

### Primary Benchmark Metrics

| Campaign | Dataset | Test | System | Path | Fairness | Metric | Unit | Samples | Success | Mean | p95 | p99 | Caveat |
|---|---|---|---|---|---|---|---|---:|---:|---:|---:|---:|---|
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `auditor-read` | `Hyperledger Fabric` | `native` | `native-comparable` | `authorized-read-latency-ms` | `ms` | 300 | 100.00% | 5.460 | 6.367 | 6.869 | server-reported policy decision latency; not full API round-trip |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `auditor-read` | `Hyperledger Fabric` | `native` | `native-comparable` | `policy-check-latency-ms` | `ms` | 300 | 100.00% | 6.552 | 7.506 | 7.999 | client-observed policy API round-trip |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `auditor-read` | `ProvChain-Org` | `native` | `cross-model-with-caveat` | `authorized-read-latency-ms` | `ms` | 300 | 100.00% | 0.001 | 0.002 | 0.002 | server-reported policy decision latency; not full API round-trip; cross-model parity row; compare within the scenario but preserve endpoint/model caveat |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `auditor-read` | `ProvChain-Org` | `native` | `cross-model-with-caveat` | `policy-check-latency-ms` | `ms` | 300 | 100.00% | 1.947 | 2.301 | 2.863 | client-observed policy API round-trip; cross-model parity row; compare within the scenario but preserve endpoint/model caveat |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `authorized-read` | `Hyperledger Fabric` | `native` | `native-comparable` | `authorized-read-latency-ms` | `ms` | 300 | 100.00% | 5.636 | 6.792 | 7.293 | server-reported policy decision latency; not full API round-trip |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `authorized-read` | `Hyperledger Fabric` | `native` | `native-comparable` | `policy-check-latency-ms` | `ms` | 300 | 100.00% | 6.831 | 8.038 | 8.639 | client-observed policy API round-trip |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `authorized-read` | `ProvChain-Org` | `native` | `cross-model-with-caveat` | `authorized-read-latency-ms` | `ms` | 300 | 100.00% | 0.002 | 0.002 | 0.002 | server-reported policy decision latency; not full API round-trip; cross-model parity row; compare within the scenario but preserve endpoint/model caveat |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `authorized-read` | `ProvChain-Org` | `native` | `cross-model-with-caveat` | `policy-check-latency-ms` | `ms` | 300 | 100.00% | 2.249 | 2.576 | 2.700 | client-observed policy API round-trip; cross-model parity row; compare within the scenario but preserve endpoint/model caveat |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `authorized-write` | `Hyperledger Fabric` | `native` | `native-comparable` | `authorized-write-latency-ms` | `ms` | 300 | 100.00% | 5.260 | 6.024 | 6.557 | server-reported policy decision latency; not full API round-trip |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `authorized-write` | `Hyperledger Fabric` | `native` | `native-comparable` | `policy-check-latency-ms` | `ms` | 300 | 100.00% | 6.317 | 7.167 | 7.643 | client-observed policy API round-trip |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `authorized-write` | `ProvChain-Org` | `native` | `cross-model-with-caveat` | `authorized-write-latency-ms` | `ms` | 300 | 100.00% | 0.001 | 0.001 | 0.002 | server-reported policy decision latency; not full API round-trip; cross-model parity row; compare within the scenario but preserve endpoint/model caveat |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `authorized-write` | `ProvChain-Org` | `native` | `cross-model-with-caveat` | `policy-check-latency-ms` | `ms` | 300 | 100.00% | 1.870 | 2.151 | 2.539 | client-observed policy API round-trip; cross-model parity row; compare within the scenario but preserve endpoint/model caveat |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `rejected-write` | `Hyperledger Fabric` | `native` | `native-comparable` | `policy-check-latency-ms` | `ms` | 300 | 100.00% | 6.231 | 7.099 | 7.631 | client-observed policy API round-trip |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `rejected-write` | `Hyperledger Fabric` | `native` | `native-comparable` | `rejected-write-latency-ms` | `ms` | 300 | 100.00% | 5.167 | 5.947 | 6.355 | server-reported policy decision latency; not full API round-trip |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `rejected-write` | `ProvChain-Org` | `native` | `cross-model-with-caveat` | `policy-check-latency-ms` | `ms` | 300 | 100.00% | 1.864 | 2.131 | 2.296 | client-observed policy API round-trip; cross-model parity row; compare within the scenario but preserve endpoint/model caveat |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `rejected-write` | `ProvChain-Org` | `native` | `cross-model-with-caveat` | `rejected-write-latency-ms` | `ms` | 300 | 100.00% | 0.001 | 0.002 | 0.002 | server-reported policy decision latency; not full API round-trip; cross-model parity row; compare within the scenario but preserve endpoint/model caveat |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `unauthorized-read` | `Hyperledger Fabric` | `native` | `native-comparable` | `policy-check-latency-ms` | `ms` | 300 | 100.00% | 6.366 | 7.396 | 7.695 | client-observed policy API round-trip |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `unauthorized-read` | `Hyperledger Fabric` | `native` | `native-comparable` | `unauthorized-rejection-latency-ms` | `ms` | 300 | 100.00% | 5.301 | 6.280 | 6.528 | server-reported policy decision latency; not full API round-trip |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `unauthorized-read` | `ProvChain-Org` | `native` | `cross-model-with-caveat` | `policy-check-latency-ms` | `ms` | 300 | 100.00% | 1.923 | 2.315 | 2.878 | client-observed policy API round-trip; cross-model parity row; compare within the scenario but preserve endpoint/model caveat |
| `20260429_policy_supply1000_provchain-fabric_e2e_n30` | `supply_chain_1000` | `unauthorized-read` | `ProvChain-Org` | `native` | `cross-model-with-caveat` | `unauthorized-rejection-latency-ms` | `ms` | 300 | 100.00% | 0.001 | 0.002 | 0.002 | server-reported policy decision latency; not full API round-trip; cross-model parity row; compare within the scenario but preserve endpoint/model caveat |


## ledger-write

### Primary Benchmark Metrics

| Campaign | Dataset | Test | System | Path | Fairness | Metric | Unit | Samples | Success | Mean | p95 | p99 | Caveat |
|---|---|---|---|---|---|---|---|---:|---:|---:|---:|---:|---|
| `20260425_ledger_supply1000_provchain-fabric_managed_n30_fix3` | `supply_chain_1000` | `Batch Commit (100 records)` | `Hyperledger Fabric` | `legacy-not-recorded` | `legacy-not-recorded` | `legacy-latency-ms` | `ms` | 300 | 100.00% | 2165.363 | 2208.168 | 2248.422 | legacy export lacks capability/fairness metadata |
| `20260425_ledger_supply1000_provchain-fabric_managed_n30_fix3` | `supply_chain_1000` | `Batch Submit (100 records)` | `Hyperledger Fabric` | `legacy-not-recorded` | `legacy-not-recorded` | `legacy-latency-ms` | `ms` | 300 | 100.00% | 139.526 | 179.506 | 223.464 | legacy export lacks capability/fairness metadata |
| `20260425_ledger_supply1000_provchain-fabric_managed_n30_fix3` | `supply_chain_1000` | `Single Record Commit` | `Hyperledger Fabric` | `legacy-not-recorded` | `legacy-not-recorded` | `legacy-latency-ms` | `ms` | 300 | 100.00% | 2022.808 | 2025.666 | 2026.863 | legacy export lacks capability/fairness metadata |
| `20260425_ledger_supply1000_provchain-fabric_managed_n30_fix3` | `supply_chain_1000` | `Single Record Submit` | `Hyperledger Fabric` | `legacy-not-recorded` | `legacy-not-recorded` | `legacy-latency-ms` | `ms` | 300 | 100.00% | 12.809 | 15.194 | 16.436 | legacy export lacks capability/fairness metadata |
| `20260425_ledger_supply1000_provchain-fabric_managed_n30_fix3` | `supply_chain_1000` | `Single-threaded Write (100 tx)` | `ProvChain-Org` | `legacy-not-recorded` | `legacy-not-recorded` | `legacy-latency-ms` | `ms` | 300 | 100.00% | 29352.300 | 53346.500 | 55913.880 | legacy export lacks capability/fairness metadata |
| `20260428_ledger_supply1000_provchain-geth_n30_fix1` | `supply_chain_1000` | `Single-threaded Write (100 tx)` | `ProvChain-Org` | `native-rdf-path` | `native-comparable` | `submit-latency-ms` | `ms` | 300 | 100.00% | 29361.670 | 53256.050 | 55661.070 | primary scoped metric |

### Load, Import, And Reference Metrics

These rows are retained as evidence context, but they are not primary within-family winner evidence.

| Campaign | Dataset | Test | System | Path | Fairness | Metric | Unit | Samples | Success | Mean | p95 | p99 | Caveat |
|---|---|---|---|---|---|---|---|---:|---:|---:|---:|---:|---|
| `20260424_trace_supply1000_provchain-neo4j_n30` | `supply_chain_1000` | `Turtle RDF Import` | `ProvChain-Org` | `legacy-not-recorded` | `legacy-not-recorded` | `legacy-latency-ms` | `ms` | 30 | 100.00% | 12124.033 | 12310.000 | 12351.660 | load/import setup metric; not primary ledger-write evidence; legacy export lacks capability/fairness metadata |
| `20260424_trace_supply1000_provchain-neo4j_n30` | `supply_chain_1000` | `Turtle to Cypher Import` | `Neo4j` | `legacy-not-recorded` | `legacy-not-recorded` | `legacy-latency-ms` | `ms` | 30 | 100.00% | 31250.767 | 33106.250 | 33776.470 | load/import setup metric; not primary ledger-write evidence; legacy export lacks capability/fairness metadata |
| `20260428_ledger_supply1000_provchain-geth_n30_fix1` | `supply_chain_1000` | `Single Record Confirmation` | `Go Ethereum (Geth)` | `public-chain-smart-contract` | `public-chain-baseline` | `confirmation-latency-ms` | `ms` | 300 | 100.00% | 256.571 | 257.735 | 258.498 | public-chain reference; not permissioned-ledger evidence |
| `20260428_ledger_supply1000_provchain-geth_n30_fix1` | `supply_chain_1000` | `Single Record Gas Used` | `Go Ethereum (Geth)` | `public-chain-smart-contract` | `public-chain-baseline` | `gas-used` | `gas` | 300 | 100.00% | 46248.600 | 46251.000 | 46251.000 | public-chain reference; not permissioned-ledger evidence; resource-cost metric; not latency |
| `20260428_ledger_supply1000_provchain-geth_n30_fix1` | `supply_chain_1000` | `Single Record Submit` | `Go Ethereum (Geth)` | `public-chain-smart-contract` | `public-chain-baseline` | `submit-latency-ms` | `ms` | 300 | 100.00% | 2.208 | 2.479 | 2.728 | public-chain reference; not permissioned-ledger evidence |
| `20260428_trace_supply1000_provchain-neo4j-fluree_n30` | `supply_chain_1000` | `JSON-LD Import` | `Fluree` | `native-rdf-path` | `native-comparable` | `load-latency-ms` | `ms` | 30 | 100.00% | 469.767 | 570.300 | 580.810 | load/import setup metric; not primary ledger-write evidence |
| `20260428_trace_supply1000_provchain-neo4j-fluree_n30` | `supply_chain_1000` | `Turtle RDF Import` | `ProvChain-Org` | `native-rdf-path` | `native-comparable` | `load-latency-ms` | `ms` | 30 | 100.00% | 12122.067 | 12339.900 | 12405.990 | load/import setup metric; not primary ledger-write evidence |
| `20260428_trace_supply1000_provchain-neo4j-fluree_n30` | `supply_chain_1000` | `Turtle to Cypher Import` | `Neo4j` | `secondary-transactional-baseline` | `secondary-baseline` | `load-latency-ms` | `ms` | 30 | 100.00% | 30715.400 | 32151.950 | 32751.500 | load/import setup metric; not primary ledger-write evidence; secondary reference path |


## semantic

### Primary Benchmark Metrics

| Campaign | Dataset | Test | System | Path | Fairness | Metric | Unit | Samples | Success | Mean | p95 | p99 | Caveat |
|---|---|---|---|---|---|---|---|---:|---:|---:|---:|---:|---|
| `20260428_semantic_supply1000_provchain-fluree_n30` | `supply_chain_1000` | `Native RDF+SHACL Admission` | `ProvChain-Org` | `native-rdf-path` | `native-comparable` | `validation-latency-ms` | `ms` | 30 | 100.00% | 12121.933 | 12344.250 | 12834.530 | primary scoped metric |

### Load, Import, And Reference Metrics

These rows are retained as evidence context, but they are not primary within-family winner evidence.

| Campaign | Dataset | Test | System | Path | Fairness | Metric | Unit | Samples | Success | Mean | p95 | p99 | Caveat |
|---|---|---|---|---|---|---|---|---:|---:|---:|---:|---:|---|
| `20260428_semantic_supply1000_provchain-fluree_n30` | `supply_chain_1000` | `Externalized JSON-LD Admission` | `Fluree` | `external-semantic-pipeline` | `externalized-semantic-pipeline` | `validation-latency-ms` | `ms` | 30 | 100.00% | 514.667 | 566.000 | 570.970 | externalized semantic pipeline reference |


Semantic capability fields:

| System | Native Semantic Support | External Semantic Stages | Explanation Support |
|---|---:|---|---:|
| `Fluree` | `false` | `ttl-to-jsonld-translation, jsonld-ledger-insert` | `false` |
| `ProvChain-Org` | `true` | `` | `true` |

## trace-query

### Primary Benchmark Metrics

| Campaign | Dataset | Test | System | Path | Fairness | Metric | Unit | Samples | Success | Mean | p95 | p99 | Caveat |
|---|---|---|---|---|---|---|---|---:|---:|---:|---:|---:|---|
| `20260424_trace_supply1000_provchain-neo4j_n30` | `supply_chain_1000` | `Aggregation by Producer` | `Neo4j` | `legacy-not-recorded` | `legacy-not-recorded` | `legacy-latency-ms` | `ms` | 300 | 100.00% | 29.413 | 257.100 | 281.020 | legacy export lacks capability/fairness metadata |
| `20260424_trace_supply1000_provchain-neo4j_n30` | `supply_chain_1000` | `Aggregation by Producer` | `ProvChain-Org` | `legacy-not-recorded` | `legacy-not-recorded` | `legacy-latency-ms` | `ms` | 300 | 100.00% | 0.482 | 0.632 | 0.970 | legacy export lacks capability/fairness metadata |
| `20260424_trace_supply1000_provchain-neo4j_n30` | `supply_chain_1000` | `Multi-hop Traceability (10 hops)` | `Neo4j` | `legacy-not-recorded` | `legacy-not-recorded` | `legacy-latency-ms` | `ms` | 1500 | 100.00% | 13.695 | 63.000 | 260.040 | legacy export lacks capability/fairness metadata |
| `20260424_trace_supply1000_provchain-neo4j_n30` | `supply_chain_1000` | `Multi-hop Traceability (10 hops)` | `ProvChain-Org` | `legacy-not-recorded` | `legacy-not-recorded` | `legacy-latency-ms` | `ms` | 1500 | 100.00% | 0.363 | 0.538 | 0.843 | legacy export lacks capability/fairness metadata |
| `20260424_trace_supply1000_provchain-neo4j_n30` | `supply_chain_1000` | `Simple Product Lookup` | `Neo4j` | `legacy-not-recorded` | `legacy-not-recorded` | `legacy-latency-ms` | `ms` | 1500 | 100.00% | 9.389 | 37.000 | 122.060 | legacy export lacks capability/fairness metadata |
| `20260424_trace_supply1000_provchain-neo4j_n30` | `supply_chain_1000` | `Simple Product Lookup` | `ProvChain-Org` | `legacy-not-recorded` | `legacy-not-recorded` | `legacy-latency-ms` | `ms` | 1500 | 100.00% | 0.358 | 0.603 | 1.089 | legacy export lacks capability/fairness metadata |
| `20260428_trace_supply1000_provchain-neo4j-fluree_n30` | `supply_chain_1000` | `Aggregation by Producer` | `Fluree` | `native-rdf-path` | `native-comparable` | `query-latency-ms` | `ms` | 300 | 100.00% | 127.179 | 238.204 | 297.764 | primary scoped metric |
| `20260428_trace_supply1000_provchain-neo4j-fluree_n30` | `supply_chain_1000` | `Aggregation by Producer` | `Neo4j` | `translated-graph-model` | `native-comparable` | `query-latency-ms` | `ms` | 300 | 100.00% | 28.477 | 246.050 | 264.070 | primary scoped metric |
| `20260428_trace_supply1000_provchain-neo4j-fluree_n30` | `supply_chain_1000` | `Aggregation by Producer` | `ProvChain-Org` | `native-rdf-path` | `native-comparable` | `query-latency-ms` | `ms` | 300 | 100.00% | 0.566 | 0.755 | 1.204 | primary scoped metric |
| `20260428_trace_supply1000_provchain-neo4j-fluree_n30` | `supply_chain_1000` | `Multi-hop Traceability (10 hops)` | `Fluree` | `native-rdf-path` | `native-comparable` | `query-latency-ms` | `ms` | 1500 | 100.00% | 10.060 | 16.588 | 26.856 | primary scoped metric |
| `20260428_trace_supply1000_provchain-neo4j-fluree_n30` | `supply_chain_1000` | `Multi-hop Traceability (10 hops)` | `Neo4j` | `translated-graph-model` | `native-comparable` | `query-latency-ms` | `ms` | 1500 | 100.00% | 14.515 | 64.000 | 253.010 | primary scoped metric |
| `20260428_trace_supply1000_provchain-neo4j-fluree_n30` | `supply_chain_1000` | `Multi-hop Traceability (10 hops)` | `ProvChain-Org` | `native-rdf-path` | `native-comparable` | `query-latency-ms` | `ms` | 1500 | 100.00% | 0.479 | 0.594 | 0.936 | primary scoped metric |
| `20260428_trace_supply1000_provchain-neo4j-fluree_n30` | `supply_chain_1000` | `Simple Product Lookup` | `Fluree` | `native-rdf-path` | `native-comparable` | `query-latency-ms` | `ms` | 1500 | 100.00% | 9.722 | 12.069 | 85.969 | primary scoped metric |
| `20260428_trace_supply1000_provchain-neo4j-fluree_n30` | `supply_chain_1000` | `Simple Product Lookup` | `Neo4j` | `translated-graph-model` | `native-comparable` | `query-latency-ms` | `ms` | 1500 | 100.00% | 10.304 | 36.000 | 112.010 | primary scoped metric |
| `20260428_trace_supply1000_provchain-neo4j-fluree_n30` | `supply_chain_1000` | `Simple Product Lookup` | `ProvChain-Org` | `native-rdf-path` | `native-comparable` | `query-latency-ms` | `ms` | 1500 | 100.00% | 0.487 | 0.732 | 1.091 | primary scoped metric |


## Fairness And Limitations

- Interpret every result only within its benchmark family, workload, dataset slice, and capability path.
- Treat load/import rows as data-ingestion or setup-path evidence, not as primary ledger-write results.
- Do not compare native RDF trace-query latency directly with ledger finality, public-chain gas, or policy checks.
- `legacy-not-recorded` rows come from older exports that did not capture explicit capability and fairness metadata.
- `externalized-semantic-pipeline` rows include a different semantic capability path from native ProvChain validation.
- `public-chain-baseline` rows are public-chain execution evidence, not permissioned-enterprise ledger evidence.
- `cross-model-with-caveat` rows are scenario-level parity evidence and must preserve endpoint/model differences in any claim.
- Failed or partial campaigns must remain excluded from publication claims unless explicitly discussed as negative evidence.
