# Reference-System Evidence Run Summary

Mechanically derived from the raw logs in this directory by
`scripts/reference_evidence_summarize.py`. No value in this file was
entered by hand. Scope: bounded reference-system admission, membership,
PoA convergence, semantic admission, privacy lifecycle, and bridge
evidence. This campaign makes no ledger-throughput/TPS, generic OWL2,
full EPCIS, or production-pilot claim.

- **Verdict**: pass
- **Started (UTC)**: 20260925T111706Z
- **Revision**: `1e08ea43fe6070041ff20b9ffd84e14cfe6fe651` (1e08ea4)
- **Cargo.lock SHA-256**: `1eb6c24d12774230bd779227468d0533e4964123d37e9ed2acd1a917995a7ce5`
- **Rust toolchain**: rustc 1.91.1 (ed61e7d7e 2025-11-07) / cargo 1.91.1 (ea2d97820 2025-10-10)
- **OS/filesystem**: Linux 6.8.0-1047-gcp x86_64; repo fs `ext2/ext3`, tmp fs `ext2/ext3`
- **Features exercised**: default, `privacy-conformance`, `bridge-conformance`
- **Totals**: 101 passed, 0 failed, 1 ignored across 16 commands
- **Clippy warnings (default / all features)**: 0 / 0
- **rustfmt --check clean**: True
- **Vector regeneration byte-identical**: 4/4

## Commands

| id | exit | passed | failed | ignored | log |
|---|---|---|---|---|---|
| `suite:ledger_issue_2:default` | 0 | 9 | 0 | 0 | `logs/ledger_issue_2_default.log` |
| `suite:ledger_issue_3:default` | 0 | 3 | 0 | 0 | `logs/ledger_issue_3_default.log` |
| `suite:ledger_issue_4:default` | 0 | 8 | 0 | 0 | `logs/ledger_issue_4_default.log` |
| `suite:ledger_issue_5:default` | 0 | 9 | 0 | 0 | `logs/ledger_issue_5_default.log` |
| `suite:ledger_issue_6:default` | 0 | 6 | 0 | 1 | `logs/ledger_issue_6_default.log` |
| `suite:ledger_issue_7:default` | 0 | 26 | 0 | 0 | `logs/ledger_issue_7_default.log` |
| `suite:ledger_issue_10:default` | 0 | 1 | 0 | 0 | `logs/ledger_issue_10_default.log` |
| `suite:ledger_issue_11:default` | 0 | 4 | 0 | 0 | `logs/ledger_issue_11_default.log` |
| `suite:ledger_issue_12:default` | 0 | 6 | 0 | 0 | `logs/ledger_issue_12_default.log` |
| `suite:ledger_issue_8:privacy-conformance` | 0 | 11 | 0 | 0 | `logs/ledger_issue_8_privacy-conformance.log` |
| `suite:ledger_issue_9:privacy-conformance` | 0 | 6 | 0 | 0 | `logs/ledger_issue_9_privacy-conformance.log` |
| `suite:ledger_issue_11:bridge-conformance` | 0 | 5 | 0 | 0 | `logs/ledger_issue_11_bridge-conformance.log` |
| `suite:ledger_issue_12:bridge-conformance` | 0 | 7 | 0 | 0 | `logs/ledger_issue_12_bridge-conformance.log` |
| `gate:clippy:default` | 0 | 0 | 0 | 0 | `logs/clippy_default.log` |
| `gate:clippy:all-features` | 0 | 0 | 0 | 0 | `logs/clippy_all_features.log` |
| `gate:rustfmt` | 0 | 0 | 0 | 0 | `logs/fmt_check.log` |

## Conformance vectors

| fixture | byte-identical | checked-in SHA-256 |
|---|---|---|
| `tests/vectors/bridge_export_v1.json` | True | `7c2f9ba2eb72e700…` |
| `tests/vectors/privacy_lifecycle_v1.json` | True | `863098c71db0018d…` |
| `tests/vectors/protected_object_v1.json` | True | `edfd4cc8a2f852f1…` |
| `tests/vectors/privacy_grant_v1.json` | True | `cd1f0def4c87f14c…` |

## Negative and fault cases

Rejection, tamper, bounds, crash, response-loss, projection-failure,
restart, partition, rejoin, semantic-failure, privacy-denial,
bridge-conflict, and resource-incapacity behavior is asserted by the
named tests inside these suites; see
`docs/evidence/REFERENCE_SYSTEM_ACCEPTANCE_MATRIX.md` for the
case-to-test map and known limitations.
