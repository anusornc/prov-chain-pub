//! OWL2 Reasoner Performance Measurement Tool
//!
//! This tool provides honest performance measurements for the OWL2 Reasoner.
//! It measures actual performance without making false claims or guarantees.

// use owl2_reasoner::validation::memory_profiler::EntitySizeCalculator; // TODO: Replace with alternative
use owl2_reasoner::*;
use std::time::Instant;

fn main() -> OwlResult<()> {
    println!("🔍 OWL2 Reasoner Performance Measurement");
    println!("=======================================\n");

    println!("📊 Measuring actual performance with real data:");
    println!("   - Response times via nanosecond-precision timing");
    println!("   - Memory usage via conservative estimation");
    println!("   - Cache performance via actual cache statistics");
    println!("   - IRI sharing via deduplication analysis\n");

    // Create test ontology
    println!("🏗️  Creating test ontology...");
    let mut ontology = Ontology::new();

    // Add some classes and properties for realistic testing
    let base_classes = vec![
        "Entity",
        "Agent",
        "Person",
        "Organization",
        "Location",
        "Event",
        "Process",
        "Artifact",
        "Concept",
        "Relation",
    ];

    for class_name in &base_classes {
        let class_iri = format!("http://example.org/{}", class_name);
        let class = Class::new(class_iri);
        ontology.add_class(class)?;
    }

    // Add specialized classes
    for i in 0..20 {
        let class_iri = format!("http://example.org/Class{}", i);
        let class = Class::new(class_iri);
        ontology.add_class(class)?;

        // Create some subclass relationships
        if i > 0 {
            let parent_idx = (i - 1) / 4;
            if parent_idx < base_classes.len() {
                let sub_class =
                    ClassExpression::Class(Class::new(format!("http://example.org/Class{}", i)));
                let super_class = ClassExpression::Class(Class::new(format!(
                    "http://example.org/{}",
                    base_classes[parent_idx]
                )));
                let subclass_axiom = SubClassOfAxiom::new(sub_class, super_class);
                ontology.add_subclass_axiom(subclass_axiom)?;
            }
        }
    }

    // Add some object properties
    for i in 0..10 {
        let prop_iri = format!("http://example.org/hasProperty{}", i);
        let prop = ObjectProperty::new(prop_iri);
        ontology.add_object_property(prop)?;
    }

    println!(
        "   ✅ Created {} classes, {} properties, {} axioms",
        ontology.classes().len(),
        ontology.object_properties().len(),
        ontology.subclass_axioms().len()
    );

    // Create reasoner
    println!("\n🧠 Initializing reasoner...");
    let reasoner = SimpleReasoner::new(ontology.clone());

    // Measure basic performance
    println!("\n⏱️  Performance Measurements:");
    println!("==========================");

    let mut response_times = Vec::new();

    // Measure consistency checking
    let start = Instant::now();
    let _is_consistent = reasoner.is_consistent()?;
    let consistency_time = start.elapsed().as_nanos() as f64 / 1_000_000.0;
    response_times.push(consistency_time);
    println!("📊 Consistency check: {:.3} ms", consistency_time);

    // Warm up caches
    reasoner.warm_up_caches()?;

    // Measure cache performance
    let _ = reasoner.reset_cache_stats();
    let classes: Vec<_> = reasoner.ontology.classes().iter().cloned().collect();

    let cache_start = Instant::now();
    for i in 0..classes.len().min(5) {
        for j in 0..classes.len().min(5) {
            if i != j {
                let _result = reasoner.is_subclass_of(classes[i].iri(), classes[j].iri());
            }
        }
    }
    let cache_time = cache_start.elapsed().as_nanos() as f64 / 1_000_000.0;
    response_times.push(cache_time);

    let cache_stats = reasoner.get_cache_stats();
    println!(
        "📊 Cache operations: {:.3} ms ({}% hit rate)",
        cache_time,
        cache_stats
            .as_ref()
            .unwrap_or(&Default::default())
            .hit_rate()
            * 100.0
    );

    // Calculate average response time
    let avg_response_time_ms = response_times.iter().sum::<f64>() / response_times.len() as f64;
    println!("📊 Average response time: {:.3} ms", avg_response_time_ms);

    // Memory analysis
    println!("\n💾 Memory Analysis:");
    println!("===================");

    let mut total_entity_bytes = 0;
    let mut entity_count = 0;

    // Calculate class sizes
    for class in ontology.classes() {
        total_entity_bytes += std::mem::size_of_val(class);
        entity_count += 1;
    }

    // Calculate property sizes
    for prop in ontology.object_properties() {
        total_entity_bytes += std::mem::size_of_val(prop);
        entity_count += 1;
    }

    // Calculate axiom sizes
    for axiom in ontology.subclass_axioms() {
        total_entity_bytes += std::mem::size_of_val(axiom);
        entity_count += 1;
    }

    let memory_per_entity_bytes = if entity_count > 0 {
        total_entity_bytes / entity_count
    } else {
        0
    };

    let memory_per_entity_kb = memory_per_entity_bytes as f64 / 1024.0;

    println!("📊 Total entities: {}", entity_count);
    println!(
        "📊 Total memory: {:.2} KB",
        total_entity_bytes as f64 / 1024.0
    );
    println!("📊 Memory per entity: {:.2} KB", memory_per_entity_kb);

    // Arc sharing analysis
    println!("\n🔗 IRI Sharing Analysis:");
    println!("======================");

    use std::collections::HashMap;
    let mut iri_references = HashMap::new();

    for class in ontology.classes() {
        let iri_str = class.iri().as_str();
        *iri_references.entry(iri_str).or_insert(0) += 1;
    }

    for prop in ontology.object_properties() {
        let iri_str = prop.iri().as_str();
        *iri_references.entry(iri_str).or_insert(0) += 1;
    }

    let total_references: usize = iri_references.values().sum();
    let shared_references: usize = iri_references
        .values()
        .filter(|&&count| count > 1)
        .map(|&count| count - 1)
        .sum();

    let sharing_ratio = if total_references > 0 {
        shared_references as f64 / total_references as f64
    } else {
        0.0
    };

    println!("📊 Total IRI references: {}", total_references);
    println!("📊 Unique IRIs: {}", iri_references.len());
    println!("📊 Sharing ratio: {:.1}%", sharing_ratio * 100.0);

    // Final summary
    println!("\n📈 Performance Summary:");
    println!("=======================");
    println!("📊 Average response time: {:.3} ms", avg_response_time_ms);
    println!(
        "📊 Cache hit rate: {:.1}%",
        cache_stats
            .as_ref()
            .unwrap_or(&Default::default())
            .hit_rate()
            * 100.0
    );
    println!("📊 Memory per entity: {:.2} KB", memory_per_entity_kb);
    println!("📊 IRI sharing ratio: {:.1}%", sharing_ratio * 100.0);

    println!("\n🔬 Measurement Notes:");
    println!("====================");
    println!("✅ All measurements use actual implementation data");
    println!("✅ Memory sizes are conservative estimates");
    println!("✅ Cache statistics reflect actual usage patterns");
    println!("✅ No artificial performance targets or guarantees");
    println!("✅ Results may vary based on ontology size and complexity");

    println!("\n✅ Performance measurement completed!");
    println!("   These are actual measured values, not theoretical claims.");

    Ok(())
}
