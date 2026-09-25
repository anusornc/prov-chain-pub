# ProvChainOrg Experimental Benchmark Results

> **Dated mixed benchmark record — not the current paper evidence boundary (2026-08-31).** This
> file combines historical component benchmarks with a custom load-test summary. The custom
> load-test values are withdrawn because the harness did not retain an admissible raw-sample
> artifact and its running-average and percentile calculations are defective. Do not quote a
> ledger-throughput, response-time, percentile, or success-rate result from that section. Use
> [`BENCHMARK_EVIDENCE_BOUNDARY_2026-05-11.md`](BENCHMARK_EVIDENCE_BOUNDARY_2026-05-11.md) and the
> current [`PAPER_EVIDENCE_INDEX.md`](../paper_submission/PAPER_EVIDENCE_INDEX.md) to select
> admissible focused ontology-admission and curated trace-query artifacts.

**Date:** 2026-01-17
**Platform:** Linux 6.8.0-1044-gcp
**Compiler:** Rust 1.70+ (release profile with optimizations)
**Measurement Tool:** Criterion.rs 0.5.1

---

## Executive Summary

This document preserves dated outputs from multiple experimental paths. Criterion component rows
must be interpreted within their original workload, and the withdrawn custom load-test section is
not Criterion or publication evidence. No blanket validity claim applies to the whole file.

---

## 1. OWL2 Reasoner Performance

### 1.1 Consistency Checking

| Ontology Size | Simple Consistency | Tableaux Consistency | Hybrid Consistency |
|---------------|-------------------|---------------------|-------------------|
| 10 axioms | **15.65 µs** ± 0.004 | **19.72 µs** ± 0.020 | **20.01 µs** ± 0.114 |
| 50 axioms | **27.23 µs** ± 0.012 | **31.17 µs** ± 0.003 | **31.00 µs** ± 0.003 |
| 100 axioms | **41.77 µs** ± 0.018 | **45.58 µs** ± 0.020 | **45.79 µs** ± 0.004 |
| 500 axioms | **168.65 µs** ± 0.015 | **166.52 µs** ± 0.095 | **167.01 µs** ± 0.015 |

**Key Findings:**
- Consistency checking scales linearly with ontology size
- Tableaux algorithm shows 26% overhead for small ontologies but negligible difference for large ones
- All measurements use 95% confidence intervals

### 1.2 Class Satisfiability

| Ontology Size | Simple Satisfiability | Tableaux Satisfiability |
|---------------|----------------------|-------------------------|
| 10 axioms | **14.52 µs** ± 0.005 | **19.93 µs** ± 0.043 |
| 50 axioms | **18.87 µs** ± 0.003 | **31.90 µs** ± 0.064 |
| 100 axioms | **24.99 µs** ± 0.075 | **45.86 µs** ± 0.023 |
| 500 axioms | **74.52 µs** ± 0.030 | **168.00 µs** ± 0.025 |

**Key Findings:**
- Satisfiability checking is ~10% faster than consistency checking
- Tableaux overhead increases with ontology complexity

### 1.3 Subclass Checking

| Ontology Size | Simple Subclass | Tableaux Subclass |
|---------------|-----------------|-------------------|
| 10 axioms | **15.03 µs** ± 0.042 | **20.28 µs** ± 0.058 |
| 50 axioms | **20.63 µs** ± 0.003 | **33.19 µs** ± 0.011 |
| 100 axioms | **28.03 µs** ± 0.035 | **49.08 µs** ± 0.015 |
| 500 axioms | **88.37 µs** ± 0.028 | **184.82 µs** ± 0.035 |

### 1.4 Memory Usage

| Ontology Size | Simple Memory | Tableaux Memory |
|---------------|--------------|-----------------|
| 10 axioms | **14.47 µs** ± 0.003 | **20.53 µs** ± 0.025 |
| 50 axioms | **19.17 µs** ± 0.007 | **32.24 µs** ± 0.166 |
| 100 axioms | **24.85 µs** ± 0.004 | **46.97 µs** ± 0.020 |
| 500 axioms | **73.10 µs** ± 0.028 | **~169 µs** (measurement in progress) |

---

## 2. Query Performance

### 2.1 SPARQL Query Performance (Optimized vs Baseline)

| Dataset Size | Query Pattern | Optimized | Baseline | Speedup |
|--------------|--------------|-----------|----------|---------|
| 100 triples | Simple SELECT (query0) | **39.74 µs** | **35.33 µs** | 0.89× |
| 100 triples | Type query (query1) | **37.04 µs** | **31.51 µs** | 0.85× |
| 100 triples | Property query (query2) | **52.12 µs** | **51.19 µs** | 1.02× |
| 100 triples | Join query (query3) | **56.29 µs** | **56.62 µs** | 1.01× |
| 100 triples | Complex join (query4) | **140.16 µs** | **96.94 µs** | 0.69× |
| 500 triples | Simple SELECT | **170.84 µs** | **141.08 µs** | 0.79× |
| 500 triples | Type query | **153.19 µs** | **124.05 µs** | 0.77× |
| 500 triples | Property query | **472.71 µs** | **458.80 µs** | 1.03× |
| 500 triples | Join query | **490.95 µs** | **492.08 µs** | 1.00× |
| 500 triples | Complex join | **759.79 µs** | **532.50 µs** | 0.70× |
| 1,000 triples | Simple SELECT | **358.50 µs** | **295.85 µs** | 0.79× |
| 1,000 triples | Type query | **310.48 µs** | **247.85 µs** | 0.75× |
| 1,000 triples | Property query | **1.57 ms** | **1.54 ms** | 1.02× |
| 1,000 triples | Join query | **1.67 ms** | **1.67 ms** | 1.00× |
| 1,000 triples | Complex join | **1.79 ms** | **1.34 ms** | 0.75× |
| 5,000 triples | Simple SELECT | **2.46 ms** | **2.14 ms** | 0.87× |
| 5,000 triples | Type query | **2.20 ms** | **2.00 ms** | 0.91× |
| 5,000 triples | Property query | **9.63 ms** | **6.92 ms** | 0.72× |
| 5,000 triples | Join query | **10.87 ms** | **8.86 ms** | 0.81× |
| 5,000 triples | Complex join | **18.04 ms** | **15.30 ms** | 0.85× |

**Key Findings:**
- Query performance scales near-linearly with dataset size
- Simple queries: ~0.5 µs per triple
- Complex joins: ~3-4 µs per triple
- Baseline engine outperforms optimized engine on simpler queries
- Optimized engine shows benefit only on most complex patterns

### 2.2 Cache Performance

| Operation | Performance | Notes |
|-----------|-------------|-------|
| First query (cold cache) | **422.45 µs** | Cache miss overhead |
| Repeated query (warm cache) | **498.10 µs** | Cache hit (unexpectedly slower) |
| Repeated query (no cache) | **336.06 µs** | Direct execution fastest |

**Finding:** Current cache implementation shows overhead; direct query execution is faster.

### 2.3 Index Performance

| Dataset Size | Indexed Query | Non-Indexed Query | Index Benefit |
|--------------|---------------|-------------------|---------------|
| 100 triples | **39.19 µs** | **26.02 µs** | 0.67× (slower) |
| 1,000 triples | **363.59 µs** | **187.29 µs** | 0.52× (slower) |
| 5,000 triples | **2.40 ms** | **1.51 ms** | 0.63× (slower) |

**Finding:** Index overhead exceeds benefit for datasets under 5,000 triples.

### 2.4 Pattern Compilation

| Operation | Performance |
|-----------|-------------|
| With cache | **1.88 ms** |
| Without cache | **17.72 ms** |
| **Speedup** | **9.4× with cache** |

---

## 3. Memory Management

### 3.1 Memory Statistics Collection

| Operation | Latency | Throughput |
|-----------|---------|------------|
| get_memory_stats | **122.97 ns** | 8.13M ops/sec |
| get_memory_pressure_level | **121.64 ns** | 8.22M ops/sec |
| is_under_memory_pressure | **120.71 ns** | 8.28M ops/sec |
| detect_memory_leaks | **131.39 ns** | 7.61M ops/sec |

### 3.2 Memory Allocation (100 nodes)

| Configuration | Allocation Time | Per-Node Overhead |
|---------------|-----------------|-------------------|
| Without tracking | **167.27 µs** | **1.67 µs/node** |
| With tracking | **144.13 µs** | **1.44 µs/node** |
| **Savings** | **-13.8%** | Tracking is faster! |

### 3.3 Checkpoint and Rollback

| Operation | Scale | Latency |
|-----------|-------|---------|
| create_checkpoint | Base | **182.16 ns** |
| create_checkpoint | With allocations | **2.93 µs** |
| rollback_to_checkpoint | 10 operations | **5.55 µs** |
| rollback_to_checkpoint | 100 operations | **44.84 µs** |
| rollback_to_checkpoint | 1,000 operations | **518.89 µs** |

### 3.4 String Interning

| Configuration | Latency | Overhead |
|---------------|---------|----------|
| Without tracking | **403.85 ns** | baseline |
| With tracking | **801.44 ns** | **1.98× overhead** |

---

## 4. Parser Performance

### 4.1 Turtle Parsing

| Dataset Size | Parse Time | Throughput |
|--------------|------------|------------|
| Small (~10 triples) | **28.75 µs** | ~348K triples/sec |
| Medium (~100 triples) | **1.09 ms** | ~91K triples/sec |
| Large (~1,000 triples) | **46.30 ms** | ~21.6K triples/sec |

---

## 5. Concurrent Reasoning

### 5.1 Concurrent Consistency Checking

| Threads | Latency | Throughput | Speedup vs Single |
|---------|---------|------------|------------------|
| 1 | **69.17 µs** | baseline | 1.00× |
| 2 | **78.21 µs** | | 0.88× (contention) |
| 4 | **128.97 µs** | | 0.54× |
| 8 | **236.18 µs** | | 0.29× |

### 5.2 Concurrent Satisfiability

| Threads | Latency | Speedup vs Single |
|---------|---------|------------------|
| 1 | **60.37 µs** | 1.00× |
| 2 | **82.42 µs** | 0.73× |
| 4 | **113.29 µs** | 0.53× |
| 8 | **181.05 µs** | 0.33× |

**Finding:** Concurrent reasoning shows contention overhead; single-threaded is more efficient.

---

## 6. Algorithmic Complexity Analysis

### 6.1 Ontology Creation

| Axioms | Time | Per-Axiom Cost |
|--------|------|----------------|
| 10 | **16.93 µs** | **1.69 µs/axiom** |
| 50 | **100.89 µs** | **2.02 µs/axiom** |
| 100 | **212.96 µs** | **2.13 µs/axiom** |

**Complexity:** O(n) linear scaling

### 6.2 Class Operations (Add Classes)

| Classes | Time | Per-Class Cost |
|---------|------|----------------|
| 10 | **11.34 µs** | **1.13 µs/class** |
| 50 | **72.16 µs** | **1.44 µs/class** |
| 100 | **154.83 µs** | **1.55 µs/class** |

### 6.3 Large-Scale Reasoning

| Ontology Size | Simple Consistency | Tableaux Consistency | Speedup |
|---------------|-------------------|---------------------|---------|
| 1,000 axioms | **1.30 ms** | **1.69 ms** | 1.30× |
| 5,000 axioms | **10.08 ms** | **15.56 ms** | 1.54× |
| 10,000 axioms | **26.16 ms** | **32.39 ms** | 1.24× |

**Key Finding:** Tableaux algorithm overhead increases with scale (30-54% slower).

### 6.4 IRI Creation Scaling

| IRIs | Time | Per-IRI Cost |
|------|------|--------------|
| 500 | **832 µs** | **1.66 µs/IRI** |
| 1,000 | **3.43 ms** | **3.43 µs/IRI** |
| 2,500 | **15.36 ms** | **6.14 µs/IRI** |
| 5,000 | **71.67 ms** | **14.33 µs/IRI** |

**Complexity:** Super-linear scaling (O(n log n)) for IRI creation.

### 6.5 Ontology Creation Scaling

| Axioms | Time | Per-Axiom Cost |
|--------|------|----------------|
| 500 | **5.30 ms** | **10.6 µs/axiom** |
| 1,000 | **15.90 ms** | **15.9 µs/axiom** |
| 2,500 | **69.14 ms** | **27.7 µs/axiom** |
| 5,000 | **346.92 ms** | **69.4 µs/axiom** |

**Complexity:** Super-linear scaling (O(n²) behavior observed).

### 6.6 Consistency Checking Scaling (Per-Axiom)

| Axioms | Per-Axiom Time | Total Time |
|--------|---------------|------------|
| 250 | **110.9 ns** | **27.7 µs** |
| 500 | **110.7 ns** | **55.4 µs** |
| 1,000 | **110.7 ns** | **110.7 µs** |
| 2,000 | **110.8 ns** | **221.6 µs** |

**Key Finding:** Consistency checking is **O(n)** linear - excellent scaling!

### 6.7 Memory Usage Scaling

| Axioms | Time | Per-Axiom Memory |
|--------|------|------------------|
| 1,000 | **30.29 ms** | **30.3 µs/axiom** |
| 2,500 | **141.89 ms** | **56.8 µs/axiom** |
| 5,000 | **~440 ms** (est.) | **~88 µs/axiom** |

---

## Historical Methodology Annotation

The January draft recorded the following parameters for its Criterion component rows. They are not
a complete immutable environment manifest and do not apply to the withdrawn custom load test:
- **Samples:** 100 measurements per benchmark
- **Warm-up:** 3 seconds (or 200ms for micro-benchmarks)
- **Analysis:** Criterion.rs with 95% confidence intervals
- **Outliers:** Detected and reported using Quartile method
- **Platform:** Linux on Google Cloud Platform

**Evidence note:** historical output is retained for auditability, but retention does not make every
value statistically correct or admissible for a current claim.

---

## Comparison with Research Objectives

### Historical Target Comparison (from ADR 0001)

| Metric | Target | Historical component result | Status |
|--------|--------|-----------------------------|--------|
| Write Throughput | > 8,000 TPS | **Withdrawn** — no corrected ledger-throughput artifact | Not admitted |
| Read Latency (P95) | < 100ms | **0.04-18ms** (SPARQL component draft) | Historical component row; not a current service target |
| OWL2 Reasoning | < 200ms | **0.015-0.17ms** (legacy consistency path) | Historical component row; not production semantic-path evidence |
| Memory Usage | < 16 GB | **~200MB** (legacy reasoner draft) | Historical component row; not a node resource envelope |

**Withdrawn Custom Load-Test Summary (2026-01-18):**

The former summary table and “all tests pass” interpretation have been removed from the current
evidence path. In addition to the defective running-average and percentile calculations, the test
was ignored/profile-gated, exercised an in-process chain behind a process mutex, and did not emit
the raw request samples and environment/dependency metadata required for curated evidence. It
cannot establish ledger throughput, concurrency, bottlenecks, production scalability, or
multi-node behavior. A future corrected campaign must measure the exact admitted and durable write
path and archive raw samples before publishing any replacement value.

---

## Reproducibility Boundary

This mixed historical record cannot be reproduced byte-for-byte from one archived command and
environment. Running `cargo bench` today would produce a new local run, not validate every value in
this file, and a local `target/criterion/` directory is not curated evidence. Use the evidence index
for admitted artifact paths and archive commands, versions, raw samples, environment metadata, and
hashes for any replacement campaign.

---

## Next Steps for Thesis

1. **Correct the ledger-throughput harness**
   - retain raw request samples and correct running-average/percentile calculations
   - identify the exact admission, journal-commit, and acknowledgement boundary
   - record environment/dependency metadata and publish a curated artifact

2. **Implement the locked reference-system milestones in dependency order**
   - complete-envelope journal and verified replay
   - universal Final Admission
   - authenticated three-node PoA convergence
   - complete package-declared SHACL, durable privacy, and bounded six-process bridge

3. **Archive corrected end-to-end evidence**
   - retain raw samples and failure/failpoint outputs
   - record dependency, environment, topology, and exact command metadata
   - publish hashes plus a claim-to-artifact map before statistical conclusions

---

## References

- Criterion.rs documentation: https://bheisler.github.io/criterion.rs/book/
- Statistical methodology: Bootstrap confidence intervals
- Hardware specifications: See system context in `/proc/cpuinfo`
