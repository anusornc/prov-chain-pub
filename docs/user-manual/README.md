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

### By Your Role

| If you are a... | Start here... | What you'll learn |
|-----------------|---------------|-------------------|
| **Supply Chain Manager** | [Quick Start](00-quick-start/overview.md) | Submit product data, track shipments |
| **Data Analyst** | [Querying Data](03-querying-data/query-library.md) | Run SPARQL queries, analyze trends |
| **System Administrator** | [System Administration](06-system-administration/) | Deploy, configure, monitor system |
| **Quality Control** | [Submitting Data](02-submitting-data/) | Record quality checks, compliance |
| **New User** | [10-Minute Setup](00-quick-start/10-minute-setup.md) | Get started quickly |

### By What You Want to Do

| Want to... | Go to... |
|------------|----------|
| Get ProvChain running in 10 minutes | [Quick Start Guide](00-quick-start/10-minute-setup.md) |
| Submit your first product batch | [Submit Product Batch Data](02-submitting-data/product-batches.md) |
| Query product traceability data | [Query Library](03-querying-data/query-library.md) |
| Configure network peers | [Network Setup](05-configuration/network-setup.md) |
| Troubleshoot an issue | [Troubleshooting](08-troubleshooting/) |

---

## Manual Structure

### **0. Quick Start** 🚀
Get started fast with pre-built Docker images and step-by-step tutorials.

- [What is ProvChain-Org?](00-quick-start/overview.md) - Understanding the system
- [10-Minute Setup](00-quick-start/10-minute-setup.md) - Start using ProvChain now
- [Your First Transaction](00-quick-start/first-transaction.md) - Submit data to the blockchain
- [Your First Query](00-quick-start/first-query.md) - Query blockchain data

### **1. Getting Started** 📚
Foundation knowledge for all users.

- [System Requirements](01-getting-started/requirements.md)
- [Installation Options](01-getting-started/installation-options.md)
- [Initial Configuration](01-getting-started/initial-configuration.md)
- [Verifying Installation](01-getting-started/verification.md)

### **2. Submitting Data** 📝
Everything about adding data to the blockchain.

- [Understanding Transactions](02-submitting-data/understanding-transactions.md)
- [Product Batch Data](02-submitting-data/product-batches.md) - Record harvests/production
- [Activities & Events](02-submitting-data/activities-events.md) - Record processing/transport
- [Environmental Data](02-submitting-data/environmental-data.md) - Temperature, humidity, etc.
- [Quality Control Data](02-submitting-data/quality-control.md) - QC checks and inspections
- [Bulk Submission](02-submitting-data/bulk-submission.md) - Upload multiple records
- [Data Validation](02-submitting-data/data-validation.md) - Understanding validation rules
- [Error Handling](02-submitting-data/error-handling.md) - Fix submission errors

### **3. Querying Data** 🔍
Retrieve and analyze blockchain data.

- [SPARQL Basics](03-querying-data/sparql-basics.md) - Query language fundamentals
- [Query Library](03-querying-data/query-library.md) - 30+ ready-to-use queries
- [Web Query Interface](03-querying-data/web-interface.md) - Use the web UI
- [API Query Examples](03-querying-data/api-query-examples.md) - Query via REST API
- [Advanced Queries](03-querying-data/advanced-queries.md) - Complex analysis
- [Query Optimization](03-querying-data/query-optimization.md) - Performance tips

### **4. Common Workflows** 📋
Step-by-step guides for business processes.

- [Onboarding Suppliers](04-common-workflows/onboarding-suppliers.md) - Add new partners
- [Product Recalls](04-common-workflows/product-recalls.md) - Track affected products
- [Compliance Reports](04-common-workflows/compliance-reports.md) - Generate regulatory reports
- [Environmental Monitoring](04-common-workflows/environmental-monitoring.md) - Track conditions
- [Audit Trail](04-common-workflows/audit-trail.md) - View system history
- [Data Export](04-common-workflows/data-export.md) - Export data for analysis

### **5. Configuration** ⚙️
Customize ProvChain for your needs.

- [Config File Reference](05-configuration/config-file-reference.md) - All configuration options
- [Environment Variables](05-configuration/environment-variables.md) - ENV settings
- [Network Setup](05-configuration/network-setup.md) - Configure peers and networking
- [Storage Configuration](05-configuration/storage-config.md) - Data storage options
- [Security Configuration](05-configuration/security-config.md) - Authentication and encryption
- [Performance Tuning](05-configuration/performance-tuning.md) - Optimize system performance

### **6. System Administration** 🔧
Deploy, maintain, and monitor ProvChain.

- [User Management](06-system-administration/user-management.md) - Manage users and roles
- [Backup & Restore](06-system-administration/backup-restore.md) - Data protection
- [Monitoring](06-system-administration/monitoring.md) - System health and metrics
- [Logging](06-system-administration/logging.md) - Log management
- [Maintenance Tasks](06-system-administration/maintenance-tasks.md) - Regular maintenance
- [Upgrades](06-system-administration/upgrades.md) - Update procedures

### **7. API Reference** 📡
Technical API documentation for advanced users.

- [Authentication](07-api-reference/authentication.md) - JWT tokens and API keys
- [REST API Summary](07-api-reference/rest-api.md) - All API endpoints
- [Transaction API](07-api-reference/transaction-api.md) - Submit transactions
- [Query API](07-api-reference/query-api.md) - Query data
- [WebSocket API](07-api-reference/websocket-api.md) - Real-time updates
- [Error Codes](07-api-reference/error-codes.md) - API error reference

### **8. Troubleshooting** 🆘
Solve common problems quickly.

- [Installation Issues](08-troubleshooting/installation-issues.md)
- [Connection Issues](08-troubleshooting/connection-issues.md)
- [Authentication Errors](08-troubleshooting/authentication-errors.md)
- [Data Submission Errors](08-troubleshooting/submission-errors.md)
- [Query Errors](08-troubleshooting/query-errors.md)
- [Performance Issues](08-troubleshooting/performance-issues.md)
- [Getting Help](08-troubleshooting/getting-help.md)

### **9. Appendices** 📖
Reference material and additional resources.

- [Glossary](09-appendices/glossary.md) - Terminology
- [RDF Basics](09-appendices/rdf-basics.md) - Understanding RDF data
- [Ontology Reference](09-appendices/ontology-reference.md) - Supply chain ontology
- [Sample Data](09-appendices/sample-data.md) - Example datasets
- [Migration Guide](09-appendices/migration-guide.md) - Migrate from other systems

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

# View blockchain status
curl http://localhost:8080/api/blockchain/status

# View recent transactions
curl http://localhost:8080/api/transactions/recent
```

### Common API Endpoints

| Endpoint | Purpose |
|----------|---------|
| `POST /auth/login` | Get authentication token |
| `POST /api/blockchain/add-triple` | Add data to blockchain |
| `POST /api/sparql/query` | Query blockchain data |
| `GET /api/products/:id/trace` | Trace product journey |
| `GET /health` | Check system health |

---

## Support & Resources

### Getting Help

- **Documentation**: You're here! Browse the sections above
- **Issues**: [GitHub Issues](https://github.com/anusornc/provchain-org2/issues)
- **Community**: (Link to community forum/Discord when available)

### Contributing

Found an error or want to improve the documentation? Please:
1. Check for existing issues
2. Create a new issue with your suggestion
3. Submit a pull request with improvements

---

## Next Steps

**New to ProvChain?** Start with the [Quick Start Guide](00-quick-start/10-minute-setup.md)

**Know the basics?** Jump to:
- [Submitting Data](02-submitting-data/) - Add your supply chain data
- [Querying Data](03-querying-data/) - Analyze your blockchain
- [Configuration](05-configuration/) - Customize your system

**Need help now?** Check [Troubleshooting](08-troubleshooting/)

---

*Last updated: 2025-01-04*
*Version: 1.0.0*
