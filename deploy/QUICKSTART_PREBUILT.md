# Quick Start: Pre-Built Docker Images

**Get ProvChain-Org running in 3 commands**

---

## Method 1: Single Container (Fastest)

```bash
# 1. Pull the image
docker pull anusornc/provchain-org:latest

# 2. Run the container
docker run -d \
  --name provchain-org \
  -p 8080:8080 \
  -p 9090:9090 \
  -e JWT_SECRET=$(openssl rand -base64 32) \
  anusornc/provchain-org:latest

# 3. Verify it's running
curl http://localhost:8080/health
```

**Expected response:** `{"status":"healthy",...}`

**Done!** 🎉 Your node is running.

---

## Method 2: Docker Compose (Recommended)

### Step 1: Create docker-compose.yml

```bash
cat > docker-compose.yml <<'EOF'
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
      - JWT_SECRET=${JWT_SECRET}
    volumes:
      - provchain_data:/app/data
      - provchain_logs:/app/logs

volumes:
  provchain_data:
  provchain_logs:
EOF
```

### Step 2: Create .env file

```bash
echo "JWT_SECRET=$(openssl rand -base64 32)" > .env
```

### Step 3: Start

```bash
docker compose up -d
```

### Verify

```bash
curl http://localhost:8080/health
```

---

## Method 3: 3-Node Cluster (One Machine)

```bash
cat > docker-compose.yml <<'EOF'
version: '3.8'

services:
  node1:
    image: anusornc/provchain-org:latest
    container_name: provchain-node1
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      - JWT_SECRET=${JWT_SECRET}
      - PROVCHAIN_PEERS=
      - OTEL_SERVICE_NAME=provchain-node1
    volumes:
      - node1_data:/app/data

  node2:
    image: anusornc/provchain-org:latest
    container_name: provchain-node2
    restart: unless-stopped
    ports:
      - "8081:8080"
    environment:
      - JWT_SECRET=${JWT_SECRET}
      - PROVCHAIN_PEERS=node1:8080
      - OTEL_SERVICE_NAME=provchain-node2
    volumes:
      - node2_data:/app/data

  node3:
    image: anusornc/provchain-org:latest
    container_name: provchain-node3
    restart: unless-stopped
    ports:
      - "8082:8080"
    environment:
      - JWT_SECRET=${JWT_SECRET}
      - PROVCHAIN_PEERS=node1:8080,node2:8080
      - OTEL_SERVICE_NAME=provchain-node3
    volumes:
      - node3_data:/app/data

volumes:
  node1_data:
  node2_data:
  node3_data:
EOF

echo "JWT_SECRET=$(openssl rand -base64 32)" > .env
docker compose up -d
```

---

## Access Your Services

| Service | URL | Description |
|---------|-----|-------------|
| **API** | http://localhost:8080 | Main application API |
| **Health** | http://localhost:8080/health | Health check endpoint |
| **Metrics** | http://localhost:9090/metrics | Prometheus metrics |

---

## Common Commands

### View logs

```bash
docker logs -f provchain-org
```

### Stop container

```bash
docker stop provchain-org
```

### Start container

```bash
docker start provchain-org
```

### Remove container

```bash
docker rm -f provchain-org
```

### Update to latest

```bash
docker pull anusornc/provchain-org:latest
docker compose up -d
```

---

## Available Image Tags

| Tag | When to Use |
|-----|-------------|
| `anusornc/provchain-org:latest` | Most recent stable release |
| `anusornc/provchain-org:v1.0.0` | Specific version (v1.0.0) |

**Example:**
```bash
docker pull anusornc/provchain-org:v1.0.0
docker run -d --name provchain-org -p 8080:8080 anusornc/provchain-org:v1.0.0
```

---

## Troubleshooting

### Port already in use

```bash
# Check what's using port 8080
sudo lsof -i :8080

# Kill the process or use a different port
docker run -d --name provchain-org -p 8081:8080 ...
```

### Container won't start

```bash
# Check logs
docker logs provchain-org

# Common issue: Missing JWT_SECRET
# Make sure to include: -e JWT_SECRET=your-secret-here
```

### Image not found

```bash
# Make sure you pulled the image first
docker pull anusornc/provchain-org:latest
```

---

## Next Steps

- Read the full guide: [README_MULTI_NODE.md](README_MULTI_NODE.md)
- Deploy a [multi-node cluster](README_MULTI_NODE.md)
- Submit your first transaction
- Set up monitoring (Prometheus/Grafana)

## Build from Source

If you prefer to build from source or need to make modifications:

```bash
# Clone repository
git clone https://github.com/anusornc/provchain-org2.git
cd provchain-org/deploy

# Use the default docker-compose files (they build from source)
docker compose -f docker-compose.node.yml up -d --build
```

---

**Need help?**
- Repository: https://github.com/anusornc/provchain-org2
- Issues: https://github.com/anusornc/provchain-org2/issues
- Docker Hub: https://hub.docker.com/r/anusornc/provchain-org
