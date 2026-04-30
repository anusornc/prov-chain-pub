# ProvChain-Org Benchmark Toolkit

🚀 **Portable, One-Command Benchmark Suite for Any Machine**

Automated performance comparison between ProvChain-Org (blockchain with embedded ontology) and traditional systems like Neo4j.

## 🎯 What This Does

- ✅ **Auto-detects** your hardware capabilities
- ✅ **Optimizes** configuration for your machine
- ✅ **Deploys** all services (ProvChain + Neo4j + Monitoring)
- ✅ **Runs** comprehensive benchmarks
- ✅ **Generates** comparison reports
- ✅ **Visualizes** results in Grafana dashboards

## ⚡ Quick Start (4 Steps)

### 1. Extract the Toolkit

```bash
tar -xzf provchain-benchmark-toolkit.tar.gz
cd provchain-benchmark-toolkit
```

### 2. Run the Local Preflight Gate

```bash
chmod +x scripts/preflight-trace-benchmark.sh
./scripts/preflight-trace-benchmark.sh
```

### 3. Run the Benchmark

```bash
chmod +x run.sh
./run.sh
```

That's it! The script will:
- Detect your hardware (RAM, CPU, disk)
- Select optimal configuration
- Start all services
- Run benchmarks
- Display results

### 4. View Results

**Grafana Dashboard**: http://localhost:3000 (admin/admin)

## 🔧 Hardware Profiles

The toolkit automatically detects your hardware and selects the optimal profile:

| Profile | RAM | CPU | Dataset | Iterations | Best For |
|---------|-----|-----|---------|------------|----------|
| **low** | 4GB | 2 cores | 100 tx | 3 | Development laptops |
| **medium** | 8GB | 4 cores | 1,000 tx | 10 | Standard testing ✅ |
| **high** | 16GB | 8 cores | 5,000 tx | 20 | Powerful workstations |
| **ultra** | 32GB+ | 16+ cores | 10,000 tx | 50 | Servers, cloud |

### Manual Profile Selection

```bash
# Force specific profile
./run.sh medium
./run.sh high
./run.sh ultra
```

## 📊 What Gets Benchmarked

### 1. Query Performance
- Simple product lookup (by batch ID)
- Multi-hop traceability (10-hop supply chain)
- Aggregation queries (production by farm)
- **Metrics**: Query latency, throughput

### 2. Write Performance
- Single-threaded writes (1000 transactions)
- Concurrent writes (10, 50, 100 users)
- **Metrics**: Transactions/sec, confirmation time

### 3. Permission Control
- Public vs Private data overhead
- Permission check throughput
- **Metrics**: Access control overhead %

## 📁 Toolkit Structure

```
provchain-benchmark-toolkit/
├── run.sh                          # ⭐ Main script (run this!)
├── docker-compose.yml              # Service orchestration
├── configs/
│   ├── low.conf                    # Low-end hardware config
│   ├── medium.conf                 # Medium hardware config
│   ├── high.conf                   # High-end hardware config
│   ├── ultra.conf                  # Ultra hardware config
│   ├── prometheus.yml              # Metrics scraping
│   └── grafana/
│       ├── provisioning/           # Auto-provision datasources
│       └── dashboards/             # Grafana dashboards
├── data/
│   └── supply_chain.ttl            # RDF dataset (1000 triples)
├── src/                            # Benchmark runner source
│   ├── main.rs
│   ├── Cargo.toml
│   └── Dockerfile
├── results/                        # 📊 Benchmark output
│   ├── benchmark_results.json
│   ├── benchmark_results.csv
│   └── summary.md
└── logs/                           # Service logs
```

## 🖥️ System Requirements

### Minimum (Low Profile)
- **OS**: Linux, macOS, or Windows with WSL2
- **RAM**: 4GB
- **CPU**: 2 cores
- **Disk**: 10GB free
- **Docker**: 20.0+

### Recommended (Medium Profile)
- **OS**: Linux, macOS, or Windows with WSL2
- **RAM**: 8GB
- **CPU**: 4 cores
- **Disk**: 20GB free
- **Docker**: 20.0+

### For Best Results (High/Ultra Profile)
- **OS**: Linux (Ubuntu 22.04+ recommended)
- **RAM**: 16GB+
- **CPU**: 8+ cores
- **Disk**: 50GB free
- **SSD** (for better I/O performance)

## 📦 Installing Prerequisites

### Ubuntu/Debian

```bash
# Install Docker
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER

# Log out and back in for group change to take effect
```

### macOS

```bash
# Download and install Docker Desktop
# https://www.docker.com/products/docker-desktop
```

### Windows (WSL2)

```bash
# Enable WSL2
wsl --install

# Download Docker Desktop for Windows
# https://www.docker.com/products/docker-desktop
```

## 🎮 Usage Examples

### Basic Usage

```bash
# Auto-detect hardware and run
./run.sh

# Specify profile manually
./run.sh medium

# Clean old results before running
CLEAN_RESULTS=true ./run.sh
```

### Development Workflow

```bash
# Quick test (low profile, fast)
./run.sh low

# Full test (medium profile, recommended)
./run.sh medium

# View logs
docker-compose -f docker-compose.yml logs -f provchain

# Stop services
docker-compose -f docker-compose.yml down

# Clean everything (including data)
docker-compose -f docker-compose.yml down -v
rm -rf results/* logs/*
```

### Accessing Services

| Service | URL | Credentials |
|---------|-----|-------------|
| **Grafana** | http://localhost:3000 | admin/admin |
| **Prometheus** | http://localhost:9091 | - |
| **ProvChain API** | http://localhost:8080 | - |
| **Neo4j Browser** | http://localhost:7474 | neo4j/benchmark |

## 📈 Understanding Results

### Summary File

Results are saved in `results/summary.md`:

```markdown
## Query Performance

### Simple Product Lookup
- **ProvChain-Org**: 45.23 ms (22.11 ops/sec)
- **Neo4j**: 67.89 ms (14.73 ops/sec)
- **Improvement**: 33.4%
- **Winner**: ProvChain-Org ✅
```

### Grafana Dashboard

Open http://localhost:3000 and view:
- Transaction Duration (ms)
- Request Rate (req/sec)
- Latency Distribution (p50, p95)
- Error Rate (%)
- CPU/Memory Usage

### Raw Data

- **JSON**: `results/benchmark_results.json` - All detailed metrics
- **CSV**: `results/benchmark_results.csv` - Spreadsheet compatible
- **Environment Manifest**: `results/.../environment_manifest.json` - Execution environment for the run
- **Logs**: `logs/` - Service logs for debugging

## 🔍 Troubleshooting

### Docker not running

```bash
# Start Docker Desktop (macOS/Windows)
# Or start Docker service (Linux)
sudo systemctl start docker
```

### Port already in use

```bash
# Check what's using the port
lsof -i :8080  # ProvChain
lsof -i :7474  # Neo4j
lsof -i :3000  # Grafana

# Change ports in docker-compose.yml if needed
```

### Out of memory

```bash
# Use a lower profile
./run.sh low

# Or increase Docker memory limit
# Docker Desktop > Settings > Resources > Memory
```

### Services not starting

```bash
# Check service logs
docker-compose -f docker-compose.yml logs provchain
docker-compose -f docker-compose.yml logs neo4j

# Restart services
docker-compose -f docker-compose.yml restart
```

### Benchmark results are empty

```bash
# Wait for services to be healthy
curl http://localhost:8080/health  # Should return 200 OK
curl http://localhost:7474         # Neo4j browser should load

# Check benchmark runner logs
docker logs benchmark-runner
```

## 🚀 Advanced Usage

### Running on Remote Server

```bash
# Copy toolkit to server
scp -r provchain-benchmark-toolkit/ user@server:/path/to/

# SSH to server
ssh user@server

# Run benchmark
cd /path/to/provchain-benchmark-toolkit
./run.sh
```

### Custom Configuration

Edit `configs/<profile>.conf`:

```bash
# Change number of iterations
ITERATIONS=20

# Change dataset size
DATASET_SIZE=5000

# Change concurrent users
CONCURRENT_USERS=50
```

### Running Multiple Times

```bash
# Run 3 times and average results
for i in {1..3}; do
    ./run.sh medium
    mv results/summary.md "results/summary-run-$i.md"
done
```

## 📚 Thesis Integration

### Generating Figures for Thesis

```bash
# Run benchmark
./run.sh medium

# Results are in:
# - results/benchmark_results.json (data)
# - results/benchmark_results.csv (for plotting)
# - http://localhost:3000 (visualizations)

# Export Grafana dashboard
# 1. Open dashboard
# 2. Click Share > Export
# 3. Save as PNG/PDF for thesis
```

### Sample Thesis Text

```markdown
## Performance Evaluation

### Experimental Setup

We evaluated ProvChain-Org against Neo4j graph database using
a standardized RDF dataset containing 1000 triples representing
food supply chain transactions. Benchmarks were conducted on a
system with 8GB RAM and 4 CPU cores, using 10 iterations per
test.

### Results

ProvChain-Org demonstrated 33.4% faster query performance for
multi-hop traceability queries (45.23ms vs 67.89ms). Transaction
throughput reached 150 tx/sec compared to Neo4j's 95 tx/sec,
representing a 58% improvement.
```

## 🎓 Citation

If you use this benchmark toolkit in your research, please cite:

```bibtex
@mastersthesis{chaikaew2024provchain,
  title={Enhancement of Blockchain with Embedded Ontology and
         Knowledge Graph for Data Traceability},
  author={Chaikaew, Anusorn},
  year={2024},
  school={Chiang Mai University}
}
```

## 🤝 Contributing

To extend the toolkit:

1. Add new benchmarks in `src/main.rs`
2. Create new profile in `configs/`
3. Update this README
4. Test on multiple hardware configurations

## 📧 Support

For issues or questions:
- Check troubleshooting section above
- Review logs in `logs/` directory
- Check service health: `docker-compose ps`

## 📄 License

This benchmark toolkit is part of the ProvChain-Org thesis research.

---

**Version**: 1.0.0
**Last Updated**: 2024-01-04
**Compatible with**: ProvChain-Org v1.0+
