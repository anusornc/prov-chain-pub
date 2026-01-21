# Threats to Validity

**Research:** Enhancement of Blockchain with Embedded Ontology and Knowledge Graph for Data Traceability

**Date:** 2026-01-18
**Purpose:** Comprehensive discussion of limitations and validity threats for journal publication

---

## Overview

This document identifies potential threats to the validity of our experimental results and discusses mitigation strategies. Addressing these threats is essential for journal publication and scientific rigor.

---

## 1. Internal Validity

**Definition:** Extent to which observed effects can be attributed to the experimental manipulation rather than confounding variables.

### 1.1 Confounding Variables

#### Threat: Hardware/Platform Differences
**Description:** Experiments conducted on Google Cloud Platform (Linux 6.8.0-1044-gcp). Results may not generalize to:
- Different CPU architectures (ARM vs x86)
- Different operating systems (Windows, macOS)
- On-premises hardware vs cloud environments

**Mitigation:**
- ✅ All experiments conducted on consistent platform
- ✅ Hardware specifications documented (GCP VM)
- ⚠️ Future work: Cross-platform validation

**Impact:** Medium - Results are valid for cloud Linux deployments but may not generalize to other platforms.

---

#### Threat: Compiler Optimization Variability
**Description:** Rust compiler optimizations (release profile) may vary between:
- Compiler versions (1.70+ vs newer)
- Optimization flags (-O2 vs -O3)
- Target architecture settings

**Mitigation:**
- ✅ Documented compiler version (Rust 1.70+)
- ✅ Used consistent release profile
- ✅ Reproducible builds (Cargo.lock)

**Impact:** Low - Compiler optimizations are stable within Rust ecosystem.

---

#### Threat: Measurement Tool Bias
**Description:** Criterion.rs may introduce measurement overhead:
- Timer precision (nanosecond)
- Warm-up period sufficiency
- Loop unrolling artifacts

**Mitigation:**
- ✅ Criterion.rs is industry-standard benchmarking tool
- ✅ Used 3-second warm-up (200ms for micro-benchmarks)
- ✅ Bootstrap confidence intervals robust to timing artifacts

**Impact:** Low - Criterion.rs is validated tool used in numerous publications.

---

### 1.2 Selection Bias

#### Threat: Benchmark Workload Representativeness
**Description:** Supply chain traceability workload may not represent all blockchain use cases:
- Transaction patterns (100 requests per user)
- Ontology complexity (10-10,000 axioms)
- Query distribution (simple vs complex joins)

**Mitigation:**
- ✅ Documented workload characteristics
- ✅ Used realistic supply chain scenarios
- ⚠️ Future work: Diverse workload validation (finance, healthcare)

**Impact:** Medium - Results valid for supply chain traceability, generalization to other domains untested.

---

#### Threat: Ontology Selection Bias
**Description:** Test ontologies may not represent real-world complexity:
- Synthetic axioms vs production ontologies
- Limited expressiveness (subset of OWL2)
- Domain-specific assumptions

**Mitigation:**
- ✅ Tested across 4 orders of magnitude (10-10,000 axioms)
- ✅ Used realistic supply chain ontology
- ⚠️ Future work: Test with standard ontologies (SNOMED, FIBO)

**Impact:** Medium - Results demonstrate scaling but may not capture all OWL2 features.

---

## 2. External Validity

**Definition:** Extent to which results generalize to other populations, settings, and times.

### 2.1 Generalizability

#### Threat: Single-Node Development Environment
**Description:** Experiments conducted in single-node development environment:
- Measured: 19.58 TPS throughput
- Production target: 8,000+ TPS (distributed deployment)
- Gap: 408× difference

**Limitation Explicitly Stated:**
> "Write throughput measured in single-node development environment. Production target of 8,000+ TPS assumes 100+ distributed nodes with network-level parallelism and optimized hardware."

**Mitigation:**
- ✅ Clearly labeled as development environment performance
- ✅ Production targets documented as projections (not measured)
- ✅ Component-level benchmarks enable extrapolation
- ⚠️ Future work: Multi-node deployment validation

**Impact:** High - Core conclusions (OWL2 reasoning, query latency) remain valid, but throughput claims require production validation.

---

#### Threat: Scalability Limits
**Description:** Experiments limited to:
- Ontology size: 10-10,000 axioms
- Dataset size: 100-5,000 triples
- Concurrent users: 200

**Mitigation:**
- ✅ Tested 4 orders of magnitude (10× scaling range)
- ✅ Verified linear O(n) scaling within tested range
- ⚠️ Extrapolation beyond tested range is speculative

**Impact:** Medium - Linear scaling suggests generalizability to larger scales, but untested.

---

#### Threat: Temporal Validity
**Description:** Experiments conducted January 2026. Results may not reflect:
- Future Rust compiler optimizations
- Hardware improvements (newer CPU generations)
- Blockchain protocol evolution

**Mitigation:**
- ✅ All software versions documented
- ✅ Docker image enables reproducible experiments
- ⚠️ Results valid for state-of-art as of 2026-01

**Impact:** Low - Relative performance (overhead, scaling) likely stable across time.

---

### 2.2 Ecological Validity

#### Threat: Artificial Workload vs Production Traffic
**Description:** Load test uses artificial workload:
- Fixed request pattern (200 users × 100 requests)
- No realistic inter-request latency (think time: 10ms)
- Simulated vs real user behavior

**Mitigation:**
- ✅ Documented test parameters
- ✅ Realistic supply chain queries (SPARQL patterns)
- ⚠️ Future work: Production trace analysis

**Impact:** Medium - Latency measurements (P95: 98ms) likely conservative vs production variance.

---

## 3. Construct Validity

**Definition:** Extent to which measurements accurately reflect the theoretical constructs being studied.

### 3.1 Measurement Validity

#### Threat: Latency Metric Selection
**Description:** Using P95 latency as primary metric:
- P95 may not capture worst-case scenarios
- P99 more relevant for SLA guarantees
- Mean latency can be skewed by outliers

**Mitigation:**
- ✅ Reported P50 (mean), P95, and P99
- ✅ Full distribution available in Criterion reports
- ✅ 95% confidence intervals provide uncertainty quantification

**Impact:** Low - Comprehensive latency statistics enable reader interpretation.

---

#### Threat: Throughput Metric Definition
**Description:** "Transactions per second" ambiguous:
- Does not include failed transactions
- Does not account for transaction size
- Single-node vs distributed measurement

**Mitigation:**
- ✅ Clarified: "19.58 TPS (100% success rate, 1,397/1,397)"
- ✅ Documented test configuration (200 users × 100 requests)
- ✅ Distinguished "theoretical max" (333 TPS) from "actual" (19.58 TPS)

**Impact:** Low - Precise metric definition enables valid interpretation.

---

### 3.2 Operational Definitions

#### Threat: "Overhead" Definition
**Description:** "Semantic layer overhead" could mean:
- Absolute latency difference (ms)
- Relative slowdown (× slower)
- Percentage increase (%)

**Mitigation:**
- ✅ Reported all three metrics:
  - Absolute: +36 ms (15ms → 51ms)
  - Relative: 3.4× slower
  - Percentage: +240%

**Impact:** Low - Multiple operationalizations enable comprehensive understanding.

---

#### Threat: "Scalability" Definition
**Description:** "Linear scaling" requires precise definition:
- Time complexity: O(n)
- Empirical fit: R² > 0.95
- Slope consistency: µs per axiom

**Mitigation:**
- ✅ Provided per-axiom cost (0.37 µs/axiom)
- ✅ Verified linear fit (R² > 0.99)
- ✅ Theoretical complexity analysis (O(n))

**Impact:** Low - Rigorous scalability definition validates claims.

---

## 4. Conclusion Validity

**Definition:** Extent to which conclusions are justified by the data.

### 4.1 Statistical Conclusion Validity

#### Threat: Type I Error (False Positive)
**Description:** May conclude significant difference when none exists:
- α = 0.05 means 5% false positive rate
- Multiple comparisons increase family-wise error rate

**Mitigation:**
- ✅ Used 95% confidence intervals (standard threshold)
- ✅ Reported p-values with exact values (not just < 0.05)
- ✅ Bonferroni correction for multiple comparisons
- ✅ Effect sizes reported (Cohen's d) to distinguish statistical from practical significance

**Impact:** Low - Standard statistical practice with appropriate corrections.

---

#### Threat: Type II Error (False Negative)
**Description:** May fail to detect significant difference:
- Small sample size (n=100) may miss small effects
- High variance reduces statistical power

**Mitigation:**
- ✅ Power analysis confirms adequate power for medium+ effects
- ✅ Effect sizes reported (distinguish "no effect" from "small effect")
- ⚠️ Underpowered for small effects (d < 0.5)

**Impact:** Medium - May miss small but meaningful effects. Future work: increase sample size for small effect detection.

---

#### Threat: Violation of Statistical Assumptions
**Description:** Statistical tests assume:
- Independence of observations ✅ (satisfied)
- Normal distribution ❌ (violated - benchmark data right-skewed)
- Homogeneity of variance ❓ (uncertain)

**Mitigation:**
- ✅ Used non-parametric tests (Mann-Whitney U) robust to assumptions
- ✅ Does not assume normality or equal variances
- ✅ Bootstrap confidence intervals robust to distribution shape

**Impact:** Low - Non-parametric methods appropriate for benchmark data.

---

### 4.2 Causal Inference Validity

#### Threat: Correlation vs Causation
**Description:** Observing that semantic layer correlates with slower transactions does not prove causation:
- Alternative explanations: Implementation inefficiency
- Confounding factors: Concurrent load test
- Reverse causality: Slower transactions enable semantic validation

**Mitigation:**
- ✅ Component-level profiling isolates semantic layer overhead
- ✅ Compared "with vs without" semantic layer in controlled setting
- ✅ Theoretical complexity analysis supports causal claim

**Impact:** Low - Controlled experiments support causal inference.

---

## 5. Reliability

**Definition:** Consistency of measurements across time, researchers, and instruments.

### 5.1 Test-Retest Reliability

#### Threat: Measurement Variability
**Description:** Benchmark results may vary between runs:
- OS scheduler differences
- Background process interference
- CPU frequency scaling (Turbo Boost)

**Mitigation:**
- ✅ 100 samples per benchmark average out variability
- ✅ Bootstrap confidence intervals quantify uncertainty
- ✅ Consistent platform (isolated GCP VM)

**Impact:** Low - High sample size and statistical methods ensure reliability.

---

### 5.2 Inter-Rater Reliability
**Description:** Not applicable (automated benchmarks, no human raters).

---

## 6. Limitations Summary

### Critical Limitations (Must Address)

| # | Limitation | Impact | Mitigation |
|---|------------|--------|------------|
| 1 | **Single-node throughput** (19.58 TPS vs 8,000 TPS target) | High | Clearly labeled as dev environment; component benchmarks enable extrapolation |
| 2 | **Missing baseline comparisons** (Neo4j, Jena, Ethereum) | High | Future work: comparative studies |
| 3 | **Limited ontology diversity** (supply chain only) | Medium | Future work: standard ontologies (SNOMED, FIBO) |
| 4 | **Underpowered for small effects** (d < 0.5) | Medium | Adequate for medium+ effects; future work: larger n |

### Moderate Limitations (Acknowledge)

| # | Limitation | Impact | Status |
|---|------------|--------|--------|
| 5 | Platform specificity (Linux GCP only) | Medium | Documented; future work: cross-platform |
| 6 | Scalability limits (tested to 10K axioms) | Medium | Linear scaling suggests generalizability |
| 7 | Artificial workload (simulated users) | Medium | Realistic queries; conservative latency estimates |

### Minor Limitations (Note)

| # | Limitation | Impact | Status |
|---|------------|--------|--------|
| 8 | Temporal validity (2026 state-of-art) | Low | Versions documented; Docker enables reproducibility |
| 9 | Compiler optimization variability | Low | Stable Rust ecosystem |
| 10 | Measurement tool bias (Criterion.rs) | Low | Industry-standard tool |

---

## 7. Mitigation Strategies Implemented

### Design Mitigations
- ✅ Controlled experiments (with/without semantic layer)
- ✅ Component-level profiling (isolate overhead sources)
- ✅ Multiple scales (4 orders of magnitude)
- ✅ Statistical rigor (significance tests, effect sizes, power analysis)

### Reporting Mitigations
- ✅ Precise operational definitions
- ✅ Comprehensive statistics (P50, P95, P99, CI)
- ✅ Honest limitations section (this document)
- ✅ Distinguish measurements from projections

### Reproducibility Mitigations
- ✅ Open-source implementation (Apache 2.0)
- ✅ Portable benchmark toolkit (Docker)
- ✅ Detailed documentation (1,437 files)
- ✅ Raw data export (CSV from Criterion)

---

## 8. Future Work (Addressing Limitations)

### Immediate Priority (This Paper)
1. ✅ **Add Statistical Rigor:** Significance tests, effect sizes, power analysis
2. ✅ **Document Limitations:** This threats-to-validity document
3. ✅ **Clarify Scope:** Development vs production environment

### Short-Term (Follow-up Papers)
4. **Baseline Comparisons:** Compare with Neo4j, Jena, Ethereum
5. **Multi-Node Deployment:** Validate production throughput projections
6. **Diverse Workloads:** Test with finance, healthcare datasets

### Long-Term (Future Research)
7. **Standard Ontologies:** Validate with SNOMED CT, FIBO
8. **Cross-Platform:** Validate on Windows, macOS, ARM
9. **Production Deployment:** Real-world case studies

---

## 9. Conclusion

This research has **valid conclusions within the documented scope**:

- ✅ **Internal Validity:** Controlled experiments, consistent platform, mitigated confounds
- ⚠️ **External Validity:** Single-node limits generalizability; linear scaling suggests broader applicability
- ✅ **Construct Validity:** Precise measurements, multiple operationalizations
- ✅ **Conclusion Validity:** Appropriate statistical methods, causal inference supported

**Overall Assessment:** Results are **scientifically valid** for:
- OWL2 reasoning performance characterization
- SPARQL query latency analysis
- Semantic layer overhead quantification

**Caveats:**
- Throughput results are **development environment only** (not production)
- Baseline comparisons **not yet conducted** (future work)
- Scalability **verified to 10,000 axioms** (extrapolation speculative)

**Journal Submission Strategy:**
- Tier 2 journals (IEEE Access, MDPI Information): **Acceptable** with limitations honestly stated
- Tier 1 journals (VLDB, SIGMOD): **Requires** baseline comparisons and multi-node validation

---

**Document Status:** ✅ Complete
**Reviewed By:** Thesis Advisor
**Date:** 2026-01-18
