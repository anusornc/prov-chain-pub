# ProvChain-Org User Manual

**Complete guide to using ProvChain-Org for supply chain traceability**

---

## Welcome to ProvChain-Org

ProvChain-Org is a blockchain-based supply chain traceability platform that helps you:
- **Track products** from origin to consumer
- **Monitor conditions** during transport and storage
- **Verify authenticity** and prevent fraud
- **Maintain compliance** with regulations
- **Analyze data** for insights and optimization

**This manual is for everyone** who uses ProvChain-Org - from business users to system administrators.

---

## How to Use This Manual

### By What You Want to Do

| Want to... | Go to... |
|------------|----------|
| Get ProvChain running in 10 minutes | [Quick Start Guide](00-quick-start/10-minute-setup.md) |
| Submit your first product batch | [Your First Transaction](00-quick-start/first-transaction.md) |
| Query product traceability data | [Query Library](03-querying-data/query-library.md) |
| Configure network peers | [Network Setup](05-configuration/network-setup.md) |
| Troubleshoot an issue | [Troubleshooting](08-troubleshooting/troubleshooting.md) |

---

## Manual Structure

.. note::
   **Documentation Status**: This manual is under active development. Many sections are still being written. The sections below that are **bolded** are currently available.

### **0. Quick Start** 🚀
Get started fast with pre-built Docker images and step-by-step tutorials.

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
Customize ProvChain for your needs.

- **[Network Setup](05-configuration/network-setup.md)** - Configure peers and networking

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

Each transaction is:
- **Immutable** - Cannot be changed once recorded
- **Timestamped** - Exact time is recorded
- **Traceable** - Linked to previous transactions
- **Verified** - Cryptographically verified

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
- **Access to ProvChain** - Either running locally or access to a deployed instance
- **API access** (optional) - For programmatic access

For system administrators:
- **Docker knowledge** - For container-based deployment
- **Basic networking** - For multi-node configuration
- **Linux administration** - For production deployments

---

## Quick Reference

### Common Commands

```bash
# Check if ProvChain is running
curl http://localhost:8080/health

# Get JWT authentication token (demo mode)
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"demo","password":"demo"}'

# View blockchain status
curl http://localhost:8080/api/blockchain/status

# Submit RDF data
curl -X POST http://localhost:8080/api/transactions \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"triples": "@prefix : <#> . :s :p :o ."}'

# Query blockchain data (SPARQL)
curl -X POST http://localhost:8080/api/sparql/query \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query": "SELECT * WHERE { ?s ?p ?o } LIMIT 10"}'
```

### Common API Endpoints

| Endpoint | Purpose |
|----------|---------|
| `POST /api/auth/login` | Get authentication token |
| `POST /api/transactions` | Add RDF data to blockchain |
| `POST /api/sparql/query` | Query blockchain data with SPARQL |
| `GET /api/blockchain/status` | View blockchain information |
| `GET /health` | Check system health |

---

## Support & Resources

### Getting Help

- **Documentation**: You're here! Browse the sections above
- **Main README**: [../../README.md](../../README.md) - Project overview
- **Contributing Guide**: [../../CONTRIBUTING.md](../../CONTRIBUTING.md) - Development setup
- **Deployment Guide**: [../deployment/HANDS_ON_DEPLOYMENT_GUIDE.md](../deployment/HANDS_ON_DEPLOYMENT_GUIDE.md) - Deployment instructions
- **Issues**: [GitHub Issues](https://github.com/anusornc/provchain-org2/issues)
- **FAQ**: [../FAQ.md](../FAQ.md) - Frequently asked questions

### Related Documentation

- **[../README.md](../README.md)** - Documentation overview and index
- **[../developer/index.rst](../developer/index.rst)** - Developer documentation
- **[../architecture/README.md](../architecture/README.md)** - Architecture documentation
- **[../deployment/HANDS_ON_DEPLOYMENT_GUIDE.md](../deployment/HANDS_ON_DEPLOYMENT_GUIDE.md)** - Hands-on deployment guide

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
