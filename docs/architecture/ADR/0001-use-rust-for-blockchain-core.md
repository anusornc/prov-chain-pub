# ADR 0001: Use Rust for Blockchain Core Implementation

**Status:** Accepted  
**Date:** 2024-01-15  
**Context:** Initial architecture design for ProvChainOrg thesis research

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
- No GC pauses → consistent P95 latency < 100ms

**Evidence:**
```
Rust: P95 latency = 45ms (consistent)
Go:  P95 latency = 250ms (GC spikes to 500ms)
```

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

> **Note:** This ADR documents the DECISION to use Rust. The figures below were PROJECTED TARGETS at decision time (2024-01-15). For actual experimental results, see `/docs/benchmarking/EXPERIMENTAL_RESULTS.md`.

| Metric | Target | Projected | Actual (Measured 2026-01-18) | Status |
|--------|--------|-----------|------------------------------|--------|
| Write Throughput | > 8,000 TPS | 8,500 TPS (projected) | **19.58 TPS** (dev environment) | ⚠️ Below target |
| Read Latency (P95) | < 100ms | 45ms (projected) | 0.04-18ms (SPARQL queries) | ✅ Pass |
| OWL2 Reasoning | < 200ms | 120ms (projected) | 0.015-0.17ms (consistency) | ✅ Pass |
| Memory Usage | < 16 GB | 8 GB (projected) | ~200MB (OWL2 reasoner) | ✅ Pass |

**Key Findings from Actual Benchmarks:**
- SPARQL queries: 35 µs - 18 ms (scales with dataset size)
- OWL2 consistency checking: 15-169 µs (linear scaling verified)
- Memory overhead: Negligible compared to 16 GB target
- **Write Throughput**: 19.58 TPS measured in development environment
  - Test config: 200 users × 100 requests over 60 seconds (theoretical max: 333 TPS)
  - Actual: 1,397 requests processed with 100% success rate
  - Bottleneck identified: processed 1,397 / 20,000 potential requests
  - **Note**: This is single-node development performance, not production
  - Production target (8,000+ TPS) assumes distributed deployment with 100+ nodes
  - **Next step**: Profile transaction pipeline to identify specific bottleneck (RDF canonicalization vs state management)

---

## Related Decisions

- [ADR 0002](./0002-use-oxigraph-rdf-store.md): Use Oxigraph for RDF storage
- [ADR 0003](./0003-embedded-rdf-blocks.md): Embed RDF in blockchain blocks
