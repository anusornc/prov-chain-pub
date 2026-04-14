# Data Projection Scripts

These scripts project normalized records into ontology-admission-friendly artifacts.

Current script:

- `project_normalized_to_turtle.py`
- `emit_ontology_package_turtle.py`
- `ontology_package_emitters.py`
- `synthesize_uht_product_events.py`
- `synthesize_uht_epcis_events.py`
- `synthesize_healthcare_device_events.py`
- `synthesize_pharma_storage_events.py`

Example usage:

```bash
python3 scripts/data_projection/project_normalized_to_turtle.py \
  --input /tmp/openfda_drug_ndc.normalized.json \
  --output /tmp/openfda_drug_ndc.projected.ttl
```

The current projection is intentionally conservative:

- it emits `core:Participant`
- it emits a domain entity with a minimal core or healthcare class
- it emits `core:Transaction` plus `prov:Activity`

This is a projection prototype for ontology admission and benchmark wiring, not a finalized publication-grade RDF emitter.

Ontology-package-specific emission is now available for the benchmark-facing packages:

```bash
python3 scripts/data_projection/emit_ontology_package_turtle.py \
  --package uht_epcis \
  --input /tmp/fooddata_central.normalized.json \
  --limit 1 \
  --output /tmp/fooddata_central.uht_epcis.ttl
```

Supported packages:

- `uht`
- `uht_epcis`
- `healthcare_device`
- `pharma_storage`

Use `--limit 1` when you want a single-record benchmark payload instead of emitting all normalized records in the input snapshot.

Synthetic event projections are also available for the current domain SHACL paths:

```bash
python3 scripts/data_projection/synthesize_uht_product_events.py \
  --input /tmp/fooddata_central.normalized.json \
  --output /tmp/fooddata_central.uht.ttl
```

```bash
python3 scripts/data_projection/synthesize_uht_epcis_events.py \
  --input /tmp/fooddata_central.normalized.json \
  --output /tmp/fooddata_central.uht_epcis.ttl
```

```bash
python3 scripts/data_projection/synthesize_healthcare_device_events.py \
  --input /tmp/accessgudid.normalized.json \
  --output /tmp/accessgudid.healthcare.ttl
```

```bash
python3 scripts/data_projection/synthesize_pharma_storage_events.py \
  --input /tmp/openfda_drug_shortages.normalized.json \
  --output /tmp/openfda_drug_shortages.storage.ttl
```

These package emitters and synthetic projections are derived from normalized public records and are intended for reproducible benchmark scenarios, not for claiming direct one-to-one recovery of original operational events.
