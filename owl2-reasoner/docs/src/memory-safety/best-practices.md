# Memory Safety Best Practices

This comprehensive guide covers recommended patterns, guidelines, and best practices for effective use of the memory safety system in the OWL2 Reasoner.

## Overview

Effective memory safety implementation requires understanding both the technical aspects and practical patterns that lead to robust, maintainable, and efficient code. This guide provides:

- **Recommended patterns** for common scenarios
- **Anti-patterns** to avoid
- **Performance optimization** guidelines
- **Testing strategies** for memory safety
- **Debugging and troubleshooting** approaches

## Core Principles

### 1. Principle of Least Privilege

Give tests only the memory they need, not the maximum they might use.

```rust
// ✅ Good: Minimal memory allocation
memory_safe_test!(test_simple_parsing, MemorySafeTestConfig::small(), {
    let parser = TurtleParser::new();
    let result = parser.parse_str("@prefix ex: <http://example.org/> . ex:Class a owl:Class .")?;
    assert!(result.is_ok());
});

// ❌ Bad: Excessive memory allocation
memory_safe_test!(test_simple_parsing_wasteful, MemorySafeTestConfig::stress(), {
    // Using 1GB for a simple parsing test
    let parser = TurtleParser::new();
    let result = parser.parse_str("@prefix ex: <http://example.org/> . ex:Class a owl:Class .")?;
    assert!(result.is_ok());
});
```

### 2. Principle of Explicit Resource Management

Always clean up resources explicitly, even though the system provides automatic cleanup.

```rust
// ✅ Good: Explicit resource cleanup
memory_safe_test!(test_resource_management, MemorySafeTestConfig::medium(), {
    let mut large_resources = Vec::new();
    
    // Allocate resources
    for i in 0..10 {
        let resource = create_expensive_resource(i)?;
        large_resources.push(resource);
    }
    
    // Use resources
    process_resources(&large_resources)?;
    
    // Explicit cleanup
    for resource in large_resources {
        cleanup_resource(resource)?;
    }
    large_resources.clear();
    
    // Verify cleanup
    let memory_after = get_memory_stats().total_usage;
    assert!(memory_after < 100 * 1024 * 1024); // Should be < 100MB
});

// ❌ Bad: Relying only on automatic cleanup
memory_safe_test!(test_implicit_cleanup, {
    let resources = create_many_resources(); // May not be cleaned up properly
    process_resources(&resources);
    // Resources dropped automatically, but cleanup may be incomplete
});
```

### 3. Principle of Test Isolation

Each test should be independent and not affect the memory state of other tests.

```rust
// ✅ Good: Test isolation with proper setup/teardown
memory_safe_test!(test_isolated_operations, MemorySafeTestConfig::medium(), {
    // Clear any existing state
    let _ = clear_global_iri_cache();
    let _ = force_memory_cleanup();
    
    // Perform test operations
    let ontology = create_test_ontology(100)?;
    let reasoner = SimpleReasoner::new(ontology);
    let result = reasoner.is_consistent()?;
    assert!(result);
    
    // Cleanup after test
    let _ = clear_global_iri_cache();
});

// ❌ Bad: Tests that affect global state
memory_safe_test!(test_global_state_pollution, {
    // Modifies global caches without cleanup
    let mut global_cache = get_global_cache();
    global_cache.insert_large_data(); // Affects other tests
});
```

## Testing Patterns

### 1. Progressive Memory Testing

Start with small configurations and progressively increase limits for complex tests.

```rust
// ✅ Good: Progressive testing approach
mod progressive_tests {
    use super::*;
    
    // Level 1: Basic functionality
    memory_safe_test!(test_basic_functionality, MemorySafeTestConfig::small(), {
        let class = Class::new("http://example.org/Person");
        assert!(class.iri().to_string().contains("Person"));
    });
    
    // Level 2: Integration testing
    memory_safe_test!(test_integration, MemorySafeTestConfig::medium(), {
        let ontology = create_medium_ontology(100)?;
        let reasoner = SimpleReasoner::new(ontology);
        assert!(reasoner.is_consistent()?);
    });
    
    // Level 3: Complex operations
    memory_safe_test!(test_complex_operations, MemorySafeTestConfig::large(), {
        let ontology = create_large_ontology(1000)?;
        let reasoner = SimpleReasoner::new(ontology);
        let classification = reasoner.classify()?;
        assert!(!classification.is_empty());
    });
    
    // Level 4: Stress testing
    memory_safe_stress_test!(test_stress_conditions, {
        let ontology = create_massive_ontology(10000)?;
        let reasoner = SimpleReasoner::new(ontology);
        
        // Should handle gracefully under stress
        let result = reasoner.classify();
        assert!(result.is_ok() || result.is_err()); // Either way, no crash
    });
}
```

### 2. Memory Boundary Testing

Test behavior near memory limits to ensure graceful degradation.

```rust
// ✅ Good: Testing memory boundaries
memory_safe_test!(test_memory_boundaries, MemorySafeTestConfig::medium(), {
    let limit = 128 * 1024 * 1024; // 128MB
    let mut allocations = Vec::new();
    let mut total_allocated = 0;
    
    // Allocate gradually to approach limit
    while total_allocated < limit * 0.8 { // Stay below 80% of limit
        let allocation: Vec<u8> = vec![0; 1024 * 1024]; // 1MB
        total_allocated += allocation.len();
        allocations.push(allocation);
        
        // Verify operations still work under pressure
        if allocations.len() % 10 == 0 {
            let ontology = create_small_ontology();
            let reasoner = SimpleReasoner::new(ontology);
            let _ = reasoner.is_consistent()?;
        }
    }
    
    // Should be able to operate near limits
    assert!(allocations.len() >= 80); // At least 80MB allocated
    
    // Test cleanup under pressure
    allocations.clear();
    let _ = force_memory_cleanup();
    
    // Memory should be freed
    let final_memory = get_memory_stats().total_usage;
    assert!(final_memory < limit * 0.3); // Should be below 30% after cleanup
});
```

### 3. Concurrent Memory Testing

Test memory safety under concurrent access patterns.

```rust
// ✅ Good: Concurrent memory safety testing
memory_safe_test!(test_concurrent_memory_safety, MemorySafeTestConfig::medium(), {
    use std::sync::{Arc, Barrier};
    use std::thread;
    
    let num_threads = 4;
    let barrier = Arc::new(Barrier::new(num_threads));
    let results = Arc::new(Mutex::new(Vec::new()));
    
    let mut handles = Vec::new();
    
    for thread_id in 0..num_threads {
        let barrier_clone = Arc::clone(&barrier);
        let results_clone = Arc::clone(&results);
        
        let handle = thread::spawn(move || {
            // Wait for all threads to start
            barrier_clone.wait();
            
            // Each thread performs memory-intensive operations
            let mut local_allocations = Vec::new();
            for i in 0..50 {
                let allocation: Vec<u8> = vec![(thread_id * 50 + i) as u8; 1024];
                local_allocations.push(allocation);
                
                // Perform reasoning operations
                if i % 10 == 0 {
                    let ontology = create_test_ontology(10);
                    let reasoner = SimpleReasoner::new(ontology);
                    let _ = reasoner.is_consistent();
                }
            }
            
            // Store results
            let mut results = results_clone.lock().unwrap();
            results.push((thread_id, local_allocations.len()));
            
            // Cleanup
            drop(local_allocations);
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify all threads completed successfully
    let results = results.lock().unwrap();
    assert_eq!(results.len(), num_threads);
    
    for (thread_id, allocation_count) in results.iter() {
        assert_eq!(*allocation_count, 50, "Thread {} completed with {} allocations", thread_id, allocation_count);
    }
});
```

## Performance Optimization Patterns

### 1. Batch Memory Operations

Group memory operations to reduce overhead.

```rust
// ✅ Good: Batch operations for efficiency
memory_safe_test!(test_batch_operations, MemorySafeTestConfig::large(), {
    let guard = TestGuard::with_config(MemorySafeTestConfig::large());
    
    // Define operations to perform
    let operations: Vec<Box<dyn Fn() -> Result<(), OwlError>>> = vec![
        Box::new(|| create_and_validate_ontology(100)),
        Box::new(|| create_and_validate_ontology(200)),
        Box::new(|| create_and_validate_ontology(300)),
        Box::new(|| create_and_validate_ontology(400)),
        Box::new(|| create_and_validate_ontology(500)),
    ];
    
    // Execute operations with batch memory checking
    let results = guard.batch_operations(&operations);
    
    // Verify all operations succeeded
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Operation {} failed: {:?}", i, result);
    }
    
    let report = guard.finish();
    report.assert_acceptable();
});
```

### 2. Adaptive Memory Management

Adjust memory usage based on current pressure.

```rust
// ✅ Good: Adaptive memory management
memory_safe_test!(test_adaptive_memory_management, MemorySafeTestConfig::medium(), {
    let mut adaptive_allocator = AdaptiveMemoryAllocator::new();
    
    for phase in 0..5 {
        // Adjust allocation strategy based on current memory pressure
        let current_pressure = get_memory_pressure_level();
        
        let allocation_size = if current_pressure > 0.8 {
            // High pressure - allocate less
            1024 * 1024  // 1MB
        } else if current_pressure > 0.5 {
            // Medium pressure - moderate allocation
            2 * 1024 * 1024  // 2MB
        } else {
            // Low pressure - can allocate more
            5 * 1024 * 1024  // 5MB
        };
        
        // Allocate adaptively
        let allocation = adaptive_allocator.allocate(allocation_size);
        
        // Perform operations
        let ontology = create_test_ontology(phase * 100);
        let reasoner = SimpleReasoner::new(ontology);
        let _ = reasoner.is_consistent()?;
        
        // Adaptive cleanup
        if current_pressure > 0.7 {
            adaptive_allocator.cleanup_aggressive();
        } else {
            adaptive_allocator.cleanup_moderate();
        }
        
        println!("Phase {}: Allocated {}MB at {:.1}% pressure", 
                 phase + 1, 
                 allocation_size / 1024 / 1024,
                 current_pressure * 100.0);
    }
    
    // Final cleanup
    adaptive_allocator.cleanup_all();
});
```

### 3. Memory Pool Reuse

Reuse memory pools to reduce allocation overhead.

```rust
// ✅ Good: Memory pool pattern
memory_safe_test!(test_memory_pool_reuse, MemorySafeTestConfig::medium(), {
    let mut memory_pool = MemoryPool::new(10 * 1024 * 1024); // 10MB pool
    
    for iteration in 0..100 {
        // Get buffer from pool
        let mut buffer = memory_pool.get_buffer(1024 * 1024); // 1MB buffer
        
        // Use buffer for operations
        fill_buffer_with_test_data(&mut buffer, iteration);
        
        // Perform reasoning operations
        let ontology = create_ontology_from_buffer(&buffer)?;
        let reasoner = SimpleReasoner::new(ontology);
        let result = reasoner.is_consistent()?;
        assert!(result);
        
        // Return buffer to pool
        memory_pool.return_buffer(buffer);
        
        // Verify pool efficiency
        if iteration % 20 == 0 {
            let stats = memory_pool.get_stats();
            println!("Iteration {}: Pool hit rate: {:.1}%", 
                     iteration, 
                     stats.hit_rate() * 100.0);
        }
    }
    
    // Verify pool performance
    let final_stats = memory_pool.get_stats();
    assert!(final_stats.hit_rate() > 0.8, "Pool hit rate should be > 80%");
});
```

## Anti-Patterns to Avoid

### 1. Memory Leaks in Tests

```rust
// ❌ Bad: Memory leak pattern
memory_safe_test!(test_memory_leak, {
    let mut leaks = Vec::new();
    
    for i in 0..1000 {
        let large_allocation: Vec<u8> = vec![i as u8; 1024 * 1024]; // 1MB each
        leaks.push(large_allocation); // Never cleaned up
        
        // Test logic here
    }
    
    // Function ends, but large allocations persist in global state
});

// ✅ Good: Proper cleanup pattern
memory_safe_test!(test_proper_cleanup, MemorySafeTestConfig::large(), {
    let mut allocations = Vec::new();
    
    for i in 0..100 {
        let allocation: Vec<u8> = vec![i as u8; 1024 * 1024]; // 1MB each
        allocations.push(allocation);
        
        // Test logic here
    }
    
    // Explicit cleanup
    for allocation in allocations {
        drop(allocation); // Explicit drop
    }
    allocations.clear();
    
    // Force cleanup
    let _ = force_memory_cleanup();
});
```

### 2. Excessive Memory Allocations

```rust
// ❌ Bad: Wasteful memory usage
memory_safe_test!(test_wasteful_allocation, MemorySafeTestConfig::large(), {
    // Creating unnecessary large allocations
    let unnecessary_data: Vec<u8> = vec![0; 500 * 1024 * 1024]; // 500MB
    
    // Only using small portion
    let small_slice = &unnecessary_data[..1024];
    assert_eq!(small_slice.len(), 1024);
    
    // Rest of memory wasted
});

// ✅ Good: Efficient memory usage
memory_safe_test!(test_efficient_allocation, MemorySafeTestConfig::small(), {
    // Allocate only what's needed
    let necessary_data: Vec<u8> = vec![0; 1024]; // 1KB
    assert_eq!(necessary_data.len(), 1024);
    
    // No wasted memory
});
```

### 3. Ignoring Memory Pressure

```rust
// ❌ Bad: Ignoring memory pressure
memory_safe_test!(test_ignoring_pressure, {
    let mut allocations = Vec::new();
    
    // Keep allocating without checking pressure
    loop {
        let allocation: Vec<u8> = vec![0; 10 * 1024 * 1024]; // 10MB
        allocations.push(allocation);
        
        // No pressure checking - may cause OOM
    }
});

// ✅ Good: Responding to memory pressure
memory_safe_test!(test_responding_to_pressure, MemorySafeTestConfig::medium(), {
    let mut allocations = Vec::new();
    
    while get_memory_pressure_level() < 0.8 {
        let allocation: Vec<u8> = vec![0; 5 * 1024 * 1024]; // 5MB
        allocations.push(allocation);
        
        // Check pressure regularly
        if get_memory_pressure_level() > 0.6 {
            // Start cleanup when pressure increases
            if allocations.len() > 10 {
                allocations.drain(0..5); // Remove some allocations
                let _ = force_memory_cleanup();
            }
        }
    }
    
    // Should exit gracefully before OOM
    assert!(get_memory_pressure_level() < 0.9);
});
```

## Debugging and Troubleshooting

### 1. Verbose Memory Reporting

Enable verbose reporting for detailed memory tracking.

```rust
// ✅ Good: Use verbose reporting for debugging
memory_safe_test!(test_with_verbose_reporting, {
    // Enable verbose mode
    std::env::set_var("OWL2_TEST_VERBOSE", "1");
    
    let mut allocations = Vec::new();
    
    for i in 0..10 {
        let allocation: Vec<u8> = vec![i as u8; 5 * 1024 * 1024]; // 5MB
        allocations.push(allocation);
        
        // Report memory usage at each step
        let stats = get_memory_stats();
        println!("After allocation {}: {:.1}MB, {:.1}% pressure", 
                 i + 1,
                 stats.total_usage as f64 / 1024.0 / 1024.0,
                 stats.pressure_level * 100.0);
    }
    
    // Cleanup and report final state
    allocations.clear();
    let final_stats = get_memory_stats();
    println!("Final state: {:.1}MB, {:.1}% pressure", 
             final_stats.total_usage as f64 / 1024.0 / 1024.0,
             final_stats.pressure_level * 100.0);
});
```

### 2. Memory Profiling

Use profiling to identify memory hotspots.

```rust
// ✅ Good: Memory profiling pattern
memory_safe_test!(test_memory_profiling, MemorySafeTestConfig::large(), {
    let profile = MemoryProfiler::new();
    
    profile.start();
    
    // Perform operations to profile
    let operations = vec![
        ("create_ontology", || create_large_ontology(1000)),
        ("reasoning", || {
            let ontology = create_large_ontology(1000);
            let reasoner = SimpleReasoner::new(ontology);
            reasoner.classify()
        }),
        ("cache_operations", || perform_cache_operations(1000)),
    ];
    
    for (name, operation) in operations {
        let operation_profile = profile.profile_operation(name, operation);
        println!("Operation {}: {:.1}MB peak, {:.2}ms", 
                 name,
                 operation_profile.peak_memory_mb,
                 operation_profile.duration_ms);
    }
    
    let overall_profile = profile.stop();
    println!("Overall: {:.1}MB peak, {:.2}ms total", 
             overall_profile.peak_memory_mb,
             overall_profile.duration_ms);
});
```

### 3. Memory Leak Detection

Use built-in leak detection to identify issues.

```rust
// ✅ Good: Proactive leak detection
memory_safe_test!(test_leak_detection, MemorySafeTestConfig::medium(), {
    // Get baseline leak report
    let baseline_report = detect_memory_leaks();
    println!("Baseline efficiency: {:.2}", baseline_report.memory_efficiency_score);
    
    // Perform operations that might leak
    let mut potential_leaks = Vec::new();
    
    for i in 0..50 {
        // Create allocations that might not be cleaned up properly
        let allocation = Box::new(vec![i as u8; 1024 * 1024]); // 1MB
        potential_leaks.push(allocation);
        
        // Simulate some operations
        if i % 10 == 0 {
            // Intentionally "forget" some allocations to test leak detection
            if i % 20 == 0 {
                std::mem::forget(potential_leaks.pop().unwrap());
            }
        }
    }
    
    // Check for leaks after operations
    let after_report = detect_memory_leaks();
    println!("After operations efficiency: {:.2}", after_report.memory_efficiency_score);
    
    // Cleanup remaining allocations
    drop(potential_leaks);
    let _ = force_memory_cleanup();
    
    // Final leak check
    let final_report = detect_memory_leaks();
    println!("Final efficiency: {:.2}", final_report.memory_efficiency_score);
    
    // Verify no significant leaks
    assert!(final_report.memory_efficiency_score > 0.8, 
            "Memory efficiency should be > 80% after cleanup");
    
    // Report any remaining potential leaks
    if !final_report.potential_leaks.is_empty() {
        println!("Potential leaks detected:");
        for leak in &final_report.potential_leaks {
            println!("  - {}", leak);
        }
    }
});
```

## Integration Patterns

### 1. CI/CD Integration

Integrate memory safety testing into continuous integration pipelines.

```yaml
# GitHub Actions example
name: Memory-Safe CI Pipeline

on: [push, pull_request]

jobs:
  memory-safe-tests:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        
    - name: Run memory-safe unit tests
      run: |
        export OWL2_TEST_VERBOSE=1
        export OWL2_TEST_MEMORY_LIMIT_MB=256
        cargo test unit_tests --lib
        
    - name: Run memory-safe integration tests
      run: |
        export OWL2_TEST_MEMORY_LIMIT_MB=512
        cargo test integration_tests --lib
        
    - name: Run stress tests
      run: |
        export OWL2_TEST_MEMORY_LIMIT_MB=1024
        export OWL2_TEST_VERBOSE=1
        cargo test stress_tests --lib
        
    - name: Memory leak detection
      run: |
        cargo test memory_leak_detection --lib
        
    - name: Performance validation
      run: |
        cargo bench --bench memory_safety_benchmarks
        
    - name: Generate memory report
      run: |
        cargo test --lib | grep "Memory Report" > memory_report.txt
        cat memory_report.txt
```

### 2. Development Workflow Integration

Integrate memory safety into daily development practices.

```rust
// Development helper functions
pub fn dev_setup_memory_safety() {
    std::env::set_var("OWL2_TEST_VERBOSE", "1");
    std::env::set_var("OWL2_TEST_MEMORY_LIMIT_MB", "512");
}

pub fn run_memory_safe_tests() -> Result<(), Box<dyn std::error::Error>> {
    dev_setup_memory_safety();
    
    // Run critical tests with memory safety
    test_basic_functionality()?;
    test_integration_scenarios()?;
    test_performance_regression()?;
    test_memory_leak_prevention()?;
    
    Ok(())
}

// Usage in development
#[cfg(debug_assertions)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running development memory-safe tests...");
    run_memory_safe_tests()?;
    println!("All memory-safe tests passed!");
    Ok(())
}
```

### 3. Production Monitoring

Monitor memory safety in production environments.

```rust
// Production memory monitoring
pub struct ProductionMemoryMonitor {
    guard: TestMemoryGuard,
    alert_threshold: f64,
}

impl ProductionMemoryMonitor {
    pub fn new(alert_threshold_mb: usize) -> Self {
        let config = TestMemoryConfig {
            max_memory_bytes: alert_threshold_mb * 1024 * 1024,
            auto_cleanup: true,
            fail_on_limit_exceeded: false, // Don't fail in production
            warn_threshold_percent: 0.8,
            check_interval: Duration::from_secs(30),
        };
        
        let guard = TestMemoryGuard::with_config(config);
        guard.start_monitoring();
        
        Self {
            guard,
            alert_threshold: alert_threshold_mb as f64,
        }
    }
    
    pub fn check_and_alert(&self) -> Option<String> {
        let usage_percent = self.guard.memory_usage_percent();
        
        if usage_percent > 80.0 {
            Some(format!("Memory usage critical: {:.1}%", usage_percent))
        } else if usage_percent > 60.0 {
            Some(format!("Memory usage high: {:.1}%", usage_percent))
        } else {
            None
        }
    }
    
    pub fn generate_report(&self) -> String {
        let report = self.guard.stop_monitoring();
        format!("Production Memory Report: {:.1}MB used, {:.1}% of limit",
                report.end_memory as f64 / 1024.0 / 1024.0,
                (report.end_memory as f64 / report.max_memory_bytes as f64) * 100.0)
    }
}
```

## Checklist for Memory Safety

### Pre-Test Checklist

- [ ] **Choose appropriate memory configuration** for test complexity
- [ ] **Enable verbose reporting** for debugging complex tests
- [ ] **Plan explicit cleanup** for allocated resources
- [ ] **Consider concurrent access** patterns
- [ ] **Set reasonable timeouts** for long-running tests

### Post-Test Checklist

- [ ] **Verify memory usage** is within expected bounds
- [ ] **Check for memory leaks** using leak detection
- [ ] **Review performance impact** of memory safety measures
- [ ] **Validate cleanup effectiveness** through memory reports
- [ ] **Document any memory-related issues** found

### Code Review Checklist

- [ ] **Memory-safe test macros** used correctly
- [ ] **Appropriate memory limits** chosen for test type
- [ ] **Resource cleanup** handled explicitly
- [ ] **Error handling** includes memory-related failures
- [ ] **Performance impact** considered and documented

## Conclusion

Effective memory safety implementation requires attention to detail, understanding of system behavior, and adherence to proven patterns. By following these best practices:

1. **Choose appropriate configurations** for different test types
2. **Implement explicit resource management** and cleanup
3. **Use progressive testing** approaches
4. **Monitor and respond to memory pressure**
5. **Integrate memory safety** into development workflows
6. **Validate and debug** memory-related issues systematically

The memory safety system provides robust protection when used correctly, ensuring reliable test execution and system stability while maintaining excellent performance characteristics.

---

**Previous**: [Performance Impact Analysis](performance-impact.md) - Learn about performance characteristics and optimization.