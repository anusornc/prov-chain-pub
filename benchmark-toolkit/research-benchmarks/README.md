# ProvChain-Org Benchmark Suite

Current active benchmark harness for Docker/adaptor campaigns and paper-facing evidence.

Publication and thesis claims must use curated exports documented by `docs/benchmarking/BENCHMARK_EVIDENCE_BOUNDARY_2026-05-11.md`. Do not use the legacy portable runner under `benchmark-toolkit/src/` as evidence.

Automated benchmark infrastructure for comparing ProvChain-Org (blockchain with embedded ontology) against traditional systems like Neo4j, Hyperledger Fabric, Ethereum, and FlureeDB.

## Overview

This benchmark suite is designed to measure thesis research objectives with
family-scoped evidence:

1. **Query Performance**: measure trace-query behavior for supported comparator paths
2. **Cross-Chain Interchange**: exercise documented bridge/admission scenarios where implemented
3. **Permission Control Efficiency**: measure policy/access-control workloads within their caveats
4. **Comparative Metrics**: export family-scoped metrics without a single global winner claim

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Benchmark Orchestrator                     │
│                  (benchmark-runner)                          │
└─────────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ▼                   ▼                   ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│ ProvChain-Org │   │    Neo4j      │   │  Prometheus   │
│   (RDF/SPARQL)│   │  (Graph DB)   │   │  (Metrics)    │
└───────────────┘   └───────────────┘   └───────────────┘
        │                   │
        └───────────────────┴───────────────┐
                                            ▼
                                    ┌───────────────┐
                                    │    Grafana    │
                                    │ (Dashboard)   │
                                    └───────────────┘
```

## Quick Start

### Prerequisites

- Docker and Docker Compose
- 8GB RAM minimum
- 10GB disk space

### Required preflight before Docker benchmark

Run the local trace-benchmark gate first:

```bash
cd ..
./scripts/preflight-trace-benchmark.sh
```

This must pass before `docker compose` reruns. It keeps benchmark regressions in local tests instead of discovering them only after container rebuilds.

The preflight also includes the local `Fabric` gateway contract tests. These are mock-gateway tests and do not imply that a real `Fabric` runtime is already available.

## Fluree Re-Enable Workflow

`Fluree` ถูกปิดไว้โดยค่าเริ่มต้นใน trace stack ปัจจุบัน

เมื่อจะเปิดใช้งาน:

```bash
export FLUREE_IMAGE=fluree/ledger:<explicit-tag>
export FLUREE_GROUP_PRIVATE_KEY=<64-hex-secp256k1-private-key>
export BENCHMARK_SKIP_FLUREE=false

cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml fluree -- --nocapture
./benchmark-toolkit/scripts/probe-fluree-ledger.sh

docker compose -f benchmark-toolkit/docker-compose.trace.yml --profile fluree up --build
```

ห้ามใช้ `latest` เป็นหลักฐาน benchmark

### Legacy Quick Comparison (30 minutes, non-evidence)

This historical quick comparison is retained for local smoke/debug context only.
Do not use its `benchmark/results/summary.md` output for thesis or paper claims.
Use curated exports under `docs/benchmarking/data/` or
`docs/benchmarking/data/reference/` for evidence.

Compare ProvChain-Org vs Neo4j locally:

```bash
# Navigate to deploy directory
cd deploy

# Start benchmark comparison
docker-compose -f docker-compose.benchmark-comparison.yml up -d

# Run benchmarks
docker-compose -f docker-compose.benchmark-comparison.yml run --rm benchmark-runner --all

# View local non-evidence smoke results
cat ../benchmark/results/summary.md

# Access Grafana Dashboard
open http://localhost:3002/d/provchain-benchmark
```

### Using the Automated Scripts

```bash
cd benchmark

# Run all benchmarks (default 10 iterations)
./scripts/run-benchmarks.sh all 10

# Run only query benchmarks
./scripts/run-benchmarks.sh query 10

# Run only write benchmarks
./scripts/run-benchmarks.sh write 10

# Analyze results
./scripts/analyze-results.sh
```

## Datasets

### supply_chain_1000.ttl

**Format**: RDF/Turtle (TTL)
**Size**: ~1000 triples
**Content**: Food supply chain with provenance tracking

Features:
- Multi-hop traceability (10 hops)
- Multiple product types (tomato, lettuce, carrot, etc.)
- Permission variations (Public/Private/Restricted)
- Cross-ontology examples (food, pharma, automotive)
- Temporal distribution across multiple dates

**Sample Query**:
```sparql
PREFIX ex: <http://example.org/supplychain/>
PREFIX trace: <http://example.org/traceability#>

SELECT ?product ?producer ?processor
WHERE {
  ?product ex:batchId "BATCH017" .
  ?product trace:hasProducer ?producer .
  ?product trace:processedBy ?processor .
}
```

## Benchmark Scenarios

### Scenario 1: Query Performance

**Goal**: Measure trace-query latency/throughput for the supported RDF and translated graph-model comparator paths

Tests:
1. Simple product lookup (by batch ID)
2. Multi-hop traceability (10-hop supply chain)
3. Complex provenance query with temporal filters
4. Aggregation queries (production volume by farm)

**Metrics**:
- Query latency (ms)
- Throughput (queries/second)
- Memory usage during query

### Scenario 2: Write Performance

**Goal**: Compare ledger/write-path behavior only for systems and workloads with curated campaign exports

Tests:
1. Single-threaded write (1000 transactions)
2. Concurrent writes (10, 50, 100 concurrent users)
3. Burst writes (sustained 100 tx/sec for 60 seconds)

**Metrics**:
- Transactions/second
- Average confirmation time (ms)
- Block time (ms)

### Scenario 3: Permission Control Overhead

**Goal**: Measure access-control overhead within the documented policy workload caveats

Tests:
1. Write without permission check (baseline)
2. Write with public permission
3. Write with private permission (owner-only)
4. Mixed workload (50% public, 50% private)

**Metrics**:
- Write latency overhead (%)
- Query latency overhead (%)
- Permission check throughput

## Results

### Output Files

```
benchmark/results/
├── benchmark_results.json      # Raw results in JSON format
├── benchmark_results.csv       # Spreadsheet-compatible data
├── summary.json                # Statistical summary
└── summary.md                  # Human-readable report
```

### Interpreting Results

**Key Metrics**:
- **duration_ms**: Lower is better (faster queries/transactions)
- **operations_per_second**: Higher is better (more throughput)
- **improvement_percent**: Percentage improvement vs baseline
- **winner**: Which system performed better

**Legacy Example Summary**:
```
### Query Performance
- Use curated campaign exports under `docs/benchmarking/data/` or
  `docs/benchmarking/data/reference/`.
- Interpret each row only inside its benchmark family and caveats.
- Do not cite legacy portable-runner summaries as thesis/paper evidence.
```

## Monitoring

### Grafana Dashboard

Access: http://localhost:3002/d/provchain-benchmark

Panels:
1. Transaction Duration (ms)
2. Request Rate (req/sec)
3. Latency Distribution (p50, p95)
4. Error Rate (%)
5. CPU Usage (%)
6. Memory Usage (bytes)
7. Throughput (ops/sec)

### Prometheus

Access: http://localhost:9092

Key Queries:
```promql
# Average transaction duration
rate(provchain_transaction_duration_seconds_sum[5m]) * 1000

# Request rate
rate(provchain_http_requests_total[1m])

# P95 latency
histogram_quantile(0.95, rate(provchain_transaction_duration_seconds_bucket[5m])) * 1000

# Error rate
rate(provchain_http_requests_total{status=~"5.."}[5m]) / rate(provchain_http_requests_total[5m]) * 100
```

## Troubleshooting

### Containers not starting

**Problem**: Services fail healthcheck
**Solution**:
```bash
# Check logs
docker-compose -f docker-compose.benchmark-comparison.yml logs

# Restart services
docker-compose -f docker-compose.benchmark-comparison.yml restart
```

### Out of memory errors

**Problem**: Containers crash due to memory limits
**Solution**:
```bash
# Increase Docker memory limit in Docker Desktop
# Or reduce concurrent iterations:
./scripts/run-benchmarks.sh all 5
```

### Benchmark results are empty

**Problem**: No results generated
**Solution**:
```bash
# Verify services are healthy
curl http://localhost:8080/health  # ProvChain
curl http://localhost:7474         # Neo4j

# Check benchmark runner logs
docker logs benchmark-runner
```

### Neo4j connection refused

**Problem**: Cannot connect to Neo4j
**Solution**:
```bash
# Neo4j takes time to start (~45 seconds)
# Wait for healthcheck or increase wait time:
wait_for_services() {
    # Increase from 60 to 120 seconds
    for i in {1..120}; do ...
}
```

## Extending the Benchmarks

### Adding New Datasets

1. Create TTL file in `benchmark/datasets/`
2. Update `benchmark/src/main.rs` to load new dataset
3. Rebuild: `docker-compose build benchmark-runner`

### Adding New Scenarios

1. Add scenario function in `benchmark/src/main.rs`
2. Update CLI args in `Args` struct
3. Register scenario in `main()` function
4. Rebuild and rerun

### Adding Comparison Systems

1. Add service to `deploy/docker-compose.benchmark-comparison.yml`
2. Implement client in `benchmark/src/main.rs`
3. Add metrics to Prometheus config
4. Update Grafana dashboard

## Performance Tuning

### For Faster Benchmarks

- Reduce iterations: `./scripts/run-benchmarks.sh all 5`
- Use smaller dataset: Create `supply_chain_100.ttl`
- Disable monitoring: Comment out Prometheus/Grafana services

### For More Accurate Results

- Increase iterations: `./scripts/run-benchmarks.sh all 50`
- Run multiple times and average: `for i in {1..3}; do ./scripts/run-benchmarks.sh; done`
- Use production-grade hardware
- Disable other system processes

## Thesis Integration

### Generating Thesis Figures

```bash
# Run benchmarks
./scripts/run-benchmarks.sh all 10

# Generate CSV for plotting
./scripts/analyze-results.sh

# Create figures (Python)
python scripts/generate_thesis_figures.py
```

### Documenting Methodology

Use a claim-bounded thesis template:

```markdown
## Performance Evaluation

### Experimental Setup

We evaluated ProvChain-Org on the <benchmark family> workload using
the curated export <artifact path>. The campaign status, runtime
context, comparator role, and caveats are recorded in the export.

### Results

The result supports only the <trace-query / semantic-admission /
ledger-write / policy / diagnostic> claim stated by the evidence
artifact. It must not be generalized to unrelated benchmark families
or production deployment readiness.
```

## Architecture Decisions

### Why RDF/N-Triples?

Universal format compatible with:
- ProvChain-Org (native RDF storage)
- Neo4j (via SPARQL plugin)
- FlureeDB (native RDF)
- Ethereum (via serialization)

### Why Docker Compose?

- Reproducible environments
- Easy cleanup and restart
- Isolated networking
- Resource limits

### Why Rust for Benchmark Runner?

- Performance (zero-cost abstractions)
- Type safety (compile-time guarantees)
- Async support (tokio)
- Easy containerization

## Future Work

### Phase 2: Full Benchmark Suite

- Add Ethereum (dev mode)
- Add FlureeDB
- Add Hyperledger Fabric
- Cross-chain sync benchmarks
- Permission control benchmarks

### Phase 3: Advanced Features

- Distributed benchmark runners
- Real-time monitoring during execution
- Automated report generation
- Statistical significance testing

## Contributing

When adding benchmarks:
1. Follow existing patterns
2. Add comprehensive comments
3. Update this README
4. Test locally before committing

## License

This benchmark suite is part of the ProvChain-Org thesis research.

## Contact

For questions or issues, contact the research team.

---

**Last Updated**: 2024-01-04
**Version**: 0.1.0
