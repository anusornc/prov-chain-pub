# OWL2 Reasoner Ecosystem Integration Guide

This comprehensive guide demonstrates how to integrate the OWL2 reasoner with EPCIS data across different programming languages, platforms, and deployment scenarios.

## Overview

The OWL2 reasoner provides robust EPCIS (Electronic Product Code Information Service) integration capabilities for supply chain traceability, semantic reasoning, and knowledge graph applications. This guide covers:

- **Python Bindings** - Native Python API for easy integration
- **Web Services** - RESTful API for web and mobile applications
- **Data Processing Pipelines** - Stream processing for big data scenarios
- **Language Bindings** - Integration with multiple programming languages
- **Deployment Patterns** - Production deployment strategies

## Table of Contents

1. [Quick Start](#quick-start)
2. [Python Integration](#python-integration)
3. [Web Service Integration](#web-service-integration)
4. [Data Processing Pipelines](#data-processing-pipelines)
5. [Language Bindings](#language-bindings)
6. [Deployment Patterns](#deployment-patterns)
7. [Performance Optimization](#performance-optimization)
8. [Troubleshooting](#troubleshooting)

## Quick Start

### Prerequisites

- Rust 1.70+ installed
- Python 3.8+ (for Python bindings)
- Basic understanding of OWL2 and EPCIS standards

### Setup

```bash
# Clone the repository
git clone https://github.com/your-org/owl2-reasoner.git
cd owl2-reasoner

# Build the core library
cargo build --release

# Install Python dependencies (for Python bindings)
pip install maturin pyo3

# Build Python bindings
maturin develop

# Run tests
cargo test
```

### Basic Usage

```rust
use owl2_reasoner::epcis::*;
use owl2_reasoner::epcis_parser::*;
use owl2_reasoner::reasoning::SimpleReasoner;

// Create EPCIS event
let event = EPCISEvent::new("event_001".to_string(), EPCISEventType::ObjectEvent)
    .with_business_step(EPCISBusinessStep::Receiving)
    .with_disposition(EPCISDisposition::InStock);

// Convert to OWL2 ontology
let (ontology, individuals) = event.to_owl2()?;

// Create reasoner and perform reasoning
let mut reasoner = SimpleReasoner::new(ontology);
let is_consistent = reasoner.is_consistent()?;
println!("Ontology is consistent: {}", is_consistent);
```

## Python Integration

### Installation

```bash
# Install from PyPI (when published)
pip install owl2-reasoner

# Or build from source
maturin develop --release
```

### Basic Python Example

```python
import owl2_reasoner_python as owl2
import json

# Create reasoner
reasoner = owl2.create_reasoner()

# Parse EPCIS XML
sample_xml = """
<?xml version="1.0" encoding="UTF-8"?>
<EPCISDocument xmlns="urn:epcglobal:epcis:xsd:2" schemaVersion="2.0">
    <EPCISBody>
        <EventList>
            <ObjectEvent>
                <eventTime>2023-01-01T10:00:00Z</eventTime>
                <epcList>
                    <epc>urn:epc:id:sgtin:0614141.107346.2023</epc>
                </epcList>
                <action>ADD</action>
                <bizStep>urn:epcglobal:cbv:bizstep:receiving</bizStep>
            </ObjectEvent>
        </EventList>
    </EPCISBody>
</EPCISDocument>
"""

# Parse and load events
events = owl2.parse_epcis_xml_string(sample_xml)
reasoner.load_epcis_events(events)

# Perform reasoning
is_consistent = reasoner.is_consistent()
print(f"Consistent: {is_consistent}")

# Validate profiles
el_valid = reasoner.validate_el_profile()
ql_valid = reasoner.validate_ql_profile()
rl_valid = reasoner.validate_rl_profile()

print(f"EL Profile: {el_valid}")
print(f"QL Profile: {ql_valid}")
print(f"RL Profile: {rl_valid}")

# Get statistics
stats = reasoner.get_statistics()
print(f"Statistics: {stats}")
```

### Advanced Python Integration

```python
import owl2_reasoner_python as owl2
import pandas as pd
import matplotlib.pyplot as plt
from datetime import datetime

class EPCISAnalyzer:
    def __init__(self):
        self.reasoner = owl2.create_reasoner()
        self.parser = owl2.PyEPCISParser()
        self.generator = owl2.create_epcis_generator("medium")

    def load_from_file(self, file_path):
        """Load EPCIS data from XML file"""
        events = self.parser.parse_xml_file(file_path)
        self.reasoner.load_epcis_events(events)
        return events

    def analyze_supply_chain(self):
        """Perform comprehensive supply chain analysis"""
        stats = self.reasoner.get_statistics()
        is_consistent = self.reasoner.is_consistent()

        analysis = {
            'timestamp': datetime.now().isoformat(),
            'ontology_stats': stats,
            'consistency': is_consistent,
            'profile_validation': {
                'EL': self.reasoner.validate_el_profile(),
                'QL': self.reasoner.validate_ql_profile(),
                'RL': self.reasoner.validate_rl_profile()
            }
        }

        return analysis

    def generate_report(self, analysis):
        """Generate analysis report with visualizations"""
        fig, axes = plt.subplots(2, 2, figsize=(12, 8))

        # Ontology statistics
        stats = analysis['ontology_stats']
        categories = list(stats.keys())
        values = list(stats.values())

        axes[0, 0].bar(categories, values)
        axes[0, 0].set_title('Ontology Statistics')
        axes[0, 0].tick_params(axis='x', rotation=45)

        # Profile validation
        profiles = ['EL', 'QL', 'RL']
        validation_scores = [
            analysis['profile_validation']['EL'],
            analysis['profile_validation']['QL'],
            analysis['profile_validation']['RL']
        ]

        axes[0, 1].pie(validation_scores, labels=profiles, autopct='%1.0f%%')
        axes[0, 1].set_title('Profile Validation')

        # Consistency status
        axes[1, 0].pie([analysis['consistency'], not analysis['consistency']],
                      labels=['Consistent', 'Inconsistent'], autopct='%1.0f%%')
        axes[1, 0].set_title('Consistency Status')

        plt.tight_layout()
        plt.savefig('epcis_analysis_report.png')
        plt.close()

        return 'epcis_analysis_report.png'

# Usage example
analyzer = EPCISAnalyzer()
analysis = analyzer.analyze_supply_chain()
report_image = analyzer.generate_report(analysis)
print(f"Analysis complete. Report saved to: {report_image}")
```

### Python Data Science Integration

```python
import owl2_reasoner_python as owl2
import numpy as np
import networkx as nx
from sklearn.cluster import DBSCAN
from typing import Dict, List, Any

class EPCISDataScience:
    def __init__(self):
        self.reasoner = owl2.create_reasoner()
        self.parser = owl2.PyEPCISParser()

    def create_knowledge_graph(self, events):
        """Create NetworkX knowledge graph from EPCIS events"""
        G = nx.Graph()

        for event in events:
            # Add event node
            G.add_node(event.event_id, type='event',
                      event_type=event.event_type,
                      timestamp=event.event_time)

            # Add EPC nodes and connect to event
            for epc in event.epcs:
                G.add_node(epc, type='epc')
                G.add_edge(event.event_id, epc, relation='involves')

            # Add business step
            if event.biz_step:
                G.add_node(event.biz_step, type='biz_step')
                G.add_edge(event.event_id, event.biz_step, relation='has_step')

        return G

    def cluster_epcs(self, events):
        """Cluster EPCs based on co-occurrence patterns"""
        # Create EPC co-occurrence matrix
        all_epcs = list(set(epc for event in events for epc in event.epcs))
        epc_to_index = {epc: i for i, epc in enumerate(all_epcs)}

        matrix = np.zeros((len(all_epcs), len(all_epcs)))

        for event in events:
            for i, epc1 in enumerate(event.epcs):
                for j, epc2 in enumerate(event.epcs):
                    if i != j:
                        idx1 = epc_to_index[epc1]
                        idx2 = epc_to_index[epc2]
                        matrix[idx1][idx2] += 1

        # Apply DBSCAN clustering
        clustering = DBSCAN(eps=0.5, min_samples=2).fit(matrix)

        return {
            'epcs': all_epcs,
            'clusters': clustering.labels_,
            'noise_count': list(clustering.labels_).count(-1)
        }

    def analyze_temporal_patterns(self, events):
        """Analyze temporal patterns in EPCIS events"""
        import pandas as pd

        # Convert to DataFrame
        df = pd.DataFrame([
            {
                'event_id': event.event_id,
                'timestamp': pd.to_datetime(event.event_time),
                'event_type': event.event_type,
                'epc_count': len(event.epcs)
            }
            for event in events
        ])

        # Temporal analysis
        hourly_pattern = df.groupby(df['timestamp'].dt.hour)['epc_count'].sum()
        daily_pattern = df.groupby(df['timestamp'].dt.date)['epc_count'].sum()

        return {
            'hourly_pattern': hourly_pattern.to_dict(),
            'daily_pattern': daily_pattern.to_dict(),
            'peak_hour': hourly_pattern.idxmax(),
            'peak_day': daily_pattern.idxmax()
        }

# Usage example
ds_analyzer = EPCISDataScience()
events = ds_analyzer.parser.parse_xml_file('supply_chain_events.xml')
kg = ds_analyzer.create_knowledge_graph(events)
clusters = ds_analyzer.cluster_epcs(events)
temporal = ds_analyzer.analyze_temporal_patterns(events)

print(f"Knowledge graph: {kg.number_of_nodes()} nodes, {kg.number_of_edges()} edges")
print(f"EPC clusters: {len(set(clusters['clusters'])) - clusters['noise_count']} clusters")
print(f"Peak activity hour: {temporal['peak_hour']}")
```

## Web Service Integration

### Starting the Web Service

```rust
use owl2_reasoner::web_service::start_web_service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting OWL2 Reasoner Web Service...");
    start_web_service(3030).await
}
```

### API Endpoints

#### Health Check
```http
GET /health
```

Response:
```json
{
  "status": "healthy",
  "service": "OWL2 Reasoner Web Service",
  "version": "1.0.0",
  "timestamp": "2023-01-01T00:00:00Z",
  "uptime_seconds": 3600
}
```

#### Upload EPCIS Data
```http
POST /epcis
Content-Type: application/json

{
  "xml_content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?>...",
  "file_path": "/path/to/epcis_file.xml"
}
```

Response:
```json
{
  "status": "success",
  "message": "EPCIS data uploaded and processed successfully",
  "events_processed": 100,
  "execution_time_ms": 150,
  "timestamp": "2023-01-01T00:00:00Z"
}
```

#### Perform Reasoning
```http
POST /reasoning
Content-Type: application/json

{
  "check_consistency": true,
  "validate_profiles": ["EL", "QL", "RL"],
  "get_statistics": true
}
```

Response:
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2023-01-01T00:00:00Z",
  "consistency": true,
  "profile_validation": {
    "EL": true,
    "QL": true,
    "RL": false
  },
  "statistics": {
    "classes": 25,
    "object_properties": 15,
    "data_properties": 8,
    "individuals": 150,
    "axioms": 200
  },
  "execution_time_ms": 45
}
```

#### EPCIS Analysis
```http
POST /analysis
Content-Type: application/json

{
  "extract_epcs": true,
  "event_type_counts": true,
  "business_steps": true,
  "traceability_analysis": true
}
```

Response:
```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2023-01-01T00:00:00Z",
  "total_events": 100,
  "unique_epcs": ["urn:epc:id:sgtin:0614141.107346.2023"],
  "event_type_counts": {
    "ObjectEvent": 80,
    "AggregationEvent": 20
  },
  "business_steps": [
    "urn:epcglobal:cbv:bizstep:receiving",
    "urn:epcglobal:cbv:bizstep:shipping"
  ],
  "traceability_summary": "Traceability Analysis: 150 entities across 25 classes with 200 logical relationships",
  "execution_time_ms": 120
}
```

### Client Integration Examples

#### Rust Client
```rust
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let base_url = "http://localhost:3030";

    // Upload EPCIS data
    let upload_data = json!({
        "xml_content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?>..."
    });

    let response = client
        .post(&format!("{}/epcis", base_url))
        .json(&upload_data)
        .send()
        .await?;

    println!("Upload response: {:?}", response.text().await?);

    // Perform reasoning
    let reasoning_data = json!({
        "check_consistency": true,
        "validate_profiles": ["EL", "QL", "RL"],
        "get_statistics": true
    });

    let response = client
        .post(&format!("{}/reasoning", base_url))
        .json(&reasoning_data)
        .send()
        .await?;

    println!("Reasoning response: {:?}", response.text().await?);

    Ok(())
}
```

#### Python Client
```python
import requests
import json

class OWL2ReasonerClient:
    def __init__(self, base_url="http://localhost:3030"):
        self.base_url = base_url
        self.session = requests.Session()

    def health_check(self):
        """Check service health"""
        response = self.session.get(f"{self.base_url}/health")
        return response.json()

    def upload_epcis(self, xml_content=None, file_path=None):
        """Upload EPCIS data"""
        data = {}
        if xml_content:
            data["xml_content"] = xml_content
        if file_path:
            data["file_path"] = file_path

        response = self.session.post(f"{self.base_url}/epcis", json=data)
        return response.json()

    def reason(self, check_consistency=True, profiles=None, get_stats=True):
        """Perform reasoning operations"""
        data = {
            "check_consistency": check_consistency,
            "validate_profiles": profiles or ["EL", "QL", "RL"],
            "get_statistics": get_stats
        }

        response = self.session.post(f"{self.base_url}/reasoning", json=data)
        return response.json()

    def analyze(self, extract_epcs=True, event_type_counts=True,
               business_steps=True, traceability_analysis=True):
        """Analyze EPCIS data"""
        data = {
            "extract_epcs": extract_epcs,
            "event_type_counts": event_type_counts,
            "business_steps": business_steps,
            "traceability_analysis": traceability_analysis
        }

        response = self.session.post(f"{self.base_url}/analysis", json=data)
        return response.json()

    def get_statistics(self):
        """Get ontology statistics"""
        response = self.session.get(f"{self.base_url}/statistics")
        return response.json()

# Usage example
client = OWL2ReasonerClient()

# Check health
health = client.health_check()
print(f"Service health: {health}")

# Upload EPCIS data
with open("supply_chain.xml", "r") as f:
    xml_content = f.read()
upload_result = client.upload_epcis(xml_content=xml_content)
print(f"Upload result: {upload_result}")

# Perform reasoning
reasoning_result = client.reason()
print(f"Reasoning result: {reasoning_result}")

# Analyze data
analysis_result = client.analyze()
print(f"Analysis result: {analysis_result}")
```

#### JavaScript Client
```javascript
class OWL2ReasonerClient {
    constructor(baseUrl = 'http://localhost:3030') {
        this.baseUrl = baseUrl;
    }

    async healthCheck() {
        const response = await fetch(`${this.baseUrl}/health`);
        return response.json();
    }

    async uploadEPCIS(xmlContent) {
        const response = await fetch(`${this.baseUrl}/epcis`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ xml_content: xmlContent })
        });
        return response.json();
    }

    async reason(options = {}) {
        const data = {
            check_consistency: options.checkConsistency ?? true,
            validate_profiles: options.profiles ?? ['EL', 'QL', 'RL'],
            get_statistics: options.getStats ?? true
        };

        const response = await fetch(`${this.baseUrl}/reasoning`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        });
        return response.json();
    }

    async analyze(options = {}) {
        const data = {
            extract_epcs: options.extractEpcs ?? true,
            event_type_counts: options.eventTypeCounts ?? true,
            business_steps: options.businessSteps ?? true,
            traceability_analysis: options.traceabilityAnalysis ?? true
        };

        const response = await fetch(`${this.baseUrl}/analysis`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        });
        return response.json();
    }
}

// Usage example
const client = new OWL2ReasonerClient();

// Check health
client.healthCheck().then(health => {
    console.log('Service health:', health);
});

// Upload EPCIS data
fetch('supply_chain.xml')
    .then(response => response.text())
    .then(xmlContent => {
        return client.uploadEPCIS(xmlContent);
    })
    .then(result => {
        console.log('Upload result:', result);
        return client.reason();
    })
    .then(reasoningResult => {
        console.log('Reasoning result:', reasoningResult);
        return client.analyze();
    })
    .then(analysisResult => {
        console.log('Analysis result:', analysisResult);
    })
    .catch(error => {
        console.error('Error:', error);
    });
```

## Data Processing Pipelines

### Pipeline Configuration

```rust
use owl2_reasoner::pipeline::*;

let config = PipelineConfig {
    batch_size: 1000,
    max_concurrent_tasks: 4,
    enable_caching: true,
    output_format: OutputFormat::JSON,
    validation_level: ValidationLevel::Comprehensive,
};
```

### Multi-Source Processing

```rust
use owl2_reasoner::pipeline::{EPCISPipeline, EPCISSource, PipelineConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = PipelineConfig::default();
    let mut pipeline = EPCISPipeline::new(config);

    // Define multiple data sources
    let sources = vec![
        EPCISSource::File("data/warehouse_events.xml".to_string()),
        EPCISSource::Directory("data/supplier_data/".to_string()),
        EPCISSource::Generated(EPCISDataConfig::large_scale()),
    ];

    // Process all sources
    let result = pipeline.process_multiple_sources(sources).await?;

    println!("Processed {} events", result.pipeline_metrics.total_events_processed);
    println!("Throughput: {:.2} events/sec", result.pipeline_metrics.throughput_events_per_sec);

    Ok(())
}
```

### Real-time Stream Processing

```rust
use owl2_reasoner::pipeline::*;
use tokio::sync::mpsc;

async fn process_real_time_events() -> Result<(), Box<dyn std::error::Error>> {
    let config = PipelineConfig {
        batch_size: 100,
        max_concurrent_tasks: 2,
        enable_caching: true,
        output_format: OutputFormat::JSON,
        validation_level: ValidationLevel::Strict,
    };

    let (event_tx, event_rx) = mpsc::channel(1000);
    let mut pipeline = EPCISPipeline::new(config);

    // Simulate real-time event stream
    tokio::spawn(async move {
        for i in 0..1000 {
            let event = EPCISSimpleEvent {
                event_id: format!("realtime_{}", i),
                event_type: "ObjectEvent".to_string(),
                event_time: chrono::Utc::now().to_rfc3339(),
                epcs: vec![format!("urn:epc:id:sgtin:0614141.107346.{}", i)],
                biz_step: Some("urn:epcglobal:cbv:bizstep:receiving".to_string()),
                disposition: Some("urn:epcglobal:cbv:disp:in_progress".to_string()),
                action: "ADD".to_string(),
            };

            let _ = event_tx.send(vec![event]).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    });

    // Process events as they arrive
    let events_stream = ReceiverStream::new(event_rx);
    let processed_stream = pipeline.process_events_stream(events_stream);
    let results = pipeline.collect_results(processed_stream).await;

    println!("Real-time processing complete: {:?}", results.pipeline_metrics);

    Ok(())
}
```

### Monitoring and Alerting

```rust
use owl2_reasoner::pipeline::PipelineMonitor;

async fn monitor_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    let mut monitor = PipelineMonitor::new();

    // Simulate monitoring data
    let metrics_data = vec![
        ("throughput", 150.5),
        ("error_rate", 0.02),
        ("memory_usage", 512.0),
        ("processing_time", 1200.0),
    ];

    for (name, value) in metrics_data {
        monitor.update_metrics(name, value);
    }

    // Get monitoring status
    let status = monitor.get_status();
    println!("Monitoring status: {}", serde_json::to_string_pretty(&status)?);

    // Check for alerts
    if status["alert_count"].as_u64().unwrap() > 0 {
        println!("⚠️  Alerts detected:");
        for alert in status["alerts"].as_array().unwrap() {
            println!("   - {}", alert);
        }
    }

    Ok(())
}
```

## Language Bindings

### Java Integration

```java
// OWL2ReasonerJava.java - Example Java bindings
import java.io.*;
import java.net.http.*;
import java.net.URI;
import org.json.*;

public class OWL2ReasonerClient {
    private final String baseUrl;
    private final HttpClient client;

    public OWL2ReasonerClient(String baseUrl) {
        this.baseUrl = baseUrl;
        this.client = HttpClient.newHttpClient();
    }

    public JSONObject healthCheck() throws Exception {
        HttpRequest request = HttpRequest.newBuilder()
            .uri(URI.create(baseUrl + "/health"))
            .GET()
            .build();

        HttpResponse<String> response = client.send(request,
            HttpResponse.BodyHandlers.ofString());
        return new JSONObject(response.body());
    }

    public JSONObject uploadEPCIS(String xmlContent) throws Exception {
        JSONObject data = new JSONObject();
        data.put("xml_content", xmlContent);

        HttpRequest request = HttpRequest.newBuilder()
            .uri(URI.create(baseUrl + "/epcis"))
            .header("Content-Type", "application/json")
            .POST(HttpRequest.BodyPublishers.ofString(data.toString()))
            .build();

        HttpResponse<String> response = client.send(request,
            HttpResponse.BodyHandlers.ofString());
        return new JSONObject(response.body());
    }

    public JSONObject performReasoning(boolean checkConsistency,
                                      String[] profiles,
                                      boolean getStats) throws Exception {
        JSONObject data = new JSONObject();
        data.put("check_consistency", checkConsistent);
        data.put("validate_profiles", new JSONArray(profiles));
        data.put("get_statistics", getStats);

        HttpRequest request = HttpRequest.newBuilder()
            .uri(URI.create(baseUrl + "/reasoning"))
            .header("Content-Type", "application/json")
            .POST(HttpRequest.BodyPublishers.ofString(data.toString()))
            .build();

        HttpResponse<String> response = client.send(request,
            HttpResponse.BodyHandlers.ofString());
        return new JSONObject(response.body());
    }

    public static void main(String[] args) {
        try {
            OWL2ReasonerClient client = new OWL2ReasonerClient("http://localhost:3030");

            // Health check
            JSONObject health = client.healthCheck();
            System.out.println("Health: " + health);

            // Upload EPCIS data
            String xmlContent = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>...";
            JSONObject uploadResult = client.uploadEPCIS(xmlContent);
            System.out.println("Upload result: " + uploadResult);

            // Perform reasoning
            String[] profiles = {"EL", "QL", "RL"};
            JSONObject reasoningResult = client.performReasoning(true, profiles, true);
            System.out.println("Reasoning result: " + reasoningResult);

        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
```

### C# Integration

```csharp
// OWL2ReasonerClient.cs - Example C# bindings
using System;
using System.Net.Http;
using System.Net.Http.Json;
using System.Text.Json;
using System.Threading.Tasks;

public class OWL2ReasonerClient
{
    private readonly HttpClient _client;
    private readonly string _baseUrl;

    public OWL2ReasonerClient(string baseUrl = "http://localhost:3030")
    {
        _baseUrl = baseUrl;
        _client = new HttpClient();
    }

    public async Task<HealthResponse> HealthCheckAsync()
    {
        var response = await _client.GetFromJsonAsync<HealthResponse>($"{_baseUrl}/health");
        return response;
    }

    public async Task<UploadResponse> UploadEPCISAsync(string xmlContent)
    {
        var data = new { xml_content = xmlContent };
        var response = await _client.PostAsJsonAsync($"{_baseUrl}/epcis", data);
        return await response.Content.ReadFromJsonAsync<UploadResponse>();
    }

    public async Task<ReasoningResponse> ReasonAsync(ReasoningRequest request)
    {
        var response = await _client.PostAsJsonAsync($"{_baseUrl}/reasoning", request);
        return await response.Content.ReadFromJsonAsync<ReasoningResponse>();
    }

    public async Task<AnalysisResponse> AnalyzeAsync(AnalysisRequest request)
    {
        var response = await _client.PostAsJsonAsync($"{_baseUrl}/analysis", request);
        return await response.Content.ReadFromJsonAsync<AnalysisResponse>();
    }
}

public class HealthResponse
{
    public string Status { get; set; }
    public string Service { get; set; }
    public string Version { get; set; }
    public string Timestamp { get; set; }
    public long UptimeSeconds { get; set; }
}

public class ReasoningRequest
{
    public bool CheckConsistency { get; set; } = true;
    public string[] ValidateProfiles { get; set; } = new[] { "EL", "QL", "RL" };
    public bool GetStatistics { get; set; } = true;
}

// Usage example
public class Program
{
    public static async Task Main(string[] args)
    {
        var client = new OWL2ReasonerClient();

        // Health check
        var health = await client.HealthCheckAsync();
        Console.WriteLine($"Service status: {health.Status}");

        // Upload EPCIS data
        string xmlContent = File.ReadAllText("supply_chain.xml");
        var uploadResult = await client.UploadEPCISAsync(xmlContent);
        Console.WriteLine($"Uploaded {uploadResult.EventsProcessed} events");

        // Perform reasoning
        var reasoningRequest = new ReasoningRequest();
        var reasoningResult = await client.ReasonAsync(reasoningRequest);
        Console.WriteLine($"Consistent: {reasoningResult.Consistency}");
    }
}
```

### Go Integration

```go
// owl2_client.go - Example Go bindings
package main

import (
    "bytes"
    "encoding/json"
    "fmt"
    "io/ioutil"
    "net/http"
    "time"
)

type OWL2ReasonerClient struct {
    BaseURL    string
    HTTPClient *http.Client
}

func NewOWL2ReasonerClient(baseURL string) *OWL2ReasonerClient {
    return &OWL2ReasonerClient{
        BaseURL: baseURL,
        HTTPClient: &http.Client{
            Timeout: 30 * time.Second,
        },
    }
}

type HealthResponse struct {
    Status       string `json:"status"`
    Service      string `json:"service"`
    Version      string `json:"version"`
    Timestamp    string `json:"timestamp"`
    UptimeSeconds int64 `json:"uptime_seconds"`
}

type UploadRequest struct {
    XMLContent string `json:"xml_content"`
    FilePath   string `json:"file_path"`
}

type UploadResponse struct {
    Status           string `json:"status"`
    Message          string `json:"message"`
    EventsProcessed  int    `json:"events_processed"`
    ExecutionTimeMS  int64  `json:"execution_time_ms"`
    Timestamp        string `json:"timestamp"`
}

func (c *OWL2ReasonerClient) HealthCheck() (*HealthResponse, error) {
    resp, err := c.HTTPClient.Get(c.BaseURL + "/health")
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()

    var health HealthResponse
    if err := json.NewDecoder(resp.Body).Decode(&health); err != nil {
        return nil, err
    }

    return &health, nil
}

func (c *OWL2ReasonerClient) UploadEPCIS(xmlContent string) (*UploadResponse, error) {
    uploadReq := UploadRequest{
        XMLContent: xmlContent,
    }

    body, err := json.Marshal(uploadReq)
    if err != nil {
        return nil, err
    }

    resp, err := c.HTTPClient.Post(
        c.BaseURL+"/epcis",
        "application/json",
        bytes.NewBuffer(body),
    )
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()

    var uploadResp UploadResponse
    if err := json.NewDecoder(resp.Body).Decode(&uploadResp); err != nil {
        return nil, err
    }

    return &uploadResp, nil
}

func main() {
    client := NewOWL2ReasonerClient("http://localhost:3030")

    // Health check
    health, err := client.HealthCheck()
    if err != nil {
        fmt.Printf("Health check failed: %v\n", err)
        return
    }
    fmt.Printf("Service status: %s\n", health.Status)

    // Upload EPCIS data
    xmlContent := `<?xml version="1.0" encoding="UTF-8"?>
    <EPCISDocument xmlns="urn:epcglobal:epcis:xsd:2" schemaVersion="2.0">
        <EPCISBody>
            <EventList>
                <ObjectEvent>
                    <eventTime>2023-01-01T10:00:00Z</eventTime>
                    <epcList>
                        <epc>urn:epc:id:sgtin:0614141.107346.2023</epc>
                    </epcList>
                    <action>ADD</action>
                </ObjectEvent>
            </EventList>
        </EPCISBody>
    </EPCISDocument>`

    uploadResult, err := client.UploadEPCIS(xmlContent)
    if err != nil {
        fmt.Printf("Upload failed: %v\n", err)
        return
    }
    fmt.Printf("Uploaded %d events\n", uploadResult.EventsProcessed)
}
```

## Deployment Patterns

### Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.70-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/owl2-reasoner-web /usr/local/bin/
COPY --from=builder /app/examples/ /usr/local/examples/

EXPOSE 3030

CMD ["owl2-reasoner-web"]
```

```docker-compose.yml
version: '3.8'

services:
  owl2-reasoner:
    build: .
    ports:
      - "3030:3030"
    environment:
      - RUST_LOG=info
      - OWL2_PORT=3030
    volumes:
      - ./data:/app/data
      - ./logs:/app/logs
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    restart: unless-stopped

  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=owl2_reasoner
      - POSTGRES_USER=owl2
      - POSTGRES_PASSWORD=owl2_password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped

volumes:
  postgres_data:
```

### Kubernetes Deployment

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: owl2-reasoner
spec:
  replicas: 3
  selector:
    matchLabels:
      app: owl2-reasoner
  template:
    metadata:
      labels:
        app: owl2-reasoner
    spec:
      containers:
      - name: owl2-reasoner
        image: owl2-reasoner:latest
        ports:
        - containerPort: 3030
        env:
        - name: RUST_LOG
          value: "info"
        - name: OWL2_PORT
          value: "3030"
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3030
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 3030
          initialDelaySeconds: 5
          periodSeconds: 5

---
apiVersion: v1
kind: Service
metadata:
  name: owl2-reasoner-service
spec:
  selector:
    app: owl2-reasoner
  ports:
  - port: 80
    targetPort: 3030
  type: LoadBalancer
```

### Cloud Deployment (AWS)

```bash
# Deploy to AWS ECS
aws ecs register-task-definition \
    --family owl2-reasoner \
    --network-mode awsvpc \
    --requires-compatibilities FARGATE \
    --cpu "1024" \
    --memory "2048" \
    --execution-role-arn arn:aws:iam::123456789012:role/ecsTaskExecutionRole \
    --container-definitions '[{
        "name": "owl2-reasoner",
        "image": "123456789012.dkr.ecr.us-east-1.amazonaws.com/owl2-reasoner:latest",
        "portMappings": [
            {
                "containerPort": 3030,
                "protocol": "tcp"
            }
        ],
        "environment": [
            {
                "name": "RUST_LOG",
                "value": "info"
            }
        ],
        "logConfiguration": {
            "logDriver": "awslogs",
            "options": {
                "awslogs-group": "/ecs/owl2-reasoner",
                "awslogs-region": "us-east-1",
                "awslogs-stream-prefix": "ecs"
            }
        }
    }]'

# Create ECS service
aws ecs create-service \
    --cluster owl2-reasoner-cluster \
    --service-name owl2-reasoner-service \
    --task-definition owl2-reasoner \
    --desired-count 2 \
    --launch-type FARGATE \
    --network-configuration "awsvpcConfiguration={subnets=[subnet-12345],securityGroups=[sg-12345],assignPublicIp=ENABLED}"
```

### Serverless Deployment (AWS Lambda)

```rust
// lambda_function.rs - AWS Lambda integration
use lambda_runtime::{handler_fn, Context, Error};
use serde_json::{json, Value};
use owl2_reasoner::web_service::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(handler_fn(func)).await?;
    Ok(())
}

async fn func(event: Value, _: Context) -> Result<Value, Error> {
    let operation = event["operation"].as_str().unwrap_or("unknown");

    match operation {
        "health" => {
            Ok(json!({
                "status": "healthy",
                "service": "OWL2 Reasoner Lambda",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        },
        "reason" => {
            // Simplified reasoning for lambda
            Ok(json!({
                "consistent": true,
                "el_profile": true,
                "ql_profile": true,
                "rl_profile": false,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        },
        _ => {
            Ok(json!({
                "error": "Unknown operation",
                "operation": operation
            }))
        }
    }
}
```

## Performance Optimization

### Caching Strategies

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub ttl_seconds: u64,
    pub max_size: usize,
    pub enable_compression: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ttl_seconds: 300, // 5 minutes
            max_size: 1000,
            enable_compression: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: Instant,
    pub ttl: Duration,
}

impl<T> CacheEntry<T> {
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

pub struct ReasoningCache<T> {
    cache: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    config: CacheConfig,
}

impl<T: Clone> ReasoningCache<T> {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub fn get(&self, key: &str) -> Option<T> {
        let cache = self.cache.read().unwrap();
        if let Some(entry) = cache.get(key) {
            if !entry.is_expired() {
                return Some(entry.value.clone());
            }
        }
        None
    }

    pub fn put(&self, key: String, value: T) {
        let mut cache = self.cache.write().unwrap();

        // Remove expired entries
        cache.retain(|_, entry| !entry.is_expired());

        // Remove oldest entries if at capacity
        if cache.len() >= self.config.max_size {
            let oldest_key = cache.iter()
                .min_by_key(|(_, entry)| entry.created_at)
                .map(|(key, _)| key.clone());

            if let Some(key) = oldest_key {
                cache.remove(&key);
            }
        }

        let entry = CacheEntry {
            value,
            created_at: Instant::now(),
            ttl: Duration::from_secs(self.config.ttl_seconds),
        };

        cache.insert(key, entry);
    }

    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }
}
```

### Connection Pooling

```rust
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::env;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(3600),
        }
    }
}

pub struct DatabaseManager {
    pool: Pool<Postgres>,
}

impl DatabaseManager {
    pub async fn new(config: DatabaseConfig) -> Result<Self, sqlx::Error> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://owl2:owl2_password@localhost/owl2_reasoner".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .connect(&database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn execute_query(&self, query: &str, params: Vec<&str>) -> Result<sqlx::PgQueryResult, sqlx::Error> {
        sqlx::query(query)
            .bind_all(params)
            .execute(&self.pool)
            .await
    }

    pub async fn fetch_one<T>(&self, query: &str) -> Result<T, sqlx::Error>
    where
        T: sqlx::FromRow<'_, sqlx::postgres::PgRow> + Unpin + Send,
    {
        sqlx::query_as::<_, T>(query)
            .fetch_one(&self.pool)
            .await
    }
}
```

### Performance Monitoring

```rust
use prometheus::{Counter, Histogram, Gauge, Registry};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Metrics {
    pub registry: Arc<Registry>,
    pub events_processed: Counter,
    pub reasoning_duration: Histogram,
    pub active_connections: Gauge,
    pub error_count: Counter,
    pub cache_hits: Counter,
    pub cache_misses: Counter,
}

impl Metrics {
    pub fn new() -> Self {
        let registry = Arc::new(Registry::new());

        let events_processed = Counter::new("owl2_events_processed_total", "Total events processed")
            .unwrap();
        let reasoning_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new("owl2_reasoning_duration_seconds", "Reasoning duration in seconds")
        ).unwrap();
        let active_connections = Gauge::new("owl2_active_connections", "Active database connections")
            .unwrap();
        let error_count = Counter::new("owl2_errors_total", "Total errors encountered")
            .unwrap();
        let cache_hits = Counter::new("owl2_cache_hits_total", "Total cache hits")
            .unwrap();
        let cache_misses = Counter::new("owl2_cache_misses_total", "Total cache misses")
            .unwrap();

        registry.register(Box::new(events_processed.clone())).unwrap();
        registry.register(Box::new(reasoning_duration.clone())).unwrap();
        registry.register(Box::new(active_connections.clone())).unwrap();
        registry.register(Box::new(error_count.clone())).unwrap();
        registry.register(Box::new(cache_hits.clone())).unwrap();
        registry.register(Box::new(cache_misses.clone())).unwrap();

        Self {
            registry,
            events_processed,
            reasoning_duration,
            active_connections,
            error_count,
            cache_hits,
            cache_misses,
        }
    }

    pub fn increment_events(&self) {
        self.events_processed.inc();
    }

    pub fn observe_reasoning_duration(&self, duration: f64) {
        self.reasoning_duration.observe(duration);
    }

    pub fn set_active_connections(&self, count: i64) {
        self.active_connections.set(count);
    }

    pub fn increment_errors(&self) {
        self.error_count.inc();
    }

    pub fn increment_cache_hits(&self) {
        self.cache_hits.inc();
    }

    pub fn increment_cache_misses(&self) {
        self.cache_misses.inc();
    }
}
```

## Troubleshooting

### Common Issues

#### 1. Memory Usage High

**Symptoms**: High memory consumption, slow performance

**Solutions**:
- Reduce batch sizes in pipeline configuration
- Enable memory pooling in caching
- Monitor memory usage and set limits
- Consider streaming processing for large datasets

```rust
// Optimize memory usage
let config = PipelineConfig {
    batch_size: 500, // Reduce from default 1000
    max_concurrent_tasks: 2, // Reduce concurrency
    enable_caching: true,
    output_format: OutputFormat::JSON,
    validation_level: ValidationLevel::Basic, // Reduce validation overhead
};
```

#### 2. Connection Timeouts

**Symptoms**: HTTP timeouts, connection errors

**Solutions**:
- Increase timeout values in client configuration
- Implement retry mechanisms
- Use connection pooling
- Monitor network latency

```rust
// Configure timeouts for web client
let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(60))
    .pool_idle_timeout(std::time::Duration::from_secs(30))
    .pool_max_idle_per_host(10)
    .build()?;
```

#### 3. Profile Validation Failures

**Symptoms**: OWL2 profile validation returns false for valid ontologies

**Solutions**:
- Check ontology for unsupported constructs
- Verify EPCIS data format compliance
- Enable debug logging for detailed validation errors
- Use profile-specific validation settings

```rust
// Debug profile validation
let result = reasoner.validate_profile(crate::profiles::Owl2Profile::EL)?;
println!("Validation result: {:?}", result);
for violation in &result.violations {
    println!("Violation: {:?}", violation.violation_type);
}
```

#### 4. Performance Bottlenecks

**Symptoms**: Slow processing, high latency

**Solutions**:
- Enable caching for repeated operations
- Use parallel processing for independent tasks
- Optimize database queries and indexing
- Monitor and optimize resource usage

```rust
// Enable advanced caching
let mut reasoner = SimpleReasoner::new(ontology);
reasoner.set_advanced_profile_caching(true);
```

### Debug Logging

```rust
use log::{info, warn, error, debug};

// Configure logging
env_logger::Builder::from_default_env()
    .filter_level(log::LevelFilter::Debug)
    .init();

// Add logging to key operations
debug!("Processing {} events", events.len());
info!("Reasoning completed in {:?}", duration);
warn!("High memory usage detected: {}MB", memory_mb);
error!("Failed to process batch: {}", error);
```

### Health Checks

```rust
pub async fn comprehensive_health_check() -> HealthStatus {
    let mut status = HealthStatus {
        overall: HealthStatusEnum::Healthy,
        components: HashMap::new(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Check memory usage
    let memory_usage = get_memory_usage();
    if memory_usage > 1024.0 { // 1GB
        status.components.insert("memory".to_string(), HealthStatusEnum::Degraded);
        status.overall = HealthStatusEnum::Degraded;
    } else {
        status.components.insert("memory".to_string(), HealthStatusEnum::Healthy);
    }

    // Check database connectivity
    if check_database_connection().await {
        status.components.insert("database".to_string(), HealthStatusEnum::Healthy);
    } else {
        status.components.insert("database".to_string(), HealthStatusEnum::Unhealthy);
        status.overall = HealthStatusEnum::Unhealthy;
    }

    // Check cache performance
    let cache_hit_rate = get_cache_hit_rate();
    if cache_hit_rate < 0.8 {
        status.components.insert("cache".to_string(), HealthStatusEnum::Degraded);
        if status.overall == HealthStatusEnum::Healthy {
            status.overall = HealthStatusEnum::Degraded;
        }
    } else {
        status.components.insert("cache".to_string(), HealthStatusEnum::Healthy);
    }

    status
}
```

## Conclusion

This comprehensive guide demonstrates the OWL2 reasoner's powerful ecosystem integration capabilities for EPCIS data processing. The system provides:

- **Multiple Integration Patterns**: Python, web services, data pipelines, and language bindings
- **Scalable Architecture**: Designed for production deployment with monitoring and optimization
- **Comprehensive API**: RESTful endpoints and native libraries for various use cases
- **Performance Optimized**: Caching, connection pooling, and parallel processing
- **Production Ready**: Docker, Kubernetes, and cloud deployment options

The OWL2 reasoner enables organizations to leverage semantic reasoning and knowledge graph capabilities for supply chain traceability, compliance checking, and business intelligence applications.

For additional support and contributions, please refer to the project documentation and community resources.