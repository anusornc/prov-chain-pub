//! Supply chain domain adapter.
//!
//! This adapter provides lightweight domain-specific validation and enrichment
//! for the generic supply chain traceability model.

use crate::domain::plugin::{
    DomainConfig, DomainPlugin, EntityData, ProcessedEntity, ValidationResult,
};
use anyhow::Result;
use std::collections::HashMap;

/// Supply chain domain adapter.
pub struct SupplyChainAdapter {
    config: DomainConfig,
    validation_rules: HashMap<String, String>,
    domain_properties: Vec<String>,
}

impl SupplyChainAdapter {
    /// Create a supply chain adapter from configuration.
    pub fn from_config(_config: &serde_yaml::Value) -> Result<Self> {
        let domain_config = DomainConfig {
            domain_id: "supplychain".to_string(),
            name: "Supply Chain Traceability".to_string(),
            description: "General supply chain and manufacturing traceability".to_string(),
            core_ontology_path: "src/semantic/ontologies/generic_core.owl".to_string(),
            domain_ontology_path: "src/semantic/ontologies/generic_core.owl".to_string(),
            ontology_path: "src/semantic/ontologies/generic_core.owl".to_string(),
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
            "ProductBatch".to_string(),
            "Must have valid batch ID and origin information".to_string(),
        );
        self.validation_rules.insert(
            "ProcessingActivity".to_string(),
            "Must have timestamp and processing parameters".to_string(),
        );
        self.validation_rules.insert(
            "TransportActivity".to_string(),
            "Must have origin, destination, and transport parameters".to_string(),
        );
        self.validation_rules.insert(
            "QualityCheck".to_string(),
            "Must have timestamp and quality parameters".to_string(),
        );
        self.validation_rules.insert(
            "Supplier".to_string(),
            "Must have valid supplier information".to_string(),
        );
    }

    fn initialize_domain_properties(&mut self) {
        self.domain_properties.extend([
            "hasBatchID".to_string(),
            "originFarm".to_string(),
            "harvestDate".to_string(),
            "processingDate".to_string(),
            "transportDate".to_string(),
            "qualityCheckDate".to_string(),
            "hasTemperature".to_string(),
            "hasHumidity".to_string(),
            "supplierName".to_string(),
            "supplierAddress".to_string(),
        ]);
    }

    fn validate_product_batch(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        if !entity_data.properties.contains_key("hasBatchID") {
            return Ok(ValidationResult::Invalid(
                "Product batch must have hasBatchID".to_string(),
            ));
        }

        if !entity_data.properties.contains_key("originFarm") {
            return Ok(ValidationResult::Invalid(
                "Product batch must have originFarm".to_string(),
            ));
        }

        Ok(ValidationResult::Valid)
    }

    fn validate_processing_activity(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        if !entity_data.properties.contains_key("processingDate") {
            return Ok(ValidationResult::Invalid(
                "Processing activity must have processingDate".to_string(),
            ));
        }

        Ok(ValidationResult::Valid)
    }

    fn validate_transport_activity(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        if !entity_data.properties.contains_key("transportDate") {
            return Ok(ValidationResult::Invalid(
                "Transport activity must have transportDate".to_string(),
            ));
        }

        Ok(ValidationResult::Valid)
    }

    fn validate_quality_check(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        if !entity_data.properties.contains_key("qualityCheckDate") {
            return Ok(ValidationResult::Invalid(
                "Quality check must have qualityCheckDate".to_string(),
            ));
        }

        Ok(ValidationResult::Valid)
    }

    fn validate_supplier(&self, entity_data: &EntityData) -> Result<ValidationResult> {
        if !entity_data.properties.contains_key("supplierName") {
            return Ok(ValidationResult::Invalid(
                "Supplier must have supplierName".to_string(),
            ));
        }

        Ok(ValidationResult::Valid)
    }

    fn enrich_supply_chain_data(&self, entity_data: &EntityData) -> Result<String> {
        let mut enriched_data = entity_data.data.clone();
        enriched_data.push_str("\n# Supply chain context\n");
        enriched_data.push_str("@prefix supplychain: <http://provchain.org/supplychain#> .\n");
        enriched_data.push_str(&format!(
            "# Enriched by supply chain domain adapter\n# Entity: {}\n# Type: {}\n",
            entity_data.entity_id, entity_data.entity_type
        ));
        Ok(enriched_data)
    }
}

impl DomainPlugin for SupplyChainAdapter {
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
            "ProductBatch"
                | "ProcessingActivity"
                | "TransportActivity"
                | "QualityCheck"
                | "Supplier"
                | "EnvironmentalCondition"
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
            "ProductBatch" => self.validate_product_batch(entity_data),
            "ProcessingActivity" => self.validate_processing_activity(entity_data),
            "TransportActivity" => self.validate_transport_activity(entity_data),
            "QualityCheck" => self.validate_quality_check(entity_data),
            "Supplier" => self.validate_supplier(entity_data),
            _ => Ok(ValidationResult::Valid),
        }
    }

    fn process_entity(&self, entity_data: &EntityData) -> Result<ProcessedEntity> {
        Ok(ProcessedEntity {
            entity_id: entity_data.entity_id.clone(),
            entity_type: entity_data.entity_type.clone(),
            processed_data: self.enrich_supply_chain_data(entity_data)?,
            domain_context: "supplychain".to_string(),
        })
    }
}
