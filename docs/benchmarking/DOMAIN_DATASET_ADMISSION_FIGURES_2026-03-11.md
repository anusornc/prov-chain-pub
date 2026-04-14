# Domain Dataset Admission Figures - 2026-03-11

This document defines the publication-ready figure package derived from the current domain dataset admission benchmark exports.

Source artifacts:

- `docs/benchmarking/data/domain_dataset_admission_summary_2026-03-11.csv`
- `docs/benchmarking/data/domain_dataset_admission_plot_data_2026-03-11.csv`
- `scripts/generate_domain_dataset_admission_figures.py`

Generated figure directory:

- `docs/benchmarking/figures/domain_dataset_admission_2026-03-11/`

## Figure Inventory

### Figure A. Single-record path breakdown

Files:

- `figures/domain_dataset_admission_2026-03-11/single_record_path_breakdown.png`
- `figures/domain_dataset_admission_2026-03-11/single_record_path_breakdown.svg`

Purpose:

- compare `add_block only` against `setup + add_block` for single-record ontology-backed admission
- show setup-only cost separately so the sub-millisecond admission results are still readable

Recommended caption:

> Single-record ontology-backed admission latency across the current benchmark packages. The left panel compares pure `add_block` admission against end-to-end setup plus admission. The right panel isolates blockchain setup cost. Error bars show 95% confidence intervals from Criterion estimates.

### Figure B. Batch scaling curves

Files:

- `figures/domain_dataset_admission_2026-03-11/batch_scaling_curves.png`
- `figures/domain_dataset_admission_2026-03-11/batch_scaling_curves.svg`

Purpose:

- show `x1`, `x2`, and `x4` scaling trends for ontology-backed `add_block` admission
- compare scaling behavior across healthcare-device, pharma-storage, UHT, and hybrid GS1/EPCIS-UHT packages

Recommended caption:

> Batch-size scaling curves for ontology-backed `add_block` admission. Each line represents one ontology package, and the x-axis records the number of emitted records in the admission payload. Error bars show 95% confidence intervals from Criterion estimates.

### Figure C. Cross-package round-robin workloads

Files:

- `figures/domain_dataset_admission_2026-03-11/cross_package_workloads.png`
- `figures/domain_dataset_admission_2026-03-11/cross_package_workloads.svg`

Purpose:

- summarize heterogeneous evaluation behavior without claiming a single network runs multiple ontology packages at once
- show both total latency and normalized per-admission latency

Recommended caption:

> Cross-package round-robin benchmark results across separate package-specific blockchains. The left panel shows total latency for the workload, while the right panel normalizes the result by the number of admissions in the workload. Error bars show 95% confidence intervals from Criterion estimates.

## Generation Command

```bash
python3 scripts/generate_domain_dataset_admission_figures.py
```

## Interpretation Notes

- These figures are derived from the same Criterion estimates already summarized in `DOMAIN_DATASET_ADMISSION_SUMMARY_TABLES_2026-03-11.md`.
- `UHT + GS1/EPCIS` reflects the current hybrid standards-facing benchmark profile, not a full GS1 EPCIS conformance claim.
- Cross-package figures reflect interleaved benchmark workloads across separate package-specific blockchains and should not be described as a single shared network with multiple ontology packages.
