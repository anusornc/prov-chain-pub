#!/usr/bin/env bash
# Reproducible reference-system evidence campaign (spec #1 milestone 7 / issue #13).
#
# Runs the complete reference-system verification workflow from one checkout and
# records every provenance fact and raw artifact an independent reviewer needs:
# revision, lockfile identity, Rust toolchain, OS/filesystem assumptions, enabled
# features, exact commands, raw logs, exit codes, artifact hashes, conformance
# vector regeneration, and a mechanically derived summary.
#
# Usage:
#   scripts/run_reference_evidence.sh [--allow-dirty] [output-dir]
#   scripts/run_reference_evidence.sh --compare RUN_A RUN_B
#
# The default output directory is test_reports/reference-evidence/run-<UTC>-<sha>.
# The campaign fails closed when tracked files are modified (a reviewer starts
# from a clean checkout); --allow-dirty records the dirty state instead.
#
# Required tools: git, cargo, rustc, python3, sha256sum.
# The rerun/compare mode proves canonical artifacts (test verdicts and vector
# hashes) are stable across reruns, including after a machine restart.

set -euo pipefail
cd "$(dirname "$0")/.."
REPO=$(pwd)
SUMMARIZER="scripts/reference_evidence_summarize.py"

ALLOW_DIRTY=0
if [[ "${1:-}" == "--allow-dirty" ]]; then
  ALLOW_DIRTY=1
  shift
fi

if [[ "${1:-}" == "--compare" ]]; then
  shift
  exec python3 "$SUMMARIZER" compare "$@"
fi

UTC=$(date -u +%Y%m%dT%H%M%SZ)
SHORT_SHA=$(git rev-parse --short HEAD)
OUT=${1:-test_reports/reference-evidence/run-${UTC}-${SHORT_SHA}}
mkdir -p "$OUT/logs" "$OUT/provenance" "$OUT/vectors"
OUT_ABS="$REPO/$OUT"

echo "reference-system evidence campaign"
echo "  output: $OUT"

# --- tree state: a reviewer runs from a clean checkout ----------------------
git status --porcelain --untracked-files=no > "$OUT/provenance/git_tracked_changes.txt" || true
if [[ -s "$OUT/provenance/git_tracked_changes.txt" && "$ALLOW_DIRTY" -eq 0 ]]; then
  echo "REFUSING TO RUN: tracked files are modified. Commit or stash them, or pass --allow-dirty." >&2
  cat "$OUT/provenance/git_tracked_changes.txt" >&2
  rm -rf "$OUT_ABS"
  exit 2
fi
git status --porcelain > "$OUT/provenance/git_status_full.txt" || true

# --- environment and provenance ---------------------------------------------
git rev-parse HEAD > "$OUT/provenance/git_commit.txt"
git describe --always --tags --dirty > "$OUT/provenance/git_describe.txt" 2>/dev/null || echo unknown > "$OUT/provenance/git_describe.txt"
git branch --show-current > "$OUT/provenance/git_branch.txt" 2>/dev/null || echo detached > "$OUT/provenance/git_branch.txt"
rustc -Vv > "$OUT/provenance/rustc_verbose.txt" 2>/dev/null || echo unknown > "$OUT/provenance/rustc_verbose.txt"
cargo --version > "$OUT/provenance/cargo_version.txt" 2>/dev/null || echo unknown > "$OUT/provenance/cargo_version.txt"
uname -a > "$OUT/provenance/uname.txt"
(cat /etc/os-release 2>/dev/null | grep -E '^(PRETTY_NAME|VERSION)=' || echo unknown) > "$OUT/provenance/os_release.txt"
sha256sum Cargo.lock > "$OUT/provenance/cargo_lock.sha256"
DF_REPO=$(stat -f -c %T . 2>/dev/null || echo unknown)
DF_TMP=$(stat -f -c %T "${TMPDIR:-/tmp}" 2>/dev/null || echo unknown)

{
  echo "schema=reference-evidence-environment-v1"
  echo "workflow=scripts/run_reference_evidence.sh"
  echo "started_utc=$UTC"
  echo "git_commit=$(cat "$OUT/provenance/git_commit.txt")"
  echo "git_describe=$(cat "$OUT/provenance/git_describe.txt")"
  echo "git_branch=$(cat "$OUT/provenance/git_branch.txt")"
  echo "tracked_files_modified=$(grep -c . "$OUT/provenance/git_tracked_changes.txt" || true)"
  echo "cargo_version=$(cat "$OUT/provenance/cargo_version.txt")"
  echo "rustc_version=$(rustc -V 2>/dev/null || echo unknown)"
  echo "cargo_lock_sha256=$(awk '{print $1}' "$OUT/provenance/cargo_lock.sha256")"
  echo "os_kernel=$(uname -srm)"
  echo "os_release=$(head -1 "$OUT/provenance/os_release.txt")"
  echo "filesystem_repo=$DF_REPO"
  echo "filesystem_tmp=$DF_TMP"
  echo "cpu_count=$(nproc 2>/dev/null || echo unknown)"
  echo "mem_total_kb=$(awk '/MemTotal/{print $2}' /proc/meminfo 2>/dev/null || echo unknown)"
  echo "locale_lang=${LANG:-unset}"
} > "$OUT/environment.env"

# --- command runner ----------------------------------------------------------
: > "$OUT/commands.tsv"
run_cmd() {
  local id=$1 log=$2
  shift 2
  local start end rc
  start=$(date +%s.%N)
  set +e
  "$@" > "$OUT/logs/$log" 2>&1
  rc=$?
  set -e
  end=$(date +%s.%N)
  printf '%s\t%s\t%s\t%s\t%s\n' "$id" "$log" "$rc" \
    "$(awk -v s="$start" -v e="$end" 'BEGIN{printf "%.3f", e-s}')" \
    "$*" >> "$OUT/commands.tsv"
  echo "  [exit $rc] $id -> logs/$log"
}

# --- ledger reference-system suites (stable artifact names) ------------------
for n in 2 3 4 5 6 7 10 11 12; do
  run_cmd "suite:ledger_issue_${n}:default" "ledger_issue_${n}_default.log" \
    cargo test --test "ledger_issue_${n}"
done
for n in 8 9; do
  run_cmd "suite:ledger_issue_${n}:privacy-conformance" "ledger_issue_${n}_privacy-conformance.log" \
    cargo test --features privacy-conformance --test "ledger_issue_${n}"
done
for n in 11 12; do
  run_cmd "suite:ledger_issue_${n}:bridge-conformance" "ledger_issue_${n}_bridge-conformance.log" \
    cargo test --features bridge-conformance --test "ledger_issue_${n}"
done

# --- static gates -------------------------------------------------------------
run_cmd "gate:clippy:default" "clippy_default.log" cargo clippy --all-targets
run_cmd "gate:clippy:all-features" "clippy_all_features.log" \
  cargo clippy --all-targets --features bridge-conformance,privacy-conformance
run_cmd "gate:rustfmt" "fmt_check.log" cargo fmt --check

# --- conformance vector regeneration (byte-identical) ------------------------
VECTORS_TMP=$(mktemp -d)
trap 'rm -rf "$VECTORS_TMP"' EXIT
: > "$OUT/vectors.tsv"
regen_vector() {
  local fixture=$1 generator=$2
  local regen="$VECTORS_TMP/$(basename "$fixture")"
  local rc=0
  python3 "scripts/$generator" --output "$regen" > "$OUT/logs/vector_${generator}.log" 2>&1 || rc=$?
  local have want
  have=$(sha256sum "$regen" 2>/dev/null | awk '{print $1}')
  want=$(sha256sum "tests/vectors/$(basename "$fixture")" | awk '{print $1}')
  local identical=no
  [[ "$have" == "$want" && "$rc" -eq 0 ]] && identical=yes
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$(basename "$fixture")" "$generator" "$rc" "$want" "$have" "$identical" \
    >> "$OUT/vectors.tsv"
  echo "  [regen:$identical] $(basename "$fixture")"
  cp "$regen" "$OUT/vectors/$(basename "$fixture")" 2>/dev/null || true
}
regen_vector tests/vectors/bridge_export_v1.json generate_bridge_export_vectors.py
regen_vector tests/vectors/privacy_lifecycle_v1.json generate_privacy_lifecycle_vectors.py
regen_vector tests/vectors/protected_object_v1.json generate_protected_object_vectors.py
regen_vector tests/vectors/privacy_grant_v1.json generate_privacy_grant_vectors.py

# --- mechanically derived manifest and summary -------------------------------
python3 "$SUMMARIZER" build "$OUT"
echo "campaign complete: $OUT/manifest.json"
python3 "$SUMMARIZER" verdict "$OUT"
