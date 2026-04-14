# Domain Dataset Admission Summary Tables - 2026-03-11

This file converts the latest Criterion results into paper-ready summary tables.

Source benchmark:

- `target/criterion/domain_dataset_admission/*/new/estimates.json`

Exported artifacts:

- `docs/benchmarking/data/domain_dataset_admission_summary_2026-03-11.csv`
- `docs/benchmarking/data/domain_dataset_admission_plot_data_2026-03-11.csv`

## End-to-End Admission

| Benchmark | Package | Emitted Records | Mean | 95% CI |
|---|---|---:|---:|---:|
| healthcare_device_event_add_block | healthcare_device | 1 | 715.27 us | 704.54 us - 726.98 us |
| pharma_storage_event_add_block | pharma_storage | 1 | 860.80 us | 856.05 us - 866.31 us |
| uht_epcis_event_add_block | uht_epcis | 1 | 1.008 ms | 1.001 ms - 1.016 ms |
| uht_product_event_add_block | uht | 1 | 903.95 us | 896.81 us - 911.89 us |

## Setup Only

| Benchmark | Package | Emitted Records | Mean | 95% CI |
|---|---|---:|---:|---:|
| healthcare_device_event_setup_only | healthcare_device | 1 | 5.061 ms | 5.059 ms - 5.063 ms |
| pharma_storage_event_setup_only | pharma_storage | 1 | 7.783 ms | 7.782 ms - 7.785 ms |
| uht_product_event_setup_only | uht | 1 | 7.130 ms | 7.126 ms - 7.136 ms |

## Single-Record Admission Only

| Benchmark | Package | Emitted Records | Mean | 95% CI |
|---|---|---:|---:|---:|
| healthcare_device_event_add_block_only | healthcare_device | 1 | 509.93 us | 509.58 us - 510.27 us |
| pharma_storage_event_add_block_only | pharma_storage | 1 | 639.72 us | 639.12 us - 640.41 us |
| uht_epcis_event_add_block_only | uht_epcis | 1 | 786.35 us | 785.86 us - 786.91 us |
| uht_product_event_add_block_only | uht | 1 | 713.11 us | 711.69 us - 714.34 us |

## Batch Admission Only

| Benchmark | Package | Emitted Records | Mean | 95% CI |
|---|---|---:|---:|---:|
| healthcare_device_batch_add_block_only | healthcare_device | 2 | 661.18 us | 660.71 us - 661.67 us |
| pharma_storage_batch_add_block_only | pharma_storage | 2 | 824.80 us | 824.12 us - 825.59 us |
| uht_epcis_batch_add_block_only | uht_epcis | 3 | 1.447 ms | 1.444 ms - 1.453 ms |
| uht_product_batch_add_block_only | uht | 3 | 1.247 ms | 1.246 ms - 1.247 ms |

## Batch Scaling Curves

| Benchmark | Package | Emitted Records | Mean | 95% CI |
|---|---|---:|---:|---:|
| healthcare_device_batch_x2_add_block_only | healthcare_device | 4 | 960.09 us | 959.82 us - 960.40 us |
| healthcare_device_batch_x4_add_block_only | healthcare_device | 8 | 1.545 ms | 1.545 ms - 1.546 ms |
| pharma_storage_batch_x2_add_block_only | pharma_storage | 4 | 1.189 ms | 1.188 ms - 1.189 ms |
| pharma_storage_batch_x4_add_block_only | pharma_storage | 8 | 1.897 ms | 1.896 ms - 1.898 ms |
| uht_epcis_batch_x2_add_block_only | uht_epcis | 6 | 2.401 ms | 2.400 ms - 2.401 ms |
| uht_epcis_batch_x4_add_block_only | uht_epcis | 12 | 4.305 ms | 4.303 ms - 4.307 ms |
| uht_product_batch_x2_add_block_only | uht | 6 | 2.030 ms | 2.029 ms - 2.030 ms |
| uht_product_batch_x4_add_block_only | uht | 12 | 3.598 ms | 3.595 ms - 3.604 ms |

## Cross-Package Round-Robin Workloads

| Benchmark | Package | Emitted Records | Mean | 95% CI |
|---|---|---:|---:|---:|
| cross_package_round_robin_batch_add_block_only | cross_package | 10 admissions | 4.221 ms | 4.205 ms - 4.246 ms |
| cross_package_round_robin_single_record_add_block_only | cross_package | 4 admissions | 2.640 ms | 2.631 ms - 2.649 ms |

## Figure Notes

- Use `domain_dataset_admission_plot_data_2026-03-11.csv` for line plots or grouped bar charts.
- See `DOMAIN_DATASET_ADMISSION_FIGURES_2026-03-11.md` for the current generated `PNG` and `SVG` figure package.
- Recommended figures:
  - single-record admission by package
  - batch admission vs `x2/x4` scaling by package
  - cross-package round-robin single-record vs batch workload
