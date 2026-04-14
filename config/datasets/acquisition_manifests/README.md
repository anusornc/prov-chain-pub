# Dataset Snapshot Manifests

These manifests describe the benchmark sample snapshots currently used by the normalization and ontology-admission pipeline.

Current manifests:

- `fooddata_central_sample_2026_03_10.toml`
- `accessgudid_sample_2026_03_10.toml`
- `openfda_drug_shortages_sample_2026_03_10.toml`

Each manifest records:

- source provider and documentation URL
- fixture path used in the repository
- snapshot ID used during normalization
- ontology package targeted by the benchmark path
- record counts for the sample snapshot

The current `sha256` fields are placeholders and should be filled once the publication artifact bundle is frozen.
