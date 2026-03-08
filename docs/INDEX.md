# ProvChainOrg Documentation Index

This directory contains all the documentation for the ProvChainOrg blockchain traceability system.

## 📋 Project Overview
- **[Plan.md](Plan.md)** - Main project plan and roadmap
- **[../README.md](../README.md)** - Project introduction and setup instructions (at root level)
- **[Run.md](Run.md)** - How to run the system
- **[README.md](README.md)** - Documentation overview and index

## 🚀 User Guides
- **[USER_MANUAL.md](USER_MANUAL.md)** - User manual hub with comprehensive guides
- **[FAQ.md](FAQ.md)** - Frequently asked questions
- **[VERSION.md](VERSION.md)** - Version information

## 🏗️ Architecture Documentation
- **[architecture/README.md](architecture/README.md)** - Architecture documentation hub
  - C4 Model documentation (System Context, Container, Component)
  - Architectural Decision Records (ADRs)
  - Technology stack and quality attributes
- **[architecture/COMPONENT_OWNERSHIP.md](architecture/COMPONENT_OWNERSHIP.md)** - Component ownership matrix

## 🔧 Deployment Guides
- **[deployment/HANDS_ON_DEPLOYMENT_GUIDE.md](deployment/HANDS_ON_DEPLOYMENT_GUIDE.md)** - Comprehensive deployment guide
- **[deployment/DOCKER_DEPLOYMENT_ARCHITECTURE.md](deployment/DOCKER_DEPLOYMENT_ARCHITECTURE.md)** - Docker deployment architecture

## 🔒 Security Documentation
- **[security/](security/)** - Security documentation and reports
  - Security setup and configuration
  - Security test coverage reports

## 📝 Code Reviews
- **[reviews/](reviews/)** - Code review and analysis reports
  - Production feature reviews
  - Consensus implementation reviews
  - Test coverage reviews

## 📊 Benchmarking & Performance
- **[benchmarking/README.md](benchmarking/README.md)** - Research-focused benchmarking guide
- **[benchmarking/EXPERIMENTAL_RESULTS.md](benchmarking/EXPERIMENTAL_RESULTS.md)** - Real experimental results
- **[../BENCHMARKING.md](../BENCHMARKING.md)** - Central benchmarking entry point

## 📚 Research & Publication
- **[publication/README.md](publication/README.md)** - Publication package for journals
- **[publication/BASELINE_QUICKSTART.md](publication/BASELINE_QUICKSTART.md)** - Quick start for baseline comparisons

## 📖 Developer Resources
- **[../CONTRIBUTING.md](../CONTRIBUTING.md)** - Contributor guide and development setup
- **[../CLAUDE.md](../CLAUDE.md)** - Project instructions and patterns

## 📁 Directory Structure

```
docs/
├── README.md                           # Documentation overview
├── INDEX.md                            # This file (navigation index)
├── Plan.md                             # Project plan
├── Run.md                              # Execution guide
├── USER_MANUAL.md                      # User manual hub
├── FAQ.md                              # FAQ
│
├── architecture/                       # Architecture documentation
│   ├── README.md                       # C4 model documentation
│   ├── ADR/                            # Architecture Decision Records
│   └── COMPONENT_OWNERSHIP.md          # Component ownership matrix
│
├── deployment/                         # Deployment guides
│   ├── HANDS_ON_DEPLOYMENT_GUIDE.md    # Step-by-step deployment
│   └── DOCKER_DEPLOYMENT_ARCHITECTURE.md
│
├── security/                           # Security documentation
│   ├── SECURITY_SETUP.md               # Security setup guide
│   └── SECURITY_TEST_COVERAGE_REPORT.md
│
├── reviews/                            # Code review reports
│   ├── CODE_REVIEW_PRODUCTION_FEATURES.md
│   ├── PBFT_CONSENSUS_CODE_REVIEW.md
│   └── test_coverage_review_atomic_operations.md
│
├── benchmarking/                       # Performance testing
│   ├── README.md
│   └── EXPERIMENTAL_RESULTS.md
│
├── publication/                        # Research materials
│   ├── README.md
│   └── BASELINE_QUICKSTART.md
│
├── user-manual/                        # Detailed user manual
│   ├── README.md
│   ├── 00-quick-start/
│   ├── 03-querying-data/
│   └── 08-troubleshooting/
│
├── tutorials/                          # Tutorials
├── developer/                          # Developer documentation
├── project-health/                     # Project health analysis
│
└── archive/                            # Historical documents
    ├── phases/                         # Phase implementation records
    ├── status/                         # Progress tracking
    ├── old-architecture/               # Superseded architecture docs
    ├── implementation-plans/           # Experimental technical docs
    ├── experimental/                   # AI model docs
    ├── historical-reports/             # Historical reports
    └── obsolete-folders/               # Outdated documentation folders
```

## 🚀 Quick Start

1. Read **[../README.md](../README.md)** for project overview
2. Check **[Plan.md](Plan.md)** for detailed roadmap
3. Follow **[Run.md](Run.md)** for execution instructions
4. Review **[USER_MANUAL.md](USER_MANUAL.md)** for usage guides

## 📊 Current Project Health (March 2026)

| Metric | Status |
|--------|--------|
| Latest Validation Run | 2026-03-08 ✅ |
| Backend Regression Check | Passed (sandbox-only permission issues isolated) ✅ |
| Frontend CI | 86/86 tests passed + production build passed ✅ |
| OWL2 Dependency Status | SPACL `owl2-reasoner` git dependency ✅ |
| Main Project Clippy | 205 low-severity warnings |

**Latest Reports**:
- `project-health/spacl_migration_validation_2026-03-08.md` - Post-migration validation and benchmark summary
- `project-health/test_results_summary_2026-01-26.md` - Complete test results
- `project-health/clippy_analysis_2026-01-26.md` - Clippy warnings breakdown

**Latest Validation Snapshot (2026-03-08)**:
- ✅ **SPACL Migration Verified**: Local `owl2-reasoner/` removed; external Git dependency confirmed
- ✅ **Backend Validation**: Workspace tests passed except sandbox-restricted cases; restricted cases pass outside sandbox
- ✅ **Frontend Validation**: Jest and production build both passed
- 📊 **Benchmark Coverage**: Consensus + OWL2 benchmark suites recorded; trace optimization benchmark partially recorded with follow-up noted

See `project-health/spacl_migration_validation_2026-03-08.md` for details.

The ProvChainOrg project documentation has been reorganized:
- ✅ **Current documentation** aligned with codebase state
- ✅ **Historical documents** preserved in `archive/` directory
- ✅ **Duplicate content** removed and consolidated
- ✅ **C4 Model architecture** documentation up-to-date

### Key Resources
- **Architecture**: See `architecture/README.md` for C4 model documentation
- **Performance**: See `benchmarking/EXPERIMENTAL_RESULTS.md` for real experimental data
- **Deployment**: See `deployment/HANDS_ON_DEPLOYMENT_GUIDE.md` for setup
- **Research**: See `publication/README.md` for journal submission materials

## 🔍 Archived Content

Historical implementation phases, status tracking, experimental features, and obsolete technical documentation have been moved to the `archive/` directory. These documents are preserved for reference but may not reflect the current codebase state.
