use provchain_org::ontology::{DomainConfig, OntologyConfig, OntologyManager, ValidationMode};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_valid_domain_extension() {
    let temp_dir = TempDir::new().unwrap();
    let core_path = temp_dir.path().join("core.ttl");
    let domain_path = temp_dir.path().join("dairy_test.ttl");
    let core_shacl_path = temp_dir.path().join("core_shacl.ttl");
    let domain_shacl_path = temp_dir.path().join("domain_shacl.ttl");

    // Create dummy SHACL files (needed for initialization)
    fs::write(&core_shacl_path, "").unwrap();
    fs::write(&domain_shacl_path, "").unwrap();

    // 1. Create Core Ontology
    let core_content = r#"
@prefix : <http://provchain.org/core#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<http://provchain.org/core> rdf:type owl:Ontology .

:Batch rdf:type owl:Class .
:Product rdf:type owl:Class .
:Process rdf:type owl:Class .
:Participant rdf:type owl:Class .
    "#;
    fs::write(&core_path, core_content).unwrap();

    // 2. Create Valid Domain Ontology (MilkBatch subclass of Batch)
    let domain_content = r#"
@prefix : <http://provchain.org/dairy_test#> .
@prefix core: <http://provchain.org/core#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<http://provchain.org/dairy_test> rdf:type owl:Ontology ;
    owl:imports <http://provchain.org/core> .

:MilkBatch rdf:type owl:Class ;
    rdfs:subClassOf core:Batch .
    "#;
    fs::write(&domain_path, domain_content).unwrap();

    // 3. Initialize Manager
    let config = OntologyConfig {
        core_ontology_path: core_path.to_string_lossy().to_string(),
        domain_ontology_path: domain_path.to_string_lossy().to_string(),
        core_shacl_path: core_shacl_path.to_string_lossy().to_string(),
        domain_shacl_path: domain_shacl_path.to_string_lossy().to_string(),
        validation_mode: ValidationMode::Strict,
        ontology_hash: "hash".to_string(),
    };

    let mut manager = OntologyManager::new(config).expect("Failed to create manager");

    // 4. Validate Extension
    let result = manager.validate_domain_extension();
    assert!(result.is_ok(), "Valid extension should pass validation: {:?}", result.err());
}

#[test]
fn test_invalid_domain_extension() {
    let temp_dir = TempDir::new().unwrap();
    let core_path = temp_dir.path().join("core.ttl");
    let domain_path = temp_dir.path().join("dairy_test.ttl");
    let core_shacl_path = temp_dir.path().join("core_shacl.ttl");
    let domain_shacl_path = temp_dir.path().join("domain_shacl.ttl");

    fs::write(&core_shacl_path, "").unwrap();
    fs::write(&domain_shacl_path, "").unwrap();

    // 1. Create Core Ontology
    let core_content = r#"
@prefix : <http://provchain.org/core#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<http://provchain.org/core> rdf:type owl:Ontology .

:Batch rdf:type owl:Class .
    "#;
    fs::write(&core_path, core_content).unwrap();

    // 2. Create Invalid Domain Ontology (RogueClass not subclass of anything)
    let domain_content = r#"<http://provchain.org/dairy_test> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/2002/07/owl#Ontology> .
<http://provchain.org/dairy_test#RogueClass> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/2002/07/owl#Class> ."#;
    fs::write(&domain_path, domain_content).unwrap();

    // 3. Initialize Manager
    let config = OntologyConfig {
        core_ontology_path: core_path.to_string_lossy().to_string(),
        domain_ontology_path: domain_path.to_string_lossy().to_string(),
        core_shacl_path: core_shacl_path.to_string_lossy().to_string(),
        domain_shacl_path: domain_shacl_path.to_string_lossy().to_string(),
        validation_mode: ValidationMode::Strict,
        ontology_hash: "hash".to_string(),
    };

    let mut manager = OntologyManager::new(config).expect("Failed to create manager");

    // 4. Validate Extension
    let result = manager.validate_domain_extension();
    assert!(result.is_err(), "Invalid extension should fail validation");
    
    let err = result.unwrap_err();
    println!("Expected error: {}", err);
    // Check if error message contains relevant info
    assert!(err.to_string().contains("Domain class must be a subclass"));
}
