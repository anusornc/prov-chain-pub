Introduction to RDF Blockchain
==============================

RDF Blockchain is the core innovation that makes ProvChainOrg unique. Unlike traditional blockchains that store opaque binary data, RDF blockchain stores semantic data as RDF (Resource Description Framework) graphs, making every piece of information queryable and meaningful.

.. note::
   This chapter is a conceptual primer. Examples of federation, automatic validation, evolution,
   and scaling illustrate design intent and are not current end-to-end evidence. For the current
   implementation boundary, read ``docs/reviews/THESIS_CODE_ALIGNMENT_REVIEW_2026-07-10.md`` and
   ``docs/architecture/SHARED_ONTOLOGY_NETWORK_WORKING_PLAN.md``.

What is RDF?
------------

RDF (Resource Description Framework) is a W3C standard for representing information about resources on the web. It uses a simple subject-predicate-object structure called "triples" to express relationships.

Basic RDF Structure
~~~~~~~~~~~~~~~~~~~

.. code-block:: turtle

   # Traditional data: "Product ABC123 was harvested on 2024-01-15"
   # RDF representation:
   :ProductABC123 :harvestDate "2024-01-15"^^xsd:date .
   :ProductABC123 :originFarm :GreenValleyFarm .
   :ProductABC123 a :OrganicTomatoes .

Each line is a "triple" with:
- **Subject**: What we're talking about (`:ProductABC123`)
- **Predicate**: The relationship (`:harvestDate`, `:originFarm`)
- **Object**: The value or related resource (`"2024-01-15"`, `:GreenValleyFarm`)

Why RDF for Blockchain?
-----------------------

Traditional Blockchain Limitations
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: json

   // Traditional blockchain data - opaque and hard to query
   {
     "block": 123,
     "data": "0x4a7b2c8f9e1d3a5b8c2f7e9a1b4d6c8e...",
     "hash": "0x9f2e8d7c6b5a4f3e2d1c9b8a7f6e5d4c..."
   }

**Problems:**
- Data is opaque - requires specialized tools to interpret
- No standard query language
- Difficult to establish relationships between data
- Manual validation required

RDF Blockchain Advantages
~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: turtle

   # RDF blockchain data - semantic and queryable
   :Block123 {
     :ProductBatch456 a :OrganicTomatoes ;
                      :harvestDate "2024-01-15"^^xsd:date ;
                      :originFarm :GreenValleyFarm ;
                      :certifiedOrganic true ;
                      :batchSize "500kg"^^xsd:decimal .
     
     :GreenValleyFarm a :OrganicFarm ;
                      :location "California, USA" ;
                      :certificationNumber "ORG-2023-456" .
   }

**Benefits:**
- ✅ Human-readable semantic data
- ✅ Standard SPARQL query language
- ✅ Automatic relationship discovery
- ✅ Ontology-based validation
- ✅ Interoperable with web standards

How RDF Blockchain Works
------------------------

Block Structure
~~~~~~~~~~~~~~~

Each block in ProvChainOrg contains:

1. **Block Header**: Traditional blockchain metadata (hash, timestamp, previous block)
2. **RDF Graph**: Named graph containing semantic data
3. **Canonical Hash**: Deterministic hash of the RDF content

.. code-block:: rust

   pub struct Block {
       pub header: BlockHeader,
       pub rdf_graph: String,        // Turtle-formatted RDF
       pub canonical_hash: String,   // Hash of canonicalized RDF
   }

RDF Canonicalization
~~~~~~~~~~~~~~~~~~~~

The key challenge is creating deterministic hashes from RDF graphs, which can have multiple valid representations:

.. code-block:: turtle

   # These are semantically identical but syntactically different:
   
   # Representation 1:
   :Product123 :name "Tomatoes" ;
               :farm :GreenValley .
   
   # Representation 2:
   :Product123 :farm :GreenValley .
   :Product123 :name "Tomatoes" .

ProvChainOrg uses advanced canonicalization algorithms to ensure semantically equivalent graphs produce identical hashes.

Querying RDF Blockchain
------------------------

SPARQL Queries
~~~~~~~~~~~~~~

Query the entire blockchain using standard SPARQL:

.. code-block:: sparql

   # Find all organic products from a specific farm
   SELECT ?product ?batch ?date WHERE {
     ?batch a :ProductBatch ;
            :product ?product ;
            :originFarm :GreenValleyFarm ;
            :harvestDate ?date ;
            :certifiedOrganic true .
   }

Cross-Block Queries
~~~~~~~~~~~~~~~~~~~

Query relationships across multiple blocks:

.. code-block:: sparql

   # Trace a product's complete journey
   SELECT ?activity ?location ?timestamp WHERE {
     :TomatoBatch123 :involvedInActivity ?activity .
     ?activity :performedAt ?location ;
               :timestamp ?timestamp .
   }
   ORDER BY ?timestamp

Federated Queries
~~~~~~~~~~~~~~~~~~

Query across multiple ProvChainOrg nodes:

.. code-block:: sparql

   # Query multiple supply chain participants
   SELECT ?supplier ?product ?certification WHERE {
     SERVICE <http://supplier1.provchain.org/sparql> {
       ?product :suppliedBy ?supplier ;
                :certification ?certification .
     }
   }

Ontology Integration
--------------------

Automatic Validation
~~~~~~~~~~~~~~~~~~~~

All RDF data is validated against formal ontologies:

.. code-block:: turtle

   # Ontology definition
   :ProductBatch a owl:Class ;
                 rdfs:subClassOf :SupplyChainEntity .
   
   :harvestDate a owl:DatatypeProperty ;
                rdfs:domain :ProductBatch ;
                rdfs:range xsd:date .

   # Valid data
   :Batch123 a :ProductBatch ;
             :harvestDate "2024-01-15"^^xsd:date .  # ✅ Valid
   
   # Invalid data
   :Batch456 a :ProductBatch ;
             :harvestDate "not-a-date" .  # ❌ Invalid - wrong data type

Schema Evolution
~~~~~~~~~~~~~~~~

Ontologies can evolve while maintaining backward compatibility:

.. code-block:: turtle

   # Version 1.0 ontology
   :ProductBatch :harvestDate ?date .
   
   # Version 2.0 ontology - adds new properties
   :ProductBatch :harvestDate ?date ;
                 :sustainabilityScore ?score ;  # New property
                 :carbonFootprint ?footprint .  # New property

Practical Examples
------------------

Food Traceability
~~~~~~~~~~~~~~~~~

.. code-block:: turtle

   # Complete food traceability chain
   :TomatoBatch123 a :ProductBatch ;
                   :product :OrganicTomatoes ;
                   :harvestDate "2024-01-15"^^xsd:date ;
                   :originFarm :GreenValleyFarm ;
                   :processedAt :ProcessingPlant456 ;
                   :packagedAt "2024-01-16"^^xsd:date ;
                   :shippedTo :Retailer789 .

   :ProcessingPlant456 :uhtProcessing [
       :temperature "135°C"^^xsd:decimal ;
       :duration "2 seconds"^^xsd:duration ;
       :timestamp "2024-01-16T10:30:00Z"^^xsd:dateTime
   ] .

Environmental Monitoring
~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: turtle

   # Environmental conditions during transport
   :Transport123 :environmentalCondition [
       a :TemperatureReading ;
       :temperature "3.5°C"^^xsd:decimal ;
       :humidity "85%"^^xsd:decimal ;
       :location :Warehouse456 ;
       :recordedAt "2024-01-17T14:30:00Z"^^xsd:dateTime
   ] .

Quality Certifications
~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: turtle

   # Quality and certification data
   :Batch123 :qualityCheck [
       a :OrganicCertification ;
       :certifiedBy :USDAOrganic ;
       :certificationNumber "ORG-2024-123" ;
       :validUntil "2025-01-15"^^xsd:date ;
       :testResults :PassedAllTests
   ] .

Benefits for Developers
-----------------------

Standard Tools
~~~~~~~~~~~~~~

Use existing semantic web tools:

.. code-block:: bash

   # Query with any SPARQL client
   curl -X POST http://localhost:8080/sparql \
        -H "Content-Type: application/sparql-query" \
        -d "SELECT * WHERE { ?s ?p ?o } LIMIT 10"

Rich Ecosystem
~~~~~~~~~~~~~~

Leverage the semantic web ecosystem:

- **RDF Libraries**: Available in all major programming languages
- **SPARQL Endpoints**: Standard query interface
- **Ontology Tools**: Protégé, TopBraid, etc.
- **Visualization**: Graph visualization tools
- **Reasoning**: Automatic inference and validation

Interoperability
~~~~~~~~~~~~~~~~

Easy integration with existing systems:

.. code-block:: python

   # Python example using rdflib
   from rdflib import Graph
   
   # Load blockchain data
   g = Graph()
   g.parse("http://provchain.org/block/123", format="turtle")
   
   # Query with SPARQL
   results = g.query("""
       SELECT ?product ?farm WHERE {
           ?batch :product ?product ;
                  :originFarm ?farm .
       }
   """)

Performance Considerations
--------------------------

Efficient Storage
~~~~~~~~~~~~~~~~~

- **Compression**: RDF data compresses well
- **Indexing**: SPARQL engines provide efficient indexing
- **Caching**: Frequently accessed data can be cached

Scalability
~~~~~~~~~~~

- **Sharding**: Distribute data across multiple nodes
- **Federation**: Query across distributed endpoints
- **Materialization**: Pre-compute common queries

Next Steps
----------

Now that you understand RDF blockchain fundamentals:

1. **Learn Supply Chain Applications**: :doc:`intro-to-supply-chain-traceability`
2. **Compare with Traditional Systems**: :doc:`semantic-web-vs-traditional-blockchain`
3. **Understand SPARQL Queries**: :doc:`sparql-queries`
4. **Review the Current Architecture Boundary**: `Shared Ontology Network Working Plan <../architecture/SHARED_ONTOLOGY_NETWORK_WORKING_PLAN.md>`_

.. note::
   RDF blockchain represents a paradigm shift from opaque data storage to semantic, queryable information systems. This enables unprecedented transparency and interoperability in supply chain applications.

Technical Resources
-------------------

- **W3C RDF Specification**: `RDF 1.1 Concepts <https://www.w3.org/TR/rdf11-concepts/>`_
- **SPARQL Specification**: `SPARQL 1.1 Query Language <https://www.w3.org/TR/sparql11-query/>`_
- **Turtle Format**: `RDF 1.1 Turtle <https://www.w3.org/TR/turtle/>`_
- **OWL Ontologies**: `OWL 2 Web Ontology Language <https://www.w3.org/TR/owl2-overview/>`_
