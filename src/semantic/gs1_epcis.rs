//! GS1 EPCIS Integration for ProvChain UHT Supply Chain Demo
//!
//! This module provides integration with GS1 EPCIS (Electronic Product Code
//! Information Services) standards for supply chain traceability.
//!
//! ## Features
//! - Create EPCIS-compliant events for UHT supply chain
//! - Export events in EPCIS JSON-LD/Turtle format
//! - Use standard CBV business steps and dispositions
//!
//! ## Standards Compliance
//! - EPCIS 2.0 (ISO/IEC 19987:2024)
//! - CBV 2.0 (ISO/IEC 19988:2024)
//! - GS1 Web Vocabulary

use anyhow::Result;
use std::collections::HashMap;

/// GS1 EPCIS namespace constants
pub mod namespaces {
    pub const EPCIS: &str = "https://ref.gs1.org/epcis/";
    pub const CBV: &str = "https://ref.gs1.org/cbv/";
    pub const GS1: &str = "https://gs1.org/voc/";
    pub const UHT: &str = "https://provchain.org/uht/";
}

/// EPCIS Event Builder for creating UHT supply chain events
pub struct EpcisEventBuilder {
    event_type: EpcisEventType,
    properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy)]
pub enum EpcisEventType {
    ObjectEvent,
    AggregationEvent,
    AssociationEvent,
    TransactionEvent,
    TransformationEvent,
}

impl EpcisEventBuilder {
    /// Create a new event builder
    pub fn new(event_type: EpcisEventType) -> Self {
        Self {
            event_type,
            properties: HashMap::new(),
        }
    }

    /// Set event time
    pub fn event_time(mut self, timestamp: &str) -> Self {
        self.properties
            .insert("eventTime".to_string(), timestamp.to_string());
        self
    }

    /// Set action (ADD, OBSERVE, DELETE)
    pub fn action(mut self, action: &str) -> Self {
        self.properties
            .insert("action".to_string(), action.to_string());
        self
    }

    /// Set business step
    pub fn biz_step(mut self, step: &str) -> Self {
        self.properties
            .insert("bizStep".to_string(), step.to_string());
        self
    }

    /// Set disposition
    pub fn disposition(mut self, disposition: &str) -> Self {
        self.properties
            .insert("disposition".to_string(), disposition.to_string());
        self
    }

    /// Set read point (location where event was captured)
    pub fn read_point(mut self, read_point: &str) -> Self {
        self.properties
            .insert("readPoint".to_string(), read_point.to_string());
        self
    }

    /// Set business location
    pub fn biz_location(mut self, location: &str) -> Self {
        self.properties
            .insert("bizLocation".to_string(), location.to_string());
        self
    }

    /// Set batch number (UHT extension)
    pub fn batch_number(mut self, batch: &str) -> Self {
        self.properties
            .insert("batchNumber".to_string(), batch.to_string());
        self
    }

    /// Build the event as Turtle RDF
    pub fn build_turtle(&self, event_id: &str) -> String {
        let type_uri = match self.event_type {
            EpcisEventType::ObjectEvent => "epcis:ObjectEvent",
            EpcisEventType::AggregationEvent => "epcis:AggregationEvent",
            EpcisEventType::AssociationEvent => "epcis:AssociationEvent",
            EpcisEventType::TransactionEvent => "epcis:TransactionEvent",
            EpcisEventType::TransformationEvent => "epcis:TransformationEvent",
        };

        let mut turtle = format!("@prefix epcis: <{}> .\n", namespaces::EPCIS);
        turtle.push_str(&format!("@prefix uht: <{}> .\n", namespaces::UHT));
        turtle.push_str("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n");

        turtle.push_str(&format!("<{}> a {} ;\n", event_id, type_uri));

        for (key, value) in &self.properties {
            // Determine if property is from EPCIS or UHT namespace
            let is_uht_prop = matches!(
                key.as_str(),
                "batchNumber"
                    | "milkVolume"
                    | "collectionTemperature"
                    | "fatContent"
                    | "proteinContent"
                    | "pasteurizationTemperature"
                    | "holdingTime"
            );

            let prefix = if is_uht_prop { "uht" } else { "epcis" };

            // Check if value looks like a URI
            if value.starts_with("http://")
                || value.starts_with("https://")
                || value.starts_with("urn:")
            {
                turtle.push_str(&format!("    {}:{} <{}> ;\n", prefix, key, value));
            } else if value.parse::<f64>().is_ok() {
                turtle.push_str(&format!(
                    "    {}:{} \"{}\"^^xsd:decimal ;\n",
                    prefix, key, value
                ));
            } else {
                turtle.push_str(&format!("    {}:{} \"{}\" ;\n", prefix, key, value));
            }
        }

        // Remove trailing semicolon and add period
        if turtle.ends_with(";\n") {
            turtle.pop();
            turtle.pop();
            turtle.push_str(".\n");
        }

        turtle
    }

    /// Build the event as JSON-LD (EPCIS format)
    pub fn build_jsonld(&self, event_id: &str) -> serde_json::Value {
        let type_str = match self.event_type {
            EpcisEventType::ObjectEvent => "ObjectEvent",
            EpcisEventType::AggregationEvent => "AggregationEvent",
            EpcisEventType::AssociationEvent => "AssociationEvent",
            EpcisEventType::TransactionEvent => "TransactionEvent",
            EpcisEventType::TransformationEvent => "TransformationEvent",
        };

        let mut event = serde_json::json!({
            "@context": [
                "https://ref.gs1.org/standards/epcis/2.0.0/epcis-context.jsonld",
                {
                    "uht": "https://provchain.org/uht/"
                }
            ],
            "@id": event_id,
            "@type": type_str,
        });

        // Add all properties
        if let Some(obj) = event.as_object_mut() {
            for (key, value) in &self.properties {
                // Convert camelCase to proper JSON-LD property names
                let json_key = if matches!(key.as_str(), "batchNumber") {
                    format!("uht:{}", key)
                } else {
                    format!("epcis:{}", key)
                };
                obj.insert(json_key, serde_json::Value::String(value.clone()));
            }
        }

        event
    }
}

/// Predefined business steps from CBV
pub mod biz_steps {
    pub const COMMISSIONING: &str = "https://ref.gs1.org/cbv/BizStep-commissioning";
    pub const CREATING_CLASS_INSTANCE: &str =
        "https://ref.gs1.org/cbv/BizStep-creating_class_instance";
    pub const TRANSPORTING: &str = "https://ref.gs1.org/cbv/BizStep-transporting";
    pub const RECEIVING: &str = "https://ref.gs1.org/cbv/BizStep-receiving";
    pub const STORING: &str = "https://ref.gs1.org/cbv/BizStep-storing";
    pub const SHIPPING: &str = "https://ref.gs1.org/cbv/BizStep-shipping";
    pub const INSPECTING: &str = "https://ref.gs1.org/cbv/BizStep-inspecting";
    pub const SAMPLING: &str = "https://ref.gs1.org/cbv/BizStep-sampling";
    pub const PACKING: &str = "https://ref.gs1.org/cbv/BizStep-packing";
    pub const UNPACKING: &str = "https://ref.gs1.org/cbv/BizStep-unpacking";
    pub const RETAIL_SELLING: &str = "https://ref.gs1.org/cbv/BizStep-retail_selling";
    pub const SENSOR_REPORTING: &str = "https://ref.gs1.org/cbv/BizStep-sensor_reporting";
}

/// Predefined dispositions from CBV
pub mod dispositions {
    pub const ACTIVE: &str = "https://ref.gs1.org/cbv/Disp-active";
    pub const IN_TRANSIT: &str = "https://ref.gs1.org/cbv/Disp-in_transit";
    pub const IN_PROGRESS: &str = "https://ref.gs1.org/cbv/Disp-in_progress";
    pub const CONFORMANT: &str = "https://ref.gs1.org/cbv/Disp-conformant";
    pub const NON_CONFORMANT: &str = "https://ref.gs1.org/cbv/Disp-non_conformant";
    pub const EXPIRED: &str = "https://ref.gs1.org/cbv/Disp-expired";
    pub const DAMAGED: &str = "https://ref.gs1.org/cbv/Disp-damaged";
    pub const CONTAINER_CLOSED: &str = "https://ref.gs1.org/cbv/Disp-container_closed";
}

/// Helper function to create a complete EPCIS Document
pub fn create_epcis_document(events: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "@context": [
            "https://ref.gs1.org/standards/epcis/2.0.0/epcis-context.jsonld",
            {
                "uht": "https://provchain.org/uht/"
            }
        ],
        "@type": "EPCISDocument",
        "schemaVersion": "2.0",
        "creationDate": chrono::Utc::now().to_rfc3339(),
        "epcisBody": {
            "eventList": events
        }
    })
}

/// Generate example UHT supply chain events
pub fn generate_uht_supply_chain_events(batch_id: &str) -> Vec<serde_json::Value> {
    let now = chrono::Utc::now();

    vec![
        // Milk Collection
        EpcisEventBuilder::new(EpcisEventType::ObjectEvent)
            .event_time(&now.to_rfc3339())
            .action("ADD")
            .biz_step(biz_steps::COMMISSIONING)
            .disposition(dispositions::ACTIVE)
            .read_point("urn:epc:id:sgln:1234567.12345.1")
            .biz_location("urn:epc:id:sgln:1234567.12345.0")
            .batch_number(batch_id)
            .build_jsonld(&format!(
                "https://provchain.org/uht/{}/collection",
                batch_id
            )),
        // UHT Processing
        EpcisEventBuilder::new(EpcisEventType::TransformationEvent)
            .event_time(&now.to_rfc3339())
            .biz_step(biz_steps::CREATING_CLASS_INSTANCE)
            .disposition(dispositions::IN_PROGRESS)
            .read_point("urn:epc:id:sgln:1234567.12347.1")
            .biz_location("urn:epc:id:sgln:1234567.12347.0")
            .batch_number(batch_id)
            .build_jsonld(&format!(
                "https://provchain.org/uht/{}/processing",
                batch_id
            )),
        // Cold Chain Transport
        EpcisEventBuilder::new(EpcisEventType::ObjectEvent)
            .event_time(&now.to_rfc3339())
            .action("OBSERVE")
            .biz_step(biz_steps::TRANSPORTING)
            .disposition(dispositions::IN_TRANSIT)
            .read_point("urn:epc:id:sgln:1234567.12347.4")
            .biz_location("urn:epc:id:sgln:1234567.12347.0")
            .batch_number(batch_id)
            .build_jsonld(&format!("https://provchain.org/uht/{}/shipping", batch_id)),
        // Retail Receiving
        EpcisEventBuilder::new(EpcisEventType::ObjectEvent)
            .event_time(&now.to_rfc3339())
            .action("OBSERVE")
            .biz_step(biz_steps::RECEIVING)
            .disposition(dispositions::IN_PROGRESS)
            .read_point("urn:epc:id:sgln:1234567.12348.1")
            .biz_location("urn:epc:id:sgln:1234567.12348.0")
            .batch_number(batch_id)
            .build_jsonld(&format!("https://provchain.org/uht/{}/receiving", batch_id)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_builder_turtle() {
        let builder = EpcisEventBuilder::new(EpcisEventType::ObjectEvent)
            .action("ADD")
            .biz_step(biz_steps::COMMISSIONING)
            .batch_number("UHT-BATCH-001");

        let turtle = builder.build_turtle("event001");
        assert!(turtle.contains("epcis:ObjectEvent"));
        assert!(turtle.contains("epcis:action"));
        assert!(turtle.contains("uht:batchNumber"));
    }

    #[test]
    fn test_event_builder_jsonld() {
        let builder = EpcisEventBuilder::new(EpcisEventType::TransformationEvent)
            .action("OBSERVE")
            .batch_number("UHT-BATCH-001");

        let jsonld = builder.build_jsonld("event001");
        assert_eq!(jsonld["@type"], "TransformationEvent");
    }

    #[test]
    fn test_create_epcis_document() {
        let events = vec![EpcisEventBuilder::new(EpcisEventType::ObjectEvent)
            .action("ADD")
            .build_jsonld("event1")];

        let doc = create_epcis_document(events);
        assert_eq!(doc["@type"], "EPCISDocument");
        assert_eq!(doc["schemaVersion"], "2.0");
        assert!(doc["epcisBody"]["eventList"].as_array().unwrap().len() > 0);
    }
}
