# OWL2 Reasoner EPCIS Ecosystem Integration

This guide demonstrates how to use the OWL2 reasoner with EPCIS (Electronic Product Code Information Service) data across different ecosystem integration points, enabling comprehensive supply chain traceability and reasoning capabilities.

## Overview

The OWL2 reasoner provides comprehensive EPCIS integration that includes:

- **Native EPCIS Data Models**: Complete GS1 EPCIS 2.0 standard implementation
- **Multi-Platform Support**: Python bindings, REST API, and native Rust integration
- **Supply Chain Reasoning**: OWL2-based inference for traceability and compliance
- **Real-time Processing**: Stream processing and batch operations
- **Multi-Language Clients**: Java, C#, Go, JavaScript, Python, Ruby, PHP support

## Quick Start

### Installation

```bash
# Add to your Cargo.toml
[dependencies]
owl2-reasoner = "0.1.0"
```

### Basic EPCIS Processing

```rust
use owl2_reasoner::epcis_parser::*;
use owl2_reasoner::reasoning::SimpleReasoner;

// Parse EPCIS XML data
let parser = EPCISDocumentParser::default();
let events = parser.parse_xml_str(epcis_xml_content)?;

// Create OWL2 ontology and reasoner
let ontology = parser.to_ontology(&events)?;
let mut reasoner = SimpleReasoner::new(ontology);

// Perform reasoning operations
let is_consistent = reasoner.is_consistent()?;
let el_valid = reasoner.validate_profile(Owl2Profile::EL)?;
```

## EPCIS Data Models

### Core Event Structure

```rust
pub struct EPCISEvent {
    pub event_id: String,
    pub event_type: EPCISEventType,
    pub event_time: SystemTime,
    pub action: EPCISAction,
    pub epc_list: Vec<String>,
    pub biz_step: Option<EPCISBusinessStep>,
    pub disposition: Option<EPCISDisposition>,
    pub read_point: Option<ReadPoint>,
    // ... additional fields
}
```

### Event Types

- **ObjectEvent**: Tracks individual EPCs
- **AggregationEvent**: Groups EPCs into containers
- **TransactionEvent**: Tracks ownership changes
- **TransformationEvent**: Tracks product transformations

### Business Steps

```rust
pub enum EPCISBusinessStep {
    Manufacturing, Assembling, Commissioning,  // Production
    Receiving, Shipping, Loading, Unloading,    // Distribution
    Picking, Packing, Selling,                  // Retail
    Inspecting, Testing, Certifying,            // Quality
    Custom(String),                             // Custom steps
}
```

## Python Integration

### Installation

```bash
# Install Python bindings (requires PyO3)
pip install owl2-reasoner-python
```

### Basic Usage

```python
import owl2_reasoner_python

# Parse EPCIS XML
parser = owl2_reasoner_python.PyEPCISParser()
events = parser.parse_xml_string(epcis_xml)

# Create reasoner
reasoner = owl2_reasoner_python.PyOWL2Reasoner()
reasoner.load_epcis_events(events)

# Perform reasoning
is_consistent = reasoner.is_consistent()
el_valid = reasoner.validate_el_profile()
stats = reasoner.get_statistics()

print(f"Consistent: {is_consistent}")
print(f"EL Profile Valid: {el_valid}")
print(f"Statistics: {stats}")
```

### Data Science Integration

```python
import pandas as pd
import numpy as np

# Convert to DataFrame for analysis
event_data = []
for event in events:
    event_data.append({
        'event_id': event.event_id,
        'event_type': event.event_type,
        'timestamp': event.event_time,
        'epc_count': len(event.epcs),
        'business_step': event.biz_step,
        'disposition': event.disposition
    })

df = pd.DataFrame(event_data)

# Analyze supply chain patterns
print("Business Step Distribution:")
print(df['business_step'].value_counts())

# Time series analysis
df['timestamp'] = pd.to_datetime(df['timestamp'])
df.set_index('timestamp').resample('D')['epc_count'].sum().plot()
```

### EPCIS Data Generation

```python
# Generate synthetic EPCIS data for testing
generator = owl2_reasoner_python.PyEPCISGenerator("medium")
events = generator.generate_events(1000)

print(f"Generated {len(events)} EPCIS events")
print("Configuration:", generator.get_config_info())
```

## Web Service Integration

### Starting the Service

```rust
use owl2_reasoner::web_service::start_web_service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_web_service(3030).await
}
```

### API Endpoints

```
GET  /health              # Service health check
POST /epcis               # Upload EPCIS data
POST /reasoning           # Perform reasoning operations
POST /analysis            # Analyze EPCIS data
GET  /statistics          # Get ontology statistics
```

### Upload EPCIS Data

```bash
curl -X POST http://localhost:3030/epcis \
  -H "Content-Type: application/json" \
  -d '{
    "xml_content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><EPCISDocument>...</EPCISDocument>"
  }'
```

### Perform Reasoning

```bash
curl -X POST http://localhost:3030/reasoning \
  -H "Content-Type: application/json" \
  -d '{
    "check_consistency": true,
    "validate_profiles": ["EL", "QL", "RL"],
    "get_statistics": true
  }'
```

### Analyze Traceability

```bash
curl -X POST http://localhost:3030/analysis \
  -H "Content-Type: application/json" \
  -d '{
    "extract_epcs": true,
    "traceability_analysis": true,
    "business_steps": true
  }'
```

## Data Processing Pipeline

### Pipeline Configuration

```toml
[pipeline]
name = "EPCIS Supply Chain Pipeline"
batch_size = 1000
parallel_processing = true

[inputs]
epcis_xml_files = ["data/*.xml"]
api_endpoints = ["https://api.supplychain.com/epcis"]

[processing]
validation_level = "strict"
reasoning_profiles = ["EL", "QL"]
traceability_analysis = true

[outputs]
database_url = "postgresql://localhost/epcis_db"
export_formats = ["json", "csv", "xml"]
real_time_streaming = true
```

### Processing Stages

1. **Data Ingestion**: Parse EPCIS XML from files or APIs
2. **Validation**: Schema validation and business rule checking
3. **Reasoning**: OWL2 inference and consistency checking
4. **Analysis**: Traceability analysis and pattern detection
5. **Output**: Export to databases, files, or real-time streams

### Example Implementation

```rust
use owl2_reasoner::epcis_parser::*;
use owl2_reasoner::reasoning::SimpleReasoner;

async fn process_epcis_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    // Stage 1: Data Ingestion
    let parser = EPCISDocumentParser::default();
    let events = parser.parse_xml_file("data/supply_chain.xml")?;

    // Stage 2: Validation
    let validation_results = validate_epcis_events(&events);
    if !validation_results.errors.is_empty() {
        eprintln!("Validation errors: {:?}", validation_results.errors);
    }

    // Stage 3: Reasoning
    let ontology = parser.to_ontology(&events)?;
    let reasoner = SimpleReasoner::new(ontology);

    let is_consistent = reasoner.is_consistent()?;
    println!("Consistency check: {}", if is_consistent { "PASS" } else { "FAIL" });

    // Stage 4: Analysis
    let insights = extract_supply_chain_insights(&events);
    for insight in insights {
        println!("Insight: {}", insight);
    }

    // Stage 5: Output
    export_results(&reasoner, "output/results.json")?;

    Ok(())
}
```

## Multi-Language Client Integration

### Java Client

```java
import java.net.http.*;
import java.net.URI;
import com.fasterxml.jackson.databind.*;

public class EPCISClient {
    private final HttpClient client;
    private final ObjectMapper mapper;
    private final String baseUrl;

    public EPCISClient(String baseUrl) {
        this.client = HttpClient.newHttpClient();
        this.mapper = new ObjectMapper();
        this.baseUrl = baseUrl;
    }

    public void uploadEPCISData(String xmlContent) throws Exception {
        var requestBody = Map.of("xml_content", xmlContent);
        var request = HttpRequest.newBuilder()
            .uri(URI.create(baseUrl + "/epcis"))
            .header("Content-Type", "application/json")
            .POST(HttpRequest.BodyPublishers.ofString(
                mapper.writeValueAsString(requestBody)))
            .build();

        var response = client.send(request,
            HttpResponse.BodyHandlers.ofString());
        System.out.println("Upload response: " + response.body());
    }
}
```

### JavaScript/Node.js Client

```javascript
const fetch = require('node-fetch');

class EPCISClient {
    constructor(baseUrl) {
        this.baseUrl = baseUrl;
    }

    async uploadEPCISData(xmlContent) {
        const response = await fetch(`${this.baseUrl}/epcis`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ xml_content: xmlContent })
        });
        return await response.json();
    }

    async performReasoning(options = {}) {
        const defaultOptions = {
            check_consistency: true,
            validate_profiles: ['EL', 'QL', 'RL'],
            get_statistics: true
        };

        const response = await fetch(`${this.baseUrl}/reasoning`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ ...defaultOptions, ...options })
        });
        return await response.json();
    }
}
```

### C# Client

```csharp
using System;
using System.Net.Http;
using System.Text.Json;
using System.Threading.Tasks;

public class EPCISClient
{
    private readonly HttpClient _client;
    private readonly string _baseUrl;

    public EPCISClient(string baseUrl)
    {
        _client = new HttpClient();
        _baseUrl = baseUrl;
    }

    public async Task<string> UploadEPCISDataAsync(string xmlContent)
    {
        var requestBody = new { xml_content = xmlContent };
        var content = new StringContent(
            JsonSerializer.Serialize(requestBody),
            System.Text.Encoding.UTF8,
            "application/json"
        );

        var response = await _client.PostAsync($"{_baseUrl}/epcis", content);
        return await response.Content.ReadAsStringAsync();
    }
}
```

### Go Client

```go
package main

import (
    "bytes"
    "encoding/json"
    "fmt"
    "net/http"
)

type EPCISClient struct {
    baseURL string
    client  *http.Client
}

func NewEPCISClient(baseURL string) *EPCISClient {
    return &EPCISClient{
        baseURL: baseURL,
        client:  &http.Client{},
    }
}

func (c *EPCISClient) UploadEPCISData(xmlContent string) (*http.Response, error) {
    requestBody := map[string]interface{}{
        "xml_content": xmlContent,
    }

    jsonData, err := json.Marshal(requestBody)
    if err != nil {
        return nil, err
    }

    req, err := http.NewRequest("POST", c.baseURL+"/epcis", bytes.NewBuffer(jsonData))
    if err != nil {
        return nil, err
    }

    req.Header.Set("Content-Type", "application/json")
    return c.client.Do(req)
}
```

## Supply Chain Use Cases

### 1. Pharmaceutical Traceability

```python
# Track pharmaceutical products through supply chain
def track_pharmaceutical_product(epcis_events):
    parser = owl2_reasoner_python.PyEPCISParser()
    reasoner = owl2_reasoner_python.PyOWL2Reasoner()

    # Parse EPCIS events
    events = parser.parse_xml_string(epcis_events)
    reasoner.load_epcis_events(events)

    # Verify regulatory compliance
    compliance_check = reasoner.validate_el_profile()

    # Analyze temperature excursions
    temperature_events = [e for e in events if 'temperature' in e.extension]

    # Generate compliance report
    report = {
        'total_events': len(events),
        'compliance_status': compliance_check,
        'temperature_violations': len(temperature_events),
        'unique_products': len(parser.extract_all_epcs(events))
    }

    return report
```

### 2. Food Safety Monitoring

```python
# Monitor food safety through supply chain
def monitor_food_safety(epcis_data):
    # Parse and validate events
    events = parse_epcis_events(epcis_data)

    # Check for recalls or contamination events
    safety_events = [
        event for event in events
        if event.disposition in ['recalled', 'contaminated', 'quarantined']
    ]

    # Trace affected products
    affected_epcs = set()
    for event in safety_events:
        affected_epcs.update(event.epcs)

    # Generate safety alert
    return {
        'safety_alerts': len(safety_events),
        'affected_products': list(affected_epcs),
        'traceability_complete': len(affected_epcs) > 0
    }
```

### 3. Luxury Goods Authentication

```python
# Authenticate luxury goods using EPCIS and OWL2 reasoning
def authenticate_luxury_goods(product_epc, epcis_events):
    # Create product ontology
    reasoner = create_reasoner_from_events(epcis_events)

    # Verify product authenticity
    authenticity_query = f"""
        PREFIX epcis: <http://example.org/epcis/>
        ASK WHERE {{
            <{product_epc}> a epcis:AuthenticProduct .
            <{product_epc}> epcis:hasAuthenticCertificate ?cert .
        }}
    """

    is_authentic = reasoner.query(authenticity_query)

    # Trace ownership history
    ownership_history = trace_ownership(product_epc, reasoner)

    return {
        'is_authentic': is_authentic,
        'ownership_history': ownership_history,
        'verification_timestamp': datetime.now().isoformat()
    }
```

## Performance Optimization

### Caching Strategies

```rust
use owl2_reasoner::cache::{ReasoningCache, CacheConfig};

// Configure caching for EPCIS processing
let cache_config = CacheConfig {
    max_size: 10000,
    ttl_seconds: 3600,
    enable_compression: true,
};

let cache = ReasoningCache::new(cache_config);

// Cache reasoning results
let cache_key = format!("epcis_analysis_{}", hash_epcis_data(&events));
if let Some(cached_result) = cache.get(&cache_key) {
    return Ok(cached_result);
}

let result = perform_epcis_analysis(&events)?;
cache.put(&cache_key, result.clone());
```

### Parallel Processing

```rust
use rayon::prelude::*;

// Process EPCIS events in parallel
fn process_epcis_batch(events: Vec<EPCISEvent>) -> Vec<ProcessingResult> {
    events.par_iter()
        .map(|event| {
            let reasoner = SimpleReasoner::new(create_event_ontology(event));
            reasoner.analyze_event()
        })
        .collect()
}
```

## Deployment

### Docker Deployment

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/owl2-reasoner /usr/local/bin/

EXPOSE 3030
CMD ["owl2-reasoner", "serve", "--port", "3030"]
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: owl2-reasoner-epcis
spec:
  replicas: 3
  selector:
    matchLabels:
      app: owl2-reasoner-epcis
  template:
    metadata:
      labels:
        app: owl2-reasoner-epcis
    spec:
      containers:
      - name: owl2-reasoner
        image: owl2-reasoner:latest
        ports:
        - containerPort: 3030
        env:
        - name: RUST_LOG
          value: "info"
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
```

## Monitoring and Logging

### Metrics Collection

```rust
use prometheus::{Counter, Histogram};

// Define metrics
static EPCIS_EVENTS_PROCESSED: Counter =
    Counter::new("epcis_events_processed", "Total EPCIS events processed");

static REASONING_DURATION: Histogram =
    Histogram::new("reasoning_duration_seconds", "Reasoning operation duration");

// Track metrics
async fn process_epcis_with_metrics(events: Vec<EPCISEvent>) -> Result<(), Error> {
    let timer = REASONING_DURATION.start_timer();

    let result = process_epcis_events(events).await;

    EPCIS_EVENTS_PROCESSED.inc();
    timer.observe_duration();

    result
}
```

## Security Considerations

### Data Privacy

```rust
// Implement data anonymization for sensitive supply chain data
fn anonymize_epcis_data(event: EPCISEvent) -> EPCISEvent {
    EPCISEvent {
        event_id: hash_sensitive_data(&event.event_id),
        epc_list: event.epc_list.iter().map(anonymize_epc).collect(),
        // ... anonymize other sensitive fields
        ..event
    }
}
```

### Access Control

```rust
// Implement role-based access control
enum EPCISAccessRole {
    Manufacturer,
    Distributor,
    Retailer,
    Regulator,
    Consumer,
}

fn check_access_permission(role: EPCISAccessRole, operation: &str) -> bool {
    match (role, operation) {
        (EPCISAccessRole::Manufacturer, "create") => true,
        (EPCISAccessRole::Regulator, "audit") => true,
        (EPCISAccessRole::Consumer, "read") => true,
        _ => false,
    }
}
```

## Best Practices

### 1. Data Validation

```rust
fn validate_epcis_event(event: &EPCISEvent) -> Result<(), ValidationError> {
    if event.event_id.is_empty() {
        return Err(ValidationError::MissingEventId);
    }

    if event.epc_list.is_empty() {
        return Err(ValidationError::MissingEPCs);
    }

    // Validate EPC format
    for epc in &event.epc_list {
        if !is_valid_epc_format(epc) {
            return Err(ValidationError::InvalidEPCFormat(epc.clone()));
        }
    }

    Ok(())
}
```

### 2. Error Handling

```rust
#[derive(Debug)]
pub enum EPCISError {
    ParseError(String),
    ValidationError(String),
    ReasoningError(String),
    NetworkError(String),
}

impl std::fmt::Display for EPCISError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EPCISError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            EPCISError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            EPCISError::ReasoningError(msg) => write!(f, "Reasoning error: {}", msg),
            EPCISError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}
```

### 3. Performance Optimization

```rust
// Use connection pooling for database operations
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;

struct EPCISProcessor {
    db_pool: Pool<PostgresConnectionManager>,
    reasoner_cache: Arc<Mutex<HashMap<String, SimpleReasoner>>>,
}

impl EPCISProcessor {
    async fn process_with_cache(&self, events: Vec<EPCISEvent>) -> Result<AnalysisResult, EPCISError> {
        let cache_key = self.generate_cache_key(&events);

        // Check cache first
        if let Some(cached_result) = self.reasoner_cache.lock().unwrap().get(&cache_key) {
            return Ok(cached_result.clone());
        }

        // Process and cache result
        let result = self.process_events(events).await?;
        self.reasoner_cache.lock().unwrap().insert(cache_key, result.clone());

        Ok(result)
    }
}
```

## Conclusion

The OWL2 reasoner's EPCIS ecosystem integration provides a comprehensive solution for supply chain traceability and reasoning. With support for multiple programming languages, robust data models, and scalable deployment options, it enables organizations to build sophisticated supply chain applications that leverage semantic reasoning for enhanced traceability and compliance.

Key benefits include:

- **Standards Compliance**: Full GS1 EPCIS 2.0 support
- **Semantic Reasoning**: OWL2-based inference for complex supply chain logic
- **Multi-Platform**: Python, Java, JavaScript, C#, Go, and more
- **Scalable**: From small deployments to enterprise-scale systems
- **Extensible**: Plugin architecture for custom business rules
- **Production Ready**: Monitoring, logging, and security features

For more information and examples, see:
- [Documentation Overview](docs/README.md) - Complete documentation structure
- [Examples Directory](../examples/) - Working code examples
- [API Documentation](docs/API_REFERENCE.md) - Complete API reference
- [Architecture Guide](docs/architecture/ARCHITECTURE.md) - System architecture