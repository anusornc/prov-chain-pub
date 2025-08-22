Data Management
===============

Complete guide to managing supply chain data in ProvChainOrg.

.. raw:: html

   <div class="hero-section">
     <div class="hero-content">
       <h1>Data Management</h1>
       <p class="hero-subtitle">Complete guide to managing supply chain data</p>
       <div class="hero-badges">
         <span class="badge badge-data">Data</span>
         <span class="badge badge-management">Management</span>
         <span class="badge badge-guide">Guide</span>
       </div>
     </div>
   </div>

.. note::
   This guide provides comprehensive information about managing supply chain data in ProvChainOrg, including importing, exporting, validating, and organizing data for optimal performance and traceability.

Data Models and Formats
-----------------------

ProvChainOrg supports several standard semantic web data formats for representing supply chain information.

### Supported Formats

1. **Turtle (.ttl)** - Primary format for semantic data (recommended)
2. **JSON-LD (.jsonld)** - JSON-based linked data format
3. **N-Triples (.nt)** - Simple triple format
4. **RDF/XML (.rdf)** - XML-based RDF format
5. **TriG (.trig)** - TriG format for named graphs

### Core Data Model

ProvChainOrg uses an ontology-based approach to data modeling. The core concepts include:

#### Entities
- **ProductBatch**: A batch of products in the supply chain
- **Farm/Producer**: Origin of raw materials
- **ProcessingFacility**: Facilities that process products
- **Transportation**: Movement of products between locations
- **Retailer**: Final destination for products
- **EnvironmentalCondition**: Environmental data during transport/storage

#### Relationships
- **originatedFrom**: Links a batch to its origin
- **processedAt**: Links a batch to processing facility
- **transportedTo**: Links a batch to destination
- **storedAt**: Links a batch to storage location
- **testedFor**: Links a batch to quality tests

### Example Data Structure

.. code-block:: turtle

   @prefix : <http://example.org/supply-chain#> .
   @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
   @prefix prov: <http://www.w3.org/ns/prov#> .
   
   # Product batch
   :Batch001 a :ProductBatch ;
       :hasBatchID "TOMATO-2025-001" ;
       :product :OrganicTomatoes ;
       :originFarm :GreenValleyFarm ;
       :harvestDate "2025-01-15"^^xsd:date ;
       :quantity 1000 ;
       :unit "kg" ;
       :certifiedOrganic true ;
       prov:wasGeneratedBy :HarvestActivity001 .
   
   # Harvest activity
   :HarvestActivity001 a prov:Activity ;
       prov:startedAtTime "2025-01-15T08:00:00Z"^^xsd:dateTime ;
       prov:endedAtTime "2025-01-15T12:00:00Z"^^xsd:dateTime ;
       prov:wasAssociatedWith :Worker001 ;
       :weatherConditions [
           :temperature "18"^^xsd:decimal ;
           :humidity "65"^^xsd:decimal ;
           :precipitation "0"^^xsd:decimal
       ] .

Data Import
-----------

Importing data into ProvChainOrg can be done through several methods.

### Command Line Import

.. code-block:: bash

   # Import a single file
   provchain-org add-file supply_chain_data.ttl
   
   # Import multiple files
   provchain-org add-file data1.ttl data2.ttl data3.ttl
   
   # Import from stdin
   cat data.ttl | provchain-org add-file -
   
   # Import with specific format
   provchain-org add-file data.rdf --format rdfxml

### API Import

Using the REST API to import data:

.. code-block:: bash

   # Import using curl
   curl -X POST "http://localhost:8080/api/v1/data" \
        -H "Authorization: Bearer YOUR_API_KEY" \
        -H "Content-Type: text/turtle" \
        --data-binary @supply_chain_data.ttl

### Batch Import

For large datasets, use batch import:

.. code-block:: bash

   # Create a batch import file (JSON)
   {
     "batches": [
       {
         "name": "Batch001",
         "data": "@prefix : <http://example.org/> . :Batch001 :product :Tomatoes ."
       },
       {
         "name": "Batch002",
         "data": "@prefix : <http://example.org/> . :Batch002 :product :Lettuce ."
       }
     ]
   }
   
   # Import batch
   provchain-org batch import --file batches.json

### Import Validation

ProvChainOrg validates data during import to ensure:

1. **Syntax correctness**: Data is valid RDF
2. **Ontology compliance**: Data conforms to the supply chain ontology
3. **Integrity**: Required properties are present
4. **Consistency**: No conflicting information

.. code-block:: bash

   # Import with validation
   provchain-org add-file data.ttl --validate
   
   # Import without validation (faster but less safe)
   provchain-org add-file data.ttl --no-validate

Data Export
-----------

Exporting data from ProvChainOrg for analysis or sharing.

### Command Line Export

.. code-block:: bash

   # Export all data
   provchain-org export --output all_data.ttl
   
   # Export in specific format
   provchain-org export --output data.jsonld --format jsonld
   
   # Export specific block range
   provchain-org export --start 100 --end 200 --output range.ttl
   
   # Export specific named graph
   provchain-org export --graph "http://example.org/batch001" --output batch001.ttl

### API Export

Using the REST API to export data:

.. code-block:: bash

   # Export using curl
   curl -X GET "http://localhost:8080/api/v1/data" \
        -H "Authorization: Bearer YOUR_API_KEY" \
        -H "Accept: text/turtle" \
        -o exported_data.ttl

### SPARQL Export

Export data using SPARQL queries:

.. code-block:: bash

   # Export with SPARQL query
   provchain-org query export_query.sparql --output results.ttl
   
   # Export in different formats
   provchain-org query export_query.sparql --output results.json --format json
   provchain-org query export_query.sparql --output results.csv --format csv

### Selective Export

Export specific subsets of data:

.. code-block:: bash

   # Export by product type
   provchain-org export --filter "product=OrganicTomatoes" --output tomatoes.ttl
   
   # Export by date range
   provchain-org export --filter "date>=2025-01-01&date<=2025-01-31" --output january.ttl
   
   # Export by location
   provchain-org export --filter "location=GreenValleyFarm" --output farm_data.ttl

Data Validation
---------------

Ensuring data quality and compliance with standards.

### Automatic Validation

ProvChainOrg automatically validates data during import:

.. code-block:: bash

   # Import with automatic validation
   provchain-org add-file data.ttl
   
   # Check validation results
   provchain-org validate last-import

### Manual Validation

Validate existing data or external datasets:

.. code-block:: bash

   # Validate a file
   provchain-org validate file data.ttl
   
   # Validate data in the blockchain
   provchain-org validate blockchain
   
   # Validate specific blocks
   provchain-org validate blocks 100-200

### Validation Reports

Generate detailed validation reports:

.. code-block:: bash

   # Generate validation report
   provchain-org validate file data.ttl --report validation_report.json
   
   # Generate HTML report
   provchain-org validate file data.ttl --report validation_report.html --format html

### Validation Rules

ProvChainOrg checks for:

1. **Syntax errors**: Invalid RDF syntax
2. **Schema compliance**: Conformance to supply chain ontology
3. **Required properties**: Presence of mandatory fields
4. **Data types**: Correct data type usage
5. **Referential integrity**: Valid references between entities
6. **Business rules**: Domain-specific constraints

Data Organization
-----------------

Organizing data for efficient querying and management.

### Named Graphs

ProvChainOrg uses named graphs to organize data:

.. code-block:: bash

   # Import data into a specific named graph
   provchain-org add-file data.ttl --graph "http://example.org/batch001"
   
   # Query specific named graph
   provchain-org query query.sparql --graph "http://example.org/batch001"

### Data Partitioning

Organize data by:

1. **Time periods**: Monthly, quarterly, yearly
2. **Product categories**: Vegetables, fruits, grains
3. **Supply chain stages**: Farm, processing, distribution
4. **Geographic regions**: By country, state, or facility

### Indexing

ProvChainOrg automatically indexes data for efficient querying:

.. code-block:: bash

   # Rebuild indexes
   provchain-org index rebuild
   
   # Optimize indexes
   provchain-org index optimize
   
   # Check index status
   provchain-org index status

Data Archiving
--------------

Managing historical data for long-term storage.

### Archive Policies

Define policies for data archiving:

.. code-block:: toml

   [archiving]
   # Archive data older than 2 years
   retention_period_days = 730
   
   # Compress archived data
   compression_enabled = true
   
   # Store archives in separate location
   archive_path = "/archive/provchain"

### Archive Commands

.. code-block:: bash

   # Archive old data
   provchain-org archive create --before 2023-01-01 --output archive_2023.ttl
   
   # List archives
   provchain-org archive list
   
   # Restore from archive
   provchain-org archive restore --input archive_2023.ttl

### Archive Formats

Support for multiple archive formats:

1. **Turtle**: Human-readable format
2. **Binary RDF**: Compact format for large datasets
3. **Compressed**: GZIP or ZIP compressed archives

Data Security
-------------

Protecting sensitive supply chain data.

### Access Control

Control who can access what data:

.. code-block:: bash

   # Set permissions for a dataset
   provchain-org access set --dataset "batch001" --user "analyst1" --permission "read"
   
   # Revoke permissions
   provchain-org access revoke --dataset "batch001" --user "analyst1"
   
   # List permissions
   provchain-org access list --dataset "batch001"

### Encryption

Encrypt sensitive data:

.. code-block:: toml

   [security]
   # Enable encryption at rest
   encryption_enabled = true
   # Encryption key management
   key_management = "vault"

### Auditing

Track data access and modifications:

.. code-block:: bash

   # Enable auditing
   provchain-org audit enable
   
   # View audit logs
   provchain-org audit logs --tail 100
   
   # Export audit report
   provchain-org audit report --output audit_report.json

Performance Optimization
------------------------

Optimizing data management for better performance.

### Bulk Operations

Use bulk operations for large datasets:

.. code-block:: bash

   # Bulk import
   provchain-org bulk import --directory /path/to/data/files/
   
   # Bulk export
   provchain-org bulk export --directory /path/to/export/ --format turtle

### Caching

Configure caching for frequently accessed data:

.. code-block:: toml

   [performance]
   # Enable query caching
   query_cache_enabled = true
   query_cache_size_mb = 100
   
   # Enable data caching
   data_cache_enabled = true
   data_cache_size_mb = 500

### Parallel Processing

Process data in parallel:

.. code-block:: bash

   # Import with parallel processing
   provchain-org add-file data.ttl --parallel 4
   
   # Export with parallel processing
   provchain-org export --parallel 4 --output data.ttl

Data Quality Assurance
----------------------

Ensuring high-quality data throughout the supply chain.

### Data Profiling

Analyze data characteristics:

.. code-block:: bash

   # Profile a dataset
   provchain-org profile data.ttl --output profile_report.json
   
   # Compare datasets
   provchain-org profile compare dataset1.ttl dataset2.ttl

### Data Cleansing

Clean and standardize data:

.. code-block:: bash

   # Clean a dataset
   provchain-org clean data.ttl --output clean_data.ttl
   
   # Standardize formats
   provchain-org standardize data.ttl --output standardized_data.ttl

### Data Enrichment

Enhance data with additional information:

.. code-block:: bash

   # Enrich with external data
   provchain-org enrich data.ttl --with weather_data.ttl --output enriched_data.ttl

Integration with External Systems
---------------------------------

Connecting ProvChainOrg with other systems.

### API Integration

Connect via REST API:

.. code-block:: python

   import requests
   
   class ProvChainClient:
       def __init__(self, base_url, api_key):
           self.base_url = base_url
           self.headers = {
               'Authorization': f'Bearer {api_key}',
               'Content-Type': 'text/turtle'
           }
       
       def add_data(self, data):
           response = requests.post(
               f'{self.base_url}/api/v1/data',
               headers=self.headers,
               data=data
           )
           return response.json()

### Database Integration

Connect to external databases:

.. code-block:: bash

   # Import from database
   provchain-org import db --connection "postgresql://user:pass@host:port/db" \
                           --query "SELECT * FROM supply_chain_data" \
                           --output data.ttl

### File System Integration

Monitor directories for new data:

.. code-block:: bash

   # Watch directory for new files
   provchain-org watch /path/to/data/directory --pattern "*.ttl" --auto-import

### IoT Integration

Integrate with IoT devices:

.. code-block:: bash

   # Import from IoT device
   provchain-org import iot --device-id "sensor_001" --output sensor_data.ttl

Monitoring and Maintenance
--------------------------

Monitoring data management operations and maintaining system health.

### Monitoring Commands

.. code-block:: bash

   # Check data storage usage
   provchain-org storage usage
   
   # Monitor import/export operations
   provchain-org monitor operations
   
   # Check data integrity
   provchain-org integrity check

### Maintenance Tasks

Regular maintenance tasks:

.. code-block:: bash

   # Compact database
   provchain-org maintenance compact
   
   # Optimize performance
   provchain-org maintenance optimize
   
   # Clean temporary files
   provchain-org maintenance cleanup

### Health Checks

Verify system health:

.. code-block:: bash

   # Run health check
   provchain-org health check
   
   # Detailed health report
   provchain-org health report --output health_report.json

Best Practices
--------------

Guidelines for effective data management.

### Data Modeling Best Practices

1. **Use consistent URIs**: Establish naming conventions for entities
2. **Include provenance**: Track data origin and transformations
3. **Use standard vocabularies**: Leverage existing ontologies when possible
4. **Document assumptions**: Include metadata about data collection methods

### Import Best Practices

1. **Validate before import**: Check data quality before adding to blockchain
2. **Batch large imports**: Process large datasets in smaller chunks
3. **Monitor resource usage**: Watch memory and CPU during imports
4. **Keep backups**: Maintain copies of original data

### Query Best Practices

1. **Use indexes**: Structure queries to leverage indexing
2. **Limit result sets**: Avoid returning excessive data
3. **Cache frequent queries**: Store results of common queries
4. **Optimize patterns**: Use efficient SPARQL patterns

### Security Best Practices

1. **Encrypt sensitive data**: Protect confidential information
2. **Control access**: Implement role-based access control
3. **Audit changes**: Track all data modifications
4. **Regular backups**: Maintain secure copies of important data

Troubleshooting
---------------

Common issues and solutions for data management.

### Import Issues

**Problem**: Import fails with syntax error
**Solution**: Validate RDF syntax using online tools or command line validators

**Problem**: Import rejected due to ontology validation
**Solution**: Check data against the supply chain ontology and correct violations

**Problem**: Import takes too long
**Solution**: Break large files into smaller chunks and use parallel processing

### Export Issues

**Problem**: Export returns empty results
**Solution**: Check query filters and ensure data exists matching criteria

**Problem**: Export file is too large
**Solution**: Use pagination or more specific filters to limit results

**Problem**: Export format not supported
**Solution**: Check supported formats and convert data if necessary

### Performance Issues

**Problem**: Slow query performance
**Solution**: Add indexes, optimize queries, and check system resources

**Problem**: High memory usage during import
**Solution**: Reduce batch size and monitor memory consumption

**Problem**: Disk space issues
**Solution**: Clean up temporary files and archive old data

### Security Issues

**Problem**: Unauthorized data access
**Solution**: Review and update access control policies

**Problem**: Data corruption
**Solution**: Verify data integrity and restore from backups if necessary

Support
-------

For additional help with data management:

1. Check the online documentation
2. Join our community forum
3. Contact support at support@provchain-org.com
4. Refer to the API documentation for programmatic data management

.. note::
   Effective data management is crucial for successful supply chain traceability. Follow these guidelines and best practices to ensure your data is accurate, secure, and accessible for all stakeholders.