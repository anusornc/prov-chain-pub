//! Core OWL2 axioms that are most commonly used
//!
//! This module contains the most frequently used axiom types
//! extracted from the main axioms module for better organization.

use crate::axioms::types::*;
use crate::axioms::class_expressions;
use crate::entities::{AnonymousIndividual, Literal};
use crate::iri::IRI;
use std::sync::Arc;

/// Property assertion axiom: (a, b) ∈ P
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyAssertionAxiom {
    subject: Arc<IRI>,
    property: Arc<IRI>,
    object: PropertyAssertionObject,
}

impl PropertyAssertionAxiom {
    /// Create a new property assertion axiom with named individual
    pub fn new(subject: Arc<IRI>, property: Arc<IRI>, object: Arc<IRI>) -> Self {
        PropertyAssertionAxiom {
            subject,
            property,
            object: PropertyAssertionObject::Named(object),
        }
    }

    /// Create a new property assertion axiom with anonymous individual
    pub fn new_with_anonymous(
        subject: Arc<IRI>,
        property: Arc<IRI>,
        object: AnonymousIndividual,
    ) -> Self {
        PropertyAssertionAxiom {
            subject,
            property,
            object: PropertyAssertionObject::Anonymous(Box::new(object)),
        }
    }

    /// Create a new property assertion axiom with property assertion object
    pub fn new_with_object(
        subject: Arc<IRI>,
        property: Arc<IRI>,
        object: PropertyAssertionObject,
    ) -> Self {
        PropertyAssertionAxiom {
            subject,
            property,
            object,
        }
    }

    /// Get the subject
    pub fn subject(&self) -> &Arc<IRI> {
        &self.subject
    }

    /// Get the property
    pub fn property(&self) -> &Arc<IRI> {
        &self.property
    }

    /// Get the object
    pub fn object(&self) -> &PropertyAssertionObject {
        &self.object
    }

    /// Get object as IRI if it's a named individual
    pub fn object_iri(&self) -> Option<&Arc<IRI>> {
        match &self.object {
            PropertyAssertionObject::Named(iri) => Some(iri),
            PropertyAssertionObject::Anonymous(_) => None,
        }
    }

    /// Check if this axiom involves a specific subject
    pub fn involves_subject(&self, subject_iri: &IRI) -> bool {
        self.subject.as_ref() == subject_iri
    }

    /// Check if this axiom involves a specific property
    pub fn involves_property(&self, property_iri: &IRI) -> bool {
        self.property.as_ref() == property_iri
    }
}

/// Data property assertion axiom: (a, v) ∈ P where v is a literal
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPropertyAssertionAxiom {
    subject: Arc<IRI>,
    property: Arc<IRI>,
    value: Literal,
}

impl DataPropertyAssertionAxiom {
    /// Create a new data property assertion axiom
    pub fn new(subject: Arc<IRI>, property: Arc<IRI>, value: Literal) -> Self {
        DataPropertyAssertionAxiom {
            subject,
            property,
            value,
        }
    }

    /// Get the subject
    pub fn subject(&self) -> &Arc<IRI> {
        &self.subject
    }

    /// Get the property
    pub fn property(&self) -> &Arc<IRI> {
        &self.property
    }

    /// Get the literal value
    pub fn value(&self) -> &Literal {
        &self.value
    }

    /// Check if this axiom involves a specific subject
    pub fn involves_subject(&self, subject_iri: &IRI) -> bool {
        self.subject.as_ref() == subject_iri
    }

    /// Check if this axiom involves a specific property
    pub fn involves_property(&self, property_iri: &IRI) -> bool {
        self.property.as_ref() == property_iri
    }
}

/// Annotation assertion axiom: ⊤ ⊑ ∃r.{@a}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationAssertionAxiom {
    subject: Arc<IRI>,
    property: Arc<IRI>,
    value: Annotation,
}

impl AnnotationAssertionAxiom {
    /// Create a new annotation assertion axiom
    pub fn new(subject: Arc<IRI>, property: Arc<IRI>, value: Annotation) -> Self {
        AnnotationAssertionAxiom {
            subject,
            property,
            value,
        }
    }

    /// Get the subject
    pub fn subject(&self) -> &Arc<IRI> {
        &self.subject
    }

    /// Get the annotation property
    pub fn property(&self) -> &Arc<IRI> {
        &self.property
    }

    /// Get the annotation value
    pub fn value(&self) -> &Annotation {
        &self.value
    }

    /// Check if this axiom involves a specific subject
    pub fn involves_subject(&self, subject_iri: &IRI) -> bool {
        self.subject.as_ref() == subject_iri
    }
}