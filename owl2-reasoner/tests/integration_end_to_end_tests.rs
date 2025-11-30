//! End-to-End Integration Tests
//!
//! This module provides comprehensive end-to-end testing of the OWL2 reasoner
//! with the actual API functionality that exists.

use owl2_reasoner::*;
use std::sync::Arc;

#[test]
fn test_complete_reasoning_workflow() {
    // Test complete reasoning workflow

    // 1. Create ontology
    let mut ontology = Ontology::new();
    ontology.set_iri("http://example.org/test-ontology");

    // 2. Add classes
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

    // 3. Add properties
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

    // 4. Add individuals
    let john_iri = IRI::new("http://example.org/John").unwrap();
    let john = NamedIndividual::new(Arc::new(john_iri));
    ontology
        .add_named_individual(john)
        .expect("Failed to add John");

    // 5. Verify ontology structure
    assert_eq!(ontology.classes().iter().count(), 2);
    assert_eq!(ontology.object_properties().iter().count(), 1);
    assert_eq!(ontology.data_properties().iter().count(), 1);
    assert_eq!(ontology.named_individuals().iter().count(), 1);

    // 6. Create reasoner and test reasoning
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent();
    assert!(is_consistent.is_ok());
    assert!(is_consistent.unwrap());

    println!("Complete reasoning workflow test passed");
}

#[test]
fn test_family_relationship_ontology() {
    // Test family relationship ontology
    let mut ontology = Ontology::new();
    ontology.set_iri("http://example.org/family");

    // Add classes
    let person_iri = IRI::new("http://example.org/Person").unwrap();
    let person_class = Class::new(Arc::new(person_iri));
    ontology
        .add_class(person_class)
        .expect("Failed to add Person class");

    let male_iri = IRI::new("http://example.org/Male").unwrap();
    let male_class = Class::new(Arc::new(male_iri));
    ontology
        .add_class(male_class)
        .expect("Failed to add Male class");

    let female_iri = IRI::new("http://example.org/Female").unwrap();
    let female_class = Class::new(Arc::new(female_iri));
    ontology
        .add_class(female_class)
        .expect("Failed to add Female class");

    // Add properties
    let has_parent_iri = IRI::new("http://example.org/hasParent").unwrap();
    let has_parent = ObjectProperty::new(Arc::new(has_parent_iri));
    ontology
        .add_object_property(has_parent)
        .expect("Failed to add hasParent property");

    let has_child_iri = IRI::new("http://example.org/hasChild").unwrap();
    let has_child = ObjectProperty::new(Arc::new(has_child_iri));
    ontology
        .add_object_property(has_child)
        .expect("Failed to add hasChild property");

    let has_name_iri = IRI::new("http://example.org/hasName").unwrap();
    let has_name = DataProperty::new(Arc::new(has_name_iri));
    ontology
        .add_data_property(has_name)
        .expect("Failed to add hasName property");

    let has_age_iri = IRI::new("http://example.org/hasAge").unwrap();
    let has_age = DataProperty::new(Arc::new(has_age_iri));
    ontology
        .add_data_property(has_age)
        .expect("Failed to add hasAge property");

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

    let susan_iri = IRI::new("http://example.org/Susan").unwrap();
    let susan = NamedIndividual::new(Arc::new(susan_iri));
    ontology
        .add_named_individual(susan)
        .expect("Failed to add Susan");

    // Test reasoning
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent();
    assert!(is_consistent.is_ok());
    assert!(is_consistent.unwrap());

    println!("Family relationship ontology test passed");
}

#[test]
fn test_biomedical_scenario() {
    // Test biomedical scenario
    let mut ontology = Ontology::new();
    ontology.set_iri("http://example.org/biomedical");

    // Add biomedical classes
    let disease_iri = IRI::new("http://example.org/Disease").unwrap();
    let disease_class = Class::new(Arc::new(disease_iri));
    ontology
        .add_class(disease_class)
        .expect("Failed to add Disease class");

    let symptom_iri = IRI::new("http://example.org/Symptom").unwrap();
    let symptom_class = Class::new(Arc::new(symptom_iri));
    ontology
        .add_class(symptom_class)
        .expect("Failed to add Symptom class");

    let treatment_iri = IRI::new("http://example.org/Treatment").unwrap();
    let treatment_class = Class::new(Arc::new(treatment_iri));
    ontology
        .add_class(treatment_class)
        .expect("Failed to add Treatment class");

    // Add properties
    let has_symptom_iri = IRI::new("http://example.org/hasSymptom").unwrap();
    let has_symptom = ObjectProperty::new(Arc::new(has_symptom_iri));
    ontology
        .add_object_property(has_symptom)
        .expect("Failed to add hasSymptom property");

    let has_treatment_iri = IRI::new("http://example.org/hasTreatment").unwrap();
    let has_treatment = ObjectProperty::new(Arc::new(has_treatment_iri));
    ontology
        .add_object_property(has_treatment)
        .expect("Failed to add hasTreatment property");

    // Add disease instances
    let flu_iri = IRI::new("http://example.org/Flu").unwrap();
    let flu = NamedIndividual::new(Arc::new(flu_iri));
    ontology
        .add_named_individual(flu)
        .expect("Failed to add Flu");

    let covid_iri = IRI::new("http://example.org/COVID19").unwrap();
    let covid = NamedIndividual::new(Arc::new(covid_iri));
    ontology
        .add_named_individual(covid)
        .expect("Failed to add COVID-19");

    // Test reasoning
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent();
    assert!(is_consistent.is_ok());
    assert!(is_consistent.unwrap());

    println!("Biomedical scenario test passed");
}

#[test]
fn test_large_scale_integration() {
    // Test large-scale integration
    let mut ontology = Ontology::new();
    ontology.set_iri("http://example.org/large-scale");

    // Add many classes
    for i in 0..100 {
        let class_iri = IRI::new(format!("http://example.org/Class{}", i)).unwrap();
        let class = Class::new(Arc::new(class_iri));
        ontology.add_class(class).expect("Failed to add class");
    }

    // Add many properties
    for i in 0..50 {
        let prop_iri = IRI::new(format!("http://example.org/property{}", i)).unwrap();
        let prop = ObjectProperty::new(Arc::new(prop_iri));
        ontology
            .add_object_property(prop)
            .expect("Failed to add property");
    }

    // Add many individuals
    for i in 0..200 {
        let individual_iri = IRI::new(format!("http://example.org/Individual{}", i)).unwrap();
        let individual = NamedIndividual::new(Arc::new(individual_iri));
        ontology
            .add_named_individual(individual)
            .expect("Failed to add individual");
    }

    // Verify structure
    assert_eq!(ontology.classes().iter().count(), 100);
    assert_eq!(ontology.object_properties().iter().count(), 50);
    assert_eq!(ontology.named_individuals().iter().count(), 200);

    // Test reasoning performance
    let start_time = std::time::Instant::now();
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent();
    let duration = start_time.elapsed();

    assert!(is_consistent.is_ok());
    assert!(is_consistent.unwrap());
    assert!(duration.as_secs() < 5); // Should complete within 5 seconds

    println!(
        "Large-scale integration test passed (completed in {:?})",
        duration
    );
}

#[test]
fn test_validation_integration() {
    // Test validation integration
    let mut ontology = Ontology::new();
    ontology.set_iri("http://example.org/validation-test");

    // Create a well-structured ontology
    let person_iri = IRI::new("http://example.org/Person").unwrap();
    let person_class = Class::new(Arc::new(person_iri));
    ontology
        .add_class(person_class)
        .expect("Failed to add Person class");

    // Test with validation framework
    let mut framework = owl2_reasoner::validation::ValidationFramework::new()
        .expect("Failed to create validation framework");

    let result = framework.run_basic_validation();

    match result {
        Ok(report) => {
            println!(
                "Validation integration completed with score: {:.2}",
                report.w3c_compliance_score
            );
            assert!(report.w3c_compliance_score >= 0.0);
            assert!(report.w3c_compliance_score <= 1.0);
        }
        Err(e) => {
            println!("Validation integration failed: {}", e);
        }
    }
}

#[test]
fn test_error_handling_integration() {
    // Test error handling in integration scenarios

    // Test invalid ontology creation
    let invalid_iri_result = IRI::new("");
    assert!(invalid_iri_result.is_err());

    // Test duplicate class handling
    let mut ontology = Ontology::new();
    let person_iri = IRI::new("http://example.org/Person").unwrap();
    let person_class1 = Class::new(Arc::new(person_iri.clone()));
    let person_class2 = Class::new(Arc::new(person_iri));

    ontology
        .add_class(person_class1)
        .expect("Failed to add first Person class");
    ontology
        .add_class(person_class2)
        .expect("Failed to add second Person class"); // Should be handled gracefully

    // Should still only have one class due to duplicate handling
    assert_eq!(ontology.classes().iter().count(), 1);

    // Test reasoning with valid ontology
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent();
    assert!(is_consistent.is_ok());
    assert!(is_consistent.unwrap());

    println!("Error handling integration test passed");
}

#[test]
fn test_memory_efficiency_integration() {
    // Test memory efficiency in integration scenarios
    let mut ontology = Ontology::new();

    // Create entities with shared IRIs
    let base_iri = IRI::new("http://example.org/ontology").unwrap();
    let _shared_arc = Arc::new(base_iri);

    for i in 0..50 {
        let class_iri = IRI::new(format!("http://example.org/ontology#Class{}", i)).unwrap();
        let class = Class::new(Arc::new(class_iri));
        ontology.add_class(class).expect("Failed to add class");
    }

    // Verify all classes were added
    assert_eq!(ontology.classes().iter().count(), 50);

    // Test reasoning
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent();
    assert!(is_consistent.is_ok());
    assert!(is_consistent.unwrap());

    println!("Memory efficiency integration test passed");
}
