# ProvChain-Org: Setup & Installation Guide
## 3-Node Cloud Deployment

**Version:** 1.0
**Date:** 2026-01-02
**Target Audience:** DevOps Engineers, System Administrators
**Estimated Time:** 2-4 hours (Docker Compose) | 1-2 days (Kubernetes)

---

## Table of Contents

1. [Prerequisites](#1-prerequisites)
2. [Quick Start (Docker Compose)](#2-quick-start-docker-compose)
3. [Detailed Setup - Docker Compose](#3-detailed-setup---docker-compose)
4. [Kubernetes Deployment](#4-kubernetes-deployment)
5. [Native Systemd Deployment](#5-native-systemd-deployment)
6. [Post-Installation Validation](#6-post-installation-validation)
7. [Monitoring Setup](#7-monitoring-setup)
8. [Troubleshooting](#8-troubleshooting)

---

## 1. Prerequisites

### Cloud Resources

**Required:**
- [ ] 3 VMs provisioned (see specs below)
- [ ] VPC/Network configured (10.0.0.0/16 or equivalent)
- [ ] Security groups/firewall rules configured
- [ ] SSH access to all VMs

**VM Specifications:**

| Node | vCPU | RAM | Storage | IP |
|------|------|-----|---------|-----|
| VM-1 (Authority) | 2-4 | 4-8 GB | 100 GB SSD | 10.0.1.10 |
| VM-2 (Regular) | 2 | 4 GB | 50 GB SSD | 10.0.1.11 |
| VM-3 (Regular) | 2 | 4 GB | 50 GB SSD | 10.0.1.12 |

**Minimum for Testing:**
- 2 vCPU, 2 GB RAM, 20 GB SSD per node

### Firewall Rules

**Inbound Rules:**

| Port | Protocol | Source | Purpose |
|------|----------|--------|---------|
| 22 | TCP | Your IP | SSH access |
| 8080 | TCP | 0.0.0.0/0 | HTTP API + P2P |
| 9090 | TCP | VPC subnet | Metrics export |
| 3001 | TCP | Your IP | Grafana UI (monitoring) |
| 9091 | TCP | Your IP | Prometheus UI |
| 16686 | TCP | Your IP | Jaeger UI (tracing) |

**Outbound Rules:**
- Allow all (for apt updates, docker pulls, etc.)

### Software Requirements

Choose your deployment method and install dependencies:

#### Option A: Docker Compose (Recommended)

```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Log out and back in for group changes to take effect
exit
# SSH back in

# Verify Docker installation
docker --version
docker compose version

# Expected: Docker version 20.10+, Docker Compose v2.0+
```

#### Option B: Kubernetes

```bash
# Install kubectl
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl

# Verify
kubectl version --client

# Configure kubectl to access your cluster
# (Instructions depend on your K8s provider: EKS, GKE, AKS, etc.)
```

#### Option C: Native (Systemd)

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify
rustc --version  # Should be >= 1.70

# Install system dependencies
sudo apt-get update && sudo apt-get install -y \
    pkg-config \
    libssl-dev \
    libclang-dev \
    build-essential \
    ca-certificates
```

---

## 2. Quick Start (Docker Compose)

**For impatient users who want to get running ASAP:**

```bash
# On VM-1 (Authority Node)
git clone https://github.com/yourusername/provchain-org.git
cd provchain-org

# Generate authority keypair
docker run --rm -v $(pwd):/app -w /app provchain-org:latest \
  provchain-org generate-keypair --output authority.key

# Extract public key
docker run --rm -v $(pwd):/app -w /app provchain-org:latest \
  provchain-org show-pubkey --key authority.key > authority.pub

# Generate JWT secret
export JWT_SECRET=$(openssl rand -base64 32)
echo "Save this: $JWT_SECRET"

# Update config for authority node
cat > config/node1.toml <<EOF
node_id = "authority-node-1"

[network]
network_id = "provchain-prod"
listen_port = 8080
bind_address = "0.0.0.0"
known_peers = ["10.0.1.11:8080", "10.0.1.12:8080"]

[consensus]
is_authority = true
authority_key_file = "/app/authority.key"
authority_keys = ["$(cat authority.pub)"]
block_interval = 10

[storage]
data_dir = "/app/data"
EOF

# Start authority node
docker compose -f deploy/docker-compose.node.yml up -d

# Verify
sleep 10
curl http://localhost:8080/health
# Expected: {"status":"healthy"}

# On VM-2 (Regular Node)
git clone https://github.com/yourusername/provchain-org.git
cd provchain-org

# Copy authority public key from VM-1
scp user@10.0.1.10:~/provchain-org/authority.pub .

# Update config
cat > config/node2.toml <<EOF
node_id = "regular-node-2"

[network]
network_id = "provchain-prod"
listen_port = 8080
known_peers = ["10.0.1.10:8080", "10.0.1.12:8080"]

[consensus]
is_authority = false
authority_keys = ["$(cat authority.pub)"]

[storage]
data_dir = "/app/data"
EOF

# Start node
docker compose -f deploy/docker-compose.node.yml up -d

# On VM-3 (Regular Node) - Same as VM-2 but with node3.toml and different known_peers

# Verification
curl http://10.0.1.10:8080/api/peers | jq 'length'
# Expected: 2 (both peers connected)
```

**That's it!** Your 3-node cluster is running. Proceed to [Section 6](#6-post-installation-validation) for validation.

---

## 3. Detailed Setup - Docker Compose

### Step 3.1: Prepare Infrastructure

#### Create Project Directory Structure

On **all VMs** (VM-1, VM-2, VM-3):

```bash
# Create user and project directory
sudo useradd -r -s /bin/bash -m -d /opt/provchain provchain
sudo mkdir -p /opt/provchain/{config,keys,data,backups,logs}
sudo chown -R provchain:provchain /opt/provchain

# Switch to provchain user
sudo su - provchain
cd /opt/provchain

# Clone repository
git clone https://github.com/yourusername/provchain-org.git
cd provchain-org

# Verify files
ls -la
# Should see: Cargo.toml, src/, deploy/, config/, etc.
```

### Step 3.2: Generate Cryptographic Keys

**On VM-1 (Authority Node) ONLY:**

```bash
cd /opt/provchain/provchain-org

# Method 1: Using pre-built Docker image (if available)
docker pull provchain-org:latest
docker run --rm -v $(pwd):/app -w /app provchain-org:latest \
  provchain-org generate-keypair --output /app/authority.key

# Method 2: Build and run locally
cargo build --release
./target/release/provchain-org generate-keypair --output authority.key

# Secure the private key
chmod 600 authority.key
chown provchain:provchain authority.key

# Extract public key
docker run --rm -v $(pwd):/app -w /app provchain-org:latest \
  provchain-org show-pubkey --key /app/authority.key > authority.pub

# OR if using local binary:
./target/release/provchain-org show-pubkey --key authority.key > authority.pub

# Display public key (you'll need this for other nodes)
cat authority.pub
# Example output: 302a300506032b6570032100a1b2c3d4e5f6...

# BACKUP THE PRIVATE KEY SECURELY
cp authority.key /opt/provchain/backups/authority.key.backup
# Also copy to secure location (encrypted USB, KMS, etc.)
```

**Copy Authority Public Key to VM-2 and VM-3:**

```bash
# From VM-1:
scp authority.pub provchain@10.0.1.11:/opt/provchain/provchain-org/
scp authority.pub provchain@10.0.1.12:/opt/provchain/provchain-org/
```

### Step 3.3: Generate JWT Secret

**On your local machine or VM-1:**

```bash
# Generate 256-bit random secret
openssl rand -base64 32

# Example output: K7gNU3sdo+OL0wNhqoVWhr3g6s1xYv72ol/pe/Unols=

# SAVE THIS SECRET - You'll use it on all nodes
export JWT_SECRET="K7gNU3sdo+OL0wNhqoVWhr3g6s1xYv72ol/pe/Unols="
```

**IMPORTANT:** Store this secret securely. You'll need it for all nodes.

### Step 3.4: Create Node Configurations

#### VM-1 (Authority Node): `/opt/provchain/provchain-org/config/node1-authority.toml`

```bash
cat > /opt/provchain/provchain-org/config/node1-authority.toml <<EOF
# Authority Node Configuration
node_id = "authority-node-1"

[network]
network_id = "provchain-prod"
listen_port = 8080
bind_address = "0.0.0.0"
known_peers = ["10.0.1.11:8080", "10.0.1.12:8080"]
max_peers = 50
connection_timeout = 30
ping_interval = 30

[consensus]
is_authority = true
authority_key_file = "/app/keys/authority.key"
authority_keys = ["$(cat authority.pub)"]
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
EOF
```

#### VM-2 (Regular Node): `/opt/provchain/provchain-org/config/node2-regular.toml`

```bash
cat > /opt/provchain/provchain-org/config/node2-regular.toml <<EOF
# Regular Node Configuration
node_id = "regular-node-2"

[network]
network_id = "provchain-prod"
listen_port = 8080
bind_address = "0.0.0.0"
known_peers = ["10.0.1.10:8080", "10.0.1.12:8080"]
max_peers = 50
connection_timeout = 30
ping_interval = 30

[consensus]
is_authority = false
authority_keys = ["$(cat authority.pub)"]
block_interval = 10
max_block_size = 1048576

[storage]
data_dir = "/app/data"
persistent = true
store_type = "oxigraph"
cache_size_mb = 100

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
EOF
```

#### VM-3 (Regular Node): `/opt/provchain/provchain-org/config/node3-regular.toml`

```bash
cat > /opt/provchain/provchain-org/config/node3-regular.toml <<EOF
# Regular Node Configuration
node_id = "regular-node-3"

[network]
network_id = "provchain-prod"
listen_port = 8080
bind_address = "0.0.0.0"
known_peers = ["10.0.1.10:8080", "10.0.1.11:8080"]
max_peers = 50
connection_timeout = 30
ping_interval = 30

[consensus]
is_authority = false
authority_keys = ["$(cat authority.pub)"]
block_interval = 10
max_block_size = 1048576

[storage]
data_dir = "/app/data"
persistent = true
store_type = "oxigraph"
cache_size_mb = 100

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
EOF
```

### Step 3.5: Build Docker Image

**On VM-1** (you can build once and push to registry for other nodes):

```bash
cd /opt/provchain/provchain-org

# Build production image
docker build -f deploy/Dockerfile.production -t provchain-org:v1.0.0 .

# This will take 10-15 minutes on first build
# Expected size: ~150 MB

# Tag as latest
docker tag provchain-org:v1.0.0 provchain-org:latest

# Verify
docker images | grep provchain
```

**Optional: Push to Registry**

```bash
# Tag for registry
docker tag provchain-org:v1.0.0 your-registry.io/provchain-org:v1.0.0

# Push
docker push your-registry.io/provchain-org:v1.0.0

# On VM-2 and VM-3:
docker pull your-registry.io/provchain-org:v1.0.0
docker tag your-registry.io/provchain-org:v1.0.0 provchain-org:latest
```

### Step 3.6: Create Docker Compose File

**Create `deploy/docker-compose.node.yml`** (same file for all nodes, but use different configs):

#### VM-1 (Authority):

```yaml
cat > deploy/docker-compose.node.yml <<EOF
version: '3.8'

services:
  provchain:
    image: provchain-org:latest
    container_name: provchain-node1
    restart: unless-stopped
    ports:
      - "8080:8080"  # HTTP API + P2P
      - "9090:9090"  # Prometheus metrics
    environment:
      - RUST_LOG=info
      - JWT_SECRET=${JWT_SECRET}
      - CONFIG_FILE=/app/config/node1-authority.toml
    volumes:
      - ./config:/app/config:ro
      - ./authority.key:/app/keys/authority.key:ro
      - provchain_data:/app/data
      - ./logs:/app/logs
    networks:
      - provchain_net
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s

volumes:
  provchain_data:
    driver: local

networks:
  provchain_net:
    driver: bridge
EOF
```

#### VM-2 (Regular):

```yaml
# Same file, but change:
# - container_name: provchain-node2
# - CONFIG_FILE=/app/config/node2-regular.toml
# - Remove authority.key volume mount
```

#### VM-3 (Regular):

```yaml
# Same as VM-2, but change:
# - container_name: provchain-node3
# - CONFIG_FILE=/app/config/node3-regular.toml
```

### Step 3.7: Deploy Nodes

#### Deploy Authority Node (VM-1) FIRST

```bash
cd /opt/provchain/provchain-org

# Export JWT secret
export JWT_SECRET="K7gNU3sdo+OL0wNhqoVWhr3g6s1xYv72ol/pe/Unols="

# Start node
docker compose -f deploy/docker-compose.node.yml up -d

# Check logs
docker logs -f provchain-node1

# Wait for "Server listening on 0.0.0.0:8080" message
# Press Ctrl+C to stop following logs

# Verify node is running
docker ps
# Should show provchain-node1 with status "Up" and "healthy"

# Test health endpoint
curl http://localhost:8080/health
# Expected: {"status":"healthy","version":"0.1.0",...}

# Check if authority is active
curl http://localhost:8080/api/consensus/stats | jq '.'
# Should show: "is_authority": true
```

**IMPORTANT:** Wait 30 seconds for authority to fully initialize before deploying other nodes.

#### Deploy Regular Node 2 (VM-2)

```bash
cd /opt/provchain/provchain-org

# Export JWT secret (same as VM-1)
export JWT_SECRET="K7gNU3sdo+OL0wNhqoVWhr3g6s1xYv72ol/pe/Unols="

# Start node
docker compose -f deploy/docker-compose.node.yml up -d

# Check logs
docker logs -f provchain-node2

# Should see:
# - "Connected to peer 10.0.1.10:8080"
# - "Blockchain sync started"
# - "Blockchain synchronized"

# Verify
curl http://localhost:8080/health
curl http://localhost:8080/api/peers | jq 'length'
# Expected: 1 (connected to VM-1)
```

#### Deploy Regular Node 3 (VM-3)

```bash
# Same as VM-2

cd /opt/provchain/provchain-org
export JWT_SECRET="K7gNU3sdo+OL0wNhqoVWhr3g6s1xYv72ol/pe/Unols="
docker compose -f deploy/docker-compose.node.yml up -d

# Verify
curl http://localhost:8080/api/peers | jq 'length'
# Expected: 2 (connected to VM-1 and VM-2)
```

### Step 3.8: Verify Cluster

**From any node:**

```bash
# Check all nodes are healthy
for ip in 10.0.1.10 10.0.1.11 10.0.1.12; do
  echo "Node $ip:"
  curl -s http://$ip:8080/health | jq '.status'
done

# Expected: "healthy" for all three

# Check peer connections (from authority)
curl http://10.0.1.10:8080/api/peers | jq '. | length'
# Expected: 2

# Check blockchain sync
for ip in 10.0.1.10 10.0.1.11 10.0.1.12; do
  echo "Node $ip:"
  curl -s http://$ip:8080/api/blockchain/dump | jq 'length'
done

# All nodes should show same length (at least 1 for genesis block)
```

**Congratulations!** Your 3-node Docker Compose cluster is running. Proceed to [Section 6](#6-post-installation-validation).

---

## 4. Kubernetes Deployment

### Step 4.1: Prerequisites

- Kubernetes cluster (v1.24+) with at least 3 nodes
- kubectl configured and authenticated
- StorageClass available for persistent volumes
- NGINX Ingress Controller (optional, for external access)

### Step 4.2: Create Namespace

```bash
kubectl create namespace provchain

# Set as default namespace for convenience
kubectl config set-context --current --namespace=provchain
```

### Step 4.3: Create Secrets

```bash
# Generate JWT secret
JWT_SECRET=$(openssl rand -base64 32)

# Create Kubernetes secret
kubectl create secret generic provchain-secrets \
  --from-literal=jwt-secret="$JWT_SECRET" \
  -n provchain

# Generate authority keypair (on local machine)
docker run --rm -v $(pwd):/app provchain-org:latest \
  provchain-org generate-keypair --output /app/authority.key

# Create secret for authority key
kubectl create secret generic provchain-authority-key \
  --from-file=authority.key=./authority.key \
  -n provchain

# Extract public key
docker run --rm -v $(pwd):/app provchain-org:latest \
  provchain-org show-pubkey --key /app/authority.key > authority.pub

AUTHORITY_PUBKEY=$(cat authority.pub)
```

### Step 4.4: Create ConfigMaps

```bash
# Create ConfigMap with node configurations
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: ConfigMap
metadata:
  name: provchain-config
  namespace: provchain
data:
  node1.toml: |
    node_id = "authority-node-1"
    [network]
    network_id = "provchain-prod"
    listen_port = 8080
    known_peers = ["provchain-node-1.provchain-svc:8080", "provchain-node-2.provchain-svc:8080"]
    [consensus]
    is_authority = true
    authority_key_file = "/app/keys/authority.key"
    authority_keys = ["$AUTHORITY_PUBKEY"]
    block_interval = 10
    [storage]
    data_dir = "/app/data"
    cache_size_mb = 200
    [web]
    host = "0.0.0.0"
    port = 8080
    [logging]
    level = "info"
    format = "json"

  node2.toml: |
    node_id = "regular-node-2"
    [network]
    network_id = "provchain-prod"
    listen_port = 8080
    known_peers = ["provchain-node-0.provchain-svc:8080", "provchain-node-2.provchain-svc:8080"]
    [consensus]
    is_authority = false
    authority_keys = ["$AUTHORITY_PUBKEY"]
    [storage]
    data_dir = "/app/data"
    cache_size_mb = 100
    [web]
    host = "0.0.0.0"
    port = 8080
    [logging]
    level = "info"
    format = "json"

  node3.toml: |
    node_id = "regular-node-3"
    [network]
    network_id = "provchain-prod"
    listen_port = 8080
    known_peers = ["provchain-node-0.provchain-svc:8080", "provchain-node-1.provchain-svc:8080"]
    [consensus]
    is_authority = false
    authority_keys = ["$AUTHORITY_PUBKEY"]
    [storage]
    data_dir = "/app/data"
    cache_size_mb = 100
    [web]
    host = "0.0.0.0"
    port = 8080
    [logging]
    level = "info"
    format = "json"
EOF
```

### Step 4.5: Create StatefulSet

```bash
cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: provchain-node
  namespace: provchain
spec:
  serviceName: provchain-svc
  replicas: 3
  selector:
    matchLabels:
      app: provchain
  template:
    metadata:
      labels:
        app: provchain
    spec:
      containers:
      - name: provchain
        image: provchain-org:v1.0.0
        ports:
        - containerPort: 8080
          name: api
        - containerPort: 9090
          name: metrics
        env:
        - name: RUST_LOG
          value: "info"
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: provchain-secrets
              key: jwt-secret
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: CONFIG_FILE
          value: "/app/config/\$(POD_NAME).toml"
        volumeMounts:
        - name: data
          mountPath: /app/data
        - name: config
          mountPath: /app/config
        - name: authority-key
          mountPath: /app/keys
          readOnly: true
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 2
        resources:
          requests:
            cpu: "1000m"
            memory: "2Gi"
          limits:
            cpu: "2000m"
            memory: "4Gi"
      volumes:
      - name: config
        configMap:
          name: provchain-config
      - name: authority-key
        secret:
          secretName: provchain-authority-key
          defaultMode: 0400
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: "standard"  # Change to your StorageClass
      resources:
        requests:
          storage: 50Gi
EOF
```

### Step 4.6: Create Services

```bash
# Headless service for StatefulSet
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Service
metadata:
  name: provchain-svc
  namespace: provchain
spec:
  clusterIP: None
  selector:
    app: provchain
  ports:
  - port: 8080
    name: api
  - port: 9090
    name: metrics
EOF

# LoadBalancer service for external access
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Service
metadata:
  name: provchain-api
  namespace: provchain
spec:
  type: LoadBalancer
  selector:
    app: provchain
  ports:
  - port: 8080
    targetPort: 8080
    name: api
  - port: 9090
    targetPort: 9090
    name: metrics
EOF
```

### Step 4.7: Verify Deployment

```bash
# Check pods
kubectl get pods -n provchain
# Expected: 3 pods running (provchain-node-0, provchain-node-1, provchain-node-2)

# Check logs
kubectl logs -f provchain-node-0 -n provchain

# Check services
kubectl get svc -n provchain

# Get external IP
kubectl get svc provchain-api -n provchain
# Note the EXTERNAL-IP

# Test health
EXTERNAL_IP=$(kubectl get svc provchain-api -n provchain -o jsonpath='{.status.loadBalancer.ingress[0].ip}')
curl http://$EXTERNAL_IP:8080/health
```

**Congratulations!** Your Kubernetes cluster is running. Proceed to [Section 6](#6-post-installation-validation).

---

## 5. Native Systemd Deployment

### Step 5.1: Build Binary

**On each VM:**

```bash
cd /opt/provchain/provchain-org

# Build release binary
cargo build --release

# This takes 10-15 minutes on first build

# Verify binary
ls -lh target/release/provchain-org
# Should be ~80 MB

# Test binary
./target/release/provchain-org --version
```

### Step 5.2: Install Binary

```bash
# Copy binary to system location
sudo cp target/release/provchain-org /usr/local/bin/
sudo chmod +x /usr/local/bin/provchain-org

# Verify
which provchain-org
provchain-org --version
```

### Step 5.3: Create Systemd Service

**VM-1 (Authority):**

```bash
sudo cat > /etc/systemd/system/provchain.service <<EOF
[Unit]
Description=ProvChain Blockchain Node (Authority)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=provchain
Group=provchain
WorkingDirectory=/opt/provchain
Environment="RUST_LOG=info"
Environment="JWT_SECRET=K7gNU3sdo+OL0wNhqoVWhr3g6s1xYv72ol/pe/Unols="
Environment="CONFIG_FILE=/opt/provchain/provchain-org/config/node1-authority.toml"
ExecStart=/usr/local/bin/provchain-org --config \${CONFIG_FILE}
Restart=on-failure
RestartSec=10
LimitNOFILE=65535
StandardOutput=journal
StandardError=journal
SyslogIdentifier=provchain

[Install]
WantedBy=multi-user.target
EOF
```

**VM-2 & VM-3 (Regular):**

```bash
# Same as above, but change:
# - Description: "ProvChain Blockchain Node (Regular)"
# - CONFIG_FILE path to node2-regular.toml or node3-regular.toml
```

### Step 5.4: Start Services

**On all VMs:**

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable service (start on boot)
sudo systemctl enable provchain

# Start service
sudo systemctl start provchain

# Check status
sudo systemctl status provchain

# View logs
journalctl -u provchain -f
```

### Step 5.5: Verify

```bash
# Check service is running
systemctl is-active provchain
# Expected: active

# Test health
curl http://localhost:8080/health

# Check from authority node
curl http://10.0.1.10:8080/api/peers | jq '.'
```

**Congratulations!** Your systemd cluster is running. Proceed to [Section 6](#6-post-installation-validation).

---

## 6. Post-Installation Validation

### Validation Phase 1: Health Checks (5 min)

```bash
#!/bin/bash
# Save as: validate_phase1.sh

NODES=("10.0.1.10" "10.0.1.11" "10.0.1.12")
echo "=== Phase 1: Health & Connectivity ==="

for node in "${NODES[@]}"; do
  echo -n "Testing $node... "
  if curl -sf http://$node:8080/health | grep -q "healthy"; then
    echo "✓ PASS"
  else
    echo "✗ FAIL"
    exit 1
  fi
done

echo ""
echo -n "Checking peer connectivity... "
peers=$(curl -s http://10.0.1.10:8080/api/peers | jq 'length')
if [ "$peers" -eq 2 ]; then
  echo "✓ PASS ($peers/2 peers)"
else
  echo "✗ FAIL (found $peers, expected 2)"
  exit 1
fi

echo ""
echo "Phase 1: ALL TESTS PASSED ✓"
```

Run it:
```bash
chmod +x validate_phase1.sh
./validate_phase1.sh
```

### Validation Phase 2: Blockchain Operations (10 min)

```bash
#!/bin/bash
# Save as: validate_phase2.sh

AUTHORITY="10.0.1.10"
NODES=("10.0.1.10" "10.0.1.11" "10.0.1.12")

echo "=== Phase 2: Blockchain Operations ==="

# Record initial lengths
echo "Recording initial blockchain lengths..."
for node in "${NODES[@]}"; do
  length=$(curl -s http://$node:8080/api/blockchain/dump | jq 'length')
  echo "  Node $node: $length blocks"
done

# Submit test transaction
echo ""
echo "Submitting test transaction..."
tx_response=$(curl -s -X POST http://$AUTHORITY:8080/api/transactions \
  -H "Content-Type: application/json" \
  -d '{"data":"<http://test> <http://validation> \"phase2\" ."}')

if echo "$tx_response" | grep -q "accepted"; then
  echo "✓ Transaction accepted"
else
  echo "✗ Transaction rejected"
  echo "Response: $tx_response"
  exit 1
fi

# Wait for block creation
echo ""
echo "Waiting for block creation (15 seconds)..."
sleep 15

# Verify blockchain sync
echo ""
echo "Verifying blockchain synchronization..."
declare -a new_lengths
for i in "${!NODES[@]}"; do
  node="${NODES[$i]}"
  length=$(curl -s http://$node:8080/api/blockchain/dump | jq 'length')
  new_lengths[$i]=$length
  echo "  Node $node: $length blocks"
done

# Check all lengths match
if [ "${new_lengths[0]}" -eq "${new_lengths[1]}" ] && \
   [ "${new_lengths[1]}" -eq "${new_lengths[2]}" ]; then
  echo "✓ All nodes synchronized (${new_lengths[0]} blocks)"
else
  echo "✗ Blockchain sync mismatch"
  exit 1
fi

# Validate blockchain integrity
echo ""
echo "Validating blockchain integrity..."
for node in "${NODES[@]}"; do
  valid=$(curl -s http://$node:8080/api/blockchain/validate | jq -r '.valid')
  if [ "$valid" == "true" ]; then
    echo "✓ Node $node: valid"
  else
    echo "✗ Node $node: INVALID"
    exit 1
  fi
done

echo ""
echo "Phase 2: ALL TESTS PASSED ✓"
```

Run it:
```bash
chmod +x validate_phase2.sh
./validate_phase2.sh
```

### Validation Phase 3: Consensus (15 min)

```bash
#!/bin/bash
# Save as: validate_phase3.sh

AUTHORITY="10.0.1.10"

echo "=== Phase 3: Consensus Validation ==="

# Check authority status
echo "Verifying authority node..."
is_authority=$(curl -s http://$AUTHORITY:8080/api/consensus/stats | jq -r '.is_authority')
if [ "$is_authority" == "true" ]; then
  echo "✓ Authority node operational"
else
  echo "✗ Authority not detected"
  exit 1
fi

# Monitor round progression
echo ""
echo "Monitoring consensus rounds (30 seconds)..."
round1=$(curl -s http://$AUTHORITY:8080/api/consensus/stats | jq '.current_round')
echo "  Initial round: $round1"

sleep 15
round2=$(curl -s http://$AUTHORITY:8080/api/consensus/stats | jq '.current_round')
echo "  After 15s: $round2"

sleep 15
round3=$(curl -s http://$AUTHORITY:8080/api/consensus/stats | jq '.current_round')
echo "  After 30s: $round3"

if [ "$round3" -gt "$round1" ]; then
  echo "✓ Consensus rounds progressing ($round1 → $round3)"
else
  echo "✗ Consensus rounds not progressing"
  exit 1
fi

# Submit multiple transactions
echo ""
echo "Submitting 5 rapid transactions..."
for i in {1..5}; do
  curl -s -X POST http://$AUTHORITY:8080/api/transactions \
    -H "Content-Type: application/json" \
    -d "{\"data\":\"<http://test/$i> <http://prop> \\\"$i\\\" .\"}" > /dev/null &
done
wait

echo "✓ All transactions submitted"

# Wait for processing
echo "Waiting for transaction processing (20 seconds)..."
sleep 20

# Verify all nodes still synchronized
echo ""
echo "Verifying synchronization after load..."
declare -a lengths
for node in 10.0.1.10 10.0.1.11 10.0.1.12; do
  length=$(curl -s http://$node:8080/api/blockchain/dump | jq 'length')
  lengths+=($length)
  echo "  Node $node: $length blocks"
done

if [ "${lengths[0]}" -eq "${lengths[1]}" ] && [ "${lengths[1]}" -eq "${lengths[2]}" ]; then
  echo "✓ All nodes still synchronized"
else
  echo "✗ Nodes out of sync after load"
  exit 1
fi

echo ""
echo "Phase 3: ALL TESTS PASSED ✓"
```

Run it:
```bash
chmod +x validate_phase3.sh
./validate_phase3.sh
```

### Full Validation Suite

Run all phases:

```bash
./validate_phase1.sh && \
./validate_phase2.sh && \
./validate_phase3.sh

# If all pass:
echo ""
echo "======================================="
echo "  ALL VALIDATION TESTS PASSED ✓"
echo "  Your 3-node cluster is operational!"
echo "======================================="
```

---

## 7. Monitoring Setup

### Deploy Monitoring Stack (Docker Compose)

**On VM-1 (Authority Node):**

```bash
cd /opt/provchain/provchain-org

# Deploy monitoring services
docker compose -f deploy/docker-compose.monitoring.yml up -d

# This starts:
# - Prometheus (port 9091)
# - Grafana (port 3001)
# - Jaeger (port 16686)
```

### Access Monitoring UIs

1. **Grafana**: `http://10.0.1.10:3001`
   - Default login: `admin` / `admin`
   - Change password on first login

2. **Prometheus**: `http://10.0.1.10:9091`
   - No authentication by default

3. **Jaeger**: `http://10.0.1.10:16686`
   - No authentication by default

### Import Grafana Dashboards

1. Open Grafana: `http://10.0.1.10:3001`
2. Login (admin/admin)
3. Go to **Dashboards** → **Import**
4. Upload dashboard JSON from `deploy/monitoring/grafana-dashboards/`
5. Or use dashboard IDs:
   - **Node Exporter Full**: `1860`
   - **Docker Prometheus Monitoring**: `893`

### Set Up Alerts

Edit `deploy/monitoring/prometheus.yml`:

```yaml
# Add alerting rules
rule_files:
  - "alerts.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

Create `deploy/monitoring/alerts.yml`:

```yaml
groups:
  - name: provchain
    interval: 30s
    rules:
      - alert: NodeDown
        expr: up{job="provchain"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Node {{ $labels.instance }} is down"

      - alert: ConsensusStalled
        expr: rate(provchain_blocks_total[5m]) == 0
        for: 3m
        labels:
          severity: critical
        annotations:
          summary: "No blocks created in 5 minutes"
```

---

## 8. Troubleshooting

### Issue: Nodes not connecting

**Symptoms:**
- `curl http://10.0.1.10:8080/api/peers` returns empty array

**Check:**
```bash
# Test network connectivity
ping 10.0.1.11

# Test port is open
telnet 10.0.1.11 8080

# Check firewall
sudo ufw status
sudo iptables -L -n | grep 8080

# Check known_peers in config
grep known_peers config/*.toml
```

**Fix:**
```bash
# Open port 8080
sudo ufw allow 8080/tcp

# Restart node
docker compose restart
# or
sudo systemctl restart provchain
```

### Issue: Blockchain not syncing

**Symptoms:**
- Different blockchain lengths across nodes

**Check:**
```bash
# Get blockchain hashes
for ip in 10.0.1.10 10.0.1.11 10.0.1.12; do
  echo "Node $ip:"
  curl -s http://$ip:8080/api/blockchain/dump | jq -r '.[-1].hash'
done

# Check logs for errors
docker logs provchain-node1 | grep -i error
# or
journalctl -u provchain | grep -i error
```

**Fix:**
```bash
# Resync from authority
docker compose down
rm -rf data/*
docker compose up -d

# Node will resync from peers automatically
```

### Issue: Out of memory

**Symptoms:**
- Node crashes
- `docker stats` shows high memory usage

**Fix:**
```bash
# Reduce cache size in config
[storage]
cache_size_mb = 50  # Reduce from 100

# Limit Docker memory
# In docker-compose.yml:
services:
  provchain:
    mem_limit: 2g

# Restart
docker compose down && docker compose up -d
```

### Getting Help

- **Logs**: Always check logs first
  - Docker: `docker logs provchain-node1`
  - Systemd: `journalctl -u provchain -n 100`

- **GitHub Issues**: https://github.com/yourusername/provchain-org/issues

- **Comprehensive Analysis Report**: See `/home/cit/Documents/provchain-deployment/COMPREHENSIVE_ANALYSIS_REPORT.md` for detailed troubleshooting guide

---

## Summary

You've successfully deployed a 3-node ProvChain cluster!

**What you deployed:**
- ✅ 1 Authority node (creates blocks)
- ✅ 2 Regular nodes (validate and replicate)
- ✅ Proof-of-Authority consensus
- ✅ WebSocket P2P networking
- ✅ Monitoring stack (optional)

**Next steps:**
1. Run validation tests (Section 6)
2. Set up monitoring (Section 7)
3. Configure automated backups
4. Set up TLS/SSL for production
5. Implement alerting

**Useful commands:**
```bash
# Check cluster health
curl http://10.0.1.10:8080/health

# View logs
docker logs -f provchain-node1

# Submit transaction
curl -X POST http://10.0.1.10:8080/api/transactions \
  -H "Content-Type: application/json" \
  -d '{"data":"<http://test> <http://p> \"v\" ."}'

# Check blockchain
curl http://10.0.1.10:8080/api/blockchain/dump | jq '.'
```

**Support:**
- Comprehensive docs: `COMPREHENSIVE_ANALYSIS_REPORT.md`
- GitHub: https://github.com/yourusername/provchain-org
- Issues: https://github.com/yourusername/provchain-org/issues

---

**Congratulations on your deployment!** 🎉
