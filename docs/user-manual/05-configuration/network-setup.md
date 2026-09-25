# Experimental Network Configuration Reference

**Legacy/target examples for inspecting ProvChain-Org peer and container scaffolding**

> **Non-operational reference — not convergence evidence (2026-08-31).** The examples below are
> retained to document historical configuration intent. They are not a verified current runbook,
> and named environment variables, API routes, Compose commands, and expected outputs must be
> checked against the current revision before use. The repository has not yet demonstrated
> authenticated membership, scheduled-authority acceptance, exact complete-envelope replication,
> matching three-node journal prefixes, partition/rejoin recovery, or crash-safe convergence.
> PBFT and operational deployment are future work. Do not infer data replication, availability,
> finality, or production readiness from process startup or peer visibility.

---

## What You'll Learn

In this guide, you will:
- Inspect the intended peer-to-peer configuration model
- Identify single-process development settings
- Read historical multi-process examples without treating them as verified convergence
- Distinguish peer visibility from authenticated membership and replicated commitment
- Locate current source-of-truth architecture and evidence requirements

**Prerequisites**: Complete [10-Minute Setup](../00-quick-start/10-minute-setup.md)

---

## Understanding ProvChain Networking

### Peer-to-Peer Architecture

ProvChain contains **peer-to-peer (P2P) and WebSocket scaffolding**. Current code presence supports
development experiments only:

- process startup does not establish an equal-peer governance model;
- a visible connection does not authenticate consortium membership;
- data replication across all nodes has not been demonstrated; and
- no-single-point-of-failure or recovery property is currently claimed.

```
Node A ←→ Node B ←→ Node C
   ↑         ↑         ↑
   └─────────┴─────────┘
   Target topology only
```

### Bootstrap vs Regular Nodes

| Node Type | PEERS Setting | Role |
|-----------|---------------|------|
| **Bootstrap** | Empty (`PEERS=""`) | Historical configuration role; verify current loader support |
| **Regular** | List of peers (`PEERS="node1:8080,node2:8080"`) | Historical peer-discovery intent; not authenticated membership |

These roles are configuration examples, not a validated operating procedure.

## Historical Examples Below

Everything below this heading is retained as legacy/target material. Commands and “expected”
results do not represent a current passing campaign. The acceptance test for the thesis reference
system is the reproducible three-process exact-envelope convergence/recovery campaign defined by
the shared-ontology working plan.

---

## Single Node Configuration

### Environment Variables

Configure your node using environment variables:

```bash
# .env file
RUST_LOG=info                          # Log level (debug, info, warn, error)
JWT_SECRET=your-secret-key-here        # JWT authentication secret
PEERS=""                               # Comma-separated peer addresses (empty for bootstrap)
PROVCHAIN_PORT=8080                    # P2P/API port
JAEGER_ENDPOINT=                       # Optional: Jaeger tracing endpoint
```

### Docker Compose Example

```yaml
version: '3.8'

services:
  provchain-node:
    image: anusornc/provchain-org:latest
    container_name: provchain-node
    restart: unless-stopped
    ports:
      - "8080:8080"   # API & P2P
      - "9090:9090"   # Metrics
    environment:
      - RUST_LOG=info
      - JWT_SECRET=${JWT_SECRET}
      - PEERS=${PEERS}
      - PROVCHAIN_PORT=8080
    volumes:
      - ./node_data:/app/data
      - ./node_logs:/app/logs
```

### Start a Single Node

```bash
# Create .env file
cat > .env <<EOF
RUST_LOG=info
JWT_SECRET=$(openssl rand -base64 32)
PEERS=""
EOF

# Start the node
docker compose up -d
```

---

## Multi-Node Network Setup

### Scenario 1: Three Nodes on One Machine

**Use Case**: Testing, development, small deployments

**Architecture**:
```
┌─────────────────────────────────────┐
│         Single Machine              │
│                                     │
│  ┌──────────┐  ┌──────────┐        │
│  │ Node 1   │  │ Node 2   │        │
│  │ :8080    │←→│ :8081    │        │
│  │ Bootstrap│  │ Regular   │        │
│  └──────────┘  └──────────┘        │
│       ↓            ↓                │
│  ┌──────────┐                     │
│  │ Node 3   │                     │
│  │ :8082    │                     │
│  │ Regular  │                     │
│  └──────────┘                     │
└─────────────────────────────────────┘
```

**docker-compose.yml**:

```yaml
version: '3.8'

services:
  # Node 1: Bootstrap (starts the network)
  node1:
    image: anusornc/provchain-org:latest
    container_name: provchain-node1
    restart: unless-stopped
    ports:
      - "8080:8080"
      - "9090:9090"
    environment:
      - RUST_LOG=info
      - JWT_SECRET=${JWT_SECRET}
      - PEERS=                           # Empty = Bootstrap
      - OTEL_SERVICE_NAME=provchain-node1
    volumes:
      - node1_data:/app/data

  # Node 2: Regular (connects to Node 1)
  node2:
    image: anusornc/provchain-org:latest
    container_name: provchain-node2
    restart: unless-stopped
    ports:
      - "8081:8080"
    environment:
      - RUST_LOG=info
      - JWT_SECRET=${JWT_SECRET}
      - PEERS=node1:8080                 # Connect to bootstrap
      - OTEL_SERVICE_NAME=provchain-node2
    volumes:
      - node2_data:/app/data

  # Node 3: Regular (connects to both)
  node3:
    image: anusornc/provchain-org:latest
    container_name: provchain-node3
    restart: unless-stopped
    ports:
      - "8082:8080"
    environment:
      - RUST_LOG=info
      - JWT_SECRET=${JWT_SECRET}
      - PEERS=node1:8080,node2:8080      # Connect to both
      - OTEL_SERVICE_NAME=provchain-node3
    volumes:
      - node3_data:/app/data

volumes:
  node1_data:
  node2_data:
  node3_data:
```

**Start the network**:

```bash
# Create .env
echo "JWT_SECRET=$(openssl rand -base64 32)" > .env

# Start all nodes
docker compose up -d

# Verify
curl http://localhost:8080/api/peers | jq .
```

---

### Scenario 2: Three Nodes on Separate Machines

**Historical target scenario**: Geographic-distribution example only; this is not an
operational deployment or verified convergence procedure.

**Architecture**:
```
┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│  Machine 1       │     │  Machine 2       │     │  Machine 3       │
│  IP: 192.168.1.10│     │  IP: 192.168.1.11│     │  IP: 192.168.1.12│
│                  │     │                  │     │                  │
│  ┌────────────┐  │     │  ┌────────────┐  │     │  ┌────────────┐  │
│  │  Node 1    │  │     │  │  Node 2    │  │     │  │  Node 3    │  │
│  │  Bootstrap │  │     │  │  Regular   │  │     │  │  Regular   │  │
│  │  PEERS=""  │  │     │  │  PEERS=    │  │     │  │  PEERS=    │  │
│  └────────────┘  │     │  │  192.168.. │  │     │  │  192.168.. │  │
│  Port: 8080      │     │  └────────────┘  │     │  └────────────┘  │
└──────────────────┘     └──────────────────┘     └──────────────────┘
         │                        │                        │
         └────────────────────────┴────────────────────────┘
                           Network (P2P)
```

**Step 1: Configure Node 1 (Bootstrap)**

On Machine 1 (192.168.1.10):

```bash
# Clone repository
git clone https://github.com/anusornc/prov-chain.git
cd provchain-org2/deploy

# Create .env
cat > .env <<EOF
RUST_LOG=info
JWT_SECRET=$(openssl rand -base64 32)
PEERS=                           # Empty = Bootstrap
PROVCHAIN_PORT=8080
EOF

# Start Node 1
docker compose -f docker-compose.node.yml up -d

# Get IP address
hostname -I | awk '{print $1}'  # Should be 192.168.1.10
```

**Wait 30 seconds** for Node 1 to fully initialize.

**Step 2: Configure Node 2**

On Machine 2 (192.168.1.11):

```bash
# Clone repository
git clone https://github.com/anusornc/prov-chain.git
cd provchain-org2/deploy

# Create .env (connect to Node 1)
cat > .env <<EOF
RUST_LOG=info
JWT_SECRET=$(openssl rand -base64 32)
PEERS=192.168.1.10:8080          # Connect to Node 1
PROVCHAIN_PORT=8080
EOF

# Start Node 2
docker compose -f docker-compose.node.yml up -d
```

**Step 3: Configure Node 3**

On Machine 3 (192.168.1.12):

```bash
# Clone repository
git clone https://github.com/anusornc/prov-chain.git
cd provchain-org2/deploy

# Create .env (connect to both Node 1 and Node 2)
cat > .env <<EOF
RUST_LOG=info
JWT_SECRET=$(openssl rand -base64 32)
PEERS=192.168.1.10:8080,192.168.1.11:8080   # Connect to both
PROVCHAIN_PORT=8080
EOF

# Start Node 3
docker compose -f docker-compose.node.yml up -d
```

**Step 4: Verify the Network**

From any machine:

```bash
# Check peers from Node 1
curl http://192.168.1.10:8080/api/peers | jq .

# Check peers from Node 2
curl http://192.168.1.11:8080/api/peers | jq .

# Check blockchain sync
curl http://192.168.1.10:8080/api/blockchain/status | jq .
curl http://192.168.1.11:8080/api/blockchain/status | jq .
curl http://192.168.1.12:8080/api/blockchain/status | jq .
```

**Expected**: All nodes show the same `block_count` and `latest_block_hash`.

---

## Configuration Reference

### PEERS Environment Variable

The `PEERS` variable specifies which nodes to connect to.

**Format**: Comma-separated list of `host:port` pairs

**Examples**:

```bash
# Bootstrap node (no peers)
PEERS=""

# Connect to one peer
PEERS="192.168.1.10:8080"

# Connect to multiple peers
PEERS="192.168.1.10:8080,192.168.1.11:8080,192.168.1.12:8080"

# Connect using DNS names
PEERS="node1.example.com:8080,node2.example.com:8080"

# Connect using Docker service names
PEERS="node1:8080,node2:8080,node3:8080"
```

### Other Network Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PROVCHAIN_PORT` | `8080` | Port for API and P2P communication |
| `PROVCHAIN_BIND_ADDRESS` | `0.0.0.0` | Address to bind to (all interfaces) |
| `RUST_LOG` | `info` | Logging level |
| `JAEGER_ENDPOINT` | (empty) | Optional Jaeger tracing endpoint |

---

## Managing Peer Connections

### View Connected Peers

```bash
# Via API
curl http://localhost:8080/api/peers | jq .

# Expected output
{
  "peers": [
    {
      "id": "peer-id-1",
      "address": "192.168.1.11:8080",
      "status": "connected",
      "last_seen": "2025-01-04T10:30:00Z"
    },
    {
      "id": "peer-id-2",
      "address": "192.168.1.12:8080",
      "status": "connected",
      "last_seen": "2025-01-04T10:30:05Z"
    }
  ],
  "peer_count": 2
}
```

### Add a New Peer to Existing Node

**Option 1: Restart with new PEERS value**

```bash
# Stop the node
docker compose down

# Update .env with new peer
echo 'PEERS="node1:8080,node2:8080,new-node:8080"' >> .env

# Restart
docker compose up -d
```

**Option 2: Edit docker-compose.yml and restart**

```yaml
environment:
  - PEERS=node1:8080,node2:8080,new-node:8080
```

### Remove a Peer

Update `PEERS` to exclude the peer and restart:

```bash
# Update .env
echo 'PEERS="node1:8080"' > .env

# Restart
docker compose restart
```

---

## Network Verification

### Health Check Script

Save as `verify-network.sh`:

```bash
#!/bin/bash

NODE1="192.168.1.10:8080"
NODE2="192.168.1.11:8080"
NODE3="192.168.1.12:8080"

echo "=== ProvChain Network Verification ==="
echo ""

# Test 1: All nodes accessible
echo "Test 1: Node Accessibility"
for node in $NODE1 $NODE2 $NODE3; do
    if curl -sf http://$node/health > /dev/null; then
        echo "  ✓ $node: Healthy"
    else
        echo "  ✗ $node: Unreachable"
    fi
done
echo ""

# Test 2: Peer connectivity
echo "Test 2: Peer Connectivity"
peers=$(curl -s http://$NODE1/api/peers | jq '.peer_count')
echo "  Bootstrap node sees $peers peer(s)"
if [ "$peers" -eq 2 ]; then
    echo "  ✓ All peers connected"
else
    echo "  ✗ Expected 2 peers, found $peers"
fi
echo ""

# Test 3: Blockchain synchronization
echo "Test 3: Blockchain Synchronization"
length1=$(curl -s http://$NODE1/api/blockchain/dump | jq 'length')
length2=$(curl -s http://$NODE2/api/blockchain/dump | jq 'length')
length3=$(curl -s http://$NODE3/api/blockchain/dump | jq 'length')

echo "  Node 1: $length1 blocks"
echo "  Node 2: $length2 blocks"
echo "  Node 3: $length3 blocks"

if [ "$length1" -eq "$length2" ] && [ "$length2" -eq "$length3" ]; then
    echo "  ✓ Reported block counts match (not convergence proof)"
else
    echo "  ✗ Blockchain sync mismatch"
fi
echo ""

echo "=== Verification Complete ==="
```

Run it:
```bash
chmod +x verify-network.sh
./verify-network.sh
```

---

## Firewall Configuration

### Required Ports

| Port | Protocol | Purpose |
|------|----------|---------|
| `8080` | TCP | API & P2P communication |
| `9090` | TCP | Metrics (Prometheus) |

### Open Ports (UFW)

```bash
# On all nodes
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 8080/tcp  # ProvChain API/P2P
sudo ufw allow 9090/tcp  # Metrics
sudo ufw enable

# Verify
sudo ufw status
```

### Open Ports (firewalld)

```bash
# On all nodes
sudo firewall-cmd --permanent --add-port=8080/tcp
sudo firewall-cmd --permanent --add-port=9090/tcp
sudo firewall-cmd --reload

# Verify
sudo firewall-cmd --list-ports
```

---

## Troubleshooting

### Nodes Cannot Connect

**Symptoms**:
- `curl /api/peers` returns empty array `{"peers": [], "peer_count": 0}`
- Nodes show different block counts

**Diagnosis**:
```bash
# Test network connectivity
ping 192.168.1.11

# Test port accessibility
telnet 192.168.1.11 8080

# Check firewall
sudo ufw status
# OR
sudo firewall-cmd --list-all
```

**Solutions**:

1. **Check PEERS configuration**:
   ```bash
   docker exec provchain-node printenv | grep PEERS
   ```

2. **Verify ports are open**:
   ```bash
   sudo ufw allow 8080/tcp
   ```

3. **Restart nodes**:
   ```bash
   docker compose restart
   ```

4. **Check logs**:
   ```bash
   docker logs provchain-node
   ```

### Blockchain Not Syncing

**Symptoms**:
- Different blockchain lengths across nodes
- Transactions not appearing on all nodes

**Solutions**:

1. **Restart lagging node**:
   ```bash
   docker compose restart
   ```

2. **Force resync** (last resort):
   ```bash
   docker compose down
   rm -rf node_data/*
   docker compose up -d
   ```

### High Memory Usage

**Symptoms**:
- Container crashes due to OOM (Out of Memory)

**Solutions**:

Add memory limit in `docker-compose.yml`:

```yaml
services:
  provchain-node:
    deploy:
      resources:
        limits:
          memory: 2G
```

---

## Historical Experiment Checklist

1. **Start with bootstrap node** - Always have at least one node with `PEERS=""`
2. **Use stable test addresses** - Keep experiment endpoints deterministic
3. **Configure firewall** - Only open necessary ports
4. **Treat block counts as diagnostic only** - Matching counts do not prove exact-envelope convergence
5. **Preserve experiment data** - Do not assume each node has a complete durable copy
6. **Use DNS names** - Easier than IPs for peer configuration
7. **Test in single-machine mode first** - Verify scaffolding before a multi-machine experiment

---

## Historical Target Scenarios

### NAT/Firewall Traversal

If nodes are behind NAT, use:

```bash
# Expose public port
docker run -p 8080:8080 -e PROVCHAIN_PUBLIC_ADDRESS=public-ip:8080 ...
```

### TLS/SSL Encryption Target

The historical target placed a reverse proxy in front of a node. This snippet is not evidence of
authenticated consortium membership or a verified secure deployment:

```nginx
# nginx.conf
stream {
    server {
        listen 443 ssl;
        proxy_pass provchain-node:8080;
        
        ssl_certificate /etc/nginx/ssl/cert.pem;
        ssl_certificate_key /etc/nginx/ssl/key.pem;
    }
}
```

### Docker Swarm Target

This unverified orchestration sketch is retained for historical context only:

```yaml
version: '3.8'

services:
  provchain-node:
    image: anusornc/provchain-org:latest
    deploy:
      mode: replicated
      replicas: 3
    environment:
      - PEERS=tasks.provchain-node:8080
    networks:
      - provchain-net

networks:
  provchain-net:
    driver: overlay
```

---

## Next Steps

- 📖 [System Administration](../06-system-administration/) - Monitoring and maintenance
- 🔧 [Configuration Reference](config-file-reference.md) - All configuration options
- 🆘 [Troubleshooting](../08-troubleshooting/) - Solve common issues

---

*Last updated: 2025-01-04*
*Version: 1.0.0*
