# ProvChain-Org Benchmarking Resources

🎯 **Quick Start**: `cd benchmark-toolkit && ./run.sh`

Central entry point for ALL performance testing and benchmarking resources in the ProvChain-Org project.

---

## 📚 Complete Benchmarking Documentation

### 1. Portable Benchmark Toolkit (Recommended for Most Users)

**Location**: [`benchmark-toolkit/`](benchmark-toolkit/)

**Purpose**: One-command performance testing on any machine with Docker

**Key Features**:
- ✅ Auto-detects hardware (4GB-32GB+ RAM)
- ✅ One-command execution
- ✅ Portable (copy anywhere)
- ✅ Real-time Grafana dashboards
- ✅ CSV/JSON results export

**Documentation**:
- 📘 [Full Guide](benchmark-toolkit/README.md) - Complete toolkit documentation
- 🚀 [Quick Start](benchmark-toolkit/QUICKSTART.md) - Quick reference card
- 📦 [Deployment Guide](benchmark-toolkit/DEPLOYMENT_GUIDE.md) - Usage guide
- 🔄 [Portability](benchmark-toolkit/PORTABILITY.md) - Portable deployment

**Run Now**:
```bash
cd benchmark-toolkit
./run.sh
```

---

### 2. Documentation Benchmarks

**Location**: [`docs/benchmarking/`](docs/benchmarking/)

**Purpose**: Research-focused benchmarking guides and academic publication support

**Documentation**:
- 📊 [Central Guide](docs/benchmarking/README.md) - Comprehensive benchmarking guide
- 🔬 [Academic Plan](docs/BENCHMARKING_PLAN.md) - Publication strategy for high-impact journals
- 📈 [Results Summary](docs/BENCHMARK_RESULTS_SUMMARY.md) - Historical performance data
- 📏 [Criterion Guide](docs/CRITERION_BENCHMARKING_GUIDE.md) - Component micro-benchmarks
- ⚡ [Trace Optimization](docs/TRACE_OPTIMIZATION_BENCHMARK_RESULTS.md) - Frontier Reduction benchmarks

**Research Metrics**:
- Query Performance: SPARQL latency vs traditional systems
- Write Throughput: Transactions per second comparison
- Permission Overhead: Access control performance impact
- Cross-Chain Sync: Inter-chain data interchange speed
- Scalability: Performance vs dataset size analysis

---

### 3. Developer Benchmarks

**Purpose**: Component-level micro-benchmarks and performance testing

**Documentation**:
- 📖 [Performance Testing Guide](docs/performance-testing-guide.md) - Methodology and procedures
- 🧪 [Testing Guide](docs/developer/testing.md) - Unit and integration testing
- 🏗️ [Architecture](docs/developer/architecture.md) - System design and patterns

**Run Component Benchmarks**:
```bash
# Criterion micro-benchmarks
cargo bench

# Load testing
cargo test --test load_tests --release -- --ignored

# Trace optimization benchmarks
cargo bench --bench trace_optimization
```

---

## 🎯 Usage Scenarios

### Scenario 1: Quick Performance Check (5 minutes)

**When**: Quick validation after code changes

```bash
cd benchmark-toolkit
./run.sh low
```

**Output**: Basic performance metrics, minimal dataset (100 transactions)

---

### Scenario 2: Full Performance Analysis (15 minutes)

**When**: Standard testing for development and research

```bash
cd benchmark-toolkit
./run.sh medium
```

**Output**: Comprehensive metrics, standard dataset (1,000 transactions)

**Includes**:
- Query performance benchmarks
- Write throughput tests
- Permission overhead analysis
- Grafana dashboard visualizations

---

### Scenario 3: Research Publication Data (2-4 hours)

**When**: Generating data for thesis or academic papers

```bash
cd benchmark-toolkit
./run.sh high  # or ultra for extensive testing
```

**Output**: Extensive metrics, large datasets (5,000-10,000 transactions)

**Results For**:
- Thesis figures and tables
- Publication-quality graphs
- Statistical analysis data
- Comparative studies

---

### Scenario 4: Component Micro-benchmarks

**When**: Optimizing specific components or algorithms

```bash
# Run all Criterion benchmarks
cargo bench

# Run with baseline comparison
cargo bench -- --save-baseline main
cargo bench -- --baseline main
```

**Output**: Micro-benchmark data for:
- Individual function performance
- Algorithm optimization
- Regression detection

---

## 📊 Benchmark Results

### Latest Results

**Query Performance vs Neo4j** (2024-01-04)
- ProvChain-Org: 45.23ms
- Neo4j: 67.89ms
- **Improvement**: 33.4% faster ✅

**Write Throughput** (2024-01-03)
- ProvChain-Org: 150 tx/sec
- Neo4j: 95 tx/sec
- **Improvement**: 58% higher throughput ✅

**Permission Overhead**
- Additional latency: <10%
- **Result**: Minimal impact ✅

### Access Historical Results

See these directories for detailed benchmark data:
- `benchmark-toolkit/results/` - Latest benchmark runs
- `docs/benchmarking/results/` - Historical archive

### Result Formats

- **Summary**: `results/summary.md` - Human-readable
- **Raw Data**: `results/benchmark_results.json` - Machine-readable
- **Spreadsheet**: `results/benchmark_results.csv` - Excel/analysis
- **Dashboards**: http://localhost:3000 - Grafana visualizations

---

## 🔗 Quick Links

### Project Documentation
- [Main Project README](README.md) - Project overview and quick start
- [Documentation Index](docs/README.md) - Complete documentation
- [Deployment Guides](deploy/) - Production deployment
- [API Reference](docs/api/) - REST, WebSocket, SPARQL APIs

### Toolkit Access
- [Full Toolkit Documentation](benchmark-toolkit/README.md)
- [Quick Start Card](benchmark-toolkit/QUICKSTART.md)
- [Portability Guide](benchmark-toolkit/PORTABILITY.md)
- [Deployment Guide](benchmark-toolkit/DEPLOYMENT_GUIDE.md)

### Research Resources
- [Thesis Document](thesis/thesis.md) - Research objectives
- [Academic Plan](docs/BENCHMARKING_PLAN.md) - Publication strategy
- [Performance Guide](docs/performance-testing-guide.md) - Testing methodology

---

## 🚀 Getting Started

### For New Users

1. **Quick Test** (5 minutes):
   ```bash
   cd benchmark-toolkit
   ./run.sh low
   ```

2. **Review Results**:
   - Open http://localhost:3000 (Grafana)
   - Check `results/summary.md`

3. **Learn More**:
   - Read [benchmark-toolkit/README.md](benchmark-toolkit/README.md)
   - Explore [docs/benchmarking/README.md](docs/benchmarking/README.md)

### For Researchers

1. **Generate Publication Data**:
   ```bash
   cd benchmark-toolkit
   ./run.sh high  # Extensive testing
   ```

2. **Export for Thesis**:
   - Grafana > Share > Export > PNG (for figures)
   - Use `results/benchmark_results.csv` (for analysis)

3. **Document Methodology**:
   - See [docs/BENCHMARKING_PLAN.md](docs/BENCHMARKING_PLAN.md)
   - Reference [docs/performance-testing-guide.md](docs/performance-testing-guide.md)

### For Developers

1. **Component Testing**:
   ```bash
   cargo bench  # Criterion benchmarks
   ```

2. **Load Testing**:
   ```bash
   cargo test --test load_tests --release -- --ignored
   ```

3. **Regression Detection**:
   ```bash
   cargo bench -- --save-baseline main
   # Make changes...
   cargo bench -- --baseline main
   ```

---

## 📞 Troubleshooting

### Common Issues

**Issue**: Docker not running
```bash
sudo systemctl start docker  # Linux
# Or start Docker Desktop     # macOS/Windows
```

**Issue**: Port already in use
```bash
lsof -i :8080  # Check what's using the port
# Change ports in benchmark-toolkit/docker-compose.yml
```

**Issue**: Out of memory
```bash
cd benchmark-toolkit
./run.sh low  # Use lower profile
```

**Issue**: No results generated
```bash
# Check service health
curl http://localhost:8080/health
curl http://localhost:7474

# View logs
cd benchmark-toolkit
docker-compose logs -f
```

### Getting Help

- **Documentation**: Start with [docs/benchmarking/README.md](docs/benchmarking/README.md)
- **Toolkit**: See [benchmark-toolkit/README.md](benchmark-toolkit/README.md)
- **Troubleshooting**: Check [benchmark-toolkit/PORTABILITY.md](benchmark-toolkit/PORTABILITY.md)
- **Issues**: Review logs in `benchmark-toolkit/logs/`

---

## 📈 Performance Metrics Reference

### Query Performance

| Metric | ProvChain-Org | Neo4j | Improvement |
|--------|---------------|-------|-------------|
| Simple Lookup | 12ms | 18ms | 33% faster |
| Multi-hop (10) | 45ms | 68ms | 34% faster |
| Complex Query | 89ms | 134ms | 34% faster |

### Write Performance

| Metric | ProvChain-Org | Neo4j | Improvement |
|--------|---------------|-------|-------------|
| Single-threaded | 120 tx/sec | 85 tx/sec | 41% higher |
| Concurrent (10) | 150 tx/sec | 95 tx/sec | 58% higher |
| Burst (100/s) | 145 tx/sec | N/A | Sustained |

### Overhead Analysis

| Feature | Overhead | Impact |
|---------|----------|--------|
| Permission Check | <5% | Minimal |
| Encryption (ChaCha20) | <8% | Minimal |
| SHACL Validation | <12% | Acceptable |

---

**Last Updated**: 2024-01-04
**Toolkit Version**: 1.0.0
**ProvChain-Org Version**: 1.0.0
