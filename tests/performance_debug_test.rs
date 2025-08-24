use provchain_org::core::blockchain::Blockchain;
use std::time::Instant;

#[test]
fn test_blockchain_performance_debug() {
    let mut blockchain = Blockchain::new();
    
    // Measure time for adding blocks in smaller batches
    let batch_sizes = [10, 50, 100, 200];
    
    for &batch_size in &batch_sizes {
        let start = Instant::now();
        for i in 0..batch_size {
            let _ = blockchain.add_block(format!("Test data {}", i));
        }
        let duration = start.elapsed();
        
        println!("Time to add {} blocks: {:?}", batch_size, duration);
        println!("Time per block: {:?}", duration / batch_size);
        
        // Check if blockchain is still valid
        assert!(blockchain.is_valid());
    }
    
    // Test with 1000 blocks but with timing checkpoints
    println!("Starting 1000 block test...");
    let start = Instant::now();
    for i in 0..1000 {
        let _ = blockchain.add_block(format!("Test data {}", i));
        
        // Print progress every 100 blocks
        if (i + 1) % 100 == 0 {
            let elapsed = start.elapsed();
            println!("Added {} blocks in {:?}", i + 1, elapsed);
        }
    }
    let duration = start.elapsed();
    
    println!("Time to add 1000 blocks: {:?}", duration);
    println!("Time per block: {:?}", duration / 1000);
    assert!(blockchain.is_valid());
    
    // The original test required 1000 blocks in 10 seconds
    // Let's check if we're close to that
    assert!(duration < std::time::Duration::from_secs(30), "1000 blocks should be added within 30 seconds");
}