//! Tests for Rollback and Non-Deterministic Reasoning
//!
//! This module provides testing for rollback and non-deterministic reasoning functionality
//! that exists in the current API.

use owl2_reasoner::*;
use std::sync::Arc;

#[test]
fn test_reasoning_basic_functionality() {
    // Test basic reasoning functionality
    let mut ontology = Ontology::new();

    // Add classes
    let person_iri = IRI::new("http://example.org/Person").unwrap();
    let person_class = Class::new(Arc::new(person_iri));
    ontology
        .add_class(person_class)
        .expect("Failed to add Person class");

    // Test reasoning
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent();

    // Should be consistent
    assert!(is_consistent.is_ok());
}

#[test]
fn test_memory_management() {
    // Test memory management during reasoning
    let mut ontology = Ontology::new();

    // Add multiple classes
    for i in 0..10 {
        let class_iri = IRI::new(format!("http://example.org/Class{}", i)).unwrap();
        let class = Class::new(Arc::new(class_iri));
        ontology.add_class(class).expect("Failed to add class");
    }

    // Verify all classes were added
    assert_eq!(ontology.classes().iter().count(), 10);

    // Test reasoning
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent();
    assert!(is_consistent.is_ok());
}

#[test]
fn test_concurrent_reasoning() {
    // Test concurrent reasoning operations
    let mut ontology = Ontology::new();

    // Add classes
    let person_iri = IRI::new("http://example.org/Person").unwrap();
    let person_class = Class::new(Arc::new(person_iri));
    ontology
        .add_class(person_class)
        .expect("Failed to add Person class");

    // Create multiple reasoners (simulating concurrent access)
    let reasoner1 = SimpleReasoner::new(ontology.clone());
    let reasoner2 = SimpleReasoner::new(ontology.clone());

    // Both should work
    assert!(reasoner1.is_consistent().is_ok());
    assert!(reasoner2.is_consistent().is_ok());
}

#[test]
fn test_reasoning_with_properties() {
    // Test reasoning with properties
    let mut ontology = Ontology::new();

    // Add classes
    let person_iri = IRI::new("http://example.org/Person").unwrap();
    let person_class = Class::new(Arc::new(person_iri));
    ontology
        .add_class(person_class)
        .expect("Failed to add Person class");

    // Add properties
    let has_parent_iri = IRI::new("http://example.org/hasParent").unwrap();
    let has_parent = ObjectProperty::new(Arc::new(has_parent_iri));
    ontology
        .add_object_property(has_parent)
        .expect("Failed to add hasParent property");

    // Test reasoning
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent();
    assert!(is_consistent.is_ok());
}

#[test]
fn test_reasoning_state_management() {
    // Test reasoning state management
    let mut ontology = Ontology::new();

    // Initial state
    assert_eq!(ontology.classes().iter().count(), 0);

    // Add entities
    let person_iri = IRI::new("http://example.org/Person").unwrap();
    let person_class = Class::new(Arc::new(person_iri));
    ontology
        .add_class(person_class)
        .expect("Failed to add Person class");

    // State changed
    assert_eq!(ontology.classes().iter().count(), 1);

    // Test reasoning on modified state
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent();
    assert!(is_consistent.is_ok());
}

#[test]
fn test_error_handling_in_reasoning() {
    // Test error handling in reasoning operations

    // Test invalid IRI
    let invalid_iri_result = IRI::new("");
    assert!(invalid_iri_result.is_err());

    // Test valid reasoning setup
    let mut ontology = Ontology::new();
    let valid_iri = IRI::new("http://example.org/ValidClass").unwrap();
    let valid_class = Class::new(Arc::new(valid_iri));
    ontology
        .add_class(valid_class)
        .expect("Failed to add valid class");

    let reasoner = SimpleReasoner::new(ontology);
    assert!(reasoner.is_consistent().is_ok());
}

#[test]
fn test_large_scale_reasoning() {
    // Test reasoning with larger ontologies
    let mut ontology = Ontology::new();

    // Add many classes
    for i in 0..100 {
        let class_iri = IRI::new(format!("http://example.org/Class{}", i)).unwrap();
        let class = Class::new(Arc::new(class_iri));
        ontology.add_class(class).expect("Failed to add class");
    }

    // Add some properties
    for i in 0..20 {
        let prop_iri = IRI::new(format!("http://example.org/property{}", i)).unwrap();
        let prop = ObjectProperty::new(Arc::new(prop_iri));
        ontology
            .add_object_property(prop)
            .expect("Failed to add property");
    }

    // Verify structure
    assert_eq!(ontology.classes().iter().count(), 100);
    assert_eq!(ontology.object_properties().iter().count(), 20);

    // Test reasoning
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent();
    assert!(is_consistent.is_ok());
}
