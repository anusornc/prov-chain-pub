//! OWL2 Performance Benchmarks
//!
//! Benchmarks for the enhanced OWL2 features

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use provchain_org::core::blockchain::Blockchain;
use provchain_org::core::entity::{DomainType, EntityType, PropertyValue, TraceableEntity};
use provchain_org::semantic::owl2_traceability::Owl2EnhancedTraceability;

/// Benchmark entity-to-OWL2 ontology conversion
pub fn bench_entities_to_owl_ontology(c: &mut Criterion) {
    let blockchain = Blockchain::new();
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);

    // Create sample entities for benchmarking
    let mut entities = Vec::new();

    // Create 10 entities for benchmarking
    for i in 0..10 {
        let mut entity = TraceableEntity::new(
            format!("entity_{:03}", i),
            EntityType::Product,
            DomainType::SupplyChain,
        );
        entity.add_property(
            "batchId".to_string(),
            PropertyValue::String(format!("BATCH{:03}", i)),
        );
        entity.add_property(
            "sku".to_string(),
            PropertyValue::String(format!("SKU{:03}", i)),
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

        entities.push(entity);
    }

    c.bench_function("entities_to_owl_ontology_10", |b| {
        b.iter(|| {
            let _ = black_box(owl2_enhancer.entities_to_owl_ontology(&entities));
        })
    });
}

/// Benchmark owl:hasKey validation
pub fn bench_haskey_validation(c: &mut Criterion) {
    let blockchain = Blockchain::new();
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);

    // Create sample entities for benchmarking
    let mut entities = Vec::new();

    // Create 10 entities for benchmarking
    for i in 0..10 {
        let mut entity = TraceableEntity::new(
            format!("entity_{:03}", i),
            EntityType::Product,
            DomainType::SupplyChain,
        );
        entity.add_property(
            "batchId".to_string(),
            PropertyValue::String(format!("BATCH{:03}", i)),
        );
        entity.add_property(
            "sku".to_string(),
            PropertyValue::String(format!("SKU{:03}", i)),
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

        entities.push(entity);
    }

    c.bench_function("haskey_validation_10", |b| {
        b.iter(|| {
            let _ = black_box(owl2_enhancer.validate_entity_keys(&entities));
        })
    });
}

/// Benchmark property chain inference
pub fn bench_property_chain_inference(c: &mut Criterion) {
    let blockchain = Blockchain::new();
    let owl2_enhancer = Owl2EnhancedTraceability::new(blockchain);

    // Create sample entities for benchmarking
    let mut entities = Vec::new();

    // Create 10 entities for benchmarking
    for i in 0..10 {
        let mut entity = TraceableEntity::new(
            format!("entity_{:03}", i),
            EntityType::Product,
            DomainType::SupplyChain,
        );
        entity.add_property(
            "batchId".to_string(),
            PropertyValue::String(format!("BATCH{:03}", i)),
        );
        entity.add_property(
            "sku".to_string(),
            PropertyValue::String(format!("SKU{:03}", i)),
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
            PropertyValue::String(format!("Process {:03}", i)),
        );
        entity.add_property(
            "outputOf".to_string(),
            PropertyValue::String("Production XYZ".to_string()),
        );

        entities.push(entity);
    }

    c.bench_function("property_chain_inference_10", |b| {
        b.iter(|| {
            let _ = black_box(owl2_enhancer.apply_property_chain_inference(&entities));
        })
    });
}

criterion_group!(
    benches,
    bench_entities_to_owl_ontology,
    bench_haskey_validation,
    bench_property_chain_inference
);

criterion_main!(benches);
