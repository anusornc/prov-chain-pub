# ProvChain Architecture Documentation

This directory contains the public architecture documentation for ProvChain.

The architecture is centered on a permissioned traceability network in which participating nodes share:

1. a `network profile`
2. an `ontology package`

Together, these form the runtime contract for interoperability, validation, and provenance handling.

## Recommended Documents

- [SYSTEM_CONTEXT.md](SYSTEM_CONTEXT.md) - high-level system scope and actors
- [CONTAINER_ARCHITECTURE.md](CONTAINER_ARCHITECTURE.md) - major runtime containers and deployment view
- [DATA_FLOW_ARCHITECTURE.md](DATA_FLOW_ARCHITECTURE.md) - admission, validation, and persistence flow
- [SHARED_ONTOLOGY_NETWORK_USE_CASES.md](SHARED_ONTOLOGY_NETWORK_USE_CASES.md) - canonical shared-ontology lifecycle and use cases
- [SHARED_ONTOLOGY_NETWORK_ARCHITECTURE_FIGURES_2026-03-11.md](SHARED_ONTOLOGY_NETWORK_ARCHITECTURE_FIGURES_2026-03-11.md) - generated figure package used in public-facing architecture explanations
- [ADR/README.md](ADR/README.md) - architectural decision records
- [ADR/0014-use-shared-ontology-packages-and-spacl-production-path.md](ADR/0014-use-shared-ontology-packages-and-spacl-production-path.md) - production semantic path decision

## Notes

- The production semantic path is `src/ontology/*` plus SPACL `owl2-reasoner`.
- Legacy `src/semantic/*` modules are retained for experimental and migration-oriented workflows, not as the primary runtime truth.
- UHT, healthcare, and pharmaceutical assets are reference ontology packages and demos, not the architectural boundary of the platform.
