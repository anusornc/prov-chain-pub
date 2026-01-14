# Quick Start Guide: Single-Node Deployment

**Get ProvChain-Org running in 10 minutes**

This guide helps you deploy a **single-node instance** of ProvChain-Org with monitoring for development, testing, or small-scale production use.

---

## Prerequisites

**Requirements:**
- Docker 20.10+ and Docker Compose v2.0+
- 4 GB RAM minimum (8 GB recommended)
- 20 GB disk space
- Ports 8080, 9090, 3001, 9091, 16686 available

**Check Docker:**
```bash
docker --version
docker compose version
```

**Install Docker (if needed):**
```bash
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER
newgrp docker
```

---

## Quick Start (10 Minutes)

### Step 1: Clone Repository

```bash
git clone https://github.com/anusornc/provchain-org2.git
cd provchain-org/deploy
```

### Step 2: Configure Environment

```bash
# Generate JWT secret
export JWT_SECRET=$(openssl rand -base64 32)

# Optional: Set Grafana credentials
export GRAFANA_USER=admin
export GRAFANA_PASSWORD=your_secure_password
```

### Step 3: Start Services

```bash
# Start the complete stack (application + monitoring)
docker compose -f docker-compose.production.yml up -d --build
```

**Wait 2-3 minutes for all services to start.**

### Step 4: Verify Deployment

```bash
# Check all containers are running
docker ps

# Expected output:
# CONTAINER   IMAGE                     STATUS    PORTS
# provchain   provchain-org:latest      Up        0.0.0.0:8080->8080, 0.0.0.0:9090->9090
# prometheus  prom/prometheus:v2.45.0   Up        0.0.0.0:9091->9090
# grafana     grafana/grafana:10.0.0    Up        0.0.0.0:3001->3000
# jaeger      jaegertracing/all-in-one  Up        0.0.0.0:16686->16686
# nginx       nginx:1.25-alpine         Up        0.0.0.0:80->80, 0.0.0.0:443->443
# redis       redis:7.0-alpine          Up        0.0.0.0:6379->6379
```

```bash
# Check application health
curl http://localhost:8080/health

# Expected response:
# {"status":"healthy","version":"0.1.0","timestamp":"..."}
```

---

## Access Your Services

| Service | URL | Credentials |
|---------|-----|-------------|
| **Application API** | http://localhost:8080 | JWT token required |
| **Metrics** | http://localhost:9090/metrics | None |
| **Grafana** | http://localhost:3001 | admin / your_password |
| **Prometheus** | http://localhost:9091 | None |
| **Jaeger** | http://localhost:16686 | None |

### Grafana Setup

1. Open http://localhost:3001
2. Login with `admin / your_password`
3. Add Prometheus datasource:
   - URL: `http://prometheus:9090`
   - Access: Server (default)

---

## Common Operations

### View Logs

```bash
# Application logs
docker logs -f provchain-org

# All logs
docker compose -f docker-compose.production.yml logs -f
```

### Submit Test Transaction

```bash
# Generate JWT token (replace with your actual secret)
JWT_TOKEN=$(echo -n '{"sub":"test"}' | openssl enc -base64 | tr -d '=')

# Submit transaction
curl -X POST http://localhost:8080/api/transactions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{
    "data": "<http://example.org/product> <http://example.org/hasName> \"Test Product\" ."
  }'

# Expected response:
# {"status":"accepted","tx_id":"..."}
```

### Check Blockchain

```bash
# View blockchain
curl http://localhost:8080/api/blockchain/dump | jq '.'

# Check blockchain length
curl http://localhost:8080/api/blockchain/dump | jq 'length'

# Validate blockchain
curl http://localhost:8080/api/blockchain/validate | jq '.'
```

### Performance Testing

After deployment, validate performance using the benchmark toolkit:

```bash
# Navigate to benchmark toolkit
cd ../benchmark-toolkit

# Quick performance check (5 minutes)
./run.sh low

# Full benchmark suite (15 minutes)
./run.sh medium
```

**Access Results**:
- **Grafana Dashboard**: http://localhost:3000 (admin/admin)
- **Summary Report**: `benchmark-toolkit/results/summary.md`
- **Raw Data**: `benchmark-toolkit/results/benchmark_results.csv`

**What Gets Tested**:
- Query performance (SPARQL latency)
- Write throughput (transactions/sec)
- Permission control overhead
- Multi-hop traceability

**For detailed benchmarking guide**, see [BENCHMARKING.md](../BENCHMARKING.md) or [docs/benchmarking/README.md](../docs/benchmarking/README.md)

### Stop Services

```bash
# Stop all services
docker compose -f docker-compose.production.yml down

# Stop and remove volumes (deletes data)
docker compose -f docker-compose.production.yml down -v
```

### Restart Services

```bash
# Restart all services
docker compose -f docker-compose.production.yml restart

# Restart specific service
docker compose -f docker-compose.production.yml restart provchain-org
```

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      Single-Node All-in-One Deployment                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│    User/Client                                                               │
│         │                                                                    │
│         ▼                                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                         Nginx (Reverse Proxy)                       │    │
│  │                         :80/:443 → :8080                            │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                 │                                           │
│         ┌───────────────────────┼───────────────────────┐                 │
│         ▼                       ▼                       ▼                 │
│  ┌─────────────┐        ┌──────────────┐      ┌─────────────┐            │
│  │ProvChain App│        │    Redis     │      │   Backup    │            │
│  │   :8080     │        │    :6379     │      │   Service   │            │
│  │   :9090     │        │   (Cache)    │      │  (Scheduled)│            │
│  └──────┬──────┘        └──────────────┘      └─────────────┘            │
│         │                                                                  │
│         ├──────────┬────────────┬────────────┬──────────┐                  │
│         ▼          ▼            ▼            ▼          ▼                  │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐             │
│  │Prometheus│ │Grafana  │ │ Jaeger  │ │  Loki   │ │Promtail │             │
│  │  :9091   │ │ :3001   │ │ :16686  │ │ :3100   │ │ (logs)  │             │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘             │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Services:**
- **provchain-org:** Main application (8080: API, 9090: metrics)
- **nginx:** Reverse proxy (80: HTTP, 443: HTTPS)
- **redis:** Caching layer (6379)
- **prometheus:** Metrics collection (9091)
- **grafana:** Metrics visualization (3001)
- **jaeger:** Distributed tracing (16686)
- **loki:** Log aggregation (3100)
- **promtail:** Log shipping
- **backup:** Scheduled backups

---

## Configuration

### Environment Variables

Edit `deploy/.env` or set via command line:

```bash
# Required
JWT_SECRET=your-secure-secret-here

# Optional
RUST_LOG=info                    # debug, info, warn, error
GRAFANA_USER=admin
GRAFANA_PASSWORD=admin
REDIS_PASSWORD=provchain123
```

### Custom Configuration

Create a custom TOML file:

```bash
# Create config/custom.toml
cat > config/custom.toml <<EOF
node_id = "custom-node"

[network]
network_id = "provchain-custom"
listen_port = 8080

[storage]
cache_size_mb = 200
EOF

# Use custom config
export CONFIG_FILE=/app/config/custom.toml
docker compose -f docker-compose.production.yml up -d
```

---

## Troubleshooting

### Container won't start

```bash
# Check logs
docker logs provchain-org

# Check port conflicts
sudo lsof -i :8080

# Check disk space
df -h

# Clean restart
docker compose -f docker-compose.production.yml down -v
docker compose -f docker-compose.production.yml up -d
```

### Out of memory

```bash
# Add memory limit in docker-compose.production.yml
services:
  provchain-org:
    mem_limit: 2g

# Or reduce cache in config
[storage]
cache_size_mb = 50
```

### Cannot access services

```bash
# Check firewall
sudo ufw status

# Open ports if needed
sudo ufw allow 8080/tcp
sudo ufw allow 9090/tcp

# Check container status
docker ps
```

---

## Development Tips

### Hot Reload (Not Recommended for Production)

```bash
# Mount source directory for development
# Add to docker-compose.production.yml:
volumes:
  - ./src:/app/src:ro

# Rebuild on changes
docker compose -f docker-compose.production.yml up -d --build
```

### Debug Mode

```bash
# Enable debug logging
export RUST_LOG=debug

# Restart with debug logs
docker compose -f docker-compose.production.yml restart provchain-org

# View debug logs
docker logs -f provchain-org
```

### Access Container Shell

```bash
# Access running container
docker exec -it provchain-org /bin/bash

# Run commands inside container
docker exec provchain-org provchain-org --version
```

---

## Production Considerations

For production use, consider:

1. **Security:**
   - Use strong, unique JWT_SECRET
   - Enable TLS/SSL with nginx
   - Restrict firewall rules
   - Use secrets manager (Vault, AWS Secrets)

2. **Persistence:**
   - Use named volumes for data
   - Enable automated backups
   - Monitor disk usage

3. **Monitoring:**
   - Set up Grafana alerts
   - Configure log aggregation
   - Enable distributed tracing

4. **Scaling:**
   - Consider multi-node deployment
   - Use load balancer (HAProxy, NGINX)
   - Deploy on separate VMs

**For production deployment, see:**
- **Docker Architecture:** `/docs/deployment/DOCKER_DEPLOYMENT_ARCHITECTURE.md`
- **Complete Setup:** `/docs/deployment/SETUP_INSTALLATION_GUIDE.md`
- **Multi-Node:** `/deploy/README_MULTI_NODE.md`

---

## Next Steps

- [ ] Submit your first transaction
- [ ] Explore the Grafana dashboards
- [ ] Check Prometheus metrics
- [ ] Review the API documentation
- [ ] Try a multi-node deployment

---

## Support & Documentation

- **Full Documentation:** `/docs/deployment/`
- **Architecture:** `/docs/deployment/DOCKER_DEPLOYMENT_ARCHITECTURE.md`
- **Issues:** https://github.com/anusornc/provchain-org2/issues

---

`★ Insight ─────────────────────────────────────`
**Single-Node vs Multi-Node**
1. Single-node is perfect for development and testing—fast to start, minimal resources
2. Multi-node (3+) is required for production consensus—provides fault tolerance and decentralization
3. The same Docker image works for both—just change the docker-compose file and configuration
`─────────────────────────────────────────────────`

**Done!** Your single-node ProvChain-Org instance is now running.
