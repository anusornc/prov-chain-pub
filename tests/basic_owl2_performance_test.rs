//! Basic OWL2 Performance Test
//!
//! Simple test to verify OWL2 integration performance

use provchain_org::core::blockchain::Blockchain;
use provchain_org::core::entity::{DomainType, EntityType, PropertyValue, TraceableEntity};
use provchain_org::semantic::owl2_traceability::Owl2EnhancedTraceability;

/// Test basic OWL2 integration performance
#[test]
fn test_basic_owl2_integration_performance() {
    println!("=== Basic OWL2 Integration Performance Test ===");

    // Create a blockchain instance
    let blockchain = Blockchain::new();

    // Create the enhanced traceability system
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);

    // Create sample entities
    let mut entities = Vec::new();

    // Create a few entities for testing
    for i in 0..10 {
        let mut entity = TraceableEntity::new(
            format!("entity_{:02}", i),
            EntityType::Product,
            DomainType::SupplyChain,
        );
        entity.add_property(
            "batchId".to_string(),
            PropertyValue::String(format!("BATCH{:02}", i)),
        );
        entity.add_property(
            "sku".to_string(),
            PropertyValue::String(format!("SKU{:02}", i)),
        );
        entity.add_property(
            "name".to_string(),
            PropertyValue::String(format!("Product {}", i)),
        );
        entity.add_property(
            "producedBy".to_string(),
            PropertyValue::String("Producer A".to_string()),
        );
        entity.add_property(
            "locatedAt".to_string(),
            PropertyValue::String("Location B".to_string()),
        );

        entities.push(entity);
    }

    println!("Created {} sample entities", entities.len());

    // Test entity-to-OWL2 conversion performance
    let start = std::time::Instant::now();
    match owl2_enhancer.entities_to_owl_ontology(&entities) {
        Ok(ontology) => {
            let duration = start.elapsed();
            println!("✅ Entity-to-OWL2 conversion: {:?}", duration);
            println!(
                "   Generated ontology with {} axioms",
                ontology.axiom_count()
            );

            // Performance requirement
            assert!(
                duration < std::time::Duration::from_secs(5),
                "Entity-to-OWL2 conversion should complete within 5 seconds"
            );
        }
        Err(e) => {
            eprintln!("❌ Failed to convert entities to OWL2 ontology: {}", e);
            panic!("Entity-to-OWL2 conversion failed");
        }
    }

    // Test hasKey validation performance
    let start = std::time::Instant::now();
    match owl2_enhancer.validate_entity_keys(&entities) {
        Ok(errors) => {
            let duration = start.elapsed();
            println!("✅ HasKey validation: {:?}", duration);
            println!("   Found {} key validation errors", errors.len());

            // Performance requirement
            assert!(
                duration < std::time::Duration::from_secs(10),
                "HasKey validation should complete within 10 seconds"
            );
        }
        Err(e) => {
            eprintln!("❌ Failed to validate entity keys: {}", e);
            panic!("HasKey validation failed");
        }
    }

    // Test property chain inference performance
    let start = std::time::Instant::now();
    match owl2_enhancer.apply_property_chain_inference(&entities) {
        Ok(inferred_events) => {
            let duration = start.elapsed();
            println!("✅ Property chain inference: {:?}", duration);
            println!("   Found {} inferred relationships", inferred_events.len());

            // Performance requirement
            assert!(
                duration < std::time::Duration::from_secs(15),
                "Property chain inference should complete within 15 seconds"
            );
        }
        Err(e) => {
            eprintln!("❌ Failed to apply property chain inference: {}", e);
            panic!("Property chain inference failed");
        }
    }

    println!("✅ Basic OWL2 integration performance test passed!");
}
