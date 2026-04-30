Your First Supply Chain Application
===================================

This tutorial will guide you through building your first supply chain traceability application with ProvChainOrg. You'll learn how to track a product from farm to consumer using semantic blockchain technology.

What You'll Build
-----------------

By the end of this tutorial, you'll have:

- ✅ A working ProvChainOrg node
- ✅ Supply chain data stored as RDF graphs
- ✅ SPARQL queries to trace product provenance
- ✅ A web interface to visualize the supply chain

Prerequisites
-------------

Before starting, ensure you have:

- **Rust 1.70+**: `rustc --version`
- **Git**: For cloning the repository
- **Basic terminal knowledge**: Running commands and editing files

.. note::
   This tutorial takes approximately 30 minutes to complete.

Step 1: Installation and Setup
-------------------------------

Clone and Build ProvChainOrg
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Clone the repository
   git clone https://github.com/anusornc/provchain-org.git
   cd provchain-org

   # Build the project
   cargo build --release

   # Verify installation
   cargo run -- --help

You should see the ProvChainOrg command-line interface help.

Explore the Demo Data
~~~~~~~~~~~~~~~~~~~~~

ProvChainOrg comes with sample supply chain data:

.. code-block:: bash

   # View the sample RDF data
   cat demo_data/store.ttl

This file contains a complete supply chain scenario with:
- Product batches (organic tomatoes)
- Farm information
- Processing activities
- Environmental monitoring
- Quality certifications

Step 2: Run Your First Demo
----------------------------

Start with the Built-in Demo
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Run the complete demo
   cargo run demo

This command:
1. Initializes a new blockchain
2. Loads the sample supply chain data
3. Creates blocks with RDF graphs
4. Demonstrates SPARQL queries
5. Shows traceability results

Understanding the Output
~~~~~~~~~~~~~~~~~~~~~~~~

The demo output shows:

.. code-block:: text

   🚀 ProvChainOrg Demo Starting...
   📦 Loading supply chain data...
   🔗 Creating blockchain blocks...
   📊 Running traceability queries...
   
   ✅ Found 3 product batches
   ✅ Traced complete supply chain
   ✅ Verified environmental conditions

Step 3: Explore SPARQL Queries
-------------------------------

Basic Product Query
~~~~~~~~~~~~~~~~~~~

Query all products in the blockchain:

.. code-block:: bash

   # Run a basic SPARQL query
   cargo run -- query queries/trace_by_batch_ontology.sparql

This query finds all product batches and their basic information.

Custom Queries
~~~~~~~~~~~~~~

Create your own SPARQL query file:

.. code-block:: bash

   # Create a new query file
   cat > my_query.sparql << 'EOF'
   PREFIX : <http://example.org/supply-chain#>
   PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

   SELECT ?batch ?product ?farm ?date WHERE {
     ?batch a :ProductBatch ;
            :product ?product ;
            :originFarm ?farm ;
            :harvestDate ?date .
   }
   ORDER BY ?date
   EOF

   # Run your custom query
   cargo run -- query my_query.sparql

Environmental Monitoring Query
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Track environmental conditions during transport:

.. code-block:: bash

   # Create environmental monitoring query
   cat > environmental_query.sparql << 'EOF'
   PREFIX : <http://example.org/supply-chain#>

   SELECT ?batch ?temperature ?humidity ?location ?timestamp WHERE {
     ?batch :transportedThrough ?transport .
     ?transport :environmentalCondition ?condition .
     ?condition :temperature ?temperature ;
                :humidity ?humidity ;
                :location ?location ;
                :recordedAt ?timestamp .
   }
   ORDER BY ?timestamp
   EOF

   # Run the environmental query
   cargo run -- query environmental_query.sparql

Step 4: Add Your Own Data
-------------------------

Create Custom Supply Chain Data
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Create a new RDF file with your own supply chain scenario:

.. code-block:: bash

   # Create your own supply chain data
   cat > my_supply_chain.ttl << 'EOF'
   @prefix : <http://example.org/supply-chain#> .
   @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

   # Your farm
   :MyFarm a :OrganicFarm ;
           :name "My Organic Farm" ;
           :location "Your Location" ;
           :certificationNumber "ORG-2024-MY-FARM" .

   # Your product batch
   :MyBatch001 a :ProductBatch ;
               :product :OrganicCarrots ;
               :batchId "CARROT-2024-001" ;
               :harvestDate "2024-01-20"^^xsd:date ;
               :originFarm :MyFarm ;
               :batchSize "200kg"^^xsd:decimal ;
               :certifiedOrganic true .

   # Processing activity
   :MyProcessing a :ProcessingActivity ;
                 :processedBatch :MyBatch001 ;
                 :processType :Washing ;
                 :timestamp "2024-01-21T09:00:00Z"^^xsd:dateTime ;
                 :performedBy :MyProcessingPlant .

   # Environmental monitoring
   :MyTransport :environmentalCondition [
       a :EnvironmentalCondition ;
       :temperature "4.0°C"^^xsd:decimal ;
       :humidity "80%"^^xsd:decimal ;
       :location :ColdStorage ;
       :recordedAt "2024-01-22T14:30:00Z"^^xsd:dateTime
   ] .
   EOF

Add Data to Blockchain
~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Add your data to the blockchain
   cargo run -- add-file my_supply_chain.ttl

   # Verify the data was added
   cargo run -- query my_query.sparql

Step 5: Start the Web Interface
-------------------------------

Launch the Web Server
~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Start the web interface
   cargo run --bin demo_ui

The web interface will start on `http://localhost:8080`.

Explore the Web Interface
~~~~~~~~~~~~~~~~~~~~~~~~~

Open your browser and navigate to `http://localhost:8080`. You'll see:

1. **Dashboard**: Overview of blockchain status
2. **Query Interface**: Interactive SPARQL query editor
3. **Block Explorer**: Browse blockchain blocks
4. **Supply Chain Viewer**: Visualize product journeys

Try Interactive Queries
~~~~~~~~~~~~~~~~~~~~~~~

In the web interface:

1. Go to the "Query" tab
2. Enter a SPARQL query:

   .. code-block:: sparql

      SELECT ?batch ?product ?farm WHERE {
        ?batch a :ProductBatch ;
               :product ?product ;
               :originFarm ?farm .
      }

3. Click "Execute Query"
4. View the results in table format

Step 6: Advanced Features
-------------------------

Blockchain Validation
~~~~~~~~~~~~~~~~~~~~~

Verify blockchain integrity:

.. code-block:: bash

   # Validate the entire blockchain
   cargo run -- validate

   # Check specific block
   cargo run -- validate --block 1

Export Data
~~~~~~~~~~~

Export blockchain data in different formats:

.. code-block:: bash

   # Export as Turtle (RDF)
   cargo run -- export --format turtle --output my_blockchain.ttl

   # Export as JSON-LD
   cargo run -- export --format jsonld --output my_blockchain.jsonld

Network Operations
~~~~~~~~~~~~~~~~~~

Connect to other ProvChainOrg nodes:

.. code-block:: bash

   # Start as network node
   cargo run -- network --port 8081

   # Connect to another node
   cargo run -- network --connect ws://localhost:8081

Step 7: Understanding the Results
---------------------------------

Data Structure
~~~~~~~~~~~~~~

Your supply chain data is stored as RDF triples in blockchain blocks:

.. code-block:: turtle

   # Each block contains a named graph
   :Block1 {
     :MyBatch001 a :ProductBatch ;
                 :product :OrganicCarrots ;
                 :originFarm :MyFarm .
     
     :MyFarm a :OrganicFarm ;
             :location "Your Location" .
   }

Traceability Queries
~~~~~~~~~~~~~~~~~~~~

You can now trace:

- **Forward**: Where did this batch go?
- **Backward**: Where did this product come from?
- **Environmental**: What conditions was it stored under?
- **Quality**: What certifications does it have?

Blockchain Benefits
~~~~~~~~~~~~~~~~~~~

Your data now has:

- ✅ **Immutability**: Cannot be changed once recorded
- ✅ **Transparency**: All data is queryable
- ✅ **Verification**: Cryptographically secured
- ✅ **Interoperability**: Standard RDF/SPARQL formats

Next Steps
----------

Congratulations! You've built your first supply chain application with ProvChainOrg. Here's what to explore next:

**Learn More Concepts**
- :doc:`../foundational/intro-to-rdf-blockchain` - Understand the technology
- :doc:`../foundational/sparql-queries` - Master SPARQL querying
- :doc:`../foundational/ontologies-and-validation` - Learn about data validation

**Build Advanced Applications**
- :doc:`food-traceability` - Complete food safety system
- :doc:`pharmaceutical-tracking` - Drug authentication
- :doc:`api-integration` - Integrate with existing systems

**Development Resources**
- :doc:`../stack/client-apis` - REST and SPARQL APIs
- :doc:`../stack/development-frameworks` - Development tools
- :doc:`../api/rest-api` - Complete API reference

Troubleshooting
---------------

Common Issues
~~~~~~~~~~~~~

**Build Errors**
   Ensure you have Rust 1.70+ installed: `rustup update`

**Query Errors**
   Check SPARQL syntax and ensure prefixes are defined

**Network Issues**
   Verify ports are available and firewall settings

**Data Validation Errors**
   Ensure RDF data follows the ontology schema

Getting Help
~~~~~~~~~~~~

- **GitHub Issues**: Report bugs and ask questions
- **Documentation**: Comprehensive guides and API reference
- **Community**: Join discussions and share experiences

.. note::
   This tutorial covered the basics of ProvChainOrg. The platform supports much more advanced features including distributed networks, complex ontologies, and enterprise integrations.

Summary
-------

In this tutorial, you:

1. ✅ Installed and configured ProvChainOrg
2. ✅ Ran the demo and explored sample data
3. ✅ Created and executed SPARQL queries
4. ✅ Added your own supply chain data
5. ✅ Used the web interface for visualization
6. ✅ Learned about blockchain validation and export

You now have a working semantic blockchain for supply chain traceability that provides transparency, verifiability, and queryability that traditional systems cannot match.

The combination of blockchain security with semantic web technologies opens up new possibilities for supply chain transparency, regulatory compliance, and consumer trust.
