# Ontology Admission Benchmark - 2026-03-10

## Scope

This benchmark records the first dedicated Criterion run for the production ontology-admission path built on `src/ontology/*` and SPACL-backed reasoning.

The benchmark harness is implemented in:

- `benches/ontology_admission_benchmarks.rs`

It covers three focused scenarios:

1. exact-class validation without a reasoner
2. subclass-aware validation with the ontology manager and SPACL reasoner
3. invalid validation with explanation-summary generation

## Command

```bash
cargo bench --bench ontology_admission_benchmarks -- --sample-size 10
```

## Results

Environment date: 2026-03-10

### `ontology_admission_validation/validator_exact_class_fallback`

- time: `202.35-202.54 us`

Interpretation:

- baseline exact-type class enforcement without a reasoner is now fast enough to act as a lightweight fallback path

### `ontology_admission_validation/manager_subclass_reasoning`

- time: `174.19-174.37 us`

Interpretation:

- subclass-aware validation through `OntologyManager::validate_transaction(...)` is currently not slower than the fallback-only validator path in this synthetic benchmark
- this supports the architectural choice to keep the production path on `src/ontology/*` + SPACL rather than treating reasoning as a separate optional slow path

### `ontology_admission_validation/validation_failure_explanation_summary`

- time: `213.08-213.33 us`

Interpretation:

- generating deterministic explanation metadata and compact failure summaries adds only a small overhead in this focused invalid-data scenario

## What this benchmark proves

- the production ontology admission path has a dedicated, runnable benchmark artifact
- exact-class fallback and subclass reasoning can now be compared directly
- explanation-summary generation is benchmarked as part of validation, not only tested functionally

## What this benchmark does not yet prove

- real-world domain-scale performance on healthcare, pharmaceutical, or UHT ontology packages
- end-to-end node-runtime admission cost under consensus and networking
- large-shape and large-ontology scaling behavior
- statistical comparison against external baselines

## Next benchmark extensions

1. add real domain-package runs for healthcare and pharmaceutical ontologies
2. add end-to-end block-admission benchmarks that include blockchain insertion without noisy error logging
3. compare exact-type fallback, subclass reasoning, and richer explanation modes on larger datasets
