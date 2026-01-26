# File Placement Rules for AI Agents

## Context

This document defines the standard file organization for ProvChainOrg to ensure:
- Consistent project structure
- Easy navigation for developers
- Predictable locations for AI-generated content

## File Type Mappings

### Code Reviews
**Location:** `docs/reviews/`
**Pattern:** `*_CODE_REVIEW.md`, `*_review.md`
**Examples:**
- `PBFT_CONSENSUS_CODE_REVIEW.md`
- `CODE_REVIEW_PRODUCTION_FEATURES.md`
- `test_coverage_review_atomic_operations.md`

### Security Documentation
**Location:** `docs/security/`
**Pattern:** `SECURITY_*.md`, `*_SECURITY.md`
**Examples:**
- `SECURITY_SETUP.md`
- `SECURITY_TEST_COVERAGE_REPORT.md`
- `AUTH_DESIGN.md`

### Test Coverage & Quality
**Location:** `docs/project-health/`
**Pattern:** `*_COVERAGE_*.md`, `*_TEST_*.md`, `*_ANALYSIS.md`
**Examples:**
- `TEST_COVERAGE_REPORT.md`
- `clippy_analysis_*.md`
- `test_results_summary_*.md`

### Benchmarking
**Location:** `docs/benchmarking/`
**Pattern:** `*BENCHMARK*.md`, `*PERFORMANCE*.md`
**Note:** `BENCHMARKING.md` in root is a quick reference index

### Architecture
**Location:** `docs/architecture/`
**Pattern:** `ADR_*.md`, `ARCHITECTURE_*.md`

### Deployment
**Location:** `docs/deployment/`
**Pattern:** `docker-compose*.yml`, `*_deployment*.md`, `package.json`

## Decision Tree for File Placement

```
Creating a file?
├─ Is it a core project file (README, LICENSE, Cargo.toml)?
│  └─ YES → Root directory
├─ Is it a test coverage or code quality report?
│  └─ YES → docs/project-health/
├─ Is it a code review or analysis report?
│  └─ YES → docs/reviews/
├─ Is it security-related?
│  └─ YES → docs/security/
├─ Is it benchmarking or performance-related?
│  └─ YES → docs/benchmarking/
├─ Is it architecture or design-related?
│  └─ YES → docs/architecture/
├─ Is it deployment-related?
│  └─ YES → docs/deployment/
├─ Is it user-facing documentation?
│  └─ YES → docs/user-manual/
└─ Other documentation?
   └─ YES → docs/ (or most appropriate subdirectory)
```

## Prohibited Actions

AI agents MUST NOT:
1. Create new markdown files in root directory (except core project files)
2. Create top-level directories without justification
3. Scatter related files across multiple locations

AI agents MUST:
1. Check `FILE_ORGANIZATION.md` (root) before creating files
2. Group related content in appropriate subdirectories
3. Follow existing naming conventions
4. Update `docs/INDEX.md` when adding new documentation

## Quick Reference Summary

| Content Type | Target Directory |
|--------------|------------------|
| Code reviews | `docs/reviews/` |
| Security docs | `docs/security/` |
| Test coverage | `docs/project-health/` |
| Benchmarking | `docs/benchmarking/` |
| Architecture | `docs/architecture/` |
| Deployment | `docs/deployment/` |
| User guides | `docs/user-manual/` |
| Publication | `docs/publication/` |
| Other docs | `docs/` |

## Root Directory Files (Do Not Move)

These files MUST remain in the root directory:
- `README.md` - Project entry point
- `CONTRIBUTING.md` - Contributor guide (standard)
- `CLAUDE.md` - AI coding instructions
- `BENCHMARKING.md` - Quick reference (links to `docs/benchmarking/`)
- `CHANGES.md` - Local changelog (gitignored)

## For AI Agents Creating Content

Before creating any new file:
1. Read `FILE_ORGANIZATION.md` in the project root
2. Determine the file type using the decision tree above
3. Create the file in the appropriate `docs/` subdirectory
4. If unsure, default to `docs/` and organize by content type

**Example:**
- Agent creates code review → `docs/reviews/MODULE_CODE_REVIEW.md`
- Agent finds security issue → `docs/security/SECURITY_ISSUE_ANALYSIS.md`
- Agent generates test coverage → `docs/project-health/TEST_COVERAGE_ANALYSIS.md`
