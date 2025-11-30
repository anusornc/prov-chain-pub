# Memory Guard Configuration

This comprehensive guide covers configuration options, setup procedures, and customization patterns for the memory guard system in the OWL2 Reasoner.

## Overview

The memory guard system provides flexible configuration options to adapt to different testing scenarios, system constraints, and performance requirements. Configuration can be applied at multiple levels:

- **Global configuration** for system-wide defaults
- **Test-specific configuration** for individual test requirements
- **Environment variable overrides** for CI/CD and deployment scenarios
- **Runtime configuration** for dynamic adjustment

## Configuration Types

### 1. MemorySafeTestConfig

The primary configuration structure for memory-safe testing.

```rust
#[derive(Debug, Clone)]
pub struct MemorySafeTestConfig {
    pub max_memory_mb: usize,        // Maximum memory limit in MB
    pub max_cache_size: usize,       // Maximum cache entry count
    pub fail_on_limit: bool,         // Whether to fail when limits exceeded
    pub verbose: bool,               // Enable verbose reporting
}
```

### 2. TestMemoryConfig

Low-level configuration for the memory guard system.

```rust
#[derive(Debug, Clone)]
pub struct TestMemoryConfig {
    pub max_memory_bytes: usize,                    // Maximum memory in bytes
    pub max_cache_size: usize,                      // Maximum cache size
    pub check_interval: Duration,                   // Memory check frequency
    pub auto_cleanup: bool,                         // Enable automatic cleanup
    pub fail_on_limit_exceeded: bool,               // Fail on limit exceeded
    pub warn_threshold_percent: f64,                // Warning threshold (0.0-1.0)
}
```

## Predefined Configurations

### Small Configuration

**Purpose**: Unit tests and simple operations with minimal memory requirements.

```rust
impl MemorySafeTestConfig {
    pub fn small() -> Self {
        Self {
            max_memory_mb: 64,        // 64MB memory limit
            max_cache_size: 100,      // 100 cache entries
            fail_on_limit: true,      // Fail immediately on limit exceeded
            verbose: false,           // Minimal reporting
        }
    }
}
```

**Use Cases**:
- Unit tests for individual components
- Parser validation tests
- Simple entity creation tests
- Basic reasoning operations

**Example**:
```rust
memory_safe_test!(test_entity_creation, MemorySafeTestConfig::small(), {
    let class = Class::new("http://example.org/Person");
    assert!(class.iri().to_string().contains("Person"));
});
```

### Medium Configuration

**Purpose**: Integration tests with moderate memory requirements.

```rust
impl MemorySafeTestConfig {
    pub fn medium() -> Self {
        Self {
            max_memory_mb: 128,       // 128MB memory limit
            max_cache_size: 300,      // 300 cache entries
            fail_on_limit: true,      // Fail immediately on limit exceeded
            verbose: false,           // Standard reporting
        }
    }
}
```

**Use Cases**:
- Integration tests combining multiple components
- Medium-sized ontology processing
- Parser and reasoning integration
- Cache behavior validation

**Example**:
```rust
memory_safe_test!(test_ontology_integration, MemorySafeTestConfig::medium(), {
    let ontology = parse_ontology_from_file("test.ttl")?;
    let reasoner = SimpleReasoner::new(ontology);
    assert!(reasoner.is_consistent()?);
});
```

### Large Configuration

**Purpose**: Complex operations with significant memory requirements.

```rust
impl MemorySafeTestConfig {
    pub fn large() -> Self {
        Self {
            max_memory_mb: 512,       // 512MB memory limit
            max_cache_size: 1000,     // 1000 cache entries
            fail_on_limit: true,      // Fail immediately on limit exceeded
            verbose: true,            // Detailed reporting
        }
    }
}
```

**Use Cases**:
- Large ontology processing
- Complex reasoning operations
- Classification and inference tasks
- Performance validation tests

**Example**:
```rust
memory_safe_test!(test_large_ontology_classification, MemorySafeTestConfig::large(), {
    let ontology = load_large_ontology("biomedical.owl")?;
    let reasoner = SimpleReasoner::new(ontology);
    
    let classification = reasoner.classify()?;
    assert!(!classification.is_empty());
});
```

### Stress Configuration

**Purpose**: Stress testing and benchmarking with relaxed limits.

```rust
impl MemorySafeTestConfig {
    pub fn stress() -> Self {
        Self {
            max_memory_mb: 1024,      // 1GB memory limit
            max_cache_size: 2000,     // 2000 cache entries
            fail_on_limit: false,     // Don't fail, just warn
            verbose: true,            // Detailed reporting
        }
    }
}
```

**Use Cases**:
- Stress testing under high memory pressure
- Performance benchmarking
- Memory leak detection
- System limit validation

**Example**:
```rust
memory_safe_stress_test!(test_memory_pressure_handling, {
    let mut data = Vec::new();
    
    // Allocate significant memory
    for i in 0..1000 {
        let chunk: Vec<u8> = vec![i as u8; 1024 * 1024]; // 1MB
        data.push(chunk);
        
        // Perform operations under pressure
        if i % 100 == 0 {
            let ontology = create_test_ontology(i);
            let reasoner = SimpleReasoner::new(ontology);
            let _ = reasoner.is_consistent()?;
        }
    }
    
    assert_eq!(data.len(), 1000);
});
```

## Custom Configuration

### Creating Custom Configurations

```rust
use owl2_reasoner::test_helpers::MemorySafeTestConfig;

let custom_config = MemorySafeTestConfig {
    max_memory_mb: 256,       // 256MB memory limit
    max_cache_size: 500,      // 500 cache entries
    fail_on_limit: true,      // Fail on limit exceeded
    verbose: false,           // Standard reporting
};

memory_safe_test!(my_custom_test, custom_config, {
    // Test implementation with custom limits
});
```

### Advanced Configuration

```rust
use owl2_reasoner::test_memory_guard::TestMemoryConfig;
use std::time::Duration;

let advanced_config = TestMemoryConfig {
    max_memory_bytes: 512 * 1024 * 1024,  // 512MB
    max_cache_size: 1000,
    check_interval: Duration::from_millis(100),  // Check every 100ms
    auto_cleanup: true,
    fail_on_limit_exceeded: true,
    warn_threshold_percent: 0.7,  // Warn at 70% usage
};

let guard = TestMemoryGuard::with_config(advanced_config);
guard.start_monitoring();

// Your test code here

let report = guard.stop_monitoring();
report.assert_acceptable();
```

## Environment Variables

### Configuration Override Variables

```bash
# Override default memory limit (in MB)
export OWL2_TEST_MEMORY_LIMIT_MB=512

# Override default cache size
export OWL2_TEST_CACHE_LIMIT=1000

# Enable verbose memory reporting
export OWL2_TEST_VERBOSE=1

# Set warning threshold (percentage)
export OWL2_TEST_WARN_THRESHOLD=80
```

### Usage in CI/CD

#### GitHub Actions

```yaml
name: Memory-Safe Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Run tests with custom limits
      run: |
        export OWL2_TEST_MEMORY_LIMIT_MB=512
        export OWL2_TEST_CACHE_LIMIT=1000
        export OWL2_TEST_VERBOSE=1
        cargo test --lib --all-features
        
    - name: Run stress tests
      run: |
        export OWL2_TEST_MEMORY_LIMIT_MB=2048  # 2GB for stress tests
        export OWL2_TEST_VERBOSE=1
        cargo test stress_tests --lib
```

#### Jenkins

```groovy
pipeline {
    agent any
    
    stages {
        stage('Memory-Safe Tests') {
            steps {
                sh '''
                    export OWL2_TEST_MEMORY_LIMIT_MB=512
                    export OWL2_TEST_CACHE_LIMIT=1000
                    export OWL2_TEST_VERBOSE=1
                    cargo test --lib --all-features
                '''
            }
        }
        
        stage('Stress Tests') {
            steps {
                sh '''
                    export OWL2_TEST_MEMORY_LIMIT_MB=1024
                    export OWL2_TEST_VERBOSE=1
                    cargo test stress_tests --lib
                '''
            }
        }
    }
}
```

#### Docker

```dockerfile
FROM rust:1.70

# Set memory limits for container environment
ENV OWL2_TEST_MEMORY_LIMIT_MB=512
ENV OWL2_TEST_CACHE_LIMIT=1000
ENV OWL2_TEST_VERBOSE=1

WORKDIR /app
COPY . .

# Run tests with environment configuration
RUN cargo test --lib --all-features

# Override for specific test scenarios
RUN OWL2_TEST_MEMORY_LIMIT_MB=1024 cargo test stress_tests --lib
```

## Configuration Best Practices

### 1. Choose Appropriate Limits

```rust
// ✅ Good: Match limits to test complexity
mod unit_tests {
    memory_safe_test!(test_simple_operation, MemorySafeTestConfig::small(), {
        // Simple test with minimal memory usage
    });
}

mod integration_tests {
    memory_safe_test!(test_complex_integration, MemorySafeTestConfig::medium(), {
        // Integration test with moderate memory usage
    });
}

mod performance_tests {
    memory_safe_test!(test_large_scale_operation, MemorySafeTestConfig::large(), {
        // Large-scale test with significant memory usage
    });
}
```

### 2. Use Verbose Mode for Debugging

```rust
// Enable verbose reporting for complex tests
memory_safe_test!(test_complex_reasoning, MemorySafeTestConfig {
    max_memory_mb: 256,
    max_cache_size: 500,
    fail_on_limit: true,
    verbose: true,  // Enable detailed reporting
}, {
    // Complex test with detailed memory tracking
});
```

### 3. Configure for Different Environments

```rust
// Development environment - stricter limits
#[cfg(debug_assertions)]
const DEV_CONFIG: MemorySafeTestConfig = MemorySafeTestConfig {
    max_memory_mb: 128,
    max_cache_size: 300,
    fail_on_limit: true,
    verbose: false,
};

// Release environment - relaxed limits
#[cfg(not(debug_assertions))]
const RELEASE_CONFIG: MemorySafeTestConfig = MemorySafeTestConfig {
    max_memory_mb: 512,
    max_cache_size: 1000,
    fail_on_limit: true,
    verbose: false,
};

memory_safe_test!(test_environment_specific, {
    let config = if cfg!(debug_assertions) { DEV_CONFIG } else { RELEASE_CONFIG };
    // Test implementation
});
```

### 4. Adaptive Configuration

```rust
fn adaptive_config() -> MemorySafeTestConfig {
    let available_memory = get_available_memory_mb();
    
    match available_memory {
        0..=256 => MemorySafeTestConfig::small(),    // Low memory systems
        257..=1024 => MemorySafeTestConfig::medium(), // Standard systems
        1025..=4096 => MemorySafeTestConfig::large(), // High memory systems
        _ => MemorySafeTestConfig::stress(),          // Very high memory systems
    }
}

memory_safe_test!(test_adaptive_configuration, adaptive_config(), {
    // Test adapts to system capabilities
});
```

## Advanced Configuration Patterns

### 1. Configuration Builder Pattern

```rust
pub struct MemoryConfigBuilder {
    config: MemorySafeTestConfig,
}

impl MemoryConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: MemorySafeTestConfig::default(),
        }
    }
    
    pub fn memory_limit(mut self, limit_mb: usize) -> Self {
        self.config.max_memory_mb = limit_mb;
        self
    }
    
    pub fn cache_size(mut self, size: usize) -> Self {
        self.config.max_cache_size = size;
        self
    }
    
    pub fn fail_on_limit(mut self, fail: bool) -> Self {
        self.config.fail_on_limit = fail;
        self
    }
    
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.config.verbose = verbose;
        self
    }
    
    pub fn build(self) -> MemorySafeTestConfig {
        self.config
    }
}

// Usage
let config = MemoryConfigBuilder::new()
    .memory_limit(256)
    .cache_size(500)
    .verbose(true)
    .build();

memory_safe_test!(test_with_builder_config, config, {
    // Test implementation
});
```

### 2. Configuration Profiles

```rust
pub struct ConfigurationProfile {
    name: String,
    config: MemorySafeTestConfig,
}

impl ConfigurationProfile {
    pub fn unit_test() -> Self {
        Self {
            name: "unit_test".to_string(),
            config: MemorySafeTestConfig::small(),
        }
    }
    
    pub fn integration_test() -> Self {
        Self {
            name: "integration_test".to_string(),
            config: MemorySafeTestConfig::medium(),
        }
    }
    
    pub fn performance_test() -> Self {
        Self {
            name: "performance_test".to_string(),
            config: MemorySafeTestConfig::large(),
        }
    }
    
    pub fn custom(name: &str, config: MemorySafeTestConfig) -> Self {
        Self {
            name: name.to_string(),
            config,
        }
    }
}

// Usage with profile system
macro_rules! profile_test {
    ($test_name:ident, $profile:expr, $test_body:block) => {
        #[test]
        fn $test_name() {
            let profile = $profile;
            println!("Running test with profile: {}", profile.name);
            
            memory_safe_test!($test_name, profile.config, $test_body);
        }
    };
}

profile_test!(test_with_profile, ConfigurationProfile::integration_test(), {
    // Test implementation with profile configuration
});
```

### 3. Dynamic Configuration Adjustment

```rust
pub struct DynamicMemoryConfig {
    base_config: MemorySafeTestConfig,
    system_memory: usize,
    test_complexity: f64,
}

impl DynamicMemoryConfig {
    pub fn new(base_config: MemorySafeTestConfig) -> Self {
        let system_memory = get_total_system_memory_mb();
        let test_complexity = estimate_test_complexity();
        
        Self {
            base_config,
            system_memory,
            test_complexity,
        }
    }
    
    pub fn adjusted_config(&self) -> MemorySafeTestConfig {
        let memory_factor = (self.system_memory as f64 / 8192.0).min(2.0); // Scale based on 8GB baseline
        let complexity_factor = self.test_complexity;
        
        let adjusted_memory = (self.base_config.max_memory_mb as f64 * memory_factor * complexity_factor) as usize;
        
        MemorySafeTestConfig {
            max_memory_mb: adjusted_memory,
            max_cache_size: (self.base_config.max_cache_size as f64 * memory_factor) as usize,
            fail_on_limit: self.base_config.fail_on_limit,
            verbose: self.base_config.verbose,
        }
    }
}

// Usage
let dynamic_config = DynamicMemoryConfig::new(MemorySafeTestConfig::medium());
let adjusted_config = dynamic_config.adjusted_config();

memory_safe_test!(test_dynamic_config, adjusted_config, {
    // Test with dynamically adjusted configuration
});
```

## Configuration Validation

### Validation Functions

```rust
pub fn validate_config(config: &MemorySafeTestConfig) -> Result<(), ConfigError> {
    if config.max_memory_mb == 0 {
        return Err(ConfigError::InvalidMemoryLimit("Memory limit cannot be zero"));
    }
    
    if config.max_memory_mb > 10240 {  // 10GB maximum
        return Err(ConfigError::InvalidMemoryLimit("Memory limit exceeds maximum allowed"));
    }
    
    if config.max_cache_size == 0 {
        return Err(ConfigError::InvalidCacheSize("Cache size cannot be zero"));
    }
    
    if config.max_cache_size > 10000 {
        return Err(ConfigError::InvalidCacheSize("Cache size exceeds maximum allowed"));
    }
    
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid memory limit: {0}")]
    InvalidMemoryLimit(&'static str),
    
    #[error("Invalid cache size: {0}")]
    InvalidCacheSize(&'static str),
}
```

### Configuration Testing

```rust
#[cfg(test)]
mod config_tests {
    use super::*;
    
    memory_safe_test!(test_small_config_validation, MemorySafeTestConfig::small(), {
        let config = MemorySafeTestConfig::small();
        assert!(validate_config(&config).is_ok());
        assert_eq!(config.max_memory_mb, 64);
        assert_eq!(config.max_cache_size, 100);
    });
    
    memory_safe_test!(test_large_config_validation, MemorySafeTestConfig::large(), {
        let config = MemorySafeTestConfig::large();
        assert!(validate_config(&config).is_ok());
        assert_eq!(config.max_memory_mb, 512);
        assert_eq!(config.max_cache_size, 1000);
    });
    
    memory_safe_test!(test_custom_config_validation, MemorySafeTestConfig::medium(), {
        let custom_config = MemorySafeTestConfig {
            max_memory_mb: 256,
            max_cache_size: 500,
            fail_on_limit: true,
            verbose: false,
        };
        
        assert!(validate_config(&custom_config).is_ok());
        
        // Test with invalid config
        let invalid_config = MemorySafeTestConfig {
            max_memory_mb: 0,  // Invalid
            max_cache_size: 500,
            fail_on_limit: true,
            verbose: false,
        };
        
        assert!(validate_config(&invalid_config).is_err());
    });
}
```

## Troubleshooting Configuration Issues

### Common Configuration Problems

#### 1. Memory Limit Too Low

**Symptoms**: Tests fail immediately with "Memory limit exceeded" errors.

**Solutions**:
- Increase memory limit for the test
- Use a larger predefined configuration
- Optimize test to use less memory

```rust
// Increase limit
memory_safe_test!(my_test, MemorySafeTestConfig::large(), {
    // Test implementation
});

// Or use custom configuration
let custom_config = MemorySafeTestConfig {
    max_memory_mb: 1024,  // 1GB
    max_cache_size: 2000,
    fail_on_limit: false,  // Don't fail, just warn
    verbose: true,
};
```

#### 2. Cache Size Too Small

**Symptoms**: Poor performance, frequent cache evictions.

**Solutions**:
- Increase cache size
- Use larger configuration
- Optimize cache usage patterns

```rust
let config = MemorySafeTestConfig {
    max_memory_mb: 256,
    max_cache_size: 1000,  // Increased cache size
    fail_on_limit: true,
    verbose: true,
};
```

#### 3. Environment Variable Overrides Not Working

**Symptoms**: Environment variables don't affect test behavior.

**Solutions**:
- Verify variable names are correct
- Check that variables are exported properly
- Ensure variables are set before test execution

```bash
# Correct usage
export OWL2_TEST_MEMORY_LIMIT_MB=512
export OWL2_TEST_CACHE_LIMIT=1000
export OWL2_TEST_VERBOSE=1

# Verify variables are set
env | grep OWL2_TEST

# Run tests
cargo test --lib
```

---

**Next**: [Performance Impact Analysis](performance-impact.md) - Learn about performance characteristics and optimization.