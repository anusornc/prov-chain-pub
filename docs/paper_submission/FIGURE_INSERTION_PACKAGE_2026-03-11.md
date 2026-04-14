# Figure Insertion Package - 2026-03-11

This document defines the current paper-facing figure insertion package for the ProvChain manuscript.

Primary assets:

- `figures/architecture/shared_ontology_layered_architecture.png`
- `figures/architecture/shared_ontology_admission_pipeline.png`
- `figures/benchmarking/single_record_path_breakdown.png`
- `figures/benchmarking/batch_scaling_curves.png`
- `figures/benchmarking/cross_package_workloads.png`
- `snippets/architecture_figures.tex`
- `snippets/benchmark_figures.tex`

## Recommended usage in `main.tex`

### 1. Architecture section

Replace the current inline TikZ architecture and data-flow figures with:

- `\input{snippets/architecture_figures}`

Recommended location:

- inside `\section{System Architecture}` after the introductory overview paragraph and before the ontology-architecture subsection

Expected effect:

- `Figure~\ref{fig:architecture}` becomes the new layered shared-ontology architecture view
- `Figure~\ref{fig:dataflow}` becomes the new ontology-backed admission pipeline view

### 2. Evaluation section

Insert:

- `\input{snippets/benchmark_figures}`

Recommended location:

- after the evaluation subsection that introduces ontology-backed admission latency and batch-scaling evidence

Suggested narrative anchors:

- use `Figure~\ref{fig:single_record_breakdown}` when separating setup cost from admission cost
- use `Figure~\ref{fig:batch_scaling}` when discussing `x1/x2/x4` scaling behavior
- use `Figure~\ref{fig:cross_package_workloads}` when discussing heterogeneous evaluation workloads across package-specific benchmark paths

## Files copied into the submission package

Architecture figures:

- `figures/architecture/shared_ontology_layered_architecture.png`
- `figures/architecture/shared_ontology_layered_architecture.svg`
- `figures/architecture/shared_ontology_admission_pipeline.png`
- `figures/architecture/shared_ontology_admission_pipeline.svg`

Benchmark figures:

- `figures/benchmarking/single_record_path_breakdown.png`
- `figures/benchmarking/single_record_path_breakdown.svg`
- `figures/benchmarking/batch_scaling_curves.png`
- `figures/benchmarking/batch_scaling_curves.svg`
- `figures/benchmarking/cross_package_workloads.png`
- `figures/benchmarking/cross_package_workloads.svg`

## Notes

- Prefer `PNG` inside `main.tex` because the current compilation path is `pdfLaTeX`.
- Keep `SVG` copies for future editing or vector export.
- The architecture figures follow the current shared-ontology model and exclude legacy `src/semantic/*` as the production path.
- The benchmark figures are derived from Criterion exports and already aligned with the current summary tables and CSV artifacts.
