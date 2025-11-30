//! Profile Validation Performance Benchmarks
//!
//! Comprehensive benchmarks for profile validation performance,
//! measuring the improvements achieved through optimization work.
//! Includes timing comparisons, memory usage analysis, and validation throughput.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use owl2_reasoner::parser::owl_functional::OwlFunctionalSyntaxParser;
use owl2_reasoner::parser::OntologyParser;
use owl2_reasoner::profiles::{Owl2Profile, Owl2ProfileValidator, ProfileValidator};
use std::sync::Arc;

/// Benchmark data generators for different profiles
mod benchmark_data {

    /// Generate EL profile compliant ontology
    pub fn generate_el_ontology(size: usize) -> String {
        let mut ontology = String::new();
        ontology.push_str("Prefix(:=<http://example.org/el#>)\n");
        ontology.push_str("Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n");
        ontology.push_str("Ontology(<http://example.org/el>)\n");

        // Generate simple class hierarchy
        for i in 0..size {
            if i == 0 {
                ontology.push_str(&format!("Declaration(Class(:BaseClass{}))\n", i));
            } else {
                ontology.push_str(&format!("Declaration(Class(:Class{}))\n", i));
                ontology.push_str(&format!("SubClassOf(:Class{} :BaseClass{})\n", i, i % 10));
            }
        }

        // Generate existential restrictions (EL profile allows)
        for i in 0..(size / 2) {
            ontology.push_str(&format!("Declaration(ObjectProperty(:hasProp{}))\n", i));
            ontology.push_str(&format!(
                "SubClassOf(:Class{} ObjectSomeValuesFrom(:hasProp{} :BaseClass{}))\n",
                i,
                i,
                i % 10
            ));
        }

        ontology.push(')');
        ontology
    }

    /// Generate QL profile compliant ontology
    pub fn generate_ql_ontology(size: usize) -> String {
        let mut ontology = String::new();
        ontology.push_str("Prefix(:=<http://example.org/ql#>)\n");
        ontology.push_str("Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n");
        ontology.push_str("Ontology(<http://example.org/ql>)\n");

        // Generate classes and properties (QL profile restrictions)
        for i in 0..size {
            ontology.push_str(&format!("Declaration(Class(:QLClass{}))\n", i));
            if i < size / 2 {
                ontology.push_str(&format!("Declaration(ObjectProperty(:qlProp{}))\n", i));
            }
        }

        // Generate simple subclass axioms (QL profile allows)
        for i in 1..size {
            ontology.push_str(&format!("SubClassOf(:QLClass{} :QLClass{})\n", i, i - 1));
        }

        // Generate property domains and ranges
        for i in 0..(size / 2) {
            ontology.push_str(&format!(
                "ObjectPropertyDomain(:qlProp{} :QLClass{})\n",
                i, i
            ));
            ontology.push_str(&format!(
                "ObjectPropertyRange(:qlProp{} :QLClass{})\n",
                i,
                (i + 1) % size
            ));
        }

        ontology.push(')');
        ontology
    }

    /// Generate RL profile compliant ontology
    pub fn generate_rl_ontology(size: usize) -> String {
        let mut ontology = String::new();
        ontology.push_str("Prefix(:=<http://example.org/rl#>)\n");
        ontology.push_str("Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n");
        ontology.push_str("Ontology(<http://example.org/rl>)\n");

        // Generate classes with role hierarchy (RL profile focus)
        for i in 0..size {
            ontology.push_str(&format!("Declaration(Class(:RLClass{}))\n", i));
            if i < size / 2 {
                ontology.push_str(&format!("Declaration(ObjectProperty(:rlRole{}))\n", i));
            }
        }

        // Generate role hierarchy (RL profile allows)
        for i in 1..(size / 2) {
            ontology.push_str(&format!(
                "SubObjectPropertyOf(:rlRole{} :rlRole{})\n",
                i,
                i - 1
            ));
        }

        // Generate simple restrictions
        for i in 0..(size / 2) {
            ontology.push_str(&format!(
                "SubClassOf(:RLClass{} ObjectSomeValuesFrom(:rlRole{} :RLClass{}))\n",
                i,
                i % (size / 2),
                (i + 1) % size
            ));
        }

        ontology.push(')');
        ontology
    }

    /// Generate mixed profile ontology (will have violations)
    pub fn generate_mixed_ontology(size: usize) -> String {
        let mut ontology = String::new();
        ontology.push_str("Prefix(:=<http://example.org/mixed#>)\n");
        ontology.push_str("Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n");
        ontology.push_str("Ontology(<http://example.org/mixed>)\n");

        // Mixed features from different profiles
        for i in 0..size {
            ontology.push_str(&format!("Declaration(Class(:MixedClass{}))\n", i));
            if i < size / 3 {
                ontology.push_str(&format!("Declaration(ObjectProperty(:mixedProp{}))\n", i));
            }
        }

        // Add EL-compliant features
        for i in 0..(size / 3) {
            ontology.push_str(&format!(
                "SubClassOf(:MixedClass{} ObjectSomeValuesFrom(:mixedProp{} :MixedClass{}))\n",
                i,
                i,
                (i + 1) % size
            ));
        }

        // Add QL-compliant features
        for i in 0..(size / 3) {
            ontology.push_str(&format!(
                "ObjectPropertyDomain(:mixedProp{} :MixedClass{})\n",
                i, i
            ));
        }

        // Add some violations for testing
        if size > 10 {
            // This will violate EL profile (disjoint classes not allowed)
            ontology.push_str("DisjointClasses(:MixedClass0 :MixedClass1)\n");

            // This will violate QL profile (property chain not allowed)
            ontology.push_str(
                "SubObjectPropertyOf(ObjectPropertyChain(:mixedProp0 :mixedProp1) :mixedProp2)\n",
            );
        }

        ontology.push(')');
        ontology
    }
}

/// Benchmark EL profile validation performance
fn bench_el_profile_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("el_profile_validation");

    for size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("optimized", size), size, |b, &size| {
            let ontology_str = benchmark_data::generate_el_ontology(size);
            let parser = OwlFunctionalSyntaxParser::new();
            let ontology = parser.parse_str(&ontology_str).unwrap();
            let mut validator = Owl2ProfileValidator::new(Arc::new(ontology)).unwrap();

            b.iter(|| {
                let result = validator.validate_profile(black_box(Owl2Profile::EL));
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark QL profile validation performance
fn bench_ql_profile_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ql_profile_validation");

    for size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("optimized", size), size, |b, &size| {
            let ontology_str = benchmark_data::generate_ql_ontology(size);
            let parser = OwlFunctionalSyntaxParser::new();
            let ontology = parser.parse_str(&ontology_str).unwrap();
            let mut validator = Owl2ProfileValidator::new(Arc::new(ontology)).unwrap();

            b.iter(|| {
                let result = validator.validate_profile(black_box(Owl2Profile::QL));
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark RL profile validation performance
fn bench_rl_profile_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("rl_profile_validation");

    for size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("optimized", size), size, |b, &size| {
            let ontology_str = benchmark_data::generate_rl_ontology(size);
            let parser = OwlFunctionalSyntaxParser::new();
            let ontology = parser.parse_str(&ontology_str).unwrap();
            let mut validator = Owl2ProfileValidator::new(Arc::new(ontology)).unwrap();

            b.iter(|| {
                let result = validator.validate_profile(black_box(Owl2Profile::RL));
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark caching performance improvements
fn bench_caching_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("caching_performance");

    let ontology_str = benchmark_data::generate_el_ontology(100);
    let parser = OwlFunctionalSyntaxParser::new();
    let ontology = parser.parse_str(&ontology_str).unwrap();
    let _validator = Owl2ProfileValidator::new(Arc::new(ontology)).unwrap();

    // Benchmark cache hits
    group.bench_function("cache_hit", |b| {
        let ontology_str = benchmark_data::generate_el_ontology(100);
        let parser = OwlFunctionalSyntaxParser::new();
        let ontology = parser.parse_str(&ontology_str).unwrap();
        let mut validator = Owl2ProfileValidator::new(Arc::new(ontology)).unwrap();

        // Warm up cache
        let _ = validator.validate_profile(Owl2Profile::EL);

        b.iter(|| {
            let result = validator.validate_profile(black_box(Owl2Profile::EL));
            black_box(result)
        });
    });

    // Benchmark cache misses
    group.bench_function("cache_miss", |b| {
        let ontology_str2 = benchmark_data::generate_el_ontology(101); // Different size
        let parser2 = OwlFunctionalSyntaxParser::new();
        let ontology2 = parser2.parse_str(&ontology_str2).unwrap();
        let mut validator2 = Owl2ProfileValidator::new(Arc::new(ontology2)).unwrap();

        b.iter(|| {
            let result = validator2.validate_profile(black_box(Owl2Profile::EL));
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark pre-computation index performance
fn bench_precomputation_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("precomputation_performance");

    for size in [50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("index_creation", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let ontology_str = benchmark_data::generate_mixed_ontology(size);
                    let parser = OwlFunctionalSyntaxParser::new();
                    let _ontology = parser.parse_str(&ontology_str).unwrap();
                    let validator = Owl2ProfileValidator::new(Arc::new(_ontology)).unwrap();
                    let _ = validator.get_validation_stats(); // Forces index recomputation
                    black_box(())
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage and allocation efficiency
fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");

    let ontology_str = benchmark_data::generate_mixed_ontology(500);
    let parser = OwlFunctionalSyntaxParser::new();
    let ontology = parser.parse_str(&ontology_str).unwrap();

    group.bench_function("validation_memory_usage", |b| {
        b.iter(|| {
            let mut validator =
                Owl2ProfileValidator::new(Arc::new(black_box(ontology.clone()))).unwrap();
            let result = validator.validate_profile(Owl2Profile::EL);
            let owned_stats = validator.get_validation_stats().clone();
            black_box((result.unwrap(), owned_stats))
        });
    });

    group.finish();
}

/// Benchmark profile analysis and reporting
fn bench_profile_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("profile_analysis");

    for size in [100, 300, 500].iter() {
        group.bench_with_input(BenchmarkId::new("analysis", size), size, |b, &size| {
            let ontology_str = benchmark_data::generate_mixed_ontology(size);
            let parser = OwlFunctionalSyntaxParser::new();
            let ontology = parser.parse_str(&ontology_str).unwrap();
            let mut validator = Owl2ProfileValidator::new(Arc::new(ontology)).unwrap();

            b.iter(|| {
                let report = validator.validate_all_profiles();
                black_box(report)
            });
        });
    }

    group.finish();
}

/// Comprehensive throughput benchmark
fn bench_validation_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation_throughput");

    // Test multiple profiles on different ontologies
    let test_cases = vec![
        ("EL_100", benchmark_data::generate_el_ontology(100)),
        ("QL_100", benchmark_data::generate_ql_ontology(100)),
        ("RL_100", benchmark_data::generate_rl_ontology(100)),
        ("Mixed_100", benchmark_data::generate_mixed_ontology(100)),
    ];

    for (name, ontology_str) in test_cases {
        let parser = OwlFunctionalSyntaxParser::new();
        let ontology = parser.parse_str(&ontology_str).unwrap();
        let mut validator = Owl2ProfileValidator::new(Arc::new(ontology)).unwrap();

        group.bench_function(name, |b| {
            b.iter(|| {
                // Validate against all profiles
                let el_result = validator.validate_profile(black_box(Owl2Profile::EL));
                let ql_result = validator.validate_profile(black_box(Owl2Profile::QL));
                let rl_result = validator.validate_profile(black_box(Owl2Profile::RL));
                black_box((el_result, ql_result, rl_result))
            });
        });
    }

    group.finish();
}

/// Performance regression test
fn bench_performance_regression(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_regression");

    // Test specific scenarios that were previously slow
    let large_ontology = benchmark_data::generate_mixed_ontology(1000);
    let parser = OwlFunctionalSyntaxParser::new();
    let ontology = parser.parse_str(&large_ontology).unwrap();
    let mut validator = Owl2ProfileValidator::new(Arc::new(ontology)).unwrap();

    group.bench_function("large_ontology_validation", |b| {
        b.iter(|| {
            let result = validator.validate_profile(black_box(Owl2Profile::EL));
            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_el_profile_validation,
    bench_ql_profile_validation,
    bench_rl_profile_validation,
    bench_caching_performance,
    bench_precomputation_performance,
    bench_memory_efficiency,
    bench_profile_analysis,
    bench_validation_throughput,
    bench_performance_regression
);

criterion_main!(benches);
