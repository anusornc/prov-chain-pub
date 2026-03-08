//! Healthcare domain adapter.
//!
//! This adapter provides lightweight domain-specific validation and enrichment
//! for healthcare and medical traceability scenarios.

use crate::domain::plugin::{
    DomainConfig, DomainPlugin, EntityData, ProcessedEntity, ValidationResult,
};
use anyhow::Result;
use std::collections::HashMap;

/// Healthcare domain adapter.
pub struct HealthcareAdapter {
    config: DomainConfig,
    validation_rules: HashMap<String, String>,
    domain_properties: Vec<String>,
}

impl HealthcareAdapter {
    /// Create a healthcare adapter from configuration.
    pub fn from_config(_config: &serde_yaml::Value) -> Result<Self> {
        let domain_config = DomainConfig {
            domain_id: "healthcare".to_string(),
            name: "Healthcare Traceability".to_string(),
            description: "Healthcare and medical traceability".to_string(),
            core_ontology_path: "src/semantic/ontologies/generic_core.owl".to_string(),
            domain_ontology_path: "src/semantic/ontologies/healthcare.owl".to_string(),
            ontology_path: "src/semantic/ontologies/healthcare.owl".to_string(),
            shacl_shapes_path: None,
            inference_rules_path: None,
            required_properties: vec![],
            validation_queries: vec![],
            enabled: true,
            priority: 1,
            custom_properties: HashMap::new(),
        };

        let mut adapter = Self {
            config: domain_config,
            validation_rules: HashMap::new(),
            domain_properties: Vec::new(),
        };

        adapter.initialize_validation_rules();
        adapter.initialize_domain_properties();

        Ok(adapter)
    }

    fn initialize_validation_rules(&mut self) {
        self.validation_rules.insert(
            "PatientRecord".to_string(),
            "Must have valid patient ID and medical information".to_string(),
        );
        self.validation_rules.insert(
            "Treatment".to_string(),
            "Must have timestamp and treatment parameters".to_string(),
        );
        self.validation_rules.insert(
            "MedicalDevice".to_string(),
            "Must have valid serial number and device information".to_string(),
        );
        self.validation_rules.insert(
            "HealthcareProvider".to_string(),
            "Must have valid provider information".to_string(),
        );
        self.validation_rules.insert(
            "ClinicalTrial".to_string(),
            "Must have trial ID and regulatory information".to_string(),
        );
    }

    fn initialize_domain_properties(&mut self) {
        self.domain_properties.extend([
            "hasPatientID".to_string(),
            "diagnosisDate".to_string(),
            "treatmentOutcome".to_string(),
            "medicalDeviceSerial".to_string(),
            "procedureDate".to_string(),
            "healthcareProviderName".to_string(),
            "healthcareProviderAddress".to_string(),
            "clinicalTrialID".to_string(),
            "regulatoryBody".to_string(),
            "approvalDate".to_string(),
            "medicationName".to_string(),
        ]);
    }

    fn validate_patient_record(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        if !entity_data.properties.contains_key("hasPatientID") {
            return Ok(ValidationResult::Invalid(
                "Patient record must have hasPatientID".to_string(),
            ));
        }

        Ok(ValidationResult::Valid)
    }

    fn validate_treatment(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        if !entity_data.properties.contains_key("procedureDate") {
            return Ok(ValidationResult::Invalid(
                "Treatment must have procedureDate".to_string(),
            ));
        }

        Ok(ValidationResult::Valid)
    }

    fn validate_medical_device(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        if !entity_data.properties.contains_key("medicalDeviceSerial") {
            return Ok(ValidationResult::Invalid(
                "Medical device must have medicalDeviceSerial".to_string(),
            ));
        }

        Ok(ValidationResult::Valid)
    }

    fn validate_healthcare_provider(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        if !entity_data
            .properties
            .contains_key("healthcareProviderName")
        {
            return Ok(ValidationResult::Invalid(
                "Healthcare provider must have healthcareProviderName".to_string(),
            ));
        }

        Ok(ValidationResult::Valid)
    }

    fn validate_clinical_trial(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        if !entity_data.properties.contains_key("clinicalTrialID") {
            return Ok(ValidationResult::Invalid(
                "Clinical trial must have clinicalTrialID".to_string(),
            ));
        }

        Ok(ValidationResult::Valid)
    }

    fn validate_medication(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        if !entity_data.properties.contains_key("medicationName") {
            return Ok(ValidationResult::Invalid(
                "Medication must have medicationName".to_string(),
            ));
        }

        Ok(ValidationResult::Valid)
    }

    fn enrich_healthcare_data(&self, entity_data: &EntityData) -> Result<String> {
        let mut enriched_data = entity_data.data.clone();
        enriched_data.push_str("\n# Healthcare context\n");
        enriched_data.push_str("@prefix healthcare: <http://provchain.org/healthcare#> .\n");
        enriched_data.push_str(&format!(
            "# Enriched by healthcare domain adapter\n# Entity: {}\n# Type: {}\n",
            entity_data.entity_id, entity_data.entity_type
        ));
        Ok(enriched_data)
    }
}

impl DomainPlugin for HealthcareAdapter {
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
        matches!(
            entity_type,
            "PatientRecord"
                | "Treatment"
                | "MedicalDevice"
                | "HealthcareProvider"
                | "ClinicalTrial"
                | "Medication"
        )
    }

    fn validation_rules(&self) -> &HashMap<String, String> {
        &self.validation_rules
    }

    fn domain_properties(&self) -> &Vec<String> {
        &self.domain_properties
    }

    fn initialize(&mut self, _config: &DomainConfig) -> Result<()> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    fn validate_entity(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        match entity_data.entity_type.as_str() {
            "PatientRecord" => self.validate_patient_record(entity_data),
            "Treatment" => self.validate_treatment(entity_data),
            "MedicalDevice" => self.validate_medical_device(entity_data),
            "HealthcareProvider" => self.validate_healthcare_provider(entity_data),
            "ClinicalTrial" => self.validate_clinical_trial(entity_data),
            "Medication" => self.validate_medication(entity_data),
            _ => Ok(ValidationResult::Valid),
        }
    }

    fn process_entity(&self, entity_data: &EntityData) -> Result<ProcessedEntity> {
        Ok(ProcessedEntity {
            entity_id: entity_data.entity_id.clone(),
            entity_type: entity_data.entity_type.clone(),
            processed_data: self.enrich_healthcare_data(entity_data)?,
            domain_context: "healthcare".to_string(),
        })
    }
}
