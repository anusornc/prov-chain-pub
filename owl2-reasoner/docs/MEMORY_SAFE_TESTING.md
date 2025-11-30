# Memory-Safe Testing Guide

This guide provides comprehensive information about writing memory-safe tests for the OWL2 Reasoner that prevent out-of-memory errors and system hangs.

## Overview

The OWL2 Reasoner includes a sophisticated memory management system for testing that:

- **Prevents OOM errors**: Tests fail gracefully before consuming all system memory
- **Provides isolation**: Each test runs in a controlled memory environment
- **Enables monitoring**: Real-time memory tracking and reporting
- **Ensures cleanup**: Automatic cache cleanup between tests

## Quick Start

### Basic Memory-Safe Test

```rust
use owl2_reasoner::test_helpers::{memory_safe_test, MemorySafeTestConfig};

memory_safe_test!(my_test, {
    // Your test code here
    let ontology = Ontology::new();
    assert!(ontology.is_empty());
});
```

### Test with Custom Memory Limits

```rust
use owl2_reasoner::test_helpers::{memory_safe_test, MemorySafeTestConfig};

memory_safe_test!(my_limited_test, MemorySafeTestConfig::small(), {
    // Test with strict memory limits (64MB)
    let data = vec![1u8; 1024]; // 1KB
    assert_eq!(data.len(), 1024);
});
```

### Stress Test

```rust
use owl2_reasoner::test_helpers::memory_safe_stress_test;

memory_safe_stress_test!(my_stress_test, {
    // Test that can use more memory but won't cause OOM
    let large_data = vec![1u8; 50 * 1024 * 1024]; // 50MB
    assert_eq!(large_data.len(), 50 * 1024 * 1024);
});
```

## Memory Configurations

### Predefined Configurations

| Configuration | Memory Limit | Cache Limit | Use Case |
|---------------|--------------|-------------|----------|
| `small()` | 64MB | 100 entries | Unit tests, simple operations |
| `medium()` | 128MB | 300 entries | Integration tests |
| `large()` | 512MB | 1000 entries | Complex operations |
| `stress()` | 1GB | 2000 entries | Stress testing, benchmarks |

### Custom Configuration

```rust
use owl2_reasoner::test_helpers::MemorySafeTestConfig;

let config = MemorySafeTestConfig {
    max_memory_mb: 256,
    max_cache_size: 500,
    fail_on_limit: true,
    verbose: false,
};
```

## Test Types

### 1. Standard Tests

Use for regular unit tests and integration tests:

```rust
memory_safe_test!(test_parsing, {
    let parser = TurtleParser::new();
    let result = parser.parse_str("@prefix ex: <http://example.org/> . ex:Class a owl:Class .");
    assert!(result.is_ok());
});
```

### 2. Stress Tests

Use for testing with large datasets:

```rust
memory_safe_stress_test!(test_large_ontology, {
    let mut content = String::new();
    for i in 0..10000 {
        content.push_str(&format!("ex:Class{} a owl:Class .\n", i));
    }
    
    let parser = TurtleParser::new();
    let result = parser.parse_str(&content);
    assert!(result.is_ok());
});
```

### 3. Benchmark Tests

Use for performance testing with multiple iterations:

```rust
memory_safe_bench_test!(test_performance, 100, {
    let parser = TurtleParser::new();
    let result = parser.parse_str("@prefix ex: <http://example.org/> . ex:Class a owl:Class .");
    assert!(result.is_ok());
});
```

## Manual Memory Management

### Using TestGuard Directly

```rust
use owl2_reasoner::test_helpers::{TestGuard, MemorySafeTestConfig};

let guard = TestGuard::with_config(MemorySafeTestConfig::medium());

// Your test code here
let result = some_operation();

// Generate report and assert acceptable usage
let report = guard.finish();
report.assert_acceptable();
```

### Utility Functions

```rust
use owl2_reasoner::test_helpers;

// Cache test with small limits
test_helpers::create_cache_test(|| {
    // Cache-related test code
});

// Ontology test with medium limits
test_helpers::create_ontology_test(|| {
    // Ontology-related test code
});

// Stress test with lenient limits
test_helpers::create_stress_test(|| {
    // Stress test code
});
```

## Memory Monitoring

### Memory Reports

Each test generates a detailed memory report:

```
=== Test Memory Report ===
Memory Limits:
  Max Memory: 256 MB
  Max Cache Size: 500

Memory Usage:
  Start: 12.3 MB
  End: 15.7 MB
  Peak: 16.1 MB
  Delta: 3.4 MB
  Usage: 6.1% of limit

Cache Statistics:
  Size: 150 entries
  Hit Rate: 78.5%
  Evictions: 5
  Cleanups Performed: 0
========================
```

### Verbose Output

Enable verbose memory reporting:

```rust
let config = MemorySafeTestConfig {
    verbose: true,
    ..Default::default()
};
```

Or set environment variable:
```bash
OWL2_TEST_VERBOSE=1 cargo test
```

## Best Practices

### 1. Choose Appropriate Memory Limits

- **Unit tests**: Use `MemorySafeTestConfig::small()`
- **Integration tests**: Use `MemorySafeTestConfig::medium()`
- **Performance tests**: Use `MemorySafeTestConfig::large()`
- **Stress tests**: Use `MemorySafeTestConfig::stress()`

### 2. Memory-Intensive Test Management

#### Running Tests Safely by Category

For systems with limited memory, run tests selectively to avoid out-of-memory errors:

```bash
# ✅ SAFE: Core functionality tests (recommended for most systems)
cargo test reasoning --lib        # Reasoning algorithms
cargo test parser --lib           # Parser functionality
cargo test ontology --lib         # Ontology operations
cargo test memory_safety_validation --lib  # Memory safety checks

# ⚠️ MODERATE: Integration tests (may need 1GB+ RAM)
cargo test integration_tests --lib

# 🔥 HIGH MEMORY: Stress tests (requires 2GB+ RAM, run individually)
cargo test test_extreme_memory_pressure --lib
cargo test test_concurrent_memory_stress --lib
cargo test test_ontology_memory_stress --lib

# 🚫 AVOID: Comprehensive test suite (causes OOM on most systems)
# cargo test comprehensive  # DON'T RUN THIS - will exhaust memory
```

#### Memory Usage by Test Category

| Test Category | Memory Usage | Safe for | Notes |
|---------------|-------------|----------|--------|
| Unit Tests | 64-256MB | All systems | Core functionality |
| Integration Tests | 256-512MB | Most systems | Component interaction |
| Memory Safety Tests | 256-512MB | Most systems | Memory monitoring |
| Stress Tests | 200MB-1GB each | High-memory systems | Individual execution only |
| Comprehensive Tests | 1GB-4GB+ | Server-grade systems | Avoid on development machines |

#### Recommended Testing Workflow

```bash
# Daily development (safe for all systems)
cargo test reasoning parser ontology memory_safety_validation

# Weekly validation (requires good memory)
cargo test integration_tests performance_regression_tests

# Monthly stress testing (requires high memory, run individually)
for test in $(cargo test --lib -- --list | grep stress | awk '{print $1}'); do
    echo "Running $test..."
    cargo test $test --lib || echo "Test $test failed or OOM"
    sleep 5  # Allow system recovery
done
```

### 2. Handle Large Data Carefully

```rust
// Bad: Can cause OOM
memory_safe_test!(bad_test, {
    let huge_data = vec![0u8; 500 * 1024 * 1024]; // 500MB
});

// Good: Use stress test configuration
memory_safe_stress_test!(good_test, {
    let large_data = vec![0u8; 50 * 1024 * 1024]; // 50MB
});
```

### 3. Clean Up Resources

```rust
memory_safe_test!(test_with_cleanup, {
    let mut large_vec = Vec::new();
    
    // Use the vector
    large_vec.push(1);
    
    // Explicit cleanup (optional, handled automatically)
    large_vec.clear();
    large_vec.shrink_to_fit();
});
```

### 4. Test with Realistic Data Sizes

```rust
memory_safe_test!(test_realistic_ontology, {
    // Test with reasonably sized ontologies
    let ontology = create_test_ontology(1000); // 1000 classes
    assert_eq!(ontology.classes().count(), 1000);
});
```

## Troubleshooting

### Common Issues

#### 1. Test Fails Due to Memory Limit

**Error**: `Memory limit exceeded: 300 MB used > 256 MB limit`

**Solution**:
- Use a larger memory configuration
- Optimize the test to use less memory
- Split the test into smaller tests

#### 2. Test Too Slow

**Error**: Test takes too long to complete

**Solution**:
- Reduce data size
- Use more efficient algorithms
- Use benchmark tests for performance measurement

#### 3. Cache Size Exceeded

**Error**: Cache size warnings

**Solution**:
- Increase cache limit in configuration
- Clear cache manually during test
- Use test isolation

### Debugging Memory Issues

#### Enable Verbose Output

```bash
OWL2_TEST_VERBOSE=1 cargo test my_test
```

#### Use Memory Guard Directly

```rust
let guard = TestGuard::with_config(config);

// Check memory during test
guard.check_memory().unwrap();

// Your test code

let report = guard.finish();
println!("{}", report.format());
```

#### Monitor System Memory

```bash
# On Linux/macOS
watch -n 1 'ps aux | grep cargo | head -5'

# On Windows
Get-Process cargo | Select-Object Name, WorkingSet
```

## Migration Guide

### Converting Existing Tests

#### Before (Unsafe)

```rust
#[test]
fn test_large_parsing() {
    let mut content = String::new();
    for i in 0..100000 {
        content.push_str(&format!("ex:Class{} a owl:Class .\n", i));
    }
    
    let parser = TurtleParser::new();
    let result = parser.parse_str(&content);
    assert!(result.is_ok());
}
```

#### After (Memory-Safe)

```rust
memory_safe_stress_test!(test_large_parsing, {
    let mut content = String::new();
    for i in 0..100000 {
        content.push_str(&format!("ex:Class{} a owl:Class .\n", i));
    }
    
    let parser = TurtleParser::new();
    let result = parser.parse_str(&content);
    assert!(result.is_ok());
});
```

### Batch Conversion

For multiple tests, use search and replace:

1. Replace `#[test]` with `memory_safe_test!(`
2. Add appropriate configuration
3. Close test body with `});`

## Configuration

### Environment Variables

- `OWL2_TEST_VERBOSE=1`: Enable verbose memory reporting
- `OWL2_TEST_MEMORY_LIMIT_MB=512`: Override default memory limit
- `OWL2_TEST_CACHE_LIMIT=1000`: Override default cache limit

### Global Configuration

```rust
use owl2_reasoner::test_memory_guard;

// Set global test memory configuration
let config = test_memory_guard::TestMemoryConfig {
    max_memory_bytes: 512 * 1024 * 1024, // 512MB
    max_cache_size: 1000,
    auto_cleanup: true,
    fail_on_limit_exceeded: true,
    warn_threshold_percent: 0.8,
    cleanup_interval: Duration::from_secs(60),
};

test_memory_guard::init_test_memory_guard(config).unwrap();
```

## Performance Considerations

### Memory Guard Overhead

The memory guard adds minimal overhead:
- ~1-2ms per test for setup/teardown
- ~0.1% CPU usage for monitoring
- ~1MB additional memory for tracking

### Optimizations

1. **Reuse configurations**: Create configuration objects once
2. **Batch operations**: Group related operations
3. **Early cleanup**: Clear resources when no longer needed
4. **Appropriate limits**: Use the smallest effective limits

## Integration with CI/CD

### GitHub Actions

```yaml
- name: Run memory-safe tests
  run: |
    export OWL2_TEST_VERBOSE=1
    cargo test --lib tests
```

### Jenkins

```groovy
stage('Memory Safe Tests') {
    steps {
        sh 'export OWL2_TEST_VERBOSE=1 && cargo test --lib tests'
    }
}
```

### Docker

```dockerfile
# Set reasonable memory limits for container
ENV OWL2_TEST_MEMORY_LIMIT_MB=512
ENV OWL2_TEST_CACHE_LIMIT=1000

RUN cargo test --lib tests
```

## Contributing

When adding new tests:

1. **Always use memory-safe test macros**
2. **Choose appropriate memory configurations**
3. **Add memory assertions for edge cases**
4. **Document any special memory requirements**

### Test Review Checklist

- [ ] Uses memory-safe test macro
- [ ] Has appropriate memory limits
- [ ] Handles cleanup properly
- [ ] Documents memory requirements
- [ ] Tested on various system configurations

## References

- [OWL2 Reasoner Documentation](../README.md)
- [Performance Testing Guide](BENCHMARKING.md)
- [API Reference](API_REFERENCE.md)
- [Architecture Overview](../docs/technical-documentation/)