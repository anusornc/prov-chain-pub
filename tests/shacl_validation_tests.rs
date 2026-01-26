use oxigraph::model::NamedNode;
use provchain_org::semantic::shacl_validator::{ShaclConfig, ShaclValidator};
use provchain_org::storage::rdf_store::RDFStore;

#[test]
fn test_shacl_validation_with_valid_data() {
    // Create SHACL validator
    let config = ShaclConfig {
        enabled: true,
        shapes_path: "src/semantic/shapes/traceability.shacl.ttl".to_string(),
        fail_on_error: false,
    };
    let validator = ShaclValidator::new(config).expect("Failed to create SHACL validator");

    // Create RDF store with valid data
    let mut rdf_store = RDFStore::new();
    let graph_name = NamedNode::new("http://example.org/test").unwrap();

    // Load ontology data first
    let ontology_data = std::fs::read_to_string("src/semantic/ontologies/generic_core.owl")
        .expect("Failed to read ontology file");
    // For testing purposes, we'll load the ontology data into the same graph as the test data
    // This is a simplification for testing - in production, the ontology would be in a separate graph
    rdf_store.add_rdf_to_graph(&ontology_data, &graph_name);

    // Valid data that conforms to SHACL shapes
    let valid_data = r#"
        @prefix ex: <http://example.org/> .
        @prefix core: <http://provchain.org/core#> .
        @prefix prov: <http://www.w3.org/ns/prov#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:batch1 a core:Batch ;
            core:hasIdentifier "BATCH001" ;
            core:producedAt "2025-08-08T10:00:00Z"^^xsd:dateTime ;
            prov:wasAttributedTo ex:farmer1 .

        ex:farmer1 a core:Supplier ;
            rdfs:label "Green Valley Farm" .
    "#;

    rdf_store.add_rdf_to_graph(valid_data, &graph_name);

    // Debug: Print all triples in the data store
    println!("All triples in data store:");
    for quad in rdf_store.store.iter().flatten() {
        println!("  {}", quad);
    }

    // Validate the data
    let result = validator
        .validate_graph(&rdf_store.store, &graph_name)
        .expect("Failed to validate graph");

    // Should pass validation
    assert!(
        result.conforms,
        "Valid data should pass SHACL validation. Errors: {:?}",
        result.errors
    );
    assert_eq!(
        result.errors.len(),
        0,
        "Valid data should have no validation errors"
    );
}

#[test]
fn test_shacl_validation_with_missing_required_property() {
    // Create SHACL validator
    let config = ShaclConfig {
        enabled: true,
        shapes_path: "src/semantic/shapes/traceability.shacl.ttl".to_string(),
        fail_on_error: false,
    };
    let validator = ShaclValidator::new(config).expect("Failed to create SHACL validator");

    // Create RDF store with invalid data (missing required property)
    let mut rdf_store = RDFStore::new();
    let graph_name = NamedNode::new("http://example.org/test").unwrap();

    // Load ontology data first
    let ontology_data = std::fs::read_to_string("src/semantic/ontologies/generic_core.owl")
        .expect("Failed to read ontology file");
    let ontology_graph = NamedNode::new("http://provchain.org/ontology").unwrap();
    rdf_store.add_rdf_to_graph(&ontology_data, &ontology_graph);

    // Invalid data: Batch missing required hasIdentifier
    let invalid_data = r#"
        @prefix ex: <http://example.org/> .
        @prefix core: <http://provchain.org/core#> .
        @prefix prov: <http://www.w3.org/ns/prov#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:batch1 a core:Batch ;
            core:producedAt "2025-08-08T10:00:00Z"^^xsd:dateTime ;
            prov:wasAttributedTo ex:farmer1 .

        ex:farmer1 a core:Supplier ;
            rdfs:label "Green Valley Farm" .
    "#;

    rdf_store.add_rdf_to_graph(invalid_data, &graph_name);

    // Validate the data
    let result = validator
        .validate_graph(&rdf_store.store, &graph_name)
        .expect("Failed to validate graph");

    // Print debug information
    println!(
        "Validation result: conforms={}, errors={:?}",
        result.conforms, result.errors
    );

    // Should fail validation
    assert!(
        !result.conforms,
        "Invalid data should fail SHACL validation"
    );
    assert!(
        !result.errors.is_empty(),
        "Invalid data should have validation errors"
    );

    // Check that the error is about the missing required property
    let error_messages: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
    let error_text = error_messages.join(" ");
    assert!(
        error_text.contains("missing required property"),
        "Error should mention missing required property. Actual errors: {:?}",
        error_messages
    );
}

#[test]
fn test_shacl_validation_with_incorrect_datatype() {
    // Create SHACL validator
    let config = ShaclConfig {
        enabled: true,
        shapes_path: "src/semantic/shapes/traceability.shacl.ttl".to_string(),
        fail_on_error: false,
    };
    let validator = ShaclValidator::new(config).expect("Failed to create SHACL validator");

    // Create RDF store with invalid data (incorrect datatype)
    let mut rdf_store = RDFStore::new();
    let graph_name = NamedNode::new("http://example.org/test").unwrap();

    // Load ontology data first
    let ontology_data = std::fs::read_to_string("src/semantic/ontologies/generic_core.owl")
        .expect("Failed to read ontology file");
    let ontology_graph = NamedNode::new("http://provchain.org/ontology").unwrap();
    rdf_store.add_rdf_to_graph(&ontology_data, &ontology_graph);

    // Invalid data: incorrect datatype for hasIdentifier (should be string, not integer)
    let invalid_data = r#"
        @prefix ex: <http://example.org/> .
        @prefix core: <http://provchain.org/core#> .
        @prefix prov: <http://www.w3.org/ns/prov#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:batch1 a core:Batch ;
            core:hasIdentifier 123 ;
            core:producedAt "2025-08-08T10:00:00Z"^^xsd:dateTime ;
            prov:wasAttributedTo ex:farmer1 .

        ex:farmer1 a core:Supplier ;
            rdfs:label "Green Valley Farm" .
    "#;

    rdf_store.add_rdf_to_graph(invalid_data, &graph_name);

    // Validate the data
    let result = validator
        .validate_graph(&rdf_store.store, &graph_name)
        .expect("Failed to validate graph");

    // Print debug information
    println!(
        "Validation result: conforms={}, errors={:?}",
        result.conforms, result.errors
    );

    // Should fail validation
    assert!(
        !result.conforms,
        "Invalid data should fail SHACL validation"
    );
    assert!(
        !result.errors.is_empty(),
        "Invalid data should have validation errors"
    );

    // Check that the error is about the incorrect datatype
    let error_messages: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
    let error_text = error_messages.join(" ");
    assert!(
        error_text.contains("incorrect datatype"),
        "Error should mention incorrect datatype. Actual errors: {:?}",
        error_messages
    );
}

#[test]
fn test_shacl_validation_disabled() {
    // Create SHACL validator with validation disabled
    let config = ShaclConfig {
        enabled: false, // Disabled
        shapes_path: "src/semantic/shapes/traceability.shacl.ttl".to_string(),
        fail_on_error: false,
    };
    let validator = ShaclValidator::new(config).expect("Failed to create SHACL validator");

    // Create RDF store with invalid data
    let mut rdf_store = RDFStore::new();
    let graph_name = NamedNode::new("http://example.org/test").unwrap();

    // Load ontology data first
    let ontology_data = std::fs::read_to_string("src/semantic/ontologies/generic_core.owl")
        .expect("Failed to read ontology file");
    let ontology_graph = NamedNode::new("http://provchain.org/ontology").unwrap();
    rdf_store.add_rdf_to_graph(&ontology_data, &ontology_graph);

    // Invalid data that would fail validation if enabled
    let invalid_data = r#"
        @prefix ex: <http://example.org/> .
        @prefix core: <http://provchain.org/core#> .
        @prefix prov: <http://www.w3.org/ns/prov#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:batch1 a core:Batch ;
            core:producedAt "2025-08-08T10:00:00Z"^^xsd:dateTime ;
            prov:wasAttributedTo ex:farmer1 .

        ex:farmer1 a core:Supplier ;
            rdfs:label "Green Valley Farm" .
    "#;

    rdf_store.add_rdf_to_graph(invalid_data, &graph_name);

    // Validate the data
    let result = validator
        .validate_graph(&rdf_store.store, &graph_name)
        .expect("Failed to validate graph");

    // Should pass validation because it's disabled
    assert!(
        result.conforms,
        "Validation should pass when disabled even with invalid data"
    );
    assert_eq!(
        result.errors.len(),
        0,
        "Should have no validation errors when disabled"
    );
}

#[test]
fn test_format_validation_report() {
    // Create SHACL validator
    let config = ShaclConfig {
        enabled: true,
        shapes_path: "src/semantic/shapes/traceability.shacl.ttl".to_string(),
        fail_on_error: false,
    };
    let validator = ShaclValidator::new(config).expect("Failed to create SHACL validator");

    // Test formatting with conforming data
    let conforming_result = provchain_org::semantic::shacl_validator::ShaclValidationResult {
        conforms: true,
        errors: vec![],
        warnings: vec![],
    };

    let report = validator.format_validation_report(&conforming_result);
    assert!(
        report.contains("passed"),
        "Report should indicate validation passed"
    );

    // Test formatting with non-conforming data
    let non_conforming_result = provchain_org::semantic::shacl_validator::ShaclValidationResult {
        conforms: false,
        errors: vec![
            provchain_org::semantic::shacl_validator::ShaclValidationError {
                message: "Test error message".to_string(),
                focus_node: Some("http://example.org/node1".to_string()),
                path: Some("http://example.org/property".to_string()),
                value: Some("test value".to_string()),
            },
        ],
        warnings: vec![],
    };

    let report = validator.format_validation_report(&non_conforming_result);
    assert!(
        report.contains("failed"),
        "Report should indicate validation failed"
    );
    assert!(
        report.contains("Test error message"),
        "Report should include error message"
    );
}
