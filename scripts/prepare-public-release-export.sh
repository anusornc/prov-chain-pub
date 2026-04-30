#!/usr/bin/env bash
set -euo pipefail

SOURCE_REPO="${SOURCE_REPO:-}"
PUBLIC_WORKTREE="${PUBLIC_WORKTREE:-/tmp/prov-chain-pub-export}"
SOURCE_REF="${SOURCE_REF:-HEAD}"
COMMIT_CHANGES="false"
COMMIT_MESSAGE="${COMMIT_MESSAGE:-chore(public): update curated release artifacts}"

usage() {
  cat <<'EOF_USAGE'
Usage:
  scripts/prepare-public-release-export.sh [options]

Options:
  --source <path>       Source private repository. Defaults to current git root.
  --target <path>       Public release worktree. Defaults to /tmp/prov-chain-pub-export.
  --ref <git-ref>       Source ref to export. Defaults to HEAD.
  --commit              Commit the resulting public worktree changes.
  --message <message>   Commit message for --commit.
  --help                Show this help.

This script prepares the curated public release worktree. It does not push.
EOF_USAGE
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

require_value() {
  local option="$1"
  local value="${2:-}"
  if [ -z "${value}" ]; then
    die "${option} requires a value"
  fi
}

while [ $# -gt 0 ]; do
  case "$1" in
    --source)
      require_value "$1" "${2:-}"
      SOURCE_REPO="$2"
      shift 2
      ;;
    --target)
      require_value "$1" "${2:-}"
      PUBLIC_WORKTREE="$2"
      shift 2
      ;;
    --ref)
      require_value "$1" "${2:-}"
      SOURCE_REF="$2"
      shift 2
      ;;
    --commit)
      COMMIT_CHANGES="true"
      shift
      ;;
    --message)
      require_value "$1" "${2:-}"
      COMMIT_MESSAGE="$2"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      die "unknown option: $1"
      ;;
  esac
done

if [ -z "${SOURCE_REPO}" ]; then
  SOURCE_REPO="$(git rev-parse --show-toplevel)"
fi

SOURCE_REPO="$(cd "${SOURCE_REPO}" && pwd)"
PUBLIC_WORKTREE="$(cd "${PUBLIC_WORKTREE}" && pwd)"

git -C "${SOURCE_REPO}" rev-parse --git-dir >/dev/null 2>&1 ||
  die "source is not a git repository: ${SOURCE_REPO}"
git -C "${PUBLIC_WORKTREE}" rev-parse --git-dir >/dev/null 2>&1 ||
  die "target is not a git worktree: ${PUBLIC_WORKTREE}"

if [ -n "$(git -C "${PUBLIC_WORKTREE}" status --porcelain)" ]; then
  die "public worktree has uncommitted changes: ${PUBLIC_WORKTREE}"
fi

if [ -z "$(git -C "${SOURCE_REPO}" rev-parse --verify "${SOURCE_REF}^{commit}" 2>/dev/null)" ]; then
  die "source ref does not resolve to a commit: ${SOURCE_REF}"
fi

STAGE_DIR="$(mktemp -d /tmp/provchain-public-stage.XXXXXX)"
trap 'rm -rf "${STAGE_DIR}"' EXIT

printf '[public-export] source: %s (%s)\n' "${SOURCE_REPO}" "$(git -C "${SOURCE_REPO}" rev-parse --short "${SOURCE_REF}")"
printf '[public-export] target: %s\n' "${PUBLIC_WORKTREE}"

git -C "${SOURCE_REPO}" archive --format=tar "${SOURCE_REF}" | tar -xf - -C "${STAGE_DIR}"

# Public release policy: keep executable/reproducible assets, omit internal
# review/manuscript/build/runtime artifacts.
rm -rf \
  "${STAGE_DIR}/.codex" \
  "${STAGE_DIR}/AGENTS.md" \
  "${STAGE_DIR}/CLAUDE.md" \
  "${STAGE_DIR}/docs/_build" \
  "${STAGE_DIR}/docs/archive" \
  "${STAGE_DIR}/docs/BENCHMARKING_PLAN.md" \
  "${STAGE_DIR}/docs/BENCHMARK_RESULTS_SUMMARY.md" \
  "${STAGE_DIR}/docs/BUILD_BRANCH_STRUCTURE.md" \
  "${STAGE_DIR}/docs/CODEBASE_WEAKNESS_ANALYSIS_SUMMARY.md" \
  "${STAGE_DIR}/docs/E2E_TESTING_IMPLEMENTATION_SUMMARY.md" \
  "${STAGE_DIR}/docs/FILE_PLACEMENT_RULES.md" \
  "${STAGE_DIR}/docs/Plan.md" \
  "${STAGE_DIR}/docs/PRODUCTION_TESTING_PLAN.md" \
  "${STAGE_DIR}/docs/REAL_WORLD_TRACEABILITY_PLAN.md" \
  "${STAGE_DIR}/docs/REAL_WORLD_TRACEABILITY_TEST_RESULTS.md" \
  "${STAGE_DIR}/docs/RESEARCH_PUBLICATION_STRATEGY.md" \
  "${STAGE_DIR}/docs/Run.md" \
  "${STAGE_DIR}/docs/SECURITY_ANALYSIS_REPORT.md" \
  "${STAGE_DIR}/docs/THESIS_COMPLETION_REPORT.md" \
  "${STAGE_DIR}/docs/architecture/COMPONENT_OWNERSHIP.md" \
  "${STAGE_DIR}/docs/architecture/SHARED_ONTOLOGY_NETWORK_WORKING_PLAN.md" \
  "${STAGE_DIR}/docs/literature_review.tex" \
  "${STAGE_DIR}/docs/paper_submission" \
  "${STAGE_DIR}/docs/project-health" \
  "${STAGE_DIR}/docs/publication" \
  "${STAGE_DIR}/docs/research" \
  "${STAGE_DIR}/docs/reviews" \
  "${STAGE_DIR}/docs/thesis" \
  "${STAGE_DIR}/tests/COMPREHENSIVE_TEST_ANALYSIS_REPORT.md" \
  "${STAGE_DIR}/tests/FINAL_TEST_ANALYSIS_REPORT.md" \
  "${STAGE_DIR}/tests/FINAL_TEST_FIXES_SUMMARY.md" \
  "${STAGE_DIR}/tests/FINAL_TEST_REORGANIZATION_REPORT.md" \
  "${STAGE_DIR}/tests/TEST_IMPROVEMENT_SUMMARY.md" \
  "${STAGE_DIR}/tests/TEST_VALIDATION_RESULTS.md" \
  "${STAGE_DIR}/test_reports" \
  "${STAGE_DIR}/demo_basic_blockchain" \
  "${STAGE_DIR}/demo_signing" \
  "${STAGE_DIR}/data/chain.index" \
  "${STAGE_DIR}/data/rdf" \
  "${STAGE_DIR}/data/store.nq" \
  "${STAGE_DIR}/data/wal.dat" \
  "${STAGE_DIR}/data/gs1_epcis_uht_demo"

PRESERVE_PATHS=(
  ".gitignore"
  "README.md"
  "CONTRIBUTING.md"
  "docs/INDEX.md"
  "docs/README.md"
  "docs/architecture/README.md"
  "docs/benchmarking/README.md"
)

for path in "${PRESERVE_PATHS[@]}"; do
  if [ -e "${PUBLIC_WORKTREE}/${path}" ]; then
    mkdir -p "$(dirname "${STAGE_DIR}/${path}")"
    cp -a "${PUBLIC_WORKTREE}/${path}" "${STAGE_DIR}/${path}"
  fi
done

rsync -a --delete --exclude='.git' --exclude='.git/' "${STAGE_DIR}/" "${PUBLIC_WORKTREE}/"

git -C "${PUBLIC_WORKTREE}" add -A

printf '\n[public-export] resulting status:\n'
git -C "${PUBLIC_WORKTREE}" status --short

if [ "${COMMIT_CHANGES}" = "true" ]; then
  if [ -z "$(git -C "${PUBLIC_WORKTREE}" status --porcelain)" ]; then
    printf '[public-export] no changes to commit\n'
  else
    git -C "${PUBLIC_WORKTREE}" commit -m "${COMMIT_MESSAGE}"
  fi
fi

printf '\n[public-export] push command when ready:\n'
printf 'git -C %q push public public-release:main\n' "${PUBLIC_WORKTREE}"
