# ProvChain-Org User Manual

**Development/reference guide to the current ProvChain-Org research prototype**

> **Current capability boundary (2026-08-31):** this manual supports local development and
> bounded reference experiments. It does not establish authenticated membership, replicated
> three-node PoA convergence, durable privacy, operational deployment, certified authenticity,
> fraud prevention, or regulatory compliance. PBFT and production-pilot controls are future work.
> Use the [current architecture plan](../architecture/SHARED_ONTOLOGY_NETWORK_WORKING_PLAN.md)
> before relying on a workflow or network claim.

---

## Welcome to ProvChain-Org

ProvChain-Org is a blockchain-based supply chain traceability platform that helps you:
- **Explore product provenance** from origin to consumer in reference scenarios
- **Record and query conditions** during transport and storage
- **Inspect public-RDF provenance** as one input to application-level authenticity workflows
- **Explore audit-oriented queries** without claiming regulatory compliance
- **Analyze reference data** for research and development

**This manual is for everyone** who uses ProvChain-Org - from business users to system administrators.

---

## How to Use This Manual

### By What You Want to Do

| Want to... | Go to... |
|------------|----------|
| Get ProvChain running in 10 minutes | [Quick Start Guide](00-quick-start/10-minute-setup.md) |
| Submit your first product batch | [Your First Transaction](00-quick-start/first-transaction.md) |
| Query product traceability data | [Query Library](03-querying-data/query-library.md) |
| Inspect experimental network configuration | [Network Configuration Reference](05-configuration/network-setup.md) |
| Troubleshoot an issue | [Troubleshooting](08-troubleshooting/troubleshooting.md) |

---

## Manual Structure

.. note::
   **Documentation Status**: This manual is under active development. Many sections are still being written. The sections below that are **bolded** are currently available.

### **0. Quick Start** 🚀
Get started with local development/reference commands and step-by-step tutorials.

- **[What is ProvChain-Org?](00-quick-start/overview.md)** - Understanding the system
- **[10-Minute Setup](00-quick-start/10-minute-setup.md)** - Start using ProvChain now
- **[Your First Transaction](00-quick-start/first-transaction.md)** - Submit data to the blockchain

### 1. Getting Started 📚
*Foundation knowledge for all users. (Coming Soon)*

### 2. Submitting Data 📝
*Everything about adding data to the blockchain. (Coming Soon)*

### **3. Querying Data** 🔍
Retrieve and analyze blockchain data.

- **[Query Library](03-querying-data/query-library.md)** - Ready-to-use SPARQL queries for traceability analysis

### 4. Common Workflows 📋
*Step-by-step guides for business processes. (Coming Soon)*

### **5. Configuration** ⚙️
Inspect local configuration and experimental networking scaffolding.

- **[Network Configuration Reference](05-configuration/network-setup.md)** - Legacy/target peer
  examples; not authenticated replication, convergence, or deployment evidence

### 6. System Administration 🔧
*Deploy, maintain, and monitor ProvChain. (Coming Soon)*

### 7. API Reference 📡
*Technical API documentation for advanced users. (Coming Soon)*

### **8. Troubleshooting** 🆘
Solve common problems quickly.

- **[Troubleshooting Guide](08-troubleshooting/troubleshooting.md)** - Common issues and solutions

### 9. Appendices 📖
*Reference material and additional resources. (Coming Soon)*

---

## Key Concepts

### What is a Transaction?

A **transaction** is a record of an event in your supply chain, such as:
- Harvesting crops from a farm
- Processing raw materials
- Transporting goods
- Quality inspection results
- Temperature readings during storage

In the target reference-system model, each admitted provenance event is intended to be:
- **Append-only** - Recorded through the complete-envelope journal rather than updated in place
- **Timestamped** - Carries the time represented by its admitted envelope
- **Traceable** - Linked through explicit provenance relationships
- **Authenticated and validated** - Accepted only after the pending universal Final Admission checks

Those properties are acceptance targets, not evidence that the current local reference instance
already provides crash-safe immutability, authenticated membership, or end-to-end convergence.

### What is RDF?

ProvChain uses **RDF (Resource Description Framework)** to store supply chain data. RDF is a standard for representing data as triples:

```
Subject → Predicate → Object
```

Example:
```
:Batch001 → :hasOrigin → :ThailandFarm
:Batch001 → :harvestDate → "2025-01-15"
:Batch001 → :productType → "OrganicTomatoes"
```

Don't worry if you're not familiar with RDF - we provide examples and templates.

---

## Prerequisites

Before using ProvChain, you should have:

- **Basic computer literacy** - Comfortable with web browsers and forms
- **Access to ProvChain** - A local development/reference instance
- **API access** (optional) - For programmatic access

For system administrators:
- **Docker knowledge** - For container-based deployment
- **Basic networking** - For multi-node configuration
- **Linux administration** - For future controlled deployment evaluation

Operational deployment is future work; the current guides support local development and bounded
reference experiments only.

---

## Quick Reference

### Common Commands

```bash
# Check if ProvChain is running
curl http://localhost:8080/health

# Bootstrap the first admin user once.
# Set PROVCHAIN_BOOTSTRAP_TOKEN before starting the server.
curl -X POST http://localhost:8080/auth/bootstrap \
  -H "Content-Type: application/json" \
  -d '{"username":"adminroot","password":"AdminRootPassword123!","bootstrap_token":"YOUR_BOOTSTRAP_TOKEN"}'

# Get JWT authentication token
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"adminroot","password":"AdminRootPassword123!"}'

# View blockchain status
curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:8080/api/blockchain/status

# Submit one RDF triple
curl -X POST http://localhost:8080/api/blockchain/add-triple \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"subject":"http://example.org/s","predicate":"http://example.org/p","object":"http://example.org/o","graph_name":null,"privacy_key_id":null}'

# Import a Turtle dataset as one block
curl -X POST http://localhost:8080/api/datasets/import-turtle \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"turtle_data":"@prefix ex: <http://example.org/> . ex:s ex:p ex:o ."}'

# Query blockchain data (SPARQL)
curl -X POST http://localhost:8080/api/sparql/query \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query": "SELECT * WHERE { ?s ?p ?o } LIMIT 10"}'
```

### Common API Endpoints

| Endpoint | Purpose |
|----------|---------|
| `POST /auth/bootstrap` | Create the first admin user |
| `POST /auth/login` | Get authentication token |
| `POST /api/blockchain/add-triple` | Add one RDF triple to blockchain |
| `POST /api/blockchain/add-triples` | Add multiple RDF triples as one block |
| `POST /api/datasets/import-turtle` | Import a Turtle dataset as one block |
| `POST /api/transactions/create` | Create a transaction object |
| `POST /api/transactions/sign` | Sign a transaction object |
| `POST /api/transactions/submit` | Submit a signed transaction object |
| `POST /api/sparql/query` | Query blockchain data with SPARQL |
| `GET /api/blockchain/status` | View blockchain information |
| `GET /health` | Check system health |

---

## Support & Resources

### Getting Help

- **Documentation**: You're here! Browse the sections above
- **Main README**: [../../README.md](../../README.md) - Project overview
- **Contributing Guide**: [../../CONTRIBUTING.md](../../CONTRIBUTING.md) - Development setup
- **Local Execution Guide**: [../Run.md](../Run.md) - Development/reference execution
- **Current Architecture Plan**: [../architecture/SHARED_ONTOLOGY_NETWORK_WORKING_PLAN.md](../architecture/SHARED_ONTOLOGY_NETWORK_WORKING_PLAN.md) - Locked milestones and evidence boundary
- **Issues**: [GitHub Issues](https://github.com/anusornc/prov-chain/issues)
- **FAQ**: [../FAQ.md](../FAQ.md) - Frequently asked questions

### Related Documentation

- **[../README.md](../README.md)** - Documentation overview and index
- **[../developer/index.rst](../developer/index.rst)** - Developer documentation
- **[../architecture/README.md](../architecture/README.md)** - Architecture documentation
- **[../architecture/CONTAINER_ARCHITECTURE.md](../architecture/CONTAINER_ARCHITECTURE.md)** - Target/reference topology; not operational deployment evidence

### Contributing

Found an error or want to improve the documentation? Please:
1. Check for existing issues
2. Create a new issue with your suggestion
3. Submit a pull request with improvements

---

## Next Steps

**New to ProvChain?** Start with the [Quick Start Guide](00-quick-start/10-minute-setup.md)

**Know the basics?** Jump to:
- [Query Library](03-querying-data/query-library.md) - Analyze your blockchain data
- [Network Setup](05-configuration/network-setup.md) - Configure your system

**Need help now?** Check [Troubleshooting](08-troubleshooting/troubleshooting.md)

---

*Last updated: 2026-01-21*
*Version: 1.0.0*
