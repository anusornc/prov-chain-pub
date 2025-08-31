//! Demo script to showcase the new ProvChain UI
//! This script starts the web server and adds some sample data

use provchain_org::{
    core::blockchain::Blockchain,
    web::server::create_web_server,
    config::Config,
};
use tokio;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting ProvChain UI Demo");

    // Create a new blockchain instance
    let mut blockchain = Blockchain::new();

    // Add some sample data for demonstration
    add_sample_data(&mut blockchain);

    // Create config with port 8080
    let mut config = Config::load_or_default("config.toml");
    config.web.port = 8080;
    
    // Create and start the web server
    let server = create_web_server(blockchain, Some(config)).await?;
    
    info!("Demo data added to blockchain");
    info!("Open your browser and navigate to: http://localhost:8080");
    info!("You can:");
    info!("  - View the dashboard with blockchain statistics");
    info!("  - Browse blocks in the block explorer");
    info!("  - Trace products using batch IDs (try 'BATCH001' or 'BATCH002')");
    info!("  - Execute SPARQL queries on the RDF data");
    info!("  - Add new triples to the blockchain");
    info!("  - Login with any username/password (demo mode)");
    info!("");
    info!("Press Ctrl+C to stop the server");

    // Start the server (this will block until the server is stopped)
    server.start().await?;

    Ok(())
}

fn add_sample_data(blockchain: &mut Blockchain) {
    // Add sample supply chain data
    let sample_triples = vec![
        // Batch 1 - Coffee beans
        ":batch001 tc:product \"Organic Coffee Beans\" .",
        ":batch001 tc:origin \"Farm ABC, Colombia\" .",
        ":batch001 tc:currentLocation \"Warehouse XYZ, USA\" .",
        ":batch001 tc:status \"In Transit\" .",
        ":batch001 tc:batchId \"BATCH001\" .",
        
        // Batch 2 - Cocoa beans
        ":batch002 tc:product \"Fair Trade Cocoa Beans\" .",
        ":batch002 tc:origin \"Farm DEF, Ecuador\" .",
        ":batch002 tc:currentLocation \"Processing Plant, Germany\" .",
        ":batch002 tc:status \"Processing\" .",
        ":batch002 tc:batchId \"BATCH002\" .",
        
        // Environmental data
        ":batch001 tc:environmentalData :env001 .",
        ":env001 tc:temperature \"22.5\" .",
        ":env001 tc:humidity \"65.0\" .",
        ":env001 tc:co2Footprint \"1.2\" .",
        
        ":batch002 tc:environmentalData :env002 .",
        ":env002 tc:temperature \"24.0\" .",
        ":env002 tc:humidity \"70.0\" .",
        ":env002 tc:co2Footprint \"0.8\" .",
        
        // Supply chain events
        ":event001 tc:batch :batch001 .",
        ":event001 tc:actor \"Farmer John\" .",
        ":event001 tc:action \"Harvested\" .",
        ":event001 tc:location \"Farm ABC, Colombia\" .",
        ":event001 tc:timestamp \"2024-01-15T08:00:00Z\" .",
        
        ":event002 tc:batch :batch001 .",
        ":event002 tc:actor \"Transporter ABC\" .",
        ":event002 tc:action \"Shipped\" .",
        ":event002 tc:location \"Port of Cartagena\" .",
        ":event002 tc:timestamp \"2024-01-20T14:30:00Z\" .",
        
        ":event003 tc:batch :batch002 .",
        ":event003 tc:actor \"Farmer Maria\" .",
        ":event003 tc:action \"Harvested\" .",
        ":event003 tc:location \"Farm DEF, Ecuador\" .",
        ":event003 tc:timestamp \"2024-01-18T09:15:00Z\" .",
        
        // Certifications
        ":batch001 tc:certification \"Organic\" .",
        ":batch001 tc:certification \"Fair Trade\" .",
        ":batch002 tc:certification \"Fair Trade\" .",
        ":batch002 tc:certification \"Rainforest Alliance\" .",
        
        // Blockchain metadata
        ":block000 tc:blockHash \"genesis\" .",
        ":block000 tc:timestamp \"2024-01-01T00:00:00Z\" .",
        ":block000 tc:previousHash \"0000000000000000\" .",
    ];

    info!("Adding {} sample triples to blockchain", sample_triples.len());
    
    for triple in sample_triples {
        let _ = blockchain.add_block(triple.to_string());
    }
    
    info!("Sample data added successfully");
}
