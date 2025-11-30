# Memory-Safe Testing

This comprehensive guide covers memory-safe testing patterns, best practices, and implementation details for the OWL2 Reasoner testing framework.

## Overview

Memory-safe testing is a critical aspect of developing robust OWL2 reasoning systems. Traditional testing approaches can lead to out-of-memory errors, system hangs, and unreliable test results. Our memory-safe testing framework addresses these challenges by providing:

- **Protected test execution** with automatic memory monitoring
- **Configurable memory limits** for different test types
- **Graceful failure handling** before system instability
- **Comprehensive reporting** with memory usage analysis
- **Performance validation** ensuring minimal overhead

## Quick Start

### Basic Memory-Safe Test

```rust
use owl2_reasoner::test_helpers::memory_safe_test;

memory_safe_test!(test_basic_reasoning, {
    let mut ontology = Ontology::new();
    
    // Add classes and axioms
    let person_class = Class::new("http://example.org/Person");
    ontology.add_class(person_class.clone())?;
    
    // Create reasoner and test
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent()?;
    
    assert!(is_consistent);
    Ok(())
});
```

### Test with Custom Memory Limits

```rust
use owl2_reasoner::test_helpers::{memory_safe_test, MemorySafeTestConfig};

memory_safe_test!(test_large_ontology, MemorySafeTestConfig::large(), {
    // This test can use up to 512MB memory
    let mut ontology = create_large_ontology(10000)?;
    
    let reasoner = SimpleReasoner::new(ontology);
    let result = reasoner.classify()?;
    
    assert!(!result.is_empty());
    Ok(())
});
```

### Stress Test

```rust
use owl2_reasoner::test_helpers::memory_safe_stress_test;

memory_safe_stress_test!(test_memory_pressure, {
    // Stress test with relaxed limits (1GB, warnings only)
    let mut data = Vec::new();
    
    for i in 0..1000 {
        // Allocate memory gradually
        let chunk: Vec<u8> = vec![i as u8; 1024 * 1024]; // 1MB
        data.push(chunk);
        
        // Test reasoning operations
        if i % 100 == 0 {
            let ontology = create_test_ontology(i);
            let reasoner = SimpleReasoner::new(ontology);
            let _ = reasoner.is_consistent()?;
        }
    }
    
    assert_eq!(data.len(), 1000);
});
```

## Test Types and Configurations

### Memory Configurations

| Configuration | Memory Limit | Cache Limit | Fail on Limit | Use Case |
|---------------|--------------|-------------|---------------|----------|
| `small()` | 64MB | 100 entries | Yes | Unit tests, simple operations |
| `medium()` | 128MB | 300 entries | Yes | Integration tests |
| `large()` | 512MB | 1000 entries | Yes | Complex operations |
| `stress()` | 1GB | 2000 entries | No | Stress testing, benchmarks |

### Choosing the Right Configuration

#### Small Configuration
```rust
memory_safe_test!(test_parser_functionality, MemorySafeTestConfig::small(), {
    // Simple parser tests with minimal memory usage
    let parser = TurtleParser::new();
    let result = parser.parse_str("@prefix ex: <http://example.org/> . ex:Class a owl:Class .")?;
    assert!(result.is_ok());
});
```

#### Medium Configuration
```rust
memory_safe_test!(test_reasoning_integration, MemorySafeTestConfig::medium(), {
    // Integration tests with moderate memory usage
    let ontology = load_test_ontology("family.ttl")?;
    let reasoner = SimpleReasoner::new(ontology);
    
    let classification = reasoner.classify()?;
    assert!(!classification.is_empty());
});
```

#### Large Configuration
```rust
memory_safe_test!(test_large_scale_reasoning, MemorySafeTestConfig::large(), {
    // Complex reasoning with significant memory requirements
    let ontology = load_large_ontology("biomedical.owl")?;
    let reasoner = SimpleReasoner::new(ontology);
    
    // Perform complex reasoning operations
    let consistency = reasoner.is_consistent()?;
    let classification = reasoner.classify()?;
    
    assert!(consistency);
    assert!(!classification.is_empty());
});
```

## Advanced Testing Patterns

### Performance Testing

```rust
use owl2_reasoner::test_helpers::memory_safe_bench_test;

memory_safe_bench_test!(test_reasoning_performance, 100, {
    // Benchmark-style test with 100 iterations
    let ontology = create_benchmark_ontology();
    let reasoner = SimpleReasoner::new(ontology);
    
    // Measure reasoning performance
    let start = std::time::Instant::now();
    let result = reasoner.is_consistent()?;
    let duration = start.elapsed();
    
    // Performance assertions
    assert!(duration.as_millis() < 100); // Should complete in < 100ms
    assert!(result);
});
```

### Concurrent Testing

```rust
memory_safe_test!(test_concurrent_reasoning, MemorySafeTestConfig::medium(), {
    use std::sync::Arc;
    use std::thread;
    
    let ontology = Arc::new(create_test_ontology(1000)?);
    let mut handles = Vec::new();
    
    // Spawn multiple reasoning threads
    for i in 0..4 {
        let ontology_clone = Arc::clone(&ontology);
        let handle = thread::spawn(move || {
            let reasoner = SimpleReasoner::new((*ontology_clone).clone());
            let result = reasoner.is_consistent();
            (i, result)
        });
        handles.push(handle);
    }
    
    // Collect results
    for handle in handles {
        let (thread_id, result) = handle.join().unwrap();
        assert!(result.is_ok(), "Thread {} failed: {:?}", thread_id, result);
    }
});
```

### Memory Leak Testing

```rust
memory_safe_test!(test_memory_leak_prevention, MemorySafeTestConfig::large(), {
    let initial_memory = get_memory_stats().total_usage;
    
    // Perform operations that could leak memory
    for _ in 0..100 {
        let ontology = create_temporary_ontology(100)?;
        let reasoner = SimpleReasoner::new(ontology);
        let _ = reasoner.is_consistent()?;
        
        // Explicit cleanup
        drop(reasoner);
    }
    
    // Force garbage collection
    let _ = force_memory_cleanup();
    
    let final_memory = get_memory_stats().total_usage;
    let memory_increase = final_memory.saturating_sub(initial_memory);
    
    // Assert no significant memory leak
    assert!(memory_increase < 10 * 1024 * 1024, // < 10MB increase
            "Potential memory leak detected: {} bytes increase", memory_increase);
});
```

## Test Organization

### Structuring Memory-Safe Tests

```rust
#[cfg(test)]
mod reasoning_tests {
    use super::*;
    use owl2_reasoner::test_helpers::*;
    
    mod consistency_tests {
        use super::*;
        
        memory_safe_test!(test_simple_consistency, {
            let ontology = create_simple_consistent_ontology();
            let reasoner = SimpleReasoner::new(ontology);
            assert!(reasoner.is_consistent()?);
        });
        
        memory_safe_test!(test_inconsistency_detection, {
            let ontology = create_inconsistent_ontology();
            let reasoner = SimpleReasoner::new(ontology);
            assert!(!reasoner.is_consistent()?);
        });
    }
    
    mod classification_tests {
        use super::*;
        
        memory_safe_test!(test_class_hierarchy, MemorySafeTestConfig::medium(), {
            let ontology = create_class_hierarchy_ontology();
            let reasoner = SimpleReasoner::new(ontology);
            let classification = reasoner.classify()?;
            
            assert!(!classification.is_empty());
        });
    }
}
```

### Test Data Management

```rust
// Test data factories with memory efficiency
fn create_test_ontology(size: usize) -> Result<Ontology, OwlError> {
    let mut ontology = Ontology::new();
    
    for i in 0..size {
        let iri = format!("http://example.org/class{}", i);
        let class = Class::new(iri);
        ontology.add_class(class)?;
    }
    
    Ok(ontology)
}

// Memory-efficient test data cleanup
fn cleanup_test_data<T>(data: T) {
    drop(data);
    
    // Optional: Force cleanup for large allocations
    if std::mem::size_of::<T>() > 1024 * 1024 {
        let _ = force_memory_cleanup();
    }
}
```

## Error Handling and Debugging

### Handling Memory Errors

```rust
memory_safe_test!(test_memory_error_handling, MemorySafeTestConfig::small(), {
    let result = std::panic::catch_unwind(|| {
        // This might exceed memory limits
        let huge_data: Vec<u8> = vec![0; 100 * 1024 * 1024]; // 100MB
        assert_eq!(huge_data.len(), 100 * 1024 * 1024);
    });
    
    match result {
        Ok(_) => {
            // Test completed successfully
            println!("Test completed within memory limits");
        }
        Err(panic_info) => {
            // Handle panic (could be memory-related)
            if let Some(s) = panic_info.downcast_ref::<String>() {
                if s.contains("Memory limit exceeded") {
                    println!("Test correctly detected memory limit exceeded");
                } else {
                    panic!("Unexpected panic: {}", s);
                }
            }
        }
    }
});
```

### Verbose Memory Reporting

```rust
// Enable verbose reporting for debugging
memory_safe_test!(test_verbose_memory_tracking, {
    // Set verbose mode
    std::env::set_var("OWL2_TEST_VERBOSE", "1");
    
    let mut allocations = Vec::new();
    
    // Allocate memory in stages to see reporting
    for i in 0..10 {
        let allocation: Vec<u8> = vec![i as u8; 5 * 1024 * 1024]; // 5MB
        allocations.push(allocation);
        
        println!("After allocation {}: {:.1}MB", 
                 i + 1, 
                 get_memory_stats().total_usage as f64 / 1024.0 / 1024.0);
    }
    
    // Cleanup
    allocations.clear();
});
```

## Integration with CI/CD

### GitHub Actions Configuration

```yaml
name: Memory-Safe Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        
    - name: Run memory-safe tests
      run: |
        export OWL2_TEST_VERBOSE=1
        export OWL2_TEST_MEMORY_LIMIT_MB=512
        cargo test --lib --all-features
        
    - name: Run memory safety benchmarks
      run: |
        cargo bench --bench memory_safety_benchmarks
        
    - name: Check for memory leaks
      run: |
        cargo test memory_leak_detection --lib
```

### Docker Configuration

```dockerfile
FROM rust:1.70

# Set memory limits for container
ENV OWL2_TEST_MEMORY_LIMIT_MB=512
ENV OWL2_TEST_CACHE_LIMIT=1000

WORKDIR /app
COPY . .

# Run memory-safe tests
RUN cargo test --lib --all-features

# Run benchmarks
RUN cargo bench --bench memory_safety_benchmarks
```

## Best Practices

### 1. Choose Appropriate Memory Limits

```rust
// ✅ Good: Appropriate limits for test complexity
memory_safe_test!(test_simple_parsing, MemorySafeTestConfig::small(), {
    let parser = TurtleParser::new();
    let result = parser.parse_str("@prefix ex: <http://example.org/> . ex:Class a owl:Class .")?;
    assert!(result.is_ok());
});

// ❌ Bad: Excessive limits for simple test
memory_safe_test!(test_simple_parsing_wasteful, MemorySafeTestConfig::stress(), {
    // Wasting 1GB for a simple parsing test
    let parser = TurtleParser::new();
    let result = parser.parse_str("@prefix ex: <http://example.org/> . ex:Class a owl:Class .")?;
    assert!(result.is_ok());
});
```

### 2. Clean Up Resources

```rust
memory_safe_test!(test_resource_cleanup, MemorySafeTestConfig::medium(), {
    let mut large_resources = Vec::new();
    
    // Use resources
    for i in 0..10 {
        let resource = create_expensive_resource(i);
        large_resources.push(resource);
    }
    
    // Process resources
    process_resources(&large_resources);
    
    // Explicit cleanup
    for resource in large_resources {
        cleanup_resource(resource);
    }
    large_resources.clear();
    
    // Verify cleanup
    let memory_after = get_memory_stats().total_usage;
    assert!(memory_after < 100 * 1024 * 1024); // Should be < 100MB
});
```

### 3. Test Memory Edge Cases

```rust
memory_safe_test!(test_memory_boundary_conditions, MemorySafeTestConfig::medium(), {
    // Test near memory limits
    let limit = 64 * 1024 * 1024; // 64MB
    let mut total_allocated = 0;
    
    let mut allocations = Vec::new();
    while total_allocated < limit * 0.9 { // Allocate up to 90% of limit
        let allocation: Vec<u8> = vec![0; 1024 * 1024]; // 1MB
        total_allocated += allocation.len();
        allocations.push(allocation);
    }
    
    // Should still be able to perform operations
    let ontology = create_small_ontology();
    let reasoner = SimpleReasoner::new(ontology);
    let _ = reasoner.is_consistent()?;
    
    // Test behavior when approaching limit
    let additional_allocation: Vec<u8> = vec![0; 10 * 1024 * 1024]; // 10MB
    allocations.push(additional_allocation);
    
    // Should handle gracefully
    assert!(allocations.len() > 50);
});
```

### 4. Use Appropriate Test Types

```rust
// Unit tests - use small limits
mod unit_tests {
    memory_safe_test!(test_entity_creation, MemorySafeTestConfig::small(), {
        let class = Class::new("http://example.org/Class");
        assert!(class.iri().to_string().contains("Class"));
    });
}

// Integration tests - use medium limits
mod integration_tests {
    memory_safe_test!(test_parser_reasoning_integration, MemorySafeTestConfig::medium(), {
        let ontology = parse_ontology_from_file("test.ttl")?;
        let reasoner = SimpleReasoner::new(ontology);
        assert!(reasoner.is_consistent()?);
    });
}

// Stress tests - use stress configuration
mod stress_tests {
    memory_safe_stress_test!(test_large_ontology_processing, {
        let ontology = create_large_ontology(10000)?;
        let reasoner = SimpleReasoner::new(ontology);
        
        let start = std::time::Instant::now();
        let classification = reasoner.classify()?;
        let duration = start.elapsed();
        
        assert!(!classification.is_empty());
        assert!(duration.as_secs() < 30); // Should complete in < 30 seconds
    });
}
```

## Troubleshooting

### Common Issues

#### 1. Test Fails Due to Memory Limit

**Problem**: Test fails with "Memory limit exceeded" error.

**Solution**: 
- Increase memory limit for the test
- Optimize test to use less memory
- Split test into smaller components

```rust
// Increase memory limit
memory_safe_test!(my_test, MemorySafeTestConfig::large(), {
    // Test implementation
});

// Or optimize for lower memory usage
memory_safe_test!(my_optimized_test, MemorySafeTestConfig::medium(), {
    // Process data in chunks
    for chunk in data.chunks(1000) {
        process_chunk(chunk)?;
    }
});
```

#### 2. Test Performance Degradation

**Problem**: Tests run slowly with memory safety enabled.

**Solution**:
- Use appropriate memory configurations
- Optimize test data size
- Use benchmark tests for performance measurement

```rust
// Use appropriate configuration
memory_safe_test!(test_performance, MemorySafeTestConfig::medium(), {
    // Balanced test with reasonable memory limits
});

// Use benchmarks for performance measurement
memory_safe_bench_test!(performance_benchmark, 100, {
    // Benchmark-style performance test
});
```

#### 3. Memory Leak Detection

**Problem**: Tests show memory leaks in continuous integration.

**Solution**:
- Add explicit cleanup in tests
- Use leak detection utilities
- Monitor memory usage patterns

```rust
memory_safe_test!(test_with_explicit_cleanup, {
    let mut resources = Vec::new();
    
    // Use resources
    resources.push(create_resource());
    
    // Explicit cleanup
    for resource in resources {
        cleanup_resource(resource);
    }
    resources.clear();
    
    // Verify no memory leak
    let leak_report = detect_memory_leaks();
    assert!(leak_report.potential_leaks.is_empty());
});
```

---

**Next**: [Memory Guard Configuration](configuration.md) - Learn about configuration options and setup.