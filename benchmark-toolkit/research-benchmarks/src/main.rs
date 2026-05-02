use anyhow::Result;
use benchmark_runner::{
    default_trace_query_scenarios,
    neo4j_client::{Neo4jClient, Neo4jConfig},
    parse_batch_ids, BenchmarkFamily, BenchmarkResult, BenchmarkSummary, CapabilityPath,
    FabricAdapter, FabricConfig, FabricPolicyCheckRequest, FabricRecord, FabricRecordPayload,
    FabricRecordPolicy, FairnessLabel, FlureeAdapter, FlureeConfig, GethAdapter, GethConfig,
    GethTransactionRequest, GraphDbAdapter, GraphDbConfig, MetricType, MetricUnit,
    Neo4jTraceAdapter, ProvChainAdapter, ProvChainPolicyCheckRequest, SystemSummary,
    TigerGraphAdapter, TigerGraphConfig, TraceQueryAdapter, TraceQueryKind,
};
use chrono::Utc;
use clap::Parser;
use libc::statvfs;
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{error, info, warn};

const GETH_BENCHMARK_CONTRACT_INIT_CODE: &str =
    "0x601d600c600039601d6000f360016004355560043560243560443560006000a3600160005260206000f3";

fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

fn env_f64(name: &str) -> Option<f64> {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
}

fn average_timings(
    totals_ms: &HashMap<String, f64>,
    samples: usize,
) -> HashMap<String, serde_json::Value> {
    if samples == 0 {
        return HashMap::new();
    }

    totals_ms
        .iter()
        .map(|(stage, total)| (stage.clone(), serde_json::json!(total / samples as f64)))
        .collect()
}

fn benchmark_result_from_trace(
    trace_result: benchmark_runner::TraceQueryResult,
    iteration: usize,
    metadata: HashMap<String, serde_json::Value>,
) -> BenchmarkResult {
    BenchmarkResult::from_trace_result(trace_result, iteration, "Trace Query", metadata)
}

#[derive(Serialize)]
struct CsvBenchmarkRow {
    family: String,
    fairness_label: String,
    capability_path: String,
    metric_type: String,
    unit: String,
    system: String,
    scenario: String,
    test_name: String,
    iteration: usize,
    duration_ms: f64,
    operations_per_second: f64,
    success: bool,
    error_message: Option<String>,
    timestamp: String,
    metadata_json: String,
}

impl From<&BenchmarkResult> for CsvBenchmarkRow {
    fn from(result: &BenchmarkResult) -> Self {
        Self {
            family: result.family.as_str().to_string(),
            fairness_label: result.fairness_label.clone(),
            capability_path: result.capability_path.clone(),
            metric_type: result.metric_type.as_str().to_string(),
            unit: result.unit.clone(),
            system: result.system.clone(),
            scenario: result.scenario.clone(),
            test_name: result.test_name.clone(),
            iteration: result.iteration,
            duration_ms: result.duration_ms,
            operations_per_second: result.operations_per_second,
            success: result.success,
            error_message: result.error_message.clone(),
            timestamp: result.timestamp.to_rfc3339(),
            metadata_json: serde_json::to_string(&result.metadata)
                .unwrap_or_else(|_| "{}".to_string()),
        }
    }
}

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

    /// Run governance/policy benchmark only
    #[arg(long)]
    policy: bool,

    /// Run semantic admission benchmark only
    #[arg(long)]
    semantic: bool,

    /// Skip data loading benchmark rows
    #[arg(long, env = "BENCHMARK_SKIP_LOAD")]
    skip_load: bool,

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

    /// Fluree base URL
    #[arg(long, env = "FLUREE_URL", default_value = "http://localhost:8090")]
    fluree_url: String,

    /// Fluree ledger
    #[arg(long, env = "FLUREE_LEDGER", default_value = "provchain/benchmark")]
    fluree_ledger: String,

    /// GraphDB base URL
    #[arg(long, env = "GRAPHDB_URL", default_value = "http://localhost:17200")]
    graphdb_url: String,

    /// GraphDB repository
    #[arg(long, env = "GRAPHDB_REPOSITORY", default_value = "provchain_smoke")]
    graphdb_repository: String,

    /// GraphDB named graph IRI for benchmark Turtle loads
    #[arg(
        long,
        env = "GRAPHDB_GRAPH_IRI",
        default_value = "http://provchain.org/benchmark/graphdb/default-graph"
    )]
    graphdb_graph_iri: String,

    /// GraphDB request timeout in seconds
    #[arg(long, env = "GRAPHDB_TIMEOUT_SECONDS", default_value = "30")]
    graphdb_timeout_seconds: u64,

    /// Optional GraphDB username
    #[arg(long, env = "GRAPHDB_USERNAME")]
    graphdb_username: Option<String>,

    /// Optional GraphDB password
    #[arg(long, env = "GRAPHDB_PASSWORD")]
    graphdb_password: Option<String>,

    /// TigerGraph RESTPP base URL
    #[arg(long, env = "TIGERGRAPH_URL", default_value = "http://localhost:19000")]
    tigergraph_url: String,

    /// TigerGraph graph name
    #[arg(long, env = "TIGERGRAPH_GRAPH", default_value = "ProvChainTrace")]
    tigergraph_graph: String,

    /// TigerGraph request timeout in seconds
    #[arg(long, env = "TIGERGRAPH_TIMEOUT_SECONDS", default_value = "30")]
    tigergraph_timeout_seconds: u64,

    /// Fabric gateway URL
    #[arg(
        long,
        env = "FABRIC_GATEWAY_URL",
        default_value = "http://localhost:8800"
    )]
    fabric_gateway_url: String,

    /// Fabric channel
    #[arg(long, env = "FABRIC_CHANNEL", default_value = "provchain")]
    fabric_channel: String,

    /// Fabric chaincode
    #[arg(long, env = "FABRIC_CHAINCODE", default_value = "traceability")]
    fabric_chaincode: String,

    /// Geth RPC URL
    #[arg(long, env = "GETH_RPC_URL", default_value = "http://localhost:8545")]
    geth_rpc_url: String,

    /// Geth sender account. Defaults to the first eth_accounts result.
    #[arg(long, env = "GETH_SENDER_ADDRESS")]
    geth_sender_address: Option<String>,

    /// Geth benchmark contract address. If omitted, the runner deploys it.
    #[arg(long, env = "GETH_CONTRACT_ADDRESS")]
    geth_contract_address: Option<String>,

    /// Geth transaction gas limit as a hex quantity.
    #[arg(long, env = "GETH_TX_GAS", default_value = "0x100000")]
    geth_tx_gas: String,

    /// Geth receipt wait timeout in seconds.
    #[arg(long, env = "GETH_CONFIRMATION_TIMEOUT_SECONDS", default_value = "60")]
    geth_confirmation_timeout_seconds: u64,

    /// Geth receipt polling interval in milliseconds.
    #[arg(long, env = "GETH_CONFIRMATION_POLL_MS", default_value = "250")]
    geth_confirmation_poll_ms: u64,

    /// Geth mining mode label stored in benchmark metadata.
    #[arg(long, env = "GETH_MINING_MODE", default_value = "dev-auto")]
    geth_mining_mode: String,

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

    /// Skip Fluree benchmarks
    #[arg(long)]
    skip_fluree: bool,

    /// Skip GraphDB benchmarks
    #[arg(long, env = "BENCHMARK_SKIP_GRAPHDB", default_value = "true")]
    skip_graphdb: bool,

    /// Skip TigerGraph benchmarks
    #[arg(long, env = "BENCHMARK_SKIP_TIGERGRAPH", default_value = "true")]
    skip_tigergraph: bool,

    /// Skip Fabric health checks and future ledger benchmarks
    #[arg(long)]
    skip_fabric: bool,

    /// Skip Geth health checks and future ledger benchmarks
    #[arg(long)]
    skip_geth: bool,

    /// Dataset file name
    #[arg(long, default_value = "supply_chain_1000.ttl")]
    dataset_file: String,

    /// ProvChain dataset file name
    #[arg(
        long,
        env = "PROVCHAIN_DATASET_FILE",
        default_value = "supply_chain_1000.ttl"
    )]
    provchain_dataset_file: String,

    /// Neo4j dataset file name
    #[arg(
        long,
        env = "NEO4J_DATASET_FILE",
        default_value = "supply_chain_1000.ttl"
    )]
    neo4j_dataset_file: String,

    /// Fluree dataset file name
    #[arg(
        long,
        env = "FLUREE_DATASET_FILE",
        default_value = "supply_chain_1000.jsonld"
    )]
    fluree_dataset_file: String,

    /// GraphDB Turtle dataset file name
    #[arg(
        long,
        env = "GRAPHDB_DATASET_FILE",
        default_value = "supply_chain_1000.ttl"
    )]
    graphdb_dataset_file: String,

    /// Batch IDs for testing (comma-separated)
    #[arg(long, default_value = "BATCH001,BATCH010,BATCH017,BATCH025,BATCH050")]
    test_batch_ids: String,

    /// Fabric batch size for ledger write workloads
    #[arg(long, env = "FABRIC_BATCH_SIZE", default_value = "100")]
    fabric_batch_size: usize,
}

/// System clients for benchmarking
struct SystemClients {
    provchain_url: String,
    neo4j_client: Option<Neo4jClient>,
    neo4j_config: Neo4jConfig,
    fluree_adapter: FlureeAdapter,
    graphdb_adapter: GraphDbAdapter,
    tigergraph_adapter: TigerGraphAdapter,
    fabric_adapter: FabricAdapter,
    geth_adapter: GethAdapter,
}

impl SystemClients {
    fn new(args: &Args) -> Self {
        let neo4j_config = Neo4jConfig {
            uri: args.neo4j_uri.clone(),
            user: args.neo4j_user.clone(),
            password: args
                .neo4j_password
                .clone()
                .unwrap_or_else(|| "password".to_string()),
            timeout_secs: 30,
            max_connections: 10,
        };

        let fluree_config = FlureeConfig {
            base_url: args.fluree_url.clone(),
            ledger: args.fluree_ledger.clone(),
        };

        let graphdb_config = GraphDbConfig {
            base_url: args.graphdb_url.clone(),
            repository: args.graphdb_repository.clone(),
            timeout_secs: args.graphdb_timeout_seconds,
            username: args.graphdb_username.clone(),
            password: args.graphdb_password.clone(),
            graph_iri: args.graphdb_graph_iri.clone(),
        };

        let tigergraph_config = TigerGraphConfig {
            base_url: args.tigergraph_url.clone(),
            graph_name: args.tigergraph_graph.clone(),
            timeout_secs: args.tigergraph_timeout_seconds,
        };

        let fabric_config = FabricConfig {
            gateway_url: args.fabric_gateway_url.clone(),
            channel: args.fabric_channel.clone(),
            chaincode: args.fabric_chaincode.clone(),
        };

        let geth_config = GethConfig {
            rpc_url: args.geth_rpc_url.clone(),
        };

        SystemClients {
            provchain_url: args.provchain_url.clone(),
            neo4j_client: None,
            neo4j_config,
            fluree_adapter: FlureeAdapter::new(fluree_config),
            graphdb_adapter: GraphDbAdapter::new(graphdb_config),
            tigergraph_adapter: TigerGraphAdapter::new(tigergraph_config),
            fabric_adapter: FabricAdapter::new(fabric_config),
            geth_adapter: GethAdapter::new(geth_config),
        }
    }

    async fn initialize(&mut self, skip_neo4j: bool) -> Result<()> {
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

        Ok(())
    }

    async fn check_health(&self, args: &Args) -> Result<()> {
        info!("Checking system health...");

        if !args.skip_provchain {
            let provchain_adapter = ProvChainAdapter::new(self.provchain_url.clone());
            match provchain_adapter.health_check().await {
                Ok(true) => info!("✓ ProvChain-Org is healthy"),
                Ok(false) => warn!("ProvChain-Org health check returned unhealthy"),
                Err(e) => {
                    warn!("ProvChain-Org health check failed: {}", e);
                }
            }
        } else {
            info!("⊘ ProvChain-Org: SKIPPED");
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

        if !args.skip_fluree {
            match self.fluree_adapter.health_check().await {
                Ok(true) => info!("✓ Fluree is healthy"),
                Ok(false) => warn!("⚠ Fluree health check returned unhealthy"),
                Err(e) => warn!("⚠ Fluree health check failed: {}", e),
            }
        } else {
            info!("⊘ Fluree: SKIPPED");
        }

        if !args.skip_graphdb {
            match self.graphdb_adapter.health_check().await {
                Ok(true) => info!("✓ GraphDB is healthy"),
                Ok(false) => warn!("⚠ GraphDB health check returned unhealthy"),
                Err(e) => warn!("⚠ GraphDB health check failed: {}", e),
            }
        } else {
            info!("⊘ GraphDB: SKIPPED");
        }

        if !args.skip_tigergraph {
            match self.tigergraph_adapter.health_check().await {
                Ok(true) => info!("✓ TigerGraph is healthy"),
                Ok(false) => warn!("⚠ TigerGraph health check returned unhealthy"),
                Err(e) => warn!("⚠ TigerGraph health check failed: {}", e),
            }
        } else {
            info!("⊘ TigerGraph: SKIPPED");
        }

        if !args.skip_fabric {
            match self.fabric_adapter.health_check().await {
                Ok(true) => info!("✓ Hyperledger Fabric gateway is healthy"),
                Ok(false) => warn!("⚠ Hyperledger Fabric gateway health check returned unhealthy"),
                Err(e) => warn!("⚠ Hyperledger Fabric gateway health check failed: {}", e),
            }
        } else {
            info!("⊘ Hyperledger Fabric: SKIPPED");
        }

        if !args.skip_geth {
            match self.geth_adapter.health_check().await {
                Ok(true) => info!("✓ Go Ethereum (Geth) RPC is healthy"),
                Ok(false) => warn!("⚠ Go Ethereum (Geth) health check returned unhealthy"),
                Err(e) => warn!("⚠ Go Ethereum (Geth) health check failed: {}", e),
            }
        } else {
            info!("⊘ Go Ethereum (Geth): SKIPPED");
        }

        Ok(())
    }

    async fn load_dataset_neo4j(&self, dataset_path: &str, dataset_file: &str) -> Result<Duration> {
        if let Some(client) = &self.neo4j_client {
            let dataset_path = Path::new(dataset_path).join(dataset_file);
            let adapter = Neo4jTraceAdapter::new(client);
            adapter.load_dataset_turtle(&dataset_path).await
        } else {
            Err(anyhow::anyhow!("Neo4j client not initialized"))
        }
    }

    async fn load_dataset_fluree(
        &self,
        dataset_path: &str,
        dataset_file: &str,
    ) -> Result<Duration> {
        let dataset_path = Path::new(dataset_path).join(dataset_file);
        self.fluree_adapter.load_jsonld(&dataset_path).await
    }

    async fn load_dataset_graphdb(
        &self,
        dataset_path: &str,
        dataset_file: &str,
    ) -> Result<Duration> {
        let dataset_path = Path::new(dataset_path).join(dataset_file);
        self.graphdb_adapter.reset_repository().await?;
        self.graphdb_adapter.load_turtle(&dataset_path).await
    }
}

#[derive(Serialize)]
struct EnvironmentManifest {
    generated_at: String,
    benchmark_targets: Vec<String>,
    skipped_targets: Vec<String>,
    dataset_path: String,
    results_path: String,
    iterations: usize,
    warmup_iterations: usize,
    dataset_files: HashMap<String, String>,
    environment: EnvironmentDetails,
    image_refs: HashMap<String, String>,
}

#[derive(Serialize)]
struct EnvironmentDetails {
    os: String,
    arch: String,
    logical_cpu_cores: usize,
    memory_total_kib: Option<u64>,
    results_filesystem_available_bytes: Option<u64>,
}

fn read_mem_total_kib() -> Option<u64> {
    let meminfo = fs::read_to_string("/proc/meminfo").ok()?;
    for line in meminfo.lines() {
        if let Some(value) = line.strip_prefix("MemTotal:") {
            let kib = value.split_whitespace().next()?.parse::<u64>().ok()?;
            return Some(kib);
        }
    }
    None
}

fn filesystem_available_bytes(path: &str) -> Option<u64> {
    let c_path = CString::new(path).ok()?;
    let mut stats = std::mem::MaybeUninit::<statvfs>::uninit();
    let rc = unsafe { statvfs(c_path.as_ptr(), stats.as_mut_ptr()) };
    if rc != 0 {
        return None;
    }
    let stats = unsafe { stats.assume_init() };
    Some((stats.f_bavail as u64) * (stats.f_frsize as u64))
}

fn build_environment_manifest(args: &Args) -> EnvironmentManifest {
    let mut benchmark_targets = vec!["ProvChain-Org".to_string()];
    let mut skipped_targets = Vec::new();

    if args.skip_neo4j {
        skipped_targets.push("Neo4j".to_string());
    } else {
        benchmark_targets.push("Neo4j".to_string());
    }
    if args.skip_fluree {
        skipped_targets.push("Fluree".to_string());
    } else {
        benchmark_targets.push("Fluree".to_string());
    }
    if args.skip_graphdb {
        skipped_targets.push("GraphDB".to_string());
    } else {
        benchmark_targets.push("GraphDB".to_string());
    }
    if args.skip_tigergraph {
        skipped_targets.push("TigerGraph".to_string());
    } else {
        benchmark_targets.push("TigerGraph".to_string());
    }
    if args.skip_fabric {
        skipped_targets.push("Hyperledger Fabric".to_string());
    } else {
        benchmark_targets.push("Hyperledger Fabric".to_string());
    }
    if args.skip_geth {
        skipped_targets.push("Go Ethereum (Geth)".to_string());
    } else {
        benchmark_targets.push("Go Ethereum (Geth)".to_string());
    }

    let mut dataset_files = HashMap::new();
    dataset_files.insert("provchain".to_string(), args.provchain_dataset_file.clone());
    dataset_files.insert("neo4j".to_string(), args.neo4j_dataset_file.clone());
    dataset_files.insert("fluree".to_string(), args.fluree_dataset_file.clone());
    dataset_files.insert("graphdb".to_string(), args.graphdb_dataset_file.clone());
    dataset_files.insert(
        "tigergraph".to_string(),
        "translated CSV/GSQL model preloaded by install-tigergraph-trace-model.sh".to_string(),
    );
    dataset_files.insert(
        "fabric".to_string(),
        format!("logical-records:{}", args.fabric_batch_size),
    );
    dataset_files.insert(
        "geth".to_string(),
        "logical-records:single-submit".to_string(),
    );

    let mut image_refs = HashMap::new();
    if let Ok(value) = std::env::var("BENCHMARK_PROVCHAIN_IMAGE") {
        image_refs.insert("provchain".to_string(), value);
    }
    if let Ok(value) = std::env::var("BENCHMARK_NEO4J_IMAGE") {
        image_refs.insert("neo4j".to_string(), value);
    }
    if let Ok(value) = std::env::var("BENCHMARK_FLUREE_IMAGE") {
        image_refs.insert("fluree".to_string(), value);
    }
    if let Ok(value) = std::env::var("BENCHMARK_GRAPHDB_IMAGE") {
        image_refs.insert("graphdb".to_string(), value);
    }
    if let Ok(value) = std::env::var("BENCHMARK_TIGERGRAPH_IMAGE") {
        image_refs.insert("tigergraph".to_string(), value);
    }
    if let Ok(value) = std::env::var("BENCHMARK_FABRIC_GATEWAY_IMAGE") {
        image_refs.insert("fabric-gateway".to_string(), value);
    }
    if let Ok(value) = std::env::var("BENCHMARK_GETH_IMAGE") {
        image_refs.insert("geth".to_string(), value);
    }
    if let Ok(value) = std::env::var("BENCHMARK_RUNNER_IMAGE") {
        image_refs.insert("benchmark-runner".to_string(), value);
    }

    EnvironmentManifest {
        generated_at: Utc::now().to_rfc3339(),
        benchmark_targets,
        skipped_targets,
        dataset_path: args.dataset_path.clone(),
        results_path: args.results_path.clone(),
        iterations: args.iterations,
        warmup_iterations: args.warmup_iterations,
        dataset_files,
        environment: EnvironmentDetails {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            logical_cpu_cores: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
            memory_total_kib: read_mem_total_kib(),
            results_filesystem_available_bytes: filesystem_available_bytes(&args.results_path),
        },
        image_refs,
    }
}

fn write_environment_manifest(args: &Args) -> Result<()> {
    fs::create_dir_all(&args.results_path)?;
    let manifest = build_environment_manifest(args);
    let file = Path::new(&args.results_path).join("environment_manifest.json");
    let writer = File::create(&file)?;
    serde_json::to_writer_pretty(writer, &manifest)?;
    info!("Environment manifest written to: {:?}", file);
    Ok(())
}

fn percentile(sorted_values: &[f64], fraction: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    let index = ((sorted_values.len() - 1) as f64 * fraction).round() as usize;
    sorted_values[index.min(sorted_values.len() - 1)]
}

fn derive_comparison_status(systems: &[SystemSummary]) -> String {
    if systems.is_empty() {
        return "no-data".to_string();
    }
    if systems.len() == 1 {
        return "single-system-contract".to_string();
    }
    if systems.iter().all(|s| s.successful_runs > 0) {
        return "valid-comparison".to_string();
    }
    if systems.iter().any(|s| s.successful_runs > 0) {
        return "partial-comparison".to_string();
    }
    "invalid-comparison".to_string()
}

/// Run query performance benchmark comparing ProvChain and Neo4j
async fn benchmark_query_performance(
    clients: &SystemClients,
    args: &Args,
) -> Result<Vec<BenchmarkResult>> {
    info!("Starting query performance benchmark...");

    let mut results = Vec::new();
    let batch_ids = parse_batch_ids(&args.test_batch_ids);
    let scenarios = default_trace_query_scenarios(&batch_ids);
    let provchain_adapter = ProvChainAdapter::new(clients.provchain_url.clone());
    let neo4j_adapter = clients.neo4j_client.as_ref().map(Neo4jTraceAdapter::new);
    let fluree_adapter = (!args.skip_fluree).then_some(&clients.fluree_adapter);
    let graphdb_adapter = (!args.skip_graphdb).then_some(&clients.graphdb_adapter);
    let tigergraph_adapter = (!args.skip_tigergraph).then_some(&clients.tigergraph_adapter);

    for scenario in scenarios {
        info!("Trace query scenario: {}", scenario.name);
        match scenario.kind {
            TraceQueryKind::EntityLookup => {
                for batch_id in &scenario.batch_ids {
                    for i in 0..args.iterations {
                        if !args.skip_provchain {
                            let mut metadata = HashMap::new();
                            metadata.insert("batch_id".to_string(), batch_id.clone().into());
                            let trace_result = provchain_adapter.entity_lookup(batch_id).await?;
                            results.push(
                                benchmark_result_from_trace(trace_result, i, metadata)
                                    .with_fairness_label(FairnessLabel::NativeComparable)
                                    .with_capability_path(CapabilityPath::NativeRdfPath),
                            );
                        }

                        if !args.skip_neo4j {
                            if let Some(adapter) = &neo4j_adapter {
                                let mut metadata = HashMap::new();
                                metadata.insert("batch_id".to_string(), batch_id.clone().into());
                                let trace_result = adapter.entity_lookup(batch_id).await?;
                                results.push(
                                    benchmark_result_from_trace(trace_result, i, metadata)
                                        .with_fairness_label(FairnessLabel::NativeComparable)
                                        .with_capability_path(CapabilityPath::TranslatedGraphModel),
                                );
                            }
                        }

                        if let Some(adapter) = fluree_adapter {
                            let mut metadata = HashMap::new();
                            metadata.insert("batch_id".to_string(), batch_id.clone().into());
                            let trace_result = adapter.entity_lookup(batch_id).await?;
                            results.push(
                                benchmark_result_from_trace(trace_result, i, metadata)
                                    .with_fairness_label(FairnessLabel::NativeComparable)
                                    .with_capability_path(CapabilityPath::NativeRdfPath),
                            );
                        }

                        if let Some(adapter) = graphdb_adapter {
                            let mut metadata = HashMap::new();
                            metadata.insert("batch_id".to_string(), batch_id.clone().into());
                            let trace_result = adapter.entity_lookup(batch_id).await?;
                            results.push(
                                benchmark_result_from_trace(trace_result, i, metadata)
                                    .with_fairness_label(FairnessLabel::NativeComparable)
                                    .with_capability_path(CapabilityPath::NativeRdfPath),
                            );
                        }

                        if let Some(adapter) = tigergraph_adapter {
                            let mut metadata = HashMap::new();
                            metadata.insert("batch_id".to_string(), batch_id.clone().into());
                            metadata.insert(
                                "claim_boundary".to_string(),
                                "translated-property-graph-model".into(),
                            );
                            let trace_result = adapter.entity_lookup(batch_id).await?;
                            results.push(
                                benchmark_result_from_trace(trace_result, i, metadata)
                                    .with_fairness_label(FairnessLabel::SecondaryBaseline)
                                    .with_capability_path(
                                        CapabilityPath::TranslatedPropertyGraphModel,
                                    ),
                            );
                        }
                    }
                }
            }
            TraceQueryKind::MultiHop => {
                let hops = scenario.hops.unwrap_or(10);
                for batch_id in &scenario.batch_ids {
                    for i in 0..args.iterations {
                        if !args.skip_provchain {
                            let mut metadata = HashMap::new();
                            metadata.insert("batch_id".to_string(), batch_id.clone().into());
                            metadata.insert("hops".to_string(), hops.into());
                            let trace_result =
                                provchain_adapter.trace_multi_hop(batch_id, hops).await?;
                            results.push(
                                benchmark_result_from_trace(trace_result, i, metadata)
                                    .with_fairness_label(FairnessLabel::NativeComparable)
                                    .with_capability_path(CapabilityPath::NativeRdfPath),
                            );
                        }

                        if !args.skip_neo4j {
                            if let Some(adapter) = &neo4j_adapter {
                                let mut metadata = HashMap::new();
                                metadata.insert("batch_id".to_string(), batch_id.clone().into());
                                metadata.insert("hops".to_string(), hops.into());
                                let trace_result = adapter.trace_multi_hop(batch_id, hops).await?;
                                results.push(
                                    benchmark_result_from_trace(trace_result, i, metadata)
                                        .with_fairness_label(FairnessLabel::NativeComparable)
                                        .with_capability_path(CapabilityPath::TranslatedGraphModel),
                                );
                            }
                        }

                        if let Some(adapter) = fluree_adapter {
                            let mut metadata = HashMap::new();
                            metadata.insert("batch_id".to_string(), batch_id.clone().into());
                            metadata.insert("hops".to_string(), hops.into());
                            let trace_result = adapter.trace_multi_hop(batch_id, hops).await?;
                            results.push(
                                benchmark_result_from_trace(trace_result, i, metadata)
                                    .with_fairness_label(FairnessLabel::NativeComparable)
                                    .with_capability_path(CapabilityPath::NativeRdfPath),
                            );
                        }

                        if let Some(adapter) = graphdb_adapter {
                            let mut metadata = HashMap::new();
                            metadata.insert("batch_id".to_string(), batch_id.clone().into());
                            metadata.insert("hops".to_string(), hops.into());
                            let trace_result = adapter.trace_multi_hop(batch_id, hops).await?;
                            results.push(
                                benchmark_result_from_trace(trace_result, i, metadata)
                                    .with_fairness_label(FairnessLabel::NativeComparable)
                                    .with_capability_path(CapabilityPath::NativeRdfPath),
                            );
                        }

                        if let Some(adapter) = tigergraph_adapter {
                            let mut metadata = HashMap::new();
                            metadata.insert("batch_id".to_string(), batch_id.clone().into());
                            metadata.insert("hops".to_string(), hops.into());
                            metadata.insert(
                                "claim_boundary".to_string(),
                                "translated-property-graph-model".into(),
                            );
                            let trace_result = adapter.trace_multi_hop(batch_id, hops).await?;
                            results.push(
                                benchmark_result_from_trace(trace_result, i, metadata)
                                    .with_fairness_label(FairnessLabel::SecondaryBaseline)
                                    .with_capability_path(
                                        CapabilityPath::TranslatedPropertyGraphModel,
                                    ),
                            );
                        }
                    }
                }
            }
            TraceQueryKind::AggregationByProducer => {
                for i in 0..args.iterations {
                    if !args.skip_provchain {
                        let trace_result = provchain_adapter.aggregation_by_producer().await?;
                        results.push(
                            benchmark_result_from_trace(trace_result, i, HashMap::new())
                                .with_fairness_label(FairnessLabel::NativeComparable)
                                .with_capability_path(CapabilityPath::NativeRdfPath),
                        );
                    }

                    if !args.skip_neo4j {
                        if let Some(adapter) = &neo4j_adapter {
                            let trace_result = adapter.aggregation_by_producer().await?;
                            results.push(
                                benchmark_result_from_trace(trace_result, i, HashMap::new())
                                    .with_fairness_label(FairnessLabel::NativeComparable)
                                    .with_capability_path(CapabilityPath::TranslatedGraphModel),
                            );
                        }
                    }

                    if let Some(adapter) = fluree_adapter {
                        let trace_result = adapter.aggregation_by_producer().await?;
                        results.push(
                            benchmark_result_from_trace(trace_result, i, HashMap::new())
                                .with_fairness_label(FairnessLabel::NativeComparable)
                                .with_capability_path(CapabilityPath::NativeRdfPath),
                        );
                    }

                    if let Some(adapter) = graphdb_adapter {
                        let trace_result = adapter.aggregation_by_producer().await?;
                        results.push(
                            benchmark_result_from_trace(trace_result, i, HashMap::new())
                                .with_fairness_label(FairnessLabel::NativeComparable)
                                .with_capability_path(CapabilityPath::NativeRdfPath),
                        );
                    }

                    if let Some(adapter) = tigergraph_adapter {
                        let mut metadata = HashMap::new();
                        metadata.insert(
                            "claim_boundary".to_string(),
                            "translated-property-graph-model".into(),
                        );
                        let trace_result = adapter.aggregation_by_producer().await?;
                        results.push(
                            benchmark_result_from_trace(trace_result, i, metadata)
                                .with_fairness_label(FairnessLabel::SecondaryBaseline)
                                .with_capability_path(CapabilityPath::TranslatedPropertyGraphModel),
                        );
                    }
                }
            }
        }
    }

    Ok(results)
}

fn fabric_record(record_number: usize) -> FabricRecord {
    fabric_record_with_id(
        format!("fabric-record-{record_number:06}"),
        format!("BATCH{record_number:06}"),
    )
}

fn fabric_record_with_id(record_id: String, entity_id: String) -> FabricRecord {
    FabricRecord {
        record_id,
        payload: FabricRecordPayload {
            entity_id,
            entity_type: "ProductBatch".to_string(),
            event_type: "Produced".to_string(),
            timestamp: "2026-04-24T00:00:00Z".to_string(),
            actor_id: "producer-001".to_string(),
            location_id: Some("site-001".to_string()),
            previous_record_ids: Vec::new(),
            attributes: HashMap::new(),
        },
        policy: FabricRecordPolicy {
            visibility: "public".to_string(),
            owner_org: "Org1MSP".to_string(),
        },
    }
}

fn fabric_policy_record(record_id: String, entity_id: String) -> FabricRecord {
    let mut record = fabric_record_with_id(record_id, entity_id);
    record.policy = FabricRecordPolicy {
        visibility: "restricted".to_string(),
        owner_org: "Org1MSP".to_string(),
    };
    record
}

fn fabric_batch(start: usize, count: usize) -> Vec<FabricRecord> {
    (start..start + count).map(fabric_record).collect()
}

fn strip_hex_prefix(value: &str) -> &str {
    value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value)
}

fn hex_encode_prefixed(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let bytes = value.as_bytes();
    let mut encoded = String::with_capacity(2 + bytes.len() * 2);
    encoded.push_str("0x");
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn parse_hex_quantity(value: Option<&str>) -> Option<u64> {
    let value = value?;
    u64::from_str_radix(strip_hex_prefix(value), 16).ok()
}

async fn geth_sender_address(clients: &SystemClients, args: &Args) -> Result<String> {
    if let Some(sender) = &args.geth_sender_address {
        if !sender.trim().is_empty() {
            return Ok(sender.clone());
        }
    }

    clients
        .geth_adapter
        .accounts()
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Geth returned no unlocked sender accounts"))
}

async fn geth_submit_record_calldata(
    adapter: &GethAdapter,
    iteration: usize,
) -> Result<(String, HashMap<String, serde_json::Value>)> {
    let record_id = format!("geth-record-{iteration:06}");
    let entity_id = format!("BATCH-GETH-{iteration:06}");
    let payload = format!(
        "{}|{}|Produced|producer-001|2026-04-25T00:00:00Z",
        record_id, entity_id
    );

    let selector_hash = adapter
        .web3_sha3(&hex_encode_prefixed(
            "submitRecord(bytes32,bytes32,bytes32)",
        ))
        .await?;
    let selector = strip_hex_prefix(&selector_hash)
        .get(0..8)
        .ok_or_else(|| anyhow::anyhow!("Geth returned short selector hash: {selector_hash}"))?;
    let record_key = adapter.web3_sha3(&hex_encode_prefixed(&record_id)).await?;
    let entity_key = adapter.web3_sha3(&hex_encode_prefixed(&entity_id)).await?;
    let payload_hash = adapter.web3_sha3(&hex_encode_prefixed(&payload)).await?;

    let calldata = format!(
        "0x{}{}{}{}",
        selector,
        strip_hex_prefix(&record_key),
        strip_hex_prefix(&entity_key),
        strip_hex_prefix(&payload_hash)
    );

    let mut metadata = HashMap::new();
    metadata.insert("record_id".to_string(), record_id.into());
    metadata.insert("entity_id".to_string(), entity_id.into());
    metadata.insert("payload_hash".to_string(), payload_hash.into());
    metadata.insert(
        "contract_method".to_string(),
        "submitRecord(bytes32,bytes32,bytes32)".into(),
    );
    Ok((calldata, metadata))
}

async fn ensure_geth_contract(
    clients: &SystemClients,
    args: &Args,
    sender_address: &str,
) -> Result<String> {
    if let Some(address) = &args.geth_contract_address {
        if !address.trim().is_empty() {
            if clients
                .geth_adapter
                .validate_contract_address(address)
                .await?
            {
                return Ok(address.clone());
            }
            anyhow::bail!("configured Geth contract address has no code: {address}");
        }
    }

    info!("Deploying benchmark Geth contract from {sender_address}");
    let deploy_submit = clients
        .geth_adapter
        .send_transaction(&GethTransactionRequest {
            from: sender_address.to_string(),
            to: None,
            data: GETH_BENCHMARK_CONTRACT_INIT_CODE.to_string(),
            gas: Some(args.geth_tx_gas.clone()),
        })
        .await?;
    let deploy_confirmation = clients
        .geth_adapter
        .wait_for_receipt(
            &deploy_submit.tx_hash,
            Duration::from_secs(args.geth_confirmation_timeout_seconds),
            Duration::from_millis(args.geth_confirmation_poll_ms),
        )
        .await?;

    if !deploy_confirmation.success {
        anyhow::bail!(
            "Geth benchmark contract deployment failed: tx_hash={} status={:?}",
            deploy_submit.tx_hash,
            deploy_confirmation.receipt.status
        );
    }

    deploy_confirmation
        .receipt
        .contract_address
        .filter(|address| !address.is_empty())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Geth contract deployment receipt did not include contractAddress for {}",
                deploy_submit.tx_hash
            )
        })
}

/// Run write performance benchmark
async fn benchmark_write_performance(
    clients: &SystemClients,
    args: &Args,
) -> Result<Vec<BenchmarkResult>> {
    info!("Starting write performance benchmark...");

    let mut results = Vec::new();

    if !args.skip_provchain {
        info!("Test 1: Single-threaded write (100 transactions)");
        let provchain_adapter = ProvChainAdapter::new(clients.provchain_url.clone());
        let append_test_name = if args.skip_load {
            "Single-threaded Write (100 tx)"
        } else {
            "Steady-state Append After Cold Load (100 tx)"
        };
        let append_scenario = if args.skip_load {
            "Write Performance"
        } else {
            "Write Performance Steady-State Append"
        };
        let append_phase = if args.skip_load {
            "append_without_load_phase"
        } else {
            "steady_state_after_cold_load"
        };
        for i in 0..args.iterations {
            let timing = provchain_adapter.write_transaction_batch_timed(100).await?;
            let duration = timing.total_duration;

            let mut result = BenchmarkResult::new(
                "ProvChain-Org",
                BenchmarkFamily::LedgerWrite,
                MetricType::SubmitLatencyMs,
                append_scenario,
                append_test_name,
                i,
                duration.as_millis() as f64,
            );
            result.operations_per_second = 100.0 / duration.as_secs_f64();
            result = result
                .with_fairness_label(FairnessLabel::NativeComparable)
                .with_capability_path(CapabilityPath::NativeRdfPath)
                .with_metadata("append_phase", append_phase)
                .with_metadata("cold_load_included_before_append", !args.skip_load)
                .with_metadata("transaction_count", timing.transaction_count)
                .with_metadata("auth_latency_ms", duration_ms(timing.auth_duration))
                .with_metadata(
                    "client_submit_loop_latency_ms",
                    duration_ms(timing.submit_loop_duration),
                )
                .with_metadata(
                    "server_timing_totals_ms",
                    serde_json::json!(timing.server_timing_totals_ms),
                )
                .with_metadata(
                    "server_timing_avg_ms",
                    serde_json::json!(average_timings(
                        &timing.server_timing_totals_ms,
                        timing.server_timing_samples
                    )),
                )
                .with_metadata("server_timing_samples", timing.server_timing_samples)
                .with_metadata("diagnostic_scope", "client-observed");
            results.push(result);

            let auth = BenchmarkResult::new(
                "ProvChain-Org",
                BenchmarkFamily::LedgerWrite,
                MetricType::AuthenticationLatencyMs,
                "Write Performance Diagnostic",
                "Write Batch Authentication",
                i,
                duration_ms(timing.auth_duration),
            )
            .with_fairness_label(FairnessLabel::NotComparable)
            .with_capability_path(CapabilityPath::NativeRdfPath)
            .with_metadata("parent_test", append_test_name)
            .with_metadata("append_phase", append_phase)
            .with_metadata("cold_load_included_before_append", !args.skip_load)
            .with_metadata("transaction_count", timing.transaction_count)
            .with_metadata("server_timing_samples", timing.server_timing_samples)
            .with_metadata("diagnostic_scope", "client-observed");
            results.push(auth);

            let mut submit_loop = BenchmarkResult::new(
                "ProvChain-Org",
                BenchmarkFamily::LedgerWrite,
                MetricType::ClientSubmitLoopLatencyMs,
                "Write Performance Diagnostic",
                "Write Batch HTTP Submit Loop",
                i,
                duration_ms(timing.submit_loop_duration),
            )
            .with_fairness_label(FairnessLabel::NotComparable)
            .with_capability_path(CapabilityPath::NativeRdfPath)
            .with_metadata("parent_test", append_test_name)
            .with_metadata("append_phase", append_phase)
            .with_metadata("cold_load_included_before_append", !args.skip_load)
            .with_metadata("transaction_count", timing.transaction_count)
            .with_metadata(
                "server_timing_avg_ms",
                serde_json::json!(average_timings(
                    &timing.server_timing_totals_ms,
                    timing.server_timing_samples
                )),
            )
            .with_metadata("server_timing_samples", timing.server_timing_samples)
            .with_metadata("diagnostic_scope", "client-observed");
            submit_loop.operations_per_second =
                timing.transaction_count as f64 / timing.submit_loop_duration.as_secs_f64();
            results.push(submit_loop);

            let batch_block_timing = provchain_adapter
                .write_transaction_batch_as_single_block_timed(100)
                .await?;
            let batch_block_duration = batch_block_timing.total_duration;
            let mut batch_block = BenchmarkResult::new(
                "ProvChain-Org",
                BenchmarkFamily::LedgerWrite,
                MetricType::SubmitLatencyMs,
                "Write Performance Diagnostic",
                "Batched Write (100 triples, 1 block)",
                i,
                duration_ms(batch_block_duration),
            );
            batch_block.operations_per_second =
                batch_block_timing.transaction_count as f64 / batch_block_duration.as_secs_f64();
            batch_block = batch_block
                .with_fairness_label(FairnessLabel::NotComparable)
                .with_capability_path(CapabilityPath::NativeRdfPath)
                .with_metadata("parent_test", append_test_name)
                .with_metadata("append_phase", append_phase)
                .with_metadata("cold_load_included_before_append", !args.skip_load)
                .with_metadata("triple_count", batch_block_timing.transaction_count)
                .with_metadata("block_count", batch_block_timing.block_count)
                .with_metadata("batch_semantics", "one_block_many_triples")
                .with_metadata("diagnostic_scope", "single-block-batch")
                .with_metadata(
                    "auth_latency_ms",
                    duration_ms(batch_block_timing.auth_duration),
                )
                .with_metadata(
                    "client_submit_loop_latency_ms",
                    duration_ms(batch_block_timing.submit_loop_duration),
                )
                .with_metadata(
                    "server_timing_totals_ms",
                    serde_json::json!(batch_block_timing.server_timing_totals_ms),
                )
                .with_metadata(
                    "server_timing_avg_ms",
                    serde_json::json!(average_timings(
                        &batch_block_timing.server_timing_totals_ms,
                        batch_block_timing.server_timing_samples
                    )),
                )
                .with_metadata(
                    "server_timing_samples",
                    batch_block_timing.server_timing_samples,
                );
            results.push(batch_block);
        }
    }

    if !args.skip_fabric {
        info!(
            "Running Fabric ledger write workload through gateway contract, batch_size={}",
            args.fabric_batch_size
        );

        for i in 0..args.iterations {
            let single = clients
                .fabric_adapter
                .submit_record(&fabric_record(i + 1))
                .await?;

            let mut submit = BenchmarkResult::new(
                "Hyperledger Fabric",
                BenchmarkFamily::LedgerWrite,
                MetricType::SubmitLatencyMs,
                "Ledger Write",
                "Single Record Submit",
                i,
                single.submit_latency_ms,
            )
            .with_fairness_label(FairnessLabel::NativeComparable)
            .with_capability_path(CapabilityPath::Native)
            .with_metadata("tx_id", single.tx_id.clone().unwrap_or_default())
            .with_metadata("block_number", single.block_number.unwrap_or_default())
            .with_metadata("gateway_contract", "rest");
            submit.success = single.success;
            results.push(submit);

            let mut commit = BenchmarkResult::new(
                "Hyperledger Fabric",
                BenchmarkFamily::LedgerWrite,
                MetricType::CommitLatencyMs,
                "Ledger Write",
                "Single Record Commit",
                i,
                single.commit_latency_ms,
            )
            .with_fairness_label(FairnessLabel::NativeComparable)
            .with_capability_path(CapabilityPath::Native)
            .with_metadata("tx_id", single.tx_id.unwrap_or_default())
            .with_metadata("block_number", single.block_number.unwrap_or_default())
            .with_metadata("gateway_contract", "rest");
            commit.success = single.success;
            results.push(commit);

            let batch = clients
                .fabric_adapter
                .submit_batch(fabric_batch(
                    (i * args.fabric_batch_size) + 1,
                    args.fabric_batch_size,
                ))
                .await?;
            let batch_success_rate = if batch.submitted == 0 {
                0.0
            } else {
                batch.committed as f64 / batch.submitted as f64
            };

            let mut batch_submit = BenchmarkResult::new(
                "Hyperledger Fabric",
                BenchmarkFamily::LedgerWrite,
                MetricType::SubmitLatencyMs,
                "Ledger Write",
                format!("Batch Submit ({} records)", args.fabric_batch_size),
                i,
                batch.submit_latency_ms,
            )
            .with_fairness_label(FairnessLabel::NativeComparable)
            .with_capability_path(CapabilityPath::Native)
            .with_metadata("submitted", batch.submitted)
            .with_metadata("committed", batch.committed)
            .with_metadata("batch_success_rate", batch_success_rate)
            .with_metadata("gateway_contract", "rest");
            batch_submit.success = batch.success;
            batch_submit.operations_per_second = if batch.submit_latency_ms > 0.0 {
                batch.submitted as f64 / (batch.submit_latency_ms / 1000.0)
            } else {
                0.0
            };
            results.push(batch_submit);

            let mut batch_commit = BenchmarkResult::new(
                "Hyperledger Fabric",
                BenchmarkFamily::LedgerWrite,
                MetricType::CommitLatencyMs,
                "Ledger Write",
                format!("Batch Commit ({} records)", args.fabric_batch_size),
                i,
                batch.commit_latency_ms,
            )
            .with_fairness_label(FairnessLabel::NativeComparable)
            .with_capability_path(CapabilityPath::Native)
            .with_metadata("submitted", batch.submitted)
            .with_metadata("committed", batch.committed)
            .with_metadata("batch_success_rate", batch_success_rate)
            .with_metadata("gateway_contract", "rest");
            batch_commit.success = batch.success;
            batch_commit.operations_per_second = if batch.commit_latency_ms > 0.0 {
                batch.committed as f64 / (batch.commit_latency_ms / 1000.0)
            } else {
                0.0
            };
            results.push(batch_commit);
        }
    }

    if !args.skip_geth {
        info!("Running Geth ledger write workload through benchmark smart contract");
        let chain_id = clients.geth_adapter.chain_id().await?;
        let sender_address = geth_sender_address(clients, args).await?;
        let contract_address = ensure_geth_contract(clients, args, &sender_address).await?;

        for i in 0..args.iterations {
            let (calldata, base_metadata) =
                geth_submit_record_calldata(&clients.geth_adapter, i).await?;
            let submit = clients
                .geth_adapter
                .send_transaction(&GethTransactionRequest {
                    from: sender_address.clone(),
                    to: Some(contract_address.clone()),
                    data: calldata,
                    gas: Some(args.geth_tx_gas.clone()),
                })
                .await?;
            let confirmation = clients
                .geth_adapter
                .wait_for_receipt(
                    &submit.tx_hash,
                    Duration::from_secs(args.geth_confirmation_timeout_seconds),
                    Duration::from_millis(args.geth_confirmation_poll_ms),
                )
                .await?;
            let confirmation_latency_ms =
                submit.submit_latency_ms + confirmation.confirmation_latency_ms;
            let gas_used = parse_hex_quantity(confirmation.receipt.gas_used.as_deref());

            let add_common_metadata = |mut result: BenchmarkResult| {
                for (key, value) in &base_metadata {
                    result.metadata.insert(key.clone(), value.clone());
                }
                result
                    .with_fairness_label(FairnessLabel::PublicChainBaseline)
                    .with_capability_path(CapabilityPath::PublicChainSmartContract)
                    .with_metadata("chain_id", chain_id.clone())
                    .with_metadata("contract_address", contract_address.clone())
                    .with_metadata("sender_address", sender_address.clone())
                    .with_metadata("tx_hash", submit.tx_hash.clone())
                    .with_metadata(
                        "block_number",
                        confirmation
                            .receipt
                            .block_number
                            .clone()
                            .unwrap_or_default(),
                    )
                    .with_metadata(
                        "receipt_status",
                        confirmation.receipt.status.clone().unwrap_or_default(),
                    )
                    .with_metadata(
                        "gas_used",
                        confirmation.receipt.gas_used.clone().unwrap_or_default(),
                    )
                    .with_metadata("gas_used_decimal", gas_used.unwrap_or_default())
                    .with_metadata(
                        "effective_gas_price_wei",
                        confirmation
                            .receipt
                            .effective_gas_price
                            .clone()
                            .unwrap_or_default(),
                    )
                    .with_metadata("mining_mode", args.geth_mining_mode.clone())
            };

            let mut submit_result = add_common_metadata(BenchmarkResult::new(
                "Go Ethereum (Geth)",
                BenchmarkFamily::LedgerWrite,
                MetricType::SubmitLatencyMs,
                "Ledger Write",
                "Single Record Submit",
                i,
                submit.submit_latency_ms,
            ));
            submit_result.success = confirmation.success;
            results.push(submit_result);

            let mut confirmation_result = add_common_metadata(BenchmarkResult::new(
                "Go Ethereum (Geth)",
                BenchmarkFamily::LedgerWrite,
                MetricType::ConfirmationLatencyMs,
                "Ledger Write",
                "Single Record Confirmation",
                i,
                confirmation_latency_ms,
            ));
            confirmation_result.success = confirmation.success;
            results.push(confirmation_result);

            let mut gas_result = add_common_metadata(
                BenchmarkResult::new(
                    "Go Ethereum (Geth)",
                    BenchmarkFamily::LedgerWrite,
                    MetricType::GasUsed,
                    "Ledger Write",
                    "Single Record Gas Used",
                    i,
                    gas_used.unwrap_or_default() as f64,
                )
                .with_metric_unit(MetricUnit::Gas),
            );
            gas_result.operations_per_second = 0.0;
            gas_result.success = confirmation.success && gas_used.is_some();
            if gas_used.is_none() {
                gas_result.error_message = Some("receipt did not include gasUsed".to_string());
            }
            results.push(gas_result);
        }
    }

    Ok(results)
}

async fn benchmark_policy_performance(
    clients: &SystemClients,
    args: &Args,
) -> Result<Vec<BenchmarkResult>> {
    info!("Starting policy benchmark...");
    let mut results = Vec::new();

    if args.skip_fabric && args.skip_provchain {
        return Ok(results);
    }

    let provchain_adapter = ProvChainAdapter::new(clients.provchain_url.clone());
    let provchain_policy_token = if args.skip_provchain {
        None
    } else {
        Some(provchain_adapter.authenticate_benchmark_user().await?)
    };

    let checks = [
        (
            "authorized-read",
            "Org1MSP",
            "read",
            true,
            MetricType::AuthorizedReadLatencyMs,
        ),
        (
            "auditor-read",
            "AuditorMSP",
            "read",
            true,
            MetricType::AuthorizedReadLatencyMs,
        ),
        (
            "unauthorized-read",
            "Org2MSP",
            "read",
            false,
            MetricType::UnauthorizedRejectionLatencyMs,
        ),
        (
            "authorized-write",
            "Org1MSP",
            "write",
            true,
            MetricType::AuthorizedWriteLatencyMs,
        ),
        (
            "rejected-write",
            "Org2MSP",
            "write",
            false,
            MetricType::RejectedWriteLatencyMs,
        ),
    ];

    for i in 0..args.iterations {
        let record_id = format!("policy-record-{i:06}");
        if !args.skip_fabric {
            let _ = clients
                .fabric_adapter
                .submit_record(&fabric_policy_record(
                    record_id.clone(),
                    format!("BATCH-POLICY-{i:06}"),
                ))
                .await?;
        }

        for (scenario, actor_org, action, expected, metric_type) in checks {
            if !args.skip_fabric {
                let client_start = Instant::now();
                let response = clients
                    .fabric_adapter
                    .check_policy(&FabricPolicyCheckRequest {
                        record_id: record_id.clone(),
                        actor_org: actor_org.to_string(),
                        action: action.to_string(),
                    })
                    .await?;
                let client_latency_ms = duration_ms(client_start.elapsed());

                let mut result = BenchmarkResult::new(
                    "Hyperledger Fabric",
                    BenchmarkFamily::GovernancePolicy,
                    metric_type,
                    "Policy Enforcement",
                    scenario,
                    i,
                    response.policy_latency_ms,
                )
                .with_fairness_label(FairnessLabel::NativeComparable)
                .with_capability_path(CapabilityPath::Native)
                .with_metadata("actor_org", actor_org)
                .with_metadata("action", action)
                .with_metadata("record_visibility", "restricted")
                .with_metadata("owner_org", "Org1MSP")
                .with_metadata("expected_authorized", expected)
                .with_metadata("actual_authorized", response.authorized)
                .with_metadata("gateway_contract", "rest")
                .with_metadata("policy_storage_model", "ledger-state");
                result.success = response.authorized == expected;
                if !result.success {
                    result.error_message = Some(format!(
                        "policy result mismatch: expected_authorized={expected}, actual_authorized={}",
                        response.authorized
                    ));
                }
                results.push(result);

                let mut client_result = BenchmarkResult::new(
                    "Hyperledger Fabric",
                    BenchmarkFamily::GovernancePolicy,
                    MetricType::PolicyCheckLatencyMs,
                    "Policy Enforcement E2E",
                    scenario,
                    i,
                    client_latency_ms,
                )
                .with_fairness_label(FairnessLabel::NativeComparable)
                .with_capability_path(CapabilityPath::Native)
                .with_metadata("actor_org", actor_org)
                .with_metadata("action", action)
                .with_metadata("record_visibility", "restricted")
                .with_metadata("owner_org", "Org1MSP")
                .with_metadata("expected_authorized", expected)
                .with_metadata("actual_authorized", response.authorized)
                .with_metadata("gateway_contract", "rest")
                .with_metadata("policy_storage_model", "ledger-state")
                .with_metadata("latency_scope", "client-observed-http-gateway-round-trip")
                .with_metadata("reported_policy_latency_ms", response.policy_latency_ms);
                client_result.success = response.authorized == expected;
                if !client_result.success {
                    client_result.error_message = Some(format!(
                        "policy result mismatch: expected_authorized={expected}, actual_authorized={}",
                        response.authorized
                    ));
                }
                results.push(client_result);
            }

            if !args.skip_provchain {
                let request = ProvChainPolicyCheckRequest {
                    record_id: record_id.clone(),
                    actor_org: actor_org.to_string(),
                    action: action.to_string(),
                    owner_org: "Org1MSP".to_string(),
                    visibility: "restricted".to_string(),
                };
                let client_start = Instant::now();
                let response = provchain_adapter
                    .check_policy_with_token(
                        provchain_policy_token
                            .as_deref()
                            .expect("ProvChain policy token initialized"),
                        &request,
                    )
                    .await?;
                let client_latency_ms = duration_ms(client_start.elapsed());

                let mut result = BenchmarkResult::new(
                    "ProvChain-Org",
                    BenchmarkFamily::GovernancePolicy,
                    metric_type,
                    "Policy Enforcement",
                    scenario,
                    i,
                    response.policy_latency_ms,
                )
                .with_fairness_label(FairnessLabel::CrossModelWithCaveat)
                .with_capability_path(CapabilityPath::Native)
                .with_metadata("actor_org", actor_org)
                .with_metadata("action", action)
                .with_metadata("record_visibility", "restricted")
                .with_metadata("owner_org", "Org1MSP")
                .with_metadata("expected_authorized", expected)
                .with_metadata("actual_authorized", response.authorized)
                .with_metadata("gateway_contract", "authenticated-rest")
                .with_metadata("policy_storage_model", "request-carried-policy-context")
                .with_metadata("policy_engine", response.policy_engine.clone())
                .with_metadata("evaluated_by", response.evaluated_by.clone())
                .with_metadata("user_role", response.user_role.clone());
                result.success = response.authorized == expected;
                if !result.success {
                    result.error_message = Some(format!(
                        "policy result mismatch: expected_authorized={expected}, actual_authorized={}",
                        response.authorized
                    ));
                }
                results.push(result);

                let mut client_result = BenchmarkResult::new(
                    "ProvChain-Org",
                    BenchmarkFamily::GovernancePolicy,
                    MetricType::PolicyCheckLatencyMs,
                    "Policy Enforcement E2E",
                    scenario,
                    i,
                    client_latency_ms,
                )
                .with_fairness_label(FairnessLabel::CrossModelWithCaveat)
                .with_capability_path(CapabilityPath::Native)
                .with_metadata("actor_org", actor_org)
                .with_metadata("action", action)
                .with_metadata("record_visibility", "restricted")
                .with_metadata("owner_org", "Org1MSP")
                .with_metadata("expected_authorized", expected)
                .with_metadata("actual_authorized", response.authorized)
                .with_metadata("gateway_contract", "authenticated-rest")
                .with_metadata("policy_storage_model", "request-carried-policy-context")
                .with_metadata("policy_engine", response.policy_engine)
                .with_metadata("evaluated_by", response.evaluated_by)
                .with_metadata("user_role", response.user_role)
                .with_metadata(
                    "latency_scope",
                    "client-observed-authenticated-http-round-trip",
                )
                .with_metadata("reported_policy_latency_ms", response.policy_latency_ms);
                client_result.success = response.authorized == expected;
                if !client_result.success {
                    client_result.error_message = Some(format!(
                        "policy result mismatch: expected_authorized={expected}, actual_authorized={}",
                        response.authorized
                    ));
                }
                results.push(client_result);
            }
        }
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
        let provchain_adapter = ProvChainAdapter::new(clients.provchain_url.clone());
        let dataset_path = Path::new(&args.dataset_path).join(&args.provchain_dataset_file);
        match provchain_adapter
            .load_dataset_turtle_timed(&dataset_path)
            .await
        {
            Ok(timing) => {
                let duration = timing.total_duration;
                let mut result = BenchmarkResult::new(
                    "ProvChain-Org",
                    BenchmarkFamily::LedgerWrite,
                    MetricType::LoadLatencyMs,
                    "Data Loading",
                    "Turtle RDF Import",
                    0,
                    duration.as_millis() as f64,
                );
                result.operations_per_second = timing.triple_count as f64 / duration.as_secs_f64();
                result = result
                    .with_fairness_label(FairnessLabel::NativeComparable)
                    .with_capability_path(CapabilityPath::NativeRdfPath);
                result.metadata.insert(
                    "dataset".to_string(),
                    args.provchain_dataset_file.clone().into(),
                );
                result = result
                    .with_metadata("triple_count", timing.triple_count)
                    .with_metadata("dataset_bytes", timing.dataset_bytes)
                    .with_metadata("block_count", timing.block_count)
                    .with_metadata("provchain_import_mode", timing.import_mode.clone())
                    .with_metadata("load_phase", "cold_load")
                    .with_metadata("append_phase_after_load", "steady_state_after_cold_load")
                    .with_metadata("dataset_read_latency_ms", duration_ms(timing.read_duration))
                    .with_metadata(
                        "dataset_normalize_latency_ms",
                        duration_ms(timing.normalize_duration),
                    )
                    .with_metadata(
                        "dataset_parse_latency_ms",
                        duration_ms(timing.parse_duration),
                    )
                    .with_metadata("auth_latency_ms", duration_ms(timing.auth_duration))
                    .with_metadata(
                        "client_submit_loop_latency_ms",
                        duration_ms(timing.submit_loop_duration),
                    )
                    .with_metadata(
                        "server_timing_totals_ms",
                        serde_json::json!(timing.server_timing_totals_ms),
                    )
                    .with_metadata(
                        "server_timing_avg_ms",
                        serde_json::json!(average_timings(
                            &timing.server_timing_totals_ms,
                            timing.server_timing_samples
                        )),
                    )
                    .with_metadata("server_timing_samples", timing.server_timing_samples)
                    .with_metadata("diagnostic_scope", "client-observed");
                results.push(result);
            }
            Err(e) => {
                let error_chain = format!("{e:#}");
                error!("Failed to load data into ProvChain: {error_chain}");
                results.push(
                    BenchmarkResult::new(
                        "ProvChain-Org",
                        BenchmarkFamily::LedgerWrite,
                        MetricType::LoadLatencyMs,
                        "Data Loading",
                        "Turtle RDF Import",
                        0,
                        0.0,
                    )
                    .with_error(error_chain)
                    .with_fairness_label(FairnessLabel::NativeComparable)
                    .with_capability_path(CapabilityPath::NativeRdfPath),
                );
            }
        }
    }

    // Neo4j data loading
    if !args.skip_neo4j && clients.neo4j_client.is_some() {
        info!("Loading data into Neo4j...");
        match clients
            .load_dataset_neo4j(&args.dataset_path, &args.neo4j_dataset_file)
            .await
        {
            Ok(duration) => {
                let mut result = BenchmarkResult::new(
                    "Neo4j",
                    BenchmarkFamily::LedgerWrite,
                    MetricType::LoadLatencyMs,
                    "Data Loading",
                    "Turtle to Cypher Import",
                    0,
                    duration.as_millis() as f64,
                );
                result.operations_per_second = 1000.0 / duration.as_secs_f64();
                result = result
                    .with_fairness_label(FairnessLabel::SecondaryBaseline)
                    .with_capability_path(CapabilityPath::SecondaryTransactionalBaseline);
                result.metadata.insert(
                    "dataset".to_string(),
                    args.neo4j_dataset_file.clone().into(),
                );
                results.push(result);
            }
            Err(e) => {
                let error_chain = format!("{e:#}");
                error!("Failed to load data into Neo4j: {error_chain}");
                results.push(
                    BenchmarkResult::new(
                        "Neo4j",
                        BenchmarkFamily::LedgerWrite,
                        MetricType::LoadLatencyMs,
                        "Data Loading",
                        "Turtle to Cypher Import",
                        0,
                        0.0,
                    )
                    .with_error(error_chain),
                );
            }
        }
    }

    // Fluree data loading
    if !args.skip_fluree {
        let is_jsonld = args.fluree_dataset_file.ends_with(".json")
            || args.fluree_dataset_file.ends_with(".jsonld");
        if is_jsonld {
            info!("Loading data into Fluree...");
            match clients
                .load_dataset_fluree(&args.dataset_path, &args.fluree_dataset_file)
                .await
            {
                Ok(duration) => {
                    let mut result = BenchmarkResult::new(
                        "Fluree",
                        BenchmarkFamily::LedgerWrite,
                        MetricType::LoadLatencyMs,
                        "Data Loading",
                        "JSON-LD Import",
                        0,
                        duration.as_millis() as f64,
                    );
                    result.operations_per_second = 1000.0 / duration.as_secs_f64();
                    result = result
                        .with_fairness_label(FairnessLabel::NativeComparable)
                        .with_capability_path(CapabilityPath::NativeRdfPath);
                    result.metadata.insert(
                        "dataset".to_string(),
                        args.fluree_dataset_file.clone().into(),
                    );
                    results.push(result);
                }
                Err(e) => {
                    let error_chain = format!("{e:#}");
                    error!("Failed to load data into Fluree: {error_chain}");
                    results.push(
                        BenchmarkResult::new(
                            "Fluree",
                            BenchmarkFamily::LedgerWrite,
                            MetricType::LoadLatencyMs,
                            "Data Loading",
                            "JSON-LD Import",
                            0,
                            0.0,
                        )
                        .with_error(error_chain),
                    );
                }
            }
        } else {
            warn!(
                "Skipping Fluree data loading because dataset '{}' is not JSON-LD",
                args.fluree_dataset_file
            );
        }
    }

    // GraphDB data loading
    if !args.skip_graphdb {
        info!("Loading data into GraphDB...");
        match clients
            .load_dataset_graphdb(&args.dataset_path, &args.graphdb_dataset_file)
            .await
        {
            Ok(duration) => {
                let mut result = BenchmarkResult::new(
                    "GraphDB",
                    BenchmarkFamily::LedgerWrite,
                    MetricType::LoadLatencyMs,
                    "Data Loading",
                    "Turtle RDF Import",
                    0,
                    duration.as_millis() as f64,
                );
                result.operations_per_second = 1000.0 / duration.as_secs_f64();
                result = result
                    .with_fairness_label(FairnessLabel::NativeComparable)
                    .with_capability_path(CapabilityPath::NativeRdfPath);
                result.metadata.insert(
                    "dataset".to_string(),
                    args.graphdb_dataset_file.clone().into(),
                );
                result = result
                    .with_metadata("repository", args.graphdb_repository.clone())
                    .with_metadata("graph_iri", args.graphdb_graph_iri.clone());
                results.push(result);
            }
            Err(e) => {
                let error_chain = format!("{e:#}");
                error!("Failed to load data into GraphDB: {error_chain}");
                results.push(
                    BenchmarkResult::new(
                        "GraphDB",
                        BenchmarkFamily::LedgerWrite,
                        MetricType::LoadLatencyMs,
                        "Data Loading",
                        "Turtle RDF Import",
                        0,
                        0.0,
                    )
                    .with_error(error_chain),
                );
            }
        }
    }

    Ok(results)
}

/// Run semantic admission workloads.
async fn benchmark_semantic_admission(
    clients: &SystemClients,
    args: &Args,
) -> Result<Vec<BenchmarkResult>> {
    info!("Starting semantic admission benchmark...");

    let mut results = Vec::new();

    for iteration in 0..args.iterations {
        if !args.skip_provchain {
            let provchain_adapter = ProvChainAdapter::new(clients.provchain_url.clone());
            let dataset_path = Path::new(&args.dataset_path).join(&args.provchain_dataset_file);
            match provchain_adapter
                .load_dataset_turtle_timed(&dataset_path)
                .await
            {
                Ok(timing) => {
                    let duration = timing.total_duration;
                    let result = BenchmarkResult::new(
                        "ProvChain-Org",
                        BenchmarkFamily::Semantic,
                        MetricType::ValidationLatencyMs,
                        "Semantic Admission",
                        "Native RDF+SHACL Admission",
                        iteration,
                        duration.as_millis() as f64,
                    )
                    .with_fairness_label(FairnessLabel::NativeComparable)
                    .with_capability_path(CapabilityPath::NativeRdfPath)
                    .with_metadata("dataset", args.provchain_dataset_file.clone())
                    .with_metadata("native_semantic_support", true)
                    .with_metadata("external_semantic_stages", serde_json::json!([]))
                    .with_metadata("explanation_support", true)
                    .with_metadata("admission_model", "rdf-native-block-admission")
                    .with_metadata("validation_scope", "native-rdf-shacl-path")
                    .with_metadata("triple_count", timing.triple_count)
                    .with_metadata("dataset_bytes", timing.dataset_bytes)
                    .with_metadata("block_count", timing.block_count)
                    .with_metadata("provchain_import_mode", timing.import_mode.clone())
                    .with_metadata("dataset_read_latency_ms", duration_ms(timing.read_duration))
                    .with_metadata(
                        "dataset_normalize_latency_ms",
                        duration_ms(timing.normalize_duration),
                    )
                    .with_metadata(
                        "dataset_parse_latency_ms",
                        duration_ms(timing.parse_duration),
                    )
                    .with_metadata("auth_latency_ms", duration_ms(timing.auth_duration))
                    .with_metadata(
                        "client_submit_loop_latency_ms",
                        duration_ms(timing.submit_loop_duration),
                    )
                    .with_metadata(
                        "server_timing_totals_ms",
                        serde_json::json!(timing.server_timing_totals_ms),
                    )
                    .with_metadata(
                        "server_timing_avg_ms",
                        serde_json::json!(average_timings(
                            &timing.server_timing_totals_ms,
                            timing.server_timing_samples
                        )),
                    )
                    .with_metadata("server_timing_samples", timing.server_timing_samples)
                    .with_metadata("diagnostic_scope", "client-observed");
                    results.push(result);

                    for (metric_type, test_name, duration) in [
                        (
                            MetricType::DatasetReadLatencyMs,
                            "Native Semantic Dataset Read",
                            timing.read_duration,
                        ),
                        (
                            MetricType::DatasetNormalizeLatencyMs,
                            "Native Semantic Turtle Normalize",
                            timing.normalize_duration,
                        ),
                        (
                            MetricType::DatasetParseLatencyMs,
                            "Native Semantic Turtle Parse",
                            timing.parse_duration,
                        ),
                        (
                            MetricType::AuthenticationLatencyMs,
                            "Native Semantic Authentication",
                            timing.auth_duration,
                        ),
                        (
                            MetricType::ClientSubmitLoopLatencyMs,
                            "Native Semantic HTTP Submit Loop",
                            timing.submit_loop_duration,
                        ),
                    ] {
                        let mut diagnostic = BenchmarkResult::new(
                            "ProvChain-Org",
                            BenchmarkFamily::Semantic,
                            metric_type,
                            "Semantic Admission Diagnostic",
                            test_name,
                            iteration,
                            duration_ms(duration),
                        )
                        .with_fairness_label(FairnessLabel::NotComparable)
                        .with_capability_path(CapabilityPath::NativeRdfPath)
                        .with_metadata("parent_test", "Native RDF+SHACL Admission")
                        .with_metadata("dataset", args.provchain_dataset_file.clone())
                        .with_metadata("triple_count", timing.triple_count)
                        .with_metadata("diagnostic_scope", "client-observed-stage");

                        if matches!(metric_type, MetricType::ClientSubmitLoopLatencyMs) {
                            diagnostic.operations_per_second =
                                timing.triple_count as f64 / duration.as_secs_f64();
                            diagnostic = diagnostic
                                .with_metadata(
                                    "server_timing_avg_ms",
                                    serde_json::json!(average_timings(
                                        &timing.server_timing_totals_ms,
                                        timing.server_timing_samples
                                    )),
                                )
                                .with_metadata(
                                    "server_timing_samples",
                                    timing.server_timing_samples,
                                );
                        }
                        results.push(diagnostic);
                    }
                }
                Err(e) => {
                    results.push(
                        BenchmarkResult::new(
                            "ProvChain-Org",
                            BenchmarkFamily::Semantic,
                            MetricType::ValidationLatencyMs,
                            "Semantic Admission",
                            "Native RDF+SHACL Admission",
                            iteration,
                            0.0,
                        )
                        .with_error(format!("{e:#}"))
                        .with_metadata("dataset", args.provchain_dataset_file.clone())
                        .with_metadata("native_semantic_support", true)
                        .with_metadata("external_semantic_stages", serde_json::json!([]))
                        .with_metadata("explanation_support", true)
                        .with_metadata("admission_model", "rdf-native-block-admission")
                        .with_metadata("validation_scope", "native-rdf-shacl-path"),
                    );
                }
            }
        }

        if !args.skip_fluree {
            let dataset_path = Path::new(&args.dataset_path).join(&args.fluree_dataset_file);
            let is_jsonld = dataset_path
                .extension()
                .and_then(|extension| extension.to_str())
                .map(|extension| matches!(extension, "json" | "jsonld"))
                .unwrap_or(false);

            if is_jsonld {
                match clients
                    .fluree_adapter
                    .load_jsonld_timed(&dataset_path)
                    .await
                {
                    Ok(timing) => {
                        let translation_latency_ms =
                            env_f64("FLUREE_TRANSLATION_LATENCY_MS").unwrap_or(0.0);
                        let external_pipeline_latency_ms =
                            translation_latency_ms + duration_ms(timing.total_duration);

                        let result = BenchmarkResult::new(
                            "Fluree",
                            BenchmarkFamily::Semantic,
                            MetricType::ValidationLatencyMs,
                            "Semantic Admission",
                            "Externalized TTL+JSON-LD Admission",
                            iteration,
                            external_pipeline_latency_ms,
                        )
                        .with_fairness_label(FairnessLabel::ExternalizedSemanticPipeline)
                        .with_capability_path(CapabilityPath::ExternalSemanticPipeline)
                        .with_metadata("dataset", args.fluree_dataset_file.clone())
                        .with_metadata("native_semantic_support", false)
                        .with_metadata(
                            "external_semantic_stages",
                            serde_json::json!([
                                "ttl-to-jsonld-translation",
                                "jsonld-file-read",
                                "jsonld-parse",
                                "jsonld-ledger-insert"
                            ]),
                        )
                        .with_metadata("explanation_support", false)
                        .with_metadata("admission_model", "jsonld-externalized-ledger-admission")
                        .with_metadata(
                            "validation_scope",
                            "jsonld-graph-admission-without-native-shacl",
                        )
                        .with_metadata("translation_cost_included", true)
                        .with_metadata("translation_latency_ms", translation_latency_ms)
                        .with_metadata("jsonld_read_latency_ms", duration_ms(timing.read_duration))
                        .with_metadata(
                            "jsonld_parse_latency_ms",
                            duration_ms(timing.parse_duration),
                        )
                        .with_metadata(
                            "ledger_prepare_latency_ms",
                            duration_ms(timing.ledger_prepare_duration),
                        )
                        .with_metadata(
                            "jsonld_ledger_insert_latency_ms",
                            duration_ms(timing.transact_duration),
                        )
                        .with_metadata(
                            "runner_jsonld_load_latency_ms",
                            duration_ms(timing.total_duration),
                        );
                        results.push(result);

                        for (metric_type, test_name, duration_ms_value) in [
                            (
                                MetricType::MappingLatencyMs,
                                "Externalized TTL-to-JSON-LD Translation",
                                translation_latency_ms,
                            ),
                            (
                                MetricType::DatasetReadLatencyMs,
                                "Externalized JSON-LD Dataset Read",
                                duration_ms(timing.read_duration),
                            ),
                            (
                                MetricType::DatasetParseLatencyMs,
                                "Externalized JSON-LD Parse",
                                duration_ms(timing.parse_duration),
                            ),
                            (
                                MetricType::LoadLatencyMs,
                                "Externalized JSON-LD Ledger Insert",
                                duration_ms(timing.transact_duration),
                            ),
                        ] {
                            results.push(
                                BenchmarkResult::new(
                                    "Fluree",
                                    BenchmarkFamily::Semantic,
                                    metric_type,
                                    "Semantic Admission Diagnostic",
                                    test_name,
                                    iteration,
                                    duration_ms_value,
                                )
                                .with_fairness_label(FairnessLabel::NotComparable)
                                .with_capability_path(CapabilityPath::ExternalSemanticPipeline)
                                .with_metadata("parent_test", "Externalized TTL+JSON-LD Admission")
                                .with_metadata("dataset", args.fluree_dataset_file.clone())
                                .with_metadata("diagnostic_scope", "externalized-semantic-stage"),
                            );
                        }
                    }
                    Err(e) => {
                        results.push(
                            BenchmarkResult::new(
                                "Fluree",
                                BenchmarkFamily::Semantic,
                                MetricType::ValidationLatencyMs,
                                "Semantic Admission",
                                "Externalized TTL+JSON-LD Admission",
                                iteration,
                                0.0,
                            )
                            .with_error(format!("{e:#}"))
                            .with_fairness_label(FairnessLabel::ExternalizedSemanticPipeline)
                            .with_capability_path(CapabilityPath::ExternalSemanticPipeline)
                            .with_metadata("dataset", args.fluree_dataset_file.clone())
                            .with_metadata("native_semantic_support", false)
                            .with_metadata(
                                "external_semantic_stages",
                                serde_json::json!([
                                    "ttl-to-jsonld-translation",
                                    "jsonld-file-read",
                                    "jsonld-parse",
                                    "jsonld-ledger-insert"
                                ]),
                            )
                            .with_metadata("explanation_support", false)
                            .with_metadata(
                                "admission_model",
                                "jsonld-externalized-ledger-admission",
                            )
                            .with_metadata(
                                "validation_scope",
                                "jsonld-graph-admission-without-native-shacl",
                            )
                            .with_metadata("translation_cost_included", true),
                        );
                    }
                }
            } else {
                results.push(
                    BenchmarkResult::new(
                        "Fluree",
                        BenchmarkFamily::Semantic,
                        MetricType::ValidationLatencyMs,
                        "Semantic Admission",
                        "Externalized TTL+JSON-LD Admission",
                        iteration,
                        0.0,
                    )
                    .with_error(format!(
                        "Fluree semantic admission requires JSON-LD, got {}",
                        args.fluree_dataset_file
                    ))
                    .with_fairness_label(FairnessLabel::ExternalizedSemanticPipeline)
                    .with_capability_path(CapabilityPath::ExternalSemanticPipeline)
                    .with_metadata("dataset", args.fluree_dataset_file.clone())
                    .with_metadata("native_semantic_support", false)
                    .with_metadata(
                        "external_semantic_stages",
                        serde_json::json!([
                            "ttl-to-jsonld-translation",
                            "jsonld-file-read",
                            "jsonld-parse",
                            "jsonld-ledger-insert"
                        ]),
                    )
                    .with_metadata("explanation_support", false)
                    .with_metadata("admission_model", "jsonld-externalized-ledger-admission")
                    .with_metadata(
                        "validation_scope",
                        "jsonld-graph-admission-without-native-shacl",
                    )
                    .with_metadata("translation_cost_included", true),
                );
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
        csv_writer.serialize(CsvBenchmarkRow::from(result))?;
    }
    csv_writer.flush()?;
    info!("CSV results written");

    // Calculate summary statistics
    let mut summaries: Vec<BenchmarkSummary> = Vec::new();

    // Group by scenario and test_name
    let mut grouped: HashMap<String, Vec<&BenchmarkResult>> = HashMap::new();
    for result in results {
        let key = format!(
            "{}:{}:{}",
            result.family.as_str(),
            result.scenario,
            result.test_name
        );
        grouped.entry(key).or_default().push(result);
    }

    // Calculate averages and generate summary
    for (key, group_results) in &grouped {
        let mut system_groups: HashMap<String, Vec<&BenchmarkResult>> = HashMap::new();
        for result in group_results {
            system_groups
                .entry(result.system.clone())
                .or_default()
                .push(*result);
        }

        let mut systems = Vec::new();
        for (system, system_results) in &system_groups {
            let successful_runs = system_results.iter().filter(|r| r.success).count();
            let success_rate = if system_results.is_empty() {
                0.0
            } else {
                (successful_runs as f64 / system_results.len() as f64) * 100.0
            };
            let mut latency_source: Vec<f64> = if successful_runs > 0 {
                system_results
                    .iter()
                    .filter(|r| r.success)
                    .map(|r| r.duration_ms)
                    .collect()
            } else {
                system_results.iter().map(|r| r.duration_ms).collect()
            };
            latency_source.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let avg_ms = if latency_source.is_empty() {
                0.0
            } else {
                latency_source.iter().sum::<f64>() / latency_source.len() as f64
            };
            let ops_per_sec = if system_results.is_empty() {
                0.0
            } else {
                system_results
                    .iter()
                    .map(|r| r.operations_per_second)
                    .sum::<f64>()
                    / system_results.len() as f64
            };
            let fairness_label = if system_results
                .iter()
                .all(|r| r.fairness_label == system_results[0].fairness_label)
            {
                system_results[0].fairness_label.clone()
            } else {
                "mixed".to_string()
            };
            let capability_path = if system_results
                .iter()
                .all(|r| r.capability_path == system_results[0].capability_path)
            {
                system_results[0].capability_path.clone()
            } else {
                "mixed".to_string()
            };

            systems.push(SystemSummary {
                system: system.clone(),
                fairness_label,
                capability_path,
                avg_ms,
                p50_ms: percentile(&latency_source, 0.50),
                p95_ms: percentile(&latency_source, 0.95),
                p99_ms: percentile(&latency_source, 0.99),
                ops_per_sec,
                success_rate,
                total_runs: system_results.len(),
                successful_runs,
            });
        }

        systems.sort_by(|a, b| {
            b.success_rate
                .partial_cmp(&a.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.avg_ms
                        .partial_cmp(&b.avg_ms)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        let comparison_status = derive_comparison_status(&systems);
        let winner = if comparison_status == "single-system-contract" {
            "N/A (single system contract)".to_string()
        } else {
            systems
                .iter()
                .find(|summary| summary.success_rate > 0.0 && summary.avg_ms > 0.0)
                .map(|summary| summary.system.clone())
                .unwrap_or_else(|| "None".to_string())
        };
        let fairness_label = if systems
            .iter()
            .all(|summary| summary.fairness_label == systems[0].fairness_label)
        {
            systems[0].fairness_label.clone()
        } else {
            "mixed".to_string()
        };

        // Parse scenario and test_name from key
        let parts: Vec<&str> = key.split(':').collect();
        let family = match parts.first().copied().unwrap_or_default() {
            "ledger-write" => BenchmarkFamily::LedgerWrite,
            "semantic" => BenchmarkFamily::Semantic,
            "governance-policy" => BenchmarkFamily::GovernancePolicy,
            _ => BenchmarkFamily::TraceQuery,
        };
        let scenario = parts.get(1).unwrap_or(&"").to_string();
        let test_name = parts.get(2).unwrap_or(&"").to_string();

        let summary = BenchmarkSummary {
            family,
            scenario,
            test_name,
            fairness_label,
            comparison_status,
            systems,
            winner,
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
        writeln!(
            md,
            "### [{}] {}: {}",
            summary.family.as_str(),
            summary.scenario,
            summary.test_name
        )?;
        writeln!(md, "- **Comparison Status**: {}", summary.comparison_status)?;
        writeln!(md, "- **Fairness Label**: {}", summary.fairness_label)?;
        for system in &summary.systems {
            writeln!(
                md,
                "- **{}**: avg {:.2} ms, p50 {:.2} ms, p95 {:.2} ms, p99 {:.2} ms, {:.2} ops/sec, {:.1}% success ({}/{}) [{} | {}]",
                system.system,
                system.avg_ms,
                system.p50_ms,
                system.p95_ms,
                system.p99_ms,
                system.ops_per_sec,
                system.success_rate,
                system.successful_runs,
                system.total_runs,
                system.fairness_label,
                system.capability_path
            )?;
        }
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
    if !args.skip_fluree {
        info!("Fluree URL: {}", args.fluree_url);
    } else {
        info!("Fluree: SKIPPED");
    }
    if !args.skip_graphdb {
        info!("GraphDB URL: {}", args.graphdb_url);
        info!("GraphDB repository: {}", args.graphdb_repository);
    } else {
        info!("GraphDB: SKIPPED");
    }
    if !args.skip_tigergraph {
        info!("TigerGraph URL: {}", args.tigergraph_url);
        info!("TigerGraph graph: {}", args.tigergraph_graph);
    } else {
        info!("TigerGraph: SKIPPED");
    }
    if !args.skip_fabric {
        info!("Fabric gateway URL: {}", args.fabric_gateway_url);
    } else {
        info!("Hyperledger Fabric: SKIPPED");
    }
    if !args.skip_geth {
        info!("Geth RPC URL: {}", args.geth_rpc_url);
        info!("Geth contract address: {:?}", args.geth_contract_address);
        info!("Geth mining mode: {}", args.geth_mining_mode);
    } else {
        info!("Go Ethereum (Geth): SKIPPED");
    }
    info!("Dataset path: {}", args.dataset_path);
    info!("Results path: {}", args.results_path);
    info!("Iterations: {}", args.iterations);
    info!("Dataset file: {}", args.dataset_file);
    info!("ProvChain dataset file: {}", args.provchain_dataset_file);
    info!("Neo4j dataset file: {}", args.neo4j_dataset_file);
    info!("Fluree dataset file: {}", args.fluree_dataset_file);
    info!("GraphDB dataset file: {}", args.graphdb_dataset_file);
    info!("TigerGraph graph: {}", args.tigergraph_graph);
    info!("Fabric batch size: {}", args.fabric_batch_size);
    info!("═══════════════════════════════════════════════════\n");

    // Initialize clients
    let mut clients = SystemClients::new(&args);
    clients.initialize(args.skip_neo4j).await?;

    // Record the execution environment for every run before benchmarks start.
    write_environment_manifest(&args)?;

    // Wait for systems to be ready
    info!("Waiting for systems to be ready...");
    sleep(Duration::from_secs(5)).await;

    // Check health
    clients.check_health(&args).await?;

    let mut all_results = Vec::new();

    // Run data loading benchmark first unless the campaign is write-path only.
    if args.skip_load {
        info!("⊘ Data loading benchmark: SKIPPED");
    } else {
        let load_results = benchmark_data_loading(&clients, &args).await?;
        all_results.extend(load_results);
    }

    // Run selected benchmarks
    if args.all || args.query || args.compare {
        let query_results = benchmark_query_performance(&clients, &args).await?;
        all_results.extend(query_results);
    }

    if args.all || args.write {
        let write_results = benchmark_write_performance(&clients, &args).await?;
        all_results.extend(write_results);
    }

    if args.all || args.policy {
        let policy_results = benchmark_policy_performance(&clients, &args).await?;
        all_results.extend(policy_results);
    }

    if args.all || args.semantic {
        let semantic_results = benchmark_semantic_admission(&clients, &args).await?;
        all_results.extend(semantic_results);
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
