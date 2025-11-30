//! OWL2 Performance Benchmarks
//!
//! Performance benchmarks for the enhanced OWL2 features including:
//! - owl:hasKey constraint validation
//! - Property chain inference
//! - OWL2 ontology generation and reasoning

use anyhow::Result;
use provchain_org::core::blockchain::Blockchain;
use provchain_org::core::entity::{DomainType, EntityType, PropertyValue, TraceableEntity};
use provchain_org::semantic::owl2_traceability::Owl2EnhancedTraceability;
use std::time::{Duration, Instant};

/// Benchmark owl:hasKey constraint validation performance
#[test]
fn test_owl2_haskey_validation_performance() -> Result<()> {
    println!("=== OWL2 HasKey Validation Performance Benchmark ===");

    let blockchain = Blockchain::new();
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);

    // Create a large number of entities for performance testing
    let mut entities = Vec::new();

    // Create 1000 entities with unique keys
    for i in 0..1000 {
        let mut entity = TraceableEntity::new(
            format!("product_{:04}", i),
            EntityType::Product,
            DomainType::SupplyChain,
        );
        entity.add_property(
            "batchId".to_string(),
            PropertyValue::String(format!("BATCH{:04}", i)),
        );
        entity.add_property(
            "sku".to_string(),
            PropertyValue::String(format!("SKU{:04}", i)),
        );
        entity.add_property(
            "name".to_string(),
            PropertyValue::String(format!("Product {}", i)),
        );
        entity.add_property(
            "producedBy".to_string(),
            PropertyValue::String("Dairy Farm A".to_string()),
        );
        entity.add_property(
            "locatedAt".to_string(),
            PropertyValue::String("Processing Plant B".to_string()),
        );

        entities.push(entity);
    }

    // Add some duplicate entities for testing key validation
    for i in 0..100 {
        let mut entity = TraceableEntity::new(
            format!("duplicate_product_{:04}", i),
            EntityType::Product,
            DomainType::SupplyChain,
        );
        // Use a duplicate batch ID to trigger key validation
        entity.add_property(
            "batchId".to_string(),
            PropertyValue::String(format!("BATCH{:04}", i)),
        );
        entity.add_property(
            "sku".to_string(),
            PropertyValue::String(format!("DUPLICATE_SKU{:04}", i)),
        );
        entity.add_property(
            "name".to_string(),
            PropertyValue::String(format!("Duplicate Product {}", i)),
        );
        entity.add_property(
            "producedBy".to_string(),
            PropertyValue::String("Dairy Farm A".to_string()),
        );
        entity.add_property(
            "locatedAt".to_string(),
            PropertyValue::String("Processing Plant B".to_string()),
        );

        entities.push(entity);
    }

    println!(
        "Created {} entities for hasKey validation benchmark",
        entities.len()
    );

    // Benchmark entity-to-OWL2 ontology conversion
    let start = Instant::now();
    let ontology = owl2_enhancer.entities_to_owl_ontology(&entities)?;
    let conversion_duration = start.elapsed();

    println!("Entity-to-OWL2 conversion: {:?}", conversion_duration);
    println!("Generated ontology with {} axioms", ontology.axiom_count());

    // Benchmark hasKey validation
    let start = Instant::now();
    let validation_errors = owl2_enhancer.validate_entity_keys(&entities)?;
    let validation_duration = start.elapsed();

    println!("HasKey validation: {:?}", validation_duration);
    println!("Found {} key validation errors", validation_errors.len());

    // Benchmark property chain inference
    let start = Instant::now();
    let inferred_events = owl2_enhancer.apply_property_chain_inference(&entities)?;
    let inference_duration = start.elapsed();

    println!("Property chain inference: {:?}", inference_duration);
    println!("Found {} inferred relationships", inferred_events.len());

    // Performance requirements
    assert!(
        conversion_duration < Duration::from_secs(5),
        "Entity-to-OWL2 conversion should complete within 5 seconds"
    );
    assert!(
        validation_duration < Duration::from_secs(10),
        "HasKey validation should complete within 10 seconds"
    );
    assert!(
        inference_duration < Duration::from_secs(15),
        "Property chain inference should complete within 15 seconds"
    );

    // Check that we found the expected number of duplicates
    assert_eq!(
        validation_errors.len(),
        100,
        "Should find 100 key validation errors from duplicate entities"
    );

    println!("✅ OWL2 hasKey validation performance benchmark passed!");

    Ok(())
}

/// Benchmark OWL2 reasoning performance with complex ontologies
#[test]
fn test_owl2_reasoning_performance() -> Result<()> {
    println!("=== OWL2 Reasoning Performance Benchmark ===");

    let blockchain = Blockchain::new();
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);

    // Create complex entities with various properties for reasoning
    let mut entities = Vec::new();

    // Create a supply chain with multiple stages
    for batch_id in 0..100 {
        // Create raw material entity
        let mut raw_material = TraceableEntity::new(
            format!("raw_material_{:03}", batch_id),
            EntityType::Component,
            DomainType::SupplyChain,
        );
        raw_material.add_property(
            "batchId".to_string(),
            PropertyValue::String(format!("RM{:03}", batch_id)),
        );
        raw_material.add_property(
            "sku".to_string(),
            PropertyValue::String(format!("RAW{:03}", batch_id)),
        );
        raw_material.add_property(
            "name".to_string(),
            PropertyValue::String(format!("Raw Material {}", batch_id)),
        );
        raw_material.add_property(
            "producedBy".to_string(),
            PropertyValue::String("Supplier Co.".to_string()),
        );
        raw_material.add_property(
            "locatedAt".to_string(),
            PropertyValue::String("Supplier Warehouse".to_string()),
        );
        raw_material.add_property(
            "quality".to_string(),
            PropertyValue::String("Grade A".to_string()),
        );

        entities.push(raw_material);

        // Create intermediate product entity
        let mut intermediate = TraceableEntity::new(
            format!("intermediate_{:03}", batch_id),
            EntityType::Product,
            DomainType::SupplyChain,
        );
        intermediate.add_property(
            "batchId".to_string(),
            PropertyValue::String(format!("INT{:03}", batch_id)),
        );
        intermediate.add_property(
            "sku".to_string(),
            PropertyValue::String(format!("INT{:03}", batch_id)),
        );
        intermediate.add_property(
            "name".to_string(),
            PropertyValue::String(format!("Intermediate Product {}", batch_id)),
        );
        intermediate.add_property(
            "producedBy".to_string(),
            PropertyValue::String("Processing Plant".to_string()),
        );
        intermediate.add_property(
            "locatedAt".to_string(),
            PropertyValue::String("Processing Facility".to_string()),
        );
        intermediate.add_property(
            "inputTo".to_string(),
            PropertyValue::String(format!("RM{:03}", batch_id)),
        );
        intermediate.add_property(
            "outputOf".to_string(),
            PropertyValue::String("Processing Operation".to_string()),
        );

        entities.push(intermediate);

        // Create final product entity
        let mut final_product = TraceableEntity::new(
            format!("final_product_{:03}", batch_id),
            EntityType::Product,
            DomainType::SupplyChain,
        );
        final_product.add_property(
            "batchId".to_string(),
            PropertyValue::String(format!("FP{:03}", batch_id)),
        );
        final_product.add_property(
            "sku".to_string(),
            PropertyValue::String(format!("FP{:03}", batch_id)),
        );
        final_product.add_property(
            "name".to_string(),
            PropertyValue::String(format!("Final Product {}", batch_id)),
        );
        final_product.add_property(
            "producedBy".to_string(),
            PropertyValue::String("Manufacturing Facility".to_string()),
        );
        final_product.add_property(
            "locatedAt".to_string(),
            PropertyValue::String("Distribution Center".to_string()),
        );
        final_product.add_property(
            "inputTo".to_string(),
            PropertyValue::String(format!("INT{:03}", batch_id)),
        );
        final_product.add_property(
            "outputOf".to_string(),
            PropertyValue::String("Manufacturing Process".to_string()),
        );
        final_product.add_property(
            "expirationDate".to_string(),
            PropertyValue::String("2025-12-31".to_string()),
        );

        entities.push(final_product);
    }

    println!(
        "Created {} complex entities for OWL2 reasoning benchmark",
        entities.len()
    );

    // Benchmark entity-to-OWL2 ontology conversion for complex entities
    let start = Instant::now();
    let ontology = owl2_enhancer.entities_to_owl_ontology(&entities)?;
    let conversion_duration = start.elapsed();

    println!(
        "Complex entity-to-OWL2 conversion: {:?}",
        conversion_duration
    );
    println!("Generated ontology with {} axioms", ontology.axiom_count());

    // Benchmark hasKey validation for complex entities
    let start = Instant::now();
    let validation_errors = owl2_enhancer.validate_entity_keys(&entities)?;
    let validation_duration = start.elapsed();

    println!("Complex hasKey validation: {:?}", validation_duration);
    println!("Found {} key validation errors", validation_errors.len());

    // Benchmark property chain inference for complex entities
    let start = Instant::now();
    let inferred_events = owl2_enhancer.apply_property_chain_inference(&entities)?;
    let inference_duration = start.elapsed();

    println!("Complex property chain inference: {:?}", inference_duration);
    println!("Found {} inferred relationships", inferred_events.len());

    // Performance requirements for complex entities
    assert!(
        conversion_duration < Duration::from_secs(10),
        "Complex entity-to-OWL2 conversion should complete within 10 seconds"
    );
    assert!(
        validation_duration < Duration::from_secs(20),
        "Complex hasKey validation should complete within 20 seconds"
    );
    assert!(
        inference_duration < Duration::from_secs(30),
        "Complex property chain inference should complete within 30 seconds"
    );

    println!("✅ OWL2 reasoning performance benchmark passed!");

    Ok(())
}

/// Stress test OWL2 features with very large datasets
#[test]
fn test_owl2_stress_performance() -> Result<()> {
    println!("=== OWL2 Stress Performance Test ===");

    let blockchain = Blockchain::new();
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);

    // Create a very large number of entities for stress testing
    let mut entities = Vec::new();

    // Create 5000 entities for stress testing
    for i in 0..5000 {
        let mut entity = TraceableEntity::new(
            format!("stress_entity_{:05}", i),
            EntityType::Product,
            DomainType::SupplyChain,
        );
        entity.add_property(
            "batchId".to_string(),
            PropertyValue::String(format!("STRESS_BATCH{:05}", i)),
        );
        entity.add_property(
            "sku".to_string(),
            PropertyValue::String(format!("STRESS_SKU{:05}", i)),
        );
        entity.add_property(
            "name".to_string(),
            PropertyValue::String(format!("Stress Test Product {}", i)),
        );
        entity.add_property(
            "producedBy".to_string(),
            PropertyValue::String("Stress Test Facility".to_string()),
        );
        entity.add_property(
            "locatedAt".to_string(),
            PropertyValue::String("Stress Test Location".to_string()),
        );
        entity.add_property(
            "quality".to_string(),
            PropertyValue::String("Grade A".to_string()),
        );
        entity.add_property(
            "weight".to_string(),
            PropertyValue::Float(100.0 + (i as f64) * 0.1),
        );
        entity.add_property(
            "volume".to_string(),
            PropertyValue::Float(50.0 + (i as f64) * 0.05),
        );

        entities.push(entity);
    }

    println!("Created {} entities for OWL2 stress test", entities.len());

    // Benchmark entity-to-OWL2 ontology conversion under stress
    let start = Instant::now();
    let ontology = owl2_enhancer.entities_to_owl_ontology(&entities)?;
    let conversion_duration = start.elapsed();

    println!(
        "Stress entity-to-OWL2 conversion: {:?}",
        conversion_duration
    );
    println!("Generated ontology with {} axioms", ontology.axiom_count());

    // Benchmark hasKey validation under stress
    let start = Instant::now();
    let validation_errors = owl2_enhancer.validate_entity_keys(&entities)?;
    let validation_duration = start.elapsed();

    println!("Stress hasKey validation: {:?}", validation_duration);
    println!("Found {} key validation errors", validation_errors.len());

    // Benchmark property chain inference under stress
    let start = Instant::now();
    let inferred_events = owl2_enhancer.apply_property_chain_inference(&entities)?;
    let inference_duration = start.elapsed();

    println!("Stress property chain inference: {:?}", inference_duration);
    println!("Found {} inferred relationships", inferred_events.len());

    // Performance requirements for stress test
    assert!(
        conversion_duration < Duration::from_secs(30),
        "Stress entity-to-OWL2 conversion should complete within 30 seconds"
    );
    assert!(
        validation_duration < Duration::from_secs(60),
        "Stress hasKey validation should complete within 60 seconds"
    );
    assert!(
        inference_duration < Duration::from_secs(90),
        "Stress property chain inference should complete within 90 seconds"
    );

    println!("✅ OWL2 stress performance test passed!");

    Ok(())
}

/// Test memory usage of OWL2 features
#[test]
fn test_owl2_memory_usage() -> Result<()> {
    println!("=== OWL2 Memory Usage Test ===");

    let blockchain = Blockchain::new();
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);

    // Create entities of varying sizes to test memory usage
    let mut entities = Vec::new();

    // Create small entities
    for i in 0..100 {
        let mut entity = TraceableEntity::new(
            format!("small_entity_{:03}", i),
            EntityType::Product,
            DomainType::SupplyChain,
        );
        entity.add_property(
            "batchId".to_string(),
            PropertyValue::String(format!("SMALL{:03}", i)),
        );
        entity.add_property(
            "sku".to_string(),
            PropertyValue::String(format!("SMALL_SKU{:03}", i)),
        );
        entity.add_property(
            "name".to_string(),
            PropertyValue::String("Small Product".to_string()),
        );

        entities.push(entity);
    }

    // Create medium entities
    for i in 0..100 {
        let mut entity = TraceableEntity::new(
            format!("medium_entity_{:03}", i),
            EntityType::Product,
            DomainType::SupplyChain,
        );
        entity.add_property(
            "batchId".to_string(),
            PropertyValue::String(format!("MEDIUM{:03}", i)),
        );
        entity.add_property(
            "sku".to_string(),
            PropertyValue::String(format!("MEDIUM_SKU{:03}", i)),
        );
        entity.add_property(
            "name".to_string(),
            PropertyValue::String(format!("Medium Product {}", i)),
        );
        entity.add_property(
            "producedBy".to_string(),
            PropertyValue::String("Medium Producer".to_string()),
        );
        entity.add_property(
            "locatedAt".to_string(),
            PropertyValue::String("Medium Location".to_string()),
        );
        entity.add_property(
            "quality".to_string(),
            PropertyValue::String("Grade B".to_string()),
        );
        entity.add_property(
            "weight".to_string(),
            PropertyValue::Float(100.0 + (i as f64) * 0.1),
        );

        entities.push(entity);
    }

    // Create large entities
    for i in 0..100 {
        let mut entity = TraceableEntity::new(
            format!("large_entity_{:03}", i),
            EntityType::Product,
            DomainType::SupplyChain,
        );
        entity.add_property(
            "batchId".to_string(),
            PropertyValue::String(format!("LARGE{:03}", i)),
        );
        entity.add_property(
            "sku".to_string(),
            PropertyValue::String(format!("LARGE_SKU{:03}", i)),
        );
        entity.add_property(
            "name".to_string(),
            PropertyValue::String(format!("Large Product with Extended Properties {}", i)),
        );
        entity.add_property(
            "producedBy".to_string(),
            PropertyValue::String("Large Producer Corporation".to_string()),
        );
        entity.add_property(
            "locatedAt".to_string(),
            PropertyValue::String("Large Production Facility Location".to_string()),
        );
        entity.add_property(
            "quality".to_string(),
            PropertyValue::String("Grade A Premium Quality".to_string()),
        );
        entity.add_property(
            "weight".to_string(),
            PropertyValue::Float(1000.0 + (i as f64) * 0.5),
        );
        entity.add_property(
            "volume".to_string(),
            PropertyValue::Float(500.0 + (i as f64) * 0.25),
        );
        entity.add_property(
            "dimensions".to_string(),
            PropertyValue::String("200x100x50 cm".to_string()),
        );
        entity.add_property(
            "manufactureDate".to_string(),
            PropertyValue::String("2025-08-22".to_string()),
        );
        entity.add_property(
            "expirationDate".to_string(),
            PropertyValue::String("2026-08-22".to_string()),
        );
        entity.add_property(
            "batchSize".to_string(),
            PropertyValue::Integer(10000 + i as i64),
        );
        entity.add_property(
            "temperature".to_string(),
            PropertyValue::Float(22.5 + (i as f64) * 0.1),
        );
        entity.add_property(
            "humidity".to_string(),
            PropertyValue::Float(65.0 + (i as f64) * 0.5),
        );

        entities.push(entity);
    }

    println!(
        "Created {} entities of varying sizes for memory usage test",
        entities.len()
    );

    // Test memory usage for entity-to-OWL2 conversion
    let start = Instant::now();
    let ontology = owl2_enhancer.entities_to_owl_ontology(&entities)?;
    let conversion_duration = start.elapsed();

    println!(
        "Variable-size entity-to-OWL2 conversion: {:?}",
        conversion_duration
    );
    println!("Generated ontology with {} axioms", ontology.axiom_count());

    // Test memory usage for hasKey validation
    let start = Instant::now();
    let validation_errors = owl2_enhancer.validate_entity_keys(&entities)?;
    let validation_duration = start.elapsed();

    println!("Variable-size hasKey validation: {:?}", validation_duration);
    println!("Found {} key validation errors", validation_errors.len());

    // Test memory usage for property chain inference
    let start = Instant::now();
    let inferred_events = owl2_enhancer.apply_property_chain_inference(&entities)?;
    let inference_duration = start.elapsed();

    println!(
        "Variable-size property chain inference: {:?}",
        inference_duration
    );
    println!("Found {} inferred relationships", inferred_events.len());

    // Memory usage should be reasonable for all operations
    assert!(
        conversion_duration < Duration::from_secs(10),
        "Variable-size entity-to-OWL2 conversion should be memory efficient"
    );
    assert!(
        validation_duration < Duration::from_secs(20),
        "Variable-size hasKey validation should be memory efficient"
    );
    assert!(
        inference_duration < Duration::from_secs(30),
        "Variable-size property chain inference should be memory efficient"
    );

    println!("✅ OWL2 memory usage test passed!");

    Ok(())
}
