//! Web Service Example for OWL2 Reasoner
//!
//! This example demonstrates how to start the web service API.
//! Note: This example requires the "web-service" feature to be enabled.

#[cfg(feature = "web-service")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 OWL2 Reasoner Web Service Example");
    println!("====================================");
    println!("This example would start the web service on port 8080");
    println!("Available endpoints:");
    println!("   GET  /health - Health check");
    println!("   POST /epcis - Upload EPCIS data");
    println!("   POST /reasoning - Perform reasoning operations");
    println!("   POST /analysis - Analyze EPCIS data");
    println!("   GET  /statistics - Get ontology statistics");
    println!();
    println!("To actually start the web service, you would call:");
    println!("   owl2_reasoner::web_service::start_web_service(8080)");
    println!();
    println!("Note: This is a demonstration example. For production use,");
    println!("integrate the web service into your application properly.");

    Ok(())
}

#[cfg(not(feature = "web-service"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("❌ Web service feature not enabled");
    println!("Please run with: cargo run --example web_service_example --features web-service");
    Err("Web service feature not enabled".into())
}
