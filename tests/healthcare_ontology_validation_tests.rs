use provchain_org::config::Config;
use provchain_org::core::blockchain::Blockchain;
use provchain_org::ontology::OntologyConfig;

fn valid_healthcare_device_transaction() -> &'static str {
    r#"
        @prefix ex: <http://example.org/> .
        @prefix healthcare: <http://provchain.org/healthcare#> .
        @prefix trace: <http://provchain.org/trace#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        ex:deviceAudit001 a healthcare:MedicalDevice ;
            healthcare:deviceSerialNumber "MD-2026-AX91" ;
            trace:name "Sterile Infusion Pump" ;
            trace:status "Sterile" ;
            trace:location "Ward 3 Central Supply" ;
            trace:timestamp "2026-02-28T12:00:00Z"^^xsd:dateTime .
    "#
}

fn invalid_healthcare_device_transaction() -> &'static str {
    r#"
        @prefix ex: <http://example.org/> .
        @prefix healthcare: <http://provchain.org/healthcare#> .
        @prefix trace: <http://provchain.org/trace#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        ex:deviceAuditBad001 a healthcare:MedicalDevice ;
            trace:name "Sterile Infusion Pump" ;
            trace:status "Sterile" ;
            trace:location "Ward 3 Central Supply" ;
            trace:timestamp "2026-02-28T12:00:00Z"^^xsd:dateTime .
    "#
}

fn healthcare_ontology_config() -> OntologyConfig {
    let config = Config::default();
    OntologyConfig::new(
        Some("src/semantic/ontologies/healthcare.owl".to_string()),
        &config,
    )
    .expect("healthcare ontology config should load")
}

#[test]
fn test_healthcare_ontology_files_exist() {
    let ontology_config = healthcare_ontology_config();
    ontology_config
        .validate_files()
        .expect("healthcare ontology + SHACL files should exist");
    assert_eq!(ontology_config.domain_name().unwrap(), "healthcare");
}

#[test]
fn test_blockchain_accepts_valid_healthcare_device_block() {
    let ontology_config = healthcare_ontology_config();
    let mut blockchain =
        Blockchain::new_with_ontology(ontology_config).expect("ontology-backed blockchain");

    let initial_length = blockchain.chain.len();
    let result = blockchain.add_block(valid_healthcare_device_transaction().to_string());

    assert!(result.is_ok(), "valid healthcare transaction should pass");
    assert_eq!(blockchain.chain.len(), initial_length + 1);
    assert!(blockchain
        .chain
        .last()
        .unwrap()
        .data
        .contains("deviceAudit001"));
}

#[test]
fn test_blockchain_rejects_invalid_healthcare_device_block() {
    let ontology_config = healthcare_ontology_config();
    let mut blockchain =
        Blockchain::new_with_ontology(ontology_config).expect("ontology-backed blockchain");

    let initial_length = blockchain.chain.len();
    let result = blockchain.add_block(invalid_healthcare_device_transaction().to_string());

    assert!(
        result.is_err(),
        "invalid healthcare transaction should fail"
    );
    assert_eq!(blockchain.chain.len(), initial_length);

    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("Transaction validation failed"));
    assert!(error_message.contains("healthcare#deviceSerialNumber"));
    assert!(error_message.contains("constraint_breakdown="));
}
