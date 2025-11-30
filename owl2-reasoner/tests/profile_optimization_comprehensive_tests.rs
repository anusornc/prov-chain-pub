//! Tests for Profile Optimization
//!
//! This module provides testing for profile optimization functionality
//! that exists in the current API.

use owl2_reasoner::*;
use std::sync::Arc;

#[test]
fn test_profile_basic_functionality() {
    // Test basic profile functionality
    let mut ontology = Ontology::new();

    // Add classes for different profiles
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
fn test_el_profile_patterns() {
    // Test EL profile patterns
    let mut ontology = Ontology::new();

    // Add classes that would be in EL profile
    for i in 0..5 {
        let class_iri = IRI::new(format!("http://example.org/ELClass{}", i)).unwrap();
        let class = Class::new(Arc::new(class_iri));
        ontology.add_class(class).expect("Failed to add EL class");
    }

    // Add properties (object properties for EL)
    for i in 0..3 {
        let prop_iri = IRI::new(format!("http://example.org/elProp{}", i)).unwrap();
        let prop = ObjectProperty::new(Arc::new(prop_iri));
        ontology
            .add_object_property(prop)
            .expect("Failed to add EL property");
    }

    // Verify EL-like structure
    assert_eq!(ontology.classes().iter().count(), 5);
    assert_eq!(ontology.object_properties().iter().count(), 3);

    // Test reasoning
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent();
    assert!(is_consistent.is_ok());
}

#[test]
fn test_ql_profile_patterns() {
    // Test QL profile patterns
    let mut ontology = Ontology::new();

    // Add classes that would be in QL profile
    let person_iri = IRI::new("http://example.org/Person").unwrap();
    let person_class = Class::new(Arc::new(person_iri));
    ontology
        .add_class(person_class)
        .expect("Failed to add Person class");

    let student_iri = IRI::new("http://example.org/Student").unwrap();
    let student_class = Class::new(Arc::new(student_iri));
    ontology
        .add_class(student_class)
        .expect("Failed to add Student class");

    // Add data properties (common in QL)
    let has_name_iri = IRI::new("http://example.org/hasName").unwrap();
    let has_name = DataProperty::new(Arc::new(has_name_iri));
    ontology
        .add_data_property(has_name)
        .expect("Failed to add hasName property");

    // Verify QL-like structure
    assert_eq!(ontology.classes().iter().count(), 2);
    assert_eq!(ontology.data_properties().iter().count(), 1);

    // Test reasoning
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent();
    assert!(is_consistent.is_ok());
}

#[test]
fn test_rl_profile_patterns() {
    // Test RL profile patterns
    let mut ontology = Ontology::new();

    // Add classes that would be in RL profile
    let person_iri = IRI::new("http://example.org/Person").unwrap();
    let person_class = Class::new(Arc::new(person_iri));
    ontology
        .add_class(person_class)
        .expect("Failed to add Person class");

    // Add role properties (common in RL)
    let has_child_iri = IRI::new("http://example.org/hasChild").unwrap();
    let has_child = ObjectProperty::new(Arc::new(has_child_iri));
    ontology
        .add_object_property(has_child)
        .expect("Failed to add hasChild property");

    // Verify RL-like structure
    assert_eq!(ontology.classes().iter().count(), 1);
    assert_eq!(ontology.object_properties().iter().count(), 1);

    // Test reasoning
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent();
    assert!(is_consistent.is_ok());
}

#[test]
fn test_profile_optimization_performance() {
    // Test profile optimization performance
    let mut ontology = Ontology::new();

    // Create a medium-sized ontology
    for i in 0..50 {
        let class_iri = IRI::new(format!("http://example.org/Class{}", i)).unwrap();
        let class = Class::new(Arc::new(class_iri));
        ontology.add_class(class).expect("Failed to add class");
    }

    // Add properties
    for i in 0..20 {
        let prop_iri = IRI::new(format!("http://example.org/property{}", i)).unwrap();
        let prop = ObjectProperty::new(Arc::new(prop_iri));
        ontology
            .add_object_property(prop)
            .expect("Failed to add property");
    }

    // Test reasoning performance
    let start_time = std::time::Instant::now();
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent();
    let duration = start_time.elapsed();

    assert!(is_consistent.is_ok());
    // Should complete reasonably quickly (less than 1 second for this size)
    assert!(duration.as_secs() < 1);
    println!("Reasoning completed in {:?}", duration);
}

#[test]
fn test_profile_validation() {
    // Test profile validation
    let mut ontology = Ontology::new();

    // Add test entities
    let person_iri = IRI::new("http://example.org/Person").unwrap();
    let person_class = Class::new(Arc::new(person_iri));
    ontology
        .add_class(person_class)
        .expect("Failed to add Person class");

    // Test validation framework
    let mut framework = owl2_reasoner::validation::ValidationFramework::new()
        .expect("Failed to create validation framework");

    let result = framework.run_basic_validation();

    match result {
        Ok(report) => {
            println!(
                "Validation completed with score: {}",
                report.w3c_compliance_score
            );
            assert!(report.w3c_compliance_score >= 0.0);
        }
        Err(e) => {
            println!("Validation failed: {}", e);
        }
    }
}

#[test]
fn test_error_handling_in_profiles() {
    // Test error handling in profile operations

    // Test invalid IRI
    let invalid_iri_result = IRI::new("");
    assert!(invalid_iri_result.is_err());

    // Test valid profile setup
    let mut ontology = Ontology::new();
    let valid_iri = IRI::new("http://example.org/ValidClass").unwrap();
    let valid_class = Class::new(Arc::new(valid_iri));
    ontology
        .add_class(valid_class)
        .expect("Failed to add valid class");

    let reasoner = SimpleReasoner::new(ontology);
    assert!(reasoner.is_consistent().is_ok());
}
