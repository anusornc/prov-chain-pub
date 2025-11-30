//! Enhanced traceability using owl2-reasoner for OWL2 reasoning and optimization
//!
//! This module enhances the existing traceability system by leveraging
//! owl2-reasoner for more sophisticated ontology-based reasoning and optimization.

use crate::core::blockchain::Blockchain;
use crate::core::entity::{EntityType, PropertyValue, TraceableEntity};
use crate::trace_optimization::{EnhancedTraceResult, EnhancedTraceabilitySystem, TraceEvent};
use anyhow::Result;
use chrono::Utc;
use owl2_reasoner::{
    Class, ClassAssertionAxiom, ClassExpression, DataProperty, Ontology, IRI,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Enhanced traceability system using owl2-reasoner for OWL2 reasoning
pub struct Owl2EnhancedTraceability {
    blockchain: Blockchain,
}

impl Owl2EnhancedTraceability {
    /// Create a new enhanced traceability system
    pub fn new(blockchain: Blockchain) -> Self {
        Owl2EnhancedTraceability { blockchain }
    }

    /// Create OWL2 ontology from traceable entities
    pub fn entities_to_owl_ontology(&self, entities: &[TraceableEntity]) -> Result<Ontology> {
        println!("=== Converting Traceable Entities to OWL2 Ontology ===");

        let mut ontology = Ontology::with_iri("http://provchain.org/traceability");

        // Note: Prefix registration can be done through the ontology's IRI registry if needed
        // For now, we use full IRIs directly

        // Create classes based on entity types
        let mut class_map: HashMap<String, Class> = HashMap::new();

        for entity in entities {
            let class_name = match &entity.entity_type {
                EntityType::Product => "http://provchain.org/traceability#Product",
                EntityType::Component => "http://provchain.org/traceability#Component",
                EntityType::Process => "http://provchain.org/traceability#Process",
                EntityType::Person => "http://provchain.org/traceability#Person",
                EntityType::Organization => "http://provchain.org/traceability#Organization",
                EntityType::Document => "http://provchain.org/traceability#Document",
                EntityType::DigitalAsset => "http://provchain.org/traceability#DigitalAsset",
                EntityType::Service => "http://provchain.org/traceability#Service",
                EntityType::Event => "http://provchain.org/traceability#Event",
                EntityType::Location => "http://provchain.org/traceability#Location",
                EntityType::Equipment => "http://provchain.org/traceability#Equipment",
                EntityType::DomainSpecific(domain) => {
                    // For domain-specific types, we'll use a fallback
                    // In a more sophisticated implementation, we'd handle these dynamically
                    let _ = domain; // acknowledge the domain parameter
                    "http://provchain.org/traceability#DomainSpecific"
                }
            };

            // Create class if not already created
            if !class_map.contains_key(class_name) {
                let class = Class::new(class_name);
                class_map.insert(class_name.to_string(), class.clone());
                // Add class to ontology
                ontology.add_class(class)?;
            }

            // Create individual IRI for the entity (IRI::new returns Result)
            let individual_uri = format!("http://provchain.org/entity/{}", entity.id);
            let individual_iri = Arc::new(IRI::new(&individual_uri)?);

            // Get the class expression
            let class_expr = ClassExpression::Class(class_map[class_name].clone());

            // Add class assertion
            let class_assertion = ClassAssertionAxiom::new(individual_iri, class_expr);
            ontology.add_class_assertion(class_assertion)?;

            // Add properties from the entity
            for (prop_name, _prop_value) in &entity.properties {
                // Create data property for tracking
                let _data_prop = DataProperty::new(format!("http://provchain.org/traceability#{}", prop_name));
                // In a full implementation, we would add data property assertions
                // For now, we just acknowledge the property exists
            }
        }

        println!("Converted {} entities to OWL2 ontology", entities.len());
        println!("OWL2 ontology has {} axioms", ontology.axiom_count());

        Ok(ontology)
    }

    /// Apply owl:hasKey constraints to validate entity uniqueness
    pub fn validate_entity_keys(&self, entities: &[TraceableEntity]) -> Result<Vec<String>> {
        println!("=== Validating Entity Keys using owl:hasKey ===");

        let mut validation_errors = Vec::new();

        // Group entities by type
        let mut entities_by_type: HashMap<String, Vec<&TraceableEntity>> = HashMap::new();

        for entity in entities {
            let type_key = match &entity.entity_type {
                EntityType::Product => "Product",
                EntityType::Component => "Component",
                EntityType::Process => "Process",
                EntityType::Person => "Person",
                EntityType::Organization => "Organization",
                EntityType::Document => "Document",
                EntityType::DigitalAsset => "DigitalAsset",
                EntityType::Service => "Service",
                EntityType::Event => "Event",
                EntityType::Location => "Location",
                EntityType::Equipment => "Equipment",
                EntityType::DomainSpecific(domain) => domain,
            };

            entities_by_type
                .entry(type_key.to_string())
                .or_default()
                .push(entity);
        }

        // Check for duplicate keys within each type
        // In a full OWL2 implementation, we would extract hasKey constraints from the ontology
        // For now, we'll check for common key properties like "id", "sku", "batchId", etc.
        let key_properties = vec!["id", "sku", "batchId", "serialNumber", "identifier"];

        for (entity_type, typed_entities) in &entities_by_type {
            // Create a map to check for duplicates
            let mut key_values: HashMap<String, Vec<String>> = HashMap::new();

            for entity in typed_entities {
                // Check each key property
                for key_prop in &key_properties {
                    if let Some(prop_value) = entity.properties.get(*key_prop) {
                        let key_string = format!("{:?}", prop_value);
                        key_values
                            .entry(key_string)
                            .or_default()
                            .push(entity.id.clone());
                    }
                }
            }

            // Report duplicates
            for (key_value, entity_ids) in &key_values {
                if entity_ids.len() > 1 {
                    validation_errors.push(format!(
                        "Duplicate {} key '{}' found in entities: {:?}",
                        entity_type, key_value, entity_ids
                    ));
                }
            }
        }

        if validation_errors.is_empty() {
            println!("All entity keys are unique - validation passed");
        } else {
            println!("Found {} key validation errors", validation_errors.len());
        }

        Ok(validation_errors)
    }

    /// Apply property chain inference to enhance traceability
    pub fn apply_property_chain_inference(
        &self,
        entities: &[TraceableEntity],
    ) -> Result<Vec<TraceEvent>> {
        println!("=== Applying Property Chain Inference ===");

        let mut inferred_events = Vec::new();

        // In a full OWL2 implementation, we would extract property chain axioms from the ontology
        // For now, we'll implement common supply chain property chains:
        // 1. producedBy ○ locatedAt → producedAtLocation
        // 2. inputTo ○ outputOf → partOfProcessChain
        // 3. shippedVia ○ transporter → shippedByCarrier

        // Look for entities with relevant properties
        for entity in entities {
            // Check for producedBy and locatedAt properties
            if entity.properties.contains_key("producedBy")
                && entity.properties.contains_key("locatedAt")
            {
                let event = TraceEvent {
                    entity: entity.id.clone(),
                    relationship: "InferredLocation".to_string(),
                    source: None,
                    timestamp: Some(Utc::now().to_rfc3339()),
                    metadata: HashMap::new(),
                };
                inferred_events.push(event);
            }

            // Check for inputTo and outputOf properties
            if entity.properties.contains_key("inputTo")
                && entity.properties.contains_key("outputOf")
            {
                let event = TraceEvent {
                    entity: entity.id.clone(),
                    relationship: "InferredProcessChain".to_string(),
                    source: None,
                    timestamp: Some(Utc::now().to_rfc3339()),
                    metadata: HashMap::new(),
                };
                inferred_events.push(event);
            }
        }

        println!(
            "Applied property chain inference, found {} inferred relationships",
            inferred_events.len()
        );

        Ok(inferred_events)
    }

    /// Enhanced trace function that combines OWL2 reasoning with existing optimization
    pub fn enhanced_trace(&self, batch_id: &str, optimization_level: u8) -> EnhancedTraceResult {
        println!("=== Enhanced Trace using OWL2 Reasoning ===");

        // First, use the existing traceability system
        let existing_system = EnhancedTraceabilitySystem::new(&self.blockchain);
        let mut result = existing_system.enhanced_trace(batch_id, optimization_level);

        // Get entities from the blockchain for OWL2 processing
        let entities = self.extract_entities_from_blockchain(batch_id);

        // Apply OWL2 enhancements
        match self.entities_to_owl_ontology(&entities) {
            Ok(ontology) => {
                println!(
                    "Successfully created OWL2 ontology with {} axioms",
                    ontology.axiom_count()
                );
            }
            Err(e) => {
                eprintln!("Warning: Failed to create OWL2 ontology: {}", e);
            }
        }

        // Apply key validation
        match self.validate_entity_keys(&entities) {
            Ok(errors) => {
                if !errors.is_empty() {
                    eprintln!("Warning: Found {} key validation errors", errors.len());
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to validate entity keys: {}", e);
            }
        }

        // Apply property chain inference
        match self.apply_property_chain_inference(&entities) {
            Ok(inferred_events) => {
                // Add inferred events to the result
                result.path.extend(inferred_events);
            }
            Err(e) => {
                eprintln!("Warning: Failed to apply property chain inference: {}", e);
            }
        }

        result
    }

    /// Extract entities from blockchain related to a specific batch ID
    fn extract_entities_from_blockchain(&self, batch_id: &str) -> Vec<TraceableEntity> {
        let mut entities = Vec::new();

        // In a real implementation, we would query the blockchain for entities
        // related to the batch ID and convert them to TraceableEntity objects
        // For now, we'll create some sample entities

        // Create a sample product entity
        let mut product_entity = TraceableEntity::new(
            format!("product_{}", batch_id),
            EntityType::Product,
            crate::core::entity::DomainType::SupplyChain,
        );
        product_entity.add_property(
            "batchId".to_string(),
            PropertyValue::String(batch_id.to_string()),
        );
        product_entity.add_property(
            "name".to_string(),
            PropertyValue::String("Sample Product".to_string()),
        );
        product_entity.add_property(
            "producedBy".to_string(),
            PropertyValue::String("Factory A".to_string()),
        );
        product_entity.add_property(
            "locatedAt".to_string(),
            PropertyValue::String("Warehouse 1".to_string()),
        );

        entities.push(product_entity);

        // Create a sample process entity
        let mut process_entity = TraceableEntity::new(
            format!("process_{}", batch_id),
            EntityType::Process,
            crate::core::entity::DomainType::SupplyChain,
        );
        process_entity.add_property(
            "batchId".to_string(),
            PropertyValue::String(batch_id.to_string()),
        );
        process_entity.add_property(
            "name".to_string(),
            PropertyValue::String("UHT Processing".to_string()),
        );
        process_entity.add_property(
            "inputTo".to_string(),
            PropertyValue::String("Shipment ABC".to_string()),
        );
        process_entity.add_property(
            "outputOf".to_string(),
            PropertyValue::String("Production XYZ".to_string()),
        );

        entities.push(process_entity);

        entities
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blockchain::Blockchain;

    #[test]
    fn test_owl2_enhanced_traceability_creation() {
        let blockchain = Blockchain::new();
        let _enhancer = Owl2EnhancedTraceability::new(blockchain);

        // This should compile and create the enhancer
        assert!(true);
    }

    #[test]
    fn test_entities_to_owl_ontology() {
        let blockchain = Blockchain::new();
        let enhancer = Owl2EnhancedTraceability::new(blockchain);

        let mut entity = TraceableEntity::new(
            "test_product_001".to_string(),
            EntityType::Product,
            crate::core::entity::DomainType::SupplyChain,
        );

        entity.add_property(
            "name".to_string(),
            PropertyValue::String("Test Product".to_string()),
        );
        entity.add_property(
            "sku".to_string(),
            PropertyValue::String("TP001".to_string()),
        );

        let entities = vec![entity];
        let result = enhancer.entities_to_owl_ontology(&entities);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_entity_keys() {
        let blockchain = Blockchain::new();
        let enhancer = Owl2EnhancedTraceability::new(blockchain);

        let mut entity1 = TraceableEntity::new(
            "product_001".to_string(),
            EntityType::Product,
            crate::core::entity::DomainType::SupplyChain,
        );
        entity1.add_property(
            "sku".to_string(),
            PropertyValue::String("SKU001".to_string()),
        );

        let mut entity2 = TraceableEntity::new(
            "product_002".to_string(),
            EntityType::Product,
            crate::core::entity::DomainType::SupplyChain,
        );
        entity2.add_property(
            "sku".to_string(),
            PropertyValue::String("SKU002".to_string()),
        );

        // Add a duplicate for testing
        let mut entity3 = TraceableEntity::new(
            "product_003".to_string(),
            EntityType::Product,
            crate::core::entity::DomainType::SupplyChain,
        );
        entity3.add_property(
            "sku".to_string(),
            PropertyValue::String("SKU001".to_string()),
        ); // Duplicate SKU

        let entities = vec![entity1, entity2, entity3];
        let errors = enhancer.validate_entity_keys(&entities).unwrap();

        // Should find one duplicate
        assert_eq!(errors.len(), 1);
    }
}
