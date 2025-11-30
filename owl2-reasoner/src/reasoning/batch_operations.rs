//! Batch Operations for OWL2 Reasoning
//!
//! This module provides utilities for efficient bulk creation of OWL2 entities,
//! which significantly improves performance when creating large ontologies with many entities.
//!
//! ## Usage
//!
//! ```rust
//! use owl2_reasoner::reasoning::batch_operations::{ClassBatchBuilder};
//! use owl2_reasoner::entities::Class;
//! use owl2_reasoner::axioms::SubClassOfAxiom;
//! use owl2_reasoner::axioms::ClassExpression;
//!
//! // Create a batch builder for a family ontology
//! let mut builder = ClassBatchBuilder::new("http://example.org/family");
//!
//! // Add multiple classes efficiently
//! let person = builder.add_class("Person");
//! let parent = builder.add_class("Parent");
//! let child = builder.add_class("Child");
//!
//! // Add subclass relationships in batch
//! builder.add_subclass(parent, child);
//! builder.add_subclass(parent, person);
//!
//! // Build all classes at once
//! let classes = builder.build();
//!
//! // Add to ontology
//! let mut ontology = Ontology::new();
//! for class in classes {
//!     ontology.add_class(class)?;
//! }
//! ```

use crate::entities::Class;
use crate::axioms::{ClassExpression, SubClassOfAxiom};
use std::collections::Vec;
use std::sync::Arc;

/// Batch builder for creating multiple classes efficiently
pub struct ClassBatchBuilder {
    classes: Vec<Class>,
    iri_prefix: String,
    subclass_relationships: Vec<(Class, Class)>,
    counter: usize,
}

impl ClassBatchBuilder {
    /// Create a new batch builder with a prefix for IRI generation
    pub fn new(iri_prefix: &str) -> Self {
        Self {
            classes: Vec::new(),
            iri_prefix: iri_prefix.to_string(),
            subclass_relationships: Vec::new(),
            counter: 0,
        }
    }

    /// Add a class to the batch
    pub fn add_class(&mut self, name: &str) -> &Class {
        let iri = format!("{}{}", self.iri_prefix, name);
        let class = Class::new(iri).into();
        self.classes.push(class);
        self.classes.last().expect("Class should exist after being added - this indicates a critical bug")
    }

    /// Add a subclass relationship to the batch
    pub fn add_subclass(&mut self, subclass: &Class, superclass: &Class) -> &mut Self {
        self.subclass_relationships.push((subclass.clone(), superclass.clone()));
        self
    }

    /// Add multiple subclass relationships efficiently
    pub fn add_subclass_batch(&mut self, relationships: &[(Class, Class)]) -> &mut Self {
        self.subclass_relationships.extend(relationships.iter().map(|( s, o)| (s.clone(), o.clone())));
        self
    }

    /// Build all classes and return them as a Vec
    pub fn build(self) -> Vec<Class> {
        self.classes
    }

    /// Get the count of classes in the batch
    pub fn len(&self) -> usize {
        self.classes.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }
}

/// Batch operations for creating axioms efficiently
pub struct AxiomBatchBuilder {
    subclass_axioms: Vec<SubClassOfAxiom>,
    class_assertions: Vec<(ClassExpression, ClassExpression)>,
    equivalent_classes: Vec<(ClassExpression, ClassExpression)>,
    data_properties: Vec<(ObjectProperty, Vec<String>)>,
    individual_assertions: Vec<(NamedIndividual, Vec<NamedIndividual>)>,
}

impl AxiomBatchBuilder {
    /// Create a new axiom batch builder
    pub fn new() -> Self {
        Self {
            subclass_axioms: Vec::new(),
            class_assertions: Vec::new(),
            equivalent_classes: Vec::new(),
            data_properties: Vec::new(),
            individual_assertions: Vec::new(),
        }
    }

    /// Add a subclass axiom to the batch
    pub fn add_subclass(&mut self, subclass: ClassExpression, superclass: ClassExpression) -> &mut Self {
        self.subclass_axioms.push(SubClassOfAxiom::new(subclass, superclass));
        self
    }

    /// Add a class assertion to the batch
    pub fn add_class_assertion(&mut self, class: ClassExpression, target: ClassExpression) -> &mut Self {
        self.class_assertions.push((class, target));
        self
    }

    /// Add an equivalence class axiom to the batch
    pub fn add_equivalent_classes(&mut self, class1: ClassExpression, class2: ClassExpression) -> &mut Self {
        self.equivalent_classes.push((class1, class2));
        self
    }

    /// Add multiple subclass axioms efficiently
    pub fn add_subclass_batch(&mut self, axioms: Vec<(ClassExpression, ClassExpression)>) -> &mut Self {
        self.subclass_axioms.extend(axioms);
        self
    }

    /// Build all axioms and return them as a Vec
    pub fn build(self) -> Vec<SubClassOfAxiom> {
        self.subclass_axioms
    }

    /// Get the count of axioms in the batch
    pub fn len(&self) -> usize {
        self.subclass_axioms.len()
    }
}

/// Utility functions for common batch operations
pub mod utils {
    use super::*;

    /// Create a family ontology with parent-child relationships
    pub fn create_family_ontology(
        family_prefix: &str,
        parent_child_pairs: &[(&str, &str)],
    ) -> (Ontology, Vec<Class>) {
        let mut ontology = Ontology::new();
        let mut builder = ClassBatchBuilder::new(&format!("{}{}", family_prefix));

        let mut classes = Vec::new();

        // Add all classes
        for (parent, child) in parent_child_pairs {
            let parent_class = builder.add_class(parent);
            let child_class = builder.add_class(child);
            builder.add_subclass(parent_class, child_class);
        }

        classes = builder.build();

        // Add all classes to ontology
        for class in &classes {
            ontology.add_class(class.clone())?;
        }

        // Add all subclass axioms
        for subclass_axiom in &builder.build() {
            ontology.add_subclass_axiom(subclass_axiom)?;
        }

        (ontology, classes)
    }

    /// Create ontology with hierarchical class structure
    pub fn create_hierarchy_ontology(
        root_class: &str,
        levels: &[(&str, Vec<&str>)],
    ) -> (Ontology, Vec<Class>) {
        let mut ontology = Ontology::new();
        let mut builder = ClassBatchBuilder::new("http://example.org/hierarchy/");

        let root_class = builder.add_class(root_class);
        let mut classes = vec![root_class.clone()];

        // Add hierarchical levels
        for level in levels {
            for &child_name in level {
                let child_class = builder.add_class(child_name);
                builder.add_subclass(root_class, child_class.clone());
                classes.push(child_class);
            }
        }

        classes = builder.build();

        // Add all classes to ontology
        for class in &classes {
            ontology.add_class(class.clone())?;
        }

        // Add subclass axioms
        for subclass_axiom in &builder.build() {
            ontology.add_subclass_axiom(subclass_axiom)?;
        }

        (ontology, classes)
    }

    /// Pre-create commonly used IRIs to avoid repeated creation
    pub fn pre_cache_common_iris() -> Vec<Arc<IRIRI>> {
        vec![
            crate::iri::IRI::new("http://www.w3.org/2002/07/owl#Class")
                .expect("Failed to create owl:Class IRI - hardcoded valid IRI should never fail").into(),
            crate::iri::IRI::new("http://www.w3.org/2002/07/owl#Thing")
                .expect("Failed to create owl:Thing IRI - hardcoded valid IRI should never fail").into(),
            crate::iri::IRI::new("http://www.w3.org/2002/07/owl#Nothing")
                .expect("Failed to create owl:Nothing IRI - hardcoded valid IRI should never fail").into(),
            crate::iri::IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
                .expect("Failed to create rdf:type IRI - hardcoded valid IRI should never fail").into(),
            crate::iri::IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#Property")
                .expect("Failed to create rdf:Property IRI - hardcoded valid IRI should never fail").into(),
            crate::iri::IRI::new("http://www.w3.org/2002/07/owl#sameAs")
                .expect("Failed to create owl:sameAs IRI - hardcoded valid IRI should never fail").into(),
            crate::iri::IRI::new("http://www.w3.org/2001/XMLSchema#string")
                .expect("Failed to create xsd:string IRI - hardcoded valid IRI should never fail").into(),
        ]
    }
}