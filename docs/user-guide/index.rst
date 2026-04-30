User Guide
=========

Comprehensive documentation for using ProvChainOrg to build semantic blockchain applications for supply chain traceability.

.. raw:: html

   <div class="hero-section">
     <div class="hero-content">
       <h1>User Guide</h1>
       <p class="hero-subtitle">Complete guide to using ProvChainOrg for supply chain traceability</p>
       <div class="hero-badges">
         <span class="badge badge-user">User Guide</span>
         <span class="badge badge-tutorial">Tutorial</span>
         <span class="badge badge-guide">Guide</span>
         <span class="badge badge-beginner">Beginner</span>
       </div>
     </div>
   </div>

.. note::
   This user guide provides comprehensive documentation for using ProvChainOrg to build semantic blockchain applications for supply chain traceability. Whether you're a business user, administrator, or developer getting started with the platform, these resources will help you succeed.

Introduction
------------

Welcome to ProvChainOrg, a semantic blockchain platform that combines the security and immutability of blockchain technology with the expressiveness and queryability of RDF (Resource Description Framework) graphs. This guide will help you understand and use ProvChainOrg effectively for supply chain traceability applications.

**What You'll Learn:**
- How to install and configure ProvChainOrg
- Basic concepts and terminology
- How to track products through the supply chain
- How to query and analyze supply chain data
- How to use the web interface and APIs
- Best practices for data management and security

**Who This Guide Is For:**
- **Business Users**: Managers and decision-makers who need supply chain insights
- **System Administrators**: IT professionals responsible for deployment and maintenance
- **Developers**: Technical users building applications on the platform
- **Researchers**: Academics and scientists studying supply chain systems

Getting Started
---------------

Begin your journey with ProvChainOrg:

**First Steps**
.. toctree::
   :maxdepth: 1
   :caption: Getting Started

   introduction
   first-steps
   basic-concepts
   system-requirements

**Quick Start**
To get started quickly with ProvChainOrg:

.. code-block:: bash
   # Install Rust (if needed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Clone and run
   git clone https://github.com/anusornc/provchain-org.git
   cd provchain-org
   cargo run demo

   # Try a SPARQL query
   cargo run -- query queries/trace_by_batch_ontology.sparql

Core Concepts
-------------

Understanding the fundamental concepts of ProvChainOrg:

**Key Concepts**
.. toctree::
   :maxdepth: 1
   :caption: Core Concepts

   rdf-basics
   blockchain-concepts
   semantic-web
   supply-chain-modeling

**Essential Knowledge**
1. **RDF and Semantic Data**: Understanding Resource Description Framework
2. **Blockchain Fundamentals**: How blockchain technology works
3. **SPARQL Queries**: Querying semantic data
4. **Ontology Integration**: Using formal schemas for data validation
5. **Supply Chain Modeling**: Representing supply chain processes

Installation and Setup
----------------------

Complete installation and configuration guides:

**Installation Guides**
.. toctree::
   :maxdepth: 1
   :caption: Installation and Setup

   installation-guide
   configuration
   system-requirements
   troubleshooting

**Installation Options**
1. **Local Development**: Setting up for development and testing
2. **Production Deployment**: Enterprise-ready deployment
3. **Docker Deployment**: Containerized installation
4. **Cloud Deployment**: Deploying to cloud platforms

**System Requirements**
- **Operating System**: Linux, macOS, or Windows with WSL
- **Memory**: Minimum 4GB RAM (8GB recommended)
- **Storage**: Minimum 100GB SSD (500GB recommended)
- **Network**: 100Mbps connection (1Gbps recommended)

Web Interface
-------------

Using the ProvChainOrg web interface:

**Web Interface Guides**
.. toctree::
   :maxdepth: 1
   :caption: Web Interface

   web-dashboard
   query-interface
   data-visualization
   reporting-tools

**Interface Features**
1. **Dashboard**: System status and overview
2. **Query Builder**: Visual SPARQL query interface
3. **Data Explorer**: Browse blockchain blocks and data
4. **Visualization Tools**: Graphical representation of supply chains
5. **Reporting**: Generate and export reports

Command Line Interface
----------------------

Using the ProvChainOrg CLI for advanced operations:

**CLI Documentation**
.. toctree::
   :maxdepth: 1
   :caption: Command Line Interface

   cli-overview
   data-management
   query-operations
   system-administration

**Common CLI Commands**
.. code-block:: bash
   # Run the demo
   cargo run demo

   # Add RDF data
   cargo run -- add-file supply_chain_data.ttl

   # Execute SPARQL query
   cargo run -- query trace_query.sparql

   # Validate blockchain
   cargo run -- validate

   # Generate API key
   cargo run -- generate-api-key

Data Management
---------------

Managing supply chain data in ProvChainOrg:

**Data Management Guides**
.. toctree::
   :maxdepth: 1
   :caption: Data Management

   data-import
   data-export
   data-validation
   data-cleanup

**Data Operations**
1. **Importing Data**: Adding supply chain information
2. **Exporting Data**: Retrieving data in various formats
3. **Data Validation**: Ensuring data quality and compliance
4. **Data Archiving**: Managing historical data

**Supported Formats**
- **Turtle (.ttl)**: Primary format for semantic data
- **JSON-LD (.jsonld)**: JSON-based linked data format
- **N-Triples (.nt)**: Simple triple format
- **RDF/XML (.rdf)**: XML-based RDF format

Querying Data
-------------

Using SPARQL to query supply chain information:

**Query Documentation**
.. toctree::
   :maxdepth: 1
   :caption: Querying Data

   sparql-basics
   advanced-queries
   query-optimization
   query-examples

**Query Examples**
.. code-block:: sparql
   # Find all product batches from a specific farm
   PREFIX : <http://example.org/supply-chain#>
   SELECT ?batch ?product ?harvestDate WHERE {
       ?batch a :ProductBatch ;
              :originFarm :GreenValleyFarm ;
              :product ?product ;
              :harvestDate ?harvestDate .
   }

   # Track environmental conditions during transport
   PREFIX : <http://example.org/supply-chain#>
   SELECT ?batch ?temperature ?humidity ?timestamp WHERE {
       ?batch :transportedThrough ?transport .
       ?transport :environmentalCondition ?condition .
       ?condition :temperature ?temperature ;
                  :humidity ?humidity ;
                  :recordedAt ?timestamp .
   }

Supply Chain Applications
-------------------------

Building specific supply chain applications:

**Application Guides**
.. toctree::
   :maxdepth: 1
   :caption: Supply Chain Applications

   food-safety
   pharmaceutical-tracking
   quality-assurance
   compliance-reporting

**Industry Use Cases**
1. **Food Safety**: Farm-to-table tracking with environmental monitoring
2. **Pharmaceuticals**: Drug authentication and counterfeit prevention
3. **Luxury Goods**: Provenance verification and authenticity assurance
4. **Manufacturing**: Component tracking and quality control

**Example Application**
.. code-block:: turtle
   # Example: Tracking organic tomatoes through the supply chain
   @prefix : <http://example.org/supply-chain#> .
   @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

   # Farm origin
   :Batch001 a :ProductBatch ;
       :hasBatchID "TOMATO-2025-001" ;
       :product :OrganicTomatoes ;
       :originFarm :GreenValleyFarm ;
       :harvestDate "2025-01-15"^^xsd:date ;
       :certifiedOrganic true .

   # Processing
   :Processing001 a :ProcessingActivity ;
       :processedBatch :Batch001 ;
       :processType :Washing ;
       :timestamp "2025-01-16T09:00:00Z"^^xsd:dateTime ;
       :performedBy :ProcessingPlantA .

   # Transport with environmental monitoring
   :Transport001 :environmentalCondition [
       a :EnvironmentalCondition ;
       :temperature "4.2"^^xsd:decimal ;
       :humidity "78"^^xsd:decimal ;
       :location :ColdStorage ;
       :recordedAt "2025-01-16T14:30:00Z"^^xsd:dateTime
   ] .

API Integration
---------------

Integrating ProvChainOrg with external systems:

**API Documentation**
.. toctree::
   :maxdepth: 1
   :caption: API Integration

   api-basics
   rest-api
   websocket-api
   client-libraries

**Integration Examples**
.. code-block:: python
   import requests

   class ProvChainClient:
       def __init__(self, base_url, api_key):
           self.base_url = base_url
           self.headers = {
               'Authorization': f'Bearer {api_key}',
               'Content-Type': 'application/json'
           }
       
       def add_supply_chain_data(self, turtle_data):
           response = requests.post(
               f'{self.base_url}/api/data',
               headers=self.headers,
               data=turtle_data
           )
           return response.json()

User Management
---------------

Managing users and permissions:

**User Management Guides**
.. toctree::
   :maxdepth: 1
   :caption: User Management

   user-accounts
   role-management
   authentication
   access-control

**User Roles**
1. **Viewer**: Read-only access to public data
2. **User**: Standard user with read/write access to their data
3. **Manager**: Business user with extended permissions
4. **Administrator**: System administrator with full access
5. **Auditor**: Compliance auditor with read-only access

**Role-Based Access Control**
.. code-block:: json
   {
     "user_id": "user_123",
     "roles": ["user", "manager"],
     "permissions": {
       "organization:acme": {
         "read": true,
         "write": true,
         "delete": false
       }
     }
   }

Monitoring and Maintenance
--------------------------

Monitoring system health and performing maintenance:

**Operations Guides**
.. toctree::
   :maxdepth: 1
   :caption: Monitoring and Maintenance

   system-monitoring
   performance-tuning
   backup-and-recovery
   system-upgrades

**Monitoring Tools**
1. **Health Checks**: System status and component health
2. **Performance Metrics**: Resource usage and throughput
3. **Log Analysis**: System logs and error tracking
4. **Alerting**: Automated notifications for issues

**Maintenance Tasks**
.. code-block:: bash
   # Check system health
   curl http://localhost:8080/health

   # View system logs
   journalctl -u provchain-org -f

   # Perform backup
   cargo run -- backup --type full --output backup-2025-01-15.tar.gz

   # Update system
   git pull
   cargo build --release

Troubleshooting
---------------

Solving common issues and problems:

**Troubleshooting Guides**
.. toctree::
   :maxdepth: 1
   :caption: Troubleshooting

   common-issues
   error-codes
   performance-problems
   network-issues

**Common Solutions**
1. **Installation Problems**: Dependency issues and build errors
2. **Runtime Errors**: Configuration problems and data issues
3. **Performance Issues**: Slow queries and high resource usage
4. **Network Problems**: Connectivity and synchronization issues
5. **Security Issues**: Authentication and authorization problems

**Diagnostic Commands**
.. code-block:: bash
   # Check system status
   cargo run -- status

   # Validate configuration
   cargo run -- validate-config

   # Test network connectivity
   cargo run -- test-network

   # Check data integrity
   cargo run -- validate-data

Best Practices
--------------

Guidelines for effective use of ProvChainOrg:

**Best Practice Guides**
.. toctree::
   :maxdepth: 1
   :caption: Best Practices

   data-modeling
   query-optimization
   security-best-practices
   performance-tuning

**Key Recommendations**
1. **Data Modeling**: Design clear and consistent data structures
2. **Query Optimization**: Write efficient SPARQL queries
3. **Security**: Implement proper authentication and access control
4. **Performance**: Monitor and optimize system performance
5. **Backup**: Regularly backup critical data and configurations

**Data Quality Guidelines**
- **Consistent Naming**: Use standardized naming conventions
- **Complete Information**: Include all required properties
- **Valid Formats**: Ensure data conforms to expected formats
- **Regular Validation**: Validate data regularly for quality

Advanced Topics
---------------

Advanced features and capabilities:

**Advanced Guides**
.. toctree::
   :maxdepth: 1
   :caption: Advanced Topics

   ontology-extension
   custom-queries
   automation-scripts
   integration-patterns

**Advanced Features**
1. **Custom Ontologies**: Extending the traceability ontology
2. **Complex Queries**: Advanced SPARQL patterns
3. **Automation**: Scripting and workflow automation
4. **Integration**: Connecting with external systems

**Example Advanced Query**
.. code-block:: sparql
   # Complex supply chain analysis with window functions
   PREFIX : <http://example.org/supply-chain#>
   PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

   SELECT ?batch ?product ?harvestDate 
          (RANK() OVER (ORDER BY ?harvestDate DESC) AS ?rank) 
          (AVG(?temperature) AS ?avgTemp) WHERE {
       ?batch a :ProductBatch ;
              :product ?product ;
              :harvestDate ?harvestDate .
       OPTIONAL {
           ?batch :transportedThrough ?transport .
           ?transport :environmentalCondition ?condition .
           ?condition :temperature ?temperature .
       }
   }
   GROUP BY ?batch ?product ?harvestDate
   HAVING (?avgTemp < 8.0)
   ORDER BY ?rank

Compliance and Reporting
------------------------

Meeting regulatory requirements and generating reports:

**Compliance Guides**
.. toctree::
   :maxdepth: 1
   :caption: Compliance and Reporting

   regulatory-compliance
   audit-trails
   reporting-tools
   certification

**Compliance Features**
1. **Immutable Records**: Cryptographically secured audit trails
2. **Data Validation**: Automated compliance checking
3. **Reporting Tools**: Generate compliance reports
4. **Certification**: Digital certificates with proof

**Example Compliance Report**
.. code-block:: json
   {
     "report_id": "compliance_2025_001",
     "generated_at": "2025-01-15T10:30:00Z",
     "period": {
       "start": "2025-01-01T00:00:00Z",
       "end": "2025-01-15T10:30:00Z"
     },
     "compliance_status": "compliant",
     "findings": [],
     "certifications": [
       {
         "type": "organic",
         "status": "valid",
         "expires": "2025-12-31T23:59:59Z"
       }
     ]
   }

Community and Support
---------------------

Getting help and connecting with the community:

**Support Resources**
.. toctree::
   :maxdepth: 1
   :caption: Community and Support

   getting-help
   community-forum
   documentation
   training-resources

**Support Channels**
- **Documentation**: Comprehensive guides and references
- **Community Forum**: Peer support and discussions
- **Issue Tracker**: Bug reports and feature requests
- **Professional Support**: Enterprise support options

**Community Resources**
1. **GitHub Repository**: Source code and issue tracking
2. **Discussion Forum**: Technical discussions and Q&A
3. **Training Materials**: Tutorials and learning resources
4. **User Groups**: Local and regional user communities

Glossary
--------

Definitions of key terms and concepts:

**A**
- **API**: Application Programming Interface
- **Audit Trail**: Immutable record of all system activities

**B**
- **Blockchain**: Distributed ledger technology
- **Block**: Unit of data in the blockchain

**C**
- **Canonicalization**: Process of creating deterministic data representations
- **CLI**: Command Line Interface

**D**
- **Data Validation**: Process of ensuring data quality and compliance
- **Docker**: Containerization platform

**E**
- **Environmental Monitoring**: Tracking of environmental conditions

**F**
- **Federation**: Network of interconnected blockchain nodes

**G**
- **Graph**: Data structure representing relationships
- **GUI**: Graphical User Interface

**I**
- **Integration**: Connecting with external systems

**J**
- **JSON-LD**: JSON-based Linked Data format

**K**
- **Key Pair**: Public and private cryptographic keys

**L**
- **Ledger**: Record of all transactions

**M**
- **Metadata**: Data about data

**N**
- **Node**: Participant in the blockchain network

**O**
- **Ontology**: Formal specification of concepts and relationships
- **OWL**: Web Ontology Language

**P**
- **Peer**: Network participant
- **Permission**: Access control right
- **Provenance**: Origin and history of data

**Q**
- **Query**: Request for data retrieval
- **Quality Assurance**: Process of ensuring data quality

**R**
- **RDF**: Resource Description Framework
- **REST API**: Representational State Transfer API
- **Role**: User permission category

**S**
- **SPARQL**: Query language for RDF
- **Supply Chain**: Network of organizations involved in production
- **Semantic Web**: Web of data with meaning

**T**
- **Turtle**: RDF serialization format

**U**
- **URI**: Uniform Resource Identifier
- **User Management**: Administration of user accounts and permissions

**V**
- **Validation**: Process of checking data correctness
- **Verification**: Process of confirming data integrity

**W**
- **Wallet**: Cryptographic key storage
- **WebSocket**: Protocol for real-time communication

**Z**
- **Zero Knowledge**: Privacy-preserving cryptographic technique

Further Reading
---------------

Additional resources for learning more about ProvChainOrg:

**External Resources**
- **W3C Standards**: RDF, SPARQL, and related semantic web technologies
- **Blockchain Research**: Academic papers and conference proceedings
- **Supply Chain Management**: Industry best practices and case studies
- **Regulatory Compliance**: Guidelines and requirements for various industries

**Learning Path**
1. **Beginner**: Start with the introduction and first steps
2. **Intermediate**: Explore data management and querying
3. **Advanced**: Dive into API integration and advanced topics
4. **Expert**: Contribute to the community and advance the technology

.. note::
   The ProvChainOrg user guide is continuously evolving. Check back regularly for updates, new guides, and improved examples. If you have suggestions for additional documentation, please contribute through our GitHub repository.

.. raw:: html

   <div class="footer-note">
     <p><strong>Ready to get started?</strong> Begin with the <a href="introduction.html">Introduction</a> or jump straight to <a href="first-steps.html">First Steps</a> to build your first supply chain application.</p>
   </div>
