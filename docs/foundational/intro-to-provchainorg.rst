Introduction to ProvChainOrg
============================

.. warning::
   **Current evidence boundary (2026-08-31).** This page is a legacy conceptual
   introduction, not evidence of production readiness. Public RDF payloads,
   Oxigraph/SPARQL querying, and selected ``src/ontology/*`` checks are the
   implemented foundation. The complete-envelope journal/replay, atomic Final
   Admission, authenticated membership, reproducible three-node PoA, full
   package-declared SHACL enforcement, durable privacy, bounded ProvChain
   bridge, and corrected end-to-end evidence are pending. PBFT,
   heterogeneous/SPV bridging, human usability, operational deployment, and
   production-pilot controls are future work. Current status and evidence are
   tracked in ``docs/architecture/SHARED_ONTOLOGY_NETWORK_WORKING_PLAN.md`` and
   ``docs/paper_submission/PAPER_EVIDENCE_INDEX.md``; follow ``AGENTS.md`` for
   repository work.

ProvChainOrg is a semantic-ledger research prototype that combines
cryptographically linked public provenance records with the expressiveness and
queryability of RDF (Resource Description Framework) graphs. Supply-chain
domains are reference packages rather than the platform boundary.

What is ProvChainOrg?
---------------------

Think of ProvChainOrg as "blockchain with meaning." While traditional blockchains store opaque data that requires specialized tools to interpret, ProvChainOrg stores semantic data that can be queried and understood using standard web technologies.

.. code-block:: bash

   # Traditional blockchain: opaque data
   Block 1: 0x4a7b2c8f9e1d3a5b...
   
   # ProvChainOrg: semantic data
   Block 1: ProductBatch "Organic Tomatoes" 
            from Farm "Green Valley"
            processed at "2024-01-15"
            temperature "2-4°C"

Key Concepts
------------

**RDF-Native Storage**
   Public provenance payloads can be represented as RDF triples and projected
   into Oxigraph. This does not mean every ledger/control/private datum is RDF.

**SPARQL Queries**
   Query the current Oxigraph public-provenance projection using SPARQL.

**Ontology Validation**
   Selected ontology-aware paths perform focused checks; the universal
   package-declared SHACL gate at Final Admission is pending.

**Traceability Reference Focus**
   Supply-chain packages exercise product, process, and provenance models
   without defining the platform boundary.

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
     - RDF/SPARQL-based model; no blanket standards-conformance claim
   * - Auditability
     - ❌ Requires specialized tools
     - Queryable public provenance; durable journal evidence pending

Real-World Example
~~~~~~~~~~~~~~~~~~

Imagine tracking a batch of organic tomatoes:

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

This example illustrates the intended benefit of an RDF/SPARQL model without
making a universal claim about other blockchain systems.

Core Features
-------------

🔗 **Public RDF Provenance**
   Carry public RDF payloads in cryptographically linked prototype blocks

🔍 **SPARQL Query Engine**
   Query the current Oxigraph projection using SPARQL

🧠 **Ontology Integration**
   Apply selected production-path ontology checks while full package enforcement remains pending

📊 **Supply Chain Traceability**
   Model and query provenance in supply-chain reference packages

🌐 **Standards-Facing Model**
   Reuse RDF, SPARQL, PROV-O, and selected ontology concepts without claiming conformance certification

🔒 **Cryptographic Security**
   Use cryptographic components in the prototype while accepted end-to-end guarantees remain pending

Getting Started
---------------

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

   This runs a reference demo, not end-to-end conformance evidence.

2. **Try a Query**

   .. code-block:: bash

      cargo run -- query src/semantic/queries/trace_by_batch_ontology.sparql

   This invokes an existing legacy query fixture and demonstrates CLI syntax.

3. **Explore the Data**

   .. code-block:: bash

      # Inspect the current chain representation
      cargo run -- dump

   This prints the current chain representation; it is not a Verified Journal
   Replay or durable-evidence check.

Use Cases
---------

ProvChainOrg uses these domains as research and reference-package scenarios:

**Food Safety & Traceability**
   Track food products from farm to table with environmental monitoring and quality assurance.

**Pharmaceutical Supply Chains**
   Explore medicine-batch provenance; authenticity and anti-counterfeit
   outcomes require application and operational controls beyond this prototype.

**Luxury Goods Authentication**
   Explore provenance evidence that an application may use for authenticity checks.

**Regulatory Workflows**
   Explore queryable provenance for audit workflows without claiming compliance
   with a specific regulation.

**Sustainability Tracking**
   Monitor environmental impact and sustainability metrics across supply chains.

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

1. **Learn the Fundamentals**: Continue with :doc:`intro-to-rdf-blockchain` to understand the core technology
2. **Explore Use Cases**: Read :doc:`intro-to-supply-chain-traceability` for practical applications
3. **Start Building**: Jump to :doc:`../tutorials/first-supply-chain` for a hands-on tutorial
4. **Check Current Architecture Status**: Read
   ``docs/architecture/SHARED_ONTOLOGY_NETWORK_WORKING_PLAN.md`` before relying on a component or
   end-to-end capability claim

.. note::
   ProvChainOrg is based on the GraphChain research concept and extends it with
   public-RDF/Oxigraph/SPARQL foundations, selected ontology checks, and
   reference-package scenarios. Production-readiness remains outside the
   current evidence boundary.

Community & Support
--------------------

- **GitHub Repository**: `ProvChainOrg on GitHub <https://github.com/anusornc/provchain-org>`_
- **Documentation**: You're reading it! Use the navigation to explore specific topics
- **Issues**: Report bugs and request features on GitHub Issues
- **Discussions**: Join community discussions for Q&A and feature requests

ProvChainOrg is open source and welcomes contributions from developers, researchers, and supply chain professionals.
