# ProvChain-Org: Docker Deployment Architecture

**Document Version:** 1.0
**Date:** 2026-01-03
**Status:** Comprehensive Docker Architecture Reference
**Target Audience:** DevOps Engineers, System Administrators, Developers

---

## Table of Contents

1. [Overview](#overview)
2. [Dockerfile Architecture](#dockerfile-architecture)
3. [Deployment Strategies](#deployment-strategies)
4. [Service Architecture](#service-architecture)
5. [Monitoring Stack](#monitoring-stack)
6. [Networking Model](#networking-model)
7. [Storage & Volumes](#storage--volumes)
8. [Configuration Management](#configuration-management)
9. [Security Considerations](#security-considerations)
10. [Performance Optimization](#performance-optimization)
11. [Troubleshooting Guide](#troubleshooting-guide)
12. [Quick Reference](#quick-reference)

---

## Overview

ProvChain-Org provides **three Docker deployment strategies** designed for different use cases:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    Docker Deployment Strategies                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐ │
│  │  Single-Node        │  │  3-Node Cluster     │  │  Multi-Machine      │ │
│  │  All-in-One         │  │  (Single Machine)   │  │  Distributed        │ │
│  │                     │  │                     │  │                     │ │
│  │  • Development      │  │  • Consensus        │  │  • Production       │ │
│  │  • Testing          │  │    Testing          │  │  • Cloud VMs        │ │
│  │  • Small Production │  │  • Staging          │  │  • High Availability│ │
│  │                     │  │                     │  │                     │ │
│  │  docker-compose.    │  │  docker-compose.    │  │  docker-compose.    │ │
│  │  production.yml     │  │  3node.yml          │  │  node.yml           │ │
│  └─────────────────────┘  └─────────────────────┘  └─────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Deployment Comparison

| Strategy | Complexity | Cost | Use Case | Setup Time |
|----------|-----------|------|----------|------------|
| **Single-Node** | ★☆☆ | Low | Dev, testing, small production | 10-15 min |
| **3-Node (1 host)** | ★★☆ | Medium | Consensus testing, staging | 20-30 min |
| **Multi-Machine** | ★★★ | High | Production, HA | 2-4 hours |

---

## Dockerfile Architecture

### Multi-Stage Build

The `deploy/Dockerfile.production` uses a **three-stage build** to optimize image size and security:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Multi-Stage Docker Build                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐       │
│  │   Stage 1:      │     │   Stage 2:      │     │   Stage 3:      │       │
│  │   Backend       │────▶│   Frontend      │────▶│   Runtime       │       │
│  │   Builder       │     │   Builder       │     │   (Final)       │       │
│  │                 │     │                 │     │                 │       │
│  │  rust:1.75-slim │     │  node:18-alpine │     │  debian:bookworm│       │
│  │  ~2.5 GB        │     │  ~250 MB        │     │  ~150 MB        │       │
│  │                 │     │                 │     │                 │       │
│  │  • cargo build  │     │  • npm ci       │     │  • Binary only  │       │
│  │  • deps cache   │     │  • npm run build│     │  • No build tools│       │
│  │  • target/      │     │  • dist/        │     │  • Minimal OS   │       │
│  └─────────────────┘     └─────────────────┘     └─────────────────┘       │
│                                                                              │
│  Intermediate artifacts discarded → Final image: ~150 MB                    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Stage 1: Backend Builder

```dockerfile
FROM rust:1.75-slim as backend-builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libclang-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Layer 1: Cargo files (for dependency caching)
COPY Cargo.toml Cargo.lock ./
COPY owl2-reasoner/ ./owl2-reasoner/

# Create dummy main.rs to build dependencies first
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --bin provchain-org
RUN rm -rf src

# Layer 2: Source code (rebuilds only when code changes)
COPY src/ ./src/
COPY ontologies/ ./ontologies/
COPY config/ ./config/
COPY queries/ ./queries/
COPY shapes/ ./shapes/

RUN cargo build --release --bin provchain-org
```

**Key Optimizations:**
- **Dependency caching:** Build dependencies separately before adding source code
- **Layer ordering:** Frequently changed files (src/) come after rarely changed files (Cargo.toml)
- **Size reduction:** Intermediate artifacts discarded after final binary is built

### Stage 2: Frontend Builder

```dockerfile
FROM node:18-alpine as frontend-builder

WORKDIR /app/frontend

# Layer 1: Package files (for dependency caching)
COPY frontend/package*.json ./
RUN npm ci --only=production

# Layer 2: Source code
COPY frontend/ ./
RUN npm run build
```

### Stage 3: Production Runtime

```dockerfile
FROM debian:bookworm-slim

# Runtime dependencies only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false -m -d /app provchain

WORKDIR /app

# Copy compiled binary from Stage 1
COPY --from=backend-builder /app/target/release/provchain-org /usr/local/bin/provchain-org

# Copy configurations
COPY --from=backend-builder /app/config/ ./config/
COPY --from=backend-builder /app/ontologies/ ./ontologies/
COPY --from=backend-builder /app/queries/ ./queries/
COPY --from=backend-builder /app/shapes/ ./shapes/

# Copy built frontend from Stage 2
COPY --from=frontend-builder /app/frontend/dist/ ./static/

# Create directories with proper permissions
RUN mkdir -p /app/data /app/backups /app/logs \
    && chown -R provchain:provchain /app

# Non-root user for security
USER provchain

EXPOSE 8080 9090

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

ENV RUST_LOG=info
ENV JWT_SECRET=""
ENV CONFIG_FILE=/app/config/production-deployment.toml

CMD ["provchain-org", "--config", "/app/config/production-deployment.toml"]
```

---

## Deployment Strategies

### Strategy 1: Single-Node All-in-One

**File:** `deploy/docker-compose.production.yml`

Best for: Development, testing, small production deployments

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                   Single-Node All-in-One Deployment                         │
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
│         ▼                       │                       ▼                 │
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

**Services (10 containers):**
- provchain-org (8080, 9090)
- nginx (80, 443)
- redis (6379)
- prometheus (9091)
- grafana (3001)
- jaeger (16686, 14268)
- loki (3100)
- promtail
- backup

**Quick Start:**
```bash
cd /path/to/provchain-org/deploy

# Set environment variables
export JWT_SECRET=$(openssl rand -base64 32)
export GRAFANA_USER=admin
export GRAFANA_PASSWORD=secure_password

# Start all services
docker compose -f docker-compose.production.yml up -d

# Access services
# - Application: http://localhost:8080
# - Grafana: http://localhost:3001
# - Prometheus: http://localhost:9091
# - Jaeger: http://localhost:16686
```

### Strategy 2: 3-Node Cluster (Single Machine)

**File:** `deploy/docker-compose.3node.yml`

Best for: Testing consensus behavior, development clustering

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                   3-Node Cluster (Single Host)                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐│
│  │    Node 1       │◀────────▶│    Node 2       │◀────────▶│    Node 3       ││
│  │  (Bootstrap)    │         │                 │         │                 ││
│  │                 │         │                 │         │                 ││
│  │  • PEERS=""     │         │  • PEERS=       │         │  • PEERS=       ││
│  │  • Port: 8080   │         │    node1,node3  │         │    node1,node2  ││
│  │  • Auth: Yes    │         │  • Port: 8081   │         │  • Port: 8082   ││
│  │  • Data: node1  │         │  • Auth: No     │         │  • Auth: No     ││
│  └─────────┬───────┘         └─────────┬───────┘         └─────────┬───────┘│
│            │                          │                          │          │
│            └──────────────────────────┼──────────────────────────┘          │
│                                       │                                     │
│                            ┌──────────┴──────────┐                          │
│                            │                     │                          │
│                     ┌──────▼──────┐    ┌────────▼────────┐                 │
│                     │  Prometheus │    │     Grafana     │                 │
│                     │    :9091    │    │     :3001       │                 │
│                     └─────────────┘    └─────────────────┘                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Key Features:**
- **Peer Discovery:** Nodes discover each other via `PROVCHAIN_PEERS` environment variable
- **Individual Data:** Each node has isolated data volume
- **Metrics Tagging:** `OTEL_SERVICE_NAME` distinguishes metrics per node

**Quick Start:**
```bash
cd /path/to/provchain-org/deploy

# Start 3-node cluster
docker compose -f docker-compose.3node.yml up -d --build

# Verify cluster
docker ps | grep provchain-node

# Check peer connections
curl http://localhost:8080/api/peers | jq '.'
# Expected: 2 peers connected

# Check blockchain sync
curl http://localhost:8080/api/blockchain/dump | jq 'length'
curl http://localhost:8081/api/blockchain/dump | jq 'length'
curl http://localhost:8082/api/blockchain/dump | jq 'length'
# All should show same length
```

### Strategy 3: Multi-Machine Distributed Cluster

**Files:** `deploy/docker-compose.node.yml` + `deploy/docker-compose.monitoring.yml`

Best for: Production deployments across VMs or cloud infrastructure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                   Multi-Machine Distributed Deployment                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────┐    ┌─────────────────────────────┐         │
│  │   Machine 1 (192.168.1.101) │    │   Machine 2 (192.168.1.102) │         │
│  │   ┌─────────────────────┐   │    │   ┌─────────────────────┐   │         │
│  │   │   Node 1            │   │    │   │   Node 2            │   │         │
│  │   │   PEERS=""          │   │    │   │   PEERS="192.168... │   │         │
│  │   │   (Bootstrap)       │   │◀──▶│   │                     │   │         │
│  │   │   Port: 8080/9090   │   │    │   │   Port: 8080/9090   │   │         │
│  │   └─────────────────────┘   │    │   └─────────────────────┘   │         │
│  └─────────────────────────────┘    └─────────────────────────────┘         │
│                                                                              │
│                                      ┌─────────────────────────────┐         │
│                                      │   Machine 3 (192.168.1.103) │         │
│                                      │   ┌─────────────────────┐   │         │
│                                      │   │   Node 3            │   │         │
│                                      │   │   PEERS="192.168...│   │         │
│                                      │   │   Port: 8080/9090   │   │         │
│                                      │   └─────────────────────┘   │         │
│                                      └─────────────────────────────┘         │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │   Monitoring Machine (can be Machine 1 or separate)                  │    │
│  │   ┌──────────┐  ┌──────────┐  ┌──────────┐                         │    │
│  │   │Prometheus│  │ Grafana  │  │  Jaeger  │                         │    │
│  │   │  :9090   │  │  :3000   │  │  :16686  │                         │    │
│  │   └────┬─────┘  └────┬─────┘  └────┬─────┘                         │    │
│  │        └──────────────┴──────────────┘                              │    │
│  │               Scrape metrics from all nodes                          │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Configuration per node (`.env` file):**

```bash
# Machine 1 (Node 1 - Bootstrap)
PEERS=""
JWT_SECRET=your_secure_secret_here
RUST_LOG=info

# Machine 2 (Node 2)
PEERS="192.168.1.101:8080"
JWT_SECRET=your_secure_secret_here
RUST_LOG=info

# Machine 3 (Node 3)
PEERS="192.168.1.101:8080,192.168.1.102:8080"
JWT_SECRET=your_secure_secret_here
RUST_LOG=info
```

**Quick Start:**
```bash
# On Machine 1 (Bootstrap Node)
cd /path/to/provchain-org/deploy
cat > .env <<EOF
PEERS=""
JWT_SECRET=$(openssl rand -base64 32)
RUST_LOG=info
EOF

docker compose -f docker-compose.node.yml up -d --build

# On Machine 2
cat > .env <<EOF
PEERS="192.168.1.101:8080"
JWT_SECRET=$(openssl rand -base64 32)
RUST_LOG=info
EOF

docker compose -f docker-compose.node.yml up -d --build

# On Machine 3
cat > .env <<EOF
PEERS="192.168.1.101:8080,192.168.1.102:8080"
JWT_SECRET=$(openssl rand -base64 32)
RUST_LOG=info
EOF

docker compose -f docker-compose.node.yml up -d --build

# On Monitoring Machine
docker compose -f docker-compose.monitoring.yml up -d
```

---

## Service Architecture

### Core Services

#### ProvChain Application Container

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        ProvChain Application                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Port 8080: HTTP API + P2P WebSocket                                        │
│  ├─ GET  /health              (Health check)                                │
│  ├─ GET  /metrics             (Prometheus metrics)                          │
│  ├─ GET  /api/*               (REST API)                                    │
│  └─ WS   /                    (P2P communication)                           │
│                                                                              │
│  Port 9090: Metrics Export (Prometheus scraping)                            │
│                                                                              │
│  Environment Variables:                                                     │
│  ├─ RUST_LOG              (info, debug, warn, error)                        │
│  ├─ JWT_SECRET            (Authentication secret)                           │
│  ├─ CONFIG_FILE           (Path to TOML config)                             │
│  ├─ PROVCHAIN_PEERS       (Comma-separated peer list)                       │
│  ├─ PROVCHAIN_PORT        (P2P listen port)                                 │
│  └─ OTEL_SERVICE_NAME     (Service identifier for tracing)                  │
│                                                                              │
│  Volumes:                                                                    │
│  ├─ ./data:/app/data              (Blockchain storage)                      │
│  ├─ ./logs:/app/logs              (Application logs)                        │
│  └─ ./config:/app/config          (Configuration files)                     │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Nginx Reverse Proxy (Production Only)

```nginx
# /deploy/nginx/nginx.conf
events {
    worker_connections 1024;
}

http {
    upstream provchain_backend {
        server provchain-org:8080;
    }

    # HTTP → HTTPS redirect
    server {
        listen 80;
        server_name provchain.example.com;
        return 301 https://$server_name$request_uri;
    }

    # HTTPS with TLS termination
    server {
        listen 443 ssl http2;
        server_name provchain.example.com;

        ssl_certificate /etc/nginx/ssl/cert.pem;
        ssl_certificate_key /etc/nginx/ssl/key.pem;
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers HIGH:!aNULL:!MD5;

        # API endpoints
        location /api/ {
            proxy_pass http://provchain_backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # WebSocket (P2P)
        location / {
            proxy_pass http://provchain_backend;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
        }
    }
}
```

---

## Monitoring Stack

### Components

| Tool | Purpose | Port | Data Source |
|------|---------|------|-------------|
| **Prometheus** | Metrics collection | 9090/9091 | /metrics endpoint |
| **Grafana** | Metrics visualization | 3000/3001 | Prometheus |
| **Jaeger** | Distributed tracing | 16686 | OpenTelemetry |
| **Loki** | Log aggregation | 3100 | Promtail |
| **Promtail** | Log shipping | - | Container logs |

### Prometheus Configuration

```yaml
# /deploy/monitoring/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'provchain'
    static_configs:
      - targets:
          - 'node1:9090'
          - 'node2:9090'
          - 'node3:9090'
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
```

### Multi-Machine Prometheus Configuration

```yaml
# /deploy/monitoring/prometheus_multi_node.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'provchain-nodes'
    static_configs:
      # Replace with actual node IPs
      - targets:
          - 'NODE_1_IP:9090'
          - 'NODE_2_IP:9090'
          - 'NODE_3_IP:9090'
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
```

### Grafana Dashboards

**Import Dashboards:**
1. Open Grafana: http://localhost:3001
2. Navigate to **Dashboards** → **Import**
3. Use dashboard IDs:
   - **Node Exporter Full**: `1860`
   - **Docker Monitoring**: `893`
   - **Rust Applications**: Custom (see `/deploy/monitoring/grafana/dashboards/`)

---

## Networking Model

### Container Network

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Docker Network Model                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                     Bridge Network: provchain_network                │    │
│  │                     Subnet: 172.20.0.0/16                           │    │
│  │                                                                      │    │
│  │  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐          │    │
│  │  │  Container 1 │    │  Container 2 │    │  Container 3 │          │    │
│  │  │ 172.20.0.2   │◀──▶│ 172.20.0.3   │◀──▶│ 172.20.0.4   │          │    │
│  │  │ (provchain)  │    │  (nginx)     │    │  (prometheus)│          │    │
│  │  └──────────────┘    └──────────────┘    └──────────────┘          │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                        │
│                          ┌─────────┴─────────┐                              │
│                          │                   │                              │
│                   Port Mapping          Port Mapping                        │
│                   8080→8080             80→80                               │
│                   (container→host)      (container→host)                    │
│                          │                   │                              │
│                          ▼                   ▼                              │
│                    ┌────────────────────────────────┐                       │
│                    │      Host Network (eth0)       │                       │
│                    │      192.168.1.100             │                       │
│                    └────────────────────────────────┘                       │
│                                    │                                        │
│                                    ▼                                        │
│                    ┌────────────────────────────────┐                       │
│                    │         External               │                       │
│                    │         Internet               │                       │
│                    └────────────────────────────────┘                       │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Inter-Container Communication

Containers can communicate using service names:
- `provchain-org` → Application container
- `nginx` → Reverse proxy
- `prometheus` → Metrics collector
- `grafana` → Visualization

**Example:** Prometheus scrapes metrics from provchain-org using `http://provchain-org:9090/metrics`

### External Access

| Service | Internal Port | External Port | Access URL |
|---------|---------------|---------------|------------|
| Application | 8080 | 8080 | http://host:8080 |
| Metrics | 9090 | 9090 | http://host:9090 |
| Grafana | 3000 | 3001 | http://host:3001 |
| Prometheus | 9090 | 9091 | http://host:9091 |
| Jaeger | 16686 | 16686 | http://host:16686 |

---

## Storage & Volumes

### Volume Strategy

```yaml
volumes:
  # Application data (blockchain, RDF store)
  provchain_data:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /opt/provchain/data

  # Backups
  provchain_backups:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /opt/provchain/backups

  # Logs
  provchain_logs:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /opt/provchain/logs

  # Monitoring data
  prometheus_data:
    driver: local
  grafana_data:
    driver: local
  jaeger_data:
    driver: local
```

### Backup Strategy

```bash
#!/bin/bash
# /deploy/scripts/backup.sh

BACKUP_DIR="/opt/provchain/backups"
DATE=$(date +%Y%m%d_%H%M%S)

# Backup blockchain data
docker exec provchain-org \
  tar czf /app/backups/blockchain_${DATE}.tar.gz /app/data

# Backup to external location (optional)
# aws s3 cp /opt/provchain/backups/blockchain_${DATE}.tar.gz \
#   s3://provchain-backups/

# Cleanup old backups (keep last 30 days)
find $BACKUP_DIR -name "blockchain_*.tar.gz" -mtime +30 -delete
```

---

## Configuration Management

### Environment Variables

**Required:**
```bash
JWT_SECRET=your-secure-secret-here
CONFIG_FILE=/app/config/production-deployment.toml
```

**Optional:**
```bash
RUST_LOG=info                    # log level
PROVCHAIN_PEERS=192.168.1.101:8080,192.168.1.102:8080
PROVCHAIN_PORT=8080
OTEL_SERVICE_NAME=provchain-node1
JAEGER_ENDPOINT=http://jaeger:14268/api/traces
```

### Configuration File (TOML)

```toml
# /app/config/production-deployment.toml
node_id = "provchain-node-1"

[network]
network_id = "provchain-prod"
listen_port = 8080
bind_address = "0.0.0.0"
known_peers = ["192.168.1.101:8080", "192.168.1.102:8080"]
max_peers = 50
connection_timeout = 30
ping_interval = 30

[consensus]
is_authority = false
authority_keys = ["authority_public_key_here"]
block_interval = 10
max_block_size = 1048576

[storage]
data_dir = "/app/data"
persistent = true
store_type = "oxigraph"
cache_size_mb = 200

[web]
host = "0.0.0.0"
port = 8080

[web.cors]
enabled = true
allowed_origins = ["http://localhost:5173", "https://provchain.example.com"]
allowed_methods = ["GET", "POST", "OPTIONS"]
allowed_headers = ["Authorization", "Content-Type"]
allow_credentials = true
max_age = 3600

[logging]
level = "info"
format = "json"
```

---

## Security Considerations

### Container Security

✅ **Implemented:**
- Non-root user (`provchain`)
- Minimal base image (debian:bookworm-slim)
- Read-only root filesystem (where possible)
- Health checks for automatic recovery

⚠️ **Recommended:**
- Scan images for vulnerabilities: `docker scan provchain-org:latest`
- Use image signing: `docker trust sign`
- Implement seccomp profiles
- Enable AppArmor profiles

### Network Security

```yaml
# Firewall rules (ufw)
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 80/tcp    # HTTP
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow from 10.0.0.0/8 to any port 8080  # Internal API
sudo ufw allow from 10.0.0.0/8 to any port 9090  # Metrics
sudo ufw enable
```

### Secrets Management

**Option 1: Environment Variables (.env file)**
```bash
# .env (NEVER commit to git)
JWT_SECRET=your-secret-here
REDIS_PASSWORD=redis-secret
GRAFANA_PASSWORD=grafana-secret
```

**Option 2: Docker Secrets (Swarm mode)**
```bash
echo "your-secret-here" | docker secret create jwt_secret -
```

**Option 3: Kubernetes Secrets**
```bash
kubectl create secret generic provchain-secrets \
  --from-literal=jwt-secret="your-secret-here"
```

---

## Performance Optimization

### Resource Limits

```yaml
services:
  provchain-org:
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 4G
        reservations:
          cpus: '1.0'
          memory: 2G
```

### Cache Optimization

```toml
[storage]
cache_size_mb = 200  # Increase for better performance

[integrity.performance]
validation_level = "Standard"  # Balance performance and thoroughness
enable_caching = true
cache_ttl_seconds = 600
```

### Network Optimization

```yaml
# Use host networking for P2P (Linux only)
services:
  provchain-org:
    network_mode: "host"  # Bypass bridge network
```

---

## Troubleshooting Guide

### Common Issues

#### 1. Container fails to start

**Symptoms:**
```bash
$ docker compose up -d
ERROR: for provchain-org  Cannot start service provchain-org: ...
```

**Diagnosis:**
```bash
# Check logs
docker logs provchain-org

# Check if port is already in use
sudo lsof -i :8080

# Check disk space
df -h
```

**Solutions:**
```bash
# Kill process using port
sudo kill -9 $(sudo lsof -t -i:8080)

# Clean up disk space
docker system prune -a

# Restart with clean state
docker compose down -v
docker compose up -d
```

#### 2. Nodes cannot connect

**Symptoms:**
```bash
$ curl http://localhost:8080/api/peers
[]
```

**Diagnosis:**
```bash
# Check network connectivity
docker network inspect provchain_network

# Check container is running
docker ps | grep provchain

# Check firewall
sudo ufw status
sudo iptables -L -n | grep 8080
```

**Solutions:**
```bash
# Open firewall port
sudo ufw allow 8080/tcp

# Verify PEERS environment variable
docker compose exec provchain-org env | grep PEERS

# Check logs for connection errors
docker logs provchain-node1 | grep -i peer
```

#### 3. High memory usage

**Symptoms:**
```bash
$ docker stats
CONTAINER   CPU %   MEM USAGE / LIMIT
provchain   15%     2.5GB / 4GB
```

**Solutions:**
```toml
# Reduce cache size
[storage]
cache_size_mb = 50  # Reduce from 200
```

```yaml
# Add memory limit in docker-compose.yml
services:
  provchain-org:
    mem_limit: 2g
```

#### 4. Out of sync nodes

**Symptoms:**
```bash
# Different blockchain lengths
$ curl http://node1:8080/api/blockchain/dump | jq 'length'
42
$ curl http://node2:8080/api/blockchain/dump | jq 'length'
38
```

**Solutions:**
```bash
# Restart lagging node
docker compose restart provchain-node2

# Or force resync
docker compose down
rm -rf data/*
docker compose up -d
```

### Debug Mode

```bash
# Enable debug logging
export RUST_LOG=debug

# Restart with debug logs
docker compose up -d

# Follow logs
docker logs -f provchain-org

# Check container environment
docker compose exec provchain-org env

# Check container processes
docker compose exec provchain-org ps aux
```

---

## Quick Reference

### Essential Commands

```bash
# Start services
docker compose -f docker-compose.production.yml up -d

# Stop services
docker compose -f docker-compose.production.yml down

# View logs
docker logs -f provchain-org

# Check health
curl http://localhost:8080/health

# Check peers
curl http://localhost:8080/api/peers | jq '.'

# Check blockchain
curl http://localhost:8080/api/blockchain/dump | jq 'length'

# Submit transaction
curl -X POST http://localhost:8080/api/transactions \
  -H "Content-Type: application/json" \
  -d '{"data":"<http://test> <http://p> \"v\" ."}'

# Rebuild image
docker compose -f docker-compose.production.yml build --no-cache

# Clean up
docker system prune -a
```

### Port Reference

| Port | Service | Description |
|------|---------|-------------|
| 80 | Nginx | HTTP |
| 443 | Nginx | HTTPS |
| 8080-8082 | ProvChain Nodes | API & P2P |
| 9090 | Node Metrics | Prometheus scraping |
| 9091 | Prometheus | Metrics UI |
| 3000/3001 | Grafana | Dashboards |
| 16686 | Jaeger | Tracing UI |
| 14268 | Jaeger | Trace collector |
| 6379 | Redis | Caching |
| 3100 | Loki | Log aggregation |

### File Locations

| File | Purpose |
|------|---------|
| `/deploy/Dockerfile.production` | Production image |
| `/deploy/docker-compose.production.yml` | Single-node stack |
| `/deploy/docker-compose.3node.yml` | 3-node cluster (1 host) |
| `/deploy/docker-compose.node.yml` | Single node for distributed |
| `/deploy/docker-compose.monitoring.yml` | Monitoring stack |
| `/deploy/monitoring/prometheus.yml` | Prometheus config |
| `/deploy/nginx/nginx.conf` | Reverse proxy |
| `/config/production-deployment.toml` | Application config |

### Environment Variables

```bash
# Required
JWT_SECRET=your-secret-here

# Optional
RUST_LOG=info
PROVCHAIN_PEERS=192.168.1.101:8080,192.168.1.102:8080
PROVCHAIN_PORT=8080
OTEL_SERVICE_NAME=provchain-node1

# Monitoring
GRAFANA_USER=admin
GRAFANA_PASSWORD=admin
REDIS_PASSWORD=provchain123
```

---

## Related Documentation

- **Setup & Installation:** `/docs/deployment/SETUP_INSTALLATION_GUIDE.md`
- **Comprehensive Analysis:** `/docs/deployment/COMPREHENSIVE_ANALYSIS_REPORT.md`
- **Production Deployment:** `/docs/PHASE8_PRODUCTION_DEPLOYMENT_GUIDE.md`
- **Multi-Node Quick Start:** `/deploy/README_MULTI_NODE.md`

---

`★ Insight ─────────────────────────────────────`
**Docker Architecture Best Practices Applied**
1. **Multi-stage builds** reduce final image size from ~2.5GB to ~150MB by excluding build tools and dependencies
2. **Non-root container execution** enhances security by limiting attack surface—a critical practice for production deployments
3. **Health checks** enable orchestrators to automatically restart failing containers, improving reliability without manual intervention
`─────────────────────────────────────────────────`

---

**Document End**

For questions or issues, refer to the troubleshooting guide or consult the related documentation.
