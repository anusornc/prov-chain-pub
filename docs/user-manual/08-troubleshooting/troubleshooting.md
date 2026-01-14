# Troubleshooting Guide

**Solve common problems with ProvChain-Org**

---

## How to Use This Guide

1. **Identify your problem** from the categories below
2. **Follow the diagnostic steps**
3. **Try the suggested solutions** in order
4. **Verify the fix** worked
5. **Still stuck?** See [Getting Help](#getting-help)

---

## Quick Diagnostics

### Health Check Script

Run this first to diagnose issues:

```bash
#!/bin/bash

echo "=== ProvChain Health Check ==="

# 1. Container running?
echo -n "Container running: "
if docker ps | grep -q provchain-org; then
    echo "✓ Yes"
else
    echo "✗ No"
    echo "  → Start container: docker start provchain-org"
fi

# 2. Port accessible?
echo -n "Port 8080 accessible: "
if curl -sf http://localhost:8080/health > /dev/null; then
    echo "✓ Yes"
else
    echo "✗ No"
    echo "  → Check port: lsof -i :8080"
fi

# 3. API responding?
echo -n "API responding: "
if curl -sf http://localhost:8080/health | grep -q "healthy"; then
    echo "✓ Yes"
else
    echo "✗ No"
    echo "  → Check logs: docker logs provchain-org"
fi

# 4. Blockchain valid?
echo -n "Blockchain valid: "
VALID=$(curl -s http://localhost:8080/api/blockchain/validate | jq '.is_valid // false')
if [ "$VALID" = "true" ]; then
    echo "✓ Yes"
else
    echo "✗ No"
    echo "  → Blockchain may be corrupted"
fi

echo "=== Check Complete ==="
```

---

## Installation & Setup Issues

### "Port 8080 already in use"

**Symptoms**:
```
Error: bind: address already in use
```

**Diagnosis**:
```bash
# Find what's using port 8080
lsof -i :8080
# OR
netstat -tulpn | grep 8080
```

**Solutions**:

1. **Use a different port**:
   ```bash
   docker run -d --name provchain-org -p 8081:8080 anusornc/provchain-org:latest
   ```
   Then use `http://localhost:8081` instead.

2. **Stop the conflicting process**:
   ```bash
   # Kill the process (if safe to do so)
   kill -9 <PID>
   ```

3. **Wait and retry**:
   ```bash
   # Sometimes port takes time to release
   sleep 5
   docker start provchain-org
   ```

---

### "Docker command not found"

**Symptoms**:
```
bash: docker: command not found
```

**Solution**: Install Docker

```bash
# Linux
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# macOS
brew install --cask docker

# Windows
# Download Docker Desktop from https://docker.com/products/docker-desktop
```

---

### "Permission denied while trying to connect to Docker daemon"

**Symptoms**:
```
Got permission denied while trying to connect to the Docker daemon socket
```

**Solutions**:

1. **Use sudo** (temporary):
   ```bash
   sudo docker ps
   ```

2. **Add user to docker group** (permanent):
   ```bash
   sudo usermod -aG docker $USER
   # Log out and back in
   ```

3. **Fix Docker socket permissions**:
   ```bash
   sudo chmod 666 /var/run/docker.sock
   ```

---

### Container exits immediately

**Symptoms**:
```bash
$ docker ps
# (container not listed)

$ docker ps -a
CONTAINER ID   IMAGE                    STATUS                     PORTS
abc123         provchain-org:latest     Exited (1) 5 seconds ago
```

**Diagnosis**:
```bash
# Check exit code and logs
docker logs provchain-org

# Look for errors
docker logs provchain-org | grep -i error
```

**Common causes**:

1. **Missing JWT_SECRET**:
   ```bash
   # Add JWT_SECRET to run command
   docker run -d -e JWT_SECRET="$(openssl rand -base64 32)" ...
   ```

2. **Port conflicts**:
   ```bash
   # Use different ports
   docker run -d -p 8081:8080 -p 9091:9090 ...
   ```

3. **Insufficient memory**:
   ```bash
   # Check available memory
   free -h
   
   # Add memory limit
   docker run -d --memory="2g" ...
   ```

---

## Connection Issues

### "Connection refused" error

**Symptoms**:
```
curl: (7) Failed to connect to localhost port 8080: Connection refused
```

**Diagnosis**:
```bash
# Is container running?
docker ps | grep provchain

# Is port mapped?
docker port provchain-org

# Is process listening inside container?
docker exec provchain-org netstat -tulpn | grep 8080
```

**Solutions**:

1. **Container not started**:
   ```bash
   docker start provchain-org
   # Wait 30 seconds for startup
   ```

2. **Port not mapped**:
   ```bash
   docker run -d -p 8080:8080 anusornc/provchain-org:latest
   ```

3. **Wrong port**:
   ```bash
   # Check what port is actually mapped
   docker port provchain-org
   ```

---

### Cannot access from other machines

**Symptoms**:
- Works on `localhost:8080`
- Doesn't work from `http://192.168.1.10:8080`

**Diagnosis**:
```bash
# Check bind address
docker exec provchain-org printenv | grep BIND

# Check firewall
sudo ufw status
# OR
sudo firewall-cmd --list-all
```

**Solutions**:

1. **Bind to all interfaces**:
   ```bash
   docker run -d -p 0.0.0.0:8080:8080 anusornc/provchain-org:latest
   ```

2. **Open firewall**:
   ```bash
   # UFW
   sudo ufw allow 8080/tcp
   
   # firewalld
   sudo firewall-cmd --permanent --add-port=8080/tcp
   sudo firewall-cmd --reload
   ```

3. **Check network**:
   ```bash
   # Can you ping the machine?
   ping 192.168.1.10
   
   # Is the machine reachable?
   telnet 192.168.1.10 8080
   ```

---

## Authentication Issues

### "Unauthorized" or "401" error

**Symptoms**:
```json
{
  "error": "Unauthorized",
  "message": "Invalid or missing token"
}
```

**Diagnosis**:
```bash
# Is token present?
echo $TOKEN

# Is token expired?
# Decode JWT and check 'exp' claim
echo $TOKEN | cut -d. -f2 | base64 -d | jq .
```

**Solutions**:

1. **Get fresh token**:
   ```bash
   TOKEN=$(curl -s -X POST http://localhost:8080/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username":"admin","password":"admin123"}' \
     | jq -r '.token')
   ```

2. **Include Authorization header**:
   ```bash
   curl -H "Authorization: Bearer $TOKEN" \
     http://localhost:8080/api/blockchain/status
   ```

3. **Check credentials**:
   ```bash
   # Default: admin/admin123
   # Check your configuration for custom credentials
   ```

---

### "Invalid JWT token" error

**Symptoms**:
```json
{
  "error": "invalid_token",
  "message": "Token verification failed"
}
```

**Causes**:
- Token malformed
- Token signed with wrong secret
- Token expired

**Solutions**:

1. **Generate new token**:
   ```bash
   # Login again
   curl -X POST http://localhost:8080/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username":"admin","password":"admin123"}'
   ```

2. **Check JWT_SECRET matches**:
   ```bash
   # Must be same for all nodes in network
   docker exec provchain-node printenv | grep JWT_SECRET
   ```

3. **Extend token expiry** (if configurable):
   ```bash
   # Check configuration file
   grep token_expiry config.toml
   ```

---

## Data Submission Issues

### "Invalid RDF syntax" error

**Symptoms**:
```json
{
  "error": "invalid_rdf",
  "message": "Malformed Turtle data"
}
```

**Diagnosis**:
```bash
# Validate JSON first
echo '{"subject":"..."}' | jq .

# Validate RDF syntax (if you have a validator)
riot --validate your-data.ttl
```

**Solutions**:

1. **Check JSON syntax**:
   - All strings quoted
   - No trailing commas
   - Valid JSON structure

2. **Check RDF format**:
   - Subject must be URI: `http://example.org/batch/001`
   - Predicate must be URI: `http://example.org/ns#property`
   - Object can be URI or literal: `"value"` or `http://...`

3. **Escape quotes**:
   ```json
   {
     "object": "Tomatoes \"Organic\" Brand"
   }
   ```

---

### "Validation failed" error

**Symptoms**:
```json
{
  "error": "validation_failed",
  "message": "Data does not conform to SHACL shapes"
}
```

**Causes**:
- Missing required properties
- Wrong data types
- Constraint violations

**Solutions**:

1. **Check required properties**:
   ```bash
   # Review ontology/SHACL shapes
   cat ontologies/supply-chain-shapes.ttl
   ```

2. **Verify data types**:
   - Dates: `2025-01-04` (ISO 8601)
   - Numbers: `123` or `45.67` (unquoted)
   - Strings: `"text"` (quoted)

3. **Review error details**:
   ```bash
   # Response usually includes specific constraint
   curl ... | jq '.error_details'
   ```

---

### Transaction not appearing in blockchain

**Symptoms**:
- Submit returns success
- Transaction not in `/api/blockchain/dump`
- Block count hasn't increased

**Diagnosis**:
```bash
# Check transaction pool
curl http://localhost:8080/api/transactions/pending | jq .

# Check recent transactions
curl http://localhost:8080/api/transactions/recent | jq .

# Verify blockchain state
curl http://localhost:8080/api/blockchain/status | jq .
```

**Solutions**:

1. **Wait for block creation**:
   ```bash
   # Blocks are created periodically
   # Check again in 30-60 seconds
   ```

2. **Check transaction pool**:
   ```bash
   # Transaction might be pending
   curl http://localhost:8080/api/transactions/pending
   ```

3. **Verify submission**:
   ```bash
   # Resubmit the transaction
   curl -X POST http://localhost:8080/api/blockchain/add-triple ...
   ```

---

## Query Issues

### "Malformed SPARQL query" error

**Symptoms**:
```json
{
  "error": "malformed_query",
  "message": "Invalid SPARQL syntax"
}
```

**Diagnosis**:
```bash
# Validate SPARQL syntax
# Use online SPARQL validator
# Or test with simple query first
```

**Solutions**:

1. **Check basic syntax**:
   - All prefixes declared
   - Braces balanced
   - Semicolons between triple patterns
   - Period at end

2. **Test with simple query**:
   ```sparql
   SELECT * WHERE {
     ?s ?p ?o .
   }
   LIMIT 10
   ```

3. **Build up query gradually**:
   - Start simple
   - Add filters one at a time
   - Test each iteration

---

### Query returns no results

**Symptoms**:
```json
{
  "results": [],
  "count": 0
}
```

**Diagnosis**:
```bash
# Check if data exists
curl http://localhost:8080/api/blockchain/dump | jq '.[] | .triples | length'

# Query for all data first
SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10
```

**Solutions**:

1. **Verify data exists**:
   ```bash
   # Check blockchain has data
   curl http://localhost:8080/api/blockchain/status
   ```

2. **Simplify query**:
   ```sparql
   # Remove filters
   # Remove OPTIONAL clauses
   # Use wildcards
   ```

3. **Check prefixes**:
   ```sparql
   # Make sure prefixes match your data
   PREFIX : <http://example.org/ns#>
   ```

4. **Verify identifiers**:
   ```bash
   # Check exact batch ID format
   curl http://localhost:8080/api/blockchain/dump | jq '.[] | .triples[] | .subject'
   ```

---

### Query is slow

**Symptoms**:
- Query takes > 10 seconds
- Browser times out
- API timeout

**Solutions**:

1. **Add LIMIT**:
   ```sparql
   SELECT ... WHERE { ... }
   LIMIT 100
   ```

2. **Use specific identifiers**:
   ```sparql
   # Instead of: ?s a :Batch .
   # Use: :BATCH-001 a :Batch .
   ```

3. **Filter by date**:
   ```sparql
   FILTER(?timestamp >= "2025-01-01"^^xsd:dateTime)
   ```

4. **Avoid OPTIONAL**:
   ```sparql
   # OPTIONAL slows down queries
   # Use only when necessary
   ```

5. **Index common queries** (if supported):
   ```bash
   # Check configuration for indexing options
   grep index config.toml
   ```

---

## Network Issues

### Nodes not connecting

**Symptoms**:
```bash
curl http://localhost:8080/api/peers
{"peers": [], "peer_count": 0}
```

**Diagnosis**:
```bash
# Check PEERS configuration
docker exec provchain-node printenv | grep PEERS

# Test network connectivity
ping <peer-ip>
telnet <peer-ip> 8080

# Check firewall
sudo ufw status
```

**Solutions**:

1. **Verify PEERS configuration**:
   ```bash
   # Should be: "host:port,host:port"
   echo $PEERS
   ```

2. **Check network connectivity**:
   ```bash
   # Can you reach the peer?
   ping 192.168.1.11
   telnet 192.168.1.11 8080
   ```

3. **Open firewall ports**:
   ```bash
   sudo ufw allow 8080/tcp
   ```

4. **Restart nodes**:
   ```bash
   docker compose restart
   ```

---

### Blockchain out of sync

**Symptoms**:
- Different `block_count` on different nodes
- Transactions not appearing on all nodes
- Peers show different `latest_block_hash`

**Diagnosis**:
```bash
# Check each node
for node in node1 node2 node3; do
  echo "$node:"
  curl -s http://$node:8080/api/blockchain/status | jq '.block_count, .latest_block_hash'
done
```

**Solutions**:

1. **Wait for sync**:
   ```bash
   # New blocks take time to propagate
   # Wait 30-60 seconds
   ```

2. **Check network connectivity**:
   ```bash
   # Nodes must be able to communicate
   curl http://node1:8080/api/peers | jq .
   ```

3. **Restart lagging nodes**:
   ```bash
   docker compose restart node2
   ```

4. **Force resync** (last resort):
   ```bash
   docker compose down
   rm -rf node_data/*
   docker compose up -d
   ```

---

## Performance Issues

### High CPU usage

**Symptoms**:
- Container using 100% CPU
- Slow response times
- System lagging

**Diagnosis**:
```bash
# Check container stats
docker stats provchain-org

# Check process inside container
docker exec provchain-org top
```

**Solutions**:

1. **Limit CPU**:
   ```bash
   docker run --cpus="1.5" ...
   # OR in compose
   deploy:
     resources:
       limits:
         cpus: '1.5'
   ```

2. **Reduce logging**:
   ```bash
   # Change log level
   RUST_LOG=warn  # instead of info or debug
   ```

3. **Check for infinite loops** (developers):
   ```bash
   # Review code for tight loops
   # Add sleep/throttle where appropriate
   ```

---

### High memory usage

**Symptoms**:
- Container using excessive memory
- OOM killer terminates container
- System swap thrashing

**Diagnosis**:
```bash
# Check memory usage
docker stats provchain-org

# Check container limits
docker inspect provchain-org | jq '.[0].HostConfig.Memory'
```

**Solutions**:

1. **Add memory limit**:
   ```bash
   docker run --memory="2g" ...
   # OR in compose
   deploy:
     resources:
       limits:
         memory: 2G
   ```

2. **Increase system memory**:
   ```bash
   # ProvChain needs at least 2GB
   free -h
   ```

3. **Reduce blockchain size**:
   ```bash
   # Archive old blocks (if supported)
   # Or prune (careful - irreversible)
   ```

4. **Restart container**:
   ```bash
   # Memory leak?
   docker restart provchain-org
   ```

---

### Disk space issues

**Symptoms**:
- Disk full
- Cannot write new data
- Container crashes

**Diagnosis**:
```bash
# Check disk usage
df -h

# Check Docker disk usage
docker system df

# Check volume size
docker exec provchain-org du -sh /app/data
```

**Solutions**:

1. **Clean up Docker**:
   ```bash
   docker system prune -a
   ```

2. **Archive old data**:
   ```bash
   # Backup and remove old blocks
   tar czf backup-$(date +%Y%m%d).tar.gz node_data/
   rm -rf node_data/blocks/old/*
   ```

3. **Add more disk**:
   ```bash
   # Mount larger volume
   docker run -v /mnt/large-disk:/app/data ...
   ```

---

## Getting Help

### Before Asking for Help

1. **Check this guide** - Your issue might be covered
2. **Search existing issues** - https://github.com/anusornc/provchain-org2/issues
3. **Gather diagnostic info**:
   ```bash
   # System info
   uname -a
   docker --version
   docker-compose --version
   
   # Container info
   docker ps -a
   docker logs provchain-org --tail 100
   
   # Network info
   curl -s http://localhost:8080/health | jq .
   curl -s http://localhost:8080/api/blockchain/status | jq .
   ```

### When Asking for Help

Include:
- ✅ ProvChain version (`docker images | grep provchain`)
- ✅ OS and version (`uname -a`)
- ✅ Docker version (`docker --version`)
- ✅ Full error message
- ✅ Steps to reproduce
- ✅ What you expected to happen
- ✅ What actually happened
- ✅ Logs (`docker logs provchain-org --tail 100`)

### Resources

- 📖 [Full User Manual](../README.md)
- 🐛 [GitHub Issues](https://github.com/anusornc/provchain-org2/issues)
- 💬 [Community Discord](#) (when available)
- 📧 [Email Support](mailto:support@example.org) (if available)

---

## Common Error Codes

| Error Code | Meaning | Solution |
|------------|---------|----------|
| `400` | Bad Request | Check request format |
| `401` | Unauthorized | Get fresh API token |
| `403` | Forbidden | Check user permissions |
| `404` | Not Found | Verify endpoint URL |
| `409` | Conflict | Resource already exists |
| `422` | Validation Error | Check data format |
| `500` | Server Error | Check logs, report bug |
| `503` | Service Unavailable | Container not running |

---

*Last updated: 2025-01-04*
*Version: 1.0.0*
