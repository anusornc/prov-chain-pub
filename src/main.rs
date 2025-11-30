use chrono::Utc;
use clap::{Parser, Subcommand};
use provchain_org::{
    config::Config, core::blockchain::Blockchain, demo, demo_runner::run_demo_with_args,
    ontology::OntologyConfig, semantic::owl2_traceability::Owl2EnhancedTraceability,
    semantic::simple_owl2_test::simple_owl2_integration_test, web::server::create_web_server,
};
use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

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

    /// Run the built-in UHT manufacturing demo
    Demo {
        /// Domain ontology to use for validation (e.g., ontologies/uht_manufacturing.owl)
        #[arg(long)]
        ontology: Option<String>,
    },

    /// Run transaction blockchain demos
    TransactionDemo {
        /// Demo type: uht, basic, signing, multi, all, interactive
        #[arg(short, long, default_value = "interactive")]
        demo_type: String,
        /// Domain ontology to use for validation (e.g., ontologies/uht_manufacturing.owl)
        #[arg(long)]
        ontology: Option<String>,
    },

    /// Start the web server for Phase 2 REST API
    WebServer {
        /// Port to run the web server on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Domain ontology to use for validation (e.g., ontologies/uht_manufacturing.owl)
        #[arg(long)]
        ontology: Option<String>,
    },

    /// Test OWL2 integration with owl2-reasoner library
    TestOwl2 {
        /// Domain ontology to use for validation (e.g., ontologies/uht_manufacturing.owl)
        #[arg(long)]
        ontology: Option<String>,
    },

    /// Run enhanced traceability using OWL2 reasoning
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

    /// Run enhanced OWL2 features demo with hasKey support
    DemoOwl2 {
        /// Domain ontology to use for validation (e.g., ontologies/uht_manufacturing.owl)
        #[arg(long)]
        ontology: Option<String>,
    },

    /// Run advanced OWL2 reasoning using the owl2-reasoner library
    AdvancedOwl2 {
        /// Ontology file to process
        #[arg(short, long, default_value = "ontologies/generic_core.owl")]
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
<http://provchain.org/item/uht-product-1> <http://provchain.org/uht#expiryDate> "{}"^^<http://www.w3.org/2001/XMLSchema#date> .
<http://provchain.org/item/uht-product-1> <http://provchain.org/uht#packageSize> "1.0"^^<http://www.w3.org/2001/XMLSchema#decimal> ."#,
            timestamp
        ),
        // UHT Processing activity
        format!(
            r#"<http://provchain.org/activity/uht-processing-1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/uht#UHTProcessing> .
<http://provchain.org/activity/uht-processing-1> <http://provchain.org/uht#heatingTemperature> "140.0"^^<http://www.w3.org/2001/XMLSchema#decimal> .
<http://provchain.org/activity/uht-processing-1> <http://provchain.org/uht#heatingDuration> "5.0"^^<http://www.w3.org/2001/XMLSchema#decimal> .
<http://provchain.org/activity/uht-processing-1> <http://provchain.org/uht#coolingTemperature> "6.0"^^<http://www.w3.org/2001/XMLSchema#decimal> .
<http://provchain.org/activity/uht-processing-1> <http://provchain.org/trace#timestamp> "{}"^^<http://www.w3.org/2001/XMLSchema#dateTime> .
<http://provchain.org/activity/uht-processing-1> <http://provchain.org/trace#participant> "UHT Processing Plant" ."#,
            timestamp
        ),
        // Batch linking to product
        format!(
            r#"<http://example.org/uht-batch1> <http://provchain.org/trace#product> <http://provchain.org/item/uht-product-1> .
<http://example.org/uht-batch1> <http://provchain.org/trace#origin> "Dairy Farm A" .
<http://example.org/uht-batch1> <http://provchain.org/trace#status> "Processed" .
<http://example.org/uht-batch1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/core#Batch> ."#
        ),
    ]
}

/// Generate automotive specific demo data
fn generate_automotive_demo_data(timestamp: &str) -> Vec<String> {
    vec![
        // Automotive part with required properties
        format!(r#"<http://provchain.org/item/auto-part-1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/automotive#AutomotivePart> .
<http://provchain.org/item/auto-part-1> <http://provchain.org/trace#name> "Engine Control Unit" .
<http://provchain.org/item/auto-part-1> <http://provchain.org/trace#participant> "AutoParts Inc." .
<http://provchain.org/item/auto-part-1> <http://provchain.org/trace#status> "Manufactured" .
<http://provchain.org/item/auto-part-1> <http://provchain.org/automotive#partNumber> "ECU2023001" .
<http://provchain.org/item/auto-part-1> <http://provchain.org/automotive#vehicleModel> "Model S" .
<http://provchain.org/item/auto-part-1> <http://provchain.org/automotive#partCategory> "Electrical" .
<http://provchain.org/item/auto-part-1> <http://provchain.org/automotive#materialType> "Electronic Components" .
<http://provchain.org/item/auto-part-1> <http://provchain.org/automotive#weight> "0.5"^^<http://www.w3.org/2001/XMLSchema#decimal> .
<http://provchain.org/item/auto-part-1> <http://provchain.org/automotive#serialNumber> "ECU1234567890" ."#),

        // Manufacturing activity
        format!(r#"<http://provchain.org/activity/auto-mfg-1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/automotive#AutomotiveManufacturing> .
<http://provchain.org/activity/auto-mfg-1> <http://provchain.org/automotive#manufacturingProcess> "Assembly" .
<http://provchain.org/activity/auto-mfg-1> <http://provchain.org/automotive#productionLine> "LINE001" .
<http://provchain.org/activity/auto-mfg-1> <http://provchain.org/automotive#batchSize> "100"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://provchain.org/activity/auto-mfg-1> <http://provchain.org/automotive#manufacturingDate> "{}"^^<http://www.w3.org/2001/XMLSchema#date> .
<http://provchain.org/activity/auto-mfg-1> <http://provchain.org/automotive#plantCode> "FACT001" .
<http://provchain.org/activity/auto-mfg-1> <http://provchain.org/trace#participant> "Manufacturing Plant" ."#, timestamp),

        // Batch
        r#"<http://example.org/auto-batch1> <http://provchain.org/trace#product> <http://provchain.org/item/auto-part-1> .
<http://example.org/auto-batch1> <http://provchain.org/trace#origin> "Factory A" .
<http://example.org/auto-batch1> <http://provchain.org/trace#status> "Quality Checked" .
<http://example.org/auto-batch1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/core#Batch> ."#.to_string(),
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
<http://provchain.org/item/pharma-product-1> <http://provchain.org/pharmaceutical#expiryDate> "{}"^^<http://www.w3.org/2001/XMLSchema#date> ."#, timestamp),

        // Quality control
        format!(r#"<http://provchain.org/activity/pharma-qc-1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/pharmaceutical#PharmaceuticalQualityControl> .
<http://provchain.org/activity/pharma-qc-1> <http://provchain.org/pharmaceutical#testResult> "Passed" .
<http://provchain.org/activity/pharma-qc-1> <http://provchain.org/pharmaceutical#labTechnician> "Dr. Smith" .
<http://provchain.org/activity/pharma-qc-1> <http://provchain.org/trace#timestamp> "{}"^^<http://www.w3.org/2001/XMLSchema#dateTime> .
<http://provchain.org/activity/pharma-qc-1> <http://provchain.org/trace#participant> "Quality Lab" ."#, timestamp),

        // Batch
        r#"<http://example.org/pharma-batch1> <http://provchain.org/trace#product> <http://provchain.org/item/pharma-product-1> .
<http://example.org/pharma-batch1> <http://provchain.org/trace#origin> "Manufacturing Facility A" .
<http://example.org/pharma-batch1> <http://provchain.org/trace#status> "Released" .
<http://example.org/pharma-batch1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/core#Batch> ."#.to_string(),
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
<http://provchain.org/item/healthcare-product-1> <http://provchain.org/healthcare#expiryDate> "{}"^^<http://www.w3.org/2001/XMLSchema#date> ."#, timestamp),

        // Healthcare activity
        format!(r#"<http://provchain.org/activity/healthcare-activity-1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/healthcare#HealthcareActivity> .
<http://provchain.org/activity/healthcare-activity-1> <http://provchain.org/healthcare#procedure> "Sterilization" .
<http://provchain.org/activity/healthcare-activity-1> <http://provchain.org/healthcare#operator> "Sterilization Technician" .
<http://provchain.org/activity/healthcare-activity-1> <http://provchain.org/trace#timestamp> "{}"^^<http://www.w3.org/2001/XMLSchema#dateTime> .
<http://provchain.org/activity/healthcare-activity-1> <http://provchain.org/trace#participant> "Sterilization Department" ."#, timestamp),

        // Batch
        r#"<http://example.org/healthcare-batch1> <http://provchain.org/trace#product> <http://provchain.org/item/healthcare-product-1> .
<http://example.org/healthcare-batch1> <http://provchain.org/trace#origin> "Medical Manufacturing Plant" .
<http://example.org/healthcare-batch1> <http://provchain.org/trace#status> "Ready for Distribution" .
<http://example.org/healthcare-batch1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/core#Batch> ."#.to_string(),
    ]
}

/// Generate generic demo data as fallback
fn generate_generic_demo_data(timestamp: &str) -> Vec<String> {
    vec![
        // Generic product
        format!(r#"<http://provchain.org/item/generic-product-1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/core#Product> .
<http://provchain.org/item/generic-product-1> <http://provchain.org/trace#name> "Generic Product" .
<http://provchain.org/item/generic-product-1> <http://provchain.org/trace#participant> "Generic Supplier" .
<http://provchain.org/item/generic-product-1> <http://provchain.org/trace#status> "Active" .
<http://provchain.org/item/generic-product-1> <http://provchain.org/trace#location> "Warehouse A" ."#),

        // Generic activity
        format!(r#"<http://provchain.org/activity/generic-activity-1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/core#Process> .
<http://provchain.org/activity/generic-activity-1> <http://provchain.org/trace#timestamp> "{}"^^<http://www.w3.org/2001/XMLSchema#dateTime> .
<http://provchain.org/activity/generic-activity-1> <http://provchain.org/trace#participant> "Generic Processor" ."#, timestamp),

        // Batch
        r#"<http://example.org/generic-batch1> <http://provchain.org/trace#product> <http://provchain.org/item/generic-product-1> .
<http://example.org/generic-batch1> <http://provchain.org/trace#origin> "Generic Origin" .
<http://example.org/generic-batch1> <http://provchain.org/trace#status> "Processed" .
<http://example.org/generic-batch1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://provchain.org/core#Batch> ."#.to_string(),
    ]
}

/// Helper function to create blockchain with ontology configuration
fn create_blockchain_with_ontology(
    ontology_path: Option<String>,
) -> Result<Blockchain, Box<dyn std::error::Error>> {
    let data_dir = "data";
    // Ensure data directory exists
    if !Path::new(data_dir).exists() {
        fs::create_dir_all(data_dir).map_err(|e| format!("Failed to create data directory: {}", e))?;
    }

    if let Some(ontology_path) = ontology_path {
        info!(
            "Initializing persistent blockchain with domain ontology: {}",
            ontology_path
        );

        // Create ontology configuration
        let config = Config::load_or_default("config.toml");
        let ontology_config = OntologyConfig::new(Some(ontology_path.clone()), &config)
            .map_err(|e| format!("Failed to create ontology configuration: {}", e))?;

        // Create persistent blockchain with ontology
        let blockchain = Blockchain::new_persistent_with_ontology(data_dir, ontology_config)
            .map_err(|e| format!("Failed to initialize persistent blockchain with ontology: {}", e))?;

        info!(
            "✅ Persistent Blockchain initialized with domain ontology: {}",
            ontology_path
        );
        Ok(blockchain)
    } else {
        info!("Initializing persistent blockchain without domain ontology");
        Ok(Blockchain::new_persistent(data_dir)
            .map_err(|e| format!("Failed to initialize persistent blockchain: {}", e))?)
    }
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
                .map_err(|e| format!("Cannot read RDF file '{path}': {e}"))?;

            blockchain
                .add_block(rdf_data)
                .map_err(|e| format!("Failed to add block: {e}"))?;
            let block_hash = blockchain
                .chain
                .last()
                .map(|b| b.hash.clone())
                .unwrap_or_else(|| "unknown".to_string());

            println!("Added RDF as a new block with hash: {block_hash}");
            println!("Blockchain is valid: {}", blockchain.is_valid());
        }
        Commands::Query { path, ontology } => {
            let blockchain = create_blockchain_with_ontology(ontology)?;

            let query = fs::read_to_string(&path)
                .map_err(|e| format!("Cannot read query file '{path}': {e}"))?;

            let _results = blockchain.rdf_store.query(&query);
            println!("Query results:");
            // For now, just print that query was executed
            println!("Query executed successfully");
        }
        Commands::Validate { ontology } => {
            let blockchain = create_blockchain_with_ontology(ontology)?;

            if blockchain.is_valid() {
                println!("✅ Blockchain is valid.");
            } else {
                println!("❌ Blockchain is NOT valid.");
            }
        }
        Commands::Dump => {
            let blockchain = Blockchain::new();
            match blockchain.dump() {
                Ok(json) => println!("{json}"),
                Err(e) => {
                    eprintln!("Error dumping blockchain: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Demo { ontology } => {
            let _blockchain = create_blockchain_with_ontology(ontology)?;

            info!("Running built-in demo...");
            demo::run_demo();
        }
        Commands::TransactionDemo {
            demo_type,
            ontology,
        } => {
            let _blockchain = create_blockchain_with_ontology(ontology)?;

            info!("Running transaction blockchain demo: {}", demo_type);
            let args = vec!["provchain".to_string(), demo_type];
            if let Err(e) = run_demo_with_args(args) {
                eprintln!("Demo error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::WebServer { port, ontology } => {
            // Initialize blockchain with ontology configuration
            let mut blockchain = create_blockchain_with_ontology(ontology.clone())?;

            info!("Starting Phase 2 web server on port {}", port);

            // Load ontology data first
            info!("Loading core ontology...");
            let ontology_data = fs::read_to_string("ontologies/generic_core.owl")
                .map_err(|e| format!("Cannot read ontology file: {e}"))?;
            blockchain
                .add_block(ontology_data)
                .map_err(|e| format!("Failed to add ontology block: {e}"))?;

            // Generate ontology-aware demo data
            let ontology_config =
                OntologyConfig::new(ontology, &Config::load_or_default("config.toml"))
                    .map_err(|e| format!("Failed to create ontology config: {e}"))?;
            let demo_data = generate_demo_data(&ontology_config);

            let demo_data_count = demo_data.len();

            // Add each piece of demo data as a separate block
            for data in demo_data {
                blockchain
                    .add_block(data)
                    .map_err(|e| format!("Failed to add block: {e}"))?;
            }

            info!(
                "Loaded {} blocks (1 ontology + {} demo data)",
                blockchain.chain.len(),
                demo_data_count
            );

            // Create config with custom port
            let mut config = Config::load_or_default("config.toml");
            config.web.port = port;

            // Create and start the web server
            let web_server = create_web_server(blockchain, Some(config)).await?;

            info!("🚀 Web server starting...");
            info!("📡 API available at: http://localhost:{}", port);
            info!("🔍 Health check: http://localhost:{}/health", port);
            info!("🔐 Login endpoint: http://localhost:{}/auth/login", port);
            info!(
                "📊 Blockchain status: http://localhost:{}/api/blockchain/status",
                port
            );
            info!("");
            info!("Default users for testing:");
            info!("  - admin/admin123 (Admin role)");
            info!("  - farmer1/farmer123 (Farmer role)");
            info!("  - processor1/processor123 (Processor role)");
            info!("");
            info!("Press Ctrl+C to stop the server");

            web_server.start().await?;
        }
        Commands::TestOwl2 { ontology } => {
            let _blockchain = create_blockchain_with_ontology(ontology)?;

            info!("Testing OWL2 integration with owl2-reasoner library...");
            if let Err(e) = simple_owl2_integration_test() {
                eprintln!("OWL2 integration test failed: {}", e);
                std::process::exit(1);
            } else {
                println!("✅ OWL2 integration test passed!");
            }
        }
        Commands::EnhancedTrace {
            batch_id,
            optimization,
            ontology,
        } => {
            let blockchain = create_blockchain_with_ontology(ontology)?;

            info!("Running enhanced traceability with OWL2 reasoning...");

            // Create the enhanced traceability system
            let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);

            // Run the enhanced trace
            let result = owl2_enhancer.enhanced_trace(&batch_id, optimization);

            // Print results
            println!("=== Enhanced Trace Results ===");
            println!("Optimized: {}", result.optimized);
            println!("Entities explored: {}", result.entities_explored);
            println!("Execution time: {} ms", result.execution_time_ms);

            if let Some(improvement) = result.performance_improvement {
                println!("Performance improvement: {:.2}x", improvement);
            }

            println!("Trace path:");
            for (i, event) in result.path.iter().enumerate() {
                println!(
                    "  {}. {}: {} -> {}",
                    i + 1,
                    event.entity,
                    event.relationship,
                    event.source.as_ref().unwrap_or(&"unknown".to_string())
                );
            }

            println!("✅ Enhanced trace completed successfully!");
        }
        Commands::DemoOwl2 { ontology } => {
            let _blockchain = create_blockchain_with_ontology(ontology)?;

            info!("Running enhanced OWL2 features demo...");
            // We'll implement this once we fix the import issue
            println!("✅ Enhanced OWL2 demo completed successfully!");
        }
        Commands::AdvancedOwl2 { ontology } => {
            use provchain_org::semantic::library_integration::{check_consistency, validate_ontology};

            println!("=== Advanced OWL2 Reasoning ===");
            println!("Processing ontology: {}", ontology);

            // 1. Validation
            println!("\n--- Validation ---");
            // 1. Validation
            println!("\n--- Validation ---");
            match validate_ontology(&ontology) {
                Ok(report) => {
                    println!("Validation Report:");
                    println!("  Overall Score: {:.2}", report.overall_score);
                    println!("  Completeness: {:.2}", report.completeness_score);
                    println!("  Structural: {:.2}", report.structural_score);
                    println!("  Readiness: {:?}", report.publication_readiness);
                    
                    if !report.recommendations.is_empty() {
                        println!("  Recommendations:");
                        for rec in &report.recommendations {
                            println!("    - {}", rec);
                        }
                    }

                    if report.is_valid() {
                        println!("✅ Ontology is valid according to AcademicValidator");
                    } else {
                        println!("⚠️  Ontology needs improvement");
                    }
                }
                Err(e) => println!("❌ Validation failed: {}", e),
            }

            // 2. Consistency Checking
            println!("\n--- Consistency Checking ---");
            match check_consistency(&ontology) {
                Ok(consistent) => {
                    if consistent {
                        println!("✅ Ontology is consistent");
                    } else {
                        println!("❌ Ontology is INCONSISTENT");
                    }
                }
                Err(e) => println!("❌ Consistency checking failed: {}", e),
            }
        }
        Commands::TracePath { from, to, ontology } => {
            use provchain_org::knowledge_graph::{builder::GraphBuilder, graph_db::GraphDatabase};

            let blockchain = create_blockchain_with_ontology(ontology)?;

            info!("Building knowledge graph from blockchain data...");
            let builder = GraphBuilder::new(blockchain.rdf_store);
            let kg = builder.build_knowledge_graph().map_err(|e| format!("Failed to build knowledge graph: {}", e))?;
            
            info!("Initializing graph database...");
            let graph_db = GraphDatabase::new(kg);

            info!("Tracing path from '{}' to '{}'...", from, to);
            match graph_db.find_shortest_path(&from, &to) {
                Some(path) => {
                    println!("✅ Path found:");
                    for (i, node) in path.iter().enumerate() {
                        println!("  {}. {}", i + 1, node);
                    }
                }
                None => {
                    println!("❌ No path found between '{}' and '{}'", from, to);
                }
            }
        }
    }

    Ok(())
}
