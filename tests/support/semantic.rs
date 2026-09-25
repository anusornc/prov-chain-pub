//! Shared semantic-package fixture for durable-ledger integration tests.

use std::fs;
use std::sync::OnceLock;

use provchain_org::ledger::LedgerProfile;
use provchain_org::ontology::{
    ActivatedSemanticPackage, OntologyPackageManifest, SEMANTIC_EXECUTION_PROFILE_V1,
};
use tempfile::tempdir;

/// Stable ontology-package identifier used by ledger integration tests.
pub const TEST_PACKAGE_ID: &str = "provchain.test.semantic";
/// Stable ontology-package version used by ledger integration tests.
pub const TEST_PACKAGE_VERSION: &str = "1.0.0";

/// Return the process-wide activated semantic test package.
pub fn semantic_package() -> ActivatedSemanticPackage {
    static PACKAGE: OnceLock<ActivatedSemanticPackage> = OnceLock::new();
    PACKAGE.get_or_init(activate).clone()
}

/// Bind the shared semantic test-package identity to a ledger profile.
pub fn bind_profile(profile: LedgerProfile) -> LedgerProfile {
    let package = semantic_package();
    profile.with_semantic_package(
        package.package_id(),
        package.package_version(),
        package.package_hash(),
    )
}

fn activate() -> ActivatedSemanticPackage {
    let directory = tempdir().expect("semantic test-package directory");
    let core = directory.path().join("core.ttl");
    let domain = directory.path().join("domain.ttl");
    let core_shapes = directory.path().join("core-shapes.ttl");
    let domain_shapes = directory.path().join("domain-shapes.ttl");
    let ontology = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
owl:Thing a owl:Class .
"#;
    let shapes = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
<urn:provchain:test:AllSubjectsShape> a sh:NodeShape ;
    sh:targetClass owl:Thing .
"#;
    for (path, contents) in [
        (&core, ontology),
        (&domain, ontology),
        (&core_shapes, shapes),
        (&domain_shapes, shapes),
    ] {
        fs::write(path, contents).expect("write semantic test-package asset");
    }
    let manifest = OntologyPackageManifest {
        package_id: TEST_PACKAGE_ID.to_string(),
        package_version: TEST_PACKAGE_VERSION.to_string(),
        core_ontology_path: core.to_string_lossy().into_owned(),
        domain_ontology_path: domain.to_string_lossy().into_owned(),
        core_shacl_path: core_shapes.to_string_lossy().into_owned(),
        domain_shacl_path: domain_shapes.to_string_lossy().into_owned(),
        mappings: vec![],
        validation_mode: "strict".to_string(),
        semantic_execution_profile_id: SEMANTIC_EXECUTION_PROFILE_V1.to_string(),
        package_hash: None,
    };
    ActivatedSemanticPackage::activate(&manifest).expect("activate semantic test package")
}
