use provchain_org::core::Blockchain;
use provchain_org::semantic::Owl2EnhancedTraceability;
use std::time::{Instant, Duration};

/// Test basic traceability performance
#[test]
fn test_basic_traceability_performance() {
    println!("=== Basic Traceability Performance Test ===");
    
    // Traditional traceability performance
    let mut blockchain = Blockchain::new();
    
    // Add simple test data
    let test_data = r#"@prefix : <http://example.org/> .
                       @prefix core: <http://provchain.org/core#> .
                       
                       :batch001 core:product "Test Batch" ;
                                 core:batchId "TB001" ;
                                 core:origin "Test Farm" ;
                                 core:status "Produced" ;
                                 core:timestamp "2024-01-15T10:00:00Z" ."#;
    
    let _ = blockchain.add_block(test_data.to_string());
    
    let start = Instant::now();
    let _traditional_result = blockchain.enhanced_trace("batch001", 0);
    let traditional_duration = start.elapsed();
    
    println!("Traditional traceability duration: {:?}", traditional_duration);
    assert!(traditional_duration < Duration::from_secs(5));
    
    println!("✅ Basic traceability performance test passed!");
}

/// Test enhanced OWL2 traceability performance
#[test]
fn test_enhanced_owl2_traceability_performance() {
    println!("=== Enhanced OWL2 Traceability Performance Test ===");
    
    let mut blockchain = Blockchain::new();
    
    // Add simple test data
    let test_data = r#"@prefix : <http://example.org/> .
                       @prefix core: <http://provchain.org/core#> .
                       
                       :batch001 core:product "Test Batch" ;
                                 core:batchId "TB001" ;
                                 core:origin "Test Farm" ;
                                 core:status "Produced" ;
                                 core:timestamp "2024-01-15T10:00:00Z" ."#;
    
    let _ = blockchain.add_block(test_data.to_string());
    
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);
    
    let start = Instant::now();
    let _enhanced_result = owl2_enhancer.enhanced_trace("batch001", 1);
    let enhanced_duration = start.elapsed();
    
    println!("Enhanced OWL2 traceability duration: {:?}", enhanced_duration);
    assert!(enhanced_duration < Duration::from_secs(5));
    
    println!("✅ Enhanced OWL2 traceability performance test passed!");
}

/// Test basic validation performance
#[test]
fn test_basic_validation_performance() {
    println!("=== Basic Validation Performance Test ===");
    
    let blockchain = Blockchain::new();
    
    let start = Instant::now();
    let _traditional_result = blockchain.is_valid();
    let traditional_duration = start.elapsed();
    
    println!("Traditional validation duration: {:?}", traditional_duration);
    assert!(traditional_duration < Duration::from_secs(5));
    
    println!("✅ Basic validation performance test passed!");
}

/// Test basic inference performance
#[test]
fn test_basic_inference_performance() {
    println!("=== Basic Inference Performance Test ===");
    
    let blockchain = Blockchain::new();
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);
    
    let start = Instant::now();
    let _enhanced_result = owl2_enhancer.apply_property_chain_inference(&[]);
    let enhanced_duration = start.elapsed();
    
    println!("Enhanced inference duration: {:?}", enhanced_duration);
    assert!(enhanced_duration < Duration::from_secs(5));
    
    println!("✅ Basic inference performance test passed!");
}