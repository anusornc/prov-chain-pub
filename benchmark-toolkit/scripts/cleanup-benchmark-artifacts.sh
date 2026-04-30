#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

APPLY="false"
FAILED_REMOVALS=0

usage() {
  cat <<'EOF_USAGE'
Usage:
  benchmark-toolkit/scripts/cleanup-benchmark-artifacts.sh [--apply]

Removes local generated benchmark/runtime artifacts that are not publication
evidence. Dry-run is the default. Use --apply to delete.

Preserved tracked files:
  benchmark-toolkit/results/README.md
  benchmark-toolkit/results/campaigns/INDEX.md

Curated evidence under docs/benchmarking/data/ is not touched.
EOF_USAGE
}

while [ $# -gt 0 ]; do
  case "$1" in
    --apply)
      APPLY="true"
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      printf 'error: unknown option: %s\n' "$1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

tracked_guard() {
  local path="$1"
  if git -C "${REPO_ROOT}" ls-files --error-unmatch "${path}" >/dev/null 2>&1; then
    printf 'error: refusing to remove tracked path: %s\n' "${path}" >&2
    exit 1
  fi
}

remove_path() {
  local rel_path="$1"
  local abs_path="${REPO_ROOT}/${rel_path}"

  if [ ! -e "${abs_path}" ]; then
    return
  fi

  tracked_guard "${rel_path}"

  if [ "${APPLY}" = "true" ]; then
    if rm -rf "${abs_path}" 2>/dev/null; then
      printf '[cleanup] removed %s\n' "${rel_path}"
    elif [ -d "${abs_path}" ] && command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
      docker run --rm -v "${abs_path}:/cleanup-target" busybox \
        sh -c 'find /cleanup-target -mindepth 1 -maxdepth 1 -exec rm -rf {} +' >/dev/null
      rmdir "${abs_path}" 2>/dev/null || true
      if [ -e "${abs_path}" ] && [ -n "$(find "${abs_path}" -mindepth 1 -print -quit 2>/dev/null)" ]; then
        printf 'warning: failed to remove Docker-owned path: %s\n' "${rel_path}" >&2
        FAILED_REMOVALS=$((FAILED_REMOVALS + 1))
        return
      fi
      printf '[cleanup] removed %s using docker root fallback\n' "${rel_path}"
    else
      printf 'warning: failed to remove %s\n' "${rel_path}" >&2
      FAILED_REMOVALS=$((FAILED_REMOVALS + 1))
      return
    fi
  else
    printf '[cleanup] would remove %s\n' "${rel_path}"
  fi
}

remove_results_children() {
  local results_dir="${REPO_ROOT}/benchmark-toolkit/results"
  local campaigns_dir="${results_dir}/campaigns"
  local entry

  if [ -d "${campaigns_dir}" ]; then
    while IFS= read -r entry; do
      remove_path "${entry#${REPO_ROOT}/}"
    done < <(find "${campaigns_dir}" -mindepth 1 -maxdepth 1 ! -name INDEX.md -print | sort)
  fi

  if [ -d "${results_dir}" ]; then
    while IFS= read -r entry; do
      case "$(basename "${entry}")" in
        README.md|campaigns)
          ;;
        *)
          remove_path "${entry#${REPO_ROOT}/}"
          ;;
      esac
    done < <(find "${results_dir}" -mindepth 1 -maxdepth 1 -print | sort)
  fi
}

print_size() {
  printf '[cleanup] current generated artifact size:\n'
  du -sh \
    "${REPO_ROOT}/benchmark-toolkit/results" \
    "${REPO_ROOT}/benchmark-toolkit/datasets/translated" \
    "${REPO_ROOT}/data/rdf" \
    "${REPO_ROOT}/data/chain.index" \
    "${REPO_ROOT}/data/store.nq" \
    "${REPO_ROOT}/data/wal.dat" \
    "${REPO_ROOT}/docs/paper_submission/overleaf_upload/build" \
    2>/dev/null || true
}

print_size

remove_results_children
remove_path "benchmark-toolkit/datasets/translated"
remove_path "data/rdf"
remove_path "data/chain.index"
remove_path "data/store.nq"
remove_path "data/wal.dat"
remove_path "docs/paper_submission/overleaf_upload/build"

if [ "${APPLY}" != "true" ]; then
  printf '\n[cleanup] dry run only; rerun with --apply to delete.\n'
else
  if [ "${FAILED_REMOVALS}" -ne 0 ]; then
    printf '\n[cleanup] completed with %s path(s) requiring elevated cleanup.\n' "${FAILED_REMOVALS}" >&2
    exit 1
  fi
  printf '\n[cleanup] complete. Preserved curated evidence under docs/benchmarking/data/.\n'
fi
