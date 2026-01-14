# Your First Transaction

**Detailed guide to submitting your first supply chain data to the blockchain**

---

## What You'll Learn

In this guide, you will:
- Understand what a transaction is
- Prepare your data for submission
- Submit a transaction via API
- Submit a transaction via Web Interface
- Verify your data was recorded
- Handle common errors

**Prerequisites**: Complete [10-Minute Setup](10-minute-setup.md) first

---

## Understanding Transactions

### What is a Transaction?

A **transaction** is a record of an event or fact in your supply chain. Each transaction is:

- **Immutable** - Once recorded, it cannot be changed
- **Timestamped** - Exact time is automatically recorded
- **Cryptographically verified** - Tamper-evident
- **Linked** - Can reference other transactions

### Transaction Types

ProvChain supports several transaction types:

| Type | Use Case | Example |
|------|----------|---------|
| **Production** | Recording harvest/production | "Tomatoes harvested from Farm A" |
| **Processing** | Manufacturing/processing activities | "Sorted and packaged tomatoes" |
| **Transport** | Shipping and logistics | "Shipped from Thailand to Japan" |
| **Quality** | Quality control inspections | "Passed microbiological testing" |
| **Transfer** | Ownership change | "Sold to Distributor B" |
| **Environmental** | Sensor readings | "Temperature: 5°C, Humidity: 85%" |
| **Compliance** | Certification records | "USDA Organic certified" |

---

## Step 1: Prepare Your Data

### The Data Model: RDF Triples

ProvChain uses **RDF triples** to store data. A triple has three parts:

```
Subject → Predicate → Object
```

Example:
```
:Batch001 → :hasProductType → "Organic Tomatoes"
:Batch001 → :harvestedAt → "Green Valley Farm"
:Batch001 → :harvestDate → "2025-01-04"
```

### Our Example: Product Batch

We'll record a tomato harvest:

| Field | Value |
|-------|-------|
| **Subject** | `http://example.org/batch/TOMATO-2025-001` |
| **Predicate** | `http://example.org/ns#productType` |
| **Object** | `Organic Tomatoes` |

### JSON Format for API

When submitting via API, use this JSON format:

```json
{
  "subject": "http://example.org/batch/TOMATO-2025-001",
  "predicate": "http://example.org/ns#productType",
  "object": "Organic Tomatoes",
  "graph_name": null
}
```

---

## Step 2: Get Your API Token

You need an authentication token to submit data.

```bash
# Login and save token
TOKEN=$(curl -s -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}' \
  | jq -r '.token')

echo "Your token: $TOKEN"
```

**Expected output**: A long string starting with `eyJhbGciOiJIUzI1NiIs...`

**Keep this token safe** - it's your access key!

---

## Step 3: Submit via API

### Method 1: Single Triple (Simple)

Submit one piece of information:

```bash
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
  "block_hash": "0x8f3e2d1c9b8a7654...",
  "timestamp": "2025-01-04T10:30:00Z",
  "message": "Triple added successfully"
}
```

### Method 2: Multiple Triples (Complete Record)

Submit all information about a batch at once:

```bash
# Product type
curl -X POST http://localhost:8080/api/blockchain/add-triple \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "subject": "http://example.org/batch/TOMATO-2025-001",
    "predicate": "http://example.org/ns#productType",
    "object": "Organic Tomatoes"
  }'

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

# Unit
curl -X POST http://localhost:8080/api/blockchain/add-triple \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "subject": "http://example.org/batch/TOMATO-2025-001",
    "predicate": "http://example.org/ns#unit",
    "object": "kg"
  }'

# Certification
curl -X POST http://localhost:8080/api/blockchain/add-triple \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "subject": "http://example.org/batch/TOMATO-2025-001",
    "predicate": "http://example.org/ns#certification",
    "object": "USDA Organic"
  }'
```

All six pieces of information are now recorded on the blockchain!

---

## Step 4: Submit via Web Interface

If you prefer a graphical interface:

1. **Open** http://localhost:8080 in your browser
2. **Click** "Submit Data" or "Add Triple"
3. **Fill in the form**:
   - Subject: `http://example.org/batch/TOMATO-2025-001`
   - Predicate: `http://example.org/ns#productType`
   - Object: `Organic Tomatoes`
4. **Click** "Submit"

Repeat for each piece of information.

---

## Step 5: Verify Your Transaction

### Check the Blockchain Status

```bash
curl -s http://localhost:8080/api/blockchain/status | jq .
```

**Expected output**:
```json
{
  "block_count": 6,
  "transaction_count": 6,
  "latest_block_hash": "0x...",
  "is_valid": true
}
```

### View Your Data

```bash
# Get recent transactions
curl -s http://localhost:8080/api/transactions/recent | jq .

# Get specific block
curl -s http://localhost:8080/api/blockchain/blocks/1 | jq .

# Get all blocks
curl -s http://localhost:8080/api/blockchain/dump | jq .
```

### Query Your Specific Batch

```bash
curl -X POST http://localhost:8080/api/sparql/query \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "query": "PREFIX : <http://example.org/ns#> SELECT ?predicate ?object WHERE { <http://example.org/batch/TOMATO-2025-001> ?predicate ?object }"
  }'
```

**Expected output**:
```json
{
  "results": [
    {"predicate": "...#productType", "object": "Organic Tomatoes"},
    {"predicate": "...#originFarm", "object": "Green Valley Farm"},
    {"predicate": "...#harvestDate", "object": "2025-01-04"},
    {"predicate": "...#quantity", "object": "500"},
    {"predicate": "...#unit", "object": "kg"},
    {"predicate": "...#certification", "object": "USDA Organic"}
  ]
}
```

---

## Common Errors & Solutions

### Error: "Unauthorized"

**Cause**: Missing or invalid token

**Solution**:
```bash
# Get a fresh token
TOKEN=$(curl -s -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}' \
  | jq -r '.token')
```

### Error: "Invalid subject URI"

**Cause**: Subject is not a valid URI

**Solution**: Subjects must be valid URIs starting with `http://` or `https://`:
- ✅ `http://example.org/batch/BATCH-001`
- ✅ `https://mycompany.com/product/123`
- ❌ `BATCH-001` (not a URI)

### Error: "Invalid predicate URI"

**Cause**: Predicate is not a valid URI

**Solution**: Use proper predicate URIs:
- ✅ `http://example.org/ns#productType`
- ✅ `http://www.w3.org/ns/sosa/hasResult`
- ❌ `productType` (not a URI)

### Error: "Invalid object literal"

**Cause**: Malformed object value

**Solution**: 
- For URIs: `http://example.org/farms/FarmA`
- For literals: `"Text with quotes"` (wrap in quotes)
- For numbers: `123` or `45.67` (no quotes needed)

### Error: "Validation failed"

**Cause**: Data doesn't conform to SHACL shapes

**Solution**: Check required properties and data types. See [Data Validation](../02-submitting-data/data-validation.md).

---

## Best Practices

### 1. Use Consistent Identifiers

```
✅ Good: http://example.org/batch/TOMATO-2025-001
✅ Good: http://example.org/batch/TOMATO-2025-002
❌ Bad: http://example.org/batch/tomato-001
❌ Bad: http://example.org/batch/001
```

### 2. Follow Naming Conventions

- **Batches**: Use format `{PRODUCT}-{YEAR}-{NUMBER}` 
  - Example: `TOMATO-2025-001`
- **Activities**: Use format `{TYPE}-{DATE}-{ID}`
  - Example: `HARVEST-2025-01-04-001`

### 3. Include Required Properties

For product batches, always include:
- Product type
- Origin/farm
- Date
- Quantity
- Unit

### 4. Use Proper Data Types

| Property | Format | Example |
|----------|--------|---------|
| Dates | ISO 8601 | `2025-01-04` |
| Numbers | Numeric | `500` or `99.5` |
| Text | Quoted string | `"Organic Tomatoes"` |
| URIs | Full URI | `http://example.org/farm/A` |

### 5. Validate Before Submitting

Test your data structure:
```bash
# Validate JSON
echo '{"subject":"..."}' | jq .

# Test query first
curl -X POST http://localhost:8080/api/sparql/query \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"query":"YOUR_QUERY"}' | jq .
```

---

## Real-World Examples

### Example 1: Recording Harvest

```bash
curl -X POST http://localhost:8080/api/blockchain/add-triple \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "subject": "http://example.org/harvest/HARVEST-2025-001",
    "predicate": "http://www.w3.org/ns/prov#used",
    "object": "http://example.org/farm/GreenValley"
  }'
```

### Example 2: Recording Temperature During Transport

```bash
curl -X POST http://localhost:8080/api/blockchain/add-triple \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "subject": "http://example.org/sensor/TEMP-001",
    "predicate": "http://www.w3.org/ns/sosa/hasSimpleResult",
    "object": "4.5"
  }'
```

### Example 3: Quality Inspection Result

```bash
curl -X POST http://localhost:8080/api/blockchain/add-triple \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "subject": "http://example.org/inspection/QC-2025-001",
    "predicate": "http://www.w3.org/ns/sosa/hasResult",
    "object": "Pass"
  }'
```

---

## Next Steps

Now that you've submitted your first transaction:

- 📖 [Submitting Product Batches](../02-submitting-data/product-batches.md) - Complete batch submission
- 🌡️ [Environmental Data](../02-submitting-data/environmental-data.md) - Record sensor data
- ✅ [Quality Control](../02-submitting-data/quality-control.md) - QC inspections
- 🔍 [Query Library](../03-querying-data/query-library.md) - Analyze your data

---

## Summary

**You learned**:
- ✅ What transactions are and their types
- ✅ How to structure data as RDF triples
- ✅ How to submit transactions via API
- ✅ How to verify data was recorded
- ✅ Common errors and how to fix them

**Your first transaction is on the blockchain!** 🎉

*Continue to [Your First Query](first-query.md) to learn how to retrieve and analyze your data.*
