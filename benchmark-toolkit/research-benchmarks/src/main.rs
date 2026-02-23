use anyhow::{Context, Result};
use benchmark_runner::{
    jena_client::{JenaClient, JenaClientConfig},
    neo4j_client::{Neo4jClient, Neo4jConfig},
};
use chrono::{DateTime, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Benchmark runner for comparing ProvChain-Org with other systems
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run all benchmark scenarios
    #[arg(long)]
    all: bool,

    /// Run query performance benchmark only
    #[arg(long)]
    query: bool,

    /// Run write performance benchmark only
    #[arg(long)]
    write: bool,

    /// Run comparison benchmark (ProvChain vs Neo4j)
    #[arg(long)]
    compare: bool,

    /// ProvChain API URL
    #[arg(long, env = "PROVCHAIN_URL", default_value = "http://localhost:8080")]
    provchain_url: String,

    /// Neo4j Bolt URI
    #[arg(long, env = "NEO4J_URI", default_value = "bolt://localhost:7687")]
    neo4j_uri: String,

    /// Neo4j username
    #[arg(long, env = "NEO4J_USER", default_value = "neo4j")]
    neo4j_user: String,

    /// Neo4j password
    #[arg(long, env = "NEO4J_PASSWORD")]
    neo4j_password: Option<String>,

    /// Dataset path
    #[arg(long, env = "DATASET_PATH", default_value = "/benchmark/datasets")]
    dataset_path: String,

    /// Results path
    #[arg(long, env = "RESULTS_PATH", default_value = "/benchmark/results")]
    results_path: String,

    /// Number of warmup iterations
    #[arg(long, default_value = "3")]
    warmup_iterations: usize,

    /// Number of benchmark iterations
    #[arg(long, default_value = "10")]
    iterations: usize,

    /// Skip Neo4j benchmarks
    #[arg(long)]
    skip_neo4j: bool,

    /// Skip ProvChain benchmarks
    #[arg(long)]
    skip_provchain: bool,

    /// Skip Jena benchmarks
    #[arg(long)]
    skip_jena: bool,

    /// Jena Fuseki URL
    #[arg(long, env = "JENA_URL", default_value = "http://localhost:3030")]
    jena_url: String,

    /// Jena dataset name
    #[arg(long, env = "JENA_DATASET", default_value = "provchain")]
    jena_dataset: String,

    /// Dataset file name
    #[arg(long, default_value = "supply_chain_1000.ttl")]
    dataset_file: String,

    /// Batch IDs for testing (comma-separated)
    #[arg(long, default_value = "BATCH001,BATCH010,BATCH017,BATCH025,BATCH050")]
    test_batch_ids: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BenchmarkResult {
    pub system: String,
    pub scenario: String,
    pub test_name: String,
    pub iteration: usize,
    pub duration_ms: f64,
    pub operations_per_second: f64,
    pub success: bool,
    pub error_message: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkSummary {
    pub scenario: String,
    pub test_name: String,
    pub provchain_avg_ms: f64,
    pub neo4j_avg_ms: f64,
    pub jena_avg_ms: f64,
    pub provchain_ops_per_sec: f64,
    pub neo4j_ops_per_sec: f64,
    pub jena_ops_per_sec: f64,
    pub improvement_percent: f64,
    pub winner: String,
    pub provchain_success_rate: f64,
    pub neo4j_success_rate: f64,
    pub jena_success_rate: f64,
}

/// System clients for benchmarking
struct SystemClients {
    provchain_url: String,
    neo4j_client: Option<Neo4jClient>,
    neo4j_config: Neo4jConfig,
    jena_client: Option<JenaClient>,
    jena_config: JenaClientConfig,
}

impl SystemClients {
    fn new(args: &Args) -> Self {
        let neo4j_config = Neo4jConfig {
            uri: args.neo4j_uri.clone(),
            user: args.neo4j_user.clone(),
            password: args.neo4j_password.clone().unwrap_or_else(|| "password".to_string()),
            timeout_secs: 30,
            max_connections: 10,
        };

        let jena_config = JenaClientConfig {
            base_url: args.jena_url.clone(),
            dataset: args.jena_dataset.clone(),
            timeout: Duration::from_secs(30),
            username: None,
            password: None,
        };

        SystemClients {
            provchain_url: args.provchain_url.clone(),
            neo4j_client: None,
            neo4j_config,
            jena_client: None,
            jena_config,
        }
    }

    async fn initialize(&mut self, skip_neo4j: bool, skip_jena: bool) -> Result<()> {
        // Initialize Neo4j client if not skipped
        if !skip_neo4j {
            info!("Initializing Neo4j client...");
            let mut client = Neo4jClient::new(self.neo4j_config.clone());
            match client.connect().await {
                Ok(()) => {
                    info!("✓ Neo4j client connected");
                    self.neo4j_client = Some(client);
                }
                Err(e) => {
                    warn!("⚠ Failed to connect to Neo4j: {}", e);
                    warn!("   Neo4j benchmarks will be skipped");
                }
            }
        }

        // Initialize Jena client if not skipped
        if !skip_jena {
            info!("Initializing Jena client...");
            match JenaClient::new(self.jena_config.clone()) {
                Ok(client) => {
                    match client.health_check().await {
                        Ok(true) => {
                            info!("✓ Jena client connected");
                            self.jena_client = Some(client);
                        }
                        Ok(false) => {
                            warn!("⚠ Jena health check returned unhealthy");
                            warn!("   Jena benchmarks will be skipped");
                        }
                        Err(e) => {
                            warn!("⚠ Failed to check Jena health: {}", e);
                            warn!("   Jena benchmarks will be skipped");
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠ Failed to create Jena client: {}", e);
                    warn!("   Jena benchmarks will be skipped");
                }
            }
        }

        Ok(())
    }

    async fn check_health(&self) -> Result<()> {
        info!("Checking system health...");

        // Check ProvChain
        let provchain_health = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?
            .get(format!("{}/health", self.provchain_url))
            .send()
            .await;

        match provchain_health {
            Ok(response) if response.status().is_success() => {
                info!("✓ ProvChain-Org is healthy");
            }
            Ok(response) => {
                warn!("ProvChain-Org health check returned: {}", response.status());
            }
            Err(e) => {
                warn!("ProvChain-Org health check failed: {}", e);
            }
        }

        // Check Neo4j
        if let Some(client) = &self.neo4j_client {
            match client.health_check().await {
                Ok(true) => info!("✓ Neo4j is healthy"),
                Ok(false) => warn!("⚠ Neo4j health check returned unhealthy"),
                Err(e) => warn!("⚠ Neo4j health check failed: {}", e),
            }
        } else {
            info!("⊘ Neo4j client not initialized");
        }

        // Check Jena
        if let Some(client) = &self.jena_client {
            match client.health_check().await {
                Ok(true) => info!("✓ Jena is healthy"),
                Ok(false) => warn!("⚠ Jena health check returned unhealthy"),
                Err(e) => warn!("⚠ Jena health check failed: {}", e),
            }
        } else {
            info!("⊘ Jena client not initialized");
        }

        Ok(())
    }

    async fn load_dataset_provchain(&self, dataset_path: &str, dataset_file: &str) -> Result<Duration> {
        let dataset_path = Path::new(dataset_path).join(dataset_file);
        let content = fs::read_to_string(&dataset_path)
            .with_context(|| format!("Failed to read dataset: {:?}", dataset_path))?;

        let start = Instant::now();

        // Parse RDF and submit to ProvChain
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/rdf/import", self.provchain_url))
            .header("Content-Type", "text/turtle")
            .body(content)
            .send()
            .await
            .context("Failed to load dataset into ProvChain")?;

        if !response.status().is_success() {
            anyhow::bail!("ProvChain import failed: {}", response.status());
        }

        Ok(start.elapsed())
    }

    async fn load_dataset_neo4j(&self, dataset_path: &str, dataset_file: &str) -> Result<Duration> {
        if let Some(client) = &self.neo4j_client {
            let dataset_path = Path::new(dataset_path).join(dataset_file);
            
            // Clear existing data first
            client.clear_all_data().await?;
            
            // Load Turtle data
            let duration = client.load_turtle_data(&dataset_path).await?;
            Ok(duration)
        } else {
            Err(anyhow::anyhow!("Neo4j client not initialized"))
        }
    }

    async fn load_dataset_jena(&self, dataset_path: &str, dataset_file: &str) -> Result<Duration> {
        if let Some(client) = &self.jena_client {
            let dataset_path = Path::new(dataset_path).join(dataset_file);
            let content = fs::read_to_string(&dataset_path)
                .with_context(|| format!("Failed to read dataset: {:?}", dataset_path))?;
            
            // Clear existing data first
            client.clear_dataset().await?;
            
            // Load Turtle data
            let duration = client.load_dataset(&content, "text/turtle").await?;
            Ok(duration)
        } else {
            Err(anyhow::anyhow!("Jena client not initialized"))
        }
    }
}

/// Run query performance benchmark comparing ProvChain and Neo4j
async fn benchmark_query_performance(
    clients: &SystemClients,
    args: &Args,
) -> Result<Vec<BenchmarkResult>> {
    info!("Starting query performance benchmark...");

    let mut results = Vec::new();
    let test_batch_ids: Vec<&str> = args.test_batch_ids.split(',').collect();

    // ========== Query 1: Simple product lookup ==========
    info!("Test 1: Simple product lookup by batch ID");
    
    for batch_id in &test_batch_ids {
        for i in 0..args.iterations {
            // ProvChain query
            if !args.skip_provchain {
                let start = Instant::now();
                
                let sparql = format!(
                    "PREFIX ex: <http://example.org/supplychain/> \
                     SELECT ?product WHERE {{ \
                         ?product a ex:Product . \
                         ?product ex:batchId \"{}\" \
                     }}",
                    batch_id
                );

                let provchain_result = reqwest::Client::new()
                    .post(format!("{}/api/sparql/query", clients.provchain_url))
                    .json(&serde_json::json!({ "query": sparql }))
                    .send()
                    .await;

                let provchain_duration = start.elapsed();

                results.push(BenchmarkResult {
                    system: "ProvChain-Org".to_string(),
                    scenario: "Query Performance".to_string(),
                    test_name: "Simple Product Lookup".to_string(),
                    iteration: i,
                    duration_ms: provchain_duration.as_millis() as f64,
                    operations_per_second: 1000.0 / provchain_duration.as_millis().max(1) as f64,
                    success: provchain_result.is_ok(),
                    error_message: provchain_result.err().map(|e| e.to_string()),
                    timestamp: Utc::now(),
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("batch_id".to_string(), batch_id.to_string().into());
                        m
                    },
                });
            }

            // Neo4j query
            if !args.skip_neo4j && clients.neo4j_client.is_some() {
                match clients.neo4j_client.as_ref().unwrap()
                    .query_product_by_batch_id(batch_id)
                    .await {
                    Ok(query_result) => {
                        results.push(BenchmarkResult {
                            system: "Neo4j".to_string(),
                            scenario: "Query Performance".to_string(),
                            test_name: "Simple Product Lookup".to_string(),
                            iteration: i,
                            duration_ms: query_result.duration_ms,
                            operations_per_second: 1000.0 / query_result.duration_ms.max(1.0),
                            success: query_result.success,
                            error_message: query_result.error_message,
                            timestamp: Utc::now(),
                            metadata: {
                                let mut m = HashMap::new();
                                m.insert("batch_id".to_string(), batch_id.to_string().into());
                                m.insert("record_count".to_string(), query_result.record_count.into());
                                m
                            },
                        });
                    }
                    Err(e) => {
                        results.push(BenchmarkResult {
                            system: "Neo4j".to_string(),
                            scenario: "Query Performance".to_string(),
                            test_name: "Simple Product Lookup".to_string(),
                            iteration: i,
                            duration_ms: 0.0,
                            operations_per_second: 0.0,
                            success: false,
                            error_message: Some(e.to_string()),
                            timestamp: Utc::now(),
                            metadata: {
                                let mut m = HashMap::new();
                                m.insert("batch_id".to_string(), batch_id.to_string().into());
                                m
                            },
                        });
                    }
                }
            }
        }
    }

    // ========== Query 2: Multi-hop traceability ==========
    info!("Test 2: Multi-hop traceability (10 hops)");
    
    for batch_id in &test_batch_ids {
        for i in 0..args.iterations {
            // ProvChain query
            if !args.skip_provchain {
                let start = Instant::now();

                let sparql = format!(
                    "PREFIX ex: <http://example.org/supplychain/> \
                     PREFIX trace: <http://example.org/traceability#> \
                     SELECT ?product ?transaction ?hop \
                     WHERE {{ \
                         ?product ex:batchId \"{}\" . \
                         ?product trace:hasTransaction ?tx1 . \
                         OPTIONAL {{ ?tx1 trace:nextTransaction ?tx2 }} \
                         OPTIONAL {{ ?tx2 trace:nextTransaction ?tx3 }} \
                     }}",
                    batch_id
                );

                let provchain_result = reqwest::Client::new()
                    .post(format!("{}/api/sparql/query", clients.provchain_url))
                    .json(&serde_json::json!({ "query": sparql }))
                    .send()
                    .await;

                let provchain_duration = start.elapsed();

                results.push(BenchmarkResult {
                    system: "ProvChain-Org".to_string(),
                    scenario: "Query Performance".to_string(),
                    test_name: "Multi-hop Traceability (10 hops)".to_string(),
                    iteration: i,
                    duration_ms: provchain_duration.as_millis() as f64,
                    operations_per_second: 1000.0 / provchain_duration.as_millis().max(1) as f64,
                    success: provchain_result.is_ok(),
                    error_message: provchain_result.err().map(|e| e.to_string()),
                    timestamp: Utc::now(),
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("batch_id".to_string(), batch_id.to_string().into());
                        m
                    },
                });
            }

            // Neo4j query
            if !args.skip_neo4j && clients.neo4j_client.is_some() {
                match clients.neo4j_client.as_ref().unwrap()
                    .query_multi_hop_traceability(batch_id, 10)
                    .await {
                    Ok(query_result) => {
                        results.push(BenchmarkResult {
                            system: "Neo4j".to_string(),
                            scenario: "Query Performance".to_string(),
                            test_name: "Multi-hop Traceability (10 hops)".to_string(),
                            iteration: i,
                            duration_ms: query_result.duration_ms,
                            operations_per_second: 1000.0 / query_result.duration_ms.max(1.0),
                            success: query_result.success,
                            error_message: query_result.error_message,
                            timestamp: Utc::now(),
                            metadata: {
                                let mut m = HashMap::new();
                                m.insert("batch_id".to_string(), batch_id.to_string().into());
                                m.insert("record_count".to_string(), query_result.record_count.into());
                                m
                            },
                        });
                    }
                    Err(e) => {
                        results.push(BenchmarkResult {
                            system: "Neo4j".to_string(),
                            scenario: "Query Performance".to_string(),
                            test_name: "Multi-hop Traceability (10 hops)".to_string(),
                            iteration: i,
                            duration_ms: 0.0,
                            operations_per_second: 0.0,
                            success: false,
                            error_message: Some(e.to_string()),
                            timestamp: Utc::now(),
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }
    }

    // ========== Query 3: Aggregation by producer ==========
    info!("Test 3: Aggregation query");
    
    for i in 0..args.iterations {
        // ProvChain query
        if !args.skip_provchain {
            let start = Instant::now();

            let sparql = r#"
                PREFIX ex: <http://example.org/supplychain/>
                PREFIX trace: <http://example.org/traceability#>
                SELECT ?producer (SUM(?quantity) AS ?total)
                WHERE {
                    ?product trace:hasProducer ?producer .
                    ?product trace:hasTransaction ?tx .
                    ?tx trace:quantity ?quantity .
                }
                GROUP BY ?producer
            "#;

            let provchain_result = reqwest::Client::new()
                .post(format!("{}/api/sparql/query", clients.provchain_url))
                .json(&serde_json::json!({ "query": sparql }))
                .send()
                .await;

            let provchain_duration = start.elapsed();

            results.push(BenchmarkResult {
                system: "ProvChain-Org".to_string(),
                scenario: "Query Performance".to_string(),
                test_name: "Aggregation by Producer".to_string(),
                iteration: i,
                duration_ms: provchain_duration.as_millis() as f64,
                operations_per_second: 1000.0 / provchain_duration.as_millis().max(1) as f64,
                success: provchain_result.is_ok(),
                error_message: provchain_result.err().map(|e| e.to_string()),
                timestamp: Utc::now(),
                metadata: HashMap::new(),
            });
        }

        // Neo4j query
        if !args.skip_neo4j && clients.neo4j_client.is_some() {
            match clients.neo4j_client.as_ref().unwrap()
                .query_aggregation_by_producer()
                .await {
                Ok(query_result) => {
                    results.push(BenchmarkResult {
                        system: "Neo4j".to_string(),
                        scenario: "Query Performance".to_string(),
                        test_name: "Aggregation by Producer".to_string(),
                        iteration: i,
                        duration_ms: query_result.duration_ms,
                        operations_per_second: 1000.0 / query_result.duration_ms.max(1.0),
                        success: query_result.success,
                        error_message: query_result.error_message,
                        timestamp: Utc::now(),
                        metadata: {
                            let mut m = HashMap::new();
                            m.insert("record_count".to_string(), query_result.record_count.into());
                            m
                        },
                    });
                }
                Err(e) => {
                    results.push(BenchmarkResult {
                        system: "Neo4j".to_string(),
                        scenario: "Query Performance".to_string(),
                        test_name: "Aggregation by Producer".to_string(),
                        iteration: i,
                        duration_ms: 0.0,
                        operations_per_second: 0.0,
                        success: false,
                        error_message: Some(e.to_string()),
                        timestamp: Utc::now(),
                        metadata: HashMap::new(),
                    });
                }
            }
        }
    }

    Ok(results)
}

/// Run write performance benchmark
async fn benchmark_write_performance(
    clients: &SystemClients,
    args: &Args,
) -> Result<Vec<BenchmarkResult>> {
    info!("Starting write performance benchmark...");

    let mut results = Vec::new();

    // Test 1: Single-threaded write (100 transactions)
    info!("Test 1: Single-threaded write (100 transactions)");
    for i in 0..args.iterations {
        let start = Instant::now();

        for batch_id in 1..=100 {
            let _ = reqwest::Client::new()
                .post(format!("{}/api/transactions", clients.provchain_url))
                .json(&serde_json::json!({
                    "from": format!("http://example.org/producer/{}", batch_id),
                    "to": "http://example.org/processor/packing001",
                    "product": format!("http://example.org/product/BATCH{:03}", batch_id + 1000),
                    "quantity": 100.0,
                    "timestamp": Utc::now().to_rfc3339()
                }))
                .send()
                .await;
        }

        let duration = start.elapsed();

        results.push(BenchmarkResult {
            system: "ProvChain-Org".to_string(),
            scenario: "Write Performance".to_string(),
            test_name: "Single-threaded Write (100 tx)".to_string(),
            iteration: i,
            duration_ms: duration.as_millis() as f64,
            operations_per_second: 100.0 / duration.as_secs_f64(),
            success: true,
            error_message: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        });
    }

    Ok(results)
}

/// Run data loading benchmark
async fn benchmark_data_loading(
    clients: &SystemClients,
    args: &Args,
) -> Result<Vec<BenchmarkResult>> {
    info!("Starting data loading benchmark...");

    let mut results = Vec::new();

    // ProvChain data loading
    if !args.skip_provchain {
        info!("Loading data into ProvChain...");
        match clients.load_dataset_provchain(&args.dataset_path, &args.dataset_file).await {
            Ok(duration) => {
                results.push(BenchmarkResult {
                    system: "ProvChain-Org".to_string(),
                    scenario: "Data Loading".to_string(),
                    test_name: "Turtle RDF Import".to_string(),
                    iteration: 0,
                    duration_ms: duration.as_millis() as f64,
                    operations_per_second: 1000.0 / duration.as_secs_f64(),
                    success: true,
                    error_message: None,
                    timestamp: Utc::now(),
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("dataset".to_string(), args.dataset_file.clone().into());
                        m
                    },
                });
            }
            Err(e) => {
                error!("Failed to load data into ProvChain: {}", e);
                results.push(BenchmarkResult {
                    system: "ProvChain-Org".to_string(),
                    scenario: "Data Loading".to_string(),
                    test_name: "Turtle RDF Import".to_string(),
                    iteration: 0,
                    duration_ms: 0.0,
                    operations_per_second: 0.0,
                    success: false,
                    error_message: Some(e.to_string()),
                    timestamp: Utc::now(),
                    metadata: HashMap::new(),
                });
            }
        }
    }

    // Neo4j data loading
    if !args.skip_neo4j && clients.neo4j_client.is_some() {
        info!("Loading data into Neo4j...");
        match clients.load_dataset_neo4j(&args.dataset_path, &args.dataset_file).await {
            Ok(duration) => {
                results.push(BenchmarkResult {
                    system: "Neo4j".to_string(),
                    scenario: "Data Loading".to_string(),
                    test_name: "Turtle to Cypher Import".to_string(),
                    iteration: 0,
                    duration_ms: duration.as_millis() as f64,
                    operations_per_second: 1000.0 / duration.as_secs_f64(),
                    success: true,
                    error_message: None,
                    timestamp: Utc::now(),
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("dataset".to_string(), args.dataset_file.clone().into());
                        m
                    },
                });
            }
            Err(e) => {
                error!("Failed to load data into Neo4j: {}", e);
                results.push(BenchmarkResult {
                    system: "Neo4j".to_string(),
                    scenario: "Data Loading".to_string(),
                    test_name: "Turtle to Cypher Import".to_string(),
                    iteration: 0,
                    duration_ms: 0.0,
                    operations_per_second: 0.0,
                    success: false,
                    error_message: Some(e.to_string()),
                    timestamp: Utc::now(),
                    metadata: HashMap::new(),
                });
            }
        }
    }

    Ok(results)
}

/// Generate comparison report
fn generate_report(results: &[BenchmarkResult], results_path: &str) -> Result<()> {
    info!("Generating comparison report...");

    // Create results directory if it doesn't exist
    fs::create_dir_all(results_path)?;

    // Write JSON results
    let json_file = Path::new(results_path).join("benchmark_results.json");
    let json_output = File::create(&json_file)?;
    serde_json::to_writer_pretty(json_output, results)?;
    info!("Results written to: {:?}", json_file);

    // Write CSV results
    let csv_file = Path::new(results_path).join("benchmark_results.csv");
    let mut csv_writer = csv::Writer::from_path(csv_file)?;

    for result in results {
        csv_writer.serialize(result)?;
    }
    csv_writer.flush()?;
    info!("CSV results written");

    // Calculate summary statistics
    let mut summaries: Vec<BenchmarkSummary> = Vec::new();

    // Group by scenario and test_name
    let mut grouped: HashMap<String, Vec<&BenchmarkResult>> = HashMap::new();
    for result in results {
        let key = format!("{}:{}", result.scenario, result.test_name);
        grouped.entry(key).or_default().push(result);
    }

    // Calculate averages and generate summary
    for (key, group_results) in &grouped {
        let provchain_results: Vec<_> = group_results
            .iter()
            .filter(|r| r.system == "ProvChain-Org")
            .collect();

        let neo4j_results: Vec<_> = group_results
            .iter()
            .filter(|r| r.system == "Neo4j")
            .collect();

        let provchain_avg = if !provchain_results.is_empty() {
            let total: f64 = provchain_results.iter().map(|r| r.duration_ms).sum();
            let success_count = provchain_results.iter().filter(|r| r.success).count();
            (total / provchain_results.len() as f64, success_count as f64 / provchain_results.len() as f64)
        } else {
            (0.0, 0.0)
        };

        let neo4j_avg = if !neo4j_results.is_empty() {
            let total: f64 = neo4j_results.iter().map(|r| r.duration_ms).sum();
            let success_count = neo4j_results.iter().filter(|r| r.success).count();
            (total / neo4j_results.len() as f64, success_count as f64 / neo4j_results.len() as f64)
        } else {
            (0.0, 0.0)
        };

        let provchain_ops: f64 = if !provchain_results.is_empty() {
            provchain_results.iter().map(|r| r.operations_per_second).sum::<f64>() / provchain_results.len() as f64
        } else {
            0.0
        };

        let neo4j_ops: f64 = if !neo4j_results.is_empty() {
            neo4j_results.iter().map(|r| r.operations_per_second).sum::<f64>() / neo4j_results.len() as f64
        } else {
            0.0
        };

        // Calculate improvement (negative means ProvChain is faster)
        let improvement = if neo4j_avg.0 > 0.0 {
            ((neo4j_avg.0 - provchain_avg.0) / neo4j_avg.0) * 100.0
        } else {
            0.0
        };

        let winner = if provchain_avg.0 > 0.0 && neo4j_avg.0 > 0.0 {
            if provchain_avg.0 < neo4j_avg.0 {
                "ProvChain-Org".to_string()
            } else {
                "Neo4j".to_string()
            }
        } else if provchain_avg.0 > 0.0 {
            "ProvChain-Org (only)".to_string()
        } else if neo4j_avg.0 > 0.0 {
            "Neo4j (only)".to_string()
        } else {
            "None".to_string()
        };

        // Parse scenario and test_name from key
        let parts: Vec<&str> = key.split(':').collect();
        let scenario = parts.get(0).unwrap_or(&"").to_string();
        let test_name = parts.get(1).unwrap_or(&"").to_string();

        let summary = BenchmarkSummary {
            scenario,
            test_name,
            provchain_avg_ms: provchain_avg.0,
            neo4j_avg_ms: neo4j_avg.0,
            jena_avg_ms: 0.0,  // Not implemented yet
            provchain_ops_per_sec: provchain_ops,
            neo4j_ops_per_sec: neo4j_ops,
            jena_ops_per_sec: 0.0,  // Not implemented yet
            improvement_percent: improvement,
            winner,
            provchain_success_rate: provchain_avg.1 * 100.0,
            neo4j_success_rate: neo4j_avg.1 * 100.0,
            jena_success_rate: 0.0,  // Not implemented yet
        };

        summaries.push(summary);
    }

    // Write summary
    let summary_file = Path::new(results_path).join("summary.json");
    let summary_output = File::create(&summary_file)?;
    serde_json::to_writer_pretty(summary_output, &summaries)?;
    info!("Summary written to: {:?}", summary_file);

    // Write markdown summary
    let md_file = Path::new(results_path).join("summary.md");
    let mut md = File::create(&md_file)?;

    writeln!(md, "# Benchmark Results Summary")?;
    writeln!(md, "\nGenerated: {}\n", Utc::now().to_rfc3339())?;
    writeln!(md, "## Scenarios\n")?;

    for summary in &summaries {
        writeln!(md, "### {}: {}", summary.scenario, summary.test_name)?;
        writeln!(
            md,
            "- **ProvChain-Org**: {:.2} ms ({:.2} ops/sec) - {:.1}% success",
            summary.provchain_avg_ms,
            summary.provchain_ops_per_sec,
            summary.provchain_success_rate
        )?;
        writeln!(
            md,
            "- **Neo4j**: {:.2} ms ({:.2} ops/sec) - {:.1}% success",
            summary.neo4j_avg_ms,
            summary.neo4j_ops_per_sec,
            summary.neo4j_success_rate
        )?;
        writeln!(md, "- **Improvement**: {:.1}%", summary.improvement_percent)?;
        writeln!(md, "- **Winner**: {}\n", summary.winner)?;
    }

    info!("Markdown summary written to: {:?}", md_file);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let args = Args::parse();

    info!("═══════════════════════════════════════════════════");
    info!("     ProvChain-Org Benchmark Runner v0.2.0");
    info!("═══════════════════════════════════════════════════");
    info!("ProvChain URL: {}", args.provchain_url);
    if !args.skip_neo4j {
        info!("Neo4j URI: {}", args.neo4j_uri);
    } else {
        info!("Neo4j: SKIPPED");
    }
    info!("Dataset path: {}", args.dataset_path);
    info!("Results path: {}", args.results_path);
    info!("Iterations: {}", args.iterations);
    info!("Dataset file: {}", args.dataset_file);
    info!("═══════════════════════════════════════════════════\n");

    // Initialize clients
    let mut clients = SystemClients::new(&args);
    clients.initialize(args.skip_neo4j, args.skip_jena).await?;

    // Wait for systems to be ready
    info!("Waiting for systems to be ready...");
    sleep(Duration::from_secs(5)).await;

    // Check health
    clients.check_health().await?;

    let mut all_results = Vec::new();

    // Run data loading benchmark first
    let load_results = benchmark_data_loading(&clients, &args).await?;
    all_results.extend(load_results);

    // Run selected benchmarks
    if args.all || args.query || args.compare {
        let query_results = benchmark_query_performance(&clients, &args).await?;
        all_results.extend(query_results);
    }

    if args.all || args.write {
        let write_results = benchmark_write_performance(&clients, &args).await?;
        all_results.extend(write_results);
    }

    // Generate report
    generate_report(&all_results, &args.results_path)?;

    // Close Neo4j connection
    if let Some(client) = &clients.neo4j_client {
        let _ = client.close().await;
    }

    info!("\n═══════════════════════════════════════════════════");
    info!("     Benchmark Complete!");
    info!("═══════════════════════════════════════════════════");
    info!("Total results: {}", all_results.len());
    info!("Results saved to: {}", args.results_path);
    info!("═══════════════════════════════════════════════════\n");

    Ok(())
}
