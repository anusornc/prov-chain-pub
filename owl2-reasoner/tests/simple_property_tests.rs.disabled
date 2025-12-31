//! Simple property-based tests that work with current API
//!
//! These tests focus on basic IRI and ontology functionality
//! that doesn't depend on the complex tableaux reasoning engine.

use owl2_reasoner::*;
use proptest::prelude::*;

/// Strategy for generating valid IRI strings
fn valid_iri_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple HTTP IRIs
        (1usize..=10usize).prop_map(|i| format!("http://example.org/test{}", i)),
        (1usize..=10usize).prop_map(|i| format!("https://example.com/item{}", i)),
        // Local IRIs
        (1usize..=10usize).prop_map(|i| format!("urn:test:entity{}", i)),
        (1usize..=10usize).prop_map(|i| format!("file:/tmp/test{}", i)),
    ]
}

#[test]
fn test_iri_creation_roundtrip_property() {
    prop_assume!(valid_iri_strategy(), |iri_string| {
        // Test that valid IRIs can be created and round-trip correctly
        let iri = IRI::new(&iri_string).expect("Valid IRI should parse");

        // Round-trip property: IRI -> string -> IRI should be equal
        assert_eq!(iri.as_str(), iri_string, "IRI should round-trip unchanged");

        // All IRIs should have string representation
        let string_repr = iri.as_str();
        assert!(!string_repr.is_empty(), "IRI string should not be empty");
        assert!(string_repr.len() >= 10, "IRI should have reasonable length");
    });
}

#[test]
fn test_iri_equality_property() {
    prop_assume!(valid_iri_strategy(), |iri_string1| {
        let iri1 = IRI::new(&iri_string1).expect("Valid IRI should parse");

        prop_assume!(valid_iri_strategy(), |iri_string2| {
            let iri2 = IRI::new(&iri_string2).expect("Valid IRI should parse");

            // Equality should be based on string content
            assert_eq!(
                iri1 == iri2,
                iri_string1 == iri_string2,
                "IRI equality should match string equality"
            );

            // If equal, string representations should match
            if iri1 == iri2 {
                assert_eq!(
                    iri1.as_str(),
                    iri2.as_str(),
                    "Equal IRIs should have identical string representations"
                );
            }

            // Hash property: equal IRIs should have same hash
            if iri1 == iri2 {
                assert_eq!(
                    std::collections::hash_map::DefaultHasher::new().hash(&iri1),
                    std::collections::hash_map::DefaultHasher::new().hash(&iri2),
                    "Equal IRIs should have same hash"
                );
            }
        });
    });
}

#[test]
fn test_ontology_basic_operations_property() {
    prop_assume!(
        proptest::collection::vec(valid_iri_strategy(), 1..=5),
        |class_iris| {
            let mut ontology = Ontology::new();

            // Add classes to ontology
            for iri_string in &class_iris {
                let class = Class::new(IRI::new(iri_string).expect("Valid IRI"));
                ontology
                    .add_class(class)
                    .expect("Should add class successfully");
            }

            // Verify all classes are present
            let added_classes: Vec<String> = ontology
                .classes()
                .map(|class| class.iri().as_str().to_string())
                .collect();

            // Check that we added all the classes
            assert_eq!(
                added_classes.len(),
                class_iris.len(),
                "Should have same number of classes as added"
            );

            for original_iri in &class_iris {
                assert!(
                    added_classes.contains(original_iri),
                    "Should contain original class IRI: {}",
                    original_iri
                );
            }

            // Each class should be unique
            let unique_classes: std::collections::HashSet<_> = added_classes.iter().collect();
            assert_eq!(
                unique_classes.len(),
                added_classes.len(),
                "All classes should be unique"
            );
        }
    );
}

#[test]
fn test_class_axiom_property() {
    prop_assume!(valid_iri_strategy(), |parent_iri| {
        let parent_class = Class::new(IRI::new(&parent_iri).expect("Valid IRI"));

        prop_assume!(valid_iri_strategy(), |child_iri| {
            let child_class = Class::new(IRI::new(&child_iri).expect("Valid IRI"));

            let mut ontology = Ontology::new();
            ontology
                .add_class(parent_class.clone())
                .expect("Add parent");
            ontology.add_class(child_class.clone()).expect("Add child");

            // Create subclass axiom
            let subclass_axiom = SubClassOfAxiom::new(
                ClassExpression::Class(child_class),
                ClassExpression::Class(parent_class),
            );
            ontology
                .add_subclass_axiom(subclass_axiom)
                .expect("Add subclass axiom");

            // Should have exactly one subclass axiom
            let subclass_count = ontology.subclass_axioms().count();
            assert_eq!(subclass_count, 1, "Should have exactly one subclass axiom");

            // Verify the axiom relationship
            let axiom = ontology
                .subclass_axioms()
                .next()
                .expect("Should have subclass axiom");
            assert_eq!(
                axiom.sub_class().as_class().unwrap().iri().as_str(),
                &child_iri
            );
            assert_eq!(
                axiom.super_class().as_class().unwrap().iri().as_str(),
                &parent_iri
            );
        });
    });
}

#[test]
fn test_individual_assertion_property() {
    prop_assume!(valid_iri_strategy(), |class_iri| {
        let class = Class::new(IRI::new(&class_iri).expect("Valid IRI"));

        prop_assume!(
            proptest::collection::vec(valid_iri_strategy(), 1..=3),
            |individual_iris| {
                let mut ontology = Ontology::new();
                ontology.add_class(class.clone()).expect("Add class");

                // Add individuals and class assertions
                for individual_iri in &individual_iris {
                    let individual = Individual::new(IRI::new(individual_iri).expect("Valid IRI"));
                    ontology
                        .add_named_individual(individual.clone())
                        .expect("Add individual");

                    // Add class assertion
                    let assertion =
                        ClassAssertionAxiom::new(ClassExpression::Class(class.clone()), individual);
                    ontology
                        .add_class_assertion(assertion)
                        .expect("Add class assertion");
                }

                // Verify all individuals are asserted to the class
                let class_assertion_count = ontology.class_assertions().count();
                assert_eq!(
                    class_assertion_count,
                    individual_iris.len(),
                    "Should have same number of assertions as individuals"
                );

                // Each individual should be named
                let named_individual_count = ontology.named_individuals().count();
                assert_eq!(
                    named_individual_count,
                    individual_iris.len(),
                    "Should have same number of named individuals"
                );
            }
        );
    });
}

/// Generate a sequence of related IRIs (like a hierarchy)
fn hierarchy_iri_strategy() -> impl Strategy<Value = Vec<String>> {
    prop_oneof![
        // Simple linear hierarchy: A -> B -> C -> D
        proptest::collection::vec(
            (1usize..=5usize).prop_map(|i| format!("http://example.org/level{}", i)),
            2..=4
        ),
        // Animal-like hierarchy
        prop_oneof![
            prop::value(vec![
                "http://example.org/Animal".to_string(),
                "http://example.org/Mammal".to_string(),
                "http://example.org/Dog".to_string(),
            ]),
            prop::value(vec![
                "http://example.org/Vehicle".to_string(),
                "http://example.org/Car".to_string(),
                "http://example.org/ElectricCar".to_string(),
            ]),
            prop::value(vec![
                "http://example.org/Organization".to_string(),
                "http://example.org/Company".to_string(),
                "http://example.org/Startup".to_string(),
            ]),
        ],
    ]
}

#[test]
fn test_simple_hierarchy_property() {
    prop_assume!(hierarchy_iri_strategy(), |hierarchy_iris| {
        let mut ontology = Ontology::new();
        let mut classes = Vec::new();

        // Create classes
        for iri_string in &hierarchy_iris {
            let class = Class::new(IRI::new(iri_string).expect("Valid IRI"));
            classes.push(class);
            ontology.add_class(class).expect("Add class");
        }

        // Create hierarchical relationships
        for i in 1..classes.len() {
            let subclass_axiom = SubClassOfAxiom::new(
                ClassExpression::Class(classes[i].clone()),
                ClassExpression::Class(classes[i - 1].clone()),
            );
            ontology
                .add_subclass_axiom(subclass_axiom)
                .expect("Add subclass axiom");
        }

        // Should have (n-1) subclass relationships for n classes
        let expected_relations = hierarchy_iris.len().saturating_sub(1);
        assert_eq!(
            ontology.subclass_axioms().count(),
            expected_relations,
            "Should have exactly {} subclass relations",
            expected_relations
        );

        // All original classes should be present
        let class_count = ontology.classes().count();
        assert_eq!(
            class_count,
            hierarchy_iris.len(),
            "Should have exactly {} classes",
            hierarchy_iris.len()
        );
    });
}
