SPARQL Basics
=============

Introduction to querying supply chain data using SPARQL.

.. raw:: html

   <div class="hero-section">
     <div class="hero-content">
       <h1>SPARQL Basics</h1>
       <p class="hero-subtitle">Introduction to querying supply chain data with SPARQL</p>
       <div class="hero-badges">
         <span class="badge badge-sparql">SPARQL</span>
         <span class="badge badge-query">Query</span>
         <span class="badge badge-beginner">Beginner</span>
       </div>
     </div>
   </div>

.. note::
   This guide introduces SPARQL (SPARQL Protocol and RDF Query Language), the standard query language for RDF data used in ProvChainOrg. Learn how to query supply chain information effectively.

What is SPARQL?
---------------

SPARQL is a powerful query language for RDF (Resource Description Framework) data. It allows you to:

1. **Query** data stored in RDF format
2. **Update** RDF data
3. **Manage** RDF graphs
4. **Federate** queries across multiple data sources

In ProvChainOrg, SPARQL is the primary way to query supply chain data stored in the semantic blockchain.

Basic SPARQL Concepts
--------------------

### RDF Data Model

Before diving into SPARQL, understand the RDF data model:

1. **Triples**: RDF data consists of subject-predicate-object triples
2. **URIs**: Unique identifiers for resources
3. **Literals**: Data values like strings, numbers, dates
4. **Blank Nodes**: Anonymous nodes in the graph
5. **Named Graphs**: Collections of triples with a name

### Example RDF Data

.. code-block:: turtle

   @prefix : <http://example.org/supply-chain#> .
   @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
   
   :Batch001 a :ProductBatch ;
       :product "Organic Tomatoes" ;
       :originFarm :GreenValleyFarm ;
       :harvestDate "2025-01-15"^^xsd:date ;
       :quantity 1000 ;
       :unit "kg" .

This creates several triples:
- `:Batch001 rdf:type :ProductBatch`
- `:Batch001 :product "Organic Tomatoes"`
- `:Batch001 :originFarm :GreenValleyFarm`
- etc.

Basic SPARQL Queries
-------------------

### SELECT Queries

The most common type of SPARQL query retrieves data:

.. code-block:: sparql

   PREFIX : <http://example.org/supply-chain#>
   
   SELECT ?batch ?product ?farm WHERE {
       ?batch a :ProductBatch ;
              :product ?product ;
              :originFarm ?farm .
   }

This query:
1. Defines a prefix for shorter URIs
2. Selects three variables: batch, product, and farm
3. Matches triples where:
   - `?batch` is a ProductBatch
   - `?batch` has a product property with value `?product`
   - `?batch` has an originFarm property with value `?farm`

### FILTER Queries

Add conditions to filter results:

.. code-block:: sparql

   PREFIX : <http://example.org/supply-chain#>
   PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
   
   SELECT ?batch ?product ?harvestDate WHERE {
       ?batch a :ProductBatch ;
              :product ?product ;
              :harvestDate ?harvestDate .
       FILTER(?harvestDate >= "2025-01-01"^^xsd:date)
   }

### ORDER BY and LIMIT

Sort and limit results:

.. code-block:: sparql

   PREFIX : <http://example.org/supply-chain#>
   
   SELECT ?batch ?product ?harvestDate WHERE {
       ?batch a :ProductBatch ;
              :product ?product ;
              :harvestDate ?harvestDate .
   }
   ORDER BY DESC(?harvestDate)
   LIMIT 10

ProvChainOrg SPARQL Features
----------------------------

### Named Graphs

Query specific blocks or time periods using named graphs:

.. code-block:: sparql

   PREFIX : <http://example.org/supply-chain#>
   
   SELECT ?batch ?product WHERE {
       GRAPH ?g {
           ?batch a :ProductBatch ;
                  :product ?product .
       }
       FILTER(CONTAINS(STR(?g), "block/123"))
   }

### Temporal Queries

Query data based on blockchain timestamps:

.. code-block:: sparql

   PREFIX : <http://example.org/supply-chain#>
   PREFIX prov: <http://www.w3.org/ns/prov#>
   
   SELECT ?batch ?timestamp WHERE {
       ?batch a :ProductBatch ;
              prov:generatedAtTime ?timestamp .
       FILTER(?timestamp > "2025-01-15T00:00:00Z"^^xsd:dateTime)
   }

### Supply Chain Specific Patterns

Common patterns for supply chain queries:

#### Traceability Query

.. code-block:: sparql

   PREFIX : <http://example.org/supply-chain#>
   PREFIX prov: <http://www.w3.org/ns/prov#>
   
   SELECT ?batch ?location ?timestamp ?activity WHERE {
       ?batch :hasBatchID "TOMATO-2025-001" .
       ?activity prov:used ?batch ;
                 :location ?location ;
                 prov:endedAtTime ?timestamp .
   }
   ORDER BY ?timestamp

#### Quality Assurance Query

.. code-block:: sparql

   PREFIX : <http://example.org/supply-chain#>
   
   SELECT ?batch ?testType ?result ?tester WHERE {
       ?batch :qualityTest ?test .
       ?test :testType ?testType ;
             :result ?result ;
             :testedBy ?tester .
       FILTER(?result = "PASS")
   }

#### Environmental Monitoring Query

.. code-block:: sparql

   PREFIX : <http://example.org/supply-chain#>
   PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
   
   SELECT ?batch ?temperature ?humidity ?timestamp WHERE {
       ?batch :transportedThrough ?transport .
       ?transport :environmentalCondition ?condition .
       ?condition :temperature ?temperature ;
                  :humidity ?humidity ;
                  :recordedAt ?timestamp .
       FILTER(?temperature > "8.0"^^xsd:decimal)
   }

Advanced SPARQL Features
-----------------------

### OPTIONAL Patterns

Handle missing data gracefully:

.. code-block:: sparql

   PREFIX : <http://example.org/supply-chain#>
   
   SELECT ?batch ?product ?certification WHERE {
       ?batch a :ProductBatch ;
              :product ?product .
       OPTIONAL {
           ?batch :certification ?certification .
       }
   }

### UNION Patterns

Combine different patterns:

.. code-block:: sparql

   PREFIX : <http://example.org/supply-chain#>
   
   SELECT ?item ?type WHERE {
       {
           ?item a :ProductBatch ;
                 :product ?type .
       } UNION {
           ?item a :IngredientLot ;
                 :ingredient ?type .
       }
   }

### Aggregation Functions

Calculate summary statistics:

.. code-block:: sparql

   PREFIX : <http://example.org/supply-chain#>
   
   SELECT ?farm (COUNT(?batch) AS ?batchCount) 
          (SUM(?quantity) AS ?totalQuantity) WHERE {
       ?batch a :ProductBatch ;
              :originFarm ?farm ;
              :quantity ?quantity .
   }
   GROUP BY ?farm

### Subqueries

Use results from one query in another:

.. code-block:: sparql

   PREFIX : <http://example.org/supply-chain#>
   
   SELECT ?batch ?product WHERE {
       ?batch a :ProductBatch ;
              :product ?product .
       {
           SELECT ?highVolumeFarm WHERE {
               ?batch a :ProductBatch ;
                      :originFarm ?farm ;
                      :quantity ?quantity .
               FILTER(?quantity > 1000)
           }
       }
   }

Using SPARQL in ProvChainOrg
----------------------------

### Command Line Interface

Execute SPARQL queries using the CLI:

.. code-block:: bash

   # Execute a query from a file
   provchain-org query trace_query.sparql
   
   # Execute a query from stdin
   echo "SELECT * WHERE { ?s ?p ?o } LIMIT 10" | provchain-org query -
   
   # Save results to a file
   provchain-org query trace_query.sparql --output results.json

### REST API

Execute SPARQL queries using the REST API:

.. code-block:: bash

   # Query using curl
   curl -X POST "http://localhost:8080/sparql" \
        -H "Authorization: Bearer YOUR_API_KEY" \
        -H "Content-Type: application/sparql-query" \
        --data-binary @trace_query.sparql

### Web Interface

Use the web-based SPARQL query interface:

1. Navigate to `http://localhost:8080/query`
2. Enter your SPARQL query in the editor
3. Click "Execute" to run the query
4. View results in table or graph format

Query Optimization
------------------

### Performance Tips

1. **Use specific patterns**: More specific patterns execute faster
2. **Limit result sets**: Use LIMIT to avoid excessive results
3. **Index frequently queried properties**: Structure data for common queries
4. **Avoid complex filters**: Simplify WHERE clauses when possible

### Indexing

ProvChainOrg automatically indexes data for common query patterns:

.. code-block:: sparql

   # This query benefits from indexing on :ProductBatch
   SELECT ?batch ?product WHERE {
       ?batch a :ProductBatch ;
              :product ?product .
   }

### Caching

Frequently executed queries are cached for better performance:

.. code-block:: toml

   [performance]
   query_cache_enabled = true
   query_cache_size_mb = 50

Common Query Patterns
---------------------

### Product Traceability

Trace a product from origin to destination:

.. code-block:: sparql

   PREFIX : <http://example.org/supply-chain#>
   PREFIX prov: <http://www.w3.org/ns/prov#>
   
   SELECT ?batch ?activity ?location ?timestamp WHERE {
       :Batch001 prov:wasInfluencedBy* ?activity .
       ?activity :location ?location ;
                 prov:endedAtTime ?timestamp .
   }
   ORDER BY ?timestamp

### Quality Compliance

Find products meeting quality standards:

.. code-block:: sparql

   PREFIX : <http://example.org/supply-chain#>
   
   SELECT ?batch ?product ?testDate ?tester WHERE {
       ?batch :product ?product ;
              :qualityTest ?test .
       ?test :result "PASS" ;
             :testDate ?testDate ;
             :testedBy ?tester .
   }

### Environmental Monitoring

Monitor environmental conditions during transport:

.. code-block:: sparql

   PREFIX : <http://example.org/supply-chain#>
   PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
   
   SELECT ?batch ?temperature ?humidity ?location ?timestamp WHERE {
       ?batch :transportedThrough ?transport .
       ?transport :environmentalCondition ?condition .
       ?condition :temperature ?temperature ;
                  :humidity ?humidity ;
                  :location ?location ;
                  :recordedAt ?timestamp .
       FILTER(?temperature > "2.0"^^xsd:decimal && 
              ?temperature < "8.0"^^xsd:decimal)
   }

### Supply Chain Analytics

Generate supply chain analytics:

.. code-block:: sparql

   PREFIX : <http://example.org/supply-chain#>
   
   SELECT ?farm (COUNT(?batch) AS ?totalBatches) 
          (AVG(?quantity) AS ?avgQuantity) WHERE {
       ?batch :originFarm ?farm ;
              :quantity ?quantity .
   }
   GROUP BY ?farm
   ORDER BY DESC(?totalBatches)

Error Handling
--------------

### Common SPARQL Errors

1. **Syntax Errors**: Check query syntax and brackets
2. **Undefined Prefixes**: Ensure all prefixes are defined
3. **Type Mismatches**: Verify data types in filters
4. **Resource Not Found**: Confirm URIs exist in the dataset

### Debugging Queries

.. code-block:: sparql

   # Start with a simple query and build up
   SELECT * WHERE { ?s ?p ?o } LIMIT 10
   
   # Add filters gradually
   SELECT * WHERE { 
       ?s ?p ?o .
       FILTER(isIRI(?s))
   } LIMIT 10

### Query Validation

Validate queries before execution:

.. code-block:: bash

   # Validate a SPARQL query
   provchain-org query validate query.sparql

Best Practices
--------------

### Query Design

1. **Start simple**: Begin with basic patterns and add complexity
2. **Use meaningful variable names**: Make queries self-documenting
3. **Comment complex queries**: Explain business logic in comments
4. **Test with LIMIT**: Verify queries with small result sets first

### Performance Optimization

1. **Index frequently queried properties**: Structure data for common access patterns
2. **Use FILTER strategically**: Apply filters early in the query
3. **Limit result sets**: Avoid returning excessive data
4. **Cache frequent queries**: Store results of common queries

### Security Considerations

1. **Validate user input**: Sanitize parameters in dynamic queries
2. **Limit query complexity**: Prevent denial-of-service through complex queries
3. **Use parameterized queries**: Avoid injection attacks
4. **Implement rate limiting**: Control query frequency per user

### Documentation

1. **Document complex queries**: Include business context and assumptions
2. **Version query files**: Track changes to important queries
3. **Share query patterns**: Create a library of common patterns
4. **Profile query performance**: Monitor and optimize slow queries

Learning Resources
------------------

### Official Documentation

1. **SPARQL 1.1 Query Language**: W3C Recommendation
2. **SPARQL 1.1 Update**: W3C Recommendation
3. **SPARQL 1.1 Federated Query**: W3C Recommendation

### Tutorials and Guides

1. **SPARQL by Example**: Interactive tutorial
2. **Learning SPARQL**: Book by Bob DuCharme
3. **SPARQL Tutorial**: Online courses and videos

### Tools

1. **Protégé**: Ontology editor with SPARQL support
2. **YASGUI**: Web-based SPARQL editor
3. **SPARQL Query Builder**: Visual query construction tools

Support
-------

For additional help with SPARQL queries:

1. Use the web interface query builder
2. Check the online documentation
3. Join our community forum
4. Contact support at support@provchain-org.com

.. note::
   SPARQL is a powerful tool for querying supply chain data in ProvChainOrg. Start with simple queries and gradually build up to more complex patterns. The web interface provides a helpful query builder for beginners.