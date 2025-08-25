//! Pharmaceutical domain adapter
//!
//! This module provides the pharmaceutical domain adapter that extends
//! the generic traceability system with pharmaceutical specific validation
//! and processing capabilities.

use crate::domain::plugin::{DomainPlugin, DomainConfig, ValidationResult, ProcessedEntity, EntityData};
use anyhow::Result;
use std::collections::HashMap;
use tracing::{info, warn, debug};

/// Pharmaceutical domain adapter
pub struct PharmaceuticalAdapter {
    config: DomainConfig,
    validation_rules: HashMap<String, String>,
    domain_properties: Vec<String>,
}

impl PharmaceuticalAdapter {
    /// Create from configuration
    pub fn from_config(_config: &serde_yaml::Value) -> Result<Self> {
        let domain_config = DomainConfig {
            domain_id: "pharmaceutical".to_string(),
            name: "Pharmaceutical Traceability".to_string(),
            description: "Pharmaceutical and drug traceability".to_string(),
            core_ontology_path: "ontologies/generic_core.owl".to_string(),
            domain_ontology_path: "ontologies/pharmaceutical.owl".to_string(),
            ontology_path: "ontologies/pharmaceutical.owl".to_string(),
            shacl_shapes_path: None,
            inference_rules_path: None,
            required_properties: vec![],
            validation_queries: vec![],
            enabled: true,
            priority: 1,
            custom_properties: HashMap::new(),
        };
        
        let mut adapter = PharmaceuticalAdapter {
            config: domain_config,
            validation_rules: HashMap::new(),
            domain_properties: Vec::new(),
        };
        
        adapter.initialize_validation_rules();
        adapter.initialize_domain_properties();
        
        Ok(adapter)
    }
    
    /// Initialize validation rules
    fn initialize_validation_rules(&mut self) {
        self.validation_rules.insert(
            "DrugBatch".to_string(),
            "Must have valid batch ID and drug information".to_string()
        );
        self.validation_rules.insert(
            "ClinicalTrial".to_string(),
            "Must have trial ID and regulatory information".to_string()
        );
        self.validation_rules.insert(
            "RegulatoryApproval".to_string(),
            "Must have approval ID and regulatory body information".to_string()
        );
        self.validation_rules.insert(
            "ManufacturingProcess".to_string(),
            "Must have process ID and manufacturing parameters".to_string()
        );
        self.validation_rules.insert(
            "QualityControl".to_string(),
            "Must have QC ID and quality parameters".to_string()
        );
    }
    
    /// Initialize domain properties
    fn initialize_domain_properties(&mut self) {
        self.domain_properties.extend(vec![
            "hasDrugID".to_string(),
            "hasBatchNumber".to_string(),
            "manufacturingDate".to_string(),
            "expiryDate".to_string(),
            "clinicalTrialID".to_string(),
            "regulatoryBody".to_string(),
            "approvalDate".to_string(),
            "qcResult".to_string(),
            "qcDate".to_string(),
            "drugName".to_string(),
            "activeIngredient".to_string(),
            "dosageForm".to_string(),
        ]);
    }
}

impl DomainPlugin for PharmaceuticalAdapter {
    fn domain_id(&self) -> &str {
        &self.config.domain_id
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn description(&self) -> &str {
        &self.config.description
    }
    
    fn is_valid_entity_type(&self, entity_type: &str) -> bool {
        matches!(entity_type, 
            "DrugBatch" | "ClinicalTrial" | "RegulatoryApproval" | 
            "ManufacturingProcess" | "QualityControl" | "PharmaceuticalIngredient"
        )
    }
    
    fn validation_rules(&self) -> &HashMap<String, String> {
        &self.validation_rules
    }
    
    fn domain_properties(&self) -> &Vec<String> {
        &self.domain_properties
    }
    
    fn initialize(&mut self, _config: &DomainConfig) -> Result<()> {
        // Already initialized in from_config
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<()> {
        // Cleanup any resources
        Ok(())
    }
    
    fn validate_entity(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        // Validate based on entity type
        match entity_data.entity_type.as_str() {
            "DrugBatch" => self.validate_drug_batch(entity_data),
            "ClinicalTrial" => self.validate_clinical_trial(entity_data),
            "RegulatoryApproval" => self.validate_regulatory_approval(entity_data),
            "ManufacturingProcess" => self.validate_manufacturing_process(entity_data),
            "QualityControl" => self.validate_quality_control(entity_data),
            "PharmaceuticalIngredient" => self.validate_pharmaceutical_ingredient(entity_data),
            _ => Ok(ValidationResult::Valid), // Allow unknown types with warning
        }
    }
    
    fn process_entity(&self, entity_data: &EntityData) -> Result<ProcessedEntity> {
        // Process entity for pharmaceutical domain
        Ok(ProcessedEntity {
            entity_id: entity_data.entity_id.clone(),
            entity_type: entity_data.entity_type.clone(),
            processed_data: self.enrich_pharmaceutical_data(entity_data)?,
            domain_context: "pharmaceutical".to_string(),
        })
    }
}

impl PharmaceuticalAdapter {
    /// Validate drug batch
    fn validate_drug_batch(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        // Check required properties
        if !entity_data.properties.contains_key("hasDrugID") {
            return Ok(ValidationResult::Invalid("Drug batch must have hasDrugID".to_string()));
        }
        
        if !entity_data.properties.contains_key("hasBatchNumber") {
            return Ok(ValidationResult::Invalid("Drug batch must have hasBatchNumber".to_string()));
        }
        
        Ok(ValidationResult::Valid)
    }
    
    /// Validate clinical trial
    fn validate_clinical_trial(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        // Check required properties
        if !entity_data.properties.contains_key("clinicalTrialID") {
            return Ok(ValidationResult::Invalid("Clinical trial must have clinicalTrialID".to_string()));
        }
        
        Ok(ValidationResult::Valid)
    }
    
    /// Validate regulatory approval
    fn validate_regulatory_approval(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        // Check required properties
        if !entity_data.properties.contains_key("regulatoryBody") {
            return Ok(ValidationResult::Invalid("Regulatory approval must have regulatoryBody".to_string()));
        }
        
        Ok(ValidationResult::Valid)
    }
    
    /// Validate manufacturing process
    fn validate_manufacturing_process(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        // Check required properties
        if !entity_data.properties.contains_key("manufacturingDate") {
            return Ok(ValidationResult::Invalid("Manufacturing process must have manufacturingDate".to_string()));
        }
        
        Ok(ValidationResult::Valid)
    }
    
    /// Validate quality control
    fn validate_quality_control(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        // Check required properties
        if !entity_data.properties.contains_key("qcDate") {
            return Ok(ValidationResult::Invalid("Quality control must have qcDate".to_string()));
        }
        
        Ok(ValidationResult::Valid)
    }
    
    /// Validate pharmaceutical ingredient
    fn validate_pharmaceutical_ingredient(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        // Check required properties
        if !entity_data.properties.contains_key("activeIngredient") {
            return Ok(ValidationResult::Invalid("Pharmaceutical ingredient must have activeIngredient".to_string()));
        }
        
        Ok(ValidationResult::Valid)
    }
    
    /// Enrich pharmaceutical data
    fn enrich_pharmaceutical_data(&self, entity_data: &EntityData) -> Result<String> {
        // Add pharmaceutical specific enrichment
        let mut enriched_data = entity_data.data.clone();
        
        // Add pharmaceutical context
        enriched_data.push_str("\n# Pharmaceutical context\n");
        enriched_data.push_str("@prefix pharmaceutical: <http://provchain.org/pharmaceutical#> .\n");
        
        // Add domain-specific annotations
        enriched_data.push_str(&format!(
            "# Enriched by pharmaceutical domain adapter\n# Entity: {}\n# Type: {}\n",
            entity_data.entity_id, entity_data.entity_type
        ));
        
        Ok(enriched_data)
    }
}