//! OWL2 Performance Benchmarks
//!
//! Benchmarks for the enhanced OWL2 features

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use provchain_org::core::blockchain::Blockchain;
use provchain_org::core::entity::{DomainType, EntityType, PropertyValue, TraceableEntity};
use provchain_org::semantic::owl2_traceability::Owl2EnhancedTraceability;

/// Helper to generate test entities
fn generate_test_entities(count: usize) -> Vec<TraceableEntity> {
    let mut entities = Vec::with_capacity(count);
    for i in 0..count {
        let mut entity = TraceableEntity::new(
            format!("entity_{:04}", i),
            EntityType::Product,
            DomainType::SupplyChain,
        );
        entity.add_property(
            "batchId".to_string(),
            PropertyValue::String(format!("BATCH{:04}", i)),
        );
        entity.add_property(
            "sku".to_string(),
            PropertyValue::String(format!("SKU{:04}", i)),
        );
        entity.add_property(
            "name".to_string(),
            PropertyValue::String(format!("Product {}", i)),
        );
        entity.add_property(
            "producedBy".to_string(),
            PropertyValue::String("Dairy Farm A".to_string()),
        );
        entity.add_property(
            "locatedAt".to_string(),
            PropertyValue::String("Processing Plant B".to_string()),
        );
        entity.add_property(
            "inputTo".to_string(),
            PropertyValue::String(format!("Process {:04}", i)),
        );
        entity.add_property(
            "outputOf".to_string(),
            PropertyValue::String("Production XYZ".to_string()),
        );
        entities.push(entity);
    }
    entities
}

/// Benchmark entity-to-OWL2 ontology conversion
pub fn bench_entities_to_owl_ontology(c: &mut Criterion) {
    let mut group = c.benchmark_group("owl2_conversion");
    let blockchain = Blockchain::new();
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let entities = generate_test_entities(size);
            b.iter(|| {
                let _ = black_box(owl2_enhancer.entities_to_owl_ontology(&entities));
            })
        });
    }
    group.finish();
}

/// Benchmark owl:hasKey validation
pub fn bench_haskey_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("owl2_haskey_validation");
    let blockchain = Blockchain::new();
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let entities = generate_test_entities(size);
            b.iter(|| {
                let _ = black_box(owl2_enhancer.validate_entity_keys(&entities));
            })
        });
    }
    group.finish();
}

/// Benchmark property chain inference
pub fn bench_property_chain_inference(c: &mut Criterion) {
    let mut group = c.benchmark_group("owl2_property_chain_inference");
    let blockchain = Blockchain::new();
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let entities = generate_test_entities(size);
            b.iter(|| {
                let _ = black_box(owl2_enhancer.apply_property_chain_inference(&entities));
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_entities_to_owl_ontology,
    bench_haskey_validation,
    bench_property_chain_inference
);

criterion_main!(benches);
