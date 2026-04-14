use criterion::{black_box, criterion_group, criterion_main, Criterion};
use provchain_org::config::Config;
use provchain_org::ontology::{OntologyConfig, OntologyManager, ShaclValidator};
use std::fs;
use tempfile::TempDir;

struct BenchmarkAssets {
    _temp_dir: TempDir,
    ontology_path: String,
    core_shacl_path: String,
    domain_shacl_path: String,
}

fn minimal_shacl_shapes() -> &'static str {
    r#"@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.org/test#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

ex:ProductShape a sh:NodeShape ;
    sh:targetClass ex:Product ;
    sh:property [
        sh:path ex:hasOrigin ;
        sh:minCount 1 ;
        sh:class ex:Location ;
    ] .

ex:LocationShape a sh:NodeShape ;
    sh:targetClass ex:Location ;
    sh:property [
        sh:path rdf:type ;
        sh:minCount 1 ;
    ] .
"#
}

fn exact_class_ontology() -> &'static str {
    r#"<?xml version="1.0"?>
<rdf:RDF xmlns="http://example.org/test#"
         xml:base="http://example.org/test"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#">
    <owl:Ontology rdf:about="http://example.org/test" />
    <owl:Class rdf:about="http://example.org/test#Product" />
    <owl:Class rdf:about="http://example.org/test#Location" />
    <owl:ObjectProperty rdf:about="http://example.org/test#hasOrigin">
        <rdfs:domain rdf:resource="http://example.org/test#Product"/>
        <rdfs:range rdf:resource="http://example.org/test#Location"/>
    </owl:ObjectProperty>
</rdf:RDF>"#
}

fn subclass_ontology() -> &'static str {
    r#"<?xml version="1.0"?>
<rdf:RDF xmlns="http://example.org/test#"
         xml:base="http://example.org/test"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#">
    <owl:Ontology rdf:about="http://example.org/test" />
    <owl:Class rdf:about="http://example.org/test#Product" />
    <owl:Class rdf:about="http://example.org/test#Location" />
    <owl:Class rdf:about="http://example.org/test#ColdStorageLocation">
        <rdfs:subClassOf rdf:resource="http://example.org/test#Location" />
    </owl:Class>
    <owl:ObjectProperty rdf:about="http://example.org/test#hasOrigin">
        <rdfs:domain rdf:resource="http://example.org/test#Product"/>
        <rdfs:range rdf:resource="http://example.org/test#Location"/>
    </owl:ObjectProperty>
</rdf:RDF>"#
}

fn valid_exact_transaction() -> &'static str {
    r#"@prefix ex: <http://example.org/test#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

ex:product1 rdf:type ex:Product ;
    ex:hasOrigin ex:location1 .

ex:location1 rdf:type ex:Location .
"#
}

fn subclass_transaction() -> &'static str {
    r#"@prefix ex: <http://example.org/test#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

ex:product1 rdf:type ex:Product ;
    ex:hasOrigin ex:coldStorage1 .

ex:coldStorage1 rdf:type ex:ColdStorageLocation .
"#
}

fn invalid_class_transaction() -> &'static str {
    r#"@prefix ex: <http://example.org/test#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

ex:product1 rdf:type ex:Product ;
    ex:hasOrigin ex:location1 .

ex:location1 rdf:type ex:Product .
"#
}

fn write_assets(ontology_content: &str) -> BenchmarkAssets {
    let temp_dir = TempDir::new().expect("temp dir");
    let ontology_path = temp_dir.path().join("domain.owl");
    let core_shacl_path = temp_dir.path().join("core.shacl.ttl");
    let domain_shacl_path = temp_dir.path().join("domain.shacl.ttl");

    fs::write(&ontology_path, ontology_content).expect("write ontology");
    fs::write(&core_shacl_path, minimal_shacl_shapes()).expect("write core shapes");
    fs::write(&domain_shacl_path, minimal_shacl_shapes()).expect("write domain shapes");

    BenchmarkAssets {
        _temp_dir: temp_dir,
        ontology_path: ontology_path.to_string_lossy().to_string(),
        core_shacl_path: core_shacl_path.to_string_lossy().to_string(),
        domain_shacl_path: domain_shacl_path.to_string_lossy().to_string(),
    }
}

fn make_ontology_config(assets: &BenchmarkAssets) -> OntologyConfig {
    let config = Config::default();
    let mut ontology_config =
        OntologyConfig::new(Some(assets.ontology_path.clone()), &config).expect("ontology config");
    ontology_config.core_shacl_path = assets.core_shacl_path.clone();
    ontology_config.domain_shacl_path = assets.domain_shacl_path.clone();
    ontology_config
}

fn bench_ontology_admission(c: &mut Criterion) {
    let exact_assets = write_assets(exact_class_ontology());
    let subclass_assets = write_assets(subclass_ontology());

    let exact_validator = ShaclValidator::new(
        &exact_assets.core_shacl_path,
        &exact_assets.domain_shacl_path,
        "bench_exact".to_string(),
        None,
    )
    .expect("validator");

    let subclass_manager =
        OntologyManager::new(make_ontology_config(&subclass_assets)).expect("ontology manager");
    let invalid_validator = ShaclValidator::new(
        &subclass_assets.core_shacl_path,
        &subclass_assets.domain_shacl_path,
        "bench_invalid".to_string(),
        None,
    )
    .expect("validator");

    let mut group = c.benchmark_group("ontology_admission_validation");

    group.bench_function("validator_exact_class_fallback", |b| {
        b.iter(|| {
            let result = exact_validator
                .validate_transaction(black_box(valid_exact_transaction()))
                .expect("validation");
            black_box(result);
        });
    });

    group.bench_function("manager_subclass_reasoning", |b| {
        b.iter(|| {
            let result = subclass_manager
                .validate_transaction(black_box(subclass_transaction()))
                .expect("validation");
            black_box(result);
        });
    });

    group.bench_function("validation_failure_explanation_summary", |b| {
        b.iter(|| {
            let validation = invalid_validator
                .validate_transaction(black_box(invalid_class_transaction()))
                .expect("validation");
            assert!(!validation.is_valid);
            let summary = validation.explanation_summary();
            black_box(summary);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_ontology_admission);
criterion_main!(benches);
