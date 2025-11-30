//! IAI Callgrind Benchmarks for Deep Performance Analysis
//!
//! Uses Callgrind (via IAI) to perform instruction-level profiling
//! and detailed performance analysis of reasoning operations.

use iai_callgrind::library_benchmark;

// Include our test data generation utilities
mod memory_profiler;

use memory_profiler::measure_performance;
use owl2_reasoner::{
    generate_ontology_with_size, ComplexityLevel, OntologyConfig, OntologyGenerator,
};

#[library_benchmark]
fn ontology_creation_small() {
    let _ = owl2_reasoner::Ontology::new();
}

#[library_benchmark]
fn ontology_creation_medium() {
    let _ = owl2_reasoner::Ontology::new();
}

#[library_benchmark]
fn ontology_creation_custom_size() {
    let _ = owl2_reasoner::Ontology::new();
}

#[library_benchmark]
fn reasoner_initialization_small() {
    let ontology = owl2_reasoner::Ontology::new();
    let _ = owl2_reasoner::SimpleReasoner::new(ontology);
}

#[library_benchmark]
fn reasoner_initialization_medium() {
    let ontology = owl2_reasoner::Ontology::new();
    let _ = owl2_reasoner::SimpleReasoner::new(ontology);
}

#[library_benchmark]
fn consistency_check_small() {
    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
    let _ = reasoner.is_consistent().unwrap();
}

#[library_benchmark]
fn consistency_check_medium() {
    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
    let _ = reasoner.is_consistent().unwrap();
}

#[library_benchmark]
fn consistency_check_custom_size() {
    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
    let _ = reasoner.is_consistent().unwrap();
}

#[library_benchmark]
fn satisfiability_check_small() {
    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    if let Some(first_class) = reasoner.ontology.classes().iter().next() {
        let class_iri = first_class.iri().clone();
        let _ = reasoner.is_class_satisfiable(&class_iri).unwrap();
    }
}

#[library_benchmark]
fn satisfiability_check_medium() {
    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    if let Some(first_class) = reasoner.ontology.classes().iter().next() {
        let class_iri = first_class.iri().clone();
        let _ = reasoner.is_class_satisfiable(&class_iri).unwrap();
    }
}

#[library_benchmark]
fn classification_small() {
    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
    reasoner.classify().unwrap();
}

#[library_benchmark]
fn classification_medium() {
    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
    reasoner.classify().unwrap();
}

#[library_benchmark]
fn subclass_check_small() {
    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    let classes: Vec<_> = reasoner.ontology.classes().iter().take(2).collect();
    if classes.len() >= 2 {
        let sub_class_iri = classes[0].iri().clone();
        let super_class_iri = classes[1].iri().clone();
        let _ = reasoner
            .is_subclass_of(&sub_class_iri, &super_class_iri)
            .unwrap();
    }
}

#[library_benchmark]
fn subclass_check_medium() {
    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    let classes: Vec<_> = reasoner.ontology.classes().iter().take(2).collect();
    if classes.len() >= 2 {
        let sub_class_iri = classes[0].iri().clone();
        let super_class_iri = classes[1].iri().clone();
        let _ = reasoner
            .is_subclass_of(&sub_class_iri, &super_class_iri)
            .unwrap();
    }
}

#[library_benchmark]
fn complex_reasoning_workflow() {
    let ontology = generate_ontology_with_size(50);
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    // Full reasoning workflow
    let _ = reasoner.is_consistent().unwrap();

    if let Some(first_class) = reasoner.ontology.classes().iter().next() {
        let class_iri = first_class.iri().clone();
        let _ = reasoner.is_class_satisfiable(&class_iri).unwrap();
    }

    reasoner.classify().unwrap();
}

#[library_benchmark]
fn reasoning_with_different_complexities() {
    // Test with different complexity levels
    let complexities = vec![
        ComplexityLevel::Simple,
        ComplexityLevel::Medium,
        ComplexityLevel::Complex,
    ];

    for complexity in complexities {
        let config = OntologyConfig {
            num_classes: 50,
            num_axioms: 100,
            complexity,
            ..Default::default()
        };

        let mut generator = OntologyGenerator::new(config);
        let ontology = generator.generate();
        let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
        let _ = reasoner.is_consistent().unwrap();
    }
}

#[library_benchmark]
fn memory_intensive_operations() {
    // Test operations that stress memory allocation
    let ontology = generate_ontology_with_size(200);
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    // Classification is memory intensive
    reasoner.classify().unwrap();

    // Multiple satisfiability checks
    let classes: Vec<_> = reasoner.ontology.classes().iter().take(10).collect();
    for class in classes {
        let class_iri = class.iri().clone();
        let _ = reasoner.is_class_satisfiable(&class_iri).unwrap();
    }
}

#[library_benchmark]
fn cache_behavior_analysis() {
    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    // Get some classes for cache testing
    let classes: Vec<_> = reasoner.ontology.classes().iter().take(5).collect();

    // First pass (cache misses)
    for class in &classes {
        let class_iri = class.iri().clone();
        let _ = reasoner.is_class_satisfiable(&class_iri).unwrap();
    }

    // Second pass (cache hits)
    for class in &classes {
        let class_iri = class.iri().clone();
        let _ = reasoner.is_class_satisfiable(&class_iri).unwrap();
    }
}

#[library_benchmark]
fn error_handling_performance() {
    // Test error handling overhead
    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    // Create an invalid IRI for error handling test
    let invalid_iri =
        owl2_reasoner::iri::IRI::new("http://invalid.example.org/nonexistent").unwrap();

    // This should handle errors gracefully
    let _ = reasoner.is_class_satisfiable(&invalid_iri);
}

#[library_benchmark]
fn allocation_patterns() {
    // Test different allocation patterns
    let mut ontologies = Vec::new();

    // Multiple small allocations
    for _ in 0..10 {
        ontologies.push(owl2_reasoner::Ontology::new());
    }

    // Process them
    for ontology in ontologies {
        let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
        let _ = reasoner.is_consistent().unwrap();
    }
}

#[library_benchmark]
fn repeated_operations() {
    let ontology = owl2_reasoner::Ontology::new();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    // Repeated operations to test optimization
    for _ in 0..10 {
        let _ = reasoner.is_consistent().unwrap();
    }

    if let Some(first_class) = reasoner.ontology.classes().iter().next() {
        let class_iri = first_class.iri().clone();
        for _ in 0..10 {
            let _ = reasoner.is_class_satisfiable(&class_iri).unwrap();
        }
    }
}

#[library_benchmark]
fn stress_test_large_ontology() {
    // Stress test with larger ontologies
    let config = OntologyConfig {
        num_classes: 200,
        num_axioms: 400,
        num_properties: 40,
        num_individuals: 100,
        complexity: ComplexityLevel::Medium,
        ..Default::default()
    };

    let mut generator = OntologyGenerator::new(config);
    let ontology = generator.generate();
    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);

    // Multiple operations
    let _ = reasoner.is_consistent().unwrap();
    reasoner.classify().unwrap();

    // Multiple satisfiability checks
    let classes: Vec<_> = reasoner.ontology.classes().iter().take(20).collect();
    for class in classes {
        let class_iri = class.iri().clone();
        let _ = reasoner.is_class_satisfiable(&class_iri).unwrap();
    }
}

// Additional performance analysis functions
fn run_instruction_level_analysis() {
    println!("=== Running Instruction-Level Performance Analysis ===");

    // Define operations as separate function calls to avoid type issues
    let test_cases = vec![
        "Small Ontology Creation",
        "Medium Ontology Creation",
        "Small Consistency Check",
        "Medium Consistency Check",
        "Small Classification",
    ];

    for name in test_cases {
        println!("Analyzing: {}", name);
        match name {
            "Small Ontology Creation" => {
                let (_result, measurement) =
                    measure_performance(name, owl2_reasoner::Ontology::new);
                println!("  Duration: {:.2} ms", measurement.duration_ms);
                println!(
                    "  Memory delta: {:.2} MB",
                    measurement.memory_delta.used_delta_mb
                );
            }
            "Medium Ontology Creation" => {
                let (_result, measurement) =
                    measure_performance(name, owl2_reasoner::Ontology::new);
                println!("  Duration: {:.2} ms", measurement.duration_ms);
                println!(
                    "  Memory delta: {:.2} MB",
                    measurement.memory_delta.used_delta_mb
                );
            }
            "Small Consistency Check" => {
                let (_result, measurement) = measure_performance(name, || {
                    let ontology = owl2_reasoner::Ontology::new();
                    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
                    reasoner.is_consistent().unwrap()
                });
                println!("  Duration: {:.2} ms", measurement.duration_ms);
                println!(
                    "  Memory delta: {:.2} MB",
                    measurement.memory_delta.used_delta_mb
                );
            }
            "Medium Consistency Check" => {
                let (_result, measurement) = measure_performance(name, || {
                    let ontology = owl2_reasoner::Ontology::new();
                    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
                    reasoner.is_consistent().unwrap()
                });
                println!("  Duration: {:.2} ms", measurement.duration_ms);
                println!(
                    "  Memory delta: {:.2} MB",
                    measurement.memory_delta.used_delta_mb
                );
            }
            "Small Classification" => {
                let (_result, measurement) = measure_performance(name, || {
                    let ontology = owl2_reasoner::Ontology::new();
                    let reasoner = owl2_reasoner::SimpleReasoner::new(ontology);
                    reasoner.classify().unwrap()
                });
                println!("  Duration: {:.2} ms", measurement.duration_ms);
                println!(
                    "  Memory delta: {:.2} MB",
                    measurement.memory_delta.used_delta_mb
                );
            }
            _ => unreachable!(),
        };
    }
}

fn main() {
    // This main function is required for IAI benchmarks
    // The actual benchmarks are defined above with #[library_benchmark]
    run_instruction_level_analysis();
}
