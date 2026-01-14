# SPARQL Query Library

**Ready-to-use queries for supply chain traceability and analysis**

---

## How to Use This Library

These queries are copy-paste ready. Just:

1. **Find the query** that matches your use case
2. **Replace parameters** (marked in `ALL_CAPS`)
3. **Execute** via web interface or API
4. **Review results**

### Execution Methods

**Via API**:
```bash
curl -X POST http://localhost:8080/api/sparql/query \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"query": "YOUR_SPARQL_QUERY_HERE"}'
```

**Via Web Interface**:
1. Open http://localhost:8080
2. Navigate to "Query" or "SPARQL"
3. Paste the query
4. Click "Execute"

---

## Product Traceability Queries

### Query 1: Complete Product Journey

**Use Case**: Track a product batch through the entire supply chain

```sparql
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX core: <http://provchain.org/core#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

SELECT ?activity ?activityType ?timestamp ?agent ?location WHERE {
    # Find the batch
    ?batch a core:Batch ;
           core:hasIdentifier "YOUR_BATCH_ID" .
    
    # Find all activities related to this batch
    {
        ?activity prov:used ?batch .
    } UNION {
        ?activity prov:generated ?batch .
    }
    
    # Get activity details
    ?activity a ?activityType ;
              core:recordedAt ?timestamp .
    
    # Find who performed the activity
    OPTIONAL {
        ?activity prov:wasAssociatedWith ?agent .
    }
    
    # Get location if available
    OPTIONAL {
        ?activity core:atLocation ?location .
    }
}
ORDER BY ?timestamp
```

**Parameters**:
- Replace `YOUR_BATCH_ID` with your batch identifier (e.g., "TOMATO-2025-001")

**Returns**: Complete journey with activities, timestamps, agents, and locations

**Processing time**: < 1 second

---

### Query 2: Trace Product by Batch ID

**Use Case**: Get all information about a specific batch

```sparql
PREFIX core: <http://provchain.org/core#>

SELECT ?predicate ?object WHERE {
    ?batch core:hasIdentifier "YOUR_BATCH_ID" ;
           ?predicate ?object .
}
```

**Parameters**:
- Replace `YOUR_BATCH_ID` with your batch ID

**Returns**: All properties and values for the batch

---

### Query 3: Products from Specific Farm

**Use Case**: List all products from a supplier/farm

```sparql
PREFIX core: <http://provchain.org/core#>
PREFIX prov: <http://www.w3.org/ns/prov#>

SELECT ?batchId ?product ?productionDate ?quantity ?unit WHERE {
    # Find batches attributed to the farm
    ?batch a core:Batch ;
           core:hasIdentifier ?batchId ;
           core:producedAt ?productionDate ;
           prov:wasAttributedTo ?farm .
    
    # Get farm details
    ?farm a core:Supplier ;
          prov:label "FARM_NAME" .
    
    # Get product info
    ?batch core:hasProductType ?product .
    
    # Get quantity if available
    OPTIONAL {
        ?batch core:hasQuantity ?quantity ;
               core:hasUnit ?unit .
    }
}
ORDER BY DESC(?productionDate)
LIMIT 100
```

**Parameters**:
- Replace `FARM_NAME` with the farm/supplier name
- Adjust `LIMIT` for more results

**Returns**: All batches from the specified farm, most recent first

---

### Query 4: Current Location of Product

**Use Case**: Find where a product is now

```sparql
PREFIX core: <http://provchain.org/core#>
PREFIX prov: <http://www.w3.org/ns/prov#>

SELECT ?batchId ?currentLocation ?timestamp ?activity WHERE {
    # Find the batch
    ?batch a core:Batch ;
           core:hasIdentifier "YOUR_BATCH_ID" ;
           core:hasIdentifier ?batchId .
    
    # Find the most recent activity
    ?batch prov:wasGeneratedBy ?activity .
    ?activity core:recordedAt ?timestamp ;
              core:atLocation ?currentLocation .
    
    # Get only the latest activity
    {
        SELECT MAX(?ts) AS ?maxTimestamp WHERE {
            ?batch prov:wasGeneratedBy ?act .
            ?act core:recordedAt ?ts .
        }
    }
    FILTER(?timestamp = ?maxTimestamp)
}
LIMIT 1
```

**Parameters**:
- Replace `YOUR_BATCH_ID` with your batch ID

**Returns**: Current location and when it arrived there

---

## Quality & Compliance Queries

### Query 5: Quality Check Results

**Use Case**: View all QC tests for a batch

```sparql
PREFIX core: <http://provchain.org/core#>
PREFIX prov: <http://www.w3.org/ns/prov#>

SELECT ?checkType ?result ?inspector ?timestamp ?notes WHERE {
    # Find the batch
    ?batch a core:Batch ;
           core:hasIdentifier "YOUR_BATCH_ID" .
    
    # Find QC activities
    ?qcActivity a core:QualityControlProcess ;
                prov:used ?batch ;
                core:recordedAt ?timestamp ;
                core:checkType ?checkType ;
                core:checkResult ?result ;
                prov:wasAssociatedWith ?inspector .
    
    # Get notes if available
    OPTIONAL {
        ?qcActivity core:hasNotes ?notes .
    }
}
ORDER BY DESC(?timestamp)
```

**Parameters**:
- Replace `YOUR_BATCH_ID` with your batch ID

**Returns**: All quality checks performed on the batch

---

### Query 6: Products That Failed QC

**Use Case**: Find products that failed quality checks

```sparql
PREFIX core: <http://provchain.org/core#>

SELECT ?batchId ?batch ?product ?checkType ?result ?timestamp WHERE {
    # Find QC activities that failed
    ?qcActivity a core:QualityControlProcess ;
                core:checkResult "Fail" ;
                core:recordedAt ?timestamp ;
                core:checkType ?checkType ;
                prov:used ?batch .
    
    # Get batch details
    ?batch a core:Batch ;
           core:hasIdentifier ?batchId ;
           core:hasProductType ?product .
}
ORDER BY DESC(?timestamp)
```

**Returns**: All batches that failed quality checks, most recent first

---

### Query 7: Compliance Status

**Use Case**: Check if products meet certification requirements

```sparql
PREFIX core: <http://provchain.org/core#>

SELECT ?batchId ?certification ?complianceStatus ?expiryDate WHERE {
    # Find batches with certification info
    ?batch a core:Batch ;
           core:hasIdentifier ?batchId ;
           core:hasIdentifier "YOUR_BATCH_ID_OR_WILDCARD" .
    
    # Get certification info
    ?batch core:hasCertification ?certRecord .
    ?certRecord a core:Certification ;
               core:certificationType ?certification ;
               core:complianceStatus ?complianceStatus .
    
    # Get expiry if available
    OPTIONAL {
        ?certRecord core:validUntil ?expiryDate .
    }
}
```

**Parameters**:
- Replace `YOUR_BATCH_ID_OR_WILDCARD` with specific batch or leave for all batches

**Returns**: Compliance status for various certifications

---

### Query 8: Expiring Certifications

**Use Case**: Find certifications expiring soon

```sparql
PREFIX core: <http://provchain.org/core#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

SELECT ?batchId ?certification ?expiryDate ?daysRemaining WHERE {
    # Find batches with certifications
    ?batch a core:Batch ;
           core:hasIdentifier ?batchId ;
           core:hasCertification ?certRecord .
    
    ?certRecord a core:Certification ;
               core:certificationType ?certification ;
               core:validUntil ?expiryDate .
    
    # Calculate days remaining
    BIND(
        (xsd:integer(?expiryDate) - xsd:integer(now())) / 86400
        AS ?daysRemaining
    )
    
    # Filter for expiring in next 30 days
    FILTER(?daysRemaining >= 0 && ?daysRemaining <= 30)
}
ORDER BY ASC(?daysRemaining)
```

**Returns**: Certifications expiring within 30 days

---

## Environmental Monitoring Queries

### Query 9: Environmental Conditions During Transport

**Use Case**: Monitor temperature/humidity during shipping

```sparql
PREFIX core: <http://provchain.org/core#>
PREFIX prov: <http://www.w3.org/ns/prov#>

SELECT ?batchId ?conditionType ?temperature ?humidity ?location ?timestamp WHERE {
    # Find the batch
    ?batch a core:Batch ;
           core:hasIdentifier "YOUR_BATCH_ID" ;
           core:hasIdentifier ?batchId .
    
    # Find transport activities
    ?batch prov:wasGeneratedBy ?activity .
    ?activity a core:TransportProcess .
    
    # Get environmental conditions
    ?activity core:hasCondition ?condition .
    ?condition a ?conditionType ;
               core:hasTemperature ?temperature ;
               core:hasHumidity ?humidity ;
               core:recordedAt ?timestamp .
    
    # Get location if available
    OPTIONAL {
        ?condition core:atLocation ?location .
    }
}
ORDER BY ?timestamp
```

**Parameters**:
- Replace `YOUR_BATCH_ID` with your batch ID

**Returns**: Temperature and humidity readings over time

---

### Query 10: Temperature Excursions

**Use Case**: Find batches with temperature violations

```sparql
PREFIX core: <http://provchain.org/core#>

SELECT ?batchId ?batch ?temperature ?threshold ?timestamp ?excursionType WHERE {
    # Find batches with environmental data
    ?batch a core:Batch ;
           core:hasIdentifier ?batchId .
    
    ?batch prov:wasGeneratedBy ?activity .
    ?activity core:hasCondition ?condition .
    
    # Check temperature
    ?condition core:hasTemperature ?temperature ;
               core:recordedAt ?timestamp .
    
    # Alert thresholds (adjust as needed)
    BIND(8.0 AS ?threshold)
    FILTER(?temperature > ?threshold)
    
    # Determine severity
    BIND(
        IF(?temperature > 12.0, "CRITICAL",
            IF(?temperature > 10.0, "WARNING", "INFO")
        ) AS ?excursionType)
}
ORDER BY DESC(?temperature)
```

**Returns**: Batches that exceeded temperature thresholds

**Alert levels**:
- **CRITICAL**: > 12°C (severe excursion)
- **WARNING**: > 10°C (moderate excursion)
- **INFO**: > 8°C (minor excursion)

---

### Query 11: Humidity Alerts

**Use Case**: Find batches with high humidity

```sparql
PREFIX core: <http://provchain.org/core#>

SELECT ?batchId ?humidity ?timestamp ?alertType WHERE {
    # Find batches with humidity data
    ?batch a core:Batch ;
           core:hasIdentifier ?batchId .
    
    ?batch prov:wasGeneratedBy ?activity .
    ?activity core:hasCondition ?condition .
    
    # Check humidity
    ?condition core:hasHumidity ?humidity ;
               core:recordedAt ?timestamp .
    
    # Alert threshold (adjust for your products)
    FILTER(?humidity > 80)
    
    # Determine alert type
    BIND(
        IF(?humidity > 90, "CRITICAL",
            IF(?humidity > 85, "WARNING", "INFO")
        ) AS ?alertType)
}
ORDER BY DESC(?humidity)
```

**Returns**: Batches with humidity above 80%

---

## Analytics & Reporting Queries

### Query 12: Farm Performance Summary

**Use Case**: Compare production across suppliers

```sparql
PREFIX core: <http://provchain.org/core#>
PREFIX prov: <http://www.w3.org/ns/prov#>

SELECT ?farm ?product 
       COUNT(DISTINCT ?batch) AS ?batchCount 
       SUM(?quantity) AS ?totalQuantity
       MIN(?productionDate) AS ?firstBatch 
       MAX(?productionDate) AS ?lastBatch 
WHERE {
    # Find batches from farms
    ?batch a core:Batch ;
           core:producedAt ?productionDate ;
           prov:wasAttributedTo ?farm .
    
    # Get farm name
    ?farm a core:Supplier ;
          prov:label ?farm .
    
    # Get product type
    ?batch core:hasProductType ?product .
    
    # Get quantity if available
    OPTIONAL {
        ?batch core:hasQuantity ?quantity .
    }
}
GROUP BY ?farm ?product
ORDER BY DESC(?batchCount)
```

**Returns**: Production summary by farm and product

**Use for**: Supplier performance evaluation

---

### Query 13: Monthly Production Volume

**Use Case**: Track production over time

```sparql
PREFIX core: <http://provchain.org/core#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

SELECT ?year ?month 
       COUNT(DISTINCT ?batch) AS ?batchCount
       SUM(COALESCE(?quantity, 0)) AS ?totalVolume
WHERE {
    # Find all batches
    ?batch a core:Batch ;
           core:producedAt ?productionDate .
    
    # Extract year and month
    BIND(year(?productionDate) AS ?year)
    BIND(month(?productionDate) AS ?month)
    
    # Get quantity
    OPTIONAL {
        ?batch core:hasQuantity ?quantity .
    }
}
GROUP BY ?year ?month
ORDER BY DESC(?year) DESC(?month)
```

**Returns**: Production volumes by month

---

### Query 14: Products by Type

**Use Case**: Inventory breakdown by product type

```sparql
PREFIX core: <http://provchain.org/core#>

SELECT ?productType 
       COUNT(DISTINCT ?batch) AS ?batchCount
       SUM(COALESCE(?quantity, 0)) AS ?totalQuantity
       ?unit
WHERE {
    # Find all batches
    ?batch a core:Batch ;
           core:hasProductType ?productType .
    
    # Get quantity
    OPTIONAL {
        ?batch core:hasQuantity ?quantity ;
               core:hasUnit ?unit .
    }
}
GROUP BY ?productType ?unit
ORDER BY DESC(?batchCount)
```

**Returns**: Product inventory summary

---

### Query 15: Supplier Compliance Report

**Use Case**: Report on supplier compliance status

```sparql
PREFIX core: <http://provchain.org/core#>
PREFIX prov: <http://www.w3.org/ns/prov#>

SELECT ?farm 
       COUNT(DISTINCT ?batch) AS ?totalBatches
       SUM(IF(?compliant = true, 1, 0)) AS ?compliantBatches
       SUM(IF(?compliant = false, 1, 0)) AS ?nonCompliantBatches
       (SUM(IF(?compliant = true, 1, 0)) * 100.0 / COUNT(DISTINCT ?batch)) AS ?complianceRate
WHERE {
    # Find batches from farms
    ?batch a core:Batch ;
           prov:wasAttributedTo ?farm .
    
    ?farm a core:Supplier ;
          prov:label ?farm .
    
    # Check compliance
    OPTIONAL {
        ?batch core:hasCertification ?cert .
        ?cert core:complianceStatus ?status .
        BIND(IF(?status = "compliant", true, false) AS ?compliant)
    }
    
    # If no certification, assume non-compliant
    BIND(IF(BOUND(?status), IF(?status = "compliant", true, false), false) AS ?compliant)
}
GROUP BY ?farm
ORDER BY DESC(?complianceRate)
```

**Returns**: Compliance rates by supplier

---

## Advanced Queries

### Query 16: Product Genealogy

**Use Case**: Trace product back to source materials

```sparql
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX core: <http://provchain.org/core#>

SELECT ?productId ?sourceId ?sourceType ?path WHERE {
    # Start with product
    ?product a core:Batch ;
             core:hasIdentifier "YOUR_PRODUCT_ID" ;
             core:hasIdentifier ?productId .
    
    # Trace backwards through activities
    ?product prov:wasGeneratedBy ?activity1 .
    ?activity1 prov:used ?source .
    ?source a core:Batch ;
            core:hasIdentifier ?sourceId ;
            core:hasProductType ?sourceType .
    
    # Build path
    BIND(CONCAT(?productId, " -> ", ?sourceId) AS ?path)
}
```

**Parameters**:
- Replace `YOUR_PRODUCT_ID` with your product ID

**Returns**: Source materials used to create the product

---

### Query 17: Related Products

**Use Case**: Find products that share common sources

```sparql
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX core: <http://provchain.org/core#>

SELECT ?relatedBatchId ?relationship WHERE {
    # Find the source batch
    ?batch a core:Batch ;
           core:hasIdentifier "YOUR_BATCH_ID" ;
           prov:wasGeneratedBy ?activity .
    
    # Find what was used
    ?activity prov:used ?source .
    ?source a core:Batch .
    
    # Find other products using same source
    ?otherActivity prov:used ?source ;
                   prov:generated ?relatedBatch .
    
    ?relatedBatch a core:Batch ;
                 core:hasIdentifier ?relatedBatchId .
    
    # Exclude the original batch
    FILTER(?relatedBatchId != "YOUR_BATCH_ID")
    
    # Determine relationship
    BIND("Common Source" AS ?relationship)
}
```

**Parameters**:
- Replace `YOUR_BATCH_ID` with your batch ID

**Returns**: Products that share ingredients/materials

---

### Query 18: Full Audit Trail

**Use Case**: Complete history for compliance auditing

```sparql
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX core: <http://provchain.org/core#>

SELECT ?timestamp 
       ?activityType 
       ?agent 
       ?location 
       ?details 
WHERE {
    {
        # Activities that used the batch
        ?activity prov:used ?batch .
        BIND("Input" AS ?details)
    } UNION {
        # Activities that generated the batch
        ?activity prov:generated ?batch .
        BIND("Output" AS ?details)
    } UNION {
        # Activities associated with the batch
        ?activity prov:used ?batch .
        BIND("Associated" AS ?details)
    }
    
    # Get batch
    ?batch a core:Batch ;
           core:hasIdentifier "YOUR_BATCH_ID" .
    
    # Get activity details
    ?activity a ?activityType ;
              core:recordedAt ?timestamp .
    
    # Get agent
    OPTIONAL {
        ?activity prov:wasAssociatedWith ?agent .
    }
    
    # Get location
    OPTIONAL {
        ?activity core:atLocation ?location .
    }
}
ORDER BY ?timestamp
```

**Parameters**:
- Replace `YOUR_BATCH_ID` with your batch ID

**Returns**: Complete audit trail for compliance

---

## Performance Optimization

### Query 19: Fast Batch Lookup

**Use Case**: Quick batch lookup (optimized)

```sparql
PREFIX core: <http://provchain.org/core#>

SELECT ?predicate ?object WHERE {
    # Direct batch lookup (most efficient)
    ?batch core:hasIdentifier "YOUR_BATCH_ID" ;
           ?predicate ?object .
}
LIMIT 100
```

**Optimization tips**:
- Use `core:hasIdentifier` for direct lookups
- Add `LIMIT` to prevent large result sets
- Avoid `OPTIONAL` unless needed

---

### Query 20: Efficient Recent Activity

**Use Case**: Get recent activities (optimized)

```sparql
PREFIX core: <http://provchain.org/core#>

SELECT ?activityType ?timestamp ?batch WHERE {
    # Find recent activities
    ?activity a ?activityType ;
              core:recordedAt ?timestamp .
    
    # Get associated batch
    OPTIONAL {
        ?activity prov:generated ?batch .
        ?batch a core:Batch ;
               core:hasIdentifier ?batchId .
    }
    
    # Filter by date (adjust as needed)
    FILTER(?timestamp >= "2025-01-01T00:00:00Z"^^xsd:dateTime)
}
ORDER BY DESC(?timestamp)
LIMIT 50
```

**Optimization**: Date filtering reduces result set size

---

## Query Templates

### Template 1: Time Range Filter

Add date filtering to any query:

```sparql
# Add this WHERE clause
FILTER(?timestamp >= "START_DATE"^^xsd:dateTime && 
       ?timestamp <= "END_DATE"^^xsd:dateTime)
```

**Example**:
```sparql
FILTER(?productionDate >= "2025-01-01T00:00:00Z"^^xsd:dateTime && 
       ?productionDate <= "2025-01-31T23:59:59Z^^xsd:dateTime)
```

---

### Template 2: Multiple Batch IDs

Query multiple batches:

```sparql
VALUES ?batchId { 
    "BATCH-001" 
    "BATCH-002" 
    "BATCH-003" 
}

?batch core:hasIdentifier ?batchId .
```

---

### Template 3: Aggregation

Calculate statistics:

```sparql
SELECT ?farm 
       AVG(?temperature) AS ?avgTemp 
       MIN(?temperature) AS ?minTemp 
       MAX(?temperature) AS ?maxTemp 
       COUNT(?batch) AS ?batchCount 
WHERE {
    # ... your query ...
}
GROUP BY ?farm
```

---

## Best Practices

1. **Use specific identifiers** for faster queries
2. **Add LIMIT** to prevent large result sets
3. **Filter by date** when querying historical data
4. **Use OPTIONAL** for non-critical properties
5. **Test complex queries** with small LIMIT first
6. **Use EXPLAIN** to analyze query performance (if available)

---

## Need More Queries?

- See [Advanced Queries](advanced-queries.md) for complex analysis
- Check [Query Optimization](query-optimization.md) for performance tips
- Review [SPARQL Basics](sparql-basics.md) for query language fundamentals

---

*Last updated: 2025-01-04*
*Version: 1.0.0*
