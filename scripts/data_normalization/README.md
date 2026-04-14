# Data Normalization Scripts

These scripts convert raw public-dataset snapshots into the intermediate schema defined in:

- `docs/benchmarking/NORMALIZATION_SCHEMA_2026-03-10.md`

Current scripts:

- `normalize_fooddata_central.py`
- `normalize_openfda_drug_ndc.py`
- `normalize_openfda_drug_shortages.py`
- `normalize_accessgudid.py`

Example usage:

```bash
python3 scripts/data_normalization/normalize_fooddata_central.py \
  --input config/datasets/raw_examples/fooddata_central_sample.json \
  --output /tmp/fooddata_central.normalized.json \
  --snapshot-id fooddata_central_sample
```

```bash
python3 scripts/data_normalization/normalize_openfda_drug_ndc.py \
  --input config/datasets/raw_examples/openfda_drug_ndc_sample.json \
  --output /tmp/openfda_drug_ndc.normalized.json \
  --snapshot-id openfda_drug_ndc_sample
```

```bash
python3 scripts/data_normalization/normalize_openfda_drug_shortages.py \
  --input config/datasets/raw_examples/openfda_drug_shortages_sample.json \
  --output /tmp/openfda_drug_shortages.normalized.json \
  --snapshot-id openfda_drug_shortages_sample
```

```bash
python3 scripts/data_normalization/normalize_accessgudid.py \
  --input config/datasets/raw_examples/accessgudid_sample.csv \
  --output /tmp/accessgudid.normalized.json \
  --snapshot-id accessgudid_sample
```

These scripts intentionally emit normalized JSON only. RDF emission and ontology-package projection remain separate next-layer responsibilities.
