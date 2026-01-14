# Multi-Node Deployment Guide

**Quick Start Guide for Distributed ProvChain-Org Deployment**

This guide explains how to deploy ProvChain-Org on **three separate machines** (VMs or physical servers) to create a distributed blockchain network with monitoring.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                   Multi-Machine Distributed Deployment                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────┐    ┌─────────────────────────────┐         │
│  │   Machine 1 (Bootstrap)     │    │   Machine 2                 │         │
│  │   IP: 192.168.1.101         │    │   IP: 192.168.1.102         │         │
│  │   ┌─────────────────────┐   │    │   ┌─────────────────────┐   │         │
│  │   │   Node 1            │   │    │   │   Node 2            │   │         │
│  │   │   PEERS=""          │   │◀──▶│   │   PEERS="192.168...│   │         │
│  │   │   (Bootstrap)       │   │    │   │                     │   │         │
│  │   │   Port: 8080/9090   │   │    │   │   Port: 8080/9090   │   │         │
│  │   └─────────────────────┘   │    │   └─────────────────────┘   │         │
│  └─────────────────────────────┘    └─────────────────────────────┘         │
│                                                                              │
│                                      ┌─────────────────────────────┐         │
│                                      │   Machine 3                 │         │
│                                      │   IP: 192.168.1.103         │         │
│                                      │   ┌─────────────────────┐   │         │
│                                      │   │   Node 3            │   │         │
│                                      │   │   PEERS="192.168...│   │         │
│                                      │   │   Port: 8080/9090   │   │         │
│                                      │   └─────────────────────┘   │         │
│                                      └─────────────────────────────┘         │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │   Monitoring Machine (can be Machine 1 or separate)                  │    │
│  │   IP: 192.168.1.101 (if on Machine 1)                                │    │
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

---

## Prerequisites

### Hardware Requirements

| Node Type | vCPU | RAM | Storage | Network |
|-----------|------|-----|---------|---------|
| **Bootstrap** | 2-4 | 4-8 GB | 100 GB SSD | 1 Gbps |
| **Regular** | 2 | 4 GB | 50 GB SSD | 1 Gbps |
| **Monitoring** | 1-2 | 2-4 GB | 50 GB SSD | 1 Gbps |

**Minimum for Testing:** 2 vCPU, 2 GB RAM, 20 GB SSD per node

### Software Requirements

All machines must have:
- **Docker:** 20.10 or higher
- **Docker Compose:** v2.0 or higher
- **Git:** For cloning the repository
- **Open ports:** 8080 (API/P2P), 9090 (metrics)

### Network Requirements

- All machines must be able to communicate with each other
- Ports 8080 and 9090 must be open between nodes
- Static IP addresses recommended
- NTP synchronization enabled on all nodes

**Firewall Configuration:**
```bash
# On all nodes
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 8080/tcp  # ProvChain API/P2P
sudo ufw allow 9090/tcp  # Metrics
sudo ufw enable
```

---

## Quick Start (15 Minutes)

> **Choose Your Deployment Method:**
>
> - **Option A: Prebuilt Images (Recommended)** - Faster deployment, no build required
> - **Option B: Build from Source** - For development or custom builds
>
> See details below.

### Step 1: Prepare All Machines

**On ALL machines (Node 1, Node 2, Node 3):**

```bash
# Install Docker (if not already installed)
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Log out and back in for group changes to take effect
# Or use: newgrp docker

# Verify installation
docker --version
docker compose version

# Clone repository
git clone https://github.com/anusornc/provchain-org2.git
cd provchain-org/deploy
```

---

### Deployment Method Selection

#### **Option A: Prebuilt Images (Recommended)**

Skip the build process and use official Docker Hub images:

```bash
# Pull the latest image
docker pull anusornc/provchain-org:latest

# Edit docker-compose.node.yml to use prebuilt image
# Comment out the 'build' section and uncomment 'image':
sed -i 's|    # image: anusornc/provchain-org:latest|    image: anusornc/provchain-org:latest|' docker-compose.node.yml
sed -i 's|    build:|    # build:|' docker-compose.node.yml
sed -i 's|      context: ..|    #   context: ..|' docker-compose.node.yml
sed -i 's|      dockerfile: deploy/Dockerfile.production|    #   dockerfile: deploy/Dockerfile.production|' docker-compose.node.yml
```

**Benefits:**
- ✅ Fast deployment (no build time)
- ✅ Smaller download (optimized image)
- ✅ Official releases tested and verified

---

#### **Option B: Build from Source**

Build the Docker image locally from source code:

```bash
# The default docker-compose.node.yml uses build from source
# Just run the commands as shown below - no changes needed
```

**Use this when:**
- You want to modify the source code
- You need to test local changes
- Build time is not a concern

---

### Step 2: Configure Node 1 (Bootstrap)

**On Machine 1:**

```bash
cd provchain-org/deploy

# Create .env file
cat > .env <<EOF
# Bootstrap Node Configuration
PEERS=""
JWT_SECRET=$(openssl rand -base64 32)
RUST_LOG=info
EOF

# Start the container
# If using prebuilt images (Option A):
docker compose -f docker-compose.node.yml up -d
# If building from source (Option B):
# docker compose -f docker-compose.node.yml up -d --build

# Get IP address
IP_ADDRESS=$(hostname -I | awk '{print $1}')
echo "Node 1 IP: $IP_ADDRESS"
```

**Wait 30 seconds for Node 1 to fully initialize.**

### Step 3: Configure Node 2

**On Machine 2:**

```bash
cd provchain-org/deploy

# Replace NODE1_IP with actual IP from Step 2
cat > .env <<EOF
# Node 2 Configuration
PEERS="NODE1_IP:8080"
JWT_SECRET=$(openssl rand -base64 32)
RUST_LOG=info
EOF

# Start the container
# If using prebuilt images (Option A):
docker compose -f docker-compose.node.yml up -d
# If building from source (Option B):
# docker compose -f docker-compose.node.yml up -d --build

# Get IP address
IP_ADDRESS=$(hostname -I | awk '{print $1}')
echo "Node 2 IP: $IP_ADDRESS"
```

### Step 4: Configure Node 3

**On Machine 3:**

```bash
cd provchain-org/deploy

# Replace NODE1_IP and NODE2_IP with actual IPs
cat > .env <<EOF
# Node 3 Configuration
PEERS="NODE1_IP:8080,NODE2_IP:8080"
JWT_SECRET=$(openssl rand -base64 32)
RUST_LOG=info
EOF

# Start the container
# If using prebuilt images (Option A):
docker compose -f docker-compose.node.yml up -d
# If building from source (Option B):
# docker compose -f docker-compose.node.yml up -d --build

# Get IP address
IP_ADDRESS=$(hostname -I | awk '{print $1}')
echo "Node 3 IP: $IP_ADDRESS"
```

### Step 5: Configure Monitoring

**On Machine 1 (or separate monitoring machine):**

```bash
cd provchain-org/deploy

# Update Prometheus configuration with node IPs
cat > monitoring/prometheus_multi_node.yml <<EOF
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'provchain-nodes'
    static_configs:
      - targets:
          - 'NODE1_IP:9090'
          - 'NODE2_IP:9090'
          - 'NODE3_IP:9090'
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
EOF

# Start monitoring stack
docker compose -f docker-compose.monitoring.yml up -d
```

---

## Verification

### Health Check Script

**Save as `verify_cluster.sh` and run on any machine:**

```bash
#!/bin/bash

# Replace with your actual IPs
NODE1="192.168.1.101"
NODE2="192.168.1.102"
NODE3="192.168.1.103"

echo "=== ProvChain Cluster Verification ==="
echo ""

# Test 1: Health endpoints
echo "Test 1: Health Endpoints"
for node in $NODE1 $NODE2 $NODE3; do
  if curl -sf http://$node:8080/health | grep -q "healthy"; then
    echo "  ✓ Node $node: Healthy"
  else
    echo "  ✗ Node $node: Unreachable"
  fi
done
echo ""

# Test 2: Peer connectivity
echo "Test 2: Peer Connectivity"
peers=$(curl -s http://$NODE1:8080/api/peers | jq 'length')
echo "  Bootstrap node sees $peers peer(s)"
if [ "$peers" -eq 2 ]; then
  echo "  ✓ All peers connected"
else
  echo "  ✗ Expected 2 peers, found $peers"
fi
echo ""

# Test 3: Blockchain synchronization
echo "Test 3: Blockchain Synchronization"
length1=$(curl -s http://$NODE1:8080/api/blockchain/dump | jq 'length')
length2=$(curl -s http://$NODE2:8080/api/blockchain/dump | jq 'length')
length3=$(curl -s http://$NODE3:8080/api/blockchain/dump | jq 'length')

echo "  Node 1: $length1 blocks"
echo "  Node 2: $length2 blocks"
echo "  Node 3: $length3 blocks"

if [ "$length1" -eq "$length2" ] && [ "$length2" -eq "$length3" ]; then
  echo "  ✓ All nodes synchronized"
else
  echo "  ✗ Blockchain sync mismatch"
fi
echo ""

# Test 4: Monitoring
echo "Test 4: Monitoring Stack"
if curl -sf http://$NODE1:9090/-/healthy > /dev/null; then
  echo "  ✓ Prometheus is running"
fi
if curl -sf http://$NODE1:3000/api/health > /dev/null; then
  echo "  ✓ Grafana is running"
fi
echo ""

echo "=== Verification Complete ==="
```

**Run the script:**
```bash
chmod +x verify_cluster.sh
./verify_cluster.sh
```

### Manual Verification

```bash
# Check each node's health
curl http://NODE1_IP:8080/health
curl http://NODE2_IP:8080/health
curl http://NODE3_IP:8080/health

# Check peer connections
curl http://NODE1_IP:8080/api/peers | jq '.'

# Check blockchain sync
curl http://NODE1_IP:8080/api/blockchain/dump | jq 'length'
curl http://NODE2_IP:8080/api/blockchain/dump | jq 'length'
curl http://NODE3_IP:8080/api/blockchain/dump | jq 'length'

# Access monitoring dashboards
# Grafana: http://NODE1_IP:3000 (admin/admin)
# Prometheus: http://NODE1_IP:9090
# Jaeger: http://NODE1_IP:16686
```

---

## Configuration Details

### Environment Variables

| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
| `PEERS` | Comma-separated list of peer addresses | Yes | - |
| `JWT_SECRET` | JWT authentication secret | Yes | - |
| `RUST_LOG` | Logging level (info, debug, warn, error) | No | info |
| `JAEGER_ENDPOINT` | Jaeger tracing endpoint | No | - |

### Peer Discovery Pattern

```
Node 1 (Bootstrap):
  PEERS=""                      # Starts the network

Node 2:
  PEERS="192.168.1.101:8080"   # Connects to Node 1

Node 3:
  PEERS="192.168.1.101:8080,192.168.1.102:8080"  # Connects to both
```

---

## Troubleshooting

### Nodes Cannot Connect

**Symptoms:**
- `curl http://NODE_IP:8080/api/peers` returns empty array `[]`

**Diagnosis:**
```bash
# Test network connectivity
ping NODE2_IP

# Test port accessibility
telnet NODE2_IP 8080

# Check firewall
sudo ufw status
```

**Solution:**
```bash
# Open firewall ports
sudo ufw allow 8080/tcp
sudo ufw allow 9090/tcp

# Restart node
docker compose -f docker-compose.node.yml restart
```

### Blockchain Not Syncing

**Symptoms:**
- Different blockchain lengths across nodes

**Solution:**
```bash
# Restart lagging node
docker compose -f docker-compose.node.yml restart

# Or force resync
docker compose -f docker-compose.node.yml down
rm -rf node_data/*
docker compose -f docker-compose.node.yml up -d
```

### High Memory Usage

**Symptoms:**
- Container crashes due to OOM

**Solution:**
```bash
# Add memory limit in docker-compose.node.yml
services:
  provchain-node:
    mem_limit: 2g
```

### View Logs

```bash
# View container logs
docker logs -f provchain-node

# Check all containers
docker ps

# Inspect container environment
docker inspect provchain-node | jq '.[0].Config.Env'
```

---

## Common Operations

### Submit Transaction

```bash
# From any machine, submit a test transaction
curl -X POST http://localhost:8080/api/transactions \
  -H "Content-Type: application/json" \
  -d '{"data":"<http://example.org/test> <http://example.org/p> \"test\" ."}'
```

### Performance Testing

After cluster deployment, validate performance across multiple nodes:

```bash
# Navigate to benchmark toolkit (from any node)
cd ../benchmark-toolkit

# Quick performance check (5 minutes)
./run.sh low

# Full benchmark suite (15 minutes)
./run.sh medium

# Extensive testing for multi-node (45 minutes)
./run.sh high
```

**Multi-Node Testing Considerations**:
- Run benchmarks from Node 1 (coordinator node)
- Toolkit automatically tests distributed consensus
- Results include network latency between nodes
- Cross-chain sync performance is measured

**Access Results**:
- **Grafana Dashboard**: http://node1.example.com:3000
- **Summary Report**: `benchmark-toolkit/results/summary.md`
- **Raw Data**: `benchmark-toolkit/results/benchmark_results.csv`

**Multi-Node Specific Metrics**:
- Distributed consensus overhead (PoA/PBFT)
- Cross-node communication latency
- Network throughput between nodes
- Data synchronization speed

**For detailed benchmarking guide**, see [BENCHMARKING.md](../BENCHMARKING.md) or [docs/benchmarking/README.md](../docs/benchmarking/README.md)

### Stop Cluster

```bash
# Stop on all machines
docker compose -f docker-compose.node.yml down

# Stop monitoring
docker compose -f docker-compose.monitoring.yml down
```

### Start Cluster

```bash
# Start on all machines (Node 1 first)
docker compose -f docker-compose.node.yml up -d
```

### Update Application

```bash
# If using prebuilt images (Option A):
docker pull anusornc/provchain-org:latest
docker compose -f docker-compose.node.yml up -d

# If building from source (Option B):
# git pull
# docker compose -f docker-compose.node.yml up -d --build
```

---

## Advanced Configuration

### Custom Configuration File

Create a custom TOML configuration:

```bash
# Create config/custom-node.toml
cat > config/custom-node.toml <<EOF
node_id = "custom-node-1"

[network]
network_id = "provchain-custom"
listen_port = 8080
bind_address = "0.0.0.0"
known_peers = ["192.168.1.101:8080"]

[storage]
data_dir = "/app/data"
cache_size_mb = 200

[web]
host = "0.0.0.0"
port = 8080
EOF

# Use in docker-compose.node.yml
environment:
  - CONFIG_FILE=/app/config/custom-node.toml
```

### Enable TLS/SSL

```bash
# Generate self-signed certificates
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

# Add nginx reverse proxy (see docker-compose.production.yml)
```

---

## Reference Documentation

For more detailed information, see:

- **Docker Architecture:** `/docs/deployment/DOCKER_DEPLOYMENT_ARCHITECTURE.md`
- **Complete Setup Guide:** `/docs/deployment/SETUP_INSTALLATION_GUIDE.md`
- **Production Deployment:** `/docs/PHASE8_PRODUCTION_DEPLOYMENT_GUIDE.md`
- **Comprehensive Analysis:** `/docs/deployment/COMPREHENSIVE_ANALYSIS_REPORT.md`

---

## Support

- **Issues:** https://github.com/anusornc/provchain-org2/issues
- **Documentation:** `/docs/deployment/`
- **Logs:** Always check container logs first: `docker logs provchain-node`

---

**Quick Checklist:**

- [ ] 3 machines with Docker installed
- [ ] Network connectivity between all machines
- [ ] Firewall ports 8080, 9090 open
- [ ] Node 1 started with empty PEERS
- [ ] Node 2, 3 started with correct PEERS
- [ ] Monitoring stack started (optional)
- [ ] Health checks pass
- [ ] Peers connected (2 peers on bootstrap node)
- [ ] Blockchain synchronized across all nodes

**Done!** Your multi-node cluster is now running.

---

## Available Docker Images

Official images are available on Docker Hub:

| Tag | Description | Use Case |
|-----|-------------|----------|
| `anusornc/provchain-org:latest` | Latest stable release | Production deployments |
| `anusornc/provchain-org:v1.0.0` | Versioned release | Specific version pinning |

**Pull specific version:**
```bash
docker pull anusornc/provchain-org:v1.0.0
# Then update docker-compose.node.yml to use:
# image: anusornc/provchain-org:v1.0.0
```

---

## Switching Between Deployment Methods

### From Prebuilt to Build from Source

```bash
# Edit docker-compose.node.yml
# Comment out: image: anusornc/provchain-org:latest
# Uncomment: build: section

docker compose -f docker-compose.node.yml up -d --build
```

### From Build to Prebuilt

```bash
# Edit docker-compose.node.yml
# Uncomment: image: anusornc/provchain-org:latest
# Comment out: build: section

docker compose -f docker-compose.node.yml up -d
```

`★ Insight ─────────────────────────────────────`
**Bootstrap Node Pattern**
1. Node 1 starts with empty `PEERS` list, serving as the network bootstrap point
2. Subsequent nodes specify one or more peers, creating a resilient mesh network
3. Each node maintains connections to all specified peers, preventing network partition
`─────────────────────────────────────────────────`
