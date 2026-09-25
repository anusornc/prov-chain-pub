#!/usr/bin/env python3
"""Mechanically derive the reference-system evidence manifest and summary.

Modes:
  build <run-dir>    parse logs/commands/vectors/environment and write
                     manifest.json + summary.md for one campaign run
  compare <A> <B>    compare two run dirs on canonical artifacts (test verdict
                     counts, exit codes, vector hashes); write
                     rerun_comparison.json into B; exit 1 on divergence
  verdict <run-dir>  print the overall pass/fail verdict and exit accordingly

Only the Python standard library is used. Every number in summary.md comes
from parsing the raw logs in the run directory; nothing is hand-entered.
"""

from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path

TEST_RESULT_RE = re.compile(
    r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; "
    r"(\d+) ignored; (\d+) measured; (\d+) filtered out"
)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def parse_env(run: Path) -> dict:
    env: dict = {}
    env_path = run / "environment.env"
    if env_path.exists():
        for line in env_path.read_text().splitlines():
            if "=" in line:
                key, value = line.split("=", 1)
                env[key] = value
    return env


def parse_commands(run: Path) -> list[dict]:
    commands = []
    tsv = run / "commands.tsv"
    if tsv.exists():
        for line in tsv.read_text().splitlines():
            parts = line.split("\t")
            if len(parts) != 5:
                continue
            cid, log, rc, duration, argv = parts
            log_path = run / "logs" / log
            entry = {
                "id": cid,
                "command": argv,
                "log": f"logs/{log}",
                "exit_code": int(rc),
                "duration_seconds": float(duration),
            }
            if log_path.exists():
                entry["log_sha256"] = sha256(log_path)
                entry["log_bytes"] = log_path.stat().st_size
                text = log_path.read_text(errors="replace")
                entry["tests_passed"] = sum(
                    int(m.group(2)) for m in TEST_RESULT_RE.finditer(text)
                )
                entry["tests_failed"] = sum(
                    int(m.group(3)) for m in TEST_RESULT_RE.finditer(text)
                )
                entry["tests_ignored"] = sum(
                    int(m.group(4)) for m in TEST_RESULT_RE.finditer(text)
                )
                entry["test_result_lines"] = [
                    m.group(0) for m in TEST_RESULT_RE.finditer(text)
                ]
                if cid.startswith("gate:clippy"):
                    entry["clippy_warnings"] = sum(
                        1 for ln in text.splitlines() if ln.startswith("warning")
                    )
                    entry["clippy_errors"] = sum(
                        1 for ln in text.splitlines() if ln.startswith("error")
                    )
            commands.append(entry)
    return commands


def parse_vectors(run: Path) -> list[dict]:
    vectors = []
    tsv = run / "vectors.tsv"
    if tsv.exists():
        for line in tsv.read_text().splitlines():
            parts = line.split("\t")
            if len(parts) != 6:
                continue
            name, generator, rc, checked, regen, identical = parts
            vectors.append(
                {
                    "fixture": f"tests/vectors/{name}",
                    "generator": f"scripts/{generator}",
                    "generator_exit_code": int(rc),
                    "checked_in_sha256": checked,
                    "regenerated_sha256": regen,
                    "regeneration_byte_identical": identical == "yes",
                }
            )
    return vectors


def provenance_files(run: Path) -> list[dict]:
    prov = run / "provenance"
    files = []
    if prov.is_dir():
        for path in sorted(prov.iterdir()):
            if path.is_file():
                files.append(
                    {"file": f"provenance/{path.name}", "sha256": sha256(path)}
                )
    return files


def build(run: Path) -> dict:
    env = parse_env(run)
    commands = parse_commands(run)
    vectors = parse_vectors(run)
    failed_ids = [
        c["id"]
        for c in commands
        if c["exit_code"] != 0
        or c.get("tests_failed", 0) > 0
        or c.get("clippy_errors", 0) > 0
    ]
    bad_vectors = [v["fixture"] for v in vectors if not v["regeneration_byte_identical"]]
    manifest = {
        "schema": "reference-evidence-manifest-v1",
        "run": {
            "started_utc": env.get("started_utc", "unknown"),
            "output_dir": run.name,
            "workflow": env.get("workflow", "scripts/run_reference_evidence.sh"),
            "verdict": "pass" if not failed_ids and not bad_vectors else "fail",
            "failed_command_ids": failed_ids,
            "non_identical_vectors": bad_vectors,
        },
        "environment": env,
        "commands": commands,
        "vectors": vectors,
        "provenance": provenance_files(run),
        "summary": {
            "tests_passed": sum(c.get("tests_passed", 0) for c in commands),
            "tests_failed": sum(c.get("tests_failed", 0) for c in commands),
            "tests_ignored": sum(c.get("tests_ignored", 0) for c in commands),
            "clippy_warnings_default": next(
                (c.get("clippy_warnings", 0) for c in commands
                 if c["id"] == "gate:clippy:default"), None),
            "clippy_warnings_all_features": next(
                (c.get("clippy_warnings", 0) for c in commands
                 if c["id"] == "gate:clippy:all-features"), None),
            "rustfmt_clean": next(
                (c["exit_code"] == 0 for c in commands
                 if c["id"] == "gate:rustfmt"), None),
        },
    }
    (run / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    (run / "summary.md").write_text(render_summary(manifest))
    return manifest


def render_summary(manifest: dict) -> str:
    env = manifest["environment"]
    total = manifest["summary"]
    run = manifest["run"]
    lines = [
        "# Reference-System Evidence Run Summary",
        "",
        "Mechanically derived from the raw logs in this directory by",
        "`scripts/reference_evidence_summarize.py`. No value in this file was",
        "entered by hand. Scope: bounded reference-system admission, membership,",
        "PoA convergence, semantic admission, privacy lifecycle, and bridge",
        "evidence. This campaign makes no ledger-throughput/TPS, generic OWL2,",
        "full EPCIS, or production-pilot claim.",
        "",
        f"- **Verdict**: {run['verdict']}",
        f"- **Started (UTC)**: {env.get('started_utc', 'unknown')}",
        f"- **Revision**: `{env.get('git_commit', 'unknown')}`"
        f" ({env.get('git_describe', 'unknown')})",
        f"- **Cargo.lock SHA-256**: `{env.get('cargo_lock_sha256', 'unknown')}`",
        f"- **Rust toolchain**: {env.get('rustc_version', 'unknown')}"
        f" / {env.get('cargo_version', 'unknown')}",
        f"- **OS/filesystem**: {env.get('os_kernel', 'unknown')};"
        f" repo fs `{env.get('filesystem_repo', 'unknown')}`,"
        f" tmp fs `{env.get('filesystem_tmp', 'unknown')}`",
        f"- **Features exercised**: default, `privacy-conformance`,"
        " `bridge-conformance`",
        f"- **Totals**: {total['tests_passed']} passed,"
        f" {total['tests_failed']} failed,"
        f" {total['tests_ignored']} ignored"
        f" across {len(manifest['commands'])} commands",
        f"- **Clippy warnings (default / all features)**:"
        f" {total.get('clippy_warnings_default')} /"
        f" {total.get('clippy_warnings_all_features')}",
        f"- **rustfmt --check clean**: {total.get('rustfmt_clean')}",
        f"- **Vector regeneration byte-identical**:"
        f" {sum(1 for v in manifest['vectors'] if v['regeneration_byte_identical'])}"
        f"/{len(manifest['vectors'])}",
    ]
    if run["failed_command_ids"]:
        lines.append(f"- **Failed commands**: {', '.join(run['failed_command_ids'])}")
    lines += ["", "## Commands", "", "| id | exit | passed | failed | ignored | log |", "|---|---|---|---|---|---|"]
    for c in manifest["commands"]:
        lines.append(
            f"| `{c['id']}` | {c['exit_code']} | {c.get('tests_passed', '-')} | "
            f"{c.get('tests_failed', '-')} | {c.get('tests_ignored', '-')} | "
            f"`{c['log']}` |"
        )
    lines += ["", "## Conformance vectors", "", "| fixture | byte-identical | checked-in SHA-256 |", "|---|---|---|"]
    for v in manifest["vectors"]:
        lines.append(
            f"| `{v['fixture']}` | {v['regeneration_byte_identical']} | "
            f"`{v['checked_in_sha256'][:16]}…` |"
        )
    lines += [
        "",
        "## Negative and fault cases",
        "",
        "Rejection, tamper, bounds, crash, response-loss, projection-failure,",
        "restart, partition, rejoin, semantic-failure, privacy-denial,",
        "bridge-conflict, and resource-incapacity behavior is asserted by the",
        "named tests inside these suites; see",
        "`docs/evidence/REFERENCE_SYSTEM_ACCEPTANCE_MATRIX.md` for the",
        "case-to-test map and known limitations.",
        "",
    ]
    return "\n".join(lines)


def compare(run_a: Path, run_b: Path) -> int:
    ma = json.loads((run_a / "manifest.json").read_text())
    mb = json.loads((run_b / "manifest.json").read_text())
    divergences = []
    matches = []
    by_id_a = {c["id"]: c for c in ma["commands"]}
    by_id_b = {c["id"]: c for c in mb["commands"]}
    for cid in sorted(set(by_id_a) & set(by_id_b)):
        a, b = by_id_a[cid], by_id_b[cid]
        keys = ("exit_code", "tests_passed", "tests_failed", "tests_ignored",
                "clippy_warnings", "clippy_errors")
        diffs = {k: (a.get(k), b.get(k)) for k in keys if a.get(k) != b.get(k)}
        if diffs:
            divergences.append({"id": cid, "fields": diffs})
        else:
            matches.append(cid)
    ids_only_a = sorted(set(by_id_a) - set(by_id_b))
    ids_only_b = sorted(set(by_id_b) - set(by_id_a))
    vec_a = {v["fixture"]: v for v in ma["vectors"]}
    vec_b = {v["fixture"]: v for v in mb["vectors"]}
    vector_stable = []
    for fixture in sorted(set(vec_a) & set(vec_b)):
        same = (
            vec_a[fixture]["regenerated_sha256"]
            == vec_b[fixture]["regenerated_sha256"]
            and vec_a[fixture]["regeneration_byte_identical"]
            and vec_b[fixture]["regeneration_byte_identical"]
        )
        vector_stable.append({"fixture": fixture, "stable_across_runs": same})
        if not same:
            divergences.append({"id": f"vector:{fixture}", "fields": {}})
    comparison = {
        "schema": "reference-evidence-rerun-comparison-v1",
        "run_a": {
            "dir": run_a.name,
            "started_utc": ma["run"]["started_utc"],
            "revision": ma["environment"].get("git_commit"),
        },
        "run_b": {
            "dir": run_b.name,
            "started_utc": mb["run"]["started_utc"],
            "revision": mb["environment"].get("git_commit"),
        },
        "same_revision": ma["environment"].get("git_commit")
        == mb["environment"].get("git_commit"),
        "command_ids_compared": len(matches) + len(divergences),
        "command_ids_matching": matches,
        "command_ids_only_in_a": ids_only_a,
        "command_ids_only_in_b": ids_only_b,
        "vectors": vector_stable,
        "canonical_artifacts_stable": not divergences,
        "divergences": divergences,
    }
    (run_b / "rerun_comparison.json").write_text(json.dumps(comparison, indent=2) + "\n")
    print(f"compared {comparison['command_ids_compared']} commands across runs")
    print(f"canonical artifacts stable: {comparison['canonical_artifacts_stable']}")
    for d in divergences:
        print(f"  DIVERGED: {d['id']} {d.get('fields', '')}")
    return 0 if comparison["canonical_artifacts_stable"] else 1


def verdict(run: Path) -> int:
    manifest = json.loads((run / "manifest.json").read_text())
    v = manifest["run"]["verdict"]
    print(f"verdict: {v}")
    for cid in manifest["run"]["failed_command_ids"]:
        print(f"  failed: {cid}")
    for fixture in manifest["run"]["non_identical_vectors"]:
        print(f"  non-identical vector: {fixture}")
    return 0 if v == "pass" else 1


def main(argv: list[str]) -> int:
    if len(argv) < 3:
        print(__doc__)
        return 2
    mode = argv[1]
    if mode == "build":
        build(Path(argv[2]))
        return 0
    if mode == "compare":
        return compare(Path(argv[2]), Path(argv[3]))
    if mode == "verdict":
        return verdict(Path(argv[2]))
    print(f"unknown mode: {mode}")
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
