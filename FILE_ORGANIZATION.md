# ProvChainOrg File Organization Standards

## Quick Reference

| File Type | Location | Examples |
|-----------|----------|----------|
| Code Reviews | `docs/reviews/` | `*_CODE_REVIEW.md` |
| Security Docs | `docs/security/` | `SECURITY_*.md` |
| Test Coverage | `docs/project-health/` | `*_COVERAGE_REPORT.md` |
| Architecture | `docs/architecture/` | `ADR_*.md` |
| Benchmarking | `docs/benchmarking/` | `*BENCHMARK*.md` |
| Deployment | `docs/deployment/` | `docker-compose*`, `package.json` |
| User Guides | `docs/user-manual/` | Usage guides |

## Root Directory (Keep Minimal)

**Files that MUST stay in root:**
- `README.md` - Project entry point
- `CONTRIBUTING.md` - Contributor guide
- `CLAUDE.md` - AI coding instructions
- `BENCHMARKING.md` - Quick reference (links to `docs/benchmarking/`)
- `CHANGES.md` - Local changelog (gitignored)

**Files that MUST NOT be in root:**
- Code review reports
- Security documentation
- Test coverage reports
- Temporary analysis files

## Directory Creation Rules

Before creating a new directory in root, ask:
1. Is this documentation? → Put in `docs/`
2. Is this a code review? → Put in `docs/reviews/`
3. Is this security-related? → Put in `docs/security/`
4. Is this deployment-related? → Put in `docs/deployment/`

## AI Agent Guidelines

When creating new files:
1. Check this `FILE_ORGANIZATION.md` first
2. Use `docs/reviews/` for any analysis/review reports
3. Use `docs/security/` for security findings
4. Use `docs/project-health/` for test coverage/metrics
5. NEVER create new markdown files in root unless they are core project files

## File Type Mappings

### Code Reviews
**Location:** `docs/reviews/`
**Pattern:** `*_CODE_REVIEW.md`, `*_review.md`

### Security Documentation
**Location:** `docs/security/`
**Pattern:** `SECURITY_*.md`, `*_SECURITY.md`

### Test Coverage & Quality
**Location:** `docs/project-health/`
**Pattern:** `*_COVERAGE_*.md`, `*_TEST_*.md`, `*_ANALYSIS.md`

### Benchmarking
**Location:** `docs/benchmarking/`
**Pattern:** `*BENCHMARK*.md`, `*PERFORMANCE*.md`

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

## Rule of Thumb

**If it's a report, review, or analysis → it goes in `docs/` subdirectory.**
