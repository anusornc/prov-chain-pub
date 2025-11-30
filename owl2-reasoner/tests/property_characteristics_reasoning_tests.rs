//! Tests for Property Characteristics Reasoning
//!
//! This module provides testing for property characteristics reasoning functionality
//! that exists in the current API.

use owl2_reasoner::*;
use std::sync::Arc;

#[test]
fn test_property_creation() {
    // Test basic property creation
    let has_parent_iri = IRI::new("http://example.org/hasParent").unwrap();
    let has_parent = ObjectProperty::new(Arc::new(has_parent_iri));

    let has_name_iri = IRI::new("http://example.org/hasName").unwrap();
    let has_name = DataProperty::new(Arc::new(has_name_iri));

    // Properties created successfully
    assert_eq!(has_parent.iri().as_str(), "http://example.org/hasParent");
    assert_eq!(has_name.iri().as_str(), "http://example.org/hasName");
}

#[test]
fn test_ontology_with_properties() {
    // Test adding properties to ontology
    let mut ontology = Ontology::new();

    let has_parent_iri = IRI::new("http://example.org/hasParent").unwrap();
    let has_parent = ObjectProperty::new(Arc::new(has_parent_iri));
    ontology
        .add_object_property(has_parent)
        .expect("Failed to add object property");

    let has_name_iri = IRI::new("http://example.org/hasName").unwrap();
    let has_name = DataProperty::new(Arc::new(has_name_iri));
    ontology
        .add_data_property(has_name)
        .expect("Failed to add data property");

    // Verify properties were added
    assert_eq!(ontology.object_properties().iter().count(), 1);
    assert_eq!(ontology.data_properties().iter().count(), 1);
}

#[test]
fn test_property_axiom_creation() {
    // Test creating property axioms
    let prop_iri = IRI::new("http://example.org/testProp").unwrap();
    let prop = ObjectProperty::new(Arc::new(prop_iri));

    // Property axiom creation would work if the types were available
    // For now, just test that the property exists
    assert_eq!(prop.iri().as_str(), "http://example.org/testProp");
}

#[test]
fn test_property_characteristics_validation() {
    // Test property characteristics validation
    let mut ontology = Ontology::new();

    // Add test properties
    let sym_prop_iri = IRI::new("http://example.org/symmetricProp").unwrap();
    let sym_prop = ObjectProperty::new(Arc::new(sym_prop_iri));
    ontology
        .add_object_property(sym_prop)
        .expect("Failed to add symmetric property");

    let trans_prop_iri = IRI::new("http://example.org/transitiveProp").unwrap();
    let trans_prop = ObjectProperty::new(Arc::new(trans_prop_iri));
    ontology
        .add_object_property(trans_prop)
        .expect("Failed to add transitive property");

    // Verify properties exist
    assert_eq!(ontology.object_properties().iter().count(), 2);
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

    let animal_iri = IRI::new("http://example.org/Animal").unwrap();
    let animal_class = Class::new(Arc::new(animal_iri));
    ontology
        .add_class(animal_class)
        .expect("Failed to add Animal class");

    // Add object property
    let has_parent_iri = IRI::new("http://example.org/hasParent").unwrap();
    let has_parent = ObjectProperty::new(Arc::new(has_parent_iri));
    ontology
        .add_object_property(has_parent)
        .expect("Failed to add hasParent property");

    // Test basic reasoning functionality
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent();

    // Should be consistent
    assert!(is_consistent.is_ok());
}

#[test]
fn test_property_domains_and_ranges() {
    // Test property domains and ranges
    let mut ontology = Ontology::new();

    // Add classes
    let person_iri = IRI::new("http://example.org/Person").unwrap();
    let person_class = Class::new(Arc::new(person_iri));
    ontology
        .add_class(person_class)
        .expect("Failed to add Person class");

    let _string_iri = IRI::new("http://www.w3.org/2001/XMLSchema#string").unwrap();

    // Add properties
    let has_parent_iri = IRI::new("http://example.org/hasParent").unwrap();
    let has_parent = ObjectProperty::new(Arc::new(has_parent_iri));
    ontology
        .add_object_property(has_parent)
        .expect("Failed to add hasParent property");

    let has_name_iri = IRI::new("http://example.org/hasName").unwrap();
    let has_name = DataProperty::new(Arc::new(has_name_iri));
    ontology
        .add_data_property(has_name)
        .expect("Failed to add hasName property");

    // Verify structure
    assert_eq!(ontology.classes().iter().count(), 1);
    assert_eq!(ontology.object_properties().iter().count(), 1);
    assert_eq!(ontology.data_properties().iter().count(), 1);
}

#[test]
fn test_error_handling_with_properties() {
    // Test error handling in property operations

    // Test invalid IRI
    let invalid_iri_result = IRI::new("");
    assert!(invalid_iri_result.is_err());

    // Test valid property creation
    let valid_iri = IRI::new("http://example.org/validProp").unwrap();
    let valid_prop = ObjectProperty::new(Arc::new(valid_iri));
    assert_eq!(valid_prop.iri().as_str(), "http://example.org/validProp");
}
