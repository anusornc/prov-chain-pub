# Test Data for ProvChain Blockchain

This directory contains RDF test data files for validating the ProvChain blockchain system with supply chain traceability scenarios. All files use Turtle (.ttl) format and are designed to work with the ontology integration system.

## Test Data Files

### 1. minimal_test_data.ttl
**Purpose**: Basic supply chain entities with simple structure for lightweight testing.

**Content**:
- Simple product batch with basic provenance
- Minimal agent information
- Basic processing activity
- Lightweight for quick testing scenarios

**Use Cases**:
- Unit testing with minimal overhead
- Basic blockchain functionality validation
- Quick development testing
- Performance baseline testing

**Example Usage**:
```bash
cargo run -- add-file test_data/minimal_test_data.ttl
cargo test test_minimal_test_data_file
```

### 2. simple_supply_chain_test.ttl
**Purpose**: Mid-complexity supply chain scenario with balanced feature coverage.

**Content**:
- Product batches with proper batch IDs
- Processing and transport activities
- Agent attribution with roles
- Environmental conditions
- Provenance relationships

**Use Cases**:
- Integration testing
- Supply chain traceability validation
- SPARQL query testing
- Ontology validation testing

**Example Usage**:
```bash
cargo run -- add-file test_data/simple_supply_chain_test.ttl
cargo test test_blockchain_with_simple_supply_chain_data
```

### 3. complete_supply_chain_test.ttl
**Purpose**: Comprehensive supply chain scenario with complex relationships and environmental monitoring.

**Content**:
- Multiple product batches with full traceability
- Complex processing activities with ingredient relationships
- Transport activities with environmental conditions
- Geographic location data with blank nodes
- Quality control activities
- Complete agent information with roles
- Certification and compliance data

**Use Cases**:
- Comprehensive integration testing
- Complex RDF canonicalization testing
- Full supply chain traceability validation
- Environmental monitoring testing
- Blank node handling validation

**Example Usage**:
```bash
cargo run -- add-file test_data/complete_supply_chain_test.ttl
cargo test test_blockchain_with_complete_supply_chain_data
```

## Ontology Compliance

All test data files are designed to work with the traceability ontology (`ontology/traceability.owl.ttl`) and include:

### Required Ontology Classes
- `trace:ProductBatch` - Product batches with unique identifiers
- `trace:ProcessingActivity` - Manufacturing and processing activities
- `trace:TransportActivity` - Logistics and transport activities
- `trace:Farmer` - Agricultural producers
- `trace:Manufacturer` - Processing facilities
- `trace:LogisticsProvider` - Transport companies
- `trace:EnvironmentalCondition` - Temperature and humidity monitoring

### Required Properties
- `trace:hasBatchID` - Unique batch identifier (required for ProductBatch)
- `trace:recordedAt` - Timestamp for activities (required for activities)
- `trace:hasTemperature` - Temperature readings for environmental conditions
- `trace:hasHumidity` - Humidity readings for environmental conditions
- `prov:wasAttributedTo` - Agent attribution for provenance
- `prov:used` - Input relationships for activities
- `prov:wasGeneratedBy` - Output relationships for products

### Standard Prefixes Used
```turtle
@prefix ex: <http://example.org/> .
@prefix trace: <http://provchain.org/trace#> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix geo: <http://www.w3.org/2003/01/geo/wgs84_pos#> .
```

## Testing Integration

### Automated Tests
Each test data file is validated by corresponding test cases:

```bash
# Test minimal data structure
cargo test test_minimal_test_data_file

# Test complete supply chain data
cargo test test_complete_supply_chain_test_file

# Test supply chain provenance relationships
cargo test test_supply_chain_provenance_relationships

# Test blockchain integration with test data
cargo test test_blockchain_with_minimal_test_data
cargo test test_blockchain_with_complete_supply_chain_data
cargo test test_blockchain_with_both_test_files
```

### Manual Testing
```bash
# Add test data to blockchain
cargo run -- add-file test_data/simple_supply_chain_test.ttl

# Validate blockchain integrity
cargo run -- validate

# Query the test data
cargo run -- query queries/trace_by_batch_ontology.sparql
```

## Data Structure Examples

### Product Batch Example
```turtle
ex:milkBatch1 a trace:ProductBatch ;
    trace:hasBatchID "MB001" ;
    trace:producedAt "2025-08-08T10:00:00Z"^^xsd:dateTime ;
    prov:wasAttributedTo ex:FarmerJohn .
```

### Processing Activity Example
```turtle
ex:uhtProcess1 a trace:ProcessingActivity ;
    trace:recordedAt "2025-08-08T12:00:00Z"^^xsd:dateTime ;
    prov:used ex:milkBatch1 ;
    prov:wasAssociatedWith ex:UHTFactory .
```

### Environmental Condition Example
```turtle
ex:condition1 a trace:EnvironmentalCondition ;
    trace:hasTemperature "4.2"^^xsd:decimal ;
    trace:hasHumidity "65.0"^^xsd:decimal ;
    trace:hasConditionTimestamp "2025-08-08T14:00:00Z"^^xsd:dateTime .
```

### Agent Example
```turtle
ex:FarmerJohn a trace:Farmer ;
    rdfs:label "John's Dairy Farm" ;
    geo:lat "40.7128"^^xsd:decimal ;
    geo:long "-74.0060"^^xsd:decimal .
```

## Supply Chain Scenarios

### Scenario 1: Basic Milk Processing (simple_supply_chain_test.ttl)
1. **Farm Origin**: Raw milk batch from dairy farm
2. **Processing**: UHT processing at manufacturing facility
3. **Transport**: Cold chain logistics with environmental monitoring
4. **Traceability**: Complete provenance chain from farm to transport

### Scenario 2: Complex Multi-Stage Processing (complete_supply_chain_test.ttl)
1. **Multiple Farms**: Various ingredient sources
2. **Processing Stages**: Multiple transformation activities
3. **Quality Control**: Certification and testing activities
4. **Environmental Monitoring**: Temperature and humidity tracking
5. **Geographic Tracking**: Location data for all activities
6. **Complex Relationships**: Ingredient derivation and batch merging

## RDF Canonicalization Testing

The test data files are specifically designed to test RDF canonicalization features:

### Blank Node Handling
- Geographic coordinates using blank nodes
- Complex nested structures
- Anonymous environmental conditions

### Semantic Equivalence
- Different serialization orders
- Equivalent but differently structured RDF
- Blank node identifier variations

### Magic Placeholder Testing
- Blank nodes in subject positions (Magic_S)
- Blank nodes in object positions (Magic_O)
- Complex blank node relationships

## SPARQL Query Testing

Test data supports various SPARQL query patterns:

### Batch Traceability Queries
```sparql
SELECT ?batch ?activity ?agent ?timestamp WHERE {
    ?batch a trace:ProductBatch ;
           trace:hasBatchID "MB001" .
    ?activity prov:used ?batch ;
              prov:wasAssociatedWith ?agent ;
              trace:recordedAt ?timestamp .
} ORDER BY ?timestamp
```

### Environmental Condition Queries
```sparql
SELECT ?batch ?temp ?humidity WHERE {
    ?activity prov:used ?batch ;
              trace:hasCondition ?condition .
    ?condition trace:hasTemperature ?temp ;
               trace:hasHumidity ?humidity .
    FILTER(?temp > 5.0)
}
```

### Agent Analysis Queries
```sparql
SELECT ?agentType (COUNT(?agent) AS ?count) WHERE {
    ?activity prov:wasAssociatedWith ?agent .
    ?agent a ?agentType .
    ?agentType rdfs:subClassOf trace:TraceAgent .
} GROUP BY ?agentType
```

## Creating Custom Test Data

### Guidelines for New Test Data
1. **Use Standard Prefixes**: Follow the established prefix conventions
2. **Include Required Properties**: Ensure all required ontology properties are present
3. **Provide Timestamps**: Use ISO 8601 format with XSD dateTime type
4. **Add Agent Attribution**: Include proper agent roles and attribution
5. **Environmental Data**: Include temperature and humidity for transport activities
6. **Unique Identifiers**: Use unique batch IDs and entity URIs

### Template Structure
```turtle
@prefix ex: <http://example.org/> .
@prefix trace: <http://provchain.org/trace#> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

# Product batch with required properties
ex:yourBatch a trace:ProductBatch ;
    trace:hasBatchID "YOUR_BATCH_ID" ;
    trace:producedAt "2025-08-09T10:00:00Z"^^xsd:dateTime ;
    prov:wasAttributedTo ex:yourAgent .

# Agent with proper classification
ex:yourAgent a trace:Farmer ;
    rdfs:label "Your Agent Name" .

# Activity with timestamp and relationships
ex:yourActivity a trace:ProcessingActivity ;
    trace:recordedAt "2025-08-09T12:00:00Z"^^xsd:dateTime ;
    prov:used ex:yourBatch ;
    prov:wasAssociatedWith ex:yourAgent .
```

## Validation and Quality Assurance

### Ontology Validation
All test data files are validated against the traceability ontology:
- Class membership validation
- Required property checking
- Data type validation
- Relationship consistency

### Data Integrity Testing
- RDF parsing validation
- Blank node handling
- Canonicalization consistency
- SPARQL query compatibility

### Performance Testing
- File size and complexity metrics
- Parsing and processing time
- Memory usage during processing
- Canonicalization performance

## Usage in Development

### Quick Testing
```bash
# Test with minimal data
cargo run -- add-file test_data/minimal_test_data.ttl
cargo run -- validate

# Test with complex data
cargo run -- add-file test_data/complete_supply_chain_test.ttl
cargo run -- query queries/trace_by_batch_ontology.sparql
```

### Integration Testing
```bash
# Run all data validation tests
cargo test test_data_validation

# Run blockchain integration tests
cargo test blockchain_with_test_data

# Run ontology integration tests
cargo test ontology_integration_tests
```

### Development Workflow
1. **Start with minimal data** for basic functionality testing
2. **Progress to simple data** for feature validation
3. **Use complete data** for comprehensive testing
4. **Create custom data** for specific use cases

## File Status and Maintenance

### Current Status
- ✅ **minimal_test_data.ttl**: Validated and working
- ✅ **simple_supply_chain_test.ttl**: Validated and working
- ✅ **complete_supply_chain_test.ttl**: Validated and working

### Maintenance Notes
- All files are ontology-compliant
- Regular validation against current ontology version
- Updated to use current namespace (`http://provchain.org/`)
- Compatible with RDF canonicalization algorithm
- Tested with all SPARQL queries

### Future Enhancements
- Additional supply chain scenarios
- More complex environmental monitoring data
- Geographic information system (GIS) integration
- Multi-language support for labels
- Extended certification and compliance data

For more information about using these test data files, see:
- [Run.md](../Run.md) - Instructions for running with test data
- [TESTING_SUMMARY.md](../TESTING_SUMMARY.md) - Comprehensive testing analysis
- [ONTOLOGY_INTEGRATION_COMPLETE.md](../ONTOLOGY_INTEGRATION_COMPLETE.md) - Ontology features
