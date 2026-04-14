use provchain_org::config::Config;
use provchain_org::core::blockchain::Blockchain;
use provchain_org::ontology::OntologyConfig;

fn valid_pharmaceutical_storage_transaction() -> &'static str {
    r#"
        @prefix ex: <http://example.org/> .
        @prefix pharma: <http://provchain.org/pharma#> .
        @prefix trace: <http://provchain.org/trace#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        ex:storageCheck001 a pharma:PharmaceuticalStorage ;
            trace:temperature "5.2"^^xsd:decimal ;
            trace:humidity "55.0"^^xsd:decimal ;
            pharma:lightProtection "true"^^xsd:boolean ;
            pharma:controlledSubstance "false"^^xsd:boolean ;
            pharma:securityLevel "High" ;
            trace:recordedAt "2026-02-28T10:00:00Z"^^xsd:dateTime .
    "#
}

fn invalid_pharmaceutical_storage_transaction() -> &'static str {
    r#"
        @prefix ex: <http://example.org/> .
        @prefix pharma: <http://provchain.org/pharma#> .
        @prefix trace: <http://provchain.org/trace#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        ex:storageCheckBad001 a pharma:PharmaceuticalStorage ;
            trace:temperature "5.2"^^xsd:decimal ;
            trace:humidity "55.0"^^xsd:decimal ;
            pharma:securityLevel "High" ;
            trace:recordedAt "2026-02-28T10:00:00Z"^^xsd:dateTime .
    "#
}

fn pharmaceutical_ontology_config() -> OntologyConfig {
    let config = Config::default();
    OntologyConfig::new(
        Some("src/semantic/ontologies/pharmaceutical.owl".to_string()),
        &config,
    )
    .expect("pharmaceutical ontology config should load")
}

#[test]
fn test_pharmaceutical_ontology_files_exist() {
    let ontology_config = pharmaceutical_ontology_config();
    ontology_config
        .validate_files()
        .expect("pharmaceutical ontology + SHACL files should exist");
    assert_eq!(ontology_config.domain_name().unwrap(), "pharmaceutical");
}

#[test]
fn test_blockchain_accepts_valid_pharmaceutical_storage_block() {
    let ontology_config = pharmaceutical_ontology_config();
    let mut blockchain =
        Blockchain::new_with_ontology(ontology_config).expect("ontology-backed blockchain");

    let initial_length = blockchain.chain.len();
    let result = blockchain.add_block(valid_pharmaceutical_storage_transaction().to_string());

    assert!(
        result.is_ok(),
        "valid pharmaceutical transaction should pass"
    );
    assert_eq!(blockchain.chain.len(), initial_length + 1);
    assert!(blockchain
        .chain
        .last()
        .unwrap()
        .data
        .contains("storageCheck001"));
}

#[test]
fn test_blockchain_rejects_invalid_pharmaceutical_storage_block() {
    let ontology_config = pharmaceutical_ontology_config();
    let mut blockchain =
        Blockchain::new_with_ontology(ontology_config).expect("ontology-backed blockchain");

    let initial_length = blockchain.chain.len();
    let result = blockchain.add_block(invalid_pharmaceutical_storage_transaction().to_string());

    assert!(
        result.is_err(),
        "invalid pharmaceutical transaction should fail"
    );
    assert_eq!(blockchain.chain.len(), initial_length);

    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("Transaction validation failed"));
    assert!(error_message.contains("pharma#lightProtection"));
    assert!(error_message.contains("constraint_breakdown="));
}
