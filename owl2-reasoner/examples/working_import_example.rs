//! Working example demonstrating actual import resolution with local files
//!
//! This example shows how the import resolver works with real ontology files
//! and demonstrates caching, error handling, and statistics tracking.

use owl2_reasoner::{
    ontology::Ontology, ImportResolver, ImportResolverConfig, OwlResult, ParserFactory,
};
use std::time::Duration;

fn main() -> OwlResult<()> {
    // Initialize logging
    env_logger::init();

    println!("🔧 OWL2 Import Resolution - Working Example");
    println!("==========================================");

    // Get the path to the test ontology files
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let test_ontologies_dir = current_dir.join("examples/test_ontologies");

    println!("Test ontology directory: {}", test_ontologies_dir.display());

    // Test 1: Configure import resolver with custom base directories
    configure_import_resolver(&test_ontologies_dir)?;

    // Test 2: Parse main ontology with imports
    parse_main_ontology_with_imports(&test_ontologies_dir)?;

    // Test 3: Test caching behavior
    test_caching_behavior(&test_ontologies_dir)?;

    // Test 4: Test error handling
    test_error_handling()?;

    println!("\n✅ All working examples completed successfully!");

    Ok(())
}

fn configure_import_resolver(_test_ontologies_dir: &std::path::Path) -> OwlResult<()> {
    println!("\n1. 📋 Configuring Import Resolver");

    // Create a custom import resolver configuration
    let config = ImportResolverConfig {
        max_depth: 5,
        timeout: Duration::from_secs(10),
        max_cache_size: 100,
        cache_ttl: Duration::from_secs(3600),
        enable_concurrent_resolution: false, // Disabled for simplicity
        max_concurrent_resolutions: 1,
        follow_redirects: true,
        max_redirects: 3,
        user_agent: "OWL2-Reasoner-Example/1.0.0".to_string(),
    };

    println!("✅ Import resolver configuration created:");
    println!("   - Max depth: {}", config.max_depth);
    println!("   - Timeout: {:?}", config.timeout);
    println!("   - Cache size: {}", config.max_cache_size);
    println!("   - Cache TTL: {:?}", config.cache_ttl);

    let resolver = ImportResolver::with_config(config)?;
    println!("✅ ImportResolver created successfully");

    // Show resolver statistics
    let stats = resolver.stats();
    println!("📊 Initial resolver statistics:");
    println!("   - Imports resolved: {}", stats.imports_resolved);
    println!("   - Cache hits: {}", stats.cache_hits);
    println!("   - Cache misses: {}", stats.cache_misses);
    println!("   - Failed resolutions: {}", stats.failed_resolutions);

    Ok(())
}

fn parse_main_ontology_with_imports(test_ontologies_dir: &std::path::Path) -> OwlResult<()> {
    println!("\n2. 📄 Parsing Main Ontology with Imports");

    // Create import resolver with base directory pointing to test ontologies
    let mut resolver = ImportResolver::new()?;

    // Add base directory for local file resolution
    let mut file_source = owl2_reasoner::parser::FileSystemImportSource::new();
    file_source.add_base_directory(test_ontologies_dir);

    resolver.add_source(Box::new(file_source));

    // Parse the main ontology
    let main_ontology_path = test_ontologies_dir.join("main_ontology.ttl");
    println!(
        "📂 Parsing main ontology from: {}",
        main_ontology_path.display()
    );

    let parser = ParserFactory::for_file_extension("ttl").ok_or_else(|| {
        owl2_reasoner::OwlError::ParseError("Failed to create Turtle parser".to_string())
    })?;

    let mut ontology = parser.parse_file(&main_ontology_path)?;
    println!("✅ Main ontology parsed successfully");
    println!("   - Ontology IRI: {:?}", ontology.iri());
    println!("   - Classes: {}", ontology.classes().len());
    println!(
        "   - Object properties: {}",
        ontology.object_properties().len()
    );
    println!("   - Data properties: {}", ontology.data_properties().len());
    println!(
        "   - Named individuals: {}",
        ontology.named_individuals().len()
    );
    println!("   - Imports: {}", ontology.imports().len());

    // Resolve imports
    println!("\n🔗 Resolving imports...");
    let _initial_stats = resolver.stats();

    match resolver.resolve_imports(&mut ontology) {
        Ok(()) => {
            println!("✅ Import resolution completed successfully");

            let final_stats = resolver.stats();
            println!("📊 Import resolution statistics:");
            println!("   - Imports resolved: {}", final_stats.imports_resolved);
            println!("   - Cache hits: {}", final_stats.cache_hits);
            println!("   - Cache misses: {}", final_stats.cache_misses);
            println!(
                "   - Failed resolutions: {}",
                final_stats.failed_resolutions
            );
            println!(
                "   - Total resolution time: {:?}",
                final_stats.total_resolution_time
            );

            // Show merged ontology statistics
            println!("\n📈 Merged ontology statistics:");
            println!("   - Classes: {}", ontology.classes().len());
            println!(
                "   - Object properties: {}",
                ontology.object_properties().len()
            );
            println!("   - Data properties: {}", ontology.data_properties().len());
            println!(
                "   - Named individuals: {}",
                ontology.named_individuals().len()
            );
            println!("   - Axioms: {}", ontology.axiom_count());
        }
        Err(e) => {
            println!("⚠️ Import resolution partially failed: {}", e);

            let final_stats = resolver.stats();
            println!("📊 Partial resolution statistics:");
            println!("   - Imports resolved: {}", final_stats.imports_resolved);
            println!(
                "   - Failed resolutions: {}",
                final_stats.failed_resolutions
            );
        }
    }

    Ok(())
}

fn test_caching_behavior(test_ontologies_dir: &std::path::Path) -> OwlResult<()> {
    println!("\n3. 💾 Testing Caching Behavior");

    let mut resolver = ImportResolver::new()?;

    // Add base directory
    let mut file_source = owl2_reasoner::parser::FileSystemImportSource::new();
    file_source.add_base_directory(test_ontologies_dir);
    resolver.add_source(Box::new(file_source));

    // Create test ontology that imports foundation
    let mut ontology1 = Ontology::new();
    ontology1.set_iri("http://example.org/test1");
    ontology1.add_import("http://example.org/foundation");

    // Parse foundation ontology and add to cache
    let foundation_path = test_ontologies_dir.join("foundation.ttl");
    let parser = ParserFactory::for_file_extension("ttl").unwrap();
    let _foundation_ontology = parser.parse_file(&foundation_path)?;

    // Manually add to cache by resolving it
    println!("📂 First resolution (cache miss)...");
    let _stats_before = resolver.stats();
    match resolver.resolve_imports(&mut ontology1) {
        Ok(()) => println!("✅ First import resolution successful"),
        Err(e) => println!("⚠️ First import resolution failed: {}", e),
    }
    let stats_after_first = resolver.stats();

    println!("📊 First resolution stats:");
    println!("   - Cache misses: {}", stats_after_first.cache_misses);
    println!("   - Cache hits: {}", stats_after_first.cache_hits);

    // Create second ontology with same import (should be cache hit)
    let mut ontology2 = Ontology::new();
    ontology2.set_iri("http://example.org/test2");
    ontology2.add_import("http://example.org/foundation");

    println!("\n📂 Second resolution (should be cache hit)...");
    match resolver.resolve_imports(&mut ontology2) {
        Ok(()) => println!("✅ Second import resolution successful"),
        Err(e) => println!("⚠️ Second import resolution failed: {}", e),
    }
    let stats_after_second = resolver.stats();

    println!("📊 Second resolution stats:");
    println!(
        "   - Additional cache misses: {}",
        stats_after_second.cache_misses - stats_after_first.cache_misses
    );
    println!("   - Cache hits: {}", stats_after_second.cache_hits);

    // Show cache statistics
    let cache_stats = resolver.cache_stats();
    println!("\n💾 Cache statistics:");
    println!("   - Entries: {}", cache_stats.entries);
    println!("   - Total size: {} bytes", cache_stats.total_size);
    println!("   - Max size: {} bytes", cache_stats.max_size);

    Ok(())
}

fn test_error_handling() -> OwlResult<()> {
    println!("\n4. ❌ Testing Error Handling");

    let mut resolver = ImportResolver::new()?;

    // Test with non-existent import
    let mut ontology = Ontology::new();
    ontology.set_iri("http://example.org/error-test");
    ontology.add_import("http://example.org/non-existent-ontology");

    println!("📂 Testing resolution of non-existent import...");
    match resolver.resolve_imports(&mut ontology) {
        Ok(()) => println!("⚠️ Unexpected success resolving non-existent import"),
        Err(e) => {
            println!("✅ Expected error caught: {}", e);
            if e.to_string().contains("Import resolution error") {
                println!("✅ Correct error type detected");
            }
        }
    }

    // Test with circular import
    let mut circular_ontology = Ontology::new();
    circular_ontology.set_iri("http://example.org/circular-test");
    circular_ontology.add_import("http://example.org/circular-test");

    println!("\n🔄 Testing circular import detection...");
    match resolver.resolve_imports(&mut circular_ontology) {
        Ok(()) => println!("⚠️ Circular import not detected (may be expected behavior)"),
        Err(e) => {
            if e.to_string().contains("circular") {
                println!("✅ Circular import correctly detected: {}", e);
            } else {
                println!("⚠️ Different error occurred: {}", e);
            }
        }
    }

    // Show final statistics
    let final_stats = resolver.stats();
    println!("\n📊 Final resolver statistics:");
    println!("   - Imports resolved: {}", final_stats.imports_resolved);
    println!("   - Cache hits: {}", final_stats.cache_hits);
    println!("   - Cache misses: {}", final_stats.cache_misses);
    println!(
        "   - Failed resolutions: {}",
        final_stats.failed_resolutions
    );
    println!(
        "   - Circular dependencies detected: {}",
        final_stats.circular_dependencies_detected
    );

    Ok(())
}
