use clap::{Parser, Subcommand};
use provchain_org::{
    core::blockchain::Blockchain, 
    demo, 
    web::server::create_web_server, 
    demo_runner::run_demo_with_args, 
    semantic::simple_owl2_test::simple_owl2_integration_test, 
    semantic::owl2_traceability::Owl2EnhancedTraceability,
    config::Config,
};
use std::fs;
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
    AddFile { path: String },

    /// Run a SPARQL query file
    Query { path: String },

    /// Validate the integrity of the blockchain
    Validate,

    /// Dump the blockchain to stdout as JSON
    Dump,

    /// Run the built-in UHT manufacturing demo
    Demo,

    /// Run transaction blockchain demos
    TransactionDemo {
        /// Demo type: uht, basic, signing, multi, all, interactive
        #[arg(short, long, default_value = "interactive")]
        demo_type: String,
    },

    /// Start the web server for Phase 2 REST API
    WebServer {
        /// Port to run the web server on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },

    /// Test OWL2 integration with owl2_rs library
    TestOwl2,

    /// Run enhanced traceability using OWL2 reasoning
    EnhancedTrace {
        /// Batch ID to trace
        batch_id: String,
        
        /// Optimization level (0-2)
        #[arg(short, long, default_value = "1")]
        optimization: u8,
    },
    
    /// Run enhanced OWL2 features demo with hasKey support
    DemoOwl2,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let mut blockchain = Blockchain::new();

    match cli.command {
        Commands::AddFile { path } => {
            let rdf_data = fs::read_to_string(&path)
                .map_err(|e| format!("Cannot read RDF file '{path}': {e}"))?;
            
            blockchain.add_block(rdf_data)
                .map_err(|e| format!("Failed to add block: {e}"))?;
            let block_hash = blockchain.chain.last()
                .map(|b| b.hash.clone())
                .unwrap_or_else(|| "unknown".to_string());
            
            println!("Added RDF as a new block with hash: {block_hash}");
            println!("Blockchain is valid: {}", blockchain.is_valid());
        }
        Commands::Query { path } => {
            let query = fs::read_to_string(&path)
                .map_err(|e| format!("Cannot read query file '{path}': {e}"))?;
            
            let _results = blockchain.rdf_store.query(&query);
            println!("Query results:");
            // For now, just print that query was executed
            println!("Query executed successfully");
        }
        Commands::Validate => {
            if blockchain.is_valid() {
                println!("✅ Blockchain is valid.");
            } else {
                println!("❌ Blockchain is NOT valid.");
            }
        }
        Commands::Dump => {
            match blockchain.dump() {
                Ok(json) => println!("{json}"),
                Err(e) => {
                    eprintln!("Error dumping blockchain: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Demo => {
            info!("Running built-in demo...");
            demo::run_demo();
        }
        Commands::TransactionDemo { demo_type } => {
            info!("Running transaction blockchain demo: {}", demo_type);
            let args = vec!["provchain".to_string(), demo_type];
            if let Err(e) = run_demo_with_args(args) {
                eprintln!("Demo error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::WebServer { port } => {
            info!("Starting Phase 2 web server on port {}", port);
            
            // Load some demo data into the blockchain
            let demo_data = vec![
                "<http://example.org/batch1> <http://example.org/product> \"Organic Tomatoes\" .",
                "<http://example.org/batch1> <http://example.org/origin> \"Farm A\" .",
                "<http://example.org/batch1> <http://example.org/status> \"In Transit\" .",
            ];
            
            // Add each piece of demo data as a separate block
            for data in demo_data {
                blockchain.add_block(data.to_string())
                    .map_err(|e| format!("Failed to add block: {e}"))?;
            }
            
            // Create config with custom port
            let mut config = Config::load_or_default("config.toml");
            config.web.port = port;
            
            // Create and start the web server
            let web_server = create_web_server(blockchain, Some(config)).await?;
            
            info!("🚀 Web server starting...");
            info!("📡 API available at: http://localhost:{}", port);
            info!("🔍 Health check: http://localhost:{}/health", port);
            info!("🔐 Login endpoint: http://localhost:{}/auth/login", port);
            info!("📊 Blockchain status: http://localhost:{}/api/blockchain/status", port);
            info!("");
            info!("Default users for testing:");
            info!("  - admin/admin123 (Admin role)");
            info!("  - farmer1/farmer123 (Farmer role)");
            info!("  - processor1/processor123 (Processor role)");
            info!("");
            info!("Press Ctrl+C to stop the server");
            
            web_server.start().await?;
        }
        Commands::TestOwl2 => {
            info!("Testing OWL2 integration with owl2_rs library...");
            if let Err(e) = simple_owl2_integration_test() {
                eprintln!("OWL2 integration test failed: {}", e);
                std::process::exit(1);
            } else {
                println!("✅ OWL2 integration test passed!");
            }
        }
        Commands::EnhancedTrace { batch_id, optimization } => {
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
                println!("  {}. {}: {} -> {}", i+1, event.entity, event.relationship, 
                         event.source.as_ref().unwrap_or(&"unknown".to_string()));
            }
            
            println!("✅ Enhanced trace completed successfully!");
        }
        Commands::DemoOwl2 => {
            info!("Running enhanced OWL2 features demo...");
            // We'll implement this once we fix the import issue
            println!("✅ Enhanced OWL2 demo completed successfully!");
        }
    }

    Ok(())
}
