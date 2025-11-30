//! Storage backends for OWL2 ontologies
//!
//! Provides different storage strategies for ontologies with different
//! performance characteristics.

use crate::error::{OwlError, OwlResult};
use crate::ontology::Ontology;
use hashbrown::HashMap;

/// Trait for ontology storage backends
pub trait StorageBackend {
    /// Store an ontology
    fn store(&mut self, ontology: Ontology) -> OwlResult<()>;

    /// Retrieve an ontology
    fn retrieve(&self) -> OwlResult<&Ontology>;

    /// Clear all stored data
    fn clear(&mut self) -> OwlResult<()>;
}

/// In-memory storage backend
#[derive(Debug, Default)]
pub struct MemoryStorage {
    ontology: Option<Ontology>,
}

impl MemoryStorage {
    /// Create a new empty memory storage
    pub fn new() -> Self {
        Self::default()
    }
}

impl StorageBackend for MemoryStorage {
    fn store(&mut self, ontology: Ontology) -> OwlResult<()> {
        self.ontology = Some(ontology);
        Ok(())
    }

    fn retrieve(&self) -> OwlResult<&Ontology> {
        self.ontology
            .as_ref()
            .ok_or_else(|| OwlError::StorageError("No ontology stored".to_string()))
    }

    fn clear(&mut self) -> OwlResult<()> {
        self.ontology = None;
        Ok(())
    }
}

/// Indexed storage backend for faster access
#[derive(Debug, Default)]
pub struct IndexedStorage {
    ontology: Option<Ontology>,
    // Indexes for faster access
    class_index: HashMap<String, usize>,
    property_index: HashMap<String, usize>,
    individual_index: HashMap<String, usize>,
}

impl IndexedStorage {
    /// Create a new empty indexed storage
    pub fn new() -> Self {
        Self::default()
    }

    /// Build indexes from the ontology
    fn build_indexes(&mut self, ontology: &Ontology) {
        // Clear existing indexes
        self.class_index.clear();
        self.property_index.clear();
        self.individual_index.clear();

        // Index classes
        for (idx, class) in ontology.classes().iter().enumerate() {
            self.class_index
                .insert(class.iri().as_str().to_string(), idx);
        }

        // Index object properties
        for (idx, prop) in ontology.object_properties().iter().enumerate() {
            self.property_index
                .insert(prop.iri().as_str().to_string(), idx);
        }

        // Index data properties
        for (idx, prop) in ontology.data_properties().iter().enumerate() {
            self.property_index
                .insert(prop.iri().as_str().to_string(), idx);
        }

        // Index individuals
        for (idx, individual) in ontology.named_individuals().iter().enumerate() {
            self.individual_index
                .insert(individual.iri().as_str().to_string(), idx);
        }
    }
}

impl StorageBackend for IndexedStorage {
    fn store(&mut self, ontology: Ontology) -> OwlResult<()> {
        self.build_indexes(&ontology);
        self.ontology = Some(ontology);
        Ok(())
    }

    fn retrieve(&self) -> OwlResult<&Ontology> {
        self.ontology
            .as_ref()
            .ok_or_else(|| OwlError::StorageError("No ontology stored".to_string()))
    }

    fn clear(&mut self) -> OwlResult<()> {
        self.ontology = None;
        self.class_index.clear();
        self.property_index.clear();
        self.individual_index.clear();
        Ok(())
    }
}
