use provchain_org::config::Config;
use provchain_org::core::blockchain::Blockchain;
use provchain_org::ontology::OntologyConfig;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn synthesized_healthcare_device_event() -> &'static str {
    r#"
        @prefix ex: <http://example.org/> .
        @prefix healthcare: <http://provchain.org/healthcare#> .
        @prefix trace: <http://provchain.org/trace#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        ex:deviceAudit-00889842000123 a healthcare:MedicalDevice ;
            healthcare:deviceSerialNumber "MD-42000123" ;
            trace:name "Example Infusion Pump" ;
            trace:status "Sterile" ;
            trace:location "Registry Catalog" ;
            trace:timestamp "2026-02-20T00:00:00Z"^^xsd:dateTime .
    "#
}

fn synthesized_uht_product_event() -> &'static str {
    r#"
        @prefix ex: <http://example.org/> .
        @prefix uht: <http://provchain.org/uht#> .
        @prefix core: <http://provchain.org/core#> .
        @prefix trace: <http://provchain.org/trace#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        ex:uhtProduct-1001 a uht:UHTProduct , core:Product ;
            trace:name "UHT whole milk" ;
            trace:participant "agent:brand:unknown-food-owner" ;
            trace:status "Released" ;
            uht:milkType "Whole" ;
            uht:fatContent "3.5"^^xsd:decimal ;
            uht:proteinContent "3.2"^^xsd:decimal ;
            uht:expiryDate "2026-12-31"^^xsd:date ;
            uht:packageSize "1.0"^^xsd:decimal .
    "#
}

fn synthesized_uht_epcis_event() -> &'static str {
    r#"
        @prefix ex: <http://example.org/> .
        @prefix uht: <http://provchain.org/uht#> .
        @prefix core: <http://provchain.org/core#> .
        @prefix trace: <http://provchain.org/trace#> .
        @prefix epcis: <https://ns.gs1.org/epcis/> .
        @prefix cbv: <https://ref.gs1.org/cbv/> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        ex:uhtEpcisProduct-1001 a uht:UHTProduct , core:Product , epcis:ObjectEvent ;
            trace:name "UHT whole milk" ;
            trace:participant "agent:brand:unknown-food-owner" ;
            trace:status "Released" ;
            uht:milkType "Whole" ;
            uht:fatContent "3.5"^^xsd:decimal ;
            uht:proteinContent "3.2"^^xsd:decimal ;
            uht:expiryDate "2026-12-31"^^xsd:date ;
            uht:packageSize "1.0"^^xsd:decimal ;
            epcis:eventTime "2025-12-01T00:00:00Z"^^xsd:dateTime ;
            epcis:action "ADD" ;
            epcis:bizStep cbv:BizStep-commissioning ;
            epcis:disposition cbv:Disp-active ;
            epcis:readPoint <urn:epc:id:sgln:1234567.12345.1> ;
            epcis:bizLocation <urn:epc:id:sgln:1234567.12345.0> .
    "#
}

fn synthesized_pharma_storage_event() -> &'static str {
    r#"
        @prefix ex: <http://example.org/> .
        @prefix pharma: <http://provchain.org/pharma#> .
        @prefix trace: <http://provchain.org/trace#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        ex:storageCheck-54868-6180-0 a pharma:PharmaceuticalStorage ;
            trace:temperature "5.2"^^xsd:decimal ;
            trace:humidity "55.0"^^xsd:decimal ;
            pharma:lightProtection "true"^^xsd:boolean ;
            pharma:controlledSubstance "false"^^xsd:boolean ;
            pharma:securityLevel "High" ;
            trace:recordedAt "2026-02-15T00:00:00Z"^^xsd:dateTime .
    "#
}

fn uht_ontology_config() -> OntologyConfig {
    let config = Config::default();
    OntologyConfig::new(
        Some("src/semantic/ontologies/uht_manufacturing.owl".to_string()),
        &config,
    )
    .expect("uht ontology config should load")
}

fn healthcare_ontology_config() -> OntologyConfig {
    let config = Config::default();
    OntologyConfig::new(
        Some("src/semantic/ontologies/healthcare.owl".to_string()),
        &config,
    )
    .expect("healthcare ontology config should load")
}

fn pharmaceutical_ontology_config() -> OntologyConfig {
    let config = Config::default();
    OntologyConfig::new(
        Some("src/semantic/ontologies/pharmaceutical.owl".to_string()),
        &config,
    )
    .expect("pharmaceutical ontology config should load")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn temp_path(name: &str, extension: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{name}_{unique}.{extension}"))
}

fn run_python_script(script: &Path, args: &[&str]) {
    let output = Command::new("python3")
        .arg(script)
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("python script should run");

    assert!(
        output.status.success(),
        "script {:?} failed: stdout={} stderr={}",
        script,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn normalizer_snapshot_id(script: &Path) -> &'static str {
    match script.file_name().and_then(|name| name.to_str()) {
        Some("normalize_fooddata_central.py") => "fooddata_central_sample_2026_03_10",
        Some("normalize_accessgudid.py") => "accessgudid_sample_2026_03_10",
        Some("normalize_openfda_drug_shortages.py") => "openfda_drug_shortages_sample_2026_03_10",
        Some("normalize_openfda_drug_ndc.py") => "openfda_drug_ndc_sample_2026_03_10",
        _ => "dataset_snapshot_2026_03_10",
    }
}

#[test]
fn test_synthesized_uht_product_event_passes_ontology_admission() {
    let ontology_config = uht_ontology_config();
    let mut blockchain =
        Blockchain::new_with_ontology(ontology_config).expect("ontology-backed blockchain");

    let initial_length = blockchain.chain.len();
    let result = blockchain.add_block(synthesized_uht_product_event().to_string());

    assert!(result.is_ok(), "synthesized uht event should pass");
    assert_eq!(blockchain.chain.len(), initial_length + 1);
}

#[test]
fn test_synthesized_uht_epcis_event_passes_ontology_admission() {
    let ontology_config = uht_ontology_config();
    let mut blockchain =
        Blockchain::new_with_ontology(ontology_config).expect("ontology-backed blockchain");

    let initial_length = blockchain.chain.len();
    let result = blockchain.add_block(synthesized_uht_epcis_event().to_string());

    assert!(result.is_ok(), "synthesized uht epcis event should pass");
    assert_eq!(blockchain.chain.len(), initial_length + 1);
}

#[test]
fn test_synthesized_healthcare_device_event_passes_ontology_admission() {
    let ontology_config = healthcare_ontology_config();
    let mut blockchain =
        Blockchain::new_with_ontology(ontology_config).expect("ontology-backed blockchain");

    let initial_length = blockchain.chain.len();
    let result = blockchain.add_block(synthesized_healthcare_device_event().to_string());

    assert!(result.is_ok(), "synthesized healthcare event should pass");
    assert_eq!(blockchain.chain.len(), initial_length + 1);
}

#[test]
fn test_synthesized_pharma_storage_event_passes_ontology_admission() {
    let ontology_config = pharmaceutical_ontology_config();
    let mut blockchain =
        Blockchain::new_with_ontology(ontology_config).expect("ontology-backed blockchain");

    let initial_length = blockchain.chain.len();
    let result = blockchain.add_block(synthesized_pharma_storage_event().to_string());

    assert!(result.is_ok(), "synthesized pharma storage event should pass");
    assert_eq!(blockchain.chain.len(), initial_length + 1);
}

#[test]
fn test_ontology_package_emitters_produce_admissible_payloads() {
    let repo = repo_root();

    let cases = [
        (
            repo.join("scripts/data_normalization/normalize_fooddata_central.py"),
            repo.join("config/datasets/raw_examples/fooddata_central_sample.json"),
            "uht",
            uht_ontology_config(),
        ),
        (
            repo.join("scripts/data_normalization/normalize_fooddata_central.py"),
            repo.join("config/datasets/raw_examples/fooddata_central_sample.json"),
            "uht_epcis",
            uht_ontology_config(),
        ),
        (
            repo.join("scripts/data_normalization/normalize_accessgudid.py"),
            repo.join("config/datasets/raw_examples/accessgudid_sample.csv"),
            "healthcare_device",
            healthcare_ontology_config(),
        ),
        (
            repo.join("scripts/data_normalization/normalize_openfda_drug_shortages.py"),
            repo.join("config/datasets/raw_examples/openfda_drug_shortages_sample.json"),
            "pharma_storage",
            pharmaceutical_ontology_config(),
        ),
    ];

    for (normalizer, raw_input, package, ontology_config) in cases {
        let normalized_path = temp_path(package, "normalized.json");
        let turtle_path = temp_path(package, "ttl");

        run_python_script(
            &normalizer,
            &[
                "--input",
                raw_input.to_str().expect("utf-8 path"),
                "--output",
                normalized_path.to_str().expect("utf-8 path"),
                "--snapshot-id",
                normalizer_snapshot_id(&normalizer),
            ],
        );
        run_python_script(
            &repo.join("scripts/data_projection/emit_ontology_package_turtle.py"),
            &[
                "--package",
                package,
                "--input",
                normalized_path.to_str().expect("utf-8 path"),
                "--output",
                turtle_path.to_str().expect("utf-8 path"),
            ],
        );

        let payload = fs::read_to_string(&turtle_path).expect("emitted turtle should exist");
        let mut blockchain =
            Blockchain::new_with_ontology(ontology_config).expect("ontology-backed blockchain");

        let initial_length = blockchain.chain.len();
        let result = blockchain.add_block(payload);

        assert!(result.is_ok(), "package {package} payload should pass");
        assert_eq!(blockchain.chain.len(), initial_length + 1);

        let _ = fs::remove_file(normalized_path);
        let _ = fs::remove_file(turtle_path);
    }
}
