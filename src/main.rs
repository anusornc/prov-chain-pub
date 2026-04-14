use chrono::Utc;
use clap::{Parser, Subcommand};
use provchain_org::{
    config::Config,
    core::blockchain::Blockchain,
    demo,
    demo_runner::run_demo_with_args,
    network::{consensus::ConsensusManager, profile::NetworkProfile, NetworkManager},
    ontology::{OntologyConfig, OntologyPackageManifest},
    security::keys::generate_signing_key,
    semantic::enhanced_owl2_demo::run_enhanced_owl2_demo,
    semantic::owl2_traceability::Owl2EnhancedTraceability,
    semantic::simple_owl2_test::simple_owl2_integration_test,
    storage::rdf_store::StorageConfig,
    utils::config::load_config,
    web::server::create_web_server,
};

use std::fs;
use std::path::Path;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "TraceChain")]
#[command(about = "Blockchain + RDF Traceability CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a Turtle RDF file as a new block
    AddFile {
        path: String,
        /// Domain ontology to use for validation (e.g., ontologies/uht_manufacturing.owl)
        #[arg(long)]
        ontology: Option<String>,
    },

    /// Run a SPARQL query file
    Query {
        path: String,
        /// Domain ontology to use for validation (e.g., ontologies/uht_manufacturing.owl)
        #[arg(long)]
        ontology: Option<String>,
    },

    /// Validate the integrity of the blockchain
    Validate {
        /// Domain ontology to use for validation (e.g., ontologies/uht_manufacturing.owl)
        #[arg(long)]
        ontology: Option<String>,
    },

    /// Dump the blockchain to stdout as JSON
    Dump,

    /// Run example demonstrations and demos
    Examples {
        #[command(subcommand)]
        command: ExampleCommands,
    },

    /// Run legacy enhanced traceability demo using experimental OWL2 code
    EnhancedTrace {
        /// Batch ID to trace
        batch_id: String,

        /// Optimization level (0-2)
        #[arg(short, long, default_value = "1")]
        optimization: u8,

        /// Domain ontology to use for validation (e.g., ontologies/uht_manufacturing.owl)
        #[arg(long)]
        ontology: Option<String>,
    },

    /// Run experimental OWL2 reasoning demo using the owl2-reasoner library
    AdvancedOwl2 {
        /// Ontology file to process
        #[arg(
            short,
            long,
            default_value = "src/semantic/ontologies/generic_core.owl"
        )]
        ontology: String,
    },
    /// Trace the shortest path between two entities in the knowledge graph
    TracePath {
        /// Start entity URI
        #[arg(long)]
        from: String,
        /// End entity URI
        #[arg(long)]
        to: String,
        /// Domain ontology to use for validation (e.g., ontologies/uht_manufacturing.owl)
        #[arg(long)]
        ontology: Option<String>,
    },

    /// Start a full node with networking and consensus
    StartNode {
        /// Config file path
        #[arg(short, long)]
        config: Option<String>,
    },

    /// Generate a new Ed25519 keypair for authority nodes
    GenerateKey {
        /// Output file path for the private key
        #[arg(short, long)]
        out: String,
    },
}

#[derive(Subcommand)]
enum ExampleCommands {
    /// List all available examples and demos
    List,

    /// Run basic supply chain demo (simple traceability)
    BasicSupplyChain {
        /// Domain ontology to use for validation
        #[arg(long)]
        ontology: Option<String>,
    },

    /// Run transaction workflow demo (signing, multi-party)
    TransactionWorkflow {
        /// Demo type: uht, basic, signing, multi, all, interactive
        #[arg(short, long, default_value = "interactive")]
        workflow_type: String,
        /// Domain ontology to use for validation
        #[arg(long)]
        ontology: Option<String>,
    },

    /// Run OWL2 reasoning demo (hasKey, property chains, cardinality)
    Owl2Reasoning {
        /// Domain ontology to use for validation
        #[arg(long)]
        ontology: Option<String>,
    },

    /// Run GS1 EPCIS UHT supply chain demo (comprehensive)
    Gs1EpcisUht {
        /// Domain ontology to use for validation
        #[arg(long)]
        ontology: Option<String>,
    },

    /// Start web server with demo data
    WebServer {
        /// Port to run the web server on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Domain ontology to use for validation
        #[arg(long)]
        ontology: Option<String>,
    },
}

/// Generate demo data based on the selected ontology
fn generate_demo_data(ontology_config: &OntologyConfig) -> Vec<String> {
    let domain_name = ontology_config
        .domain_name()
        .unwrap_or_else(|_| "generic".to_string());
    let timestamp = Utc::now().to_rfc3339();

    match domain_name.as_str() {
        "uht_manufacturing" => generate_uht_demo_data(&timestamp),
        "automotive" => generate_automotive_demo_data(&timestamp),
        "pharmaceutical" => generate_pharmaceutical_demo_data(&timestamp),
        "healthcare" => generate_healthcare_demo_data(&timestamp),
        _ => generate_generic_demo_data(&timestamp),
    }
}

/// Generate UHT manufacturing specific demo data
fn generate_uht_demo_data(timestamp: &str) -> Vec<String> {
    vec![
        // UHT Product with required properties
        format!(
            r#"<http://provchain.org/item/uht-product-1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/uht#UHTProduct> .
<http://provchain.org/item/uht-product-1> <http://provchain.org/trace#name> "Organic Whole Milk" .
<http://provchain.org/item/uht-product-1> <http://provchain.org/trace#participant> "Dairy Farms Co." .
<http://provchain.org/item/uht-product-1> <http://provchain.org/trace#status> "Fresh" .
<http://provchain.org/item/uht-product-1> <http://provchain.org/uht#milkType> "Whole" .
<http://provchain.org/item/uht-product-1> <http://provchain.org/uht#fatContent> "3.5"^^<http://www.w3.org/2001/XMLSchema#decimal> .
<http://provchain.org/item/uht-product-1> <http://provchain.org/uht#proteinContent> "3.2"^^<http://www.w3.org/2001/XMLSchema#decimal> .
<http://provchain.org/item/uht-product-1> <http://provchain.org/uht#expiryDate> "{timestamp}"^^<http://www.w3.org/2001/XMLSchema#date> .
<http://provchain.org/item/uht-product-1> <http://provchain.org/uht#packageSize> "1.0"^^<http://www.w3.org/2001/XMLSchema#decimal> ."#
        ),
        // UHT Processing activity
        format!(
            r#"<http://provchain.org/activity/uht-processing-1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/uht#UHTProcessing> .
<http://provchain.org/activity/uht-processing-1> <http://provchain.org/uht#heatingTemperature> "140.0"^^<http://www.w3.org/2001/XMLSchema#decimal> .
<http://provchain.org/activity/uht-processing-1> <http://provchain.org/uht#heatingDuration> "5.0"^^<http://www.w3.org/2001/XMLSchema#decimal> .
<http://provchain.org/activity/uht-processing-1> <http://provchain.org/uht#coolingTemperature> "6.0"^^<http://www.w3.org/2001/XMLSchema#decimal> .
<http://provchain.org/activity/uht-processing-1> <http://provchain.org/trace#timestamp> "{timestamp}"^^<http://www.w3.org/2001/XMLSchema#dateTime> .
<http://provchain.org/activity/uht-processing-1> <http://provchain.org/trace#participant> "UHT Processing Plant" ."#
        ),
        // Batch linking to product
        format!(
            r###"<http://example.org/uht-batch1> <http://provchain.org/trace#product> <http://provchain.org/item/uht-product-1> .\n<http://example.org/uht-batch1> <http://provchain.org/trace#origin> \"Dairy Farm A\" .\n<http://example.org/uht-batch1> <http://provchain.org/trace#status> \"Processed\" .\n<http://example.org/uht-batch1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/core#Batch> ."###
        ),
    ]
}

/// Generate automotive specific demo data
fn generate_automotive_demo_data(timestamp: &str) -> Vec<String> {
    vec![
        // Automotive part with required properties
        format!(r###"<http://provchain.org/item/auto-part-1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/automotive#AutomotivePart> .\n<http://provchain.org/item/auto-part-1> <http://provchain.org/trace#name> \"Engine Control Unit\" .\n<http://provchain.org/item/auto-part-1> <http://provchain.org/trace#participant> \"AutoParts Inc.\" .\n<http://provchain.org/item/auto-part-1> <http://provchain.org/trace#status> \"Manufactured\" .\n<http://provchain.org/item/auto-part-1> <http://provchain.org/automotive#partNumber> \"ECU2023001\" .\n<http://provchain.org/item/auto-part-1> <http://provchain.org/automotive#vehicleModel> \"Model S\" .\n<http://provchain.org/item/auto-part-1> <http://provchain.org/automotive#partCategory> \"Electrical\" .\n<http://provchain.org/item/auto-part-1> <http://provchain.org/automotive#materialType> \"Electronic Components\" .\n<http://provchain.org/item/auto-part-1> <http://provchain.org/automotive#weight> \"0.5\"^^<http://www.w3.org/2001/XMLSchema#decimal> .\n<http://provchain.org/item/auto-part-1> <http://provchain.org/automotive#serialNumber> \"ECU1234567890\" ."###),

        // Manufacturing activity
        format!(r#"<http://provchain.org/activity/auto-mfg-1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/automotive#AutomotiveManufacturing> .
<http://provchain.org/activity/auto-mfg-1> <http://provchain.org/automotive#manufacturingProcess> "Assembly" .
<http://provchain.org/activity/auto-mfg-1> <http://provchain.org/automotive#productionLine> "LINE001" .
<http://provchain.org/activity/auto-mfg-1> <http://provchain.org/automotive#batchSize> "100"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://provchain.org/activity/auto-mfg-1> <http://provchain.org/automotive#manufacturingDate> "{timestamp}"^^<http://www.w3.org/2001/XMLSchema#date> .
<http://provchain.org/activity/auto-mfg-1> <http://provchain.org/automotive#plantCode> "FACT001" .
<http://provchain.org/activity/auto-mfg-1> <http://provchain.org/trace#participant> "Manufacturing Plant" ."#),

        // Batch
        r###"<http://example.org/auto-batch1> <http://provchain.org/trace#product> <http://provchain.org/item/auto-part-1> .\n<http://example.org/auto-batch1> <http://provchain.org/trace#origin> \"Factory A\" .\n<http://example.org/auto-batch1> <http://provchain.org/trace#status> \"Quality Checked\" .\n<http://example.org/auto-batch1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/core#Batch> ."###.to_string(),
    ]
}

/// Generate pharmaceutical specific demo data
fn generate_pharmaceutical_demo_data(timestamp: &str) -> Vec<String> {
    vec![
        // Pharmaceutical product
        format!(r#"<http://provchain.org/item/pharma-product-1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/pharmaceutical#PharmaceuticalProduct> .
<http://provchain.org/item/pharma-product-1> <http://provchain.org/trace#name> "Amoxicillin 500mg" .
<http://provchain.org/item/pharma-product-1> <http://provchain.org/trace#participant> "PharmaCorp" .
<http://provchain.org/item/pharma-product-1> <http://provchain.org/trace#status> "Approved" .
<http://provchain.org/item/pharma-product-1> <http://provchain.org/pharmaceutical#batchNumber> "PHA-2023-001" .
<http://provchain.org/item/pharma-product-1> <http://provchain.org/pharmaceutical#dosage> "500mg" .
<http://provchain.org/item/pharma-product-1> <http://provchain.org/pharmaceutical#expiryDate> "{timestamp}"^^<http://www.w3.org/2001/XMLSchema#date> ."#),

        // Quality control
        format!(r#"<http://provchain.org/activity/pharma-qc-1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/pharmaceutical#PharmaceuticalQualityControl> .
<http://provchain.org/activity/pharma-qc-1> <http://provchain.org/pharmaceutical#testResult> "Passed" .
<http://provchain.org/activity/pharma-qc-1> <http://provchain.org/pharmaceutical#labTechnician> "Dr. Smith" .
<http://provchain.org/activity/pharma-qc-1> <http://provchain.org/trace#timestamp> "{timestamp}"^^<http://www.w3.org/2001/XMLSchema#dateTime> .
<http://provchain.org/activity/pharma-qc-1> <http://provchain.org/trace#participant> "Quality Lab" ."#),

        // Batch
        r###"<http://example.org/pharma-batch1> <http://provchain.org/trace#product> <http://provchain.org/item/pharma-product-1> .\n<http://example.org/pharma-batch1> <http://provchain.org/trace#origin> \"Manufacturing Facility A\" .\n<http://example.org/pharma-batch1> <http://provchain.org/trace#status> \"Released\" .\n<http://example.org/pharma-batch1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/core#Batch> ."###.to_string(),
    ]
}

/// Generate healthcare specific demo data
fn generate_healthcare_demo_data(timestamp: &str) -> Vec<String> {
    vec![
        // Healthcare product
        format!(r#"<http://provchain.org/item/healthcare-product-1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/healthcare#HealthcareProduct> .
<http://provchain.org/item/healthcare-product-1> <http://provchain.org/trace#name> "Surgical Gloves" .
<http://provchain.org/item/healthcare-product-1> <http://provchain.org/trace#participant> "MediSupply Inc." .
<http://provchain.org/item/healthcare-product-1> <http://provchain.org/trace#status> "Sterilized" .
<http://provchain.org/item/healthcare-product-1> <http://provchain.org/healthcare#sterilizationMethod> "Gamma Radiation" .
<http://provchain.org/item/healthcare-product-1> <http://provchain.org/healthcare#lotNumber> "MED-2023-001" .
<http://provchain.org/item/healthcare-product-1> <http://provchain.org/healthcare#expiryDate> "{timestamp}"^^<http://www.w3.org/2001/XMLSchema#date> ."#),

        // Healthcare activity
        format!(r#"<http://provchain.org/activity/healthcare-activity-1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/healthcare#HealthcareActivity> .
<http://provchain.org/activity/healthcare-activity-1> <http://provchain.org/healthcare#procedure> "Sterilization" .
<http://provchain.org/activity/healthcare-activity-1> <http://provchain.org/healthcare#operator> "Sterilization Technician" .
<http://provchain.org/activity/healthcare-activity-1> <http://provchain.org/trace#timestamp> "{timestamp}"^^<http://www.w3.org/2001/XMLSchema#dateTime> .
<http://provchain.org/activity/healthcare-activity-1> <http://provchain.org/trace#participant> "Sterilization Department" ."#),

        // Batch
        r###"<http://example.org/healthcare-batch1> <http://provchain.org/trace#product> <http://provchain.org/item/healthcare-product-1> .\n<http://example.org/healthcare-batch1> <http://provchain.org/trace#origin> \"Medical Manufacturing Plant\" .\n<http://example.org/healthcare-batch1> <http://provchain.org/trace#status> \"Ready for Distribution\" .\n<http://example.org/healthcare-batch1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/core#Batch> ."###.to_string(),
    ]
}

/// Generate generic demo data as fallback
fn generate_generic_demo_data(timestamp: &str) -> Vec<String> {
    vec![
        // Generic product
        format!(r###"<http://provchain.org/item/generic-product-1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/core#Product> .\n<http://provchain.org/item/generic-product-1> <http://provchain.org/trace#name> \"Generic Product\" .\n<http://provchain.org/item/generic-product-1> <http://provchain.org/trace#participant> \"Generic Supplier\" .\n<http://provchain.org/item/generic-product-1> <http://provchain.org/trace#status> \"Active\" .\n<http://provchain.org/item/generic-product-1> <http://provchain.org/trace#location> \"Warehouse A\" ."###),

        // Generic activity
        format!(r###"<http://provchain.org/activity/generic-activity-1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/core#Process> .\n<http://provchain.org/activity/generic-activity-1> <http://provchain.org/trace#timestamp> \"{timestamp}\"^^<http://www.w3.org/2001/XMLSchema#dateTime> .\n<http://provchain.org/activity/generic-activity-1> <http://provchain.org/trace#participant> \"Generic Processor\" ."###),

        // Batch
        r###"<http://example.org/generic-batch1> <http://provchain.org/trace#product> <http://provchain.org/item/generic-product-1> .\n<http://example.org/generic-batch1> <http://provchain.org/trace#origin> \"Generic Origin\" .\n<http://example.org/generic-batch1> <http://provchain.org/trace#status> \"Processed\" .\n<http://example.org/generic-batch1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/core#Batch> ."###.to_string(),
    ]
}

/// Helper function to create blockchain with ontology configuration
fn create_blockchain_with_ontology(
    ontology_path: Option<String>,
) -> Result<Blockchain, Box<dyn std::error::Error>> {
    let data_dir = "data";
    // Ensure data directory exists
    if !Path::new(data_dir).exists() {
        fs::create_dir_all(data_dir)
            .map_err(|e| format!("Failed to create data directory: {e}"))?;
    }

    if let Some(ontology_path) = ontology_path {
        info!(
            "Initializing persistent blockchain with domain ontology: {}\n",
            ontology_path
        );

        // Create ontology configuration
        let config = Config::load_or_default("config/config.toml");
        let ontology_config = OntologyConfig::new(Some(ontology_path.clone()), &config)
            .map_err(|e| format!("Failed to create ontology configuration: {e}"))?;

        // Create persistent blockchain with ontology
        let blockchain = Blockchain::new_persistent_with_ontology(data_dir, ontology_config)
            .map_err(|e| {
                format!(
                    "Failed to initialize persistent blockchain with ontology: {}\n",
                    e
                )
            })?;

        info!(
            "✅ Persistent Blockchain initialized with domain ontology: {}\n",
            ontology_path
        );
        Ok(blockchain)
    } else {
        info!("Initializing persistent blockchain without domain ontology\n");
        Ok(Blockchain::new_persistent(data_dir)
            .map_err(|e| format!("Failed to initialize persistent blockchain: {e}\n"))?)
    }
}

/// Create blockchain for node runtime using node-local storage config and optional ontology config
fn create_blockchain_for_node(
    node_config: &provchain_org::utils::config::NodeConfig,
) -> Result<Blockchain, Box<dyn std::error::Error>> {
    let storage_config = StorageConfig {
        data_dir: std::path::PathBuf::from(node_config.storage.data_dir.clone()),
        enable_backup: true,
        backup_interval_hours: 24,
        max_backup_files: 7,
        enable_compression: true,
        enable_encryption: false,
        cache_size: 1000,
        warm_cache_on_startup: false,
        flush_interval: 1,
    };

    if let Some(ontology) = &node_config.ontology {
        if ontology.auto_load || ontology.validate_data {
            let ontology_config = if let Some(manifest_path) = &ontology.package_manifest_path {
                let manifest = OntologyPackageManifest::load_from_file(manifest_path)
                    .map_err(|e| format!("Failed to load ontology package manifest: {e}"))?;
                manifest
                    .to_ontology_config()
                    .map_err(|e| format!("Failed to create ontology configuration: {e}"))?
            } else {
                let app_config = Config::load_or_default("config/config.toml");
                OntologyConfig::new(Some(ontology.path.clone()), &app_config)
                    .map_err(|e| format!("Failed to create ontology configuration: {e}"))?
            };

            info!(
                "Initializing node blockchain with shared ontology package: {} (hash: {})",
                ontology_config
                    .domain_name()
                    .unwrap_or_else(|_| "unknown".to_string()),
                ontology_config.ontology_hash
            );

            return Blockchain::new_persistent_with_config_and_ontology(
                storage_config,
                ontology_config,
            )
            .map_err(|e| format!("Failed to initialize ontology-aware blockchain: {e}").into());
        }
    }

    Blockchain::new_persistent_with_config(storage_config)
        .map_err(|e| format!("Failed to initialize blockchain: {e}").into())
}

fn validate_node_startup_contract(
    node_config: &provchain_org::utils::config::NodeConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(profile_path) = &node_config.network_profile_path {
        let profile = NetworkProfile::load_from_file(profile_path)
            .map_err(|e| format!("Failed to load network profile: {e}"))?;
        profile
            .validate_node_config(node_config)
            .map_err(|e| format!("Node config does not match network profile: {e}"))?;

        if let Some(ontology) = &node_config.ontology {
            if let Some(manifest_path) = &ontology.package_manifest_path {
                let manifest = OntologyPackageManifest::load_from_file(manifest_path)
                    .map_err(|e| format!("Failed to load ontology package manifest: {e}"))?;
                profile.validate_manifest(&manifest).map_err(|e| {
                    format!("Ontology package manifest does not match network profile: {e}")
                })?;

                if ontology.path != manifest.domain_ontology_path {
                    return Err(format!(
                        "Node ontology path '{}' does not match ontology package manifest '{}'",
                        ontology.path, manifest.domain_ontology_path
                    )
                    .into());
                }
            }
        }

        info!(
            "Validated network profile '{}' for network '{}'",
            profile.profile_id, profile.network_id
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::AddFile { path, ontology } => {
            let mut blockchain = create_blockchain_with_ontology(ontology)?;

            let rdf_data = fs::read_to_string(&path)
                .map_err(|e| format!("Cannot read RDF file '{path}': {e}\n"))?;

            blockchain
                .add_block(rdf_data)
                .map_err(|e| format!("Failed to add block: {e}\n"))?;
            let block_hash = blockchain
                .chain
                .last()
                .map(|b| b.hash.clone())
                .unwrap_or_else(|| "unknown".to_string());

            println!("Added RDF as a new block with hash: {block_hash}\n");
            println!("Blockchain is valid: {}\n", blockchain.is_valid());
        }
        Commands::Query { path, ontology } => {
            let blockchain = create_blockchain_with_ontology(ontology)?;

            let query = fs::read_to_string(&path)
                .map_err(|e| format!("Cannot read query file '{path}': {e}\n"))?;

            let _results = blockchain.rdf_store.query(&query);
            println!("Query results:\n");
            // For now, just print that query was executed
            println!("Query executed successfully\n");
        }
        Commands::Validate { ontology } => {
            let blockchain = create_blockchain_with_ontology(ontology)?;

            if blockchain.is_valid() {
                println!("✅ Blockchain is valid.\n");
            } else {
                println!("❌ Blockchain is NOT valid.\n");
            }
        }
        Commands::Dump => {
            let blockchain = Blockchain::new();
            match blockchain.dump() {
                Ok(json) => println!("{json}\n"),
                Err(e) => {
                    eprintln!("Error dumping blockchain: {e}\n");
                    std::process::exit(1);
                }
            }
        }
        Commands::Examples { command } => {
            match command {
                ExampleCommands::List => {
                    println!(
                        "\n╔══════════════════════════════════════════════════════════════════╗"
                    );
                    println!(
                        "║              AVAILABLE EXAMPLES AND DEMOS                        ║"
                    );
                    println!(
                        "╠══════════════════════════════════════════════════════════════════╣"
                    );
                    println!(
                        "║                                                                  ║"
                    );
                    println!(
                        "║  Built-in Examples:                                              ║"
                    );
                    println!(
                        "║    examples basic-supply-chain    Simple supply chain demo       ║"
                    );
                    println!(
                        "║    examples transaction-workflow  Signing & multi-party          ║"
                    );
                    println!(
                        "║    examples owl2-reasoning        OWL2 features demo             ║"
                    );
                    println!(
                        "║    examples gs1-epcis-uht         GS1 EPCIS UHT supply chain     ║"
                    );
                    println!(
                        "║    examples web-server            Web UI with demo data          ║"
                    );
                    println!(
                        "║                                                                  ║"
                    );
                    println!("║  Standalone Examples (cargo run --example <name>):              ║");
                    println!(
                        "║    gs1_epcis_uht_demo             Full UHT demo (776 lines)      ║"
                    );
                    println!(
                        "║    demo_ui                        Web UI starter                 ║"
                    );
                    println!(
                        "║    persistence_demo               Persistence layer demo         ║"
                    );
                    println!(
                        "║                                                                  ║"
                    );
                    println!(
                        "║  Legacy Commands (still available):                              ║"
                    );
                    println!(
                        "║    transaction-demo               (use: examples list)           ║"
                    );
                    println!(
                        "║    demo                           (use: examples list)           ║"
                    );
                    println!(
                        "║                                                                  ║"
                    );
                    println!(
                        "╚══════════════════════════════════════════════════════════════════╝\n"
                    );
                }
                ExampleCommands::BasicSupplyChain { ontology } => {
                    let _blockchain = create_blockchain_with_ontology(ontology)?;
                    info!("Running basic supply chain demo...\n");
                    demo::run_demo();
                }
                ExampleCommands::TransactionWorkflow {
                    workflow_type,
                    ontology,
                } => {
                    let _blockchain = create_blockchain_with_ontology(ontology)?;
                    info!("Running transaction workflow demo: {}\n", workflow_type);
                    let args = vec!["provchain".to_string(), workflow_type];
                    if let Err(e) = run_demo_with_args(args) {
                        eprintln!("Demo error: {}\n", e);
                        std::process::exit(1);
                    }
                }
                ExampleCommands::Owl2Reasoning { ontology } => {
                    let _blockchain = create_blockchain_with_ontology(ontology)?;
                    info!("Running OWL2 reasoning demo...\n");

                    println!("\n--- Phase 1: Simple Integration Test ---\n");
                    if let Err(e) = simple_owl2_integration_test() {
                        eprintln!("OWL2 integration test failed: {}\n", e);
                        std::process::exit(1);
                    }
                    println!("✅ OWL2 integration test passed!\n");

                    println!("\n--- Phase 2: Enhanced OWL2 Features Demo ---\n");
                    run_enhanced_owl2_demo();
                    println!("✅ Enhanced OWL2 demo completed successfully!\n");
                }
                ExampleCommands::Gs1EpcisUht { ontology: _ } => {
                    println!("\nℹ️  To run the full GS1 EPCIS UHT demo, use:\n");
                    println!("   cargo run --example gs1_epcis_uht_demo\n");
                    println!("   This provides the complete 8-phase demonstration\n");
                }
                ExampleCommands::WebServer { port, ontology } => {
                    // Initialize blockchain with ontology configuration
                    let mut blockchain = create_blockchain_with_ontology(ontology.clone())?;

                    info!("Starting web server with demo data on port {}\n", port);

                    // Load ontology data first
                    info!("Loading core ontology...\n");
                    let ontology_data =
                        fs::read_to_string("src/semantic/ontologies/generic_core.owl")
                            .map_err(|e| format!("Cannot read ontology file: {e}\n"))?;
                    blockchain
                        .add_block(ontology_data)
                        .map_err(|e| format!("Failed to add ontology block: {e}\n"))?;

                    // Generate ontology-aware demo data
                    let ontology_config = OntologyConfig::new(
                        ontology,
                        &Config::load_or_default("config/config.toml"),
                    )
                    .map_err(|e| format!("Failed to create ontology config: {e}\n"))?;
                    let demo_data = generate_demo_data(&ontology_config);

                    let demo_data_count = demo_data.len();

                    // Add each piece of demo data as a separate block
                    for data in demo_data {
                        blockchain
                            .add_block(data)
                            .map_err(|e| format!("Failed to add block: {e}\n"))?;
                    }

                    info!(
                        "Loaded {} blocks (1 ontology + {} demo data)\n",
                        blockchain.chain.len(),
                        demo_data_count
                    );

                    // Create config with custom port
                    let mut config = Config::load_or_default("config/config.toml");
                    config.web.port = port;

                    // Create and start the web server
                    let web_server = create_web_server(blockchain, Some(config)).await?;

                    info!("🚀 Web server starting...\n");
                    info!("📡 API available at: http://localhost:{}\n", port);
                    info!("🔍 Health check: http://localhost:{}/health\n", port);
                    info!("🔐 Login endpoint: http://localhost:{}/auth/login\n", port);
                    info!(
                        "📊 Blockchain status: http://localhost:{}/api/blockchain/status\n",
                        port
                    );
                    info!("");
                    info!("Default users for testing:\n");
                    info!("  - admin/admin123 (Admin role)\n");
                    info!("  - farmer1/farmer123 (Farmer role)\n");
                    info!("  - processor1/processor123 (Processor role)\n");
                    info!("");
                    info!("Press Ctrl+C to stop the server\n");

                    web_server.start().await?;
                }
            }
        }
        Commands::EnhancedTrace {
            batch_id,
            optimization,
            ontology,
        } => {
            let blockchain = create_blockchain_with_ontology(ontology)?;

            info!("Running enhanced traceability with OWL2 reasoning...\n");

            // Create the enhanced traceability system
            let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);

            // Run the enhanced trace
            let result = owl2_enhancer.enhanced_trace(&batch_id, optimization);

            // Print results
            println!("=== Enhanced Trace Results ===\n");
            println!("Optimized: {}\n", result.optimized);
            println!("Entities explored: {}\n", result.entities_explored);
            println!("Execution time: {} ms\n", result.execution_time_ms);

            if let Some(improvement) = result.performance_improvement {
                println!("Performance improvement: {:.2}x\n", improvement);
            }

            println!("Trace path:\n");
            for (i, event) in result.path.iter().enumerate() {
                println!(
                    "  {}. {}: {} -> {}\n",
                    i + 1,
                    event.entity,
                    event.relationship,
                    event.source.as_ref().unwrap_or(&"unknown".to_string())
                );
            }

            println!("✅ Enhanced trace completed successfully!\n");
        }
        Commands::AdvancedOwl2 { ontology } => {
            use provchain_org::semantic::library_integration::{
                check_consistency, validate_ontology,
            };

            println!("=== Advanced OWL2 Reasoning ===\n");
            println!("Processing ontology: {}\n", ontology);

            // 1. Validation
            println!("\n--- Validation ---\n");
            match validate_ontology(&ontology) {
                Ok(report) => {
                    println!("Validation Report:\n");
                    println!("  All Passed: {}\n", report.all_passed);
                    println!("  Total Tests: {}\n", report.statistics.total_tests);
                    println!("  Passed: {}\n", report.statistics.passed_tests);
                    println!("  Failed: {}\n", report.statistics.failed_tests);
                    println!(
                        "  Total Time: {} ms\n",
                        report.statistics.total_execution_time_ms
                    );

                    if !report.results.is_empty() {
                        println!("  Results:\n");
                        for result in &report.results {
                            println!(
                                "    - {}: {} ({})\n",
                                result.test_name,
                                if result.passed { "PASS" } else { "FAIL" },
                                result.message
                            );
                        }
                    }

                    if report.all_passed {
                        println!("✅ Ontology is valid according to AcademicValidator\n");
                    } else {
                        println!("⚠️  Ontology needs improvement\n");
                    }
                }
                Err(e) => println!("❌ Validation failed: {}\n", e),
            }

            // 2. Consistency Checking
            println!("\n--- Consistency Checking ---\n");
            match check_consistency(&ontology) {
                Ok(consistent) => {
                    if consistent {
                        println!("✅ Ontology is consistent\n");
                    } else {
                        println!("❌ Ontology is INCONSISTENT\n");
                    }
                }
                Err(e) => println!("❌ Consistency checking failed: {}\n", e),
            }
        }
        Commands::TracePath { from, to, ontology } => {
            use provchain_org::knowledge_graph::{builder::GraphBuilder, graph_db::GraphDatabase};

            let blockchain = create_blockchain_with_ontology(ontology)?;

            info!("Building knowledge graph from blockchain data...\n");
            let builder = GraphBuilder::new(blockchain.rdf_store);
            let kg = builder
                .build_knowledge_graph()
                .map_err(|e| format!("Failed to build knowledge graph: {e}\n"))?;

            info!("Initializing graph database...\n");
            let graph_db = GraphDatabase::new(kg);

            info!("Tracing path from '{}' to '{}'...\n", from, to);
            match graph_db.find_shortest_path(&from, &to) {
                Some(path) => {
                    println!("✅ Path found:\n");
                    for (i, node) in path.iter().enumerate() {
                        println!("  {}. {}\n", i + 1, node);
                    }
                }
                None => {
                    println!("❌ No path found between '{}' and '{}'\n", from, to);
                }
            }
        }
        Commands::StartNode { config } => {
            // Load configuration
            let node_config = load_config(config.as_deref())
                .map_err(|e| format!("Failed to load config: {e}\n"))?;
            validate_node_startup_contract(&node_config)
                .map_err(|e| format!("Startup contract validation failed: {e}\n"))?;

            info!("Starting node {}...\n", node_config.node_id);
            info!("Network ID: {}\n", node_config.network.network_id);
            info!("Listen Address: {}\n", node_config.listen_address());

            // Initialize components
            let blockchain = Arc::new(RwLock::new(create_blockchain_for_node(&node_config)?));

            let network = NetworkManager::new(node_config.clone());
            let network_arc = Arc::new(network);

            let consensus = ConsensusManager::new(
                node_config.consensus.clone(),
                network_arc.clone(),
                blockchain.clone(),
            )
            .await
            .map_err(|e| format!("Failed to initialize consensus: {e}\n"))?;

            // Register consensus manager as message handler
            network_arc
                .add_message_handler(Box::new(consensus.clone()))
                .await;

            // Start services
            tokio::spawn(async move {
                if let Err(e) = consensus.start().await {
                    error!("Consensus error: {}\n", e);
                }
            });

            tokio::spawn(async move {
                if let Err(e) = network_arc.start().await {
                    error!("Network error: {}\n", e);
                }
            });

            info!("Node started successfully. Press Ctrl+C to stop.\n");

            // Wait for interrupt signal
            tokio::signal::ctrl_c().await?;
            info!("Shutting down...\n");
        }
        Commands::GenerateKey { out } => {
            // Use cryptographically secure key generation
            let keypair = generate_signing_key()
                .map_err(|e| format!("Failed to generate secure key: {e}\n"))?;
            let public_key = keypair.verifying_key();

            // Save private key to file
            fs::write(&out, keypair.to_bytes())
                .map_err(|e| format!("Failed to write key to file: {e}\n"))?;

            println!("Generated new authority keypair\n");
            println!("Private key saved to: {}\n", out);
            println!("Public key (hex): {}\n", hex::encode(public_key.to_bytes()));
        }
    }

    Ok(())
}
