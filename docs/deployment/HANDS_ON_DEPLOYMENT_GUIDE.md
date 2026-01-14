# ProvChain-Org: Hands-On Deployment Guide

**Step-by-Step Guide to Deploy Your ProvChain-Org Blockchain**

**Time Required:** 30-60 minutes
**Difficulty Level:** Beginner-Friendly
**Last Updated:** 2026-01-03

---

## Table of Contents

1. [Before You Start](#1-before-you-start)
2. [Choose Your Deployment Type](#2-choose-your-deployment-type)
3. [Deployment Paths](#3-deployment-paths)
   - [Path A: Single-Node (Quickest)](#path-a-single-node-quickest)
   - [Path B: 3-Node Cluster (One Machine)](#path-b-3-node-cluster-one-machine)
   - [Path C: Multi-Machine Distributed](#path-c-multi-machine-distributed)
4. [Post-Deployment Verification](#4-post-deployment-verification)
5. [Common First Operations](#5-common-first-operations)
6. [Troubleshooting](#6-troubleshooting)

---

## 1. Before You Start

### Check Your System

**Step 1.1: Open a terminal**

- **Linux:** Press `Ctrl + Alt + T`
- **macOS:** Open Terminal from Applications
- **Windows:** Open PowerShell or WSL

**Step 1.2: Check available disk space**

```bash
df -h .
```

**Expected output:** You should have at least 20 GB available.

```
Filesystem      Size  Used Avail Use% Mounted on
/dev/sda1       100G   60G   40G  60% /
```

✅ **If you have less than 20 GB:** Free up disk space before continuing.

---

**Step 1.3: Check if Docker is installed**

```bash
docker --version
```

**If Docker is installed, you'll see:**
```
Docker version 24.0.7, build afdd53b
```

❌ **If you see "command not found":** Install Docker first.

```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Add your user to docker group
sudo usermod -aG docker $USER

# IMPORTANT: Log out and log back in for group changes to take effect
# Or run: newgrp docker
```

---

**Step 1.4: Check Docker Compose**

```bash
docker compose version
```

**Expected output:**
```
Docker Compose version v2.23.0
```

---

**Step 1.5: Verify you can run Docker**

```bash
docker run hello-world
```

**Expected output:**
```
Hello from Docker!
This message shows that your installation appears to be working correctly.
```

✅ **If you see this message:** Docker is ready!
❌ **If you see a permission error:** Log out and log back in, then try again.

---

## 2. Choose Your Deployment Type

Read through all three options and choose the one that matches your needs:

### 📊 Decision Matrix

| I want to... | Use this Path | Time | Machines |
|--------------|---------------|------|----------|
| **Learn and experiment** | Path A: Single-Node | 10 min | 1 |
| **Test consensus behavior** | Path B: 3-Node (1 machine) | 20 min | 1 |
| **Run production cluster** | Path C: Multi-Machine | 1-2 hours | 3+ |

---

### Path A: Single-Node (Quickest) ⭐ Recommended for Beginners

**Best for:**
- First-time users
- Learning the system
- Development and testing
- Quick demonstrations

**What you'll get:**
- 1 ProvChain node
- Monitoring (Prometheus, Grafana, Jaeger)
- All services running on one machine

**Skip to:** [Path A: Single-Node](#path-a-single-node-quickest)

---

### Path B: 3-Node Cluster (One Machine)

**Best for:**
- Testing blockchain consensus
- Learning how nodes communicate
- Staging environment
- Development clustering

**What you'll get:**
- 3 ProvChain nodes on one machine
- All nodes communicate with each other
- Monitoring stack

**Skip to:** [Path B: 3-Node Cluster](#path-b-3-node-cluster-one-machine)

---

### Path C: Multi-Machine Distributed

**Best for:**
- Production deployment
- High availability
- Real distributed network testing

**What you'll get:**
- 3 ProvChain nodes on separate machines
- Distributed network with real peer discovery
- Centralized monitoring

**Skip to:** [Path C: Multi-Machine](#path-c-multi-machine-distributed)

---

## 3. Deployment Paths

---

## Path A: Single-Node (Quickest)

**Total Time:** ~10 minutes
**Machines Needed:** 1

### STEP A1: Download the Project

**Step A1.1: Navigate to your workspace**

```bash
cd ~
```

**Step A1.2: Clone the repository**

```bash
git clone https://github.com/anusornc/provchain-org2.git
```

**Expected output:**
```
Cloning into 'provchain-org'...
remote: Enumerating objects: xxxx, done.
remote: Total xxxx (delta 0), reused 0 (delta 0), pack-reused 0
Receiving objects: 100% (xxxx/xxxx), 50.00 MiB | 20.00 MiB/s, done.
```

---

**Step A1.3: Enter the project directory**

```bash
cd provchain-org
```

**Step A1.4: Verify project structure**

```bash
ls -la
```

**Expected output (you should see these key items):**
```
drwxr-xr-x  src/
drwxr-xr-x  deploy/
drwxr-xr-x  config/
-rw-r--r--  Cargo.toml
-rw-r--r--  Dockerfile.production
```

---

**Step A1.5: Navigate to deploy directory**

```bash
cd deploy
```

✅ **Checkpoint:** You should now be in `~/provchain-org/deploy`

---

### STEP A2: Generate Secrets

**Step A2.1: Generate a JWT secret**

```bash
export JWT_SECRET=$(openssl rand -base64 32)
echo "Your JWT_SECRET is: $JWT_SECRET"
```

**Expected output:**
```
Your JWT_SECRET is: K7gNU3sdo+OL0wNhqoVWhr3g6s1xYv72ol/pe/Unols=
```

⚠️ **IMPORTANT:** Save this secret somewhere safe! You'll need it if you restart.

---

**Step A2.2: (Optional) Set Grafana password**

```bash
export GRAFANA_PASSWORD=MySecurePassword123!
```

---

### STEP A3: Start the Application

**Step A3.1: Start all services**

```bash
docker compose -f docker-compose.production.yml up -d --build
```

**This command will:**
- Download Docker images (if not cached)
- Build the ProvChain application (may take 5-10 minutes)
- Start all containers in the background

**Expected output:**
```
[+] Building 145.3s (18/18) FINISHED
 => [internal] load build definition
 => => transferring dockerfile:
 => [internal] load .dockerignore
 => ...
[+] Running 10/10
 ✔ Network provchain_network         Created
 ✔ Volume "provchain_data"          Created
 ✔ Volume "provchain_logs"          Created
 ✔ Container provchain-prometheus   Started
 ✔ Container provchain-grafana      Started
 ✔ Container provchain-org          Started
 ✔ Container provchain-jaeger       Started
 ✔ Container provchain-nginx        Started
 ✔ Container provchain-redis        Started
```

---

**Step A3.2: Wait for containers to be healthy**

```bash
# Watch container status
docker ps
```

**Expected output (wait until all show "Up" and "healthy"):**
```
CONTAINER ID   IMAGE                     STATUS                   PORTS
abc123def     provchain-org:latest      Up 2 minutes (healthy)    0.0.0.0:8080->8080, 0.0.0.0:9090->9090
def456ghi     prometheus:v2.45.0        Up 2 minutes              0.0.0.0:9091->9090
ghi789jkl     grafana:10.0.0            Up 2 minutes              0.0.0.0:3001->3000
```

⏳ **If STATUS shows "health: starting":** Wait another 30 seconds and check again.

---

### STEP A4: Verify Deployment

**Step A4.1: Check application health**

```bash
curl http://localhost:8080/health
```

**Expected output:**
```json
{"status":"healthy","version":"0.1.0","timestamp":"2026-01-03T12:00:00Z"}
```

✅ **Success!** Your node is running.

❌ **If you get "Connection refused":** See [Troubleshooting](#6-troubleshooting)

---

**Step A4.2: Check your services**

```bash
# Test each service
echo "Testing ProvChain API..."
curl -s http://localhost:8080/health | jq .

echo "Testing Prometheus..."
curl -s http://localhost:9091/-/healthy

echo "Testing Grafana..."
curl -s http://localhost:3001/api/health
```

---

**Step A4.3: View your running containers**

```bash
docker compose -f docker-compose.production.yml ps
```

**You should see 8-10 containers running.**

---

### STEP A5: Access Your Services

**Step A5.1: Open your web browser**

Navigate to these URLs:

| Service | URL | Login |
|---------|-----|-------|
| **ProvChain API** | http://localhost:8080/health | No login |
| **Grafana** | http://localhost:3001 | admin / YourPassword |
| **Prometheus** | http://localhost:9090 | No login |
| **Jaeger Tracing** | http://localhost:16686 | No login |

---

**Step A5.2: Configure Grafana (First Time Only)**

1. Open http://localhost:3001
2. Login with `admin` / the password you set
3. Click "Add your first data source"
4. Select "Prometheus"
5. Set URL to: `http://prometheus:9090`
6. Click "Save & Test"
7. You should see "Successfully connected to Prometheus"

---

### 🎉 Congratulations!

**You now have a running ProvChain-Org node!**

**Next Steps:**
- [ ] Submit your first transaction (see [Common Operations](#5-common-first-operations))
- [ ] Explore the Grafana dashboards
- [ ] Check out the API documentation

**For production deployment:** Consider Path B or Path C for better fault tolerance.

---

## Path B: 3-Node Cluster (One Machine)

**Total Time:** ~20 minutes
**Machines Needed:** 1

### STEP B1: Download the Project

**Complete steps A1.1 through A1.5 from Path A**

You should end up in the `~/provchain-org/deploy` directory.

---

### STEP B2: Launch 3-Node Cluster

**Step B2.1: Start the 3-node cluster**

```bash
docker compose -f docker-compose.3node.yml up -d --build
```

**Expected output:**
```
[+] Building 145.3s (18/18) FINISHED
[+] Running 6/6
 ✔ Container provchain-node1   Started
 ✔ Container provchain-node2   Started
 ✔ Container provchain-node3   Started
 ✔ Container provchain-prometheus   Started
 ✔ Container provchain-grafana      Started
 ✔ Container provchain-jaeger       Started
```

---

**Step B2.2: Wait for nodes to start**

```bash
# Watch all containers
docker ps | grep provchain
```

**Expected output:**
```
CONTAINER ID   IMAGE                 STATUS
abc123def     provchain-org:latest   Up 1 minute (healthy)
def456ghi     provchain-org:latest   Up 1 minute (healthy)
ghi789jkl     provchain-org:latest   Up 1 minute (healthy)
```

---

### STEP B3: Verify Cluster Formation

**Step B3.1: Check node 1 (bootstrap node)**

```bash
curl http://localhost:8080/health
```

**Expected output:**
```json
{"status":"healthy","node_id":"provchain-node1",...}
```

---

**Step B3.2: Check peer connections**

```bash
curl http://localhost:8080/api/peers | jq .
```

**Expected output:**
```json
[
  {"node_id":"provchain-node2","address":"node2:8080","connected":true},
  {"node_id":"provchain-node3","address":"node3:8080","connected":true}
]
```

✅ **Success!** Node 1 sees 2 peers.

---

**Step B3.3: Check all nodes**

```bash
echo "Node 1 blockchain length:"
curl -s http://localhost:8080/api/blockchain/dump | jq 'length'

echo "Node 2 blockchain length:"
curl -s http://localhost:8081/api/blockchain/dump | jq 'length'

echo "Node 3 blockchain length:"
curl -s http://localhost:8082/api/blockchain/dump | jq 'length'
```

**Expected output:** All three nodes should show the same number.

```
Node 1 blockchain length:
1
Node 2 blockchain length:
1
Node 3 blockchain length:
1
```

✅ **All nodes synchronized!**

---

### STEP B4: Access Monitoring

**Step B4.1: Open Grafana**

Navigate to: http://localhost:3001

Login: `admin` / `admin` (change on first login)

---

**Step B4.2: Add Prometheus datasource**

1. Click **Configuration** → **Data Sources**
2. Click **Add data source**
3. Select **Prometheus**
4. URL: `http://prometheus:9090`
5. Click **Save & Test**

---

**Step B4.3: View metrics**

In Prometheus (http://localhost:9091), run this query:

```
up{job="provchain"}
```

You should see 3 results (one for each node).

---

### 🎉 Congratulations!

**You now have a 3-node blockchain cluster running on one machine!**

**What makes this different from Path A:**
- 3 independent nodes communicating
- Peer discovery working
- Blockchain synchronization across nodes
- Real consensus behavior

**Test the consensus:**
Try stopping one node and see if the others keep working!

```bash
docker stop provchain-node2
# Node 1 and 3 should continue operating
docker start provchain-node2
# Node 2 will rejoin and sync
```

---

## Path C: Multi-Machine Distributed

**Total Time:** 1-2 hours
**Machines Needed:** 3 (minimum)

### Prerequisites Checklist

Before starting, ensure you have:

- [ ] 3 machines (physical or virtual)
- [ ] All machines can network with each other
- [ ] Docker installed on all machines
- [ ] SSH access to all machines
- [ ] Know the IP addresses of all machines

---

### STEP C1: Prepare All Machines

**Repeat these steps on ALL 3 machines:**

**Step C1.1: Install Docker**

```bash
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER
```

Then log out and log back in.

---

**Step C1.2: Clone the repository**

```bash
cd ~
git clone https://github.com/anusornc/provchain-org2.git
cd provchain-org/deploy
```

---

**Step C1.3: Find your IP address**

```bash
hostname -I | awk '{print $1}'
```

**Write down each machine's IP:**
- Machine 1 (Bootstrap): `_____________`
- Machine 2: `_____________`
- Machine 3: `_____________`

---

### STEP C2: Configure Machine 1 (Bootstrap Node)

**Step C2.1: SSH into Machine 1**

```bash
ssh user@MACHINE1_IP
cd ~/provchain-org/deploy
```

---

**Step C2.2: Create environment file**

```bash
cat > .env <<EOF
# Bootstrap Node Configuration
PEERS=""
JWT_SECRET=$(openssl rand -base64 32)
RUST_LOG=info
EOF
```

---

**Step C2.3: Verify the file**

```bash
cat .env
```

**You should see:**
```
# Bootstrap Node Configuration
PEERS=""
JWT_SECRET=SomeRandomString...
RUST_LOG=info
```

---

**Step C2.4: Start the bootstrap node**

```bash
docker compose -f docker-compose.node.yml up -d --build
```

**Wait for the build to complete (5-10 minutes).**

---

**Step C2.5: Verify the node is running**

```bash
docker ps
curl http://localhost:8080/health
```

**Expected output:**
```json
{"status":"healthy",...}
```

---

**Step C2.6: Note the bootstrap node IP**

```bash
echo "Bootstrap node IP: $(hostname -I | awk '{print $1}')"
```

**Write this down:** Bootstrap IP: `_____________`

⏳ **Wait 30 seconds** for the bootstrap node to fully initialize before proceeding.

---

### STEP C3: Configure Machine 2

**Step C3.1: SSH into Machine 2**

```bash
ssh user@MACHINE2_IP
cd ~/provchain-org/deploy
```

---

**Step C3.2: Create environment file**

```bash
# REPLACE with actual Bootstrap IP from Step C2.6
cat > .env <<EOF
# Node 2 Configuration
PEERS="BOOTSTRAP_IP:8080"
JWT_SECRET=$(openssl rand -base64 32)
RUST_LOG=info
EOF
```

**Example:**
```bash
PEERS="192.168.1.101:8080"
```

---

**Step C3.3: Start the node**

```bash
docker compose -f docker-compose.node.yml up -d --build
```

---

**Step C3.4: Verify the node**

```bash
docker ps
curl http://localhost:8080/health
```

---

**Step C3.5: Check if connected to bootstrap**

```bash
docker logs provchain-node | grep -i peer
```

**Look for lines like:**
```
Connected to peer 192.168.1.101:8080
Peer connection established
```

---

### STEP C4: Configure Machine 3

**Step C4.1: SSH into Machine 3**

```bash
ssh user@MACHINE3_IP
cd ~/provchain-org/deploy
```

---

**Step C4.2: Create environment file**

```bash
# REPLACE with actual IPs from Steps C2.6 and C3.1
cat > .env <<EOF
# Node 3 Configuration
PEERS="BOOTSTRAP_IP:8080,NODE2_IP:8080"
JWT_SECRET=$(openssl rand -base64 32)
RUST_LOG=info
EOF
```

**Example:**
```bash
PEERS="192.168.1.101:8080,192.168.1.102:8080"
```

---

**Step C4.3: Start the node**

```bash
docker compose -f docker-compose.node.yml up -d --build
```

---

**Step C4.4: Verify the node**

```bash
docker ps
curl http://localhost:8080/health
```

---

### STEP C5: Verify Cluster Formation

**Run these from Machine 1 (Bootstrap):**

**Step C5.1: Check peer connections**

```bash
curl http://localhost:8080/api/peers | jq .
```

**Expected output:**
```json
[
  {"node_id":"provchain-node2","address":"192.168.1.102:8080","connected":true},
  {"node_id":"provchain-node3","address":"192.168.1.103:8080","connected":true}
]
```

✅ **Success!** Bootstrap sees 2 peers.

---

**Step C5.2: Check blockchain sync**

```bash
# From Machine 1
curl -s http://localhost:8080/api/blockchain/dump | jq 'length'

# From Machine 2
ssh user@MACHINE2_IP 'curl -s http://localhost:8080/api/blockchain/dump | jq length'

# From Machine 3
ssh user@MACHINE3_IP 'curl -s http://localhost:8080/api/blockchain/dump | jq length'
```

**All three should return the same number.**

---

### STEP C6: Deploy Monitoring (Optional)

**Step C6.1: On Machine 1, create Prometheus config**

```bash
cat > monitoring/prometheus_multi_node.yml <<EOF
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'provchain-nodes'
    static_configs:
      - targets:
          - 'MACHINE1_IP:9090'
          - 'MACHINE2_IP:9090'
          - 'MACHINE3_IP:9090'
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
EOF
```

**Replace with actual IPs.**

---

**Step C6.2: Start monitoring**

```bash
docker compose -f docker-compose.monitoring.yml up -d
```

---

**Step C6.3: Access monitoring**

- Grafana: http://MACHINE1_IP:3001
- Prometheus: http://MACHINE1_IP:9090
- Jaeger: http://MACHINE1_IP:16686

---

### 🎉 Congratulations!

**You now have a production-ready multi-machine blockchain cluster!**

**What you have:**
- 3 independent nodes on separate machines
- Real peer discovery across network
- Blockchain synchronization
- Centralized monitoring
- Fault tolerance (can lose 1 node and keep running)

---

## 4. Post-Deployment Verification

### Complete Verification Checklist

**Run these checks after ANY deployment:**

---

### ✅ Check 1: All Containers Running

```bash
docker ps
```

**Verify:**
- All containers show "Up" status
- No containers show "Exited" or "Restarting"

---

### ✅ Check 2: Application Health

```bash
curl http://localhost:8080/health
```

**Expected:**
```json
{"status":"healthy"}
```

---

### ✅ Check 3: Metrics Available

```bash
curl http://localhost:9090/metrics
```

**Expected:** Long text output with metrics (starting with `# HELP` or `go_`)

---

### ✅ Check 4: No Critical Errors in Logs

```bash
docker logs provchain-org 2>&1 | grep -i error | tail -20
```

**Expected:** No output (or only non-critical warnings)

---

### ✅ Check 5: Blockchain Valid

```bash
curl http://localhost:8080/api/blockchain/validate | jq .
```

**Expected:**
```json
{"valid":true}
```

---

### ✅ Check 6: Monitoring Accessible

```bash
# Test Prometheus
curl -s http://localhost:9091/-/healthy

# Test Grafana
curl -s http://localhost:3001/api/health
```

**Both should return success.**

---

### 🎯 All Checks Passed?

If all 6 checks pass, your deployment is successful!

**Next:** Try submitting a transaction (see below).

---

## 5. Common First Operations

### Submit Your First Transaction

**Step 1: Generate a test token**

```bash
# For testing only - use proper auth in production
TEST_TOKEN="test-token-$(date +%s)"
```

---

**Step 2: Submit a transaction**

```bash
curl -X POST http://localhost:8080/api/transactions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TEST_TOKEN" \
  -d '{
    "data": "<http://example.org/product1> <http://example.org/hasName> \"Test Product\" ."
  }'
```

**Expected response:**
```json
{"status":"accepted","tx_id":"0x123abc..."}
```

---

**Step 3: Wait for block creation**

```bash
echo "Waiting 15 seconds for block creation..."
sleep 15
```

---

**Step 4: Check if transaction was included**

```bash
curl http://localhost:8080/api/blockchain/dump | jq '.[-1]'
```

**You should see your transaction in the latest block.**

---

### View Your Blockchain

```bash
# View entire blockchain
curl http://localhost:8080/api/blockchain/dump | jq .

# View blockchain length
curl http://localhost:8080/api/blockchain/dump | jq 'length'

# View specific block
curl http://localhost:8080/api/blockchain/dump | jq '.[0]'
```

---

### Explore Grafana Dashboards

**Step 1: Open Grafana**

http://localhost:3001

**Step 2: Login**
- Username: `admin`
- Password: (what you set)

---

**Step 3: Import a dashboard**

1. Click **+** → **Import**
2. Enter dashboard ID: `1860` (Node Exporter Full)
3. Click **Load**
4. Select **Prometheus** as datasource
5. Click **Import**

---

**Step 4: Explore metrics**

Try these queries in Prometheus (http://localhost:9091):

```
# Number of blocks
rate(provchain_blocks_total[5m])

# Peer connections
provchain_peers_connected

# Memory usage
process_resident_memory_bytes
```

---

## 6. Troubleshooting

### Problem: Container won't start

**Symptoms:**
```bash
docker ps
# Shows no containers or "Exited" status
```

**Diagnosis:**
```bash
# Check logs
docker logs provchain-org

# Check if ports are in use
sudo lsof -i :8080
```

**Solutions:**

1. **Port already in use:**
```bash
# Kill the process using the port
sudo kill -9 $(sudo lsof -t -i:8080)

# Restart
docker compose -f docker-compose.production.yml up -d
```

2. **Out of disk space:**
```bash
# Check disk space
df -h

# Clean up Docker
docker system prune -a

# Restart
docker compose -f docker-compose.production.yml up -d
```

3. **Permission denied:**
```bash
# Ensure you're in the docker group
groups

# If not, add yourself and re-login
sudo usermod -aG docker $USER
# Log out and log back in
```

---

### Problem: Health check fails

**Symptoms:**
```bash
curl http://localhost:8080/health
curl: (7) Failed to connect to localhost port 8080: Connection refused
```

**Diagnosis:**
```bash
# Check if container is running
docker ps | grep provchain

# Check logs for errors
docker logs provchain-org --tail 50
```

**Solutions:**

1. **Container not running:**
```bash
# Start it
docker compose -f docker-compose.production.yml up -d
```

2. **Container crashing:**
```bash
# View crash logs
docker logs provchain-org

# Look for specific error messages and search online
# Common issues: invalid config, missing files, out of memory
```

3. **Waiting period:**
```bash
# Health checks may need time
# Wait 30 seconds and try again
sleep 30
curl http://localhost:8080/health
```

---

### Problem: Nodes can't connect (Multi-node)

**Symptoms:**
```bash
curl http://localhost:8080/api/peers
[]  # Empty array
```

**Diagnosis:**
```bash
# Check network connectivity
ping NODE2_IP

# Check if port is accessible
telnet NODE2_IP 8080

# Check firewall
sudo ufw status
```

**Solutions:**

1. **Firewall blocking:**
```bash
# Open ports
sudo ufw allow 8080/tcp
sudo ufw allow 9090/tcp

# Retry
```

2. **Wrong IP in config:**
```bash
# Verify IP addresses
hostname -I

# Update .env file with correct IPs
nano .env

# Restart
docker compose -f docker-compose.node.yml restart
```

3. **Network not reachable:**
```bash
# Test connectivity
ping NODE2_IP
traceroute NODE2_IP

# Check if both nodes on same network
ip addr show
```

---

### Problem: Out of memory

**Symptoms:**
```bash
docker stats
# Shows high memory usage
```

**Container exits unexpectedly.**

**Solutions:**

1. **Reduce cache size:**
```bash
# Edit config file
nano config/production-deployment.toml

# Change:
[storage]
cache_size_mb = 50  # Reduce from 200

# Restart
docker compose restart
```

2. **Add memory limit:**
```bash
# Edit docker-compose.yml
nano docker-compose.production.yml

# Add to service:
services:
  provchain-org:
    mem_limit: 2g

# Restart
docker compose -f docker-compose.production.yml up -d
```

3. **Free system memory:**
```bash
# Clear system cache
sudo sync && sudo sysctl -w vm.drop_caches=3

# Stop unnecessary services
# (depending on your system)
```

---

### Problem: Can't access Grafana/Prometheus

**Symptoms:**
- Browser shows "Connection refused"
- Services seem to be running

**Diagnosis:**
```bash
# Check if service is running
docker ps | grep grafana

# Check logs
docker logs provchain-grafana
```

**Solutions:**

1. **Wrong port:**
```bash
# Grafana is on 3001, not 3000
http://localhost:3001  # Correct
http://localhost:3000  # Wrong
```

2. **Service not ready:**
```bash
# Wait longer
# Some services take 1-2 minutes to start

# Check logs for progress
docker logs -f provchain-grafana
```

3. **Default credentials:**
```
Username: admin
Password: admin  # Change on first login
```

---

## Getting Help

If you're still stuck:

1. **Check the logs:**
```bash
docker logs provchain-org
docker logs provchain-prometheus
docker logs provchain-grafana
```

2. **Check container status:**
```bash
docker ps -a
```

3. **Search the documentation:**
- `/docs/deployment/DOCKER_DEPLOYMENT_ARCHITECTURE.md`
- `/docs/deployment/SETUP_INSTALLATION_GUIDE.md`

4. **Open an issue:**
https://github.com/anusornc/provchain-org2/issues

**When reporting issues, include:**
- Output of `docker ps`
- Output of `docker logs provchain-org`
- Your deployment type (A, B, or C)
- Your OS and Docker version

---

## Quick Reference Cards

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

# Submit transaction
curl -X POST http://localhost:8080/api/transactions \
  -H "Content-Type: application/json" \
  -d '{"data":"<s> <p> \"o\" ."}'

# View blockchain
curl http://localhost:8080/api/blockchain/dump | jq .
```

---

### Port Reference

| Port | Service | Access |
|------|---------|--------|
| 8080 | ProvChain API | http://localhost:8080 |
| 9090 | Metrics | http://localhost:9090/metrics |
| 9091 | Prometheus | http://localhost:9091 |
| 3001 | Grafana | http://localhost:3001 |
| 16686 | Jaeger | http://localhost:16686 |

---

## Success! 🎉

**You've successfully deployed ProvChain-Org!**

**What's Next?**
- [ ] Read the API documentation
- [ ] Explore the example applications
- [ ] Set up automated backups
- [ ] Configure alerts in Grafana
- [ ] Join the community discussions

---

**Thank you for using ProvChain-Org!**

`★ Insight ─────────────────────────────────────`
**Deployment Best Practices**
1. Always verify each step before proceeding—catch issues early
2. Save your secrets (JWT_SECRET) securely—you'll need them for restarts
3. Start with Path A for learning, then progress to Path B or C for production
`─────────────────────────────────────────────────`
