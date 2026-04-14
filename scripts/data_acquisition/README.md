# Data Acquisition Scripts

These scripts implement the first reproducible snapshot workflow described in:

- `docs/benchmarking/DATASET_ACQUISITION_PLAN_2026-03-10.md`

Current scripts:

- `fetch_gs1_examples.py`
- `fetch_openfda_snapshot.py`
- `fetch_accessgudid_snapshot.py`

All scripts write:

- raw files under `data/external/raw/...`
- a TOML acquisition manifest next to the raw snapshot

Example usage:

```bash
python3 scripts/data_acquisition/fetch_openfda_snapshot.py \
  --source drug_ndc \
  --snapshot-id openfda_drug_ndc_2026_03_10 \
  --limit 25
```

```bash
python3 scripts/data_acquisition/fetch_accessgudid_snapshot.py \
  --url "https://accessgudid.nlm.nih.gov/download/2026-03-09/accessgudid.zip" \
  --snapshot-id accessgudid_2026_03_09 \
  --release-date 2026-03-09
```

```bash
python3 scripts/data_acquisition/fetch_gs1_examples.py \
  --snapshot-id gs1_examples_2026_03_10 \
  --url "https://ref.gs1.org/docs/epcis/examples/example.jsonld"
```

The scripts do not normalize data yet. They only capture reproducible raw snapshots plus acquisition metadata.
