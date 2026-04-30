# End-to-End Testing Guide for ProvChainOrg

## Overview

This comprehensive guide covers the end-to-end (E2E) testing framework for ProvChainOrg, a blockchain-based supply chain traceability system. The E2E testing suite validates complete user workflows from browser interactions through to blockchain storage and retrieval, ensuring system reliability and performance.

## Table of Contents

1. [Test Architecture](#test-architecture)
2. [Test Suites](#test-suites)
3. [Setup and Installation](#setup-and-installation)
4. [Running Tests](#running-tests)
5. [Test Configuration](#test-configuration)
6. [Performance Benchmarks](#performance-benchmarks)
7. [Compliance Testing](#compliance-testing)
8. [Troubleshooting](#troubleshooting)
9. [Contributing](#contributing)

## Test Architecture

### Testing Pyramid

```
                    ┌─────────────────┐
                    │   E2E Tests     │ ← Full system integration
                    │   (Browser +    │
                    │    API + DB)    │
                    └─────────────────┘
                  ┌───────────────────────┐
                  │  Integration Tests    │ ← Component integration
                  │  (API + Database)     │
                  └───────────────────────┘
              ┌─────────────────────────────────┐
              │        Unit Tests               │ ← Individual functions
              │   (Business Logic)              │
              └─────────────────────────────────┘
```

### Test Components

- **Browser Automation**: Headless Chrome/Firefox for UI testing
- **API Testing**: RESTful API endpoint validation
- **Database Testing**: RDF store and blockchain integrity
- **Performance Testing**: Load, stress, and scalability testing
- **Security Testing**: Authentication, authorization, and vulnerability testing
- **Compliance Testing**: Industry-specific regulatory compliance

## Test Suites

### 1. User Journey Tests (`tests/e2e_user_journeys.rs`)

Tests complete user workflows from authentication to task completion:

- **Supply Chain Manager Journey**: Product batch creation → tracking → verification
- **Quality Auditor Journey**: Compliance queries → audit report generation
- **Consumer Journey**: Public access → product tracing → certification verification
- **Administrator Journey**: System monitoring → blockchain validation → user management

### 2. Web Interface Tests (`tests/e2e_web_interface.rs`)

Validates all UI components and interactions:

- **Dashboard Functionality**: Real-time stats, health indicators
- **Block Explorer**: Search, pagination, block details
- **Product Traceability**: Batch search, timeline visualization
- **SPARQL Interface**: Query templates, custom queries, result formatting
- **Authentication Flow**: Login/logout, session management
- **Responsive Design**: Mobile, tablet, desktop compatibility

### 3. API Workflow Tests (`tests/e2e_api_workflows.rs`)

Tests complete API data flows:

- **Data Ingestion Pipeline**: RDF parsing → blockchain storage → verification
- **Query Processing**: SPARQL execution → result formatting → response delivery
- **Traceability Pipeline**: Batch lookup → graph traversal → timeline construction
- **Blockchain Validation**: Integrity checks → hash verification → chain validation

### 4. Test Runner (`tests/e2e_test_runner.rs`)

Orchestrates test execution with comprehensive reporting:

- **Parallel Execution**: Concurrent test running for faster feedback
- **Performance Metrics**: Response times, throughput, resource usage
- **Detailed Reporting**: JSON export, failure analysis, trend tracking
- **Configurable Thresholds**: Performance and quality gates

## Setup and Installation

### Prerequisites

```bash
# Install Rust and Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Chrome/Chromium for browser testing
# Ubuntu/Debian:
sudo apt-get install chromium-browser

# macOS:
brew install --cask google-chrome

# Windows:
# Download and install Chrome from https://www.google.com/chrome/
```

### Dependencies

Add to `Cargo.toml`:

```toml
[dev-dependencies]
# HTTP client for API testing
reqwest = { version = "0.11", features = ["json"] }

# Browser automation
headless_chrome = "1.0"
selenium-rs = "0.1"
fantoccini = "0.19"
webdriver = "0.46"

# Test utilities
tempfile = "3.8"
criterion = { version = "0.5", features = ["html_reports"] }
```

### Environment Setup

```bash
# Clone the repository
git clone https://github.com/anusornc/provchain-org.git
cd provchain-org

# Install dependencies
cargo build

# Set up test environment variables
export E2E_TEST_TIMEOUT=300
export E2E_PARALLEL_TESTS=true
export E2E_BROWSER_HEADLESS=true
export E2E_SERVER_PORT=8080
```

## Running Tests

### Quick Start

```bash
# Run all end-to-end tests
cargo test --test e2e_user_journeys
cargo test --test e2e_web_interface
cargo test --test e2e_api_workflows

# Run with verbose output
cargo test --test e2e_user_journeys -- --nocapture

# Run specific test
cargo test test_supply_chain_manager_complete_journey
```

### When to Use the E2E Test Script vs. Cargo Commands

Both approaches have their specific use cases:

**Use the E2E Test Script When:**
- Running complete test suite with reporting (CI/CD integration)
- Quick verification with `--quick` option
- Need structured test reports and logs
- Automated testing environments

**Use Cargo Commands When:**
- Development and debugging with detailed output
- Running specific test suites
- Working on individual test cases

For web interface tests, you need both the web server and ChromeDriver running:
```bash
# Terminal 1: Start web server
cargo run --bin provchain-org -- web-server --port 8080

# Terminal 2: Start ChromeDriver
chromedriver --port=9515

# Terminal 3: Run tests
cargo test --test e2e_web_interface -- --nocapture
# OR use the script for complete testing
./scripts/run_e2e_tests.sh
```

### Using the Test Runner

```rust
use provchain_org::tests::e2e_test_runner::{E2ETestRunner, TestSuiteConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestSuiteConfig::default();
    let mut runner = E2ETestRunner::new(config);
    
    let report = runner.run_all_tests().await?;
    
    // Export results
    std::fs::write("test_report.json", report.to_json().to_string())?;
    
    Ok(())
}
```

### Continuous Integration

```yaml
# .github/workflows/e2e-tests.yml
name: End-to-End Tests

on: [push, pull_request]

jobs:
  e2e-tests:
    runs-on: ubuntu-latest
    
    services:
      chrome:
        image: selenium/standalone-chrome:latest
        ports:
          - 4444:4444
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    
    - name: Run E2E Tests
      run: |
        cargo test --test e2e_user_journeys
        cargo test --test e2e_web_interface
        cargo test --test e2e_api_workflows
    
    - name: Upload Test Results
      uses: actions/upload-artifact@v3
      with:
        name: test-results
        path: test_report.json
```

## Test Configuration

### Configuration File (`e2e_config.toml`)

```toml
[test_suite]
parallel_execution = true
timeout_seconds = 300
retry_count = 2

[performance_thresholds]
max_response_time_ms = 5000
max_query_time_ms = 2000
min_throughput_ops_per_sec = 10

[browser]
headless = true
window_width = 1920
window_height = 1080
implicit_wait_ms = 5000

[server]
base_url = "http://localhost:8080"
startup_timeout_ms = 10000

[database]
cleanup_after_tests = true
seed_data = true
```

### Environment Variables

```bash
# Test execution
export E2E_PARALLEL=true
export E2E_TIMEOUT=300
export E2E_RETRY_COUNT=2

# Browser configuration
export E2E_BROWSER_HEADLESS=true
export E2E_BROWSER_WIDTH=1920
export E2E_BROWSER_HEIGHT=1080

# Server configuration
export E2E_SERVER_HOST=localhost
export E2E_SERVER_PORT=8080

# Performance thresholds
export E2E_MAX_RESPONSE_TIME=5000
export E2E_MAX_QUERY_TIME=2000
export E2E_MIN_THROUGHPUT=10
```

## Performance Benchmarks

### Response Time Targets

| Operation | Target (ms) | Acceptable (ms) | Critical (ms) |
|-----------|-------------|-----------------|---------------|
| Page Load | < 1000 | < 2000 | < 5000 |
| API Call | < 500 | < 1000 | < 2000 |
| SPARQL Query | < 1000 | < 2000 | < 5000 |
| Blockchain Write | < 2000 | < 5000 | < 10000 |
| Traceability Search | < 1500 | < 3000 | < 6000 |

### Throughput Targets

| Operation | Target (ops/sec) | Acceptable | Critical |
|-----------|------------------|------------|----------|
| API Requests | > 100 | > 50 | > 10 |
| SPARQL Queries | > 50 | > 25 | > 5 |
| Blockchain Writes | > 20 | > 10 | > 2 |
| Concurrent Users | > 100 | > 50 | > 10 |

### Load Testing Scenarios

```rust
#[tokio::test]
async fn test_load_scenario_normal_operation() {
    // Simulate 50 concurrent users for 5 minutes
    let concurrent_users = 50;
    let duration = Duration::from_secs(300);
    
    // Mix of operations:
    // 40% read operations (queries, traceability)
    // 30% write operations (add triples)
    // 20% UI interactions
    // 10% admin operations
}

#[tokio::test]
async fn test_stress_scenario_peak_load() {
    // Simulate 200 concurrent users for 10 minutes
    let concurrent_users = 200;
    let duration = Duration::from_secs(600);
    
    // Heavy load scenario with error rate monitoring
}
```

## Compliance Testing

### FSMA Food Safety Modernization Act

```rust
#[tokio::test]
async fn test_fsma_24_hour_traceability() {
    // Test 24-hour traceability requirement
    // - Record keeping for high-risk foods
    // - Rapid recall capability
    // - Supply chain transparency
}
```

### Pharmaceutical Cold Chain

```rust
#[tokio::test]
async fn test_pharmaceutical_cold_chain_compliance() {
    // Test temperature monitoring and validation
    // - Continuous temperature logging
    // - Excursion detection and alerting
    // - Chain of custody documentation
}
```

### Conflict Minerals (3TG)

```rust
#[tokio::test]
async fn test_conflict_minerals_due_diligence() {
    // Test 3TG material tracking
    // - Smelter certification verification
    // - Due diligence documentation
    // - Supply chain mapping
}
```

## Test Data Management

### Seed Data

```rust
fn create_test_supply_chain_data() -> Vec<String> {
    vec![
        // Product batches
        ":batch001 tc:product \"Organic Coffee Beans\" .",
        ":batch001 tc:origin \"Farm ABC, Colombia\" .",
        ":batch001 tc:batchId \"BATCH001\" .",
        
        // Supply chain events
        ":event001 tc:batch :batch001 .",
        ":event001 tc:actor \"Farmer John\" .",
        ":event001 tc:action \"Harvested\" .",
        ":event001 tc:timestamp \"2024-01-15T08:00:00Z\" .",
        
        // Environmental conditions
        ":batch001 tc:hasTemperature \"22.5\" .",
        ":batch001 tc:hasHumidity \"65.0\" .",
        
        // Certifications
        ":batch001 tc:certification \"Organic\" .",
        ":batch001 tc:certification \"Fair Trade\" .",
    ]
}
```

### Data Cleanup

```rust
async fn cleanup_test_data(blockchain: &mut Blockchain) -> Result<()> {
    // Remove test-specific data
    // Reset blockchain to clean state
    // Clear temporary files
    Ok(())
}
```

## Troubleshooting

### Common Issues

#### Browser Tests Failing

```bash
# Check Chrome/Chromium installation
which google-chrome
which chromium-browser

# Verify WebDriver
chromedriver --version

# Run with visible browser for debugging
export E2E_BROWSER_HEADLESS=false
cargo test test_dashboard_functionality -- --nocapture
```

#### API Tests Timing Out

```bash
# Check server startup
curl http://localhost:8080/health

# Increase timeout
export E2E_TIMEOUT=600

# Check server logs
tail -f server.log
```

#### Database Connection Issues

```bash
# Verify RDF store
ls -la data/rdf_store/

# Check permissions
chmod 755 data/rdf_store/

# Reset database
rm -rf data/rdf_store/*
```

### Debug Mode

```rust
#[tokio::test]
async fn debug_test_with_logging() {
    env_logger::init();
    
    // Enable detailed logging
    std::env::set_var("RUST_LOG", "debug");
    
    // Your test code here
}
```

### Performance Debugging

```rust
use std::time::Instant;

#[tokio::test]
async fn debug_performance_bottleneck() {
    let start = Instant::now();
    
    // Operation under test
    let result = slow_operation().await;
    
    let duration = start.elapsed();
    println!("Operation took: {:?}", duration);
    
    // Add detailed timing for sub-operations
}
```

## Test Reporting

### HTML Reports

```bash
# Generate HTML test report
cargo test --test e2e_user_journeys -- --format=json | \
  tee test_results.json | \
  cargo2junit > test_results.xml

# Convert to HTML
junit2html test_results.xml test_report.html
```

### Metrics Dashboard

```rust
// Export metrics to Prometheus format
fn export_test_metrics(report: &TestSuiteReport) -> String {
    format!(
        "# HELP e2e_test_duration_seconds Duration of E2E tests\n\
         # TYPE e2e_test_duration_seconds gauge\n\
         e2e_test_duration_seconds {}\n\
         # HELP e2e_test_success_rate Success rate of E2E tests\n\
         # TYPE e2e_test_success_rate gauge\n\
         e2e_test_success_rate {}\n",
        report.total_duration.as_secs_f64(),
        report.success_rate / 100.0
    )
}
```

## Contributing

### Adding New Tests

1. **Create test file**: Follow naming convention `test_<feature>_<scenario>.rs`
2. **Use test helpers**: Leverage existing utilities for server setup and authentication
3. **Add documentation**: Include test purpose and expected behavior
4. **Update CI**: Add new tests to continuous integration pipeline

### Test Guidelines

- **Isolation**: Each test should be independent and not rely on other tests
- **Cleanup**: Always clean up test data and resources
- **Assertions**: Use descriptive assertion messages
- **Performance**: Set reasonable timeouts and performance expectations
- **Documentation**: Document test scenarios and expected outcomes

### Example Test Template

```rust
#[tokio::test]
async fn test_new_feature_scenario() -> Result<()> {
    // Setup
    let (port, _server_handle) = start_test_server().await?;
    let base_url = format!("http://localhost:{}", port);
    let client = Client::new();
    
    // Authentication
    let token = authenticate_user(&client, &base_url, "user", "password").await?;
    
    // Test execution
    let response = client
        .post(&format!("{}/api/new-endpoint", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&test_data)
        .send()
        .await?;
    
    // Assertions
    assert!(response.status().is_success(), "Should succeed");
    let data: serde_json::Value = response.json().await?;
    assert_eq!(data["status"], "success", "Should return success status");
    
    // Cleanup (if needed)
    cleanup_test_data().await?;
    
    Ok(())
}
```

## Conclusion

This comprehensive E2E testing framework ensures the reliability, performance, and compliance of the ProvChainOrg system. Regular execution of these tests provides confidence in system stability and helps identify issues before they reach production.

For questions or support, please refer to the project documentation or open an issue in the repository.
