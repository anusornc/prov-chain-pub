# ProvChain

ProvChain is a Rust-based permissioned blockchain for traceability workloads that use a shared ontology package as the semantic contract across organizations. Nodes in the same network validate traceability events against the same ontology, SHACL shapes, and network profile before data is committed to the chain.

This public repository focuses on the executable system, reference demos, and reproducible benchmark artifacts. Internal review notes and journal submission working files are intentionally excluded.

## What It Does

- Runs a permissioned blockchain with configurable consensus (`PoA`, `PBFT` prototype)
- Uses a shared ontology package for semantic validation and provenance tracking
- Supports ontology-aware block admission through `src/ontology/*` and SPACL-backed reasoning
- Provides reference demos for UHT, healthcare, and pharmaceutical traceability flows
- Includes dataset normalization, projection, and benchmark tooling for public evaluation

## Architecture

ProvChain is organized around two contracts that all nodes in the same network must share:

1. `Network profile`
   Defines `network_id`, consensus settings, validator set, and other runtime compatibility checks.
2. `Ontology package`
   Defines ontology files, SHACL shapes, package version/hash, and validation mode.

The production semantic path is:

- `src/ontology/*`
- SPACL `owl2-reasoner`

Legacy modules under `src/semantic/*` are retained only for experimental and migration-oriented workflows.

## Repository Layout

```text
prov-chain-pub/
├── src/                    # Blockchain core, networking, ontology management
├── tests/                  # Integration tests
├── benches/                # Criterion benchmarks
├── config/                 # Example node/network/ontology package configuration
├── scripts/                # Data acquisition, normalization, projection, reporting
├── docs/architecture/      # Shared-ontology network model and architecture figures
├── docs/benchmarking/      # Benchmark methodology, tables, figures, CSV exports
└── examples/               # Reference demos
```

## Quick Start

### Prerequisites

- Rust 1.70+
- Cargo

### Clone

```bash
git clone https://github.com/anusornc/prov-chain-pub.git
cd prov-chain-pub
```

### Build

```bash
cargo build
```

### Run a Reference Demo

```bash
cargo run -- examples basic-supply-chain
```

Or run the UHT reference demo:

```bash
cargo run --example gs1_epcis_uht_demo
```

### Start a Node

```bash
cargo run -- start-node --config config/config.toml
```

## Configuration

Key public example configs:

- `config/config.toml`
- `config/network_profile.toml`
- `config/ontology_package.toml`

These files show how a node joins a shared-ontology network and validates local startup compatibility before participating.

## Examples

Built-in CLI examples:

- `cargo run -- examples list`
- `cargo run -- examples basic-supply-chain`
- `cargo run -- examples transaction-workflow`
- `cargo run -- examples owl2-reasoning`
- `cargo run -- examples web-server`
- `cargo run -- examples gs1-epcis-uht`

Standalone examples:

- `cargo run --example gs1_epcis_uht_demo`
- `cargo run --example demo_ui`
- `cargo run --example persistence_demo`

## Benchmarks and Public Data Pipeline

The repository includes:

- ontology admission benchmarks in `benches/`
- dataset acquisition manifests in `config/datasets/`
- data acquisition scripts in `scripts/data_acquisition/`
- normalization scripts in `scripts/data_normalization/`
- ontology-package projection scripts in `scripts/data_projection/`
- benchmark tables and figure assets in `docs/benchmarking/`

Start with:

- `docs/benchmarking/README.md`
- `docs/benchmarking/DOMAIN_DATASET_ADMISSION_BENCHMARK_2026-03-10.md`
- `docs/benchmarking/DOMAIN_DATASET_ADMISSION_SUMMARY_TABLES_2026-03-11.md`

## Documentation

- Architecture index: `docs/architecture/README.md`
- Shared ontology network model: `docs/architecture/SHARED_ONTOLOGY_NETWORK_USE_CASES.md`
- Architecture figures and diagrams: `docs/architecture/SHARED_ONTOLOGY_NETWORK_ARCHITECTURE_FIGURES_2026-03-11.md`
- Benchmarking index: `docs/benchmarking/README.md`

## Development Notes

- `src/ontology/*` is the production semantic path.
- `src/semantic/*` is experimental and should not be treated as the authoritative runtime path.
- UHT, healthcare, and pharmaceutical assets are reference ontology packages and demos, not the fixed scope of the platform.

## Contributing

See `CONTRIBUTING.md`.
