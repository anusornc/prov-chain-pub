# ADR 0001: Use Rust for Blockchain Core Implementation

**Status:** Accepted  
**Date:** 2024-01-15  
**Context:** Initial architecture design for ProvChainOrg thesis research

> **Evidence amendment (2026-08-31):** this ADR still governs the language choice, but its
> historical performance targets are not current results. The former custom load-test statistics
> are withdrawn because the harness has defective running-average/percentile calculations and no
> admitted raw-sample artifact. No ledger-throughput value is established by this ADR.

---

## Context

ProvChainOrg requires a blockchain implementation that prioritizes:
- Performance: Target > 8,000 TPS throughput
- Security: Memory safety and cryptographic correctness
- Concurrency: Multi-threaded transaction processing
- Reliability: Predictable performance under load

Language options considered:
- **Go:** Good concurrency, but GC pauses cause latency spikes
- **C++:** Maximum performance, but memory safety concerns
- **Java:** Mature ecosystem, but GC and startup overhead
- **Rust:** Memory safety + zero-cost abstractions + fearless concurrency

---

## Decision

**Use Rust as the primary implementation language for blockchain core components.**

### Scope

**Implemented in Rust:**
- Blockchain core (state management, consensus, block creation)
- P2P networking
- Semantic layer (OWL2 reasoner, SHACL validator)
- Security (encryption, key management)
- Storage (RDF store integration)

---

## Rationale

### 1. Memory Safety Without GC

**Problem:** Garbage collection causes unpredictable latency spikes

**Rust Solution:**
- Compile-time memory safety guarantees (borrow checker)
- Deterministic memory deallocation (RAII pattern)
- No language-runtime garbage collector

This rationale is qualitative. The project does not admit a controlled Rust-versus-Go latency
comparison from this ADR.

### 2. Zero-Cost Abstractions

- Monomorphization (compile-time polymorphism)
- Inline optimization by default
- LLVM backend for machine code generation

### 3. Fearless Concurrency

- `Send` and `Sync` traits prevent data races at compile time
- `async/await` with Tokio runtime
- Message passing with `mpsc` channels

### 4. Strong Type System

- Newtype pattern for wrappers
- Compile-time type checking
- No null values (Option<T> forces handling)

---

## Performance Validation

> **Note:** This ADR documents the decision to use Rust. The target/projected columns below are
> historical design inputs from 2024-01-15. Dated component outputs must be checked through the
> current benchmark evidence boundary; `EXPERIMENTAL_RESULTS.md` is a mixed historical record, not
> an authority for current paper claims.

| Metric | Target | Projected | Dated component status | Evidence status |
|--------|--------|-----------|------------------------|-----------------|
| Write Throughput | > 8,000 TPS | 8,500 TPS (projected) | Custom load-test result withdrawn | No corrected ledger-throughput artifact |
| Read Latency (P95) | < 100ms | 45ms (projected) | 0.04-18ms (historical SPARQL component draft) | Historical only; not admitted here |
| OWL2 Reasoning | < 200ms | 120ms (projected) | 0.015-0.17ms (legacy consistency path) | Historical only; not production semantic-path evidence |
| Memory Usage | < 16 GB | 8 GB (projected) | ~200MB (historical reasoner-helper draft) | Historical only; environment artifact incomplete |

**Historical draft annotations (not current admitted findings):**
- SPARQL queries: 35 µs - 18 ms was recorded without a complete admitted artifact here
- OWL2 consistency checking: 15-169 µs was measured on a legacy component path
- Memory overhead: no current claim is admitted from the historical comparison to the 16 GB target
- **Write throughput**: no value admitted; a corrected harness must measure the exact durable
  admission/commit path, retain raw samples, and archive reproducible environment metadata

---

## Related Decisions

- [ADR 0002](./0002-use-oxigraph-rdf-store.md): Use Oxigraph for RDF storage
- [ADR 0003](./0003-embedded-rdf-blocks.md): Embed RDF in blockchain blocks
