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
  - Shared ontology network use-case reference
  - Shared ontology network working plan
  - Shared ontology network paper-ready architecture figures
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
  - Top-journal repositioning and novelty roadmap
  - Structural audit for node startup, config, and ontology runtime

## 📊 Benchmarking & Performance
- **[benchmarking/README.md](benchmarking/README.md)** - Research-focused benchmarking guide
- **[benchmarking/EXPERIMENTAL_RESULTS.md](benchmarking/EXPERIMENTAL_RESULTS.md)** - Real experimental results
- **[benchmarking/ONTOLOGY_ADMISSION_BENCHMARK_2026-03-10.md](benchmarking/ONTOLOGY_ADMISSION_BENCHMARK_2026-03-10.md)** - Focused benchmark for ontology admission, subclass reasoning, and explanation summaries
- **[benchmarking/DATASET_ACQUISITION_PLAN_2026-03-10.md](benchmarking/DATASET_ACQUISITION_PLAN_2026-03-10.md)** - Official-source dataset strategy for standards examples, public registries, incident data, and reproducible synthesis
- **[benchmarking/NORMALIZATION_SCHEMA_2026-03-10.md](benchmarking/NORMALIZATION_SCHEMA_2026-03-10.md)** - Intermediate normalization schema for external datasets before ontology-package emission
- **[benchmarking/DOMAIN_DATASET_ADMISSION_BENCHMARK_2026-03-10.md](benchmarking/DOMAIN_DATASET_ADMISSION_BENCHMARK_2026-03-10.md)** - Benchmark artifact for UHT, hybrid GS1/EPCIS-UHT, healthcare, and pharmaceutical synthetic events, scaling curves, and cross-package workloads through ontology-backed block admission
- **[benchmarking/DOMAIN_DATASET_ADMISSION_SUMMARY_TABLES_2026-03-11.md](benchmarking/DOMAIN_DATASET_ADMISSION_SUMMARY_TABLES_2026-03-11.md)** - Publication-ready Markdown tables generated from Criterion benchmark estimates
- **[benchmarking/DOMAIN_DATASET_ADMISSION_FIGURES_2026-03-11.md](benchmarking/DOMAIN_DATASET_ADMISSION_FIGURES_2026-03-11.md)** - Publication-ready figure package with reproducible PNG/SVG outputs and paper caption guidance
- **[benchmarking/data/README.md](benchmarking/data/README.md)** - CSV exports for plotting and reproducing the domain dataset admission figures
- **[architecture/SHARED_ONTOLOGY_NETWORK_ARCHITECTURE_FIGURES_2026-03-11.md](architecture/SHARED_ONTOLOGY_NETWORK_ARCHITECTURE_FIGURES_2026-03-11.md)** - Publication-ready layered architecture and semantic admission pipeline figures for the shared-ontology network model
- **[../config/datasets/acquisition_manifests/README.md](../config/datasets/acquisition_manifests/README.md)** - Benchmark snapshot manifests for the raw sample fixtures used by normalization and ontology admission
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
│   ├── SHARED_ONTOLOGY_NETWORK_USE_CASES.md
│   ├── SHARED_ONTOLOGY_NETWORK_WORKING_PLAN.md
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
│   ├── STRUCTURAL_AUDIT_NODE_CONFIG_ONTOLOGY_2026-03-09.md
│   ├── TOP_JOURNAL_REPOSITIONING_ROADMAP.md
│   └── test_coverage_review_atomic_operations.md
│
├── benchmarking/                       # Performance testing
│   ├── README.md
│   ├── EXPERIMENTAL_RESULTS.md
│   └── ONTOLOGY_ADMISSION_BENCHMARK_2026-03-10.md
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
- `reviews/TOP_JOURNAL_REPOSITIONING_ROADMAP.md` - Research repositioning, novelty, and implementation roadmap for top-tier submission
- `reviews/STRUCTURAL_AUDIT_NODE_CONFIG_ONTOLOGY_2026-03-09.md` - Current execution-path audit for startup, config, consensus, and ontology wiring
- `architecture/ADR/0014-use-shared-ontology-packages-and-spacl-production-path.md` - Architectural decision for the production semantic path and ontology-package model
- `architecture/SHARED_ONTOLOGY_NETWORK_USE_CASES.md` - Canonical actors, lifecycle, boundaries, and end-to-end flows for the shared-ontology network model
- `architecture/SHARED_ONTOLOGY_NETWORK_WORKING_PLAN.md` - Long-lived working memory for the shared ontology network design
- `benchmarking/ONTOLOGY_ADMISSION_BENCHMARK_2026-03-10.md` - First focused benchmark artifact for production ontology admission
- `benchmarking/DATASET_ACQUISITION_PLAN_2026-03-10.md` - Dataset strategy for real public sources, standards examples, and synthesis-driven evaluation
- `benchmarking/NORMALIZATION_SCHEMA_2026-03-10.md` - First normalized-record schema for external food, pharmaceutical, and device datasets
- `benchmarking/DOMAIN_DATASET_ADMISSION_BENCHMARK_2026-03-10.md` - Dataset-derived domain admission benchmark with split setup, single-record admission, batch admission, scaling-curve metrics, and cross-package round-robin workloads across UHT, hybrid GS1/EPCIS-UHT, healthcare, and pharmaceutical paths
- `benchmarking/DOMAIN_DATASET_ADMISSION_SUMMARY_TABLES_2026-03-11.md` - Paper-ready summary tables generated from Criterion results for direct inclusion in manuscript drafts
- `benchmarking/data/domain_dataset_admission_summary_2026-03-11.csv` - Machine-readable export of all domain dataset admission metrics
- `benchmarking/data/domain_dataset_admission_plot_data_2026-03-11.csv` - Figure-ready benchmark data for grouped bar charts and scaling plots

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
