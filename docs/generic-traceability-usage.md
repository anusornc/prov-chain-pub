# Generic Traceability System Usage Guide

## Overview

The generic traceability system provides a flexible framework for tracking entities across multiple domains while maintaining domain-specific validation and processing capabilities. This guide explains how to use the core components.

## Core Concepts

### TraceableEntity
The fundamental unit of traceability that can represent any object in any domain.

```rust
use provchain_org::core::entity::{TraceableEntity, EntityType, DomainType};

// Create a generic product entity
let entity = TraceableEntity::new(
    "product_12345".to_string(),        // Entity ID
    EntityType::Product,                // Entity type
    DomainType::SupplyChain             // Domain context
);

// Add properties
entity.add_property("name".to_string(), PropertyValue::String("Milk".to_string()));
entity.add_property("batch_id".to_string(), PropertyValue::String("BATCH001".to_string()));
```

### Domain Plugins
Domain plugins provide domain-specific validation and processing capabilities.

```rust
use provchain_org::domain::{DomainManager, DomainPlugin};
use provchain_org::domain::adapters::OwlDomainAdapter;

// Create domain manager
let mut manager = DomainManager::new();

// Create domain configuration
let config = serde_yaml::Value::default();

// Create OWL domain adapter
let adapter = Box::new(OwlDomainAdapter::from_config(&config)?);

// Register plugin
manager.register_plugin(adapter)?;
```

### OWL2 Reasoning
The enhanced OWL reasoner provides advanced semantic capabilities.

```rust
use provchain_org::semantic::{OwlReasonerConfig, OwlReasoner};

// Configure OWL2 features
let mut config = OwlReasonerConfig::default();
config.process_owl2_features = true;
config.enable_has_key_validation = true;
config.enable_property_chain_inference = true;
config.enable_qualified_cardinality_validation = true;

// Create reasoner
let reasoner = OwlReasoner::new(config)?;
```

## Domain Extension

### Creating a New Domain Adapter
To support a new domain, create a domain adapter by implementing the `DomainPlugin` trait:

```rust
use provchain_org::domain::plugin::{DomainPlugin, DomainConfig, ValidationResult, ProcessedEntity, EntityData};

pub struct CustomDomainAdapter {
    config: DomainConfig,
}

impl DomainPlugin for CustomDomainAdapter {
    fn domain_id(&self) -> &str { &self.config.domain_id }
    fn name(&self) -> &str { &self.config.name }
    fn description(&self) -> &str { &self.config.description }
    
    fn is_valid_entity_type(&self, entity_type: &str) -> bool {
        // Domain-specific entity type validation
        !entity_type.is_empty()
    }
    
    fn validate_entity(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        // Domain-specific entity validation
        if entity_data.entity_id.is_empty() {
            return Ok(ValidationResult::Invalid("Entity ID is required".to_string()));
        }
        Ok(ValidationResult::Valid)
    }
    
    fn process_entity(&self, entity_data: &EntityData) -> Result<ProcessedEntity> {
        // Domain-specific entity processing
        Ok(ProcessedEntity {
            entity_id: entity_data.entity_id.clone(),
            entity_type: entity_data.entity_type.clone(),
            processed_data: entity_data.data.clone(),
            domain_context: self.config.domain_id.clone(),
        })
    }
}
```

### Using OWL Domain Adapter
The OWL domain adapter loads configuration from OWL ontology files:

```rust
use provchain_org::domain::adapters::OwlDomainAdapter;
use serde_yaml::Value;

let config_yaml = r#"
    domain_id: "custom_domain"
    name: "Custom Domain"
    description: "Custom traceability domain"
    core_ontology_path: "ontologies/core.owl"
    domain_ontology_path: "ontologies/custom.owl"
    ontology_path: "ontologies/custom.owl"
    enabled: true
    priority: 1
"#;

let config: Value = serde_yaml::from_str(config_yaml)?;
let adapter = Box::new(OwlDomainAdapter::from_config(&config)?);
```

## OWL2 Features

### owl:hasKey Support
Define uniqueness constraints using owl:hasKey axioms in your ontology:

```turtle
@prefix : <http://provchain.org/custom#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

:Batch a owl:Class ;
    rdfs:label "Batch" ;
    owl:hasKey ( :batchId ) ;
    rdfs:comment "A batch with a unique batch ID" .

:ProductBatch a owl:Class ;
    rdfs:label "ProductBatch" ;
    owl:hasKey ( :productId :batchNumber ) ;
    rdfs:comment "A product batch with a composite key" .
```

### Property Chain Axioms
Define transitive relationships using property chain axioms:

```turtle
:suppliedTo a owl:ObjectProperty ;
    rdfs:label "suppliedTo" .

:transitivelySuppliedTo a owl:ObjectProperty ;
    rdfs:label "transitivelySuppliedTo" ;
    owl:propertyChainAxiom ( :suppliedTo :suppliedTo ) .
```

### Qualified Cardinality Restrictions
Define complex cardinality constraints:

```turtle
:ManufacturingProcess a owl:Class ;
    rdfs:label "ManufacturingProcess" ;
    rdfs:subClassOf [
        a owl:Restriction ;
        owl:onProperty :requiresIngredient ;
        owl:qualifiedCardinality "2"^^xsd:nonNegativeInteger ;
        owl:onClass :MilkIngredient
    ] ;
    rdfs:subClassOf [
        a owl:Restriction ;
        owl:onProperty :requiresCertificate ;
        owl:minQualifiedCardinality "1"^^xsd:nonNegativeInteger ;
        owl:onClass :SafetyCertificate
    ] .
```

## Testing

The system includes comprehensive tests for all features:

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test owl2_generic_traceability_tests
cargo test enhanced_owl2_tests
```

## Best Practices

1. **Use Domain Plugins**: Always use domain plugins for domain-specific validation
2. **Leverage OWL2**: Use owl:hasKey for uniqueness constraints and property chains for transitive relationships
3. **Configure Properly**: Set up domain configurations in YAML or OWL files for easy deployment
4. **Test Thoroughly**: Write tests for domain-specific validation and processing logic
5. **Follow Patterns**: Use the established patterns for entity creation and domain extension