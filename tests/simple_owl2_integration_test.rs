//! Simple OWL2 Integration Test
//!
//! Basic test to verify OWL2 integration is working

use provchain_org::core::blockchain::Blockchain;
use provchain_org::core::entity::{DomainType, EntityType, PropertyValue, TraceableEntity};
use provchain_org::semantic::owl2_traceability::Owl2EnhancedTraceability;

/// Simple test to verify OWL2 integration is working
#[test]
fn test_simple_owl2_integration() {
    println!("=== Simple OWL2 Integration Test ===");

    // Create a blockchain instance
    let blockchain = Blockchain::new();

    // Create the enhanced traceability system
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);

    // Create a simple entity
    let mut entity = TraceableEntity::new(
        "test_product_001".to_string(),
        EntityType::Product,
        DomainType::SupplyChain,
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

    // Test entity-to-OWL2 conversion
    match owl2_enhancer.entities_to_owl_ontology(&entities) {
        Ok(ontology) => {
            println!(
                "✅ Successfully created ontology with {} axioms",
                ontology.axiom_count()
            );
            assert!(ontology.axiom_count() > 0);
        }
        Err(e) => {
            eprintln!("❌ Failed to create ontology: {}", e);
            panic!("OWL2 integration test failed");
        }
    }

    println!("✅ Simple OWL2 integration test passed!");
}
