# Unified OWL2 + Generic Traceability Implementation Plan

## Overview

This document outlines the unified implementation plan for combining OWL2 features with generic traceability in the ProvChainOrg platform. The integration will create a powerful, flexible, and semantically rich traceability system.

## Integration Architecture

### Core Components
1. **Generic Traceability Core** - Domain-agnostic foundation
2. **OWL2 Reasoning Engine** - Advanced semantic reasoning capabilities
3. **Domain Extension System** - Pluggable domain-specific implementations
4. **Configuration Manager** - Runtime configuration and ontology loading
5. **Cross-Domain Bridge** - Multi-domain relationship management

### Integration Points
1. **Ontology Layer** - Generic core + domain extensions + OWL2 reasoning
2. **Blockchain Layer** - Generic entities + domain context + semantic validation
3. **API Layer** - Unified interface with domain switching + OWL2 features
4. **Storage Layer** - oxigraph integration with inferred relationships
5. **Validation Layer** - Cross-domain OWL2 validation

## Implementation Phases

### Phase 1: Foundation Integration (Weeks 1-2)

#### Task 1.1: Unified Ontology Structure
- [ ] Refactor `core.owl` to generic concepts with OWL2 features
- [ ] Create domain extension pattern using OWL2 import mechanisms
- [ ] Implement OWL2 feature placeholders in generic core
- [ ] Add domain switching context to ontology management

#### Task 1.2: Cross-Domain Reasoning Framework
- [ ] Implement cross-domain OWL2 reasoning context
- [ ] Create domain-aware reasoner instantiation
- [ ] Add domain context to OWL2 feature processing
- [ ] Implement cross-domain property chain inference

### Phase 2: OWL2 Feature Integration (Weeks 3-4)

#### Task 2.1: Enhanced OWL Reasoner
- [ ] Complete `owl:hasKey` implementation with domain awareness
- [ ] Implement property chain inference with cross-domain support
- [ ] Add qualified cardinality validation with domain context
- [ ] Integrate with oxigraph for inferred relationship storage

#### Task 2.2: Semantic Validation
- [ ] Implement cross-domain entity uniqueness validation
- [ ] Add property chain inference for multi-domain relationships
- [ ] Create qualified cardinality validation for complex domains
- [ ] Integrate validation results with blockchain operations

### Phase 3: Blockchain Integration (Weeks 5-6)

#### Task 3.1: Generic Entity Support
- [ ] Update blockchain to support generic entities with domain context
- [ ] Add OWL2 validation to block creation process
- [ ] Implement domain-aware canonicalization
- [ ] Add cross-domain relationship tracking

#### Task 3.2: Performance Optimization
- [ ] Optimize cross-domain OWL2 reasoning
- [ ] Implement caching for frequently used inferences
- [ ] Add lazy evaluation for complex reasoning tasks
- [ ] Optimize storage of inferred relationships

### Phase 4: API and CLI Integration (Weeks 7-8)

#### Task 4.1: Unified API
- [ ] Add domain switching to REST API
- [ ] Implement OWL2 feature endpoints
- [ ] Create cross-domain query interface
- [ ] Add semantic validation to API operations

#### Task 4.2: Enhanced CLI
- [ ] Add domain switching commands
- [ ] Implement OWL2 feature commands
- [ ] Create cross-domain query commands
- [ ] Add ontology management commands

## Detailed Implementation Tasks

### Task 1.1: Unified Ontology Structure

#### Subtask 1.1.1: Generic Core Ontology with OWL2 Features
```turtle
# ontologies/core.owl (refactored)
@prefix : <http://provchain.org/trace#> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

:TracedEntity a owl:Class ;
    rdfs:label "TracedEntity" ;
    rdfs:subClassOf prov:Entity ;
    rdfs:comment "Abstract entity that can be traced through a process chain" .

:TracedActivity a owl:Class ;
    rdfs:label "TracedActivity" ;
    rdfs:subClassOf prov:Activity ;
    rdfs:comment "Abstract activity that transforms or moves traced entities" .

:TracedAgent a owl:Class ;
    rdfs:label "TracedAgent" ;
    rdfs:subClassOf prov:Agent ;
    rdfs:comment "Abstract actor participating in traceability processes" .

# Generic properties with OWL2 features
:hasUniqueId a owl:DatatypeProperty ;
    rdfs:domain :TracedEntity ;
    rdfs:range xsd:string ;
    rdfs:comment "Unique identifier for the traced entity" .

:hasTimestamp a owl:DatatypeProperty ;
    rdfs:domain :TracedActivity ;
    rdfs:range xsd:dateTime ;
    rdfs:comment "Timestamp when the activity occurred" .

# OWL2 feature placeholders in generic core
# These will be specialized in domain extensions
```

#### Subtask 1.1.2: Domain Extension Pattern
```turtle
# ontologies/supplychain.owl (domain extension example)
@prefix : <http://provchain.org/supplychain#> .
@prefix trace: <http://provchain.org/trace#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

# Import generic core ontology
owl:imports <http://provchain.org/trace> .

# Specialized classes
:ProductBatch a owl:Class ;
    rdfs:subClassOf trace:TracedEntity ;
    # Domain-specific hasKey constraint
    owl:hasKey ( :batchId ) ;
    rdfs:label "ProductBatch" ;
    rdfs:comment "A batch or lot of product in the supply chain" .

# Domain-specific properties
:batchId a owl:DatatypeProperty ;
    rdfs:subClassOf trace:hasUniqueId ;
    rdfs:domain :ProductBatch ;
    rdfs:range xsd:string ;
    rdfs:comment "Unique batch identifier" .

# Domain-specific property chains
:suppliedTo a owl:ObjectProperty ;
    rdfs:domain :ProductBatch ;
    rdfs:range :Supplier ;
    rdfs:comment "Direct supply relationship" .

:transitivelySuppliedTo a owl:ObjectProperty ;
    rdfs:domain :ProductBatch ;
    rdfs:range :Supplier ;
    rdfs:comment "Transitive supply relationship" ;
    owl:propertyChainAxiom ( :suppliedTo :suppliedTo ) .
```

### Task 2.1: Enhanced OWL Reasoner

#### Subtask 2.1.1: Domain-Aware hasKey Implementation
```rust
// src/semantic/owl_reasoner.rs
impl OwlReasoner {
    /// Process owl:hasKey axioms with domain awareness
    pub fn process_has_key_axioms(&mut self, domain_context: &str) -> Result<()> {
        info!("Processing owl:hasKey axioms for domain: {}", domain_context);
        
        // Query for domain-specific owl:hasKey axioms
        let query = format!(r#"
            PREFIX owl: <http://www.w3.org/2002/07/owl#>
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
            
            SELECT ?class ?keyList WHERE {{
                GRAPH <http://provchain.org/domain/{}> {{
                    ?class owl:hasKey ?keyList .
                }}
            }}
        "#, domain_context);
        
        match self.ontology_store.query(&query) {
            Ok(QueryResults::Solutions(solutions)) => {
                for solution in solutions {
                    if let Ok(sol) = solution {
                        if let (Some(class_term), Some(key_list_term)) = (
                            sol.get("class"),
                            sol.get("keyList")
                        ) {
                            let class_iri = class_term.to_string();
                            let key_list_iri = key_list_term.to_string();
                            
                            // Extract all properties in the key list
                            let properties = self.extract_key_properties(&key_list_iri)?;
                            
                            // Add to has_key_constraints with domain context
                            let domain_class_iri = format!("{}#{}", domain_context, class_iri);
                            self.has_key_constraints
                                .entry(domain_class_iri)
                                .or_insert_with(Vec::new)
                                .extend(properties);
                        }
                    }
                }
            }
            Ok(_) => {
                debug!("No owl:hasKey axioms found for domain: {}", domain_context);
            }
            Err(e) => {
                warn!("Failed to query for owl:hasKey axioms in domain {}: {}", domain_context, e);
            }
        }
        
        info!("Processed {} owl:hasKey axioms for domain {}", 
              self.has_key_constraints.len(), domain_context);
        Ok(())
    }
}
```

#### Subtask 2.1.2: Cross-Domain Property Chain Inference
```rust
// src/semantic/owl_reasoner.rs
impl OwlReasoner {
    /// Apply property chain inference with cross-domain support
    pub fn apply_property_chain_inference(&mut self, graph_data: &str, domain_contexts: &[String]) -> Result<InferredGraph> {
        info!("Applying property chain inference for domains: {:?}", domain_contexts);
        
        let mut combined_inferred_graph = InferredGraph::new();
        
        // Apply inference for each domain context
        for domain_context in domain_contexts {
            // Process property chains in this domain
            for (super_property, chain_properties) in &self.property_chains {
                // Check if this property chain belongs to current domain
                if super_property.starts_with(&format!("http://provchain.org/{}/", domain_context)) {
                    // Apply chain inference using SPARQL queries
                    let query = self.generate_chain_inference_query(super_property, chain_properties)?;
                    let inferred_triples = self.execute_chain_inference_query(&query)?;
                    
                    // Add inferred triples to the combined graph
                    combined_inferred_graph.add_triples(inferred_triples);
                }
            }
        }
        
        // Store inferred relationships in oxigraph for efficient querying
        self.store_inferred_axioms(&combined_inferred_graph)?;
        
        Ok(combined_inferred_graph)
    }
}
```

### Task 3.1: Generic Entity Support

#### Subtask 3.1.1: Domain-Aware Blockchain Entities
```rust
// src/core/blockchain.rs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: String,
    pub data: TracedEntityData, // Generic traceable entity data
    pub previous_hash: String,
    pub hash: String,
    pub state_root: String,
    pub domain_context: String, // Domain context for this block
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TracedEntityData {
    pub entity_id: String,
    pub entity_type: String,
    pub domain: String, // Domain this entity belongs to
    pub properties: HashMap<String, String>,
    pub relationships: Vec<Relationship>,
    pub domain_specific_data: DomainSpecificData,
}

impl Block {
    pub fn new(index: u64, data: TracedEntityData, previous_hash: String, state_root: String) -> Self {
        let timestamp = Utc::now().to_rfc3339();
        let mut block = Block {
            index,
            timestamp,
            data,
            previous_hash,
            hash: String::new(),
            state_root,
            domain_context: String::new(), // Will be set during creation
        };
        block.hash = block.calculate_hash();
        block
    }
}
```

#### Subtask 3.1.2: OWL2 Validation in Block Creation
```rust
// src/core/blockchain.rs
impl Blockchain {
    pub fn add_block(&mut self, data: TracedEntityData) -> Result<()> {
        // Ensure we have at least a genesis block
        if self.chain.is_empty() {
            let genesis_block = self.create_genesis_block();
            let graph_name = NamedNode::new("http://provchain.org/block/0").unwrap();
            self.rdf_store.add_rdf_to_graph(&genesis_block.data, &graph_name);
            self.rdf_store.add_block_metadata(&genesis_block);
            self.chain.push(genesis_block);
        }

        let prev_block = self.chain.last().unwrap();
        // Calculate the state root before creating the new block
        let state_root = self.rdf_store.calculate_state_root();
        
        // Set domain context for the block
        let domain_context = data.domain.clone();
        
        let mut new_block = Block::new(
            prev_block.index + 1, 
            data.clone(), 
            prev_block.hash.clone(), 
            state_root
        );
        new_block.domain_context = domain_context.clone();

        // Use atomic operations to ensure consistency
        let mut atomic_context = AtomicOperationContext::new(&mut self.rdf_store);
        
        // Validate entity using OWL2 features with domain context
        if let Err(validation_error) = self.validate_entity_with_owl2(&data, &domain_context) {
            warn!("Entity validation failed: {}", validation_error);
            return Err(validation_error);
        }
        
        // Add RDF data and block metadata atomically
        atomic_context.add_block_atomically(&new_block)?;
        
        // Recalculate hash using RDF canonicalization after successful atomic operation
        let graph_name = NamedNode::new(format!("http://provchain.org/block/{}", new_block.index)).unwrap();
        let canonical_hash = if let Some(cached_hash) = self.canonicalization_cache.get(&format!("block_{}", new_block.index)) {
            cached_hash.clone()
        } else {
            let hash = self.rdf_store.canonicalize_graph(&graph_name);
            self.canonicalization_cache.insert(format!("block_{}", new_block.index), hash.clone());
            hash
        };
        
        new_block.hash = new_block.calculate_hash_with_store_and_canonical_hash(Some(&self.rdf_store), &canonical_hash);
        
        // Update the block metadata with the new hash
        self.rdf_store.add_block_metadata(&new_block);

        self.chain.push(new_block);
        
        // Batch disk writes for better performance
        if !self.disable_disk_persistence {
            self.disk_write_counter += 1;
            if self.disk_write_counter >= self.disk_write_batch_size {
                self.rdf_store.save_to_disk()?;
                self.disk_write_counter = 0;
            }
        }
        
        Ok(())
    }
    
    /// Validate entity using OWL2 features with domain context
    fn validate_entity_with_owl2(&self, entity_data: &TracedEntityData, domain_context: &str) -> Result<()> {
        // Get domain-specific OWL reasoner
        if let Some(reasoner) = self.domain_reasoners.get(domain_context) {
            // Validate entity uniqueness based on hasKey constraints
            let uniqueness_result = reasoner.validate_entity_uniqueness(&entity_data.entity_id)?;
            if !matches!(uniqueness_result, ValidationResult::Valid) {
                return Err(anyhow::anyhow!("Entity uniqueness validation failed: {:?}", uniqueness_result));
            }
            
            // Validate qualified cardinality constraints
            let cardinality_result = reasoner.validate_qualified_cardinality(&entity_data.entity_id)?;
            if !matches!(cardinality_result, ValidationResult::Valid) {
                return Err(anyhow::anyhow!("Qualified cardinality validation failed: {:?}", cardinality_result));
            }
        }
        
        Ok(())
    }
}
```

### Task 4.1: Unified API

#### Subtask 4.1.1: Domain Switching API Endpoints
```rust
// src/web/handlers.rs
/// Switch to a different domain context
pub async fn switch_domain_handler(
    Json(payload): Json<SwitchDomainRequest>,
) -> Result<Json<SwitchDomainResponse>, AppError> {
    let blockchain = get_blockchain_instance();
    
    // Switch domain context
    blockchain.switch_domain(&payload.domain)?;
    
    Ok(Json(SwitchDomainResponse {
        status: "success".to_string(),
        message: format!("Switched to domain: {}", payload.domain),
        domain: payload.domain,
    }))
}

/// Get current domain context
pub async fn get_domain_handler() -> Result<Json<GetDomainResponse>, AppError> {
    let blockchain = get_blockchain_instance();
    
    let current_domain = blockchain.get_current_domain()?;
    
    Ok(Json(GetDomainResponse {
        domain: current_domain,
    }))
}

/// Add entity with domain context
pub async fn add_entity_handler(
    Json(payload): Json<AddEntityRequest>,
) -> Result<Json<AddEntityResponse>, AppError> {
    let blockchain = get_blockchain_instance();
    
    // Create traced entity data with domain context
    let entity_data = TracedEntityData {
        entity_id: payload.entity_id,
        entity_type: payload.entity_type,
        domain: payload.domain.unwrap_or_else(|| blockchain.get_current_domain().unwrap_or("default".to_string())),
        properties: payload.properties,
        relationships: payload.relationships,
        domain_specific_data: payload.domain_specific_data,
    };
    
    // Add entity to blockchain
    let block_index = blockchain.add_entity(entity_data)?;
    
    Ok(Json(AddEntityResponse {
        status: "success".to_string(),
        block_index,
        message: format!("Entity added to block {}", block_index),
    }))
}
```

#### Subtask 4.1.2: OWL2 Feature API Endpoints
```rust
// src/web/handlers.rs
/// Validate entity using OWL2 features
pub async fn validate_entity_handler(
    Json(payload): Json<ValidateEntityRequest>,
) -> Result<Json<ValidateEntityResponse>, AppError> {
    let blockchain = get_blockchain_instance();
    
    // Validate entity using OWL2 features with domain context
    let validation_result = blockchain.validate_entity_with_owl2(
        &payload.entity_id,
        &payload.domain.unwrap_or_else(|| blockchain.get_current_domain().unwrap_or("default".to_string()))
    )?;
    
    Ok(Json(ValidateEntityResponse {
        valid: matches!(validation_result, ValidationResult::Valid),
        errors: match validation_result {
            ValidationResult::Invalid(msg) => vec![msg],
            ValidationResult::Warning(msg) => vec![msg],
            ValidationResult::Valid => vec![],
        },
        warnings: match validation_result {
            ValidationResult::Warning(msg) => vec![msg],
            _ => vec![],
        },
    }))
}

/// Apply property chain inference
pub async fn apply_property_chain_inference_handler(
    Json(payload): Json<ApplyInferenceRequest>,
) -> Result<Json<ApplyInferenceResponse>, AppError> {
    let blockchain = get_blockchain_instance();
    
    // Apply property chain inference with domain context
    let inferred_graph = blockchain.apply_property_chain_inference(
        &payload.graph_data,
        &payload.domains
    )?;
    
    Ok(Json(ApplyInferenceResponse {
        status: "success".to_string(),
        inferred_triples: inferred_graph.triples().clone(),
        message: format!("Inferred {} new relationships", inferred_graph.triples().len()),
    }))
}
```

## Integration Testing Plan

### Phase 1: Unit Testing
1. **OWL2 Feature Unit Tests**
   - Test `owl:hasKey` constraint validation
   - Test property chain inference
   - Test qualified cardinality validation
   - Test cross-domain OWL2 reasoning

2. **Generic Traceability Unit Tests**
   - Test domain switching functionality
   - Test generic entity validation
   - Test cross-domain relationship handling
   - Test domain extension pattern

### Phase 2: Integration Testing
1. **Combined Feature Integration Tests**
   - Test OWL2 features with generic traceability
   - Test cross-domain OWL2 reasoning
   - Test domain-aware entity validation
   - Test unified API endpoints

2. **Performance Integration Tests**
   - Test cross-domain query performance
   - Test OWL2 reasoning performance
   - Test domain switching overhead
   - Test inferred relationship storage

### Phase 3: System Testing
1. **End-to-End Testing**
   - Test complete workflow with domain switching
   - Test cross-domain traceability scenarios
   - Test OWL2 feature validation in real scenarios
   - Test unified API with multiple domains

2. **Regression Testing**
   - Verify all existing functionality still works
   - Test backward compatibility
   - Validate performance optimizations
   - Check documentation accuracy

## Success Metrics

### Functional Success Metrics
- [ ] Generic traceability system supports any domain
- [ ] OWL2 features (`owl:hasKey`, property chains, qualified cardinality) fully implemented
- [ ] Configuration-driven ontology loading works correctly
- [ ] Domain switching operates seamlessly
- [ ] Cross-domain queries return accurate results

### Performance Success Metrics
- [ ] Property chain inference under 100ms for typical supply chains
- [ ] Uniqueness constraint validation under 50ms
- [ ] Domain switching operations under 1 second
- [ ] Cross-domain queries maintain acceptable performance

### Quality Success Metrics
- [ ] All existing functionality preserved
- [ ] Backward compatibility maintained
- [ ] Comprehensive test coverage (>85%)
- [ ] Documentation complete and accurate

## Risk Mitigation

### Technical Risks
1. **Complexity Management**
   - Solution: Modular implementation with clear interfaces
   - Solution: Incremental development and testing

2. **Performance Overhead**
   - Solution: Caching strategies and lazy evaluation
   - Solution: Efficient domain context switching

3. **Integration Challenges**
   - Solution: Well-defined APIs and extensive testing
   - Solution: Backward compatibility layers

### Operational Risks
1. **Deployment Complexity**
   - Solution: Comprehensive documentation and examples
   - Solution: Gradual migration path

2. **Migration Effort**
   - Solution: Backward compatibility and migration tools
   - Solution: Clear upgrade procedures

3. **Learning Curve**
   - Solution: Training materials and intuitive interfaces
   - Solution: Comprehensive examples and tutorials

## Expected Benefits

### Business Benefits
- Universal traceability platform supporting multiple industries
- Enhanced data quality through advanced OWL2 reasoning
- Reduced development time for new domain implementations
- Competitive advantage through advanced semantic capabilities

### Technical Benefits
- Decoupled architecture enabling independent evolution
- Configuration-driven flexibility for diverse use cases
- Performance-optimized reasoning for complex ontologies
- Extensible design supporting future enhancements

### Community Benefits
- Open platform for domain-specific extensions
- Standardized approach to traceability across domains
- Shared core infrastructure reducing development costs
- Collaborative ecosystem for domain ontologies

This unified implementation plan provides a roadmap for integrating OWL2 features with generic traceability in the ProvChainOrg platform, creating a powerful and flexible semantic blockchain system.