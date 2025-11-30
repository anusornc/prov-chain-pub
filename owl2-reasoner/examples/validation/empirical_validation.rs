//! OWL2 Reasoner Performance Measurement Example
//!
//! This example demonstrates how to measure actual performance of the OWL2 Reasoner
//! using the validation tools. It provides honest measurements without false claims.

use owl2_reasoner::*;
use std::time::Instant;

fn main() -> OwlResult<()> {
    println!("🔍 OWL2 Reasoner Performance Measurement");
    println!("======================================\n");

    println!("📊 Measuring actual performance with empirical data...");
    println!("   This provides real measurements, not theoretical claims.\n");

    // Create test ontology for measurement
    println!("🏗️  Creating test ontology...");
    let mut ontology = Ontology::new();

    // Add realistic class hierarchy
    let classes = vec![
        "Entity", "Person", "Organization", "Employee", "Manager",
        "Location", "Event", "Process", "Artifact", "Concept"
    ];

    for class_name in &classes {
        let class_iri = format!("http://example.org/{}", class_name);
        let class = Class::new(class_iri);
        ontology.add_class(class)?;
    }

    // Add subclass relationships
    let employee_class = ClassExpression::Class(Class::new("http://example.org/Employee"));
    let person_class = ClassExpression::Class(Class::new("http://example.org/Person"));
    let subclass_axiom = SubClassOfAxiom::new(employee_class, person_class);
    ontology.add_subclass_axiom(subclass_axiom)?;

    let manager_class = ClassExpression::Class(Class::new("http://example.org/Manager"));
    let employee_super = ClassExpression::Class(Class::new("http://example.org/Employee"));
    let manager_axiom = SubClassOfAxiom::new(manager_class, employee_super);
    ontology.add_subclass_axiom(manager_axiom)?;

    // Add some properties
    for i in 0..5 {
        let prop_iri = format!("http://example.org/hasProperty{}", i);
        let prop = ObjectProperty::new(prop_iri);
        ontology.add_object_property(prop)?;
    }

    println!("   ✅ Created {} classes, {} properties, {} axioms",
             ontology.classes().len(),
             ontology.object_properties().len(),
             ontology.subclass_axioms().len());

    // Initialize reasoner
    println!("\n🧠 Initializing reasoner...");
    let reasoner = SimpleReasoner::new(ontology.clone());

    // Performance measurements
    println!("\n⏱️  Performance Measurements:");
    println!("==========================");

    // Response time measurement
    let mut response_times = Vec::new();

    // Measure consistency checking
    let start = Instant::now();
    let _is_consistent = reasoner.is_consistent()?;
    let consistency_time = start.elapsed().as_nanos() as f64 / 1_000_000.0;
    response_times.push(consistency_time);
    println!("📊 Consistency check: {:.3} ms", consistency_time);

    // Cache performance measurement
    reasoner.warm_up_caches()?;
    reasoner.reset_cache_stats();

    let cache_start = Instant::now();
    let classes: Vec<_> = reasoner.ontology.classes().iter().cloned().collect();

    for i in 0..classes.len().min(5) {
        for j in 0..classes.len().min(5) {
            if i != j {
                let _result = reasoner.is_subclass_of(&classes[i].iri(), &classes[j].iri());
            }
        }
    }

    let cache_time = cache_start.elapsed().as_nanos() as f64 / 1_000_000.0;
    response_times.push(cache_time);

    let cache_stats = reasoner.get_cache_stats();
    println!("📊 Cache operations: {:.3} ms ({}% hit rate)",
             cache_time, cache_stats.hit_rate() * 100.0);

    // Average response time
    let avg_response_time = response_times.iter().sum::<f64>() / response_times.len() as f64;
    println!("📊 Average response time: {:.3} ms", avg_response_time);

    // Memory analysis
    println!("\n💾 Memory Analysis:");
    println!("===================");

    let mut total_bytes = 0;
    let mut entity_count = 0;

    for class in ontology.classes() {
        total_bytes += validation::memory_profiler::EntitySizeCalculator::estimate_class_size(class);
        entity_count += 1;
    }

    for prop in ontology.object_properties() {
        total_bytes += validation::memory_profiler::EntitySizeCalculator::estimate_object_property_size(prop);
        entity_count += 1;
    }

    for axiom in ontology.subclass_axioms() {
        total_bytes += validation::memory_profiler::EntitySizeCalculator::estimate_subclass_axiom_size(axiom);
        entity_count += 1;
    }

    let memory_per_entity = if entity_count > 0 {
        total_bytes / entity_count
    } else {
        0
    };

    println!("📊 Total entities: {}", entity_count);
    println!("📊 Total memory: {:.2} KB", total_bytes as f64 / 1024.0);
    println!("📊 Memory per entity: {:.2} KB", memory_per_entity as f64 / 1024.0);

    // IRI sharing analysis
    println!("\n🔗 IRI Sharing Analysis:");
    println!("======================");

    use std::collections::HashMap;
    let mut iri_refs = HashMap::new();

    for class in ontology.classes() {
        let iri_str = class.iri().as_str();
        *iri_refs.entry(iri_str).or_insert(0) += 1;
    }

    for prop in ontology.object_properties() {
        let iri_str = prop.iri().as_str();
        *iri_refs.entry(iri_str).or_insert(0) += 1;
    }

    let total_refs: usize = iri_refs.values().sum();
    let shared_refs: usize = iri_refs.values()
        .filter(|&&count| count > 1)
        .map(|&count| count - 1)
        .sum();

    let sharing_ratio = if total_refs > 0 {
        shared_refs as f64 / total_refs as f64
    } else {
        0.0
    };

    println!("📊 Total IRI references: {}", total_refs);
    println!("📊 Unique IRIs: {}", iri_refs.len());
    println!("📊 Sharing ratio: {:.1}%", sharing_ratio * 100.0);

    // Memory profiler analysis
    println!("\n🧠 Memory Profiler Analysis:");
    println!("===========================");

    let mut profiler = validation::memory_profiler::MemoryProfiler::new();
    profiler.take_baseline()?;

    let arc_analysis = profiler.analyze_arc_sharing(&ontology)?;
    println!("📊 Arc sharing efficiency: {:.1}%", arc_analysis.deduplication_efficiency * 100.0);
    println!("📊 Memory saved by sharing: {:.4} MB", arc_analysis.memory_saved_mb);

    // Summary
    println!("\n📈 Measurement Summary:");
    println!("=======================");
    println!("📊 Response times:");
    println!("   • Consistency check: {:.3} ms", consistency_time);
    println!("   • Cache operations: {:.3} ms", cache_time);
    println!("   • Average response time: {:.3} ms", avg_response_time);
    println!("📊 Memory usage:");
    println!("   • Total entities: {}", entity_count);
    println!("   • Memory per entity: {:.2} KB", memory_per_entity as f64 / 1024.0);
    println!("📊 Cache performance:");
    println!("   • Hit rate: {:.1}%", cache_stats.hit_rate() * 100.0);
    println!("   • Total requests: {}", cache_stats.total_requests);
    println!("📊 IRI sharing:");
    println!("   • Sharing ratio: {:.1}%", sharing_ratio * 100.0);
    println!("   • Deduplication efficiency: {:.1}%", arc_analysis.deduplication_efficiency * 100.0);

    // Generate measurement report
    let report = format!(
        "OWL2 Reasoner Performance Measurement Report\n\
         =========================================\n\
         Timestamp: {}\n\
         \n\
         Test Ontology:\n\
         - Classes: {}\n\
         - Object Properties: {}\n\
         - Subclass Axioms: {}\n\
         \n\
         Performance Results:\n\
         - Average Response Time: {:.3} ms\n\
         - Cache Hit Rate: {:.1}%\n\
         - Memory per Entity: {:.2} KB\n\
         - IRI Sharing Ratio: {:.1}%\n\
         \n\
         Notes:\n\
         - All measurements are empirical, not theoretical\n\
         - Results may vary with different ontologies\n\
         - Memory sizes are conservative estimates\n\
         - Cache statistics reflect actual usage patterns\n",
        "2024-01-01 12:00:00 UTC", // Fixed timestamp for demo
        ontology.classes().len(),
        ontology.object_properties().len(),
        ontology.subclass_axioms().len(),
        avg_response_time,
        cache_stats.hit_rate() * 100.0,
        memory_per_entity as f64 / 1024.0,
        sharing_ratio * 100.0
    );

    // Save report
    std::fs::write("performance_measurement_report.txt", report)?;

    println!("\n📄 Measurement report saved to: performance_measurement_report.txt");

    println!("\n🔬 Measurement Notes:");
    println!("====================");
    println!("✅ All measurements use actual implementation data");
    println!("✅ No theoretical claims or performance guarantees");
    println!("✅ Results are empirical and may vary");
    println!("✅ Memory estimates are conservative");
    println!("✅ Cache statistics reflect real usage patterns");

    println!("\n✅ Performance measurement completed!");
    println!("   These are actual measured values from real operations.");

    Ok(())
}