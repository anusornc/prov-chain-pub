# Reference-System Evidence Workflow

This directory documents the corrected, rerunnable evidence workflow for the
thesis-defensible ProvChain reference system (spec issue #1, milestone 7;
implementation issue #13). The workflow exists so an independent reviewer can
reproduce every bounded reference-system claim from a clean checkout and see
exactly which claims are supported, descriptive, or out of scope.

- [`REFERENCE_SYSTEM_ACCEPTANCE_MATRIX.md`](REFERENCE_SYSTEM_ACCEPTANCE_MATRIX.md)
  maps each of the seven milestones to named tests, stable log artifacts, and
  known limitations, plus the cross-cutting case-class coverage table.
- [`../paper_submission/PAPER_EVIDENCE_INDEX.md`](../paper_submission/PAPER_EVIDENCE_INDEX.md)
  is the claim map: every paper table, figure, and claim traced to an artifact
  or an explicit unsupported/descriptive status.

## Rerunning from a clean checkout

```bash
git clone https://github.com/anusornc/prov-chain.git
cd prov-chain
./scripts/run_reference_evidence.sh            # run the campaign
./scripts/run_reference_evidence.sh            # run it again (e.g. after a restart)
./scripts/run_reference_evidence.sh --compare test_reports/reference-evidence/<run-A> \
                                            test_reports/reference-evidence/<run-B>
```

The campaign fails closed if tracked files are modified (pass `--allow-dirty`
only to record an explicitly dirty tree). It requires `git`, `cargo`/`rustc`,
`python3`, and `sha256sum`. The script exits nonzero when any command fails,
any suite reports a failed test, clippy reports errors, or a conformance
vector does not regenerate byte-identically.

## What each run records (issue #13 criterion 1)

- exact git revision, describe string, branch, and tree state;
- `Cargo.lock` SHA-256 (dependency lockfile identity);
- enabled feature sets per command (`default`, `privacy-conformance`,
  `bridge-conformance`);
- Rust toolchain (`rustc -Vv`, `cargo --version`);
- OS, kernel, repo and temp filesystem types, CPU count, memory;
- every exact command with exit code, wall time, and raw log;
- SHA-256 of every raw log, provenance file, and vector artifact.

## Stable artifact names (criterion 2)

```text
test_reports/reference-evidence/run-<UTC>-<short-sha>/
├── environment.env                 # key=value environment facts
├── commands.tsv                    # id, log, exit code, duration, argv
├── vectors.tsv                     # fixture, generator, hashes, byte-identity
├── manifest.json                   # mechanically derived manifest (schema v1)
├── summary.md                      # mechanically derived summary
├── rerun_comparison.json           # written by --compare into run B
├── logs/
│   ├── ledger_issue_<2..12>_<default|privacy-conformance|bridge-conformance>.log
│   ├── clippy_default.log
│   ├── clippy_all_features.log
│   ├── fmt_check.log
│   └── vector_<generator>.log
├── provenance/                     # git, toolchain, OS, lockfile facts
└── vectors/                        # regenerated conformance vector bytes
```

Raw logs carry the negative-case identifiers themselves: every rejection,
tamper, bounds, crash, response-loss, projection-failure, restart, partition,
rejoin, semantic-failure, privacy-denial, bridge-conflict, and
resource-incapacity assertion is a named test inside the suites (see the
matrix for the case-to-test map).

## Scope rules and claim exclusions (criteria 5–6)

- Focused ontology-admission and trace-query benchmark artifacts keep only
  their documented scopes in
  [`../benchmarking/BENCHMARK_EVIDENCE_BOUNDARY_2026-05-11.md`](../benchmarking/BENCHMARK_EVIDENCE_BOUNDARY_2026-05-11.md)
  and the paper evidence index; they are not reference-system evidence.
- Descriptive or manually curated values (comparison, domain, ontology, stack,
  and event tables) are separated from measured artifacts in the claim map and
  are never presented as measured results.
- No ledger-throughput/TPS, generic OWL2 reasoning, full GS1/EPCIS compliance,
  or production-pilot claim is supported by this workflow. Such claims remain
  explicitly out of scope until a corrected harness with raw statistical
  evidence exists.
- `test_reports/summary.md` is a withdrawn historical snapshot; the corrected
  summaries are the mechanically derived `summary.md` files inside each run.
