# Performance Testing Guide for ProvChain-Org

## Overview

This comprehensive performance testing suite provides production-ready validation of the ProvChain-Org blockchain supply chain traceability system. The suite covers all aspects of system performance including backend operations, frontend rendering, load handling, and system resilience.

## Performance Testing Architecture

### 1. Backend Performance Testing

#### Comprehensive Benchmarks (`benches/comprehensive_performance_benchmarks.rs`)

**Coverage:**
- **Blockchain Performance**: Block creation, transaction processing, validation
- **API Performance**: REST API response times, WebSocket performance
- **Real-time Traceability**: Complex SPARQL query performance
- **Resource Utilization**: Memory efficiency, CPU usage patterns
- **Supply Chain Workflows**: UHT milk processing, pharmaceutical tracking, automotive parts traceability
- **OWL2 Reasoning**: Cross-ontology reasoning performance
- **Resilience Testing**: Network partitions, error recovery, resource exhaustion

**Key Metrics:**
- Block creation time (< 50ms target)
- Transaction throughput (> 1000 TPS target)
- Query response time (< 500ms for complex queries)
- Memory usage patterns and optimization
- CPU utilization under load

#### Usage:
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench comprehensive_performance_benchmarks

# Generate HTML reports
cargo bench --bench comprehensive_performance_benchmarks | cargo2junit > benchmarks.xml
```

### 2. Load Testing Scenarios

#### Load Tests (`tests/load_tests.rs`)

**Scenarios:**

1. **High-Volume Transaction Processing**
   - 50 concurrent users
   - 200 requests per user
   - 2-minute duration
   - Target: >1000 TPS

2. **Concurrent API User Simulation**
   - 1000 concurrent users
   - 20 requests per user
   - 3-minute duration
   - Target: <200ms average response time

3. **Real-time Traceability Queries**
   - 200 concurrent users
   - 100 queries per user
   - 90-second duration
   - Target: <2s for complex queries

4. **Supply Chain Workloads**
   - UHT milk processing simulation
   - Pharmaceutical tracking
   - Automotive parts traceability
   - Cross-border trade scenarios

#### Usage:
```bash
# Run all load tests
cargo test --release --test load_tests

# Run specific load test
cargo test --release --test load_tests test_high_volume_transaction_processing

# Run with custom configuration
LOAD_TEST_USERS=500 LOAD_TEST_DURATION=300 cargo test --release --test load_tests
```

### 3. Stress Testing Framework

#### Stress Tests (`tests/stress_tests.rs`)

**Test Categories:**

1. **Maximum System Capacity**
   - Incrementally increasing load
   - Resource limit detection
   - Capacity planning data

2. **Resource Exhaustion**
   - Memory pressure scenarios
   - CPU saturation testing
   - File descriptor limits
   - Network connection limits

3. **Network Failure Resilience**
   - High latency simulation
   - Packet loss scenarios
   - Random failure injection
   - Recovery time measurement

4. **Database Contention**
   - Concurrent read/write operations
   - Lock contention analysis
   - Query optimization validation

#### Usage:
```bash
# Run all stress tests
cargo test --release --test stress_tests

# Run specific stress test
cargo test --release --test stress_tests test_maximum_system_capacity

# Monitor system during stress tests
./scripts/performance_monitoring.sh --interval 1 &
cargo test --release --test stress_tests
```

### 4. Frontend Performance Testing

#### Bundle Size Testing (`frontend/src/tests/performance/bundleSize.test.ts`)

**Coverage:**
- Main bundle size (< 500KB target)
- Vendor bundle size (< 800KB target)
- Code splitting effectiveness
- Asset optimization validation
- Cache performance analysis

#### Rendering Performance (`frontend/src/tests/performance/rendering.test.ts`)

**Coverage:**
- Component rendering speed (< 100ms simple, < 1s complex)
- Large data visualization (1000+ nodes)
- Real-time dashboard updates
- Memory leak detection
- Animation performance (60fps target)

#### User Interaction (`frontend/src/tests/performance/userInteraction.test.ts`)

**Coverage:**
- Button response time (< 100ms)
- Form validation performance
- Search and filtering efficiency
- Navigation transitions
- Debouncing effectiveness

#### Usage:
```bash
cd frontend

# Run all performance tests
npm run test:performance

# Run specific performance test
npm run test:performance -- bundleSize

# Run with coverage
npm run test:performance -- --coverage
```

## Performance Monitoring

### Real-time Monitoring (`scripts/performance_monitoring.sh`)

**Features:**
- System resource monitoring (CPU, memory, disk, network)
- Application-specific metrics
- Database query performance
- Network latency monitoring
- Automated alerting
- Historical data collection

**Usage:**
```bash
# Start monitoring with default settings
./scripts/performance_monitoring.sh

# Custom monitoring interval
./scripts/performance_monitoring.sh --interval 10

# Custom alert thresholds
./scripts/performance_monitoring.sh --cpu-threshold 90 --memory-threshold 80

# Generate report only
./scripts/performance_monitoring.sh --report-only
```

**Environment Variables:**
```bash
export SLACK_WEBHOOK_URL="https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK"
export ALERT_EMAIL="alerts@yourcompany.com"

./scripts/performance_monitoring.sh
```

## Performance Regression Detection

### Automated Regression Detection (`scripts/performance_regression_detector.py`)

**Features:**
- Baseline comparison
- Trend analysis
- Statistical significance testing
- Automated alerting
- Visualization generation
- CI/CD integration

**Usage:**
```bash
# Detect regressions against baseline
python3 scripts/performance_regression_detector.py

# Generate performance report only
python3 scripts/performance_regression_detector.py --generate-report

# Generate visualization charts
python3 scripts/performance_regression_detector.py --generate-charts

# Update baseline with current metrics
python3 scripts/performance_regression_detector.py --update-baseline
```

**Configuration (`performance_config.json`):**
```json
{
  "regression_threshold": 15.0,
  "confidence_level": 0.95,
  "min_sample_size": 10,
  "email": {
    "smtp_server": "smtp.gmail.com",
    "smtp_port": 587,
    "username": "your-email@gmail.com",
    "password": "your-app-password",
    "from": "alerts@yourcompany.com",
    "to": "team@yourcompany.com"
  },
  "slack_webhook": "https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK"
}
```

## CI/CD Integration

### GitHub Actions Workflow (`.github/workflows/performance-testing.yml`)

**Triggers:**
- Push to main/develop branches
- Pull requests
- Daily scheduled runs
- Manual dispatch

**Jobs:**
1. **Benchmarks**: Comprehensive performance benchmarks
2. **Load Tests**: High-volume load testing scenarios
3. **Stress Tests**: System limit and resilience testing
4. **Frontend Performance**: Bundle analysis and rendering tests
5. **Regression Detection**: Automated regression analysis
6. **Report Generation**: Comprehensive performance reports

**Configuration:**
```yaml
# Trigger specific test type
on:
  workflow_dispatch:
    inputs:
      test_type:
        description: 'Type of performance test to run'
        required: true
        default: 'all'
        type: choice
        options:
          - all
          - benchmarks
          - load-tests
          - stress-tests
          - regression-detection
```

## Performance Targets and SLAs

### Blockchain Performance
- **Block Creation Time**: < 50ms average
- **Transaction Throughput**: > 1000 TPS
- **Query Performance**: < 500ms for complex SPARQL queries
- **Validation Speed**: < 100ms for blockchain validation
- **Memory Usage**: < 2GB for typical operations

### API Performance
- **Response Time**: < 200ms average, < 1s P95
- **Concurrent Users**: Support 1000+ concurrent users
- **WebSocket Updates**: < 100ms for real-time updates
- **Error Rate**: < 1% under normal load

### Frontend Performance
- **Initial Load Time**: < 3 seconds
- **Interaction Response**: < 100ms for UI interactions
- **Bundle Size**: < 1.3MB total (main + vendor)
- **Frame Rate**: Maintain 60fps for animations

### System Resilience
- **Availability**: > 99.9% uptime
- **Recovery Time**: < 30 seconds for failure recovery
- **Resource Limits**: Graceful degradation at 85% resource utilization
- **Data Consistency**: 100% data integrity under all conditions

## Performance Profiling Tools

### Backend Profiling
```bash
# CPU profiling
perf record --call-graph=dwarf cargo run --release
perf report

# Memory profiling
valgrind --tool=massif cargo run --release
ms_print massif.out.*

# Heap profiling
heaptrack cargo run --release

# Flame graph generation
cargo flamegraph --bin provchain-org -- --web-server
```

### Frontend Profiling
```bash
# Bundle analysis
npm install -g webpack-bundle-analyzer
npx webpack-bundle-analyzer dist/static/js/*.js

# Lighthouse CI
npm install -g @lhci/cli
lhci autorun

# Runtime performance profiling
# Use Chrome DevTools Performance tab
# Generate flame graphs for JavaScript execution
```

## Performance Optimization Recommendations

### Backend Optimizations

1. **Database Optimization**
   - Implement query result caching
   - Use connection pooling
   - Optimize SPARQL queries with proper indexing
   - Consider read replicas for query-heavy workloads

2. **Memory Management**
   - Implement object pooling for frequently allocated structures
   - Use memory-efficient data structures
   - Implement streaming for large data processing
   - Add periodic garbage collection triggers

3. **Concurrency Optimization**
   - Use async/await patterns for I/O operations
   - Implement proper thread pool sizing
   - Use lock-free data structures where appropriate
   - Optimize critical sections

### Frontend Optimizations

1. **Bundle Optimization**
   - Implement code splitting for large dependencies
   - Use dynamic imports for rarely used components
   - Optimize vendor bundle with tree shaking
   - Minimize third-party dependencies

2. **Rendering Optimization**
   - Use React.memo for expensive components
   - Implement virtual scrolling for large lists
   - Optimize Cytoscape graph rendering
   - Use Web Workers for heavy computations

3. **Caching Strategy**
   - Implement service worker caching
   - Use HTTP caching headers properly
   - Cache API responses appropriately
   - Implement local storage for frequently accessed data

## Troubleshooting Performance Issues

### Common Issues and Solutions

1. **High Memory Usage**
   ```bash
   # Monitor memory usage
   ./scripts/performance_monitoring.sh

   # Identify memory leaks
   valgrind --leak-check=full cargo run --release

   # Check for large allocations
   cargo run --features profiling
   ```

2. **Slow Query Performance**
   ```bash
   # Enable query logging
   RUST_LOG=debug cargo run --release

   # Analyze query patterns
   curl -X POST http://localhost:8080/api/traceability/query \
     -H "Content-Type: application/sparql-query" \
     -d "EXPLAIN ANALYZE SELECT * WHERE { ?s ?p ?o }"
   ```

3. **Frontend Performance Issues**
   ```bash
   # Analyze bundle size
   npm run build:analyze

   # Run Lighthouse audit
   npx lighthouse http://localhost:3000 --output html --output-path ./lighthouse-report.html
   ```

## Performance Testing Best Practices

1. **Test in Production-like Environment**
   - Use realistic data volumes
   - Test with actual hardware specifications
   - Simulate real user behavior patterns

2. **Establish Performance Baselines**
   - Document current performance metrics
   - Track performance over time
   - Set realistic performance targets

3. **Continuous Performance Monitoring**
   - Monitor in production continuously
   - Set up automated alerting
   - Regularly review performance trends

4. **Performance Regression Prevention**
   - Include performance tests in CI/CD
   - Set up performance gates for deployment
   - Regularly review and update baselines

## Reporting and Documentation

### Performance Report Generation
```bash
# Generate comprehensive report
python3 scripts/generate_comprehensive_report.py \
  --input-dir performance-results/ \
  --output-dir reports/ \
  --format html,pdf
```

### Report Components
- Executive Summary
- Performance Metrics Overview
- Benchmark Results Analysis
- Load Testing Outcomes
- Stress Testing Findings
- Regression Detection Results
- Recommendations and Action Items
- Historical Performance Trends

## Maintenance and Updates

### Regular Maintenance Tasks
1. **Update Baselines**: Monthly or after major feature releases
2. **Review Test Scenarios**: Quarterly to ensure relevance
3. **Update Performance Targets**: Based on business requirements
4. **Review Monitoring Configuration**: Ensure alerting thresholds are appropriate

### Continuous Improvement
1. **Analyze Performance Trends**: Look for long-term patterns
2. **Optimize Hot Paths**: Focus on frequently executed code
3. **Monitor Third-party Dependencies**: Watch for performance regressions
4. **Stay Updated with Tools**: Keep profiling and monitoring tools current

This comprehensive performance testing suite ensures that ProvChain-Org maintains high performance standards and provides reliable service for supply chain traceability requirements.