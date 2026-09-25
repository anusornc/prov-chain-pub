Introduction to ProvChainOrg
===========================

.. warning::
   **Current evidence boundary (2026-08-31).** This page is a legacy,
   feature-oriented introduction to a research prototype. The implemented
   foundation is public RDF payload handling with Oxigraph/SPARQL plus selected
   ontology-aware checks. The complete-envelope journal and replay, atomic
   Final Admission, authenticated membership, reproducible three-node PoA,
   full package-declared SHACL enforcement, durable privacy lifecycle, bounded
   ProvChain bridge, and corrected end-to-end evidence are pending. PBFT,
   heterogeneous/SPV bridging, human usability, operational deployment, and
   production-pilot controls are future work. Consult
   ``docs/architecture/SHARED_ONTOLOGY_NETWORK_WORKING_PLAN.md`` and
   ``docs/paper_submission/PAPER_EVIDENCE_INDEX.md`` before relying on a
   capability claim; ``AGENTS.md`` governs repository work.

Welcome to ProvChainOrg, a semantic-ledger research prototype that combines
cryptographically linked public provenance records with the expressiveness and
queryability of RDF (Resource Description Framework) graphs.

.. raw:: html

   <div class="hero-section">
     <div class="hero-content">
       <h1>Introduction to ProvChainOrg</h1>
       <p class="hero-subtitle">Queryable public-RDF provenance in a semantic-ledger research prototype</p>
       <div class="hero-badges">
         <span class="badge badge-user">User Guide</span>
         <span class="badge badge-introduction">Introduction</span>
         <span class="badge badge-concepts">Concepts</span>
         <span class="badge badge-overview">Overview</span>
       </div>
     </div>
   </div>

What is ProvChainOrg?
---------------------

ProvChainOrg explores the combination of two technologies:

1. **Cryptographically linked ledger structures**: The repository includes local/prototype block and PoA components; durable commitment and three-node convergence remain pending
2. **Semantic Web Technologies**: Enables rich, queryable data with formal semantics and relationships

Unlike traditional blockchains that store opaque data, ProvChainOrg stores semantic data that can be queried and understood using standard web technologies. This makes it particularly well-suited for supply chain traceability applications where transparency, verifiability, and semantic richness are essential.

Key Concepts
------------

**RDF-Native Storage**
   Public provenance payloads can be represented as RDF triples and projected
   into Oxigraph for querying. Private and control data are not covered by an
   "all data is RDF" claim.

**SPARQL Queries**
   Query the current Oxigraph provenance projection using SPARQL, the standard
   query language for RDF data.

**Ontology Validation**
   Selected ontology-aware paths perform focused checks. Universal,
   package-declared full-state SHACL enforcement at Final Admission remains a
   target milestone and no industry-conformance certification is implied.

**Traceability Reference Focus**
   Supply-chain packages exercise product, process, provenance, and
   environmental-condition models without defining the platform boundary.

Why Use ProvChainOrg?
---------------------

Traditional Solutions vs. ProvChainOrg
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 30 35 35

   * - Requirement
     - Traditional Blockchain
     - ProvChainOrg
   * - Data Transparency
     - ❌ Opaque data
     - ✅ Semantic, queryable data
   * - Supply Chain Queries
     - ❌ Complex custom code
     - ✅ Standard SPARQL queries
   * - Data Validation
     - ❌ Manual validation
     - Selected ontology checks; universal Final Admission gate pending
   * - Interoperability
     - ❌ Vendor-specific formats
     - RDF/SPARQL-based data and query model; no blanket conformance claim
   * - Auditability
     - ❌ Requires specialized tools
     - Queryable public provenance; durable journal evidence pending

Real-World Example
~~~~~~~~~~~~~~~~~~

Imagine tracking a batch of organic tomatoes through the supply chain:

.. code-block:: sparql

   # Find all products from a specific farm
   SELECT ?product ?batch ?date WHERE {
     ?batch a :ProductBatch ;
            :product ?product ;
            :originFarm :GreenValleyFarm ;
            :harvestDate ?date .
   }

   # Trace temperature history during transport
   SELECT ?location ?temperature ?timestamp WHERE {
     :TomatoBatch123 :transportedThrough ?transport .
     ?transport :atLocation ?location ;
                :environmentalCondition ?condition .
     ?condition :temperature ?temperature ;
                :recordedAt ?timestamp .
   }

This example illustrates the kind of query the RDF/SPARQL model is intended to
support without a bespoke query language.

Core Features
-------------

🔗 **Public RDF Provenance**
   Carry public RDF payloads in cryptographically linked prototype blocks

🔍 **SPARQL Query Engine**
   Query the current Oxigraph projection using SPARQL

🧠 **Ontology Integration**
   Apply selected production-path ontology checks while the complete package gate remains pending

📊 **Supply Chain Traceability**
   Model and query provenance in supply-chain reference packages

🌐 **Standards-Facing Model**
   Reuse RDF, SPARQL, PROV-O, and selected ontology concepts without claiming full conformance certification

🔒 **Cryptographic Security**
   Use established cryptographic components in the prototype; the accepted end-to-end security lifecycle remains pending

🌡️ **Environmental Monitoring**
   Track temperature, humidity, and other conditions throughout the supply chain

📋 **Regulatory Workflows**
   Explore queryable provenance as input to regulatory and audit workflows; compliance is not certified

User Interface Overview
----------------------

The repository includes web/API and frontend scaffolding. Available screens and
routes depend on the selected runtime and should be verified before a demo;
the intended interface areas include:

**Dashboard**
   Get an overview of your blockchain status, recent activities, and key metrics at a glance.

**Data Entry**
   Easily add new supply chain data through forms or bulk import functionality.

**Query Interface**
   Run SPARQL queries directly through the web interface with syntax highlighting and auto-completion.

**Reporting Tools**
   Generate standard and custom reports with visualizations and export options.

**Administration Panel**
   Manage users, configure system settings, and monitor system health.

Target Industries
----------------

ProvChainOrg uses the following as research or reference-package scenarios:

**Food & Agriculture**
   Track food products from farm to table with environmental monitoring and quality assurance.

**Pharmaceuticals**
   Explore medicine-batch provenance and evidence that applications may use in
   authenticity or anti-counterfeit workflows.

**Luxury Goods**
   Explore provenance inputs to application-level authenticity checks.

**Manufacturing**
   Track components and materials through complex manufacturing processes.

**Logistics & Transportation**
   Monitor environmental conditions and handling throughout transport.

**Regulatory Workflows**
   Explore provenance records that may support regulatory workflows; no
   regulatory-compliance result is claimed.

Getting Started
--------------

Quick Installation
~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Prerequisites: Rust 1.87+
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Clone and build
   git clone https://github.com/anusornc/provchain-org.git
   cd provchain-org
   cargo build --release

First Steps
~~~~~~~~~~~

1. **Run the Demo**

   .. code-block:: bash

      cargo run -- demo

   This runs a reference demo; it is not proof of the pending end-to-end
   guarantees listed above.

2. **Try a Query**

   .. code-block:: bash

      cargo run -- query src/semantic/queries/trace_by_batch_ontology.sparql

   This invokes an existing legacy query fixture. It demonstrates CLI syntax,
   not the package-declared production semantic contract.

3. **Explore the Web Interface**

   .. code-block:: bash

      cargo run --example demo_ui

   Open your browser to http://localhost:8080 to explore the web interface.

Use Cases
---------

ProvChainOrg is being evaluated against several reference use cases:

**Cross-Organization Provenance Views**
   Model and query linked public provenance across reference workflows; data
   completeness is not guaranteed by the current prototype.

**Quality Assurance**
   Monitor environmental conditions, processing parameters, and quality checks throughout the supply chain.

**Counterfeit Prevention**
   Explore provenance evidence that applications may use during authenticity
   checks; product authenticity is not established automatically.

**Regulatory Workflows**
   Explore queryable provenance for audit workflows without claiming that
   records meet any specific regulation.

**Sustainability Tracking**
   Monitor environmental impact and sustainability metrics across supply chains.

**Recall Management**
   Explore queries that help identify potentially affected products; operational
   isolation remains an application responsibility.

Architecture Overview
---------------------

ProvChainOrg consists of several key components:

.. code-block:: text

   ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
   │   Web Interface │    │   REST API      │    │   SPARQL API    │
   └─────────────────┘    └─────────────────┘    └─────────────────┘
            │                       │                       │
   ┌─────────────────────────────────────────────────────────────────┐
   │                    Core Blockchain Engine                      │
   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
   │  │ RDF Store   │  │ Ontology    │  │ Canonicalization        │ │
   │  │ (Oxigraph)  │  │ Validator   │  │ Engine                  │ │
   │  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
   └─────────────────────────────────────────────────────────────────┘
            │                       │                       │
   ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
   │   P2P Network   │    │   Consensus     │    │   Storage       │
   │   Protocol      │    │   Mechanism     │    │   Layer         │
   └─────────────────┘    └─────────────────┘    └─────────────────┘

Next Steps
----------

Now that you understand what ProvChainOrg is, you can:

1. **Learn the Fundamentals**: Continue with :doc:`basic-concepts` to understand core terminology
2. **Install the Platform**: Follow :doc:`installation` to set up your system
3. **Try Your First Steps**: Work through :doc:`first-steps` for hands-on experience
4. **Explore Use Cases**: Read about :doc:`food-safety` and other industry applications

.. note::
   ProvChainOrg is based on the GraphChain research concept and extends it with
   public-RDF/Oxigraph/SPARQL foundations, selected ontology checks, and
   reference-package use cases. It is not yet the thesis-defensible end-to-end
   reference system described in the warning above.

Community & Support
--------------------

- **Documentation**: You're reading it! Use the navigation to explore specific topics
- **GitHub Repository**: `ProvChainOrg on GitHub <https://github.com/anusornc/provchain-org>`_
- **Issues**: Report bugs and request features on GitHub Issues
- **Discussions**: Join community discussions for Q&A and feature requests

ProvChainOrg is open source and welcomes contributions from users, developers, and supply chain professionals.

.. raw:: html

   <div class="footer-note">
     <p><strong>Ready to get started?</strong> Continue with <a href="basic-concepts.html">Basic Concepts</a> or jump to <a href="installation.html">Installation</a>.</p>
   </div>
