/// Test traceability performance with and without OWL2 enhancements
#[test]
fn test_traceability_performance_comparison() -> Result<()> {
    println!("=== Traceability Performance Comparison ===");
    
    let mut blockchain = Blockchain::new();
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);
    
    // Add test data to blockchain
    let test_data = vec![
        r#"@prefix : <http://example.org/> .
           @prefix tc: <http://provchain.org/trace#> .
           
           :batch001 tc:product "Organic Milk" ;
                     tc:origin "Dairy Farm A" ;
                     tc:batchId "MB001" ;
                     tc:status "Produced" ;
                     tc:timestamp "2024-01-15T10:00:00Z" ."#,
                     
        r#"@prefix : <http://example.org/> .
           @prefix tc: <http://provchain.org/trace#> .
           
           :batch002 tc:product "UHT Milk" ;
                     tc:origin "Processing Plant B" ;
                     tc:batchId "UMB001" ;
                     tc:status "Processed" ;
                     tc:timestamp "2024-01-15T14:00:00Z" ;
                     tc:inputTo :batch001 ."#,
                     
        r#"@prefix : <http://example.org/> .
           @prefix tc: <http://provchain.org/trace#> .
           
           :batch003 tc:product "Packaged Milk" ;
                     tc:origin "Packaging Facility C" ;
                     tc:batchId "PMB001" ;
                     tc:status "Packaged" ;
                     tc:timestamp "2024-01-15T16:00:00Z" ;
                     tc:inputTo :batch002 ."#,
    ];
    
    // Add data to blockchain
    for data in test_data {
        let _ = blockchain.add_block(data.to_string());
    }
    
    // Traditional traceability performance
    let start = Instant::now();
    let _traditional_result = blockchain.trace_entity("batch001");
    let traditional_duration = start.elapsed();
    
    // Enhanced OWL2 traceability performance
    let start = Instant::now();
    let _enhanced_result = owl2_enhancer.enhanced_trace("batch001", 1);
    let enhanced_duration = start.elapsed();
    
    println!("Traditional traceability: {:?}", traditional_duration);
    println!("Enhanced OWL2 traceability: {:?}", enhanced_duration);
    
    // Performance comparison
    let improvement_factor = if enhanced_duration.as_millis() > 0 {
        traditional_duration.as_millis() as f64 / enhanced_duration.as_millis() as f64
    } else {
        1.0
    };
    
    println!("Performance improvement factor: {:.2}x", improvement_factor);
    
    // Enhanced traceability should be competitive even with additional OWL2 reasoning
    assert!(enhanced_duration < Duration::from_secs(5), 
            "Enhanced OWL2 traceability should complete within 5 seconds");
    
    println!("✅ Traceability performance comparison test passed!");
    
    Ok(())
}

/// Test entity validation performance with and without OWL2 hasKey constraints
#[test]
fn test_entity_validation_performance() -> Result<()> {
    println!("=== Entity Validation Performance Comparison ===");
    
    let mut blockchain = Blockchain::new();
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);
    
    // Create entities with unique keys for testing
    let mut entities = Vec::new();
    
    // Create 500 entities with unique batch IDs
    for i in 0..500 {
        let mut entity = provchain_org::core::entity::TraceableEntity::new(
            format!("entity_{:03}", i),
            provchain_org::core::entity::EntityType::Product,
            provchain_org::core::entity::DomainType::SupplyChain,
        );
        entity.add_property("batchId".to_string(), 
                           provchain_org::core::entity::PropertyValue::String(format!("BATCH{:03}", i)));
        entity.add_property("sku".to_string(), 
                           provchain_org::core::entity::PropertyValue::String(format!("SKU{:03}", i)));
        entity.add_property("name".to_string(), 
                           provchain_org::core::entity::PropertyValue::String(format!("Product {}", i)));
        entity.add_property("producedBy".to_string(), 
                           provchain_org::core::entity::PropertyValue::String("Producer A".to_string()));
        entity.add_property("locatedAt".to_string(), 
                           provchain_org::core::entity::PropertyValue::String("Location B".to_string()));
        
        entities.push(entity);
    }
    
    // Traditional validation performance (baseline)
    let start = Instant::now();
    let traditional_errors = blockchain.validate_entities(&entities);
    let traditional_duration = start.elapsed();
    
    // Enhanced OWL2 validation with hasKey constraints
    let start = Instant::now();
    let owl2_errors = owl2_enhancer.validate_entity_keys(&entities)?;
    let owl2_duration = start.elapsed();
    
    println!("Traditional entity validation: {:?}", traditional_duration);
    println!("Enhanced OWL2 hasKey validation: {:?}", owl2_duration);
    println!("Traditional errors: {}, OWL2 errors: {}", 
             traditional_errors.len(), owl2_errors.len());
    
    // Performance comparison
    let improvement_factor = if owl2_duration.as_millis() > 0 {
        traditional_duration.as_millis() as f64 / owl2_duration.as_millis() as f64
    } else {
        1.0
    };
    
    println!("Validation performance factor: {:.2}x", improvement_factor);
    
    // Both should complete within reasonable time
    assert!(traditional_duration < Duration::from_secs(10), 
            "Traditional validation should complete within 10 seconds");
    assert!(owl2_duration < Duration::from_secs(15), 
            "Enhanced OWL2 validation should complete within 15 seconds");
    
    println!("✅ Entity validation performance test passed!");
    
    Ok(())
}

/// Test property chain inference performance
#[test]
fn test_property_chain_inference_performance() -> Result<()> {
    println!("=== Property Chain Inference Performance Test ===");
    
    let mut blockchain = Blockchain::new();
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);
    
    // Create entities with chainable properties
    let mut entities = Vec::new();
    
    // Create a supply chain with multiple linked entities
    for i in 0..200 {
        // Raw material
        let mut raw_material = provchain_org::core::entity::TraceableEntity::new(
            format!("raw_{}", i),
            provchain_org::core::entity::EntityType::Component,
            provchain_org::core::entity::DomainType::SupplyChain,
        );
        raw_material.add_property("batchId".to_string(), 
                                provchain_org::core::entity::PropertyValue::String(format!("RAW{:03}", i)));
        raw_material.add_property("producedBy".to_string(), 
                                provchain_org::core::entity::PropertyValue::String("Supplier A".to_string()));
        raw_material.add_property("locatedAt".to_string(), 
                                provchain_org::core::entity::PropertyValue::String("Supplier Warehouse".to_string()));
        
        entities.push(raw_material);
        
        // Processed material
        let mut processed = provchain_org::core::entity::TraceableEntity::new(
            format!("processed_{}", i),
            provchain_org::core::entity::EntityType::Product,
            provchain_org::core::entity::DomainType::SupplyChain,
        );
        processed.add_property("batchId".to_string(), 
                              provchain_org::core::entity::PropertyValue::String(format!("PROC{:03}", i)));
        processed.add_property("producedBy".to_string(), 
                              provchain_org::core::entity::PropertyValue::String("Processing Plant B".to_string()));
        processed.add_property("locatedAt".to_string(), 
                              provchain_org::core::entity::PropertyValue::String("Processing Facility".to_string()));
        processed.add_property("inputTo".to_string(), 
                              provchain_org::core::entity::PropertyValue::String(format!("RAW{:03}", i)));
        processed.add_property("outputOf".to_string(), 
                              provchain_org::core::entity::PropertyValue::String("Processing Operation".to_string()));
        
        entities.push(processed);
        
        // Final product
        let mut final_product = provchain_org::core::entity::TraceableEntity::new(
            format!("final_{}", i),
            provchain_org::core::entity::EntityType::Product,
            provchain_org::core::entity::DomainType::SupplyChain,
        );
        final_product.add_property("batchId".to_string(), 
                                 provchain_org::core::entity::PropertyValue::String(format!("FIN{:03}", i)));
        final_product.add_property("producedBy".to_string(), 
                                 provchain_org::core::entity::PropertyValue::String("Manufacturing Facility C".to_string()));
        final_product.add_property("locatedAt".to_string(), 
                                 provchain_org::core::entity::PropertyValue::String("Manufacturing Plant".to_string()));
        final_product.add_property("inputTo".to_string(), 
                                 provchain_org::core::entity::PropertyValue::String(format!("PROC{:03}", i)));
        final_product.add_property("outputOf".to_string(), 
                                 provchain_org::core::entity::PropertyValue::String("Manufacturing Process".to_string()));
        
        entities.push(final_product);
    }
    
    println!("Created {} entities for property chain inference test", entities.len());
    
    // Benchmark property chain inference performance
    let start = Instant::now();
    let inferred_events = owl2_enhancer.apply_property_chain_inference(&entities)?;
    let inference_duration = start.elapsed();
    
    println!("Property chain inference: {:?}", inference_duration);
    println!("Found {} inferred relationships", inferred_events.len());
    
    // Performance requirements
    assert!(inference_duration < Duration::from_secs(30), 
            "Property chain inference should complete within 30 seconds");
    assert!(inferred_events.len() >= entities.len() / 3, 
            "Should infer meaningful relationships from chainable properties");
    
    println!("✅ Property chain inference performance test passed!");
    
    Ok(())
}

/// Test OWL2 ontology generation performance
#[test]
fn test_owl2_ontology_generation_performance() -> Result<()> {
    println!("=== OWL2 Ontology Generation Performance Test ===");
    
    let mut blockchain = Blockchain::new();
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);
    
    // Create a large number of diverse entities
    let mut entities = Vec::new();
    
    // Create 300 entities of different types
    for i in 0..100 {
        // Product entities
        let mut product = provchain_org::core::entity::TraceableEntity::new(
            format!("product_{}", i),
            provchain_org::core::entity::EntityType::Product,
            provchain_org::core::entity::DomainType::SupplyChain,
        );
        product.add_property("batchId".to_string(), 
                            provchain_org::core::entity::PropertyValue::String(format!("PROD{:03}", i)));
        product.add_property("sku".to_string(), 
                            provchain_org::core::entity::PropertyValue::String(format!("SKU{:03}", i)));
        product.add_property("name".to_string(), 
                            provchain_org::core::entity::PropertyValue::String(format!("Product {}", i)));
        product.add_property("producedBy".to_string(), 
                            provchain_org::core::entity::PropertyValue::String("Production Line A".to_string()));
        product.add_property("locatedAt".to_string(), 
                            provchain_org::core::entity::PropertyValue::String("Factory Floor 1".to_string()));
        
        entities.push(product);
        
        // Process entities
        let mut process = provchain_org::core::entity::TraceableEntity::new(
            format!("process_{}", i),
            provchain_org::core::entity::EntityType::Process,
            provchain_org::core::entity::DomainType::SupplyChain,
        );
        process.add_property("batchId".to_string(), 
                            provchain_org::core::entity::PropertyValue::String(format!("PROC{:03}", i)));
        process.add_property("name".to_string(), 
                            provchain_org::core::entity::PropertyValue::String(format!("Process {}", i)));
        process.add_property("inputTo".to_string(), 
                            provchain_org::core::entity::PropertyValue::String(format!("PROD{:03}", i)));
        process.add_property("outputOf".to_string(), 
                            provchain_org::core::entity::PropertyValue::String("Manufacturing Operation".to_string()));
        process.add_property("startTime".to_string(), 
                            provchain_org::core::entity::PropertyValue::String("2024-01-15T10:00:00Z".to_string()));
        process.add_property("endTime".to_string(), 
                            provchain_org::core::entity::PropertyValue::String("2024-01-15T12:00:00Z".to_string()));
        
        entities.push(process);
        
        // Person entities
        let mut person = provchain_org::core::entity::TraceableEntity::new(
            format!("person_{}", i),
            provchain_org::core::entity::EntityType::Person,
            provchain_org::core::entity::DomainType::SupplyChain,
        );
        person.add_property("employeeId".to_string(), 
                           provchain_org::core::entity::PropertyValue::String(format!("EMP{:03}", i)));
        person.add_property("name".to_string(), 
                           provchain_org::core::entity::PropertyValue::String(format!("Employee {}", i)));
        person.add_property("role".to_string(), 
                           provchain_org::core::entity::PropertyValue::String("Operator".to_string()));
        person.add_property("department".to_string(), 
                           provchain_org::core::entity::PropertyValue::String("Production".to_string()));
        person.add_property("certifications".to_string(), 
                           provchain_org::core::entity::PropertyValue::String("Safety Certified".to_string()));
        
        entities.push(person);
    }
    
    println!("Created {} diverse entities for OWL2 ontology generation test", entities.len());
    
    // Benchmark entity-to-OWL2 ontology conversion
    let start = Instant::now();
    let ontology = owl2_enhancer.entities_to_owl_ontology(&entities)?;
    let conversion_duration = start.elapsed();
    
    println!("Entity-to-OWL2 ontology conversion: {:?}", conversion_duration);
    println!("Generated ontology with {} axioms", ontology.axiom_count());
    
    // Performance requirements
    assert!(conversion_duration < Duration::from_secs(15), 
            "Entity-to-OWL2 conversion should complete within 15 seconds");
    assert!(ontology.axiom_count() >= entities.len(), 
            "Generated ontology should have at least as many axioms as entities");
    
    println!("✅ OWL2 ontology generation performance test passed!");
    
    Ok(())
}