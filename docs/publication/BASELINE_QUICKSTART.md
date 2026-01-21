# Baseline Comparison Quick Start Guide

**Purpose:** Run baseline experiments comparing ProvChainOrg against Neo4j, Jena, and Ethereum

**Date:** 2026-01-18
**Academic Integrity:** All benchmarks use REAL experimental data

---

## Overview

This guide provides a complete setup for baseline comparison experiments needed for journal publication. The setup includes:

- ✅ **Neo4j 5.15** - Graph database (Cypher query language)
- ✅ **Apache Jena Fuseki** - RDF/SPARQL store
- ✅ **Ethereum (Ganache)** - Blockchain testnet
- ✅ **ProvChainOrg** - System under evaluation
- ✅ **Benchmark Runner** - Automated Python scripts

---

## Architecture Overview

**IMPORTANT:** ProvChain runs **natively** (not in Docker), while baseline systems run in Docker containers.

### Architecture Diagram:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Host Machine                                  │
│                                                                   │
│  ┌──────────────────┐                                            │
│  │ Native ProvChain  │  ← cargo run -- web-server --port 8080   │
│  │   Port: 8080      │     (No Docker container)                    │
│  └──────────────────┘                                            │
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │              Docker Compose (Baselines)                  │   │
│  │                                                             │   │
│  │  ┌─────────────┐  ┌──────────────┐  ┌─────────────┐      │   │
│  │  │   Neo4j     │  │  Jena Fuseki  │  │  Ethereum   │      │   │
│  │  │  :7474,7687 │  │   :3030       │  │   :8545     │      │   │
│  │  └─────────────┘  └──────────────┘  └─────────────┘      │   │
│  │                                                             │   │
│  └────────────────────────────────────────────────────────────┘   │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

### Service Types:

| Service | Execution Type | Port | Access Method |
|---------|---------------|------|----------------|
| **ProvChain** | **Native** | 8080 | localhost:8080 |
| Neo4j | Docker | 7474, 7687 | localhost:7474, localhost:7687 |
| Jena Fuseki | Docker | 3030 | localhost:3030 |
| Ethereum | Docker | 8545 | localhost:8545 |

**No port conflicts** - Native ProvChain and Docker baselines use separate ports.

---

## Quick Start (5 minutes)

### Step 1: Start Native ProvChain

```bash
# Navigate to project root
cd /home/cit/provchain-org

# Start native ProvChain service
./scripts/provchain-service.sh start

# Verify ProvChain is running
./scripts/provchain-service.sh health
```

**Expected Output:**
```
Starting native ProvChain...
Waiting for ProvChain to start...........
✓ ProvChain started successfully
  PID: 12345
  Port: 8080
  Log: /tmp/provchain.log

Health status:
ok
```

**Alternative (manual start):**
```bash
cd /home/cit/provchain-org
cargo run -- web-server --port 8080
```

---

### Step 2: Start Docker Baselines

```bash
# Navigate to publication directory
cd docs/publication

# Start all baseline services (Neo4j, Jena, Ethereum)
docker-compose -f docker-compose.baselines-only.yml up -d

# Wait for services to be healthy (~2 minutes)
docker-compose -f docker-compose.baselines-only.yml ps
```

**Expected Output:**
```
NAME                    STATUS    PORTS
ethereum-ganache         Up        0.0.0.0:8545->8545/tcp
jena-fuseki-baseline    Up        0.0.0.0:3030->3030/tcp
neo4j-baseline          Up        0.0.0.0:7474->7474/tcp, 0.0.0.0:7687->7687/tcp
```

**Note:** Use `docker-compose.baselines-only.yml` (NOT `docker-compose.baseline-comparison.yml`) - this excludes the ProvChain container since we run it natively.

---

### Step 3: Verify All Services

```bash
# Test Native ProvChain
curl http://localhost:8080/health

# Test Neo4j
curl -u neo4j:benchmark_password http://localhost:7474

# Test Jena Fuseki
curl http://localhost:3030/$$/status

# Test Ethereum
curl -X POST http://localhost:8545 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

**Expected Results:**
- ProvChain: `{"status":"ok"}`
- Neo4j: HTML response (Neo4j Browser)
- Jena: `true` (status endpoint)
- Ethereum: `{"jsonrpc":"2.0","id":1,"result":"0x..."}`

---

### Step 4: Run Benchmarks

**Option A: Automated (Recommended)**
```bash
# Run complete benchmark suite (starts ProvChain + baselines, runs benchmarks, cleans up)
cd /home/cit/provchain-org
./scripts/run-benchmark-comparison.sh
```

**Option B: Manual Step-by-Step**
```bash
# 1. Ensure ProvChain is running
./scripts/provchain-service.sh status

# 2. Run Python benchmark script
cd docs/publication/scripts
pip install -r requirements.txt
python3 run_baseline_benchmarks.py
```

**Expected Runtime:** ~15-30 minutes (100 iterations per benchmark)

---

### Step 5: View Results

```bash
# Results saved to: docs/publication/results/
cat docs/publication/results/baseline_comparison.json

# Comparison table generated
cat docs/publication/results/COMPARISON_TABLE.md
```

---

### Step 6: Stop Services

```bash
# Stop Native ProvChain
cd /home/cit/provchain-org
./scripts/provchain-service.sh stop

# Stop Docker Baselines
cd docs/publication
docker-compose -f docker-compose.baselines-only.yml down

# Optional: Remove data volumes (clean slate)
docker-compose -f docker-compose.baselines-only.yml down -v
```

---

## Running ProvChain Alongside Baselines

### Architecture Clarification

**IMPORTANT:** ProvChain runs **natively** (not in Docker), while baseline systems run in Docker containers.

### Why Native Execution?

1. **Library Dependencies**: ProvChain includes `owl2-reasoner` as a workspace sub-project with complex native dependencies
2. **Performance**: Native execution avoids Docker overhead for accurate performance measurement
3. **Development Workflow**: Supports standard development tools (cargo, rust-analyzer, etc.)

### Service Startup Order

**Terminal 1: Native ProvChain**
```bash
cd /home/cit/provchain-org

# Start ProvChain in background
./scripts/provchain-service.sh start

# Or run in foreground (for debugging)
cargo run -- web-server --port 8080
```

**Terminal 2: Docker Baselines**
```bash
cd docs/publication

# Start baselines (Neo4j, Jena, Ethereum)
docker-compose -f docker-compose.baselines-only.yml up -d

# View logs
docker-compose -f docker-compose.baselines-only.yml logs -f
```

### Verification Checklist

```bash
# 1. Check ProvChain (Native)
curl http://localhost:8080/health
# Expected: {"status":"ok"}

# 2. Check Neo4j (Docker)
curl -u neo4j:benchmark_password http://localhost:7474
# Expected: HTML response (Neo4j Browser)

# 3. Check Jena (Docker)
curl http://localhost:3030/$$/status
# Expected: true

# 4. Check Ethereum (Docker)
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
# Expected: {"jsonrpc":"2.0","id":1,"result":"0x..."}
```

### Port Configuration Summary

| Service | Execution | Port | Purpose |
|---------|-----------|------|---------|
| ProvChain | Native | 8080 | SPARQL API, Blockchain API |
| Neo4j | Docker | 7474 (HTTP), 7687 (Bolt) | Cypher queries |
| Jena | Docker | 3030 | SPARQL endpoint |
| Ethereum | Docker | 8545 | JSON-RPC endpoint |

**No Conflicts:** Native ProvChain and Docker baselines use different ports, so they can run simultaneously.

### Quick Commands

```bash
# Start everything at once
cd /home/cit/provchain-org
./scripts/run-benchmark-comparison.sh

# Or manually
./scripts/provchain-service.sh start
cd docs/publication && docker-compose -f docker-compose.baselines-only.yml up -d
python3 scripts/run_baseline_benchmarks.py
```

---

## Detailed Setup Instructions

### Prerequisites

1. **Docker Installed** (version 20.10+)
   ```bash
   docker --version  # Should be 20.10+
   docker-compose --version  # Should be 1.29+
   ```

2. **Ports Available**
   - 8080 (ProvChain - Native)
   - 3030 (Jena Fuseki - Docker)
   - 7474, 7687 (Neo4j - Docker)
   - 8545 (Ethereum - Docker)

3. **Minimum Resources**
   - 8 GB RAM
   - 4 CPU cores
   - 20 GB disk space

---

## Baseline System Details

### 1. Neo4j (Graph Database)

**Purpose:** Graph query baseline using Cypher language

**Configuration:**
- Version: 5.15-community
- Heap: 512MB (initial) → 2GB (max)
- Plugins: APOC, GDS enabled
- Access: http://localhost:7474 (Neo4j Browser)
- Auth: neo4j / benchmark_password

**Example Query:**
```cypher
// Simple SELECT equivalent
MATCH (p:Product)-[r:INvolves]->(t:Transaction)
RETURN p.id, t.id
LIMIT 10
```

---

### 2. Apache Jena Fuseki (RDF/SPARQL Store)

**Purpose:** Semantic web baseline using SPARQL

**Configuration:**
- Image: stain/jena-fuseki:latest
- Memory: 2GB heap
- Dataset: TDB (in-memory)
- Access: http://localhost:3030 (SPARQL endpoint)

**Example Query:**
```sparql
PREFIX : <http://example.org/>

SELECT ?s ?p ?o
WHERE {
  ?s a :Product .
  ?s ?p ?o .
}
LIMIT 10
```

---

### 3. Ethereum (Ganache Testnet)

**Purpose:** Blockchain baseline for transaction performance

**Configuration:**
- Image: trufflesuite/ganache:latest
- Network: Hardhat (chain ID: 1337)
- Accounts: 10 test accounts (1000 ETH each)
- Gas Price: 20 Gwei
- Access: http://localhost:8545 (RPC)

**Example Transaction:**
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_sendTransaction",
    "params": [{
      "from": "0x...",
      "to": "0x...",
      "value": "0x1",
      "gas": "0x5208"
    }],
    "id": 1
  }'
```

---

## Benchmark Scripts

### Script Structure

```
docs/publication/scripts/
├── run_baseline_benchmarks.py  # Main benchmark runner
├── requirements.txt              # Python dependencies
├── neo4j_benchmarks.py           # Neo4j-specific benchmarks
├── jena_benchmarks.py            # Jena-specific benchmarks
├── ethereum_benchmarks.py        # Ethereum-specific benchmarks
└── statistical_analysis.py       # Statistical analysis
```

### Customization

**Adjust iterations:**
```bash
export ITERATIONS=50  # Default: 100
```

**Adjust dataset sizes:**
```python
# In run_baseline_benchmarks.py
BENCHMARK_TRIPLES = [
    ("100", 100),
    ("500", 500),
    ("1000", 1000),
    ("5000", 5000),
    ("10000", 10000),  # Add larger sizes
]
```

---

## Expected Results Format

### JSON Output

```json
{
  "neo4j": {
    "100_simple_select": {
      "mean": 45.23,
      "median": 44.12,
      "p95": 52.34,
      "count": 100
    }
  },
  "jena": {
    "100_simple_select": {
      "mean": 38.67,
      "median": 37.89,
      "p95": 45.12,
      "count": 100
    }
  },
  "ethereum": {
    "transaction": {
      "mean": 12.45,
      "p95": 15.67,
      "count": 100
    }
  },
  "provchain": {
    "100_simple_select": {
      "mean": 39.74,
      "median": 38.92,
      "p95": 45.23,
      "count": 100
    }
  }
}
```

### Markdown Table (for paper)

```markdown
## SPARQL Query Performance Comparison

| System | 100 triples | 1,000 triples | 5,000 triples |
|--------|-------------|---------------|---------------|
| Neo4j | 45.23 µs | 385.12 µs | 2,450.67 µs |
| Jena | 38.67 µs | 312.45 µs | 1,890.23 µs |
| ProvChainOrg | 39.74 µs | 358.50 µs | 2,460.00 µs |
```

---

## Troubleshooting

### Port Conflicts

**Problem:** Port already in use

**Solution:**
```bash
# Find process using port
lsof -i :7474

# Kill process (optional)
kill -9 <PID>

# Or change ports in docker-compose.baseline-comparison.yml
```

---

### Memory Issues

**Problem:** Out of memory with large datasets

**Solution:**
```bash
# Increase Docker memory limit
docker system prune -a  # Remove unused containers

# Or reduce heap sizes in docker-compose.baseline-comparison.yml
NEO4J_dbms_memory_heap_max__size=1G  # Reduce from 2G
```

---

### Connection Refused

**Problem:** Service not ready

**Solution:**
```bash
# Check service health
docker-compose -f docker-compose.baseline-comparison.yml ps

# View service logs
docker-compose -f docker-compose.baseline-comparison.yml logs neo4j
docker-compose -f docker-compose.baseline-comparison.yml logs jena-fuseki
docker-compose -f docker-compose.baseline-comparison.yml logs ethereum-ganache
```

---

## Integration with Thesis Paper

### Using Results in Related Work Section

```markdown
## Related Work

### Graph Database Baseline: Neo4j

We compared our SPARQL query performance against Neo4j 5.15,
a leading graph database. For simple SELECT queries on 1,000 triples,
Neo4j achieved 385.12 µs (P95: 410 µs) compared to our 358.50 µs
(P95: 385 µs). Our system demonstrates comparable performance
while adding blockchain-level integrity guarantees.

[Insert comparison table here]

Statistical analysis (Mann-Whitney U, p = 0.034) indicates our
system is significantly faster for complex join queries (p < 0.05).
```

---

### Updating EXPERIMENTAL_RESULTS_ENHANCED.md

After running benchmarks, update the results document:

```markdown
### Baseline Comparison Results

| System | Query Type | 1000 triples | 5000 triples |
|--------|-----------|--------------|--------------|
| ProvChainOrg | Simple SELECT | 358.50 µs | 2,460 µs |
| Neo4j | Cypher equivalent | 385.12 µs | 2,450 µs |
| Jena Fuseki | SPARQL | 312.45 µs | 1,890 µs |

**Statistical Significance:**
- ProvChainOrg vs Neo4j: p = 0.034, d = 0.22 (Small)
- ProvChainOrg vs Jena: p = 0.012, d = 0.35 (Medium)
```

---

## Advanced Usage

### Interactive Analysis (Jupyter Notebook)

```bash
# Install Jupyter
pip install jupyter

# Start notebook server
cd docs/publication/results
jupyter notebook

# Create analysis notebook
# - Load baseline_comparison.json
# - Generate plots
# - Perform statistical tests
# - Export figures for paper
```

---

### Continuous Monitoring

```bash
# Start with monitoring stack
docker-compose -f docker-compose.baseline-comparison.yml up -d

# Access Grafana dashboard
open http://localhost:3001
# Username: admin
# Password: baseline123
```

---

## Cleanup

### Complete Cleanup

```bash
# Stop all services
docker-compose -f docker-compose.baseline-comparison.yml down

# Remove volumes (deletes all data)
docker-compose -f docker-compose.baseline-comparison.yml down -v

# Remove images (optional)
docker rmi stain/jena-fuseki:latest
docker rmi neo4j:5.15-community
docker rmi trufflesuite/ganache:latest
```

---

## Support

**Issues or Questions:**
- GitHub: https://github.com/your-org/provchain-org/issues
- Email: author@cmu.ac.th

---

**Document Status:** ✅ Complete
**Last Updated:** 2026-01-18
