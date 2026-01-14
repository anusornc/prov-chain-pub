# 🚀 Quick Reference Card

## One-Command Setup (Any Machine)

```bash
tar -xzf provchain-benchmark-toolkit-*.tar.gz
cd provchain-benchmark-toolkit-*
./run.sh
```

## Access Points

| Service | URL | Login |
|---------|-----|-------|
| Grafana | http://localhost:3000 | admin/admin |
| ProvChain API | http://localhost:8080 | - |
| Neo4j | http://localhost:7474 | neo4j/benchmark |
| Prometheus | http://localhost:9091 | - |

## Hardware Profiles

```bash
./run.sh low     # 4GB RAM, 2 cores (minimal)
./run.sh medium  # 8GB RAM, 4 cores (recommended) ✅
./run.sh high    # 16GB RAM, 8 cores (extensive)
./run.sh ultra   # 32GB+ RAM, 16+ cores (full)
```

## Common Commands

```bash
# Stop services
docker-compose down

# Start services only
docker-compose up -d

# View logs
docker-compose logs -f provchain

# Clean everything
docker-compose down -v
rm -rf results/* logs/*

# Rebuild benchmark runner
docker-compose build benchmark-runner
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Docker not running | Start Docker Desktop/service |
| Port already in use | Edit docker-compose.yml ports |
| Out of memory | Use lower profile: `./run.sh low` |
| Services not starting | Check logs: `docker-compose logs` |

## Results Location

- **Summary**: `results/summary.md`
- **JSON Data**: `results/benchmark_results.json`
- **CSV**: `results/benchmark_results.csv`
- **Logs**: `logs/`

## Package for Distribution

```bash
./package.sh
# Creates: dist/provchain-benchmark-toolkit-v1.0.0-YYYYMMDD.tar.gz
```

---

**Full Documentation**: See README.md
