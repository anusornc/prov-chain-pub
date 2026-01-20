# ProvChainOrg Transaction System Guide

## Overview

The ProvChainOrg transaction system provides a comprehensive blockchain-based solution for supply chain traceability with multi-participant support, digital signatures, and environmental monitoring. This guide explains how to use the transaction system for various supply chain scenarios.

## Architecture

### Core Components

1. **Transaction System** (`src/transaction.rs`)
   - Structured transaction types for supply chain operations
   - Digital signature support with Ed25519 cryptography
   - Transaction validation and business logic
   - Multi-signature support for critical operations

2. **Wallet System** (`src/wallet.rs`)
   - Multi-participant wallet management
   - Secure key storage and management
   - Participant identity and role management
   - Certificate management for compliance

3. **Transaction Blockchain** (`src/transaction_blockchain.rs`)
   - Blockchain implementation for transaction storage
   - UTXO (Unspent Transaction Output) model
   - Block creation and validation
   - Persistent storage with disk serialization

4. **Demo System** (`src/uht_demo.rs`, `src/demo_runner.rs`)
   - Comprehensive UHT manufacturing demo
   - Multiple demo scenarios
   - Interactive demo runner

## Transaction Types

### 1. Production Transactions
- **Purpose**: Record raw material production (e.g., milk from farms)
- **Participants**: Producers (farmers, suppliers)
- **Data**: Batch ID, quantity, location, environmental conditions
- **Example**: Farmer produces 2000L of organic milk

### 2. Processing Transactions
- **Purpose**: Record manufacturing processes (e.g., UHT pasteurization)
- **Participants**: Manufacturers (processors, packagers)
- **Data**: Input batches, output batch, process type, conditions
- **Example**: UHT processor converts raw milk to shelf-stable milk

### 3. Quality Transactions
- **Purpose**: Record quality control and testing results
- **Participants**: Quality labs, auditors
- **Data**: Test type, results, values, lab certification
- **Example**: Microbiological testing of milk batch

### 4. Transport Transactions
- **Purpose**: Record logistics and transport activities
- **Participants**: Logistics providers, carriers
- **Data**: Origin, destination, transport conditions, cold chain data
- **Example**: Refrigerated transport from processor to retailer

### 5. Transfer Transactions
- **Purpose**: Record ownership transfers between participants
- **Participants**: Any participant type
- **Data**: Asset transfer, ownership change
- **Example**: Transfer of processed goods from manufacturer to retailer

### 6. Environmental Transactions
- **Purpose**: Record environmental monitoring data
- **Participants**: Any participant with sensors
- **Data**: Temperature, humidity, pressure, sensor readings
- **Example**: Cold chain monitoring during transport

### 7. Compliance Transactions
- **Purpose**: Record regulatory compliance events
- **Participants**: Auditors, regulatory authorities
- **Data**: Regulation type, compliance status, certificates
- **Example**: FDA inspection and approval

## Participant Types

### 1. Producer
- **Role**: Raw material production
- **Permissions**: Can create production transactions, transfer ownership
- **Examples**: Farmers, raw material suppliers
- **Certificates**: Organic certification, quality standards

### 2. Manufacturer
- **Role**: Processing and manufacturing
- **Permissions**: Can create processing transactions, transfer ownership
- **Examples**: UHT processors, food manufacturers
- **Certificates**: FDA approval, ISO certifications

### 3. Logistics Provider
- **Role**: Transportation and logistics
- **Permissions**: Can create transport transactions
- **Examples**: Shipping companies, cold chain providers
- **Certificates**: Transport licenses, cold chain certifications

### 4. Quality Lab
- **Role**: Quality control and testing
- **Permissions**: Can create quality transactions
- **Examples**: Testing laboratories, certification bodies
- **Certificates**: ISO 17025, accreditation certificates

### 5. Auditor
- **Role**: Regulatory compliance and auditing
- **Permissions**: Can create compliance transactions, view all data
- **Examples**: Regulatory authorities, third-party auditors
- **Certificates**: Regulatory authority credentials

### 6. Retailer
- **Role**: Final distribution and sales
- **Permissions**: Can transfer ownership, receive goods
- **Examples**: Supermarkets, distributors
- **Certificates**: Retail licenses, food safety certifications

### 7. Administrator
- **Role**: System administration
- **Permissions**: Full access to all operations
- **Examples**: System operators, network administrators
- **Certificates**: Administrative credentials

## Digital Signatures and Security

### Cryptographic Implementation
- **Algorithm**: Ed25519 (Edwards-curve Digital Signature Algorithm)
- **Key Size**: 256-bit private keys, 256-bit public keys
- **Signature Size**: 512-bit signatures
- **Security**: Quantum-resistant, high performance

### Signature Process
1. **Transaction Creation**: Participant creates transaction with data
2. **Hash Calculation**: Transaction data is hashed using SHA-256
3. **Signing**: Private key signs the transaction hash
4. **Verification**: Public key verifies the signature
5. **Multi-Signature**: Critical transactions require multiple signatures

### Multi-Signature Requirements
- **Compliance Transactions**: Require 2 signatures (auditor + authority)
- **Quality Transactions**: Require 2 signatures (lab + authority)
- **Large Value Transfers**: Require 2 signatures (sender + receiver)
- **Regular Transactions**: Require 1 signature (participant)

## Environmental Monitoring

### Sensor Data Integration
```rust
EnvironmentalConditions {
    temperature: Some(4.0),     // Celsius
    humidity: Some(60.0),       // Percentage
    pressure: Some(1010.0),     // hPa
    timestamp: Utc::now(),      // UTC timestamp
    sensor_id: Some("SENSOR-001".to_string()),
}
```

### Cold Chain Monitoring
- **Temperature Tracking**: Continuous monitoring during transport
- **Humidity Control**: Moisture level monitoring
- **Pressure Monitoring**: Atmospheric pressure tracking
- **Sensor Validation**: Sensor ID and calibration tracking

## Running Demos

### Command Line Interface

#### 1. UHT Manufacturing Demo
```bash
# Run the complete UHT manufacturing supply chain demo
cargo run -- transaction-demo --demo-type uht

# Alternative command
cargo run -- transaction-demo -d manufacturing
```

#### 2. Basic Blockchain Demo
```bash
# Run basic blockchain operations demo
cargo run -- transaction-demo --demo-type basic

# Alternative command
cargo run -- transaction-demo -d blockchain
```

#### 3. Transaction Signing Demo
```bash
# Run transaction signing and validation demo
cargo run -- transaction-demo --demo-type signing

# Alternative command
cargo run -- transaction-demo -d signatures
```

#### 4. Multi-Participant Demo
```bash
# Run multi-participant network demo
cargo run -- transaction-demo --demo-type multi

# Alternative command
cargo run -- transaction-demo -d participants
```

#### 5. Interactive Demo Menu
```bash
# Run interactive demo menu (default)
cargo run -- transaction-demo

# Explicit interactive mode
cargo run -- transaction-demo --demo-type interactive
```

#### 6. All Demos
```bash
# Run all demos in sequence
cargo run -- transaction-demo --demo-type all
```

### Demo Scenarios

#### UHT Manufacturing Demo Flow
1. **Participant Setup**: Register 6 participants with certificates
2. **Milk Production**: Farmers produce organic and premium milk
3. **Quality Testing**: Lab tests raw milk for safety
4. **UHT Processing**: Processor pasteurizes milk at 138°C
5. **Post-Processing QC**: Lab tests processed milk
6. **Distribution Transport**: Cold chain transport to distribution center
7. **Final Distribution**: Delivery to retail locations
8. **Blockchain Finalization**: Create blocks and validate chain

#### Expected Output
```
🥛 ProvChainOrg UHT Manufacturing Supply Chain Demo
============================================================
Demonstrating complete milk traceability from farm to shelf
with multiple participants, digital signatures, and environmental monitoring.

🏭 Setting up UHT Manufacturing Supply Chain Participants...
✅ Registered Farmer John (Organic Dairy) - ID: [UUID]
✅ Registered Farmer Mary (Premium Dairy) - ID: [UUID]
✅ Registered UHT Processor (Alpine Corp) - ID: [UUID]
✅ Registered Quality Lab (Midwest Testing) - ID: [UUID]
✅ Registered Logistics Provider (ColdChain Express) - ID: [UUID]
✅ Registered Retailer (FreshMart) - ID: [UUID]

🥛 Starting Complete UHT Manufacturing Supply Chain Demo
============================================================

📍 Step 1: Milk Production
------------------------------
✅ Farmer John produced 2000L organic milk - TX: [TX_ID]
✅ Farmer Mary produced 1500L premium milk - TX: [TX_ID]

🔬 Step 2: Quality Testing of Raw Milk
----------------------------------------
✅ Quality test for organic milk batch - PASSED - TX: [TX_ID]
✅ Quality test for premium milk batch - PASSED - TX: [TX_ID]

🏭 Step 3: UHT Processing
-------------------------
✅ UHT processing completed - 3500L processed - TX: [TX_ID]

🔍 Step 4: Post-Processing Quality Control
---------------------------------------------
✅ Post-UHT microbiological test - PASSED - TX: [TX_ID]
✅ Nutritional analysis - PASSED (3.2% fat) - TX: [TX_ID]

🚛 Step 5: Transport to Distribution Center
---------------------------------------------
✅ Transport to distribution center - Cold chain maintained - TX: [TX_ID]

🏪 Step 6: Final Distribution to Retailer
------------------------------------------
✅ Final distribution to retailer - TX: [TX_ID]

⛓️  Step 7: Finalizing Transactions on Blockchain
--------------------------------------------------
✅ All transactions finalized and saved to blockchain

📊 Demo Results and Statistics
========================================
Blockchain Statistics:
  📦 Total Blocks: 1
  ⏳ Pending Transactions: 0
  👥 Total Participants: 6
  💰 Total UTXOs: 9

Participant Distribution:
  Producer: 2
  Manufacturer: 1
  QualityLab: 1
  LogisticsProvider: 1
  Retailer: 1

Transaction Distribution:
  Production: 2
  Quality: 4
  Processing: 1
  Transport: 2

🎉 UHT Manufacturing Supply Chain Demo Completed Successfully!
   Complete traceability from farm to shelf achieved.
   All transactions signed and verified by participants.
   Environmental conditions monitored throughout the chain.
   Quality control checkpoints passed at each stage.

✅ Blockchain validation: PASSED
```

## API Usage

### Creating Participants
```rust
use provchain_org::wallet::Participant;

// Create a farmer
let farmer = Participant::new_farmer(
    "John's Organic Dairy Farm".to_string(),
    "Vermont, USA".to_string(),
);

// Create a UHT processor
let processor = Participant::new_uht_manufacturer(
    "Alpine UHT Processing Corp".to_string(),
    "Wisconsin, USA".to_string(),
);
```

### Creating Transactions
```rust
use provchain_org::transaction_blockchain::TransactionBlockchain;

let mut blockchain = TransactionBlockchain::new("./data")?;

// Register participants
let farmer_id = blockchain.register_participant(farmer)?;
let processor_id = blockchain.register_participant(processor)?;

// Create production transaction
let tx = blockchain.create_production_transaction(
    farmer_id,
    "ORGANIC-MILK-BATCH-001".to_string(),
    2000.0, // 2000 liters
    "Vermont, USA".to_string(),
    Some(environmental_conditions),
    false, // No encryption
)?;

// Submit transaction
let tx_id = blockchain.submit_transaction(tx)?;
```

### Environmental Monitoring
```rust
use provchain_org::transaction::EnvironmentalConditions;
use chrono::Utc;

let conditions = EnvironmentalConditions {
    temperature: Some(4.0),     // Cold chain temperature
    humidity: Some(60.0),       // Humidity percentage
    pressure: Some(1010.0),     // Atmospheric pressure
    timestamp: Utc::now(),      // Current timestamp
    sensor_id: Some("TRUCK-SENSOR-001".to_string()),
};
```

## Testing

### Unit Tests
```bash
# Run all tests
cargo test

# Run specific module tests
cargo test transaction
cargo test wallet
cargo test uht_demo
```

### Integration Tests
```bash
# Run integration tests
cargo test --test '*'

# Run specific integration test
cargo test --test real_world_traceability_tests
```

### Benchmarks
```bash
# Run performance benchmarks
cargo bench

# Run specific benchmarks
cargo bench blockchain_performance
cargo bench consensus_benchmarks
```

## Data Persistence

### Storage Structure
```
./data/
├── blockchain.json          # Blockchain data
├── wallets/                 # Participant wallets
│   ├── [participant-id].wallet
│   └── ...
├── utxos.json              # UTXO set
└── metadata.json           # Blockchain metadata
```

### Backup and Recovery
```rust
// Create backup
let backup_path = blockchain.create_backup()?;

// Load from backup
let blockchain = TransactionBlockchain::load_from_backup(&backup_path)?;
```

## Security Considerations

### Key Management
- Private keys are stored encrypted on disk
- In production, use hardware security modules (HSMs)
- Implement proper key rotation policies
- Use secure random number generation

### Network Security
- Implement TLS for network communication
- Use certificate pinning for participant verification
- Implement rate limiting and DDoS protection
- Regular security audits and penetration testing

### Data Privacy
- Implement data encryption at rest
- Use zero-knowledge proofs for sensitive data
- Implement access control and authorization
- Comply with data protection regulations (GDPR, etc.)

## Production Deployment

### System Requirements
- **CPU**: Multi-core processor (4+ cores recommended)
- **Memory**: 8GB+ RAM for production workloads
- **Storage**: SSD storage for blockchain data
- **Network**: Stable internet connection with low latency

### Configuration
```toml
# config.toml
[blockchain]
max_block_size = 1000
max_transactions_per_block = 100
block_time_seconds = 30

[network]
port = 8080
max_peers = 50
sync_interval_seconds = 10

[storage]
data_directory = "./data"
backup_interval_hours = 24
max_backup_files = 7
```

### Monitoring
- Implement comprehensive logging
- Set up metrics collection (Prometheus/Grafana)
- Monitor blockchain health and performance
- Set up alerting for critical issues

## Troubleshooting

### Common Issues

#### 1. Transaction Validation Errors
```
Error: Invalid signatures
```
**Solution**: Ensure all required signatures are present and valid.

#### 2. Insufficient Permissions
```
Error: Participant does not have permission for this operation
```
**Solution**: Check participant type and permissions configuration.

#### 3. UTXO Not Found
```
Error: Referenced UTXO not found
```
**Solution**: Ensure input transactions exist and are unspent.

#### 4. Storage Issues
```
Error: Cannot write to blockchain file
```
**Solution**: Check file permissions and disk space.

### Debug Mode
```bash
# Run with debug logging
RUST_LOG=debug cargo run -- transaction-demo --demo-type uht

# Run with trace logging
RUST_LOG=trace cargo run -- transaction-demo --demo-type uht
```

## Future Enhancements

### Planned Features
1. **Consensus Mechanisms**: Implement PoS or PoA consensus
2. **Smart Contracts**: Add programmable transaction logic
3. **Privacy Features**: Implement zero-knowledge proofs
4. **Scalability**: Add sharding and layer-2 solutions
5. **Interoperability**: Connect with other blockchain networks

### Contributing
1. Fork the repository
2. Create a feature branch
3. Implement changes with tests
4. Submit a pull request
5. Follow code review process

## Support

For questions, issues, or contributions:
- GitHub Issues: [Repository Issues](https://github.com/anusornc/provchain-org/issues)
- Documentation: [Project Documentation](./README.md)
- Email: [Contact Information]

---

*This guide provides comprehensive information about the ProvChainOrg transaction system. For the latest updates and detailed API documentation, please refer to the source code and inline documentation.*
