# Ontology Integration Guide for ProvChain

## Overview
The `ontology/traceability.owl.ttl` file contains a comprehensive PROV-O extension ontology for supply chain traceability. This document explains how to integrate and use this ontology within the ProvChain system.

## Current Ontology Structure

### Core Classes
The ontology defines a hierarchical class structure based on PROV-O:

**Entities (Things being traced)**:
- `TraceEntity` (base class for all traceable items)
  - `ProductBatch` (finished product batches)
  - `IngredientLot` (raw materials/ingredients)

**Activities (Processes/Events)**:
- `TraceActivity` (base class for all activities)
  - `ProcessingActivity` (manufacturing steps like UHT)
  - `TransportActivity` (logistics/shipping)
  - `QualityCheck` (testing/inspection)

**Agents (Actors in supply chain)**:
- `TraceAgent` (base class for all actors)
  - `Farmer`, `Manufacturer`, `LogisticsProvider`, `Retailer`, `Customer`

**Supporting Classes**:
- `EnvironmentalCondition` (temperature, humidity readings)
- `Certificate` (quality certifications, test reports)

### Key Properties
- `hasBatchID`: Unique identifier for batches/lots
- `originLocation`: Geographic origin using GeoSPARQL
- `producedAt`, `recordedAt`: Timestamps
- `hasCondition`: Links to environmental readings
- `hasTemperature`, `hasHumidity`: Environmental measurements
- `hasCertificate`: Links to quality documents
- `lotDerivedFrom`: Ingredient traceability

## Integration Strategies

### 1. Automatic Ontology Loading (Recommended)

Add ontology loading to the blockchain initialization:

```rust
// In src/blockchain.rs
impl Blockchain {
    pub fn new() -> Self {
        let mut bc = Blockchain {
            chain: vec![],
            rdf_store: RDFStore::new(),
        };
        
        // Load the traceability ontology
        bc.load_ontology();
        
        // Create genesis block
        bc.chain.push(Block::genesis());
        bc
    }
    
    fn load_ontology(&mut self) {
        if let Ok(ontology_data) = std::fs::read_to_string("ontology/traceability.owl.ttl") {
            let ontology_graph = oxigraph::model::NamedNode::new("http://provchain.org/ontology").unwrap();
            self.rdf_store.load_ontology(&ontology_data, &ontology_graph);
            println!("Loaded traceability ontology");
        } else {
            eprintln!("Warning: Could not load ontology file");
        }
    }
}
```

### 2. Configuration-Based Loading

Add ontology path to `config.toml`:

```toml
[ontology]
path = "ontology/traceability.owl.ttl"
graph_name = "http://provchain.org/ontology"
auto_load = true
```

Then load based on configuration:

```rust
// In src/config.rs
#[derive(Debug, Deserialize)]
pub struct OntologyConfig {
    pub path: String,
    pub graph_name: String,
    pub auto_load: bool,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    // ... existing fields
    pub ontology: Option<OntologyConfig>,
}
```

### 3. Demo Integration

Update `src/demo.rs` to use ontology classes:

```rust
pub fn run_demo() {
    let mut bc = Blockchain::new(); // Now loads ontology automatically

    // Use ontology classes in demo data
    let farmer_data = r#"
        @prefix trace: <http://provchain.org/trace#> .
        @prefix prov: <http://www.w3.org/ns/prov#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        ex:milkBatch1 a trace:ProductBatch ;
            trace:hasBatchID "MB001" ;
            trace:producedAt "2025-08-08T10:00:00Z"^^xsd:dateTime ;
            prov:wasAttributedTo ex:FarmerJohn .

        ex:FarmerJohn a trace:Farmer ;
            rdfs:label "John's Dairy Farm" .
    "#;
    bc.add_block(farmer_data.into());

    let processing_data = r#"
        @prefix trace: <http://provchain.org/trace#> .
        @prefix prov: <http://www.w3.org/ns/prov#> .

        ex:uhtProcess1 a trace:ProcessingActivity ;
            trace:recordedAt "2025-08-08T12:00:00Z"^^xsd:dateTime ;
            prov:used ex:milkBatch1 ;
            prov:wasAssociatedWith ex:UHTFactory .

        ex:uhtMilk1 a trace:ProductBatch ;
            trace:hasBatchID "UHT001" ;
            prov:wasGeneratedBy ex:uhtProcess1 ;
            trace:lotDerivedFrom ex:milkBatch1 .
    "#;
    bc.add_block(processing_data.into());
}
```

## Usage Examples

### 1. Validating Data Against Ontology

Create validation functions that check if RDF data conforms to the ontology:

```rust
impl RDFStore {
    pub fn validate_against_ontology(&self, data_graph: &NamedNode) -> bool {
        // Query to check if all entities have proper types
        let validation_query = r#"
            PREFIX trace: <http://provchain.org/trace#>
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            
            ASK {
                GRAPH ?dataGraph {
                    ?entity rdf:type ?type .
                    FILTER(?type = trace:ProductBatch || ?type = trace:IngredientLot || 
                           ?type = trace:ProcessingActivity || ?type = trace:TransportActivity)
                }
            }
        "#;
        
        // Execute validation query
        match self.query(validation_query) {
            oxigraph::sparql::QueryResults::Boolean(result) => result,
            _ => false,
        }
    }
}
```

### 2. Ontology-Aware SPARQL Queries

Update the queries in `queries/` directory to use ontology classes:

```sparql
# queries/trace_by_batch_ontology.sparql
PREFIX trace: <http://provchain.org/trace#>
PREFIX prov: <http://www.w3.org/ns/prov#>

SELECT ?batch ?activity ?agent ?timestamp WHERE {
    ?batch a trace:ProductBatch ;
           trace:hasBatchID ?batchId ;
           prov:wasGeneratedBy ?activity .
    
    ?activity a trace:ProcessingActivity ;
              trace:recordedAt ?timestamp ;
              prov:wasAssociatedWith ?agent .
    
    ?agent a trace:Manufacturer .
    
    FILTER(?batchId = "UHT001")
}
```

### 3. Environmental Conditions Integration

```rust
// Example of adding environmental data using ontology
let environmental_data = r#"
    @prefix trace: <http://provchain.org/trace#> .
    @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

    ex:transport1 a trace:TransportActivity ;
        trace:recordedAt "2025-08-08T14:00:00Z"^^xsd:dateTime ;
        trace:hasCondition ex:condition1 .

    ex:condition1 a trace:EnvironmentalCondition ;
        trace:hasTemperature "4.2"^^xsd:decimal ;
        trace:hasHumidity "65.0"^^xsd:decimal ;
        trace:hasConditionTimestamp "2025-08-08T14:00:00Z"^^xsd:dateTime .
"#;
```

## Implementation Priority

### Phase 1: Basic Integration
1. ✅ **Ontology file exists** - `ontology/traceability.owl.ttl`
2. ✅ **RDF store has load_ontology method** - Available but unused
3. 🔄 **Automatic loading in blockchain initialization**
4. 🔄 **Update demo to use ontology classes**

### Phase 2: Enhanced Features
1. 🔄 **Configuration-based ontology loading**
2. 🔄 **Data validation against ontology**
3. 🔄 **Ontology-aware SPARQL queries**
4. 🔄 **Test cases for ontology integration**

### Phase 3: Advanced Features
1. 🔄 **Multiple ontology support**
2. 🔄 **Ontology versioning**
3. 🔄 **Reasoning capabilities**
4. 🔄 **Schema validation for incoming RDF**

## Benefits of Integration

1. **Standardization**: Ensures all supply chain data follows consistent vocabulary
2. **Validation**: Can validate incoming RDF data against expected schema
3. **Interoperability**: Standard ontology enables data exchange between systems
4. **Reasoning**: Enables inference of implicit relationships
5. **Documentation**: Ontology serves as living documentation of data model

## Next Steps

1. Implement automatic ontology loading in blockchain initialization
2. Update demo and test data to use ontology classes
3. Create validation functions for ontology compliance
4. Add test cases for ontology integration
5. Update SPARQL queries to leverage ontology structure

The ontology provides a solid foundation for standardized supply chain traceability data and should be integrated as a core component of the ProvChain system.
