# Dataset Acquisition Plan - 2026-03-10

## Purpose

This document defines the dataset strategy for the next journal-facing benchmark phase of ProvChain.

The goal is not to claim access to fully open, real inter-organizational traceability ledgers. Those are rarely public. Instead, the evaluation package should combine:

1. official standards-aligned example event data,
2. official public registry and incident data,
3. reproducible synthesis of end-to-end provenance events,
4. and ontology-package mappings into the ProvChain shared-ontology model.

This matches the current architecture:

- `PROV-O` as the foundational provenance layer,
- shared ontology packages as the network semantic contract,
- and `src/ontology/*` + SPACL as the production semantic path.

## Evaluation Roles

### 1. Standards-aligned event layer

Use official GS1 EPCIS example data to validate event-shape compatibility and external standards mapping.

Primary source:

- GS1 EPCIS / CBV examples: <https://ref.gs1.org/docs/epcis/examples>

Why this source:

- official GS1 reference material,
- JSON / JSON-LD / XML examples,
- appropriate for standards-aligned event ingestion and mapping,
- but not a substitute for full real-world supply-chain history.

### 2. Real public identifier and master-data layer

Use official public registries to represent products, drugs, and devices that can be mapped into ontology packages.

Primary sources:

- USDA FoodData Central downloads: <https://fdc.nal.usda.gov/download-datasets/>
- openFDA Drug NDC API: <https://open.fda.gov/apis/drug/ndc/>
- AccessGUDID downloads: <https://accessgudid.nlm.nih.gov/download>
- openFDA Device UDI API: <https://open.fda.gov/apis/device/udi/>

Why these sources:

- official and public,
- stable identifiers exist,
- update frequencies are documented,
- suitable for reproducible acquisition and field mapping.

### 3. Real public incident / disruption layer

Use recall and shortage data to evaluate traceability queries, disruption propagation, and semantic linking between provenance records and safety events.

Primary sources:

- USDA FSIS Recall API: <https://www.fsis.usda.gov/science-data/developer-resources/recall-api>
- openFDA Food Enforcement API: <https://open.fda.gov/apis/food/enforcement/>
- openFDA Drug Shortages API: <https://open.fda.gov/apis/drug/drugshortages/>
- openFDA Device Enforcement API: <https://open.fda.gov/apis/device/enforcement/>

Why these sources:

- official public safety or enforcement datasets,
- machine-readable APIs,
- suitable for evaluation of recall-linked provenance and exception tracing.

### 4. Synthetic end-to-end event layer

Use reproducible synthesis to generate multi-step provenance sequences from the official sources above.

Optional source for healthcare-oriented synthetic records:

- Synthea: <https://synthetichealth.github.io/synthea/>

Why synthesis is still required:

- fully open cross-organization provenance logs are uncommon,
- official public registries do not contain the full event chain needed for end-to-end blockchain admission tests,
- the synthesis step lets us publish generation scripts and reproduce scenarios exactly.

## Domain Packages and Recommended Source Bundles

### A. Food / UHT reference package

Recommended bundle:

- GS1 EPCIS example events
- USDA FoodData Central
- USDA FSIS Recall API
- optional openFDA Food Enforcement API for FDA-regulated food categories outside FSIS scope

Role in evaluation:

- standards mapping,
- product and batch master data,
- recall-linked provenance queries,
- UHT demo continuity with a stronger public-data basis.

### B. Pharmaceutical reference package

Recommended bundle:

- openFDA Drug NDC
- openFDA Drug Shortages
- optional openFDA Drug Enforcement / recalls when recall linkage is needed: <https://open.fda.gov/apis/drug/enforcement/>

Role in evaluation:

- drug identifier normalization,
- shortage-aware provenance reasoning,
- semantic linking of product identifiers to disruption events.

### C. Medical device / healthcare asset reference package

Recommended bundle:

- AccessGUDID
- openFDA Device UDI
- openFDA Device Enforcement

Optional extension:

- Synthea only for synthetic patient-context demonstrations, not for device supply-chain truth.

Role in evaluation:

- device identifier normalization,
- provenance-to-recall linkage,
- network-package portability beyond food and pharma.

## Acquisition Priorities

### Priority 1: Immediate benchmark-ready sources

These should be acquired first because they are official, machine-readable, and low-friction:

1. GS1 EPCIS example files
2. USDA FoodData Central sample release
3. openFDA Drug NDC sample pull
4. AccessGUDID monthly or daily sample release
5. openFDA Drug Shortages sample pull
6. openFDA Device Enforcement sample pull

### Priority 2: Expansion sources

These improve coverage but are not required to start:

1. USDA FSIS Recall API historical pulls
2. openFDA Food Enforcement
3. openFDA Device UDI supplemental pulls
4. Synthea export samples for healthcare-context demos

## Recommended Storage Layout

Create a reproducible dataset workspace under `datasets/` or `data/external/` once implementation begins.

Recommended structure:

```text
data/external/
├── raw/
│   ├── gs1_epcis_examples/
│   ├── usda_fooddata_central/
│   ├── fsis_recalls/
│   ├── openfda_drug_ndc/
│   ├── openfda_drug_shortages/
│   ├── accessgudid/
│   ├── openfda_device_udi/
│   ├── openfda_device_enforcement/
│   └── synthea_optional/
├── normalized/
│   ├── food/
│   ├── pharma/
│   └── device/
└── synthesized/
    ├── food_traceability/
    ├── pharma_traceability/
    └── device_traceability/
```

Raw files should remain immutable after download. Normalized outputs and synthesized provenance graphs should record:

- source URL,
- retrieval date,
- source release date,
- local file hash,
- normalization script version,
- ontology package version,
- and benchmark scenario ID.

## Minimal Field Mapping into PROV-O and Ontology Packages

### Core cross-domain mapping

The following mapping should stay stable across domains:

| Public data concept | PROV-O / core traceability role | Notes |
|---|---|---|
| product / device / drug identifier | `prov:Entity` | domain package adds subtype |
| manufacturer / firm / labeler | `prov:Agent` | may also map to organization class |
| recall / shortage / enforcement action | `prov:Activity` or governed event class | depends on ontology package |
| publication / notification time | `prov:generatedAtTime` or event timestamp | keep original source timestamp too |
| affected lot / batch / catalog code | entity attribute or linked identifier | package-specific |
| source registry record | provenance evidence entity | preserve source URL and dataset name |

### Food package

- FoodData Central food item or branded food -> food entity
- GS1 EPCIS event example -> traceability activity/event
- FSIS or FDA recall record -> recall activity linked to affected entity or batch

### Pharmaceutical package

- NDC product -> drug product entity
- shortage record -> supply disruption activity
- enforcement or recall record -> regulatory event linked to product or lot

### Device package

- GUDID / UDI device identifier -> medical device entity
- device enforcement record -> recall or corrective action activity
- manufacturer record -> agent

## Acquisition Method

### Standards examples

- vendor or standards-hosted static files should be copied into `raw/`
- preserve original filenames and formats
- record the exact retrieval date in a manifest

### API sources

- use deterministic query snapshots
- keep a checked-in acquisition manifest, but do not commit bulky raw dumps unless the publication package explicitly requires a sample
- store API query strings and response hashes
- prefer small, representative snapshot files for repository inclusion

### Large downloadable releases

- acquire one stable monthly release for reproducible publication artifacts
- optionally acquire daily or weekly releases for freshness-sensitive experiments
- avoid using only rolling latest data in journal figures

## Benchmark and Evaluation Design

### Track A: Standards alignment

Use GS1 EPCIS examples to evaluate:

- parsing and ingestion success,
- ontology-package mapping completeness,
- SHACL validation pass or fail behavior,
- explanation quality for invalid or unsupported cases.

### Track B: Cross-domain generalization

Use food, pharma, and device packages to evaluate:

- same blockchain core,
- same network-profile model,
- same ontology-aware admission path,
- different ontology packages and datasets.

### Track C: Incident-linked provenance

Use recalls and shortages to evaluate:

- batch or product lookup,
- provenance trace reconstruction,
- anomaly or disruption linkage,
- explanation metadata under invalid or incomplete data.

### Track D: Synthesized end-to-end network benchmarks

Build reproducible event chains from the public sources above to evaluate:

- block admission latency,
- ontology validation cost,
- subclass reasoning impact,
- explanation-summary overhead,
- and end-to-end trace query performance.

## What this plan should not claim

This acquisition plan does not justify claiming:

- access to fully real inter-organizational blockchain histories,
- full GS1 conformance,
- complete domain coverage,
- or direct clinical correctness from openFDA or Synthea data.

The correct claim is narrower:

- ProvChain is evaluated with official standards-aligned examples, official public registries and incident datasets, and reproducible synthesized provenance scenarios across multiple ontology packages.

## Immediate Next Steps

1. Add acquisition scripts for a small reproducible snapshot from each Priority 1 source.
2. Create a machine-readable acquisition manifest format with retrieval date, source URL, and file hash.
3. Build normalization scripts that map food, pharma, and device records into ontology-package input records.
4. Add domain benchmark scenarios that consume these normalized and synthesized datasets through the production ontology path.

Update on 2026-03-10:

- the initial acquisition manifest template now exists at `config/datasets/acquisition_manifest_template.toml`
- the initial source catalog now exists at `config/datasets/priority1_sources.toml`
- the first acquisition scripts now exist under `scripts/data_acquisition/`
- normalization and benchmark-consumption layers are still pending

## Source Notes Verified on 2026-03-10

- GS1 EPCIS examples page lists non-normative examples in XML and JSON / JSON-LD.
- USDA FoodData Central download page lists current downloadable releases, including December 2025 Foundation Foods and Branded Foods.
- openFDA Drug NDC states the NDC Directory endpoint is updated daily.
- openFDA Drug Shortages states the API covers data from 2012 onward and updates daily.
- AccessGUDID download page exposes daily, weekly, and monthly release archives, including a daily release dated 2026-03-09.
- openFDA Device UDI states the API covers 2013 onward and updates weekly.
- openFDA Food Enforcement and Device Enforcement both state public recall enforcement data is updated weekly.
- USDA FSIS documents a public Recall API endpoint at `https://www.fsis.usda.gov/fsis/api/recall/v/1`.
