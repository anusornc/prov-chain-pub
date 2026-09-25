ProvChainOrg Documentation
=========================

.. warning::
   **Current evidence boundary (2026-08-31).** This is a research-prototype
   documentation entry page, not a production-readiness or end-to-end
   conformance statement. Public RDF payload handling, Oxigraph/SPARQL query
   support, and selected ``src/ontology/*`` checks are implemented foundations.
   The complete-envelope journal and Verified Journal Replay, atomic Final
   Admission, authenticated membership, reproducible three-node PoA
   convergence, full package-declared SHACL enforcement, durable privacy
   lifecycle, bounded ProvChain-to-ProvChain bridge, and corrected end-to-end
   evidence are still pending. PBFT, heterogeneous/SPV bridging, a human
   usability study, operational deployment, and production-pilot controls are
   future work. Use
   ``docs/architecture/SHARED_ONTOLOGY_NETWORK_WORKING_PLAN.md`` and
   ``docs/paper_submission/PAPER_EVIDENCE_INDEX.md`` for current status and
   evidence boundaries; repository work is governed by ``AGENTS.md``.

.. raw:: html

   <div class="hero-section">
     <div class="hero-content">
       <h1>Build with Semantic Blockchain Technology</h1>
       <p class="hero-subtitle">ProvChainOrg is a research prototype for cryptographically linked, queryable public-RDF provenance.</p>
       <div class="hero-badges">
         <span class="badge badge-version">Version 0.1.0</span>
         <span class="badge badge-rust">Rust 1.87+</span>
         <span class="badge badge-license">MIT License</span>
       </div>
     </div>
   </div>

.. note::
   This documentation provides comprehensive coverage for all ProvChainOrg users, from business users to developers and researchers. Navigate to the appropriate section based on your role and interests.

Quick Start
-----------

Get up and running with ProvChainOrg in minutes:

.. code-block:: bash

   # Install Rust (if needed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Clone and run
   git clone https://github.com/anusornc/provchain-org.git
   cd provchain-org
   cargo run -- demo

   # Try an existing legacy SPARQL query fixture
   cargo run -- query src/semantic/queries/trace_by_batch_ontology.sparql

.. raw:: html

   <div class="quick-links">
     <a href="user-guide/index.html" class="quick-link">
       <h3>👥 For Business Users</h3>
       <p>User guide for supply chain managers and business users</p>
     </a>
     <a href="api/index.html" class="quick-link">
       <h3>💻 For Developers</h3>
       <p>API reference and technical documentation</p>
     </a>
     <a href="paper_submission/PAPER_EVIDENCE_INDEX.md" class="quick-link">
       <h3>🔬 For Researchers</h3>
       <p>Current claim-to-artifact evidence boundary</p>
     </a>
   </div>

Comprehensive Documentation
---------------------------

This tree contains both current sources and legacy material. Treat the warning above and the linked
working plan/evidence index as authoritative when another page conflicts.

User Documentation
~~~~~~~~~~~~~~~~~~

For end users, business analysts, and system administrators:

.. toctree::
   :maxdepth: 2
   :caption: User Documentation

   user-guide/index
   user-guide/introduction
   user-guide/first-steps

Developer Documentation
~~~~~~~~~~~~~~~~~~~~~~

For developers, technical architects, and integration specialists:

.. toctree::
   :maxdepth: 2
   :caption: Developer Documentation

   developer/index
   api/index
   api/rest-api
   api/sparql-api
   api/websocket-api
   api/authentication
   api/client-libraries

Research Documentation
~~~~~~~~~~~~~~~~~~~~~~

For researchers, academics, and advanced technical users. The quarantined historical research
landing page is intentionally absent from active navigation:

.. toctree::
   :maxdepth: 2
   :caption: Research Documentation

   research/rdf-canonicalization-algorithm
   research/technical-specifications

Foundational Topics
~~~~~~~~~~~~~~~~~~~

Learn the core concepts that make ProvChainOrg unique:

.. toctree::
   :maxdepth: 1

   foundational/intro-to-provchainorg
   foundational/intro-to-rdf-blockchain
   foundational/intro-to-supply-chain-traceability

Legacy Stack Page
~~~~~~~~~~~~~~~~~

The old stack overview is quarantined because it names unverified SDKs, routes, integrations, and
deployment commands. Use ``AGENTS.md`` plus the shared-ontology working plan for current repository
and architecture guidance.

Tutorials & Guides
~~~~~~~~~~~~~~~~~~

Step-by-step guides for common use cases:

.. toctree::
   :maxdepth: 1

   tutorials/first-supply-chain

What Makes ProvChainOrg Different?
----------------------------------

.. raw:: html

   <div class="feature-grid">
     <div class="feature-item">
       <h3>🔗 Public RDF Payloads</h3>
       <p>Represent public provenance payloads as RDF in cryptographically linked prototype blocks</p>
     </div>
     <div class="feature-item">
       <h3>🔍 SPARQL Queries</h3>
       <p>Query the current Oxigraph projection using SPARQL</p>
     </div>
     <div class="feature-item">
       <h3>🧠 Ontology Validation</h3>
       <p>Exercise selected ontology-aware checks; the universal package-declared gate is pending</p>
     </div>
     <div class="feature-item">
       <h3>📊 Traceability References</h3>
       <p>Use supply-chain domains as reference packages without hardcoding the platform boundary</p>
     </div>
   </div>

Use Cases
---------

ProvChainOrg uses the following as research and reference-package scenarios;
this list does not establish regulatory conformance or production deployment:

- **Food Safety**: Track products from farm to table with environmental monitoring
- **Pharmaceutical Traceability**: Explore medicine-batch provenance and anti-counterfeit evidence
- **Luxury Goods Authentication**: Explore provenance inputs to application-level authenticity checks
- **Regulatory Review**: Explore queryable provenance records as inputs to audit workflows
- **Sustainability Tracking**: Monitor environmental impact across supply chains

Community & Support
--------------------

.. raw:: html

   <div class="community-links">
     <a href="https://github.com/anusornc/provchain-org" class="community-link">
       <h4>📦 GitHub Repository</h4>
       <p>Source code, issues, and contributions</p>
     </a>
     <a href="https://github.com/anusornc/provchain-org/discussions" class="community-link">
       <h4>💬 Discussions</h4>
       <p>Community Q&A and feature requests</p>
     </a>
     <a href="https://github.com/anusornc/provchain-org/issues" class="community-link">
       <h4>🐛 Issue Tracker</h4>
       <p>Bug reports and feature requests</p>
     </a>
   </div>

Contributing
------------

ProvChainOrg is open source and welcomes contributions:

- **Documentation**: Help improve these docs
- **Code**: Submit bug fixes and new features
- **Testing**: Help test new releases
- **Examples**: Share your use cases and implementations

See our `Contributing Guide <https://github.com/anusornc/provchain-org/blob/main/CONTRIBUTING.md>`_ for details.

Research Background
-------------------

ProvChainOrg is based on the GraphChain research concept:

.. epigraph::

   "GraphChain – A Distributed Database with Explicit Semantics and Chained RDF Graphs"
   
   -- Sopek, M., et al. (2018), The 2018 Web Conference

This prototype extends the original research with public-RDF provenance,
Oxigraph/SPARQL foundations, selected ontology checks, and supply-chain
reference packages. The end-to-end guarantees listed in the warning above
remain implementation and evidence milestones.

License
-------

ProvChainOrg is released under the `MIT License <https://github.com/anusornc/provchain-org/blob/main/LICENSE>`_.

.. raw:: html

   <div class="footer-note">
     <p><strong>Ready to get started?</strong> Begin with <a href="foundational/intro-to-provchainorg.html">Introduction to ProvChainOrg</a> or jump straight to <a href="tutorials/first-supply-chain.html">building your first application</a>.</p>
   </div>
