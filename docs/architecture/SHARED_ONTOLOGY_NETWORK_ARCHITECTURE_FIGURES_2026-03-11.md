# Shared Ontology Network Architecture Figures - 2026-03-11

This document defines the current paper-ready architecture figure package for the shared-ontology permissioned-network model.

Source references:

- `docs/architecture/SHARED_ONTOLOGY_NETWORK_USE_CASES.md`
- `docs/architecture/ADR/0014-use-shared-ontology-packages-and-spacl-production-path.md`
- `scripts/generate_shared_ontology_architecture_figures.py`

Generated figure directory:

- `docs/architecture/figures/shared_ontology_network_2026-03-11/`

## Figure Inventory

### Figure 1. Shared-ontology layered architecture

Files:

- `figures/shared_ontology_network_2026-03-11/shared_ontology_layered_architecture.png`
- `figures/shared_ontology_network_2026-03-11/shared_ontology_layered_architecture.svg`

What it shows:

- consortium and organization actors,
- the network-wide contract split into `network profile` and `shared ontology package`,
- the node runtime with semantic enforcement in the production path,
- and the provenance, query, and audit outputs produced by the system.

Recommended caption:

> Layered architecture of the ProvChain shared-ontology permissioned network. Participating organizations in the same network share a common network profile and ontology package. Within each node, the production semantic path is implemented through `src/ontology/*` and SPACL-backed validation and reasoning before immutable RDF provenance is committed and exposed for traceability and audit.

### Figure 2. Ontology-backed admission pipeline

Files:

- `figures/shared_ontology_network_2026-03-11/shared_ontology_admission_pipeline.png`
- `figures/shared_ontology_network_2026-03-11/shared_ontology_admission_pipeline.svg`

What it shows:

- how local source data is mapped into shared semantics,
- where semantic contract checks happen,
- where SHACL and SPACL participate in admission,
- and how admitted data becomes committed provenance output.

Recommended caption:

> Ontology-backed admission pipeline in ProvChain. Local source data is mapped to the shared ontology contract, validated through `OntologyManager`, SHACL constraints, and SPACL-backed reasoning, and then committed through the selected consensus protocol before becoming queryable provenance and audit evidence.

## Generation Command

```bash
python3 scripts/generate_shared_ontology_architecture_figures.py
```

## Usage Notes

- These figures are aligned with the current architectural truth in the shared-ontology working plan and ADR 0014.
- They intentionally exclude legacy `src/semantic/*` OWL modules from the production architecture view.
- They treat `GS1/EPCIS` as an optional mapping layer inside the shared ontology package, not as the full architectural boundary of the platform.
