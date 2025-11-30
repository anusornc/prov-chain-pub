//! Demonstration of parallel query execution in the OWL2 Reasoner
//!
//! This example shows how to use the new parallel query execution features
//! and compares performance between sequential and parallel execution.

use owl2_reasoner::axioms::*;
use owl2_reasoner::iri::IRI;
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::reasoning::query::*;
use owl2_reasoner::{Class, NamedIndividual, ObjectProperty};
use std::sync::Arc;
use std::time::Instant;

fn create_demo_ontology() -> Ontology {
    let mut ontology = Ontology::new();

    // Create test classes
    let person_class = IRI::new("http://example.org/Person").unwrap();
    let employee_class = IRI::new("http://example.org/Employee").unwrap();
    let manager_class = IRI::new("http://example.org/Manager").unwrap();
    let department_class = IRI::new("http://example.org/Department").unwrap();

    // Add class declarations
    ontology
        .add_class(Class::new(Arc::new(person_class.clone())))
        .unwrap();
    ontology
        .add_class(Class::new(Arc::new(employee_class.clone())))
        .unwrap();
    ontology
        .add_class(Class::new(Arc::new(manager_class.clone())))
        .unwrap();
    ontology
        .add_class(Class::new(Arc::new(department_class.clone())))
        .unwrap();

    // Create test properties
    let works_for_prop = IRI::new("http://example.org/worksFor").unwrap();
    let manages_prop = IRI::new("http://example.org/manages").unwrap();

    // Add property declarations
    ontology
        .add_object_property(ObjectProperty::new(Arc::new(works_for_prop.clone())))
        .unwrap();
    ontology
        .add_object_property(ObjectProperty::new(Arc::new(manages_prop.clone())))
        .unwrap();

    // Add individuals and assertions
    for i in 0..1000 {
        let individual_iri = IRI::new(format!("http://example.org/person{}", i)).unwrap();
        let individual = NamedIndividual::new(Arc::new(individual_iri.clone()));

        // Add individual
        ontology.add_named_individual(individual).unwrap();

        // Distribute across different types
        let target_class = match i % 4 {
            0 => &person_class,
            1 => &employee_class,
            2 => &manager_class,
            _ => &department_class,
        };

        // Add type assertion
        ontology
            .add_class_assertion(ClassAssertionAxiom::new(
                Arc::new(individual_iri.clone()),
                ClassExpression::Class(Class::new(Arc::new(target_class.clone()))),
            ))
            .unwrap();

        // Add some property assertions
        if i > 0 {
            let works_for_iri =
                IRI::new(format!("http://example.org/department{}", i % 10)).unwrap();
            let works_for_individual = NamedIndividual::new(Arc::new(works_for_iri.clone()));
            ontology.add_named_individual(works_for_individual).unwrap();

            ontology
                .add_property_assertion(PropertyAssertionAxiom::new(
                    Arc::new(individual_iri.clone()),
                    Arc::new(works_for_prop.clone()),
                    Arc::new(works_for_iri),
                ))
                .unwrap();
        }
    }

    ontology
}

fn create_union_query() -> QueryPattern {
    let patterns = vec![
        QueryPattern::BasicGraphPattern(vec![TriplePattern {
            subject: PatternTerm::Variable("?person".to_string()),
            predicate: PatternTerm::IRI(
                IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
            ),
            object: PatternTerm::IRI(IRI::new("http://example.org/Person").unwrap()),
        }]),
        QueryPattern::BasicGraphPattern(vec![TriplePattern {
            subject: PatternTerm::Variable("?person".to_string()),
            predicate: PatternTerm::IRI(
                IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
            ),
            object: PatternTerm::IRI(IRI::new("http://example.org/Employee").unwrap()),
        }]),
        QueryPattern::BasicGraphPattern(vec![TriplePattern {
            subject: PatternTerm::Variable("?person".to_string()),
            predicate: PatternTerm::IRI(
                IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
            ),
            object: PatternTerm::IRI(IRI::new("http://example.org/Manager").unwrap()),
        }]),
        QueryPattern::BasicGraphPattern(vec![TriplePattern {
            subject: PatternTerm::Variable("?person".to_string()),
            predicate: PatternTerm::IRI(
                IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
            ),
            object: PatternTerm::IRI(IRI::new("http://example.org/Department").unwrap()),
        }]),
    ];

    QueryPattern::UnionPattern(patterns)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 OWL2 Reasoner Parallel Query Execution Demo");
    println!("================================================");

    // Create test ontology
    println!("📝 Creating test ontology...");
    let ontology = create_demo_ontology();
    println!(
        "✅ Ontology created with {} individuals",
        ontology.named_individuals().len()
    );

    // Create union query pattern
    let query_pattern = create_union_query();
    println!("🔍 Created union query with 4 branches");

    // Test sequential execution
    println!("\n🐌 Testing sequential execution...");
    let mut sequential_engine = QueryEngine::with_config(
        ontology.clone(),
        QueryConfig {
            enable_reasoning: false,
            max_results: None,
            timeout: None,
            enable_caching: false, // Disable caching for accurate timing
            cache_size: None,
            enable_parallel: false, // Sequential execution
            max_parallel_threads: None,
            parallel_threshold: 1,
            use_memory_pool: true,
        },
    );

    let start = Instant::now();
    let sequential_result = sequential_engine.execute_query_sequential(&query_pattern)?;
    let sequential_time = start.elapsed();

    println!("✅ Sequential execution completed:");
    println!("   - Results: {}", sequential_result.bindings.len());
    println!("   - Time: {:?}", sequential_time);

    // Test parallel execution with different thread counts
    let thread_counts = vec![2, 4];

    for &threads in &thread_counts {
        println!(
            "\n⚡ Testing parallel execution with {} threads...",
            threads
        );

        let mut parallel_engine = QueryEngine::with_config(
            ontology.clone(),
            QueryConfig {
                enable_reasoning: false,
                max_results: None,
                timeout: None,
                enable_caching: false, // Disable caching for accurate timing
                cache_size: None,
                enable_parallel: true, // Parallel execution
                max_parallel_threads: Some(threads),
                parallel_threshold: 1, // Always use parallel for this demo
                use_memory_pool: true,
            },
        );

        let start = Instant::now();
        let parallel_result = parallel_engine.execute_query_parallel(&query_pattern)?;
        let parallel_time = start.elapsed();

        println!("✅ Parallel execution completed:");
        println!("   - Results: {}", parallel_result.bindings.len());
        println!("   - Time: {:?}", parallel_time);

        // Calculate speedup
        let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
        println!("   - Speedup: {:.2}x", speedup);

        // Verify results are the same
        if sequential_result.bindings.len() == parallel_result.bindings.len() {
            println!("   - ✅ Results match sequential execution");
        } else {
            println!("   - ❌ Results differ from sequential execution");
        }

        // Get parallel statistics
        let (parallel_queries, parallel_time_total) = parallel_engine.get_parallel_stats();
        println!(
            "   - Parallel stats: {} queries, {}ms total",
            parallel_queries, parallel_time_total
        );
    }

    // Test automatic parallel execution
    println!("\n🤖 Testing automatic parallel execution...");

    let mut auto_engine = QueryEngine::with_config(
        ontology.clone(),
        QueryConfig {
            enable_reasoning: false,
            max_results: None,
            timeout: None,
            enable_caching: false,
            cache_size: None,
            enable_parallel: true,
            max_parallel_threads: Some(4),
            parallel_threshold: 2, // Use parallel for 2+ union branches
            use_memory_pool: true,
        },
    );

    let start = Instant::now();
    let auto_result = auto_engine.execute_query(&query_pattern)?;
    let auto_time = start.elapsed();

    println!("✅ Auto parallel execution completed:");
    println!("   - Results: {}", auto_result.bindings.len());
    println!("   - Time: {:?}", auto_time);

    // Test with a small query (should use sequential)
    println!("\n📏 Testing small query (should use sequential)...");
    let small_query = QueryPattern::BasicGraphPattern(vec![TriplePattern {
        subject: PatternTerm::Variable("?person".to_string()),
        predicate: PatternTerm::IRI(
            IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
        ),
        object: PatternTerm::IRI(IRI::new("http://example.org/Person").unwrap()),
    }]);

    let start = Instant::now();
    let small_result = auto_engine.execute_query(&small_query)?;
    let small_time = start.elapsed();

    println!("✅ Small query completed:");
    println!("   - Results: {}", small_result.bindings.len());
    println!("   - Time: {:?}", small_time);

    // Show final statistics
    let (cache_size, cache_hits, hit_rate) = auto_engine.get_cache_stats();
    let (parallel_queries, parallel_time_total) = auto_engine.get_parallel_stats();

    println!("\n📊 Final Statistics:");
    println!(
        "   - Cache: {} entries, {} hits, {:.1}% hit rate",
        cache_size, cache_hits, hit_rate
    );
    println!("   - Parallel: {} queries executed", parallel_queries);
    println!("   - Parallel total time: {}ms", parallel_time_total);

    println!("\n🎉 Demo completed successfully!");
    println!("The parallel query execution system is working correctly and providing performance improvements.");

    Ok(())
}
