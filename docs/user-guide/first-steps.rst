First Steps with ProvChainOrg
============================

This guide will walk you through your first experience with ProvChainOrg, from installation to running your first queries and exploring the web interface.

.. raw:: html

   <div class="hero-section">
     <div class="hero-content">
       <h1>First Steps with ProvChainOrg</h1>
       <p class="hero-subtitle">Your hands-on introduction to semantic blockchain traceability</p>
       <div class="hero-badges">
         <span class="badge badge-user">User Guide</span>
         <span class="badge badge-tutorial">Tutorial</span>
         <span class="badge badge-hands-on">Hands-on</span>
         <span class="badge badge-beginner">Beginner</span>
       </div>
     </div>
   </div>

What You'll Accomplish
----------------------

By the end of this guide, you'll have:

- ✅ Installed and configured ProvChainOrg
- ✅ Run the complete supply chain demo
- ✅ Executed your first SPARQL queries
- ✅ Explored the web interface
- ✅ Added your own supply chain data
- ✅ Understood the basic concepts

Prerequisites
-------------

Before starting, ensure you have:

- **Operating System**: Linux, macOS, or Windows with WSL
- **Rust 1.70+**: `rustc --version` (install with `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **Git**: For cloning the repository
- **Basic terminal knowledge**: Running commands and navigating directories
- **Web browser**: For accessing the web interface

.. note::
   This tutorial takes approximately 30 minutes to complete and requires approximately 2GB of disk space.

Step 1: Installation and Setup
------------------------------

Clone and Build ProvChainOrg
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Open your terminal and run these commands:

.. code-block:: bash

   # Clone the repository
   git clone https://github.com/anusornc/provchain-org.git
   cd provchain-org

   # Build the project (this may take several minutes)
   cargo build --release

   # Verify installation
   cargo run -- --help

You should see the ProvChainOrg command-line interface help, confirming that the installation was successful.

Explore the Demo Data
~~~~~~~~~~~~~~~~~~~~~

ProvChainOrg comes with sample supply chain data that demonstrates a complete traceability scenario:

.. code-block:: bash

   # View the sample RDF data
   cat demo_data/store.ttl

This file contains a complete supply chain scenario with:
- Product batches (organic tomatoes)
- Farm information
- Processing activities
- Environmental monitoring
- Quality certifications

The data is stored in Turtle format, a standard RDF serialization that's both human-readable and machine-processable.

Step 2: Run Your First Demo
----------------------------

Start with the Built-in Demo
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Run the complete demo
   cargo run demo

This command will:
1. Initialize a new blockchain
2. Load the sample supply chain data
3. Create blocks with RDF graphs
4. Demonstrate SPARQL queries
5. Show traceability results

Understanding the Demo Output
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The demo output shows:

.. code-block:: text

   🚀 ProvChainOrg Demo Starting...
   📦 Loading supply chain data...
   🔗 Creating blockchain blocks...
   📊 Running traceability queries...
   
   ✅ Found 3 product batches
   ✅ Traced complete supply chain
   ✅ Verified environmental conditions

The demo creates a blockchain with multiple blocks, each containing semantic data about different stages of the supply chain.

Step 3: Explore SPARQL Queries
-------------------------------

Basic Product Query
~~~~~~~~~~~~~~~~~~~

Query all products in the blockchain:

.. code-block:: bash

   # Run a basic SPARQL query
   cargo run -- query queries/trace_by_batch_ontology.sparql

This query finds all product batches and their basic information, demonstrating how you can query the entire blockchain using standard SPARQL.

Custom Queries
~~~~~~~~~~~~~~

Create and run your own SPARQL query:

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

   # Verify the data was added by running a query
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

1. **Dashboard**: Overview of blockchain status, recent blocks, and key metrics
2. **Query Interface**: Interactive SPARQL query editor with syntax highlighting
3. **Block Explorer**: Browse blockchain blocks and their contents
4. **Supply Chain Viewer**: Visualize product journeys and relationships
5. **Data Entry**: Forms for adding new supply chain data
6. **Reports**: Generate and export reports from your data

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
4. View the results in table format with sorting and filtering options

Step 6: Understanding the Results
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

Step 7: Advanced Features
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

Understanding the Concepts
--------------------------

Key Terms
~~~~~~~~~

**Block**: A cryptographically secured container for RDF data
**RDF**: Resource Description Framework - a standard model for data interchange
**SPARQL**: SPARQL Protocol and RDF Query Language - the standard query language for RDF
**Ontology**: A formal specification of shared conceptualization for a domain
**Canonicalization**: The process of converting data to a standard, deterministic form

Data Model
~~~~~~~~~~

ProvChainOrg uses a semantic data model based on:

1. **Entities**: Real-world objects like products, farms, and batches
2. **Properties**: Relationships and attributes of entities
3. **Classes**: Categories that entities belong to
4. **Named Graphs**: Logical containers for related triples

Best Practices
~~~~~~~~~~~~~~

1. **Data Quality**: Ensure consistent naming and formatting
2. **Ontology Compliance**: Follow the defined schema for your industry
3. **Regular Backups**: Export your blockchain data regularly
4. **Access Control**: Use appropriate user permissions
5. **Monitoring**: Regularly check system health and performance

Troubleshooting Common Issues
-----------------------------

Build Errors
~~~~~~~~~~~~

If you encounter build errors:

.. code-block:: bash

   # Update Rust toolchain
   rustup update

   # Clean and rebuild
   cargo clean
   cargo build --release

Query Errors
~~~~~~~~~~~~

For SPARQL query issues:

1. Check syntax and ensure all prefixes are defined
2. Verify that URIs and entity names match your data
3. Use the web interface query editor for syntax highlighting

Network Issues
~~~~~~~~~~~~~~

If the web interface doesn't start:

1. Verify port 8080 is available
2. Check firewall settings
3. Try a different port: `cargo run --bin demo_ui --port 8081`

Data Validation Errors
~~~~~~~~~~~~~~~~~~~~~~

If data validation fails:

1. Ensure RDF syntax is correct
2. Check that all required properties are present
3. Verify ontology compliance

Next Steps
----------

Congratulations! You've completed your first steps with ProvChainOrg. Here's what to explore next:

**User Guide Topics**
- :doc:`data-import` - Learn advanced data import techniques
- :doc:`query-interface` - Master the SPARQL query interface
- :doc:`reporting-tools` - Create custom reports and dashboards
- :doc:`user-management` - Manage users and permissions

**Industry Applications**
- :doc:`food-safety` - Complete food safety tracking system
- :doc:`pharmaceutical-tracking` - Drug authentication and traceability
- :doc:`compliance-reporting` - Regulatory compliance reporting

**Advanced Features**
- :doc:`api-basics` - Programmatic access to ProvChainOrg
- :doc:`ontology-extension` - Customizing the traceability ontology
- :doc:`network-configuration` - Setting up multi-node networks

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

.. raw:: html

   <div class="footer-note">
     <p><strong>Ready to dive deeper?</strong> Continue with <a href="data-import.html">Data Import</a> or explore <a href="../api/index.html">API Documentation</a> for programmatic access.</p>
   </div>
