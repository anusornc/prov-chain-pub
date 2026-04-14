# Domain Dataset Admission Benchmark - 2026-03-10

## Scope

This benchmark records the first ontology-backed block-admission run using synthetic domain events derived from the dataset acquisition and normalization pipeline.

It is built on:

- normalized public-source records,
- ontology-package-specific emission from normalized records,
- benchmark-time single-record extraction via `emit_ontology_package_turtle.py --limit 1`,
- `Blockchain::new_with_ontology(...)`,
- and `Blockchain::add_block(...)`.

Benchmark harness:

- `benches/domain_dataset_admission_benchmarks.rs`

## Command

```bash
cargo bench --bench domain_dataset_admission_benchmarks -- --sample-size 10
```

## Validation Prerequisite

Before the benchmark run, the following integration test passed:

```bash
cargo test --test dataset_projection_pipeline_tests -- --nocapture
```

This confirms that the synthetic UHT-product, hybrid GS1/EPCIS-UHT, healthcare-device, and pharma-storage events used by the benchmark are accepted by the current ontology-backed admission path.

## Results

Environment date: 2026-03-10

### End-to-end setup plus admission

These runs include blockchain creation plus ontology-backed `add_block(...)`.

#### `domain_dataset_admission/healthcare_device_event_add_block`

- time: `716.81-741.81 us`

Interpretation:

- a synthetic healthcare device event derived from AccessGUDID-style normalized data can pass the current healthcare ontology admission path in under 1 ms in this focused benchmark.

#### `domain_dataset_admission/uht_product_event_add_block`

- time: `912.38-928.30 us`

Interpretation:

- a synthetic UHT product event derived from FoodData Central-style normalized data can pass the current UHT ontology admission path in under 1 ms in this focused benchmark.

#### `domain_dataset_admission/uht_epcis_event_add_block`

- time: `1.078-1.097 ms`

Interpretation:

- a standards-aligned hybrid GS1/EPCIS plus UHT event derived from normalized food records can pass the same ontology-backed admission path in approximately 1 ms, showing the current benchmark can exercise a standards-facing semantic payload rather than only a domain-only synthetic event.

#### `domain_dataset_admission/pharma_storage_event_add_block`

- time: `849.20-874.81 us`

Interpretation:

- a synthetic pharmaceutical storage event derived from shortage-oriented normalized data can pass the current pharmaceutical ontology admission path in under 1 ms in this focused benchmark.

## What this benchmark proves

- the data pipeline now extends beyond planning:
  - acquisition strategy,
  - food, pharma, and device sample normalization,
  - intermediate normalization,
  - ontology-package-specific emitters,
  - benchmark harness loading package-emitter CLI outputs,
  - ontology-backed block admission,
  - and runnable Criterion benchmarks
- current UHT, hybrid GS1/EPCIS-UHT, healthcare, and pharmaceutical SHACL paths can be exercised from dataset-derived synthetic inputs
- the benchmark now separates setup cost from pure block-admission cost, which makes the academic interpretation cleaner

## What this benchmark does not yet prove

- direct one-to-one recovery of real operational events from public datasets
- end-to-end node-runtime performance under networking and consensus
- full standards-complete GS1 EPCIS interoperability coverage
- full ontology-package-specific RDF emission beyond the current benchmark-focused packages

## Important Caveat

The earlier benchmark run was noisy because `Blockchain::initialize_ontology_system(...)` printed domain initialization lines during each iteration.

That path has now been switched to structured logging, which makes the benchmark output much cleaner.

This improves benchmark usability, but it does not change the main interpretation limits:

- these are still focused synthetic-event benchmarks,
- and only part of the benchmark set excludes blockchain construction cost.

## Split setup and admission cost

These runs separate blockchain construction from ontology-backed admission.

### Single-record setup only

- `uht_product_event_setup_only`: time `7.136-7.149 ms`
- `healthcare_device_event_setup_only`: time `5.064-5.071 ms`
- `pharma_storage_event_setup_only`: time `8.066-8.149 ms`

### Single-record admission only

- `uht_product_event_add_block_only`: time `717.84-718.60 us`
- `uht_epcis_event_add_block_only`: time `793.53-796.62 us`
- `healthcare_device_event_add_block_only`: time `511.64-512.30 us`
- `pharma_storage_event_add_block_only`: time `642.58-643.54 us`

These values come from the CLI-driven benchmark harness after constraining emitter output to a single normalized record per run. This keeps the metric semantics aligned with the benchmark names.

### Batch admission only

These runs use the full emitted payload from the current sample fixture for each ontology package.

- `uht_product_batch_add_block_only` with 3 emitted records: time `1.2465-1.2477 ms`
- `uht_epcis_batch_add_block_only` with 3 emitted records: time `1.4443-1.4480 ms`
- `healthcare_device_batch_add_block_only` with 2 emitted records: time `660.97-661.91 us`
- `pharma_storage_batch_add_block_only` with 2 emitted records: time `824.09-825.14 us`

### Batch-size scaling curves

These runs expand the emitted batch payloads deterministically to `x2` and `x4` while preserving unique `ex:` identifiers.

- `uht_product_batch_x2_add_block_only`: time `2.0298-2.0305 ms`
- `uht_product_batch_x4_add_block_only`: time `3.5944-3.5965 ms`
- `uht_epcis_batch_x2_add_block_only`: time `2.3997-2.4010 ms`
- `uht_epcis_batch_x4_add_block_only`: time `4.3043-4.3066 ms`
- `healthcare_device_batch_x2_add_block_only`: time `959.74-960.13 us`
- `healthcare_device_batch_x4_add_block_only`: time `1.5444-1.5461 ms`
- `pharma_storage_batch_x2_add_block_only`: time `1.1884-1.1895 ms`
- `pharma_storage_batch_x4_add_block_only`: time `1.8958-1.8990 ms`

This gives the first scaling curve evidence for ontology-backed block admission on the current benchmark packages, rather than only single-point measurements.

### Cross-package round-robin workloads

These runs measure interleaved admissions across separate package-specific blockchains in one benchmark harness. They do **not** claim a single network is running multiple ontology packages at once.

- `cross_package_round_robin_single_record_add_block_only`: time `2.6314-2.6581 ms`
- `cross_package_round_robin_batch_add_block_only`: time `4.2037-4.2854 ms`

This provides the first heterogeneous workload evidence for the current evaluation stack while staying within the project's `one network = one shared ontology package` model.

## Publication Artifacts Derived from This Benchmark

The current publication-facing derivatives of this benchmark are:

- `docs/benchmarking/DOMAIN_DATASET_ADMISSION_SUMMARY_TABLES_2026-03-11.md`
- `docs/benchmarking/DOMAIN_DATASET_ADMISSION_FIGURES_2026-03-11.md`
- `docs/benchmarking/data/domain_dataset_admission_summary_2026-03-11.csv`
- `docs/benchmarking/data/domain_dataset_admission_plot_data_2026-03-11.csv`

The raw sample metadata for these benchmark fixtures is tracked in:

- `config/datasets/acquisition_manifests/fooddata_central_sample_2026_03_10.toml`
- `config/datasets/acquisition_manifests/accessgudid_sample_2026_03_10.toml`
- `config/datasets/acquisition_manifests/openfda_drug_shortages_sample_2026_03_10.toml`

## Next Benchmark Extensions

1. extend scaling curves beyond `x4` and compare scaling efficiency across packages
2. deepen GS1/EPCIS coverage beyond the current hybrid UHT object-event profile
3. expand ontology-package-specific RDF emission beyond the current benchmark-focused packages
