# 10-Minute Quick Start Guide

**Get ProvChain-Org running in 10 minutes with pre-built Docker images**

---

## What You'll Accomplish

In this guide, you will:
- ✅ Start ProvChain-Org using Docker
- ✅ Access the web interface
- ✅ Generate an API key
- ✅ Submit your first data to the blockchain
- ✅ Query your data back

**Time required**: 10 minutes

---

## Prerequisites

Check these boxes before starting:

- [ ] **Docker is installed** on your computer
  - Verify: Run `docker --version` in terminal
  - Don't have Docker? [Install here](https://docs.docker.com/get-docker/)
  
- [ ] **4GB of memory** available on your computer
  
- [ ] **Ports 8080 and 3001** are available
  - Check: `lsof -i :8080` and `lsof -i :3001`
  - If in use, we'll use different ports

- [ ] **Internet connection** (to download Docker image)

---

## Step 1: Pull the Docker Image (2 minutes)

Open your terminal (Command Prompt on Windows, Terminal on Mac/Linux) and run:

```bash
docker pull anusornc/provchain-org:latest
```

**What's happening**: Docker is downloading the ProvChain-Org application (about 94MB).

**Expected output**:
```
latest: Pulling from anusornc/provchain-org
Digest: sha256:abc123...
Status: Downloaded newer image for anusornc/provchain-org:latest
```

**Troubleshooting**:
- If you get "permission denied", try: `sudo docker pull anusornc/provchain-org:latest`
- If download is slow, be patient - it depends on your internet speed

---

## Step 2: Start ProvChain (1 minute)

Run this command to start ProvChain:

```bash
docker run -d \
  --name provchain-org \
  -p 8080:8080 \
  -p 9090:9090 \
  -e JWT_SECRET="$(openssl rand -base64 32)" \
  anusornc/provchain-org:latest
```

**What's happening**:
- `-d` - Run in background (detached mode)
- `--name provchain-org` - Name the container
- `-p 8080:8080` - Map port 8080 (API)
- `-p 9090:9090` - Map port 9090 (metrics)
- `-e JWT_SECRET` - Generate secure secret for authentication

**Expected output**: A long container ID like `a1b2c3d4e5f6...`

**Troubleshooting**:
- **Port already in use?** Use different ports:
  ```bash
  docker run -d --name provchain-org -p 8081:8080 -p 9091:9090 anusornc/provchain-org:latest
  ```
  Then use port 8081 instead of 8080 in all examples.

- **OpenSSL error?** You're on Windows. Use a fixed secret:
  ```bash
  docker run -d --name provchain-org -p 8080:8080 -e JWT_SECRET="my-secret-key-change-in-production" anusornc/provchain-org:latest
  ```

---

## Step 3: Verify ProvChain is Running (1 minute)

Wait 30-60 seconds for the application to start, then check:

```bash
# Check container is running
docker ps | grep provchain-org

# Check health status
curl http://localhost:8080/health
```

**Expected output**:
```json
{"status":"healthy","version":"1.0.0"}
```

**Troubleshooting**:
- **Container not running?** Check logs: `docker logs provchain-org`
- **Health check fails?** Wait a bit longer and try again: `curl http://localhost:8080/health`
- **Connection refused?** Make sure you're using the right port (8081 if you changed it)

---

## Step 4: Access the Web Interface (1 minute)

Open your web browser and go to:

```
http://localhost:8080
```

You should see the ProvChain-Org dashboard.

**What you'll see**:
- Welcome screen with system overview
- Blockchain status (blocks, transactions)
- Quick action buttons
- Navigation menu

**Can't access?**
- Make sure the container is running: `docker ps`
- Check the port: If you used 8081, go to `http://localhost:8081`
- Check firewall settings

---

## Step 5: Generate API Key (1 minute)

You need an API key to submit data.

### Option A: Using Web Interface (Recommended)

1. Click **"API Keys"** in the menu
2. Click **"Generate New Key"**
3. Copy the key (save it securely - you won't see it again!)

### Option B: Using Command Line

```bash
# Login to get token
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}' \
  | jq -r '.token'
```

**Expected output**: A long JWT token string

**Save this token** - you'll need it for the next steps!

---

## Step 6: Submit Your First Data (3 minutes)

Now let's add your first supply chain data to the blockchain.

### What We're Submitting

A product batch record representing:
- **Batch**: TOMATO-2025-001
- **Product**: Organic Tomatoes
- **Origin**: Green Valley Farm
- **Date**: Today's date
- **Quantity**: 500 kg

### Submit via API

Replace `YOUR_API_TOKEN` with the token from Step 5:

```bash
TOKEN="YOUR_API_TOKEN"

curl -X POST http://localhost:8080/api/blockchain/add-triple \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "subject": "http://example.org/batch/TOMATO-2025-001",
    "predicate": "http://example.org/ns#productType",
    "object": "Organic Tomatoes"
  }'
```

**Expected response**:
```json
{
  "success": true,
  "block_index": 1,
  "block_hash": "0x8f3e2d1c...",
  "message": "Triple added successfully"
}
```

**Congratulations!** You just submitted your first data to the blockchain!

### Submit More Data

Let's add more information about this batch:

```bash
# Origin farm
curl -X POST http://localhost:8080/api/blockchain/add-triple \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "subject": "http://example.org/batch/TOMATO-2025-001",
    "predicate": "http://example.org/ns#originFarm",
    "object": "Green Valley Farm"
  }'

# Harvest date
curl -X POST http://localhost:8080/api/blockchain/add-triple \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "subject": "http://example.org/batch/TOMATO-2025-001",
    "predicate": "http://example.org/ns#harvestDate",
    "object": "2025-01-04"
  }'

# Quantity
curl -X POST http://localhost:8080/api/blockchain/add-triple \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "subject": "http://example.org/batch/TOMATO-2025-001",
    "predicate": "http://example.org/ns#quantity",
    "object": "500"
  }'
```

---

## Step 7: Query Your Data (1 minute)

Now let's retrieve the data you just submitted.

```bash
# View blockchain status
curl http://localhost:8080/api/blockchain/status | jq .

# View recent transactions
curl http://localhost:8080/api/transactions/recent | jq .

# View all blocks
curl http://localhost:8080/api/blockchain/dump | jq .
```

**Expected output**: JSON data showing the blocks and transactions you just created!

---

## Verification Checklist

Make sure everything is working:

- [ ] Container is running: `docker ps | grep provchain-org`
- [ ] Health check passes: `curl http://localhost:8080/health`
- [ ] Web interface loads: http://localhost:8080
- [ ] API token works: Use it in a command
- [ ] Data submitted: You got "success": true response
- [ ] Data retrieved: Query returns your data

**All boxes checked?** You're ready to use ProvChain!

---

## What's Next?

### Learn More

- 📖 [Your First Transaction](first-transaction.md) - Detailed transaction submission guide
- 🔍 [Your First Query](first-query.md) - Learn to query blockchain data
- 📝 [Submitting Product Batches](../02-submitting-data/product-batches.md) - Record harvest/production data
- 🔎 [Query Library](../03-querying-data/query-library.md) - Ready-to-use SPARQL queries

### Common Next Steps

1. **Submit more data** - Add activities, environmental data, quality checks
2. **Query your data** - Use SPARQL to analyze supply chain information
3. **Configure peers** - Connect multiple nodes for a network
4. **Set up monitoring** - Track system health and performance

---

## Stopping ProvChain

When you're done, you can stop the container:

```bash
# Stop the container
docker stop provchain-org

# Start it again later
docker start provchain-org

# Remove container (careful - data in container will be lost!)
docker rm provchain-org
```

**To keep your data**, use Docker volumes:

```bash
docker run -d \
  --name provchain-org \
  -p 8080:8080 \
  -p 9090:9090 \
  -v provchain_data:/app/data \
  -e JWT_SECRET="$(openssl rand -base64 32)" \
  anusornc/provchain-org:latest
```

---

## Common Issues

### "Port 8080 already in use"

**Solution**: Use a different port
```bash
docker run -d --name provchain-org -p 8081:8080 anusornc/provchain-org:latest
```

Then use `http://localhost:8081` instead.

### "Connection refused"

**Cause**: Container isn't running yet

**Solution**: Wait 30-60 seconds, then check:
```bash
docker logs provchain-org
```

Look for "Server listening on 0.0.0.0:8080" message.

### "Unauthorized" error

**Cause**: Missing or invalid API token

**Solution**: 
1. Get a fresh token: See Step 5
2. Make sure you include `Authorization: Bearer $TOKEN` header
3. Check the token isn't expired

### "Invalid RDF syntax" error

**Cause**: Malformed data in your request

**Solution**: 
- Check JSON syntax (use a JSON validator)
- Make sure strings are properly quoted
- Verify subject/predicate/object format

---

## Getting Help

Still stuck? Check these resources:

- 📚 [Troubleshooting Guide](../08-troubleshooting/)
- 🐛 [GitHub Issues](https://github.com/anusornc/provchain-org2/issues)
- 📖 [Full User Manual](../README.md)

---

**Congratulations!** 🎉 You've successfully set up ProvChain-Org and submitted your first blockchain data. You're now ready to explore the full capabilities of the platform!

*Next recommended: [Your First Transaction](first-transaction.md)*
