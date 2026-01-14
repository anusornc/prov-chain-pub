# ProvChain-Org: Pre-Built Docker Image Deployment

**Deploy ProvChain-Org in 5 minutes using pre-built Docker images**

---

## Overview

Instead of building from source (which takes 10-15 minutes), you can use pre-built Docker images that are ready to run instantly.

---

## Quick Start (5 Minutes)

### Option 1: Using Docker Hub (Recommended)

```bash
# Pull the latest image
docker pull anusornc/provchain-org:latest

# Run the container
docker run -d \
  --name provchain-org \
  -p 8080:8080 \
  -p 9090:9090 \
  -e JWT_SECRET=$(openssl rand -base64 32) \
  anusornc/provchain-org:latest

# Check health
curl http://localhost:8080/health
```

**That's it!** Your node is running.

---

### Option 2: Using Docker Compose (Easier)

**Step 1: Create docker-compose.yml**

```bash
cat > docker-compose.yml <<EOF
version: '3.8'

services:
  provchain-org:
    image: anusornc/provchain-org:latest
    container_name: provchain-org
    restart: unless-stopped
    ports:
      - "8080:8080"
      - "9090:9090"
    environment:
      - RUST_LOG=info
      - JWT_SECRET=\${JWT_SECRET}
    volumes:
      - provchain_data:/app/data
      - provchain_logs:/app/logs

volumes:
  provchain_data:
    driver: local
  provchain_logs:
    driver: local
EOF
```

**Step 2: Create .env file**

```bash
echo "JWT_SECRET=$(openssl rand -base64 32)" > .env
```

**Step 3: Start the container**

```bash
docker compose up -d
```

---

## Available Images

### Image Tags

| Tag | Description | Use Case |
|-----|-------------|----------|
| `anusornc/provchain-org:latest` | Most recent stable release | Production deployments |
| `anusornc/provchain-org:v1.0.0` | Version 1.0.0 release | Version pinning |

### Pull Specific Version

```bash
# Pull specific version
docker pull anusornc/provchain-org:v1.0.0

# Always pull latest for production
docker pull anusornc/provchain-org:latest
```

---

## Complete Deployment Stack

### All-in-One Stack (With Monitoring)

```bash
cat > docker-compose.yml <<EOF
version: '3.8'

services:
  # Main application
  provchain-org:
    image: anusornc/provchain-org:latest
    container_name: provchain-org
    restart: unless-stopped
    ports:
      - "8080:8080"
      - "9090:9090"
    environment:
      - RUST_LOG=info
      - JWT_SECRET=\${JWT_SECRET}
    volumes:
      - provchain_data:/app/data
      - provchain_logs:/app/logs
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  # Prometheus for metrics
  prometheus:
    image: prom/prometheus:v2.45.0
    container_name: provchain-prometheus
    restart: unless-stopped
    ports:
      - "9091:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'

  # Grafana for dashboards
  grafana:
    image: grafana/grafana:10.0.0
    container_name: provchain-grafana
    restart: unless-stopped
    ports:
      - "3001:3000"
    environment:
      - GF_SECURITY_ADMIN_USER=\${GRAFANA_USER:-admin}
      - GF_SECURITY_ADMIN_PASSWORD=\${GRAFANA_PASSWORD:-admin}
    volumes:
      - grafana_data:/var/lib/grafana

volumes:
  provchain_data:
  provchain_logs:
  prometheus_data:
  grafana_data:
EOF
```

**Create Prometheus config:**

```bash
cat > prometheus.yml <<EOF
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'provchain'
    static_configs:
      - targets: ['provchain-org:9090']
EOF
```

**Create environment file:**

```bash
cat > .env <<EOF
JWT_SECRET=$(openssl rand -base64 32)
GRAFANA_USER=admin
GRAFANA_PASSWORD=admin
EOF
```

**Start everything:**

```bash
docker compose up -d
```

---

## Multi-Node Cluster Using Pre-Built Images

### 3-Node Cluster (Single Machine)

```bash
cat > docker-compose-3node.yml <<EOF
version: '3.8'

services:
  # Node 1 (Bootstrap)
  node1:
    image: anusornc/provchain-org:latest
    container_name: provchain-node1
    restart: unless-stopped
    ports:
      - "8080:8080"
      - "9090:9090"
    environment:
      - RUST_LOG=info
      - JWT_SECRET=\${JWT_SECRET}
      - PROVCHAIN_PEERS=
      - OTEL_SERVICE_NAME=provchain-node1
    volumes:
      - node1_data:/app/data

  # Node 2
  node2:
    image: anusornc/provchain-org:latest
    container_name: provchain-node2
    restart: unless-stopped
    ports:
      - "8081:8080"
    environment:
      - RUST_LOG=info
      - JWT_SECRET=\${JWT_SECRET}
      - PROVCHAIN_PEERS=node1:8080
      - OTEL_SERVICE_NAME=provchain-node2
    volumes:
      - node2_data:/app/data

  # Node 3
  node3:
    image: anusornc/provchain-org:latest
    container_name: provchain-node3
    restart: unless-stopped
    ports:
      - "8082:8080"
    environment:
      - RUST_LOG=info
      - JWT_SECRET=\${JWT_SECRET}
      - PROVCHAIN_PEERS=node1:8080,node2:8080
      - OTEL_SERVICE_NAME=provchain-node3
    volumes:
      - node3_data:/app/data

volumes:
  node1_data:
  node2_data:
  node3_data:
EOF
```

**Start cluster:**

```bash
echo "JWT_SECRET=$(openssl rand -base64 32)" > .env
docker compose -f docker-compose-3node.yml up -d
```

---

## Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `JWT_SECRET` | Yes | - | JWT authentication secret |
| `RUST_LOG` | No | info | Log level (debug, info, warn, error) |
| `PROVCHAIN_PEERS` | No* | - | Comma-separated peer list |
| `PROVCHAIN_PORT` | No | 8080 | P2P listen port |
| `OTEL_SERVICE_NAME` | No | provchain | Service name for tracing |

*Required for non-bootstrap nodes in multi-node setups

### Custom Configuration

You can provide a custom TOML configuration:

```bash
docker run -d \
  --name provchain-org \
  -p 8080:8080 \
  -p 9090:9090 \
  -v $(pwd)/custom-config.toml:/app/config/custom.toml:ro \
  -e CONFIG_FILE=/app/config/custom.toml \
  -e JWT_SECRET=your-secret-here \
  anusornc/provchain-org:latest
```

---

## Common Operations

### Update to Latest Version

```bash
# Pull latest image
docker pull anusornc/provchain-org:latest

# Stop and remove old container
docker compose down

# Start with new image
docker compose up -d
```

### View Logs

```bash
# Follow logs
docker logs -f provchain-org

# View last 100 lines
docker logs --tail 100 provchain-org
```

### Access Container Shell

```bash
docker exec -it provchain-org /bin/bash
```

### Backup Data

```bash
# Create backup
docker run --rm \
  -v provchain_data:/data \
  -v $(pwd):/backup \
  alpine tar czf /backup/provchain-backup-$(date +%Y%m%d).tar.gz /data
```

### Restore Data

```bash
# Restore from backup
docker run --rm \
  -v provchain_data:/data \
  -v $(pwd):/backup \
  alpine tar xzf /backup/provchain-backup-YYYYMMDD.tar.gz -C /
```

---

## Registry Options

### Docker Hub (Default)

```bash
docker pull anusornc/provchain-org:latest
```

### GitHub Container Registry (Alternative)

```bash
# Pull from GHCR (if published there)
# docker pull ghcr.io/anusornc/provchain-org:latest

# Use in docker-compose.yml
# image: ghcr.io/anusornc/provchain-org:latest
```

### Build Your Own

If you want to build from source:

```bash
# Clone repository
git clone https://github.com/anusornc/provchain-org2.git
cd provchain-org2/deploy

# Build image locally using the provided Dockerfile
docker build -f Dockerfile.production -t my-custom-tag ..

# Or use docker-compose to build
docker compose -f docker-compose.production.yml build
```

---

## Verification

### Health Check

```bash
curl http://localhost:8080/health
```

**Expected response:**
```json
{"status":"healthy","version":"1.0.0"}
```

### Check Blockchain

```bash
# View blockchain
curl http://localhost:8080/api/blockchain/dump | jq .

# Check blockchain length
curl http://localhost:8080/api/blockchain/dump | jq 'length'

# Validate blockchain
curl http://localhost:8080/api/blockchain/validate | jq .
```

### Test Multi-Node Cluster

```bash
# Check peer connections (from bootstrap node)
curl http://localhost:8080/api/peers | jq .

# Expected: Array of connected peers
```

---

## Troubleshooting

### Image Not Found

**Error:**
```
Error: image 'anusornc/provchain-org:latest' not found
```

**Solution:**
```bash
# Pull the image first
docker pull anusornc/provchain-org:latest

# Or build it yourself
cd provchain-org2/deploy
docker build -f Dockerfile.production -t latest ..
```

### Container Crashes Immediately

**Check logs:**
```bash
docker logs provchain-org
```

**Common issues:**
- Missing `JWT_SECRET` environment variable
- Port already in use
- Insufficient memory

### Cannot Access Application

**Check if container is running:**
```bash
docker ps | grep provchain
```

**Check health status:**
```bash
docker inspect provchain-org | jq '.[0].State.Health'
```

---

## Migration from Source Build

If you were previously building from source:

**Old way (10-15 minutes):**
```bash
docker compose -f docker-compose.production.yml up -d --build
```

**New way (30 seconds):**
```bash
docker pull anusornc/provchain-org:latest
docker compose up -d
```

Update your `docker-compose.yml`:
```yaml
services:
  provchain-org:
    image: anusornc/provchain-org:latest  # Changed from: build: ./
    # Remove: build: . and dockerfile: ...
```

---

## Performance Tips

### Use Docker BuildKit

```bash
export DOCKER_BUILDKIT=1
docker build -f deploy/Dockerfile.production -t my-image .
```

### Enable Multi-Platform Builds

```bash
# Build for multiple architectures
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -f deploy/Dockerfile.production \
  -t anusornc/provchain-org:latest \
  . --push
```

### Layer Caching

The pre-built images already have all dependencies cached, making pulls much faster than builds.

---

## Security Best Practices

### 1. Pin Specific Versions

```yaml
# Instead of :latest
image: anusornc/provchain-org:latest

# Use specific version
image: anusornc/provchain-org:v1.0.0
```

### 2. Scan Images

```bash
# Scan for vulnerabilities
docker scan anusornc/provchain-org:latest
```

### 3. Use Secrets Manager

```bash
# Instead of environment variables
echo "JWT_SECRET=secret" > .env

# Use Docker secrets (in Swarm mode)
echo "secret" | docker secret create jwt_secret -
```

### 4. Run as Non-Root

The pre-built images already run as non-root user by default.

---

## Support

- **Repository:** https://github.com/anusornc/provchain-org
- **Issues:** https://github.com/anusornc/provchain-org/issues
- **Docker Hub:** https://hub.docker.com/r/anusornc/provchain-org

---

`★ Insight ─────────────────────────────────────`
**Pre-Built Images vs Building from Source**
1. **Speed:** Pre-built images deploy in seconds vs 10-15 minutes for builds
2. **Reliability:** Images are tested before publishing, reducing runtime errors
3. **Consistency:** Everyone uses the same image, eliminating environment differences
4. **Updates:** Simple `docker pull` gets the latest version
`─────────────────────────────────────────────────`

---

**Done!** You're now using pre-built Docker images for faster, easier deployment.
