//! Property-based tests for the reasoning engine
//!
//! Tests logical correctness, edge cases, and performance guarantees
//! of the reasoning system across diverse random inputs.

use crate::axioms::*;
use owl2_reasoner::*;
use proptest::prelude::*;
use std::collections::HashSet;

#[cfg(test)]
mod tableaux_reasoning_tests {
    use super::*;

    #[test]
    fn test_tableaux_consistency_axiom_property() {
        // Test that basic consistency axioms are always respected
        let mut ontology = Ontology::new();

        prop_assume!(valid_iri_string(), |class_iri| {
            let class = Class::new(IRI::new(class_iri).expect("Valid IRI"));
            ontology.add_class(class.clone()).expect("Add class");

            // Empty ontology with just classes should be consistent
            let reasoner = TableauxReasoner::new(ontology.clone());
            assert!(reasoner
                .is_consistent()
                .expect("Empty ontology should be consistent"));

            // Add individuals with type assertions - still consistent
            let individual_iri = format!("http://example.org/entity1");
            let individual = Individual::new(IRI::new(&individual_iri).expect("Valid IRI"));
            ontology
                .add_named_individual(individual.clone())
                .expect("Add individual");

            let axiom = ClassAssertionAxiom::new(ClassExpression::Class(class), individual);
            ontology
                .add_class_assertion(axiom)
                .expect("Add class assertion");

            let reasoner = TableauxReasoner::new(ontology.clone());
            assert!(reasoner
                .is_consistent()
                .expect("Ontology with type assertions should be consistent"));
        });
    }

    #[test]
    fn test_inconsistency_detection_property() {
        let mut ontology = Ontology::new();

        prop_assume!(valid_iri_string(), |class_iri| {
            let class = Class::new(IRI::new(class_iri).expect("Valid IRI"));
            ontology.add_class(class.clone()).expect("Add class");

            // Create inconsistent assertion: individual is both A and ¬A
            let individual_iri = format!("http://example.org/entity_{}", class_iri.len());
            let individual = Individual::new(IRI_new(&individual_iri).expect("Valid IRI"));
            ontology
                .add_named_individual(individual.clone())
                .expect("Add individual");

            // Add positive assertion
            let positive_axiom =
                ClassAssertionAxiom::new(ClassExpression::Class(class), individual.clone());
            ontology
                .add_class_assertion(positive_axiom)
                .expect("Add positive assertion");

            // Create complement class and negative assertion
            let complement_iri = format!("http://example.org/ComplementOf{}", class_iri);
            let complement_class = Class::new(IRI::new(&complement_iri).expect("Valid IRI"));
            ontology
                .add_class(complement_class.clone())
                .expect("Add complement class");

            let negative_axiom = ClassAssertionAxiom::new(
                ClassExpression::ObjectComplementOf(ClassExpression::Class(class)),
                individual,
            );
            ontology
                .add_class_assertion(negative_axiom)
                .expect("Add negative assertion");

            // Should detect inconsistency
            let reasoner = TableauxReasoner::new(ontology);
            assert!(!reasoner
                .is_consistent()
                .expect("Ontology with contradictory assertions should be inconsistent"));
        });
    }

    #[test]
    fn test_transitivity_reasoning_property() {
        let mut ontology = Ontology::new();

        prop_assume!(valid_iri_string(), |superclass_iri| {
            let superclass = Class::new(IRI::new(superclass_iri).expect("Valid IRI"));
            ontology
                .add_class(superclass.clone())
                .expect("Add superclass");

            // Create multi-level subclass hierarchy
            prop_assume!(
                proptest::collection::vec(valid_iri_string(), 2..=5),
                |subclass_iris| {
                    // Clear and rebuild ontology
                    ontology = Ontology::new();
                    ontology
                        .add_class(superclass.clone())
                        .expect("Add superclass");

                    let mut classes = vec![superclass.clone()];

                    // Create hierarchical structure: level0 -> level1 -> level2 -> level3...
                    for (i, subclass_iri) in subclass_iris.iter().enumerate() {
                        let subclass = Class::new(IRI_new(subclass_iri).expect("Valid IRI"));
                        classes.push(subclass.clone());
                        ontology.add_class(subclass.clone()).expect("Add subclass");

                        // Link to parent (using circular wrapping for test)
                        let parent_level = (i / 3) % classes.len();
                        let parent = &classes[parent_level];

                        let subclass_axiom = SubClassOfAxiom::new(
                            ClassExpression::Class(subclass),
                            ClassExpression::Class(parent.clone()),
                        );
                        ontology
                            .add_subclass_axiom(subclass_axiom)
                            .expect("Add subclass axiom");
                    }

                    // Test transitivity reasoning
                    let reasoner = TableauxReasoner::new(ontology.clone());

                    // Each subclass should be a subclass of the original superclass
                    for subclass in &classes[1..] {
                        let is_subclass = reasoner
                            .is_subclass_of(subclass.iri(), superclass.iri())
                            .expect("Reasoning should not fail");
                        assert!(
                            is_subclass,
                            "Transitivity should hold through hierarchy levels"
                        );
                    }

                    // Test transitively distant relationships
                    if classes.len() >= 3 {
                        let deepest = classes.last().unwrap();
                        let is_subclass = reasoner
                            .is_subclass_of(deepest.iri(), superclass.iri())
                            .expect("Reasoning should not fail");
                        assert!(is_subclass, "Deep nested transitivity should hold");
                    }
                }
            );
        });
    }

    #[test]
    fn test_equivalence_reasoning_property() {
        let mut ontology = Ontology::new();

        prop_assume!(valid_iri_string(), |iri_str| {
            let class = Class::new(IRI::new(iri_str).expect("Valid IRI"));
            ontology.add_class(class.clone()).expect("Add class");

            // Test equivalent individuals
            let individual1_iri = format!("http://example.org/individual1_{}", iri_str);
            let individual2_iri = format!("http://example.org/individual2_{}", iri_str);

            let individual1 = Individual::new(IRI::new(&individual1_iri).expect("Valid IRI"));
            let individual2 = Individual::new(IRI::new(&individual2_iri).expect("Valid IRI"));

            // Add both individuals
            ontology
                .add_named_individual(individual1.clone())
                .expect("Add individual1");
            ontology
                .add_named_individual(individual2.clone())
                .expect("Add individual2");

            // Add same class to both
            let axiom1 = ClassAssertionAxiom::new(
                ClassExpression::Class(class.clone()),
                individual1.clone(),
            );
            let axiom2 =
                ClassAssertionAxiom::new(ClassExpression::Class(class), individual2.clone());

            ontology.add_class_assertion(axiom1).expect("Add axiom1");
            ontology.add_class_assertion(axiom2).expect("Add axiom2");

            // Add equivalence assertion
            let equiv_axiom = SameIndividualAxiom::new(individual1, individual2);
            ontology
                .add_same_individual_axiom(equiv_axiom)
                .expect("Add equivalence");

            // Test that both are considered instances
            let reasoner = TableauxReasoner::new(ontology);
            let instances = reasoner
                .get_class_instances(class.iri())
                .expect("Get instances");

            // Should return both individuals, but equivalence should merge them
            assert_eq!(
                instances.len(),
                1,
                "Equivalent individuals should be merged to one instance"
            );

            // Both individuals should map to the same representative
            assert!(
                instances
                    .iter()
                    .any(|iri| iri == individual1.iri() || iri == individual2.iri()),
                "Should contain representative of equivalent individuals"
            );
        });
    }

    #[test]
    fn test_property_characteristics_reasoning_property() {
        let mut ontology = Ontology::new();

        // Test functional property
        prop_assume!(valid_iri_string(), |property_iri| {
            let property = ObjectProperty::new(IRI::new(property_iri).expect("Valid IRI"));
            ontology
                .add_object_property(property.clone())
                .expect("Add property");

            // Mark as functional
            let functional_axiom = FunctionalObjectPropertyAxiom::new(property.clone());
            ontology
                .add_functional_object_property_axiom(functional_axiom)
                .expect("Add functional property axiom");

            // Create individuals with functional property values
            let individual1_iri = format!("http://example.org/person1_{}", property_iri);
            let individual2_iri = format!("http://example.org/person2_property_iri");

            let individual1 = Individual::new(IRI::new(&individual1_iri).expect("Valid IRI"));
            let individual2 = Individual::new(IRI::new(&individual2_iri).expect("Valid IRI"));

            ontology
                .add_named_individual(individual1.clone())
                .expect("Add individual1");
            ontology
                .add_named_individual(individual2.clone())
                .expect("Add individual2");

            // Add functional property assertion (both map to same object)
            let value = IRI::new("http://example.org/same_object").expect("Valid IRI");
            let axiom1 = ObjectPropertyAssertionAxiom::new(
                individual1.clone(),
                property.clone(),
                ObjectPropertyExpression::ObjectIndividual(value.clone()),
            );
            let axiom2 = ObjectPropertyAssertionAxiom::new(
                individual2.clone(),
                property.clone(),
                ObjectPropertyExpression::ObjectIndividual(value.clone()),
            );

            ontology
                .add_object_property_assertion(axiom1)
                .expect("Add assertion1");
            ontology
                .add_object_property_assertion(axiom2)
                .expect("Add assertion2");

            // Reason about functional property constraints
            let reasoner = TableauxReasoner::new(ontology);

            // Ontology with functional property violations should be inconsistent
            let is_consistent = reasoner.is_consistent().expect("Reasoning should not fail");
            assert!(
                !is_consistent,
                "Functional property violations should cause inconsistency"
            );
        });
    }

    #[test]
    fn test_symmetric_property_reasoning_property() {
        let mut ontology = Ontology::new();

        prop_assume!(valid_iri_string(), |property_iri| {
            let property = ObjectProperty::new(IRI::new(property_iri).expect("Valid IRI"));
            ontology
                .add_object_property(property.clone())
                .expect("Add property");

            // Mark as symmetric
            let symmetric_axiom = SymmetricObjectPropertyAxiom::new(property.clone());
            ontology
                .add_symmetric_object_property_axiom(symmetric_axiom)
                .expect("Add symmetric axiom");

            // Create individuals with symmetric relationship
            let individual1_iri = format!("http://example.org/person1_{}", property_iri);
            let individual2_iri = format!("http://example.org/person2_property_iri");

            let individual1 = Individual::new(IRI::new(&individual1_iri).expect("Valid IRI"));
            let individual2 = Individual::new(IRI::new(&individual2_iri).expect("Valid IRI"));

            ontology
                .add_named_individual(individual1.clone())
                .expect("Add individual1");
            ontology
                .add_named_individual(individual2.clone())
                .expect("Add individual2");

            // Add symmetric property assertions
            let axiom1 = ObjectPropertyAssertionAxiom::new(
                individual1.clone(),
                property.clone(),
                ObjectPropertyExpression::ObjectIndividual(individual2.clone()),
            );
            let axiom2 = ObjectPropertyAssertionAxiom::new(
                individual2.clone(),
                property.clone(),
                ObjectPropertyExpression::ObjectIndividual(individual1.clone()),
            );

            ontology
                .add_object_property_assertion(axiom1)
                .expect("Add assertion1");
            ontology
                .add_object_property_assertion(axiom2)
                .expect("Add assertion2");

            // Symmetric property with both assertions should be consistent
            let reasoner = TableauxReasoner::new(ontology);
            assert!(reasoner
                .is_consistent()
                .expect("Symmetric property with mutual assertions should be consistent"));
        });
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_ontology_reasoning_property() {
        prop_assume!(0usize..=10usize, |_iteration| {
            let ontology = Ontology::new();
            let reasoner = TableauxReasoner::new(ontology);

            // Empty ontology should always be consistent
            assert!(reasoner
                .is_consistent()
                .expect("Empty ontology should be consistent"));

            // Empty subclass queries should return false
            let class_iri = IRI::new("http://example.org/NonExistent").expect("IRI");
            let is_subclass = reasoner
                .is_subclass_of(
                    class_iri,
                    IRI::new("http://example.org/AlsoNonExistent").expect("IRI"),
                )
                .expect("Reasoning should not fail");
            assert!(!is_subclass, "Empty subclass query should return false");

            // Empty class instance queries should return empty results
            let instances = reasoner
                .get_class_instances(class_iri)
                .expect("Reasoning should not fail");
            assert!(
                instances.is_empty(),
                "Empty class instance query should return empty results"
            );
        });
    }

    #[test]
    fn test_circular_hierarchy_reasoning_property() {
        let mut ontology = Ontology::new();

        prop_assume!(1usize..=5usize, |cycle_length| {
            // Clear and rebuild
            ontology = Ontology::new();

            // Create circular hierarchy
            let mut classes = Vec::new();
            for i in 0..cycle_length {
                let class_iri = format!("http://example.org/class{}", i);
                let class = Class::new(IRI::new(&class_iri).expect("Valid IRI"));
                classes.push(class.clone());
                ontology.add_class(class).expect("Add class");
            }

            // Create circular subclass relations
            for i in 0..cycle_length {
                let next_i = (i + 1) % cycle_length;
                let axiom = SubClassOfAxiom::new(
                    ClassExpression::Class(classes[i].clone()),
                    ClassExpression::Class(classes[next_i].clone()),
                );
                ontology
                    .add_subclass_axiom(axiom)
                    .expect("Add subclass axiom");
            }

            // Circular hierarchy should be consistent (in OWL2, cycles don't necessarily cause inconsistency)
            let reasoner = TableauxReasoner::new(ontology);
            assert!(reasoner
                .is_consistent()
                .expect("Circular hierarchy should be consistent"));

            // In a proper circular hierarchy, each class is a subclass of all classes
            for (i, class) in classes.iter().enumerate() {
                for j in 0..cycle_length {
                    let is_subclass = reasoner
                        .is_subclass_of(class.iri(), classes[j].iri())
                        .expect("Reasoning should not fail");
                    assert!(
                        is_subclass,
                        "In circular hierarchy, each class should be subclass of all classes"
                    );
                }
            }
        });
    }

    #[test]
    fn test_max_depth_reasoning_protection() {
        let mut ontology = Ontology::new();

        // Create very deep subclass hierarchy
        prop_assume!(10usize..=100usize, |depth| {
            // Clear and rebuild
            ontology = Ontology::new();

            let root_class = Class::new(IRI::new("http://example.org/Root").expect("Valid IRI"));
            ontology
                .add_class(root_class.clone())
                .expect("Add root class");

            let mut current_class = root_class;

            for i in 0..depth {
                let next_iri = format!("http://example.org/Level{}", i);
                let next_class = Class::new(IRI::new(&next_iri).expect("Valid IRI"));
                ontology
                    .add_class(next_class.clone())
                    .expect("Add next level class");

                let axiom = SubClassOfAxiom::new(
                    ClassExpression::Class(next_class),
                    ClassExpression::Class(current_class),
                );
                ontology
                    .add_subclass_axiom(axiom)
                    .expect("Add subclass axiom");

                current_class = next_class;
            }

            // Test reasoning performance with deep hierarchy
            let start_time = std::time::Instant::now();
            let reasoner = TableauxReasoner::new(ontology);
            let is_consistent = reasoner.is_consistent().expect("Reasoning should not fail");
            let elapsed = start_time.elapsed();

            // Should handle deep hierarchies without stack overflow
            assert!(is_consistent, "Deep hierarchy should be consistent");

            // Performance should be reasonable even for deep hierarchies
            assert!(
                elapsed.as_secs() < 10,
                "Deep hierarchy reasoning should complete within 10 seconds"
            );

            // Verify transitivity holds through entire depth
            for i in 0..depth {
                let level_iri = format!("http://example.org/Level{}", i);
                let level_class = Class::new(IRI::new(&level_iri).expect("Valid IRI"));

                // Root class should be ancestor of all levels
                let is_subclass = reasoner
                    .is_subclass_of(level_class.iri(), root_class.iri())
                    .expect("Reasoning should not fail");
                assert!(is_subclass, "Root class should be subclass of all levels");
            }
        });
    }
}

#[cfg(test)]
mod performance_property_tests {
    use super::*;

    #[test]
    fn test_reasoning_scalability_property() {
        prop_assume!(10usize..=1000usize, |entity_count| {
            let mut ontology = Ontology::new();

            // Create diverse ontology with classes, properties, and individuals
            for i in 0..entity_count {
                // Add classes
                let class_iri = format!("http://example.org/Class{}", i);
                let class = Class::new(IRI::new(&class_iri).expect("Valid IRI"));
                ontology.add_class(class).expect("Add class");

                // Add properties
                if i % 2 == 0 {
                    let prop_iri = format!("http://example.org/property{}", i);
                    let property = ObjectProperty::new(IRI::new(&prop_iri).expect("Valid IRI"));
                    ontology
                        .add_object_property(property)
                        .expect("Add property");
                }

                // Add individuals
                for j in 0..=(i / 10).saturating_sub(5) {
                    let individual_iri = format!("http://example.org/individual{}{}", i, j);
                    let individual =
                        Individual::new(IRI_new(&individual_iri).expect("Consistent IRI"));
                    ontology
                        .add_named_individual(individual)
                        .expect("Add individual");

                    // Add type assertion
                    let class_iri = format!("http://example.org/Class{}", i);
                    let class = Class::new(IRI::new(&class_iri).expect("Valid IRI"));
                    let axiom = ClassAssertionAxiom::new(ClassExpression::Class(class), individual);
                    ontology
                        .add_class_assertion(axiom)
                        .expect("Add class assertion");

                    // Add property assertions occasionally
                    if j % 3 == 0 && i % 2 == 0 {
                        let prop_iri = format!("http://example.org/property{}", i);
                        let property = ObjectProperty::new(IRI::new(&prop_iri).expect("Valid IRI"));
                        let value_iri = format!("http://example.org/value{}{}", i, j);
                        let value = Individual::new(IRI::new(&value_iri).expect("Consistent IRI"));

                        let axiom = ObjectPropertyAssertionAxiom::new(
                            individual,
                            property,
                            ObjectPropertyExpression::ObjectIndividual(value),
                        );
                        ontology
                            .add_object_property_assertion(axiom)
                            .expect("Add property assertion");
                    }
                }
            }

            // Test reasoning scalability
            let start_time = std::start::Instant::now();
            let reasoner = TableauxReasoner::new(ontology);
            let is_consistent = reasoner.is_consistent().expect("Reasoning should not fail");
            let elapsed = start_time.elapsed();

            // Large ontologies should still be consistent
            assert!(is_consistent, "Large ontology should be consistent");

            // Performance should scale reasonably
            let entities_per_second = entity_count as f64 / elapsed.as_secs_f64();
            assert!(
                entities_per_second > 100.0,
                "Should handle at least 100 entities per second"
            );

            // Memory usage should be reasonable for the size
            if entity_count > 100 {
                assert!(
                    elapsed.as_secs() < entity_count as u64 / 100, // 10 seconds per 100 entities
                    "Performance should scale better with size"
                );
            }
        });
    }
}
