# 🔗 ProvChain-Org: Blockchain-based Supply Chain Traceability System

## 📋 ภาพรวมโครงการ

**ProvChain-Org** เป็นระบบ blockchain-based traceability สำหรับ supply chain ที่พัฒนาด้วย Rust โดยใช้ RDF (Resource Description Framework) และ ontology-based reasoning เพื่อสร้างระบบติดตามสินค้าที่มีความแม่นยำและสามารถตรวจสอบได้

### 🎯 วัตถุประสงค์หลัก
- สร้างระบบติดตามสินค้าที่โปร่งใสและตรวจสอบได้
- ใช้ blockchain เพื่อความปลอดภัยและไม่สามารถแก้ไขได้
- ประยุกต์ใช้ semantic web technologies (RDF, OWL, SPARQL)
- รองรับ supply chain ที่ซับซ้อนด้วย ontology-based reasoning

---

## 🏗️ สถาปัตยกรรมระบบ

### Core Components:

#### 1. **Blockchain Layer** (`src/core/`)
- **Blockchain**: Core blockchain implementation ด้วย RDF canonicalization
- **Block**: Block structure พร้อม RDF data และ state root
- **Atomic Operations**: Transaction atomicity และ consistency
- **Entity Management**: Traceable entities ด้วย semantic properties

#### 2. **Storage Layer** (`src/storage/`)
- **RDF Store**: Oxigraph-based RDF storage ด้วย SPARQL support
- **Persistence**: File-based และ in-memory storage options
- **Backup & Restore**: Automated backup system
- **Canonicalization**: RDF graph canonicalization algorithms (Custom + RDFC-1.0)

#### 3. **Transaction System** (`src/transaction/`)
- **Transaction Types**: Production, Processing, Transport, Quality, Transfer, etc.
- **Digital Signatures**: Ed25519-based signing และ verification
- **Multi-signature Support**: สำหรับ critical operations
- **Transaction Pool**: Pending transaction management

#### 4. **Semantic Layer** (`src/semantic/`)
- **OWL2 Reasoning**: Enhanced reasoning ด้วย owl2_rs library
- **SHACL Validation**: Shape-based data validation
- **Ontology Integration**: Dynamic ontology loading และ management
- **Enhanced Traceability**: OWL2-powered traceability optimization

#### 5. **Network Layer** (`src/network/`)
- **P2P Networking**: Peer-to-peer communication
- **Consensus**: Distributed consensus mechanisms
- **Discovery**: Automatic peer discovery
- **Synchronization**: Blockchain sync across nodes

#### 6. **Web Interface** (`src/web/`)
- **REST API**: HTTP API สำหรับ external integration
- **Authentication**: JWT-based authentication system
- **Authorization**: Role-based access control
- **Static Files**: Web interface สำหรับ visualization

---

## 🔧 เทคโนโลยีที่ใช้

### Core Technologies:
- **Rust**: Main programming language สำหรับ performance และ safety
- **Oxigraph**: RDF database และ SPARQL engine
- **Ed25519**: Digital signature algorithm
- **SHA-256**: Cryptographic hashing
- **Tokio**: Async runtime สำหรับ networking

### Semantic Web Stack:
- **RDF**: Resource Description Framework สำหรับ data representation
- **OWL2**: Web Ontology Language สำหรับ reasoning
- **SPARQL**: Query language สำหรับ RDF data
- **SHACL**: Shapes Constraint Language สำหรับ validation
- **Turtle/N-Quads**: RDF serialization formats

### Web Technologies:
- **Axum**: Web framework สำหรับ REST API
- **JWT**: JSON Web Tokens สำหรับ authentication
- **bcrypt**: Password hashing
- **CORS**: Cross-origin resource sharing

---

## 📁 โครงสร้างโปรเจค

```
ProvChainOrg/
├── src/
│   ├── core/                 # Core blockchain functionality
│   │   ├── blockchain.rs     # Main blockchain implementation
│   │   ├── atomic_operations.rs # Transaction atomicity
│   │   └── entity.rs         # Traceable entity management
│   ├── storage/              # Data storage and persistence
│   │   ├── rdf_store.rs      # RDF storage implementation
│   │   └── rdf_store_safe.rs # Safe RDF operations
│   ├── transaction/          # Transaction processing
│   │   ├── transaction.rs    # Transaction structure and signing
│   │   └── blockchain.rs     # Transaction blockchain
│   ├── semantic/             # Semantic web features
│   │   ├── owl2_traceability.rs # OWL2-enhanced traceability
│   │   ├── shacl_validator.rs   # SHACL validation
│   │   └── owl_reasoner.rs      # OWL reasoning
│   ├── network/              # P2P networking
│   │   ├── consensus.rs      # Consensus mechanisms
│   │   ├── discovery.rs      # Peer discovery
│   │   └── sync.rs           # Blockchain synchronization
│   ├── web/                  # Web interface and API
│   │   ├── server.rs         # Web server
│   │   ├── auth.rs           # Authentication
│   │   └── handlers.rs       # API handlers
│   ├── validation/           # Input validation and security
│   │   ├── input_validator.rs # Input validation rules
│   │   └── sanitizer.rs      # Input sanitization
│   ├── performance/          # Performance optimization
│   │   ├── memory_optimization.rs # Memory management
│   │   ├── concurrent_operations.rs # Concurrency
│   │   └── scaling.rs        # Horizontal scaling
│   ├── ontology/             # Ontology management
│   │   ├── manager.rs        # Dynamic ontology loading
│   │   └── config.rs         # Ontology configuration
│   └── error.rs              # Comprehensive error handling
├── ontologies/               # OWL ontology files
│   ├── generic_core.owl      # Core traceability ontology
│   ├── healthcare.owl        # Healthcare domain ontology
│   ├── automotive.owl        # Automotive domain ontology
│   └── pharmaceutical.owl    # Pharmaceutical domain ontology
├── queries/                  # SPARQL query templates
├── tests/                    # Test suites
├── docs/                     # Documentation
└── static/                   # Web interface assets
```

---

## 🎯 Use Cases และ Applications

### 1. **UHT Milk Processing** (Primary Demo)
- ติดตาม milk batch จาก farm → UHT processing → distribution
- Monitor environmental conditions (temperature, humidity)
- Quality control และ certification tracking
- Regulatory compliance verification

### 2. **Healthcare Supply Chain**
- Pharmaceutical product traceability
- Medical device tracking
- Cold chain monitoring
- Regulatory compliance (FDA, EMA)

### 3. **Automotive Industry**
- Parts traceability และ recall management
- Manufacturing process tracking
- Quality assurance และ testing
- Supplier verification

### 4. **Generic Supply Chain**
- Multi-tier supplier tracking
- Product authenticity verification
- Environmental impact monitoring
- Sustainability reporting

---

## 🚀 Key Features

### Blockchain Features:
- **RDF-based Blocks**: แต่ละ block เก็บ RDF data
- **Canonicalization**: Consistent hashing ด้วย RDF canonicalization
- **State Root**: Merkle-tree inspired state management
- **Atomic Operations**: Transaction consistency guarantees

### Semantic Web Features:
- **OWL2 Reasoning**: Enhanced reasoning capabilities
- **SPARQL Queries**: Flexible data querying
- **Ontology Support**: Domain-specific ontologies
- **SHACL Validation**: Shape-based data validation

### Security Features:
- **Digital Signatures**: Ed25519-based transaction signing
- **Multi-signature**: สำหรับ critical operations
- **Input Validation**: SQL injection และ XSS protection
- **JWT Authentication**: Secure API access

### Performance Features:
- **Memory Optimization**: Object pooling และ string interning
- **Concurrent Operations**: Multi-threaded processing
- **Caching**: RDF data และ query result caching
- **Compression**: Storage optimization

---

## 🔄 Workflow Example: UHT Milk Traceability

### 1. **Milk Production** (Farmer)
```turtle
@prefix core: <http://provchain.org/core#> .
@prefix prov: <http://www.w3.org/ns/prov#> .

ex:milkBatch1 a core:Batch ;
    core:hasIdentifier "MB001" ;
    core:producedAt "2025-08-08T10:00:00Z"^^xsd:dateTime ;
    prov:wasAttributedTo ex:FarmerJohn .
```

### 2. **UHT Processing** (Manufacturer)
```turtle
ex:uhtProcess1 a core:ManufacturingProcess ;
    core:recordedAt "2025-08-08T12:00:00Z"^^xsd:dateTime ;
    prov:used ex:milkBatch1 ;
    prov:wasAssociatedWith ex:UHTFactory .

ex:uhtMilk1 a core:Batch ;
    core:hasIdentifier "UHT001" ;
    prov:wasGeneratedBy ex:uhtProcess1 ;
    core:derivedFrom ex:milkBatch1 .
```

### 3. **Transport** (Logistics)
```turtle
ex:transport1 a core:TransportProcess ;
    prov:used ex:uhtMilk1 ;
    core:hasCondition ex:condition1 .

ex:condition1 a core:EnvironmentalCondition ;
    core:hasTemperature "4.2"^^xsd:decimal ;
    core:hasHumidity "65.0"^^xsd:decimal .
```

### 4. **Traceability Query**
```sparql
PREFIX core: <http://provchain.org/core#>
PREFIX prov: <http://www.w3.org/ns/prov#>

SELECT ?batch ?process ?condition WHERE {
    ?batch core:hasIdentifier "UHT001" .
    ?process prov:used ?originalBatch .
    ?batch prov:wasGeneratedBy ?process .
    ?transport prov:used ?batch .
    ?transport core:hasCondition ?condition .
}
```

---

## 🛠️ การใช้งาน

### Installation:
```bash
git clone https://github.com/anusornc/provchain-org.git
cd provchain-org
cargo build --release
```

### Basic Usage:
```bash
# Run demo
cargo run -- demo

# Add RDF data
cargo run -- add-file data/sample.ttl

# Query blockchain
cargo run -- query queries/trace_by_batch.sparql

# Start web server
cargo run -- web-server --port 8080

# Enhanced tracing
cargo run -- enhanced-trace BATCH123 --optimization 2
```

### Web API:
```bash
# Authentication
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'

# Add block
curl -X POST http://localhost:8080/api/blockchain/blocks \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"rdf_data":"@prefix ex: <http://example.org/> ..."}'

# Query data
curl -X POST http://localhost:8080/api/blockchain/query \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"sparql":"SELECT * WHERE { ?s ?p ?o }"}'
```

---

## 🧪 Testing

### Unit Tests:
```bash
cargo test --lib
```

### Integration Tests:
```bash
cargo test --test blockchain_tests
cargo test --test enhanced_traceability_demo
```

### Benchmarks:
```bash
cargo bench
```

### End-to-End Tests:
```bash
./scripts/run_e2e_tests.sh
```

---

## 📊 Performance Characteristics

### Blockchain Performance:
- **Block Creation**: ~50ms average
- **RDF Canonicalization**: ~10ms for simple graphs, ~100ms for complex
- **SPARQL Queries**: ~5ms for basic queries, ~50ms for complex joins
- **Memory Usage**: ~100MB baseline, scales with data size

### Scalability:
- **Horizontal Scaling**: Multi-node support ด้วย P2P networking
- **Concurrent Operations**: Multi-threaded processing
- **Storage Optimization**: Compression และ deduplication
- **Query Optimization**: Result caching และ index optimization

---

## 🔒 Security Model

### Authentication & Authorization:
- **JWT-based Authentication**: Secure token-based access
- **Role-based Access Control**: Admin, Farmer, Processor, etc.
- **Multi-signature Support**: สำหรับ critical transactions

### Data Security:
- **Digital Signatures**: Ed25519 signatures สำหรับ all transactions
- **Input Validation**: SQL injection และ XSS protection
- **Data Sanitization**: HTML stripping และ normalization
- **Cryptographic Hashing**: SHA-256 สำหรับ integrity

### Network Security:
- **P2P Encryption**: Secure peer-to-peer communication
- **Consensus Security**: Byzantine fault tolerance
- **Node Authentication**: Cryptographic node identity

---

## 🌐 Domain Applications

### 1. **Food & Agriculture**
- Farm-to-table traceability
- Organic certification tracking
- Food safety compliance
- Environmental impact monitoring

### 2. **Healthcare & Pharmaceuticals**
- Drug supply chain integrity
- Medical device traceability
- Cold chain monitoring
- Regulatory compliance (21 CFR Part 11)

### 3. **Manufacturing**
- Parts และ component tracking
- Quality control processes
- Supplier verification
- Recall management

### 4. **Logistics & Transportation**
- Shipment tracking
- Environmental monitoring
- Chain of custody
- Delivery verification

---

## 🔬 Research Contributions

### 1. **Hybrid RDF Canonicalization**
- Custom fast algorithm สำหรับ simple graphs
- W3C RDFC-1.0 standard สำหรับ complex graphs
- Adaptive algorithm selection based on graph complexity
- Performance benchmarking และ comparison

### 2. **OWL2-Enhanced Traceability**
- Property chain inference สำหรับ implicit relationships
- hasKey constraints สำหรับ entity uniqueness
- Qualified cardinality restrictions
- Enhanced reasoning capabilities

### 3. **Blockchain + Semantic Web Integration**
- RDF data ใน blockchain blocks
- SPARQL queries across blockchain history
- Ontology-based validation
- Semantic interoperability

### 4. **Performance Optimization**
- Memory-efficient data structures
- Concurrent processing algorithms
- Storage optimization techniques
- Query result caching

---

## 📚 Academic Context

### Related Research:
- **Blockchain Traceability**: Supply chain transparency
- **Semantic Web**: RDF, OWL, SPARQL technologies
- **Graph Canonicalization**: RDF graph isomorphism
- **Distributed Systems**: Consensus algorithms

### Publications:
- Research paper: "Blockchain-based Supply Chain Traceability with Semantic Web Technologies"
- Conference presentations on hybrid canonicalization
- Workshop papers on OWL2-enhanced traceability

---

## 🛠️ Development Status

### Current Version: v0.1.0

### Completed Features:
- ✅ Core blockchain implementation
- ✅ RDF storage และ SPARQL querying
- ✅ Transaction system ด้วย digital signatures
- ✅ Web API และ authentication
- ✅ OWL2 reasoning integration
- ✅ Performance optimization tools
- ✅ Comprehensive error handling
- ✅ Input validation และ security

### In Development:
- 🔄 Advanced consensus algorithms
- 🔄 Mobile application interface
- 🔄 Enterprise integration tools
- 🔄 Advanced analytics dashboard

---

## 🎓 Educational Value

### Learning Objectives:
1. **Blockchain Development**: Practical blockchain implementation
2. **Semantic Web Technologies**: RDF, OWL, SPARQL usage
3. **Rust Programming**: Systems programming ด้วย Rust
4. **Cryptography**: Digital signatures และ hashing
5. **Distributed Systems**: P2P networking และ consensus
6. **Web Development**: REST API และ authentication

### Key Concepts Demonstrated:
- Blockchain data structures และ algorithms
- RDF graph canonicalization
- OWL2 reasoning และ inference
- Digital signature schemes
- Consensus mechanisms
- Performance optimization techniques

---

## 🌟 Unique Features

### 1. **Semantic Blockchain**
- First implementation ที่รวม RDF canonicalization ใน blockchain
- SPARQL queries across blockchain history
- Ontology-based data validation

### 2. **Hybrid Canonicalization**
- Adaptive algorithm selection
- Performance optimization สำหรับ different graph types
- W3C standard compliance

### 3. **Domain-Specific Ontologies**
- Pluggable ontology system
- Healthcare, automotive, pharmaceutical domains
- Custom domain support

### 4. **Production-Ready Architecture**
- Comprehensive error handling
- Security best practices
- Performance optimization
- Scalability features

---

## 🎯 Target Audience

### 1. **Researchers**
- Blockchain และ semantic web researchers
- Supply chain management researchers
- Distributed systems researchers

### 2. **Developers**
- Blockchain developers
- Rust developers
- Semantic web developers
- Supply chain application developers

### 3. **Industry**
- Supply chain managers
- Quality assurance teams
- Compliance officers
- IT architects

### 4. **Students**
- Computer science students
- Blockchain course participants
- Semantic web learners
- Systems programming students

---

## 📈 Future Roadmap

### Short-term (3-6 months):
- Mobile application development
- Advanced analytics dashboard
- Enterprise integration APIs
- Performance benchmarking suite

### Medium-term (6-12 months):
- Multi-chain interoperability
- Advanced consensus algorithms
- Machine learning integration
- Regulatory compliance automation

### Long-term (1-2 years):
- IoT device integration
- Real-time monitoring systems
- Global supply chain networks
- Standardization efforts

---

## 🤝 Contributing

### Development Setup:
```bash
# Clone repository
git clone https://github.com/anusornc/provchain-org.git
cd provchain-org

# Install dependencies
cargo build

# Run tests
cargo test

# Start development server
cargo run -- web-server --port 8080
```

### Code Style:
- Follow Rust best practices
- Use comprehensive error handling
- Add tests สำหรับ new features
- Document public APIs
- Validate inputs และ sanitize data

---

## 📞 Contact & Support

### Project Maintainer:
- **Name**: Anusorn Chaikaew
- **Institution**: PhD Research Project
- **Email**: [Contact through GitHub]
- **GitHub**: https://github.com/anusornc/provchain-org

### Documentation:
- **API Documentation**: `docs/api/`
- **User Guide**: `docs/user-guide/`
- **Developer Guide**: `docs/developer/`
- **Research Papers**: `paper/`

---

## 📄 License

This project is developed as part of PhD research. Please refer to the repository for licensing information.

---

**🔗 ProvChain-Org represents a significant advancement in blockchain-based supply chain traceability, combining cutting-edge semantic web technologies with robust blockchain infrastructure to create a production-ready, secure, and scalable solution.**
