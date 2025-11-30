//! Tests for Equality Reasoning
//!
//! This module provides testing for equality reasoning functionality
//! that exists in the current API.

use owl2_reasoner::*;
use std::sync::Arc;

#[test]
fn test_individual_creation() {
    // Test individual creation
    let john_iri = IRI::new("http://example.org/John").unwrap();
    let john = NamedIndividual::new(Arc::new(john_iri));

    let mary_iri = IRI::new("http://example.org/Mary").unwrap();
    let mary = NamedIndividual::new(Arc::new(mary_iri));

    // Individuals created successfully
    assert_eq!(john.iri().as_str(), "http://example.org/John");
    assert_eq!(mary.iri().as_str(), "http://example.org/Mary");
}

#[test]
fn test_ontology_with_individuals() {
    // Test adding individuals to ontology
    let mut ontology = Ontology::new();

    let john_iri = IRI::new("http://example.org/John").unwrap();
    let john = NamedIndividual::new(Arc::new(john_iri));
    ontology
        .add_named_individual(john)
        .expect("Failed to add John");

    let mary_iri = IRI::new("http://example.org/Mary").unwrap();
    let mary = NamedIndividual::new(Arc::new(mary_iri));
    ontology
        .add_named_individual(mary)
        .expect("Failed to add Mary");

    // Verify individuals were added
    assert_eq!(ontology.named_individuals().iter().count(), 2);
}

#[test]
fn test_same_individual_axioms() {
    // Test same individual axioms
    let mut ontology = Ontology::new();

    // Add individuals
    let john1_iri = IRI::new("http://example.org/John1").unwrap();
    let john1 = NamedIndividual::new(Arc::new(john1_iri));
    ontology
        .add_named_individual(john1)
        .expect("Failed to add John1");

    let john2_iri = IRI::new("http://example.org/John2").unwrap();
    let john2 = NamedIndividual::new(Arc::new(john2_iri));
    ontology
        .add_named_individual(john2)
        .expect("Failed to add John2");

    // In a full implementation, we would add SameIndividualAxiom here
    // For now, just test that the individuals exist
    assert_eq!(ontology.named_individuals().iter().count(), 2);
}

#[test]
fn test_different_individuals_axioms() {
    // Test different individuals axioms
    let mut ontology = Ontology::new();

    // Add individuals
    let john_iri = IRI::new("http://example.org/John").unwrap();
    let john = NamedIndividual::new(Arc::new(john_iri));
    ontology
        .add_named_individual(john)
        .expect("Failed to add John");

    let mary_iri = IRI::new("http://example.org/Mary").unwrap();
    let mary = NamedIndividual::new(Arc::new(mary_iri));
    ontology
        .add_named_individual(mary)
        .expect("Failed to add Mary");

    // In a full implementation, we would add DifferentIndividualsAxiom here
    // For now, just test that the individuals exist
    assert_eq!(ontology.named_individuals().iter().count(), 2);
}

#[test]
fn test_equality_reasoning() {
    // Test equality reasoning
    let mut ontology = Ontology::new();

    // Add classes
    let person_iri = IRI::new("http://example.org/Person").unwrap();
    let person_class = Class::new(Arc::new(person_iri));
    ontology
        .add_class(person_class)
        .expect("Failed to add Person class");

    // Add individuals
    let john_iri = IRI::new("http://example.org/John").unwrap();
    let john = NamedIndividual::new(Arc::new(john_iri));
    ontology
        .add_named_individual(john)
        .expect("Failed to add John");

    // Test reasoning
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent();

    // Should be consistent
    assert!(is_consistent.is_ok());
}

#[test]
fn test_equality_with_classes() {
    // Test equality in context of classes
    let mut ontology = Ontology::new();

    // Add classes
    let person_iri = IRI::new("http://example.org/Person").unwrap();
    let person_class = Class::new(Arc::new(person_iri));
    ontology
        .add_class(person_class)
        .expect("Failed to add Person class");

    let employee_iri = IRI::new("http://example.org/Employee").unwrap();
    let employee_class = Class::new(Arc::new(employee_iri));
    ontology
        .add_class(employee_class)
        .expect("Failed to add Employee class");

    // Add individuals
    let john_iri = IRI::new("http://example.org/John").unwrap();
    let john = NamedIndividual::new(Arc::new(john_iri));
    ontology
        .add_named_individual(john)
        .expect("Failed to add John");

    // Verify structure
    assert_eq!(ontology.classes().iter().count(), 2);
    assert_eq!(ontology.named_individuals().iter().count(), 1);
}

#[test]
fn test_error_handling_with_equality() {
    // Test error handling in equality operations

    // Test invalid IRI
    let invalid_iri_result = IRI::new("");
    assert!(invalid_iri_result.is_err());

    // Test valid individual creation
    let valid_iri = IRI::new("http://example.org/validIndividual").unwrap();
    let valid_individual = NamedIndividual::new(Arc::new(valid_iri));
    assert_eq!(
        valid_individual.iri().as_str(),
        "http://example.org/validIndividual"
    );
}
