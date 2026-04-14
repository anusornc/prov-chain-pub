# Normalization Schema - 2026-03-10

## Purpose

This document defines the first normalized record shape for external datasets before they are converted into ontology-package-specific provenance graphs.

The schema is intentionally intermediate. It is not the final RDF form. Its role is to:

1. preserve important source fields,
2. align cross-domain data to a stable provenance-oriented shape,
3. and reduce coupling between raw public datasets and ontology-package emitters.

## Design Rules

- one normalized record should describe one logical entity, event, or incident
- source fidelity must be preserved through `source_record`
- cross-domain provenance roles must be explicit
- ontology-package emitters can enrich, split, or merge records later

## Common Normalized Record Shape

Each normalized record should contain:

```json
{
  "schema_version": "1.0",
  "record_id": "string",
  "source_id": "string",
  "domain": "food|pharma|device",
  "record_type": "entity|activity|incident|agent|evidence",
  "core": {
    "entity_id": "string or null",
    "entity_type": "string or null",
    "agent_id": "string or null",
    "agent_type": "string or null",
    "activity_id": "string or null",
    "activity_type": "string or null",
    "occurred_at": "RFC3339 timestamp or null",
    "published_at": "RFC3339 timestamp or null"
  },
  "identifiers": {
    "primary": "string",
    "secondary": ["..."]
  },
  "attributes": {
    "name": "string or null",
    "status": "string or null",
    "classification": "string or null",
    "lot_or_batch": "string or null",
    "catalog_code": "string or null"
  },
  "relationships": [
    {
      "predicate": "string",
      "target_id": "string",
      "target_type": "string"
    }
  ],
  "source_record": {
    "dataset_name": "string",
    "source_url": "string",
    "retrieved_at": "RFC3339 timestamp",
    "raw_snapshot_id": "string"
  }
}
```

## PROV-O Alignment

The normalized record is designed to map into PROV-O and the shared ontology package model like this:

| Normalized field | PROV-O / semantic role |
|---|---|
| `core.entity_id` | `prov:Entity` identifier |
| `core.agent_id` | `prov:Agent` identifier |
| `core.activity_id` | `prov:Activity` identifier |
| `core.occurred_at` | event time, often `prov:generatedAtTime` or domain event time |
| `relationships` | ontology-package predicates or provenance links |
| `source_record` | evidence and provenance-of-provenance metadata |

## Domain-Specific Guidance

### Food

Expected normalized record types:

- product entity
- manufacturer or brand owner agent
- recall incident
- standards-aligned EPCIS activity

Recommended identifier priority:

- GTIN when available
- FDC ID
- branded product code
- internal synthetic batch ID

### Pharmaceutical

Expected normalized record types:

- drug product entity
- labeler or manufacturer agent
- shortage incident
- recall or enforcement incident

Recommended identifier priority:

- NDC
- set ID or application number when needed
- synthetic lot ID for provenance-chain generation

### Device

Expected normalized record types:

- device entity
- manufacturer agent
- enforcement or recall incident

Recommended identifier priority:

- UDI-DI
- proprietary device identifier when required
- synthetic shipment or lot ID for provenance-chain generation

## Emission into Ontology Packages

The next layer after normalization should emit:

1. core PROV-O entities, agents, and activities
2. domain package classes and predicates
3. optional GS1 or external-standard mappings
4. benchmark scenario metadata

This means normalization should stay domain-aware, but ontology-package emission should remain the only layer that decides final RDF classes and predicates.

## Reference Examples

Example normalized records are stored in:

- `config/datasets/normalized_record_examples.json`

These examples cover:

- one food product entity,
- one pharmaceutical shortage incident,
- one device recall incident.

## Next Implementation Step

Add normalizer scripts that produce this shape from:

1. FoodData Central
2. openFDA Drug NDC / Drug Shortages
3. AccessGUDID / Device Enforcement

Those scripts should not generate RDF directly in the first pass. They should first produce normalized JSON artifacts that can be validated and diffed independently.

Update on 2026-03-10:

- first-pass normalizer scripts now exist under `scripts/data_normalization/`
- sample raw inputs now exist under `config/datasets/raw_examples/`
- next work should project normalized records into ontology-package-specific RDF or transaction payloads
