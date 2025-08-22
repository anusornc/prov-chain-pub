//! Generic entity model for universal traceability platform

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// A traceable entity that can represent anything across domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceableEntity {
    /// Unique identifier for the entity
    pub id: String,
    
    /// Type of entity (domain-specific)
    pub entity_type: EntityType,
    
    /// Domain this entity belongs to
    pub domain: DomainType,
    
    /// Key-value properties of the entity
    pub properties: HashMap<String, PropertyValue>,
    
    /// Relationships to other entities
    pub relationships: Vec<EntityRelationship>,
    
    /// Metadata about the entity
    pub metadata: EntityMetadata,
    
    /// Provenance information
    pub provenance: ProvenanceInfo,
}

/// Types of entities that can be traced
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    /// Physical products or materials
    Product,
    
    /// Components or parts
    Component,
    
    /// Processes or activities
    Process,
    
    /// People involved in the trace
    Person,
    
    /// Organizations or companies
    Organization,
    
    /// Documents or records
    Document,
    
    /// Digital assets (files, NFTs, etc.)
    DigitalAsset,
    
    /// Services provided
    Service,
    
    /// Events in the trace
    Event,
    
    /// Geographic locations
    Location,
    
    /// Equipment or machinery
    Equipment,
    
    /// Domain-specific entity types
    DomainSpecific(String),
}

/// Domains that the entity can belong to
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DomainType {
    /// Supply chain and manufacturing
    SupplyChain,
    
    /// Healthcare and medical
    Healthcare,
    
    /// Pharmaceutical industry
    Pharmaceutical,
    
    /// Automotive industry
    Automotive,
    
    /// Digital assets and NFTs
    DigitalAssets,
    
    /// Custom domain types
    Custom(String),
}

/// Values that properties can have
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    DateTime(DateTime<Utc>),
    Uri(String),
    DomainSpecific(String, String), // Custom type with value
}

/// Relationship between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRelationship {
    /// Subject entity ID
    pub subject: String,
    
    /// Predicate/relationship type
    pub predicate: RelationshipType,
    
    /// Object entity ID
    pub object: String,
    
    /// Domain context for this relationship
    pub domain_context: Option<DomainType>,
    
    /// Temporal information
    pub temporal_info: Option<TemporalInfo>,
    
    /// Confidence score (0.0 to 1.0)
    pub confidence_score: f64,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Types of relationships between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Standard PROV-O relationships
    WasGeneratedBy,
    Used,
    WasAssociatedWith,
    WasAttributedTo,
    WasDerivedFrom,
    WasInformedBy,
    
    /// Domain-specific relationships
    DomainSpecific(String),
    
    /// Cross-domain relationships
    RelatedTo,
    Influenced,
    TransformedInto,
}

/// Temporal information for entities and relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalInfo {
    /// Start time
    pub start_time: Option<DateTime<Utc>>,
    
    /// End time
    pub end_time: Option<DateTime<Utc>>,
    
    /// Timestamp of the relationship
    pub timestamp: DateTime<Utc>,
}

/// Metadata about an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMetadata {
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Version of the entity
    pub version: u32,
    
    /// Tags or labels
    pub tags: Vec<String>,
    
    /// Additional custom metadata
    pub custom_fields: HashMap<String, String>,
}

/// Provenance information for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceInfo {
    /// Creator of the entity
    pub creator: String,
    
    /// Sources of the entity data
    pub sources: Vec<String>,
    
    /// Chain of custody
    pub custody_chain: Vec<CustodyRecord>,
    
    /// Digital signatures
    pub signatures: Vec<DigitalSignature>,
}

/// Record of custody for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyRecord {
    /// Entity or person who had custody
    pub custodian: String,
    
    /// When custody started
    pub start_time: DateTime<Utc>,
    
    /// When custody ended
    pub end_time: Option<DateTime<Utc>>,
    
    /// Location of custody
    pub location: Option<String>,
}

/// Digital signature for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalSignature {
    /// Public key of signer
    pub public_key: String,
    
    /// Signature value
    pub signature: String,
    
    /// Timestamp of signing
    pub timestamp: DateTime<Utc>,
    
    /// Algorithm used
    pub algorithm: String,
}

impl TraceableEntity {
    /// Create a new traceable entity
    pub fn new(id: String, entity_type: EntityType, domain: DomainType) -> Self {
        let now = Utc::now();
        TraceableEntity {
            id,
            entity_type,
            domain,
            properties: HashMap::new(),
            relationships: Vec::new(),
            metadata: EntityMetadata {
                created_at: now,
                updated_at: now,
                version: 1,
                tags: Vec::new(),
                custom_fields: HashMap::new(),
            },
            provenance: ProvenanceInfo {
                creator: "system".to_string(),
                sources: Vec::new(),
                custody_chain: Vec::new(),
                signatures: Vec::new(),
            },
        }
    }

    /// Add a property to the entity
    pub fn add_property(&mut self, key: String, value: PropertyValue) {
        self.properties.insert(key, value);
        self.metadata.updated_at = Utc::now();
        self.metadata.version += 1;
    }

    /// Add a relationship to another entity
    pub fn add_relationship(&mut self, relationship: EntityRelationship) {
        self.relationships.push(relationship);
        self.metadata.updated_at = Utc::now();
        self.metadata.version += 1;
    }

    /// Convert entity to RDF representation
    pub fn to_rdf(&self) -> String {
        // This is a simplified RDF representation
        // A full implementation would generate proper Turtle or RDF/XML
        format!(
            "@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix trace: <http://provchain.org/trace#> .

trace:{} a trace:{} ;
    trace:domain \"{:?}\" ;
    trace:version {} .
",
            self.id, 
            match &self.entity_type {
                EntityType::Product => "Product".to_string(),
                EntityType::Component => "Component".to_string(),
                EntityType::Process => "Process".to_string(),
                EntityType::Person => "Person".to_string(),
                EntityType::Organization => "Organization".to_string(),
                EntityType::Document => "Document".to_string(),
                EntityType::DigitalAsset => "DigitalAsset".to_string(),
                EntityType::Service => "Service".to_string(),
                EntityType::Event => "Event".to_string(),
                EntityType::Location => "Location".to_string(),
                EntityType::Equipment => "Equipment".to_string(),
                EntityType::DomainSpecific(name) => name.clone(),
            },
            self.domain,
            self.metadata.version
        )
    }

    /// Update entity from RDF data
    pub fn update_from_rdf(&mut self, _rdf_data: &str) {
        // This would parse RDF data and update the entity
        // Implementation would depend on the RDF format used
        self.metadata.updated_at = Utc::now();
        self.metadata.version += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entity::{EntityType, DomainType};

    #[test]
    fn test_entity_creation() {
        let entity = TraceableEntity::new(
            "test_entity_001".to_string(),
            EntityType::Product,
            DomainType::SupplyChain
        );
        
        assert_eq!(entity.id, "test_entity_001");
        assert_eq!(entity.entity_type, EntityType::Product);
        assert_eq!(entity.domain, DomainType::SupplyChain);
        assert_eq!(entity.properties.len(), 0);
        assert_eq!(entity.relationships.len(), 0);
    }

    #[test]
    fn test_entity_rdf_conversion() {
        let entity = TraceableEntity::new(
            "test_entity_001".to_string(),
            EntityType::Product,
            DomainType::SupplyChain
        );
        
        let rdf = entity.to_rdf();
        assert!(rdf.contains("test_entity_001"));
        assert!(rdf.contains("Product"));
        assert!(rdf.contains("SupplyChain"));
    }
}