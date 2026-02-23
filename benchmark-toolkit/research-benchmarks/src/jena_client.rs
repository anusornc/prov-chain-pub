//! Jena Fuseki SPARQL Client for ProvChain Benchmark Suite
//!
//! This module provides a client for benchmarking Apache Jena Fuseki SPARQL endpoint
//! against ProvChain's native SPARQL implementation. It supports:
//! - Health checks
//! - Dataset loading via SPARQL Update
//! - Query execution with timing
//! - Standard SPARQL query patterns for supply chain traceability

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Jena Fuseki client configuration
#[derive(Debug, Clone)]
pub struct JenaClientConfig {
    /// Base URL for Fuseki server (e.g., "http://localhost:3030")
    pub base_url: String,
    /// Dataset name (e.g., "provchain")
    pub dataset: String,
    /// Request timeout
    pub timeout: Duration,
    /// Username for authentication (optional)
    pub username: Option<String>,
    /// Password for authentication (optional)
    pub password: Option<String>,
}

impl Default for JenaClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:3030".to_string(),
            dataset: "provchain".to_string(),
            timeout: Duration::from_secs(30),
            username: None,
            password: None,
        }
    }
}

/// SPARQL query result in standard JSON format
#[derive(Debug, Deserialize, Clone)]
pub struct SparqlJsonResult {
    pub head: Head,
    pub results: Results,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Head {
    pub vars: Vec<String>,
    pub link: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Results {
    pub bindings: Vec<HashMap<String, BindingValue>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BindingValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: String,
    #[serde(rename = "xml:lang")]
    pub lang: Option<String>,
    pub datatype: Option<String>,
}

/// Query execution result with timing
#[derive(Debug, Clone)]
pub struct QueryExecutionResult {
    pub success: bool,
    pub duration: Duration,
    pub result_count: usize,
    pub error_message: Option<String>,
    pub raw_response: Option<String>,
}

/// Jena Fuseki HTTP client
#[derive(Debug, Clone)]
pub struct JenaClient {
    config: JenaClientConfig,
    http_client: Client,
    query_endpoint: String,
    update_endpoint: String,
}

impl JenaClient {
    /// Create a new Jena client with the given configuration
    pub fn new(config: JenaClientConfig) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(config.timeout)
            .build()
            .context("Failed to build HTTP client")?;

        let query_endpoint = format!("{}/{}/sparql", config.base_url, config.dataset);
        let update_endpoint = format!("{}/{}/update", config.base_url, config.dataset);

        info!("Jena client configured:");
        info!("  Query endpoint: {}", query_endpoint);
        info!("  Update endpoint: {}", update_endpoint);

        Ok(Self {
            config,
            http_client,
            query_endpoint,
            update_endpoint,
        })
    }

    /// Create client from URL string
    pub fn from_url(url: &str) -> Result<Self> {
        let mut config = JenaClientConfig::default();
        config.base_url = url.to_string();
        Self::new(config)
    }

    /// Get the dataset URL for data loading
    fn data_endpoint(&self) -> String {
        format!("{}/{}/data", self.config.base_url, self.config.dataset)
    }

    /// Check if the Fuseki server is healthy
    pub async fn health_check(&self) -> Result<bool> {
        debug!("Checking Jena Fuseki health...");

        // Try the dataset endpoint first
        let response = self
            .http_client
            .get(format!("{}/{}", self.config.base_url, self.config.dataset))
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() || status == StatusCode::NOT_ACCEPTABLE {
                    // 406 is ok - means we need to specify content type
                    info!("✓ Jena Fuseki is healthy (dataset: {})", self.config.dataset);
                    Ok(true)
                } else {
                    warn!("Jena Fuseki returned status: {}", status);
                    Ok(false)
                }
            }
            Err(e) => {
                warn!("Jena Fuseki health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Wait for the server to become available
    pub async fn wait_for_ready(&self, max_attempts: u32, delay_secs: u64) -> Result<()> {
        info!("Waiting for Jena Fuseki to be ready...");

        for attempt in 1..=max_attempts {
            if self.health_check().await? {
                return Ok(());
            }
            warn!("Attempt {}/{} failed, retrying in {}s...", attempt, max_attempts, delay_secs);
            tokio::time::sleep(Duration::from_secs(delay_secs)).await;
        }

        anyhow::bail!("Jena Fuseki did not become ready after {} attempts", max_attempts)
    }

    /// Load RDF data into the dataset via SPARQL Update or direct upload
    /// 
    /// # Arguments
    /// * `rdf_data` - The RDF data as a string
    /// * `format` - The RDF format (e.g., "text/turtle", "application/rdf+xml")
    pub async fn load_dataset(&self, rdf_data: &str, format: &str) -> Result<Duration> {
        info!("Loading dataset into Jena Fuseki...");
        let start = Instant::now();

        // Use the data endpoint for direct upload (more efficient for bulk data)
        let response = self
            .http_client
            .put(&self.data_endpoint())
            .header("Content-Type", format)
            .body(rdf_data.to_string())
            .send()
            .await
            .context("Failed to send data to Jena Fuseki")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to load dataset: HTTP {} - {}", status, body);
        }

        let duration = start.elapsed();
        info!("✓ Dataset loaded in {:?}", duration);
        Ok(duration)
    }

    /// Load RDF data using SPARQL UPDATE (INSERT DATA)
    pub async fn load_dataset_via_update(&self, triples: &str) -> Result<Duration> {
        info!("Loading dataset via SPARQL UPDATE...");
        let start = Instant::now();

        // Wrap triples in INSERT DATA
        let update_query = format!("INSERT DATA {{ {} }}", triples);

        let response = self
            .http_client
            .post(&self.update_endpoint)
            .header("Content-Type", "application/sparql-update")
            .body(update_query)
            .send()
            .await
            .context("Failed to send SPARQL update")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("SPARQL update failed: HTTP {} - {}", status, body);
        }

        let duration = start.elapsed();
        info!("✓ Dataset loaded via UPDATE in {:?}", duration);
        Ok(duration)
    }

    /// Clear all data from the dataset
    pub async fn clear_dataset(&self) -> Result<()> {
        info!("Clearing Jena Fuseki dataset...");

        let update = "CLEAR ALL";
        
        let response = self
            .http_client
            .post(&self.update_endpoint)
            .header("Content-Type", "application/sparql-update")
            .body(update)
            .send()
            .await
            .context("Failed to clear dataset")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Clear failed: HTTP {} - {}", status, body);
        }

        info!("✓ Dataset cleared");
        Ok(())
    }

    /// Execute a SPARQL SELECT query and return results with timing
    pub async fn execute_query(&self, query: &str) -> Result<QueryExecutionResult> {
        debug!("Executing SPARQL query ({} chars)...", query.len());
        let start = Instant::now();

        let response = self
            .http_client
            .post(&self.query_endpoint)
            .header("Accept", "application/sparql-results+json")
            .form(&[("query", query)])
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                
                if !status.is_success() {
                    let body = resp.text().await.unwrap_or_default();
                    return Ok(QueryExecutionResult {
                        success: false,
                        duration: start.elapsed(),
                        result_count: 0,
                        error_message: Some(format!("HTTP {}: {}", status, body)),
                        raw_response: Some(body),
                    });
                }

                let body = resp.text().await.unwrap_or_default();
                let duration = start.elapsed();

                // Parse JSON results
                match serde_json::from_str::<SparqlJsonResult>(&body) {
                    Ok(result) => {
                        let count = result.results.bindings.len();
                        debug!("Query returned {} results in {:?}", count, duration);
                        Ok(QueryExecutionResult {
                            success: true,
                            duration,
                            result_count: count,
                            error_message: None,
                            raw_response: Some(body),
                        })
                    }
                    Err(e) => {
                        warn!("Failed to parse SPARQL results: {}", e);
                        Ok(QueryExecutionResult {
                            success: true, // Query succeeded, parsing failed
                            duration,
                            result_count: 0,
                            error_message: Some(format!("Parse error: {}", e)),
                            raw_response: Some(body),
                        })
                    }
                }
            }
            Err(e) => {
                error!("Query execution failed: {}", e);
                Ok(QueryExecutionResult {
                    success: false,
                    duration: start.elapsed(),
                    result_count: 0,
                    error_message: Some(e.to_string()),
                    raw_response: None,
                })
            }
        }
    }

    /// Execute a SPARQL SELECT query and return parsed results
    pub async fn query(&self, query: &str) -> Result<SparqlJsonResult> {
        debug!("Executing SPARQL query...");

        let response = self
            .http_client
            .post(&self.query_endpoint)
            .header("Accept", "application/sparql-results+json")
            .form(&[("query", query)])
            .send()
            .await
            .context("Failed to send SPARQL query")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("SPARQL query failed: HTTP {} - {}", status, body);
        }

        let result: SparqlJsonResult = response
            .json()
            .await
            .context("Failed to parse SPARQL results")?;

        Ok(result)
    }

    /// Execute a SPARQL ASK query
    pub async fn ask(&self, query: &str) -> Result<bool> {
        debug!("Executing SPARQL ASK query...");

        let response = self
            .http_client
            .post(&self.query_endpoint)
            .header("Accept", "application/sparql-results+json")
            .form(&[("query", query)])
            .send()
            .await
            .context("Failed to send SPARQL ASK query")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("SPARQL ASK query failed: HTTP {} - {}", status, body);
        }

        #[derive(Deserialize)]
        struct AskResult {
            boolean: bool,
        }

        let result: AskResult = response
            .json()
            .await
            .context("Failed to parse ASK result")?;

        Ok(result.boolean)
    }

    //========================================================================
    // Benchmark-specific query methods
    //========================================================================

    /// Query 1: Simple product lookup by batch ID
    /// 
    /// Looks up a product by its batch identifier - basic single-hop query
    pub async fn query_product_by_batch(&self, batch_id: &str) -> Result<QueryExecutionResult> {
        let query = format!(
            r#"PREFIX ex: <http://example.org/>
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX pc: <http://provchain.org/ontology#>

SELECT ?product ?name ?type ?status
WHERE {{
    ?product a ?type ;
             pc:batchId "{batch_id}" ;
             pc:name ?name .
    OPTIONAL {{ ?product pc:status ?status }}
}}"#,
            batch_id = batch_id
        );

        self.execute_query(&query).await
    }

    /// Query 2: Multi-hop traceability query (10 hops)
    /// 
    /// Traces a product through the supply chain following wasDerivedFrom links
    /// This is the most complex query for benchmarking graph traversal performance
    pub async fn query_traceability_10_hops(&self, batch_id: &str) -> Result<QueryExecutionResult> {
        let query = format!(
            r#"PREFIX ex: <http://example.org/>
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX pc: <http://provchain.org/ontology#>

SELECT DISTINCT ?origin ?hop1 ?hop2 ?hop3 ?hop4 ?hop5 ?hop6 ?hop7 ?hop8 ?hop9 ?final_product
WHERE {{
    # Start with the batch
    ?batch pc:batchId "{batch_id}" .
    
    # Follow derivation chain up to 10 hops
    ?final_product prov:wasDerivedFrom* ?batch .
    
    # Get intermediate hops
    OPTIONAL {{ ?hop1 prov:wasDerivedFrom ?batch }}
    OPTIONAL {{ ?hop2 prov:wasDerivedFrom ?hop1 }}
    OPTIONAL {{ ?hop3 prov:wasDerivedFrom ?hop2 }}
    OPTIONAL {{ ?hop4 prov:wasDerivedFrom ?hop3 }}
    OPTIONAL {{ ?hop5 prov:wasDerivedFrom ?hop4 }}
    OPTIONAL {{ ?hop6 prov:wasDerivedFrom ?hop5 }}
    OPTIONAL {{ ?hop7 prov:wasDerivedFrom ?hop6 }}
    OPTIONAL {{ ?hop8 prov:wasDerivedFrom ?hop7 }}
    OPTIONAL {{ ?hop9 prov:wasDerivedFrom ?hop8 }}
    
    # Get origin
    OPTIONAL {{ ?origin prov:wasDerivedFrom ?hop9 }}
}}
LIMIT 100"#,
            batch_id = batch_id
        );

        self.execute_query(&query).await
    }

    /// Alternative: Property path based multi-hop query (more efficient)
    pub async fn query_traceability_property_path(&self, batch_id: &str) -> Result<QueryExecutionResult> {
        let query = format!(
            r#"PREFIX ex: <http://example.org/>
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX pc: <http://provchain.org/ontology#>

SELECT DISTINCT ?product ?step ?derived_from ?agent ?timestamp ?location
WHERE {{
    # Find all products derived from this batch (transitive)
    VALUES ?start_batch {{ _:b }}
    ?start_batch pc:batchId "{batch_id}" .
    
    ?product prov:wasDerivedFrom+ ?start_batch .
    
    # Get derivation details
    ?product prov:wasDerivedFrom ?derived_from ;
             a ?type .
    
    OPTIONAL {{ ?product pc:processingStep ?step }}
    OPTIONAL {{ ?product prov:wasAttributedTo ?agent }}
    OPTIONAL {{ ?product prov:generatedAtTime ?timestamp }}
    OPTIONAL {{ ?pc:atLocation ?location }}
}}
ORDER BY ?timestamp
LIMIT 100"#,
            batch_id = batch_id
        );

        self.execute_query(&query).await
    }

    /// Query 3: Aggregation by producer
    /// 
    /// Calculates total quantity, count of products, etc. grouped by producer
    pub async fn query_aggregation_by_producer(&self) -> Result<QueryExecutionResult> {
        let query = r#"PREFIX ex: <http://example.org/>
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX pc: <http://provchain.org/ontology#>

SELECT ?producer 
       (COUNT(?product) AS ?product_count)
       (SUM(?quantity) AS ?total_quantity)
       (AVG(?quality_score) AS ?avg_quality)
       (MIN(?timestamp) AS ?first_production)
       (MAX(?timestamp) AS ?last_production)
WHERE {
    ?product a pc:Product ;
             prov:wasAttributedTo ?producer ;
             pc:quantity ?quantity .
    
    OPTIONAL { ?product pc:qualityScore ?quality_score }
    OPTIONAL { ?product prov:generatedAtTime ?timestamp }
}
GROUP BY ?producer
ORDER BY DESC(?total_quantity)"#;

        self.execute_query(query).await
    }

    /// Query 4: Complex supply chain trace with all attributes
    /// 
    /// Full traceability query used for detailed analysis
    pub async fn query_full_trace(&self, product_id: &str) -> Result<QueryExecutionResult> {
        let query = format!(
            r#"PREFIX ex: <http://example.org/>
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX pc: <http://provchain.org/ontology#>

SELECT DISTINCT ?entity ?activity ?agent ?timestamp ?location ?action ?status
WHERE {{
    # The product itself
    BIND(<{product_id}> AS ?product)
    
    # Get all derived entities
    ?entity prov:wasDerivedFrom* ?product .
    
    # Get provenance information
    OPTIONAL {{ ?entity prov:wasGeneratedBy ?activity }}
    OPTIONAL {{ ?activity prov:wasAssociatedWith ?agent }}
    OPTIONAL {{ ?entity prov:generatedAtTime ?timestamp }}
    OPTIONAL {{ ?entity pc:atLocation ?location }}
    OPTIONAL {{ ?entity pc:action ?action }}
    OPTIONAL {{ ?entity pc:status ?status }}
}}
ORDER BY ?timestamp"#,
            product_id = product_id
        );

        self.execute_query(&query).await
    }

    /// Query 5: Environmental conditions query
    /// 
    /// Retrieves IoT sensor data associated with a batch
    pub async fn query_environmental_data(&self, batch_id: &str) -> Result<QueryExecutionResult> {
        let query = format!(
            r#"PREFIX ex: <http://example.org/>
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX pc: <http://provchain.org/ontology#>
PREFIX sosa: <http://www.w3.org/ns/sosa/>

SELECT ?observation ?sensor ?observed_property ?result_time ?simple_result
WHERE {{
    ?batch pc:batchId "{batch_id}" .
    
    # Find observations related to this batch
    ?observation sosa:hasFeatureOfInterest ?batch ;
                 sosa:madeBySensor ?sensor ;
                 sosa:observedProperty ?observed_property ;
                 sosa:resultTime ?result_time ;
                 sosa:hasSimpleResult ?simple_result .
}}
ORDER BY ?result_time"#,
            batch_id = batch_id
        );

        self.execute_query(&query).await
    }

    /// Get dataset statistics
    pub async fn get_dataset_stats(&self) -> Result<HashMap<String, Value>> {
        let query = r#"PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX pc: <http://provchain.org/ontology#>

SELECT 
    (COUNT(*) AS ?total_triples)
    (COUNT(DISTINCT ?s) AS ?unique_subjects)
    (COUNT(DISTINCT ?p) AS ?unique_predicates)
    (COUNT(DISTINCT ?o) AS ?unique_objects)
WHERE {
    ?s ?p ?o .
}"#;

        let result = self.query(query).await?;
        let mut stats = HashMap::new();

        if let Some(binding) = result.results.bindings.first() {
            for (key, value) in binding {
                let val = match value.value_type.as_str() {
                    "literal" => value.value.parse::<i64>().unwrap_or(0).into(),
                    _ => Value::String(value.value.clone()),
                };
                stats.insert(key.clone(), val);
            }
        }

        Ok(stats)
    }
}

/// Benchmark result compatible with the benchmark suite
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JenaBenchmarkResult {
    pub system: String,
    pub scenario: String,
    pub test_name: String,
    pub iteration: usize,
    pub duration_ms: f64,
    pub operations_per_second: f64,
    pub success: bool,
    pub error_message: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub result_count: usize,
    pub metadata: HashMap<String, Value>,
}

impl JenaBenchmarkResult {
    /// Create a benchmark result from a query execution
    pub fn from_execution(
        scenario: &str,
        test_name: &str,
        iteration: usize,
        execution: QueryExecutionResult,
    ) -> Self {
        let duration_ms = execution.duration.as_millis() as f64;
        let operations_per_second = if duration_ms > 0.0 {
            1000.0 / duration_ms
        } else {
            0.0
        };

        let mut metadata = HashMap::new();
        metadata.insert("result_count".to_string(), execution.result_count.into());

        Self {
            system: "Apache-Jena-Fuseki".to_string(),
            scenario: scenario.to_string(),
            test_name: test_name.to_string(),
            iteration,
            duration_ms,
            operations_per_second,
            success: execution.success,
            error_message: execution.error_message,
            timestamp: Utc::now(),
            result_count: execution.result_count,
            metadata,
        }
    }
}

/// Helper functions for benchmark integration
pub mod benchmark_helpers {
    use super::*;

    /// Run a benchmark query multiple times and collect results
    pub async fn run_benchmark_iterations<F, Fut>(
        scenario: &str,
        test_name: &str,
        iterations: usize,
        warmup_iterations: usize,
        query_fn: F,
    ) -> Vec<JenaBenchmarkResult>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<QueryExecutionResult>>,
    {
        let mut results = Vec::new();

        // Warmup
        for i in 0..warmup_iterations {
            debug!("Warmup iteration {}/{}", i + 1, warmup_iterations);
            let _ = query_fn().await;
        }

        // Benchmark iterations
        for i in 0..iterations {
            let execution = match query_fn().await {
                Ok(exec) => exec,
                Err(e) => QueryExecutionResult {
                    success: false,
                    duration: Duration::from_millis(0),
                    result_count: 0,
                    error_message: Some(e.to_string()),
                    raw_response: None,
                },
            };

            results.push(JenaBenchmarkResult::from_execution(
                scenario,
                test_name,
                i,
                execution,
            ));
        }

        results
    }

    /// Convert Jena results to the standard BenchmarkResult format
    pub fn to_standard_results(
        jena_results: &[JenaBenchmarkResult],
    ) -> Vec<super::super::BenchmarkResult> {
        jena_results
            .iter()
            .map(|r| super::super::BenchmarkResult {
                system: r.system.clone(),
                scenario: r.scenario.clone(),
                test_name: r.test_name.clone(),
                iteration: r.iteration,
                duration_ms: r.duration_ms,
                operations_per_second: r.operations_per_second,
                success: r.success,
                error_message: r.error_message.clone(),
                timestamp: r.timestamp,
                metadata: r.metadata.clone(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = JenaClientConfig::default();
        assert_eq!(config.base_url, "http://localhost:3030");
        assert_eq!(config.dataset, "provchain");
    }

    #[test]
    fn test_client_creation() {
        let config = JenaClientConfig::default();
        let client = JenaClient::new(config).unwrap();
        assert!(client.query_endpoint.contains("sparql"));
        assert!(client.update_endpoint.contains("update"));
    }

    #[tokio::test]
    async fn test_parse_sparql_result() {
        let json = r#"{
            "head": {
                "vars": ["product", "name"]
            },
            "results": {
                "bindings": [
                    {
                        "product": {
                            "type": "uri",
                            "value": "http://example.org/product/1"
                        },
                        "name": {
                            "type": "literal",
                            "value": "Test Product"
                        }
                    }
                ]
            }
        }"#;

        let result: SparqlJsonResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.head.vars.len(), 2);
        assert_eq!(result.results.bindings.len(), 1);
        assert_eq!(
            result.results.bindings[0]["product"].value,
            "http://example.org/product/1"
        );
    }
}
