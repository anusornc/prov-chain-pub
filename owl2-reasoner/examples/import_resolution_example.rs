//! Example demonstrating OWL2 Import Resolution functionality
//!
//! This example shows how to create ontologies with import statements and use
//! the comprehensive import resolution system that supports:
//! - Local file and HTTP-based imports
//! - Circular dependency detection
//! - Configurable caching and timeouts
//! - Multi-format ontology parsing

use owl2_reasoner::{
    entities::*, iri::IRI, ontology::Ontology, ImportResolver, ImportResolverConfig, OwlResult,
};
use std::sync::Arc;
use std::time::Duration;

fn main() -> OwlResult<()> {
    // Initialize logging
    env_logger::init();

    println!("OWL2 Import Resolution Example");
    println!("================================");

    // Example 1: Create an ontology with imports
    create_import_example()?;

    // Example 2: Configure import resolver
    configure_import_resolver()?;

    // Example 3: Handle circular dependencies
    handle_circular_dependencies()?;

    println!("\n✅ All examples completed successfully!");

    Ok(())
}

fn create_import_example() -> OwlResult<()> {
    println!("\n1. Creating Ontology with Import Statements");

    let mut ontology = Ontology::new();
    ontology.set_iri("http://example.org/main-ontology");

    // Add some classes to the main ontology
    let person_class = Class::new(Arc::new(IRI::new("http://example.org/Person")?));
    let student_class = Class::new(Arc::new(IRI::new("http://example.org/Student")?));

    ontology.add_class(person_class)?;
    ontology.add_class(student_class)?;

    // Add import statements (these would be resolved by the import resolver)
    ontology.add_import("http://example.org/foundation-ontology");
    ontology.add_import("http://example.org/vocabulary");

    println!("Created ontology with imports:");
    println!(
        "  - Ontology IRI: {}",
        ontology.iri().map(|iri| iri.as_str()).unwrap_or("unnamed")
    );
    println!("  - Classes: {}", ontology.classes().len());
    println!("  - Imports: {}", ontology.imports().len());

    for import_iri in ontology.imports() {
        println!("    - {}", import_iri);
    }

    Ok(())
}

fn configure_import_resolver() -> OwlResult<()> {
    println!("\n2. Configuring Import Resolver");

    // Create a custom import resolver configuration
    let config = ImportResolverConfig {
        max_depth: 5,
        timeout: Duration::from_secs(15),
        max_cache_size: 50,
        enable_concurrent_resolution: true,
        user_agent: "OWL2-Reasoner-Example/0.1.0".to_string(),
        ..Default::default()
    };

    println!("Creating import resolver with custom configuration:");
    println!("  - Max depth: {}", config.max_depth);
    println!("  - Timeout: {:?}", config.timeout);
    println!("  - Cache size: {}", config.max_cache_size);
    println!(
        "  - Concurrent resolution: {}",
        config.enable_concurrent_resolution
    );

    // Create the import resolver
    let resolver = ImportResolver::with_config(config)?;
    println!("✅ ImportResolver created successfully");

    // Show resolver statistics
    let stats = resolver.stats();
    println!("Initial resolver statistics:");
    println!("  - Imports resolved: {}", stats.imports_resolved);
    println!("  - Cache hits: {}", stats.cache_hits);
    println!("  - Cache misses: {}", stats.cache_misses);
    println!("  - Failed resolutions: {}", stats.failed_resolutions);

    Ok(())
}

fn handle_circular_dependencies() -> OwlResult<()> {
    println!("\n3. Handling Circular Dependencies");

    // Create ontologies that would have circular imports
    let mut ontology_a = Ontology::new();
    ontology_a.set_iri("http://example.org/ontology-a");
    ontology_a.add_import("http://example.org/ontology-b");

    let mut ontology_b = Ontology::new();
    ontology_b.set_iri("http://example.org/ontology-b");
    ontology_b.add_import("http://example.org/ontology-a");

    println!("Created ontologies with circular import structure:");
    println!("  - Ontology A imports: http://example.org/ontology-b");
    println!("  - Ontology B imports: http://example.org/ontology-a");

    // Test circular dependency detection with import resolver
    let config = ImportResolverConfig::default();
    let mut resolver = ImportResolver::with_config(config)?;

    println!("Testing circular dependency detection:");

    // Try to resolve imports for ontology A (should detect circular dependency)
    match resolver.resolve_imports(&mut ontology_a) {
        Ok(_) => {
            println!(
                "  - Import resolution completed (no circular dependency detected in this test)"
            );
        }
        Err(e) => {
            println!("  - Import resolution failed: {}", e);
            if e.to_string().contains("circular") {
                println!("  ✅ Circular dependency correctly detected!");
            }
        }
    }

    // Show final resolver statistics
    let stats = resolver.stats();
    println!("Final resolver statistics:");
    println!("  - Imports resolved: {}", stats.imports_resolved);
    println!(
        "  - Circular dependencies detected: {}",
        stats.circular_dependencies_detected
    );
    println!("  - Failed resolutions: {}", stats.failed_resolutions);

    Ok(())
}
