# OWL2 Feature Implementation Plan

## Current State Analysis

### Existing OWL Reasoner
The current `src/semantic/owl_reasoner.rs` is minimal with only:
- Basic struct definitions (`OwlReasonerConfig`, `OwlReasoner`)
- Skeleton implementation with TODO comments
- Imports for horned-owl but no actual implementation

### Current Ontology
The `ontologies/core.owl` contains supply-chain specific concepts but no OWL2 advanced features like:
- `owl:hasKey` axioms
- `owl:propertyChainAxiom` declarations
- Qualified cardinality restrictions

## Implementation Tasks

### Task 1: Enhance OWL Reasoner for OWL2 Features

#### Subtask 1.1: Implement owl:hasKey Support
```rust
// In src/semantic/owl_reasoner.rs

impl OwlReasoner {
    /// Process owl:hasKey axioms from the ontology
    pub fn process_has_key_axioms(&mut self) -> Result<()> {
        if let Some(ref ontology) = self.ontology {
            // Extract owl:hasKey axioms
            for axiom in ontology.axioms() {
                if let Axiom::HasKey(has_key) = axiom {
                    // Process the hasKey axiom
                    let class_iri = has_key.class.as_ref().iri().unwrap().to_string();
                    let mut key_properties = Vec::new();
                    
                    for property in &has_key.object_property_expressions {
                        if let ObjectPropertyExpression::ObjectProperty(prop) = property {
                            key_properties.push(prop.as_ref().iri().unwrap().to_string());
                        }
                    }
                    
                    for property in &has_key.data_property_expressions {
                        if let DataPropertyExpression::DataProperty(prop) = property {
                            key_properties.push(prop.as_ref().iri().unwrap().to_string());
                        }
                    }
                    
                    // Store for later validation
                    self.has_key_constraints.insert(class_iri, key_properties);
                }
            }
        }
        Ok(())
    }
    
    /// Validate entity uniqueness based on hasKey constraints
    pub fn validate_entity_uniqueness(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        // For each class the entity belongs to
        for class in &entity_data.classes {
            // Check if there are hasKey constraints for this class
            if let Some(key_properties) = self.has_key_constraints.get(class) {
                // Extract key property values from entity data
                let mut key_values = Vec::new();
                for prop in key_properties {
                    if let Some(value) = entity_data.properties.get(prop) {
                        key_values.push(value.clone());
                    } else {
                        // Missing required key property
                        return Ok(ValidationResult::Invalid(format!(
                            "Missing required key property {} for class {}", 
                            prop, class
                        )));
                    }
                }
                
                // Check if another entity with same key values exists
                // This would involve querying the RDF store
                // Implementation depends on oxigraph integration
            }
        }
        Ok(ValidationResult::Valid)
    }
}
```

#### Subtask 1.2: Implement Property Chain Axiom Support
```rust
// In src/semantic/owl_reasoner.rs

impl OwlReasoner {
    /// Process owl:propertyChainAxiom from the ontology
    pub fn process_property_chain_axioms(&mut self) -> Result<()> {
        if let Some(ref ontology) = self.ontology {
            // Extract property chain axioms
            for axiom in ontology.axioms() {
                if let Axiom::SubObjectPropertyOf(sub_prop) = axiom {
                    if let ObjectPropertyExpression::ObjectPropertyChain(chain) = &sub_prop.sub_object_property_expression {
                        // This is a property chain axiom
                        let super_property = match &sub_prop.object_property_expression {
                            ObjectPropertyExpression::ObjectProperty(prop) => {
                                prop.as_ref().iri().unwrap().to_string()
                            }
                            _ => continue, // Skip if not a simple property
                        };
                        
                        let mut chain_properties = Vec::new();
                        for prop_expr in chain {
                            if let ObjectPropertyExpression::ObjectProperty(prop) = prop_expr {
                                chain_properties.push(prop.as_ref().iri().unwrap().to_string());
                            }
                        }
                        
                        // Store for inference
                        self.property_chains.insert(super_property, chain_properties);
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Apply property chain inference to generate new relationships
    pub fn apply_property_chain_inference(&self, graph: &Graph) -> Result<InferredGraph> {
        let mut inferred_graph = InferredGraph::new();
        
        // For each property chain axiom
        for (super_property, chain_properties) in &self.property_chains {
            // Apply transitive closure for this chain
            // This is a simplified implementation - real implementation would be more complex
            if chain_properties.len() == 2 {
                let prop1 = &chain_properties[0];
                let prop2 = &chain_properties[1];
                
                // Find all triples with prop1
                for triple1 in graph.triples_with_predicate(prop1) {
                    let subject = triple1.subject();
                    let intermediate = triple1.object();
                    
                    // Find all triples with prop2 where subject is the intermediate
                    for triple2 in graph.triples_with_subject_and_predicate(intermediate, prop2) {
                        let object = triple2.object();
                        
                        // Infer new triple: subject -> super_property -> object
                        inferred_graph.add_triple(Triple::new(
                            subject.clone(),
                            NamedNode::new(super_property)?,
                            object.clone()
                        ));
                    }
                }
            }
        }
        
        Ok(inferred_graph)
    }
}
```

#### Subtask 1.3: Implement Qualified Cardinality Support
```rust
// In src/semantic/owl_reasoner.rs

impl OwlReasoner {
    /// Process qualified cardinality restrictions
    pub fn process_qualified_cardinality_restrictions(&mut self) -> Result<()> {
        if let Some(ref ontology) = self.ontology {
            // Extract qualified cardinality restrictions
            for axiom in ontology.axioms() {
                if let Axiom::SubClassOf(sub_class) = axiom {
                    if let ClassExpression::ObjectSomeValuesFrom(some_values) = &sub_class.sub_class_expression {
                        // Check if this is a qualified cardinality restriction
                        // This would need to parse the complex class expressions
                        // Implementation would depend on the exact OWL2 syntax used
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Validate qualified cardinality constraints
    pub fn validate_qualified_cardinality(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        // Validate that entities satisfy qualified cardinality restrictions
        // This would involve counting relationships of specific types to specific classes
        // Implementation depends on the exact requirements
        Ok(ValidationResult::Valid)
    }
}
```

### Task 2: Update Data Structures

#### Subtask 2.1: Extend OwlReasoner Struct
```rust
// In src/semantic/owl_reasoner.rs

pub struct OwlReasoner {
    /// Configuration
    pub config: OwlReasonerConfig,
    /// Loaded ontology
    ontology: Option<IRIMappedOntology>,
    /// Inferred axioms
    inferred_axioms: HashSet<String>,
    /// HasKey constraints: class IRI -> property IRIs
    has_key_constraints: HashMap<String, Vec<String>>,
    /// Property chains: super property -> chain properties
    property_chains: HashMap<String, Vec<String>>,
    /// Qualified cardinality restrictions
    qualified_cardinality_restrictions: Vec<QualifiedCardinalityRestriction>,
    /// Reference to RDF store for validation
    rdf_store: Option<RDFStore>,
}

/// Qualified cardinality restriction definition
#[derive(Debug, Clone)]
pub struct QualifiedCardinalityRestriction {
    pub class: String,
    pub property: String,
    pub cardinality: u32,
    pub filler_class: String,
}

/// Validation result enum
#[derive(Debug, Clone)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
    Warning(String),
}
```

### Task 3: Integrate with Oxigraph

#### Subtask 3.1: Store Inferred Relationships
```rust
// In src/semantic/owl_reasoner.rs

impl OwlReasoner {
    /// Store inferred axioms in oxigraph for efficient querying
    pub fn store_inferred_axioms(&mut self, inferred: &InferredGraph) -> Result<()> {
        if let Some(ref mut store) = self.rdf_store {
            // Convert inferred triples to oxigraph format and store
            for triple in inferred.triples() {
                let subject = self.convert_to_oxigraph_subject(triple.subject())?;
                let predicate = self.convert_to_oxigraph_named_node(triple.predicate())?;
                let object = self.convert_to_oxigraph_term(triple.object())?;
                
                let quad = Quad::new(subject, predicate, object, NamedNode::new("http://provchain.org/inferred")?);
                store.store.insert(&quad)?;
            }
            
            // Update cache if it exists
            if let Some(ref mut cache) = store.memory_cache {
                cache.insert("inferred_axioms".to_string(), inferred.triples().cloned().collect());
            }
        }
        Ok(())
    }
    
    /// Query inferred relationships
    pub fn query_inferred_relationships(&self, query: &str) -> Result<QueryResults> {
        if let Some(ref store) = self.rdf_store {
            // Execute SPARQL query including both asserted and inferred data
            store.store.query(query)
        } else {
            Err(anyhow::anyhow!("No RDF store available for querying"))
        }
    }
}
```

### Task 4: Update Core Ontology with OWL2 Features

#### Subtask 4.1: Add owl:hasKey Axioms to core.owl
```turtle
# Add to ontologies/core.owl

# Ensure ProductBatch has unique batch IDs
:ProductBatch a owl:Class ;
    rdfs:subClassOf :TraceEntity ;
    owl:hasKey ( :hasBatchID ) ;
    rdfs:comment "A batch or lot of product with unique batch ID" .

# Ensure IngredientLot has unique lot IDs
:IngredientLot a owl:Class ;
    rdfs:subClassOf :TraceEntity ;
    owl:hasKey ( :hasLotID ) ;
    rdfs:comment "A lot of raw material or ingredient with unique lot ID" .

# Ensure ProcessingActivity has unique activity IDs
:ProcessingActivity a owl:Class ;
    rdfs:subClassOf :TraceActivity ;
    owl:hasKey ( :hasActivityID ) ;
    rdfs:comment "Manufacturing or processing step with unique activity ID" .
```

#### Subtask 4.2: Add Property Chain Axioms
```turtle
# Add to ontologies/core.owl

# If A is derived from B, and B is derived from C, then A is transitively derived from C
:derivedFrom a owl:ObjectProperty ;
    rdfs:domain :TraceEntity ;
    rdfs:range :TraceEntity .

:derivedFrom owl:propertyChainAxiom ( :derivedFrom :derivedFrom ) .

# If A supplies to B, and B supplies to C, then A transitively supplies to C
:suppliesTo a owl:ObjectProperty ;
    rdfs:domain :TraceAgent ;
    rdfs:range :TraceAgent .

:suppliesTo owl:propertyChainAxiom ( :suppliesTo :suppliesTo ) .
```

#### Subtask 4.3: Add Qualified Cardinality Restrictions
```turtle
# Add to ontologies/core.owl

# A ProcessingActivity must have exactly 1 origin facility
:ProcessingActivity rdfs:subClassOf [
    a owl:Restriction ;
    owl:onProperty :originFacility ;
    owl:qualifiedCardinality "1"^^xsd:nonNegativeInteger ;
    owl:onClass :Facility
] .

# A ProductBatch must have at least 1 ingredient lot
:ProductBatch rdfs:subClassOf [
    a owl:Restriction ;
    owl:onProperty :hasIngredient ;
    owl:minQualifiedCardinality "1"^^xsd:nonNegativeInteger ;
    owl:onClass :IngredientLot
] .
```

### Task 5: Integration with Blockchain

#### Subtask 5.1: Update Block Validation
```rust
// In src/core/blockchain.rs

impl Blockchain {
    /// Add block with OWL2 validation
    pub fn add_block_with_owl2_validation(&mut self, data: String) -> Result<()> {
        // Parse RDF data to extract entities
        let entities = self.extract_entities_from_rdf(&data)?;
        
        // Validate using OWL2 reasoner
        for entity in &entities {
            // Validate uniqueness constraints
            let uniqueness_result = self.rdf_store.owl_reasoner.validate_entity_uniqueness(entity)?;
            if !matches!(uniqueness_result, ValidationResult::Valid) {
                return Err(anyhow::anyhow!("Entity uniqueness validation failed: {:?}", uniqueness_result));
            }
            
            // Validate qualified cardinality
            let cardinality_result = self.rdf_store.owl_reasoner.validate_qualified_cardinality(entity)?;
            if !matches!(cardinality_result, ValidationResult::Valid) {
                return Err(anyhow::anyhow!("Qualified cardinality validation failed: {:?}", cardinality_result));
            }
        }
        
        // Apply property chain inference
        let inferred_graph = self.rdf_store.owl_reasoner.apply_property_chain_inference(&self.rdf_store.store)?;
        
        // Store inferred relationships
        self.rdf_store.owl_reasoner.store_inferred_axioms(&inferred_graph)?;
        
        // Proceed with normal block addition
        self.add_block(data)
    }
}
```

### Task 6: Testing

#### Subtask 6.1: Unit Tests for OWL2 Features
```rust
// In tests/owl2_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blockchain::Blockchain;
    
    #[test]
    fn test_has_key_validation() -> Result<()> {
        let mut blockchain = Blockchain::new();
        
        // Add an entity with a unique key
        let entity_data = r#"
        @prefix : <http://provchain.org/trace#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        
        :Batch001 a :ProductBatch ;
            :hasBatchID "BATCH-001" .
        "#;
        
        let result = blockchain.add_block_with_owl2_validation(entity_data.to_string());
        assert!(result.is_ok());
        
        // Try to add another entity with the same key (should fail)
        let duplicate_data = r#"
        @prefix : <http://provchain.org/trace#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        
        :Batch002 a :ProductBatch ;
            :hasBatchID "BATCH-001" .
        "#;
        
        let result = blockchain.add_block_with_owl2_validation(duplicate_data.to_string());
        assert!(result.is_err());
        
        Ok(())
    }
    
    #[test]
    fn test_property_chain_inference() -> Result<()> {
        let mut blockchain = Blockchain::new();
        
        // Add entities with a chain relationship
        let chain_data = r#"
        @prefix : <http://provchain.org/trace#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        
        :Batch001 a :ProductBatch ;
            :derivedFrom :Batch002 .
            
        :Batch002 a :ProductBatch ;
            :derivedFrom :Batch003 .
        "#;
        
        let result = blockchain.add_block_with_owl2_validation(chain_data.to_string());
        assert!(result.is_ok());
        
        // Query for transitive derivation (should find Batch001 derived from Batch003)
        let query = r#"
        PREFIX : <http://provchain.org/trace#>
        SELECT ?derived ?ancestor WHERE {
            ?derived :derivedFrom+ ?ancestor .
        }
        "#;
        
        let results = blockchain.rdf_store.query(query);
        // Should find the inferred relationship
        
        Ok(())
    }
    
    #[test]
    fn test_qualified_cardinality_validation() -> Result<()> {
        let mut blockchain = Blockchain::new();
        
        // Add a processing activity with required facility
        let valid_data = r#"
        @prefix : <http://provchain.org/trace#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        
        :Activity001 a :ProcessingActivity ;
            :hasActivityID "ACT-001" ;
            :originFacility :Facility001 .
            
        :Facility001 a :Facility .
        "#;
        
        let result = blockchain.add_block_with_owl2_validation(valid_data.to_string());
        assert!(result.is_ok());
        
        Ok(())
    }
}
```

## Implementation Priority

### Phase 1: Core Infrastructure (Week 1)
1. Extend `OwlReasoner` struct with OWL2 data structures
2. Implement basic parsing of OWL2 axioms
3. Update `core.owl` with basic OWL2 features

### Phase 2: Feature Implementation (Week 2-3)
1. Implement `owl:hasKey` support with validation
2. Implement property chain axiom processing and inference
3. Implement qualified cardinality restriction processing

### Phase 3: Integration (Week 4)
1. Integrate with oxigraph for inferred relationship storage
2. Update blockchain to use OWL2 validation
3. Add comprehensive unit tests

### Phase 4: Testing and Optimization (Week 5)
1. Performance testing and optimization
2. Comprehensive integration testing
3. Documentation updates

## Success Criteria

1. ✅ `owl:hasKey` axioms properly parsed and validated
2. ✅ Property chain axioms processed and inferred relationships generated
3. ✅ Qualified cardinality restrictions validated
4. ✅ Inferred relationships stored in oxigraph for efficient querying
5. ✅ Blockchain validates entities using OWL2 constraints
6. ✅ All tests pass with comprehensive coverage
7. ✅ Performance remains acceptable (< 100ms for typical operations)

This implementation plan provides a structured approach to adding OWL2 features to ProvChainOrg while maintaining compatibility with existing functionality.