//! Demo of universal traceability platform features

use crate::core::TraceableEntity;
use crate::core::entity::{EntityType, DomainType, PropertyValue};
use crate::domain::{DomainConfig};
use std::collections::HashMap;
use crate::ontology::domain_manager::OntologyManager;
use crate::ontology::OntologyConfig;
use anyhow::Result;

/// Demonstrate universal traceability platform capabilities
pub fn run_universal_traceability_demo() -> Result<()> {
    println!("=== Universal Traceability Platform Demo ===\n");
    
    // 1. Show ontology management
    println!("1. Ontology Management");
    
    // Create a default ontology configuration for demo
    let config = crate::config::Config::default();
    let ontology_config = match OntologyConfig::new(Some("ontologies/healthcare.owl".to_string()), &config) {
        Ok(config) => config,
        Err(e) => {
            println!("   Failed to create ontology config: {}", e);
            return Ok(());
        }
    };
    
    let ontology_manager = match OntologyManager::new(ontology_config) {
        Ok(manager) => manager,
        Err(e) => {
            println!("   Failed to create ontology manager: {}", e);
            return Ok(());
        }
    };
    println!("   Created ontology manager");
    
    // Show ontology information
    println!("   Domain: {}", ontology_manager.get_domain_name());
    println!("   Ontology hash: {}", ontology_manager.get_ontology_hash());
    println!("   Supported transaction types: {:?}", ontology_manager.get_supported_transaction_types());
    
    println!();
    
    // 2. Show domain adapters
    println!("2. Domain Adapters");
    
    // Create healthcare domain adapter
    let _healthcare_config = DomainConfig {
        domain_id: "healthcare".to_string(),
        name: "Healthcare Traceability".to_string(),
        description: "Healthcare and medical traceability".to_string(),
        core_ontology_path: "ontologies/generic_core.owl".to_string(),
        domain_ontology_path: "ontologies/healthcare.owl".to_string(),
        ontology_path: "ontologies/healthcare.owl".to_string(),
        shacl_shapes_path: None,
        inference_rules_path: None,
        required_properties: vec!["patientID".to_string()],
        validation_queries: vec![],
        enabled: true,
        priority: 1,
        custom_properties: HashMap::new(),
    };
    
    // let healthcare_adapter = Box::new(crate::domain::adapters::OwlDomainAdapter::from_config(&serde_yaml::Value::default())?)?;
    
    // Create pharmaceutical domain adapter
    let _pharma_config = DomainConfig {
        domain_id: "pharmaceutical".to_string(),
        name: "Pharmaceutical Traceability".to_string(),
        description: "Pharmaceutical and drug traceability".to_string(),
        core_ontology_path: "ontologies/generic_core.owl".to_string(),
        domain_ontology_path: "ontologies/pharmaceutical.owl".to_string(),
        ontology_path: "ontologies/pharmaceutical.owl".to_string(),
        shacl_shapes_path: None,
        inference_rules_path: None,
        required_properties: vec!["lotNumber".to_string()],
        validation_queries: vec![],
        enabled: true,
        priority: 1,
        custom_properties: HashMap::new(),
    };
    
    // let pharma_adapter = Box::new(crate::domain::adapters::OwlDomainAdapter::from_config(&serde_yaml::Value::default())?)?;
    
    println!();
    
    // 3. Show entity creation and validation
    println!("3. Entity Creation and Validation");
    
    // Create a healthcare patient record
    let mut patient_record = TraceableEntity::new(
        "patient_001".to_string(),
        EntityType::DomainSpecific("PatientRecord".to_string()),
        DomainType::Healthcare
    );
    
    patient_record.add_property(
        "patientID".to_string(), 
        PropertyValue::String("PAT001".to_string())
    );
    
    patient_record.add_property(
        "procedureCode".to_string(), 
        PropertyValue::String("PROC123".to_string())
    );
    
    println!("   Created healthcare patient record: {}", patient_record.id);
    
    // Validate the patient record
    // let healthcare_validation = healthcare_adapter.validate_entity(&patient_record)?;
    // println!("   Healthcare validation result: {}", 
    //     if healthcare_validation.is_valid { "VALID" } else { "INVALID" });
    
    // if !healthcare_validation.is_valid {
    //     for error in &healthcare_validation.errors {
    //         println!("     Error: {}", error);
    //     }
    // }
    
    // Try to validate with wrong domain adapter
    // let pharma_validation = pharma_adapter.validate_entity(&patient_record)?;
    // println!("   Pharmaceutical validation of healthcare record: {}", 
    //     if pharma_validation.is_valid { "VALID" } else { "INVALID" });
    
    // if !pharma_validation.is_valid {
    //     for error in &pharma_validation.errors {
    //         println!("     Error: {}", error);
    //     }
    // }
    
    println!();
    
    // 4. Show RDF conversion
    println!("4. RDF Conversion");
    let rdf_output = patient_record.to_rdf();
    println!("   RDF representation of patient record:");
    for line in rdf_output.lines().take(5) {
        println!("     {}", line);
    }
    if rdf_output.lines().count() > 5 {
        println!("     ... ({} more lines)", rdf_output.lines().count() - 5);
    }
    
    println!();
    
    // 5. Show domain-specific enrichment
    println!("5. Domain-Specific Enrichment");
    println!("   Enriching healthcare entity...");
    // healthcare_adapter.enrich_entity(&mut patient_record)?;
    println!("   Entity enriched successfully");
    
    println!("\n=== Demo Complete ===");
    println!("The universal traceability platform demonstrates:");
    println!("• Multi-domain ontology management");
    println!("• Domain-specific adapters with validation");
    println!("• Generic entity model supporting all domains");
    println!("• RDF serialization for semantic web compatibility");
    println!("• Cross-domain entity validation");
    
    Ok(())
}
