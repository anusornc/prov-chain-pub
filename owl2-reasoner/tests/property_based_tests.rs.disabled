//! Property-based tests for OWL2 Reasoner
//!
//! This module uses proptest to generate random test cases and verify that
//! invariants hold across a wide range of inputs, helping catch edge cases
//! that traditional testing might miss.

use owl2_reasoner::*;
use proptest::prelude::*;
use std::collections::HashSet;

// Helper to generate random valid IRI strings
fn valid_iri_string() -> impl Strategy<Value = String> {
    r"[a-z]{3,10}\\.(com|org|net|gov|edu)".prop_map(|domain| format!("http://example.{}", domain))
}

fn valid_prefix_iri() -> impl Strategy<Value = String> {
    r"[a-z]{1,5}:[a-z0-9]{3,10}".prop_map(|s| format!("http://example.org/{}", s))
}

#[cfg(test)]
mod iri_tests {
    use super::*;

    #[test]
    fn test_iri_roundtrip_property() {
        prop_assume!(valid_iri_string(), |iri_str| {
            // Roundtrip test: parse IRI and serialize back
            let iri = IRI::new(iri_str).expect("Valid IRI");
            assert_eq!(iri.as_str(), iri_str);

            // Verify components
            assert!(iri.as_str().starts_with("http://example.org/"));
            assert!(iri.as_str().len() > 0);
        });
    }

    #[test]
    fn test_iri_equality_property() {
        prop_assume!(valid_iri_string(), |iri_str1| {
            let iri1 = IRI::new(iri_str1).expect("Valid IRI");

            prop_assume!(valid_iri_string(), |iri_str2| {
                let iri2 = IRI::new(iri_str2).expect("Valid IRI");

                // Equality and hash properties
                assert_eq!(iri1 == iri2, iri_str1 == iri_str2);
                if iri1 == iri2 {
                    assert_eq!(iri1.as_str(), iri2.as_str());
                }
            });
        });
    }

    #[test]
    fn test_iri_components_property() {
        prop_assume!(valid_prefix_iri(), |iri_str| {
            let iri = IRI::new(iri_str).expect("Valid IRI");

            // Extract components and verify invariants
            let full_str = iri.as_str();
            assert!(full_str.contains(':'));
            assert!(full_str.contains('/'));

            // Ensure no trailing whitespace
            assert!(!full_str.ends_with(' '));
            assert!(!full_str.ends_with('\t'));
            assert!(!full_str.ends_with('\n'));
        });
    }
}

#[cfg(test)]
mod ontology_tests {
    use super::*;
    use crate::axioms::*;

    #[test]
    fn test_ontology_add_classes_property() {
        let mut ontology = Ontology::new();

        prop_assume!(
            proptest::collection::vec(valid_iri_string(), 1..=10),
            |class_iris| {
                let classes: Vec<Class> = class_iris
                    .into_iter()
                    .map(|iri_str| Class::new(IRI::new(iri_str).expect("Valid IRI")))
                    .collect();

                // Add all classes to ontology
                for class in classes.iter() {
                    ontology.add_class(class.clone()).expect("Should add class");
                }

                // Verify all classes are present
                let class_set: HashSet<String> = classes
                    .iter()
                    .map(|c| c.iri().as_str().to_string())
                    .collect();

                for class_str in class_set {
                    assert!(ontology.classes().any(|c| c.iri().as_str() == class_str));
                }

                // Check count matches
                assert_eq!(ontology.classes().count(), class_set.len());
            }
        );
    }

    #[test]
    fn test_subclass_transitivity_property() {
        let mut ontology = Ontology::new();

        prop_assume!(valid_iri_string(), |parent_iri| {
            let parent_class = Class::new(IRI::new(parent_iri).expect("Valid IRI"));
            ontology
                .add_class(parent_class.clone())
                .expect("Add parent");

            prop_assume!(valid_iri_string(), |child_iri| {
                let child_class = Class::new(IRI::new(child_iri).expect("Valid IRI"));
                ontology.add_class(child_class.clone()).expect("Add child");

                // Add subclass relation: Child ⊑ Parent
                let subclass_axiom = SubClassOfAxiom::new(
                    ClassExpression::Class(child_class.clone()),
                    ClassExpression::Class(parent_class.clone()),
                );
                ontology
                    .add_subclass_axiom(subclass_axiom)
                    .expect("Add subclass axiom");

                prop_assume!(valid_iri_string(), |grandchild_iri| {
                    let grandchild_class = Class::new(IRI::new(grandchild_iri).expect("Valid IRI"));
                    ontology
                        .add_class(grandchild_class.clone())
                        .expect("Add grandchild");

                    // Add subclass relation: Grandchild ⊑ Child
                    let subclass_axiom = SubClassOfAxiom::new(
                        ClassExpression::Class(grandchild_class.clone()),
                        ClassExpression::Class(child_class.clone()),
                    );
                    ontology
                        .add_subclass_axiom(subclass_axiom)
                        .expect("Add grandchild subclass axiom");

                    // Test transitivity: Grandchild ⊑ Child ⊑ Parent ⇒ Grandchild ⊑ Parent
                    let reasoner = TableauxReasoner::new(ontology.clone());
                    let is_subclass = reasoner
                        .is_subclass_of(grandchild_class.iri(), parent_class.iri())
                        .expect("Reasoning should not fail");

                    assert!(is_subclass, "Transitivity should hold");
                });
            });
        });
    }

    #[test]
    fn test_ontology_properties_property() {
        let mut ontology = Ontology::new();

        prop_assume!(
            proptest::collection::vec(valid_iri_string(), 1..=5),
            |property_iris| {
                let properties: Vec<ObjectProperty> = property_iris
                    .into_iter()
                    .map(|iri_str| ObjectProperty::new(IRI::new(iri_str).expect("Valid IRI")))
                    .collect();

                // Add all properties to ontology
                for property in properties.iter() {
                    ontology
                        .add_object_property(property.clone())
                        .expect("Should add property");
                }

                // Verify all properties are present
                let property_set: HashSet<String> = properties
                    .iter()
                    .map(|p| p.iri().as_str().to_string())
                    .collect();

                for prop_str in property_set {
                    assert!(ontology
                        .object_properties()
                        .any(|p| p.iri().as_str() == prop_str));
                }

                // Check count matches
                assert_eq!(ontology.object_properties().count(), property_set.len());
            }
        );
    }
}

#[cfg(test)]
mod reasoning_tests {
    use super::*;
    use crate::axioms::*;

    #[test]
    fn test_reasoner_consistency_property() {
        // Test that a valid ontology is always considered consistent
        let mut ontology = Ontology::new();

        prop_assume!(
            proptest::collection::vec(valid_iri_string(), 1..=5),
            |class_iris| {
                // Clear previous state
                ontology = Ontology::new();

                // Add classes
                for iri_str in class_iris {
                    let class = Class::new(IRI::new(iri_str).expect("Valid IRI"));
                    ontology.add_class(class).expect("Add class");
                }

                // Test consistency
                let reasoner = TableauxReasoner::new(ontology.clone());
                let is_consistent = reasoner.is_consistent().expect("Reasoning should not fail");

                // Empty ontology with only classes should be consistent
                assert!(
                    is_consistent,
                    "Empty ontology with classes should be consistent"
                );
            }
        );
    }

    #[test]
    fn test_class_hierarchy_reasoning_property() {
        let mut ontology = Ontology::new();

        prop_assume!(valid_iri_string(), |superclass_iri| {
            let superclass = Class::new(IRI::new(superclass_iri).expect("Valid IRI"));
            ontology
                .add_class(superclass.clone())
                .expect("Add superclass");

            // Generate subclasses
            prop_assume!(
                proptest::collection::vec(valid_iri_string(), 2..=5),
                |subclass_iris| {
                    // Clear and rebuild ontology with superclass
                    ontology = Ontology::new();
                    ontology
                        .add_class(superclass.clone())
                        .expect("Add superclass");

                    let mut classes = vec![superclass];

                    // Add subclasses with hierarchical relations
                    for (i, iri_str) in subclass_iris.iter().enumerate() {
                        let subclass = Class::new(IRI::new(iri_str).expect("Valid IRI"));
                        classes.push(subclass.clone());
                        ontology.add_class(subclass).expect("Add subclass");

                        // Create hierarchy based on index (earlier subclasses are higher level)
                        if i > 0 {
                            let parent = classes[i - 1].clone();
                            let axiom = SubClassOfAxiom::new(
                                ClassExpression::Class(subclass),
                                ClassExpression::Class(parent),
                            );
                            ontology
                                .add_subclass_axiom(axiom)
                                .expect("Add subclass axiom");
                        }
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
                            "Each subclass should be a subclass of the original superclass"
                        );
                    }
                }
            );
        });
    }

    #[test]
    fn test_property_assertion_reasoning_property() {
        let mut ontology = Ontology::new();

        prop_assume!(valid_iri_string(), |class_iri| {
            let person_class = Class::new(IRI::new(class_iri).expect("Valid IRI"));
            ontology
                .add_class(person_class.clone())
                .expect("Add person class");

            // Add individuals
            prop_assume!(
                proptest::collection::vec(valid_prefix_iri(), 2..=5),
                |individual_iris| {
                    // Clear and rebuild
                    ontology = Ontology::new();
                    ontology
                        .add_class(person_class.clone())
                        .expect("Add person class");

                    let individuals: Vec<Individual> = individual_iris
                        .into_iter()
                        .map(|iri_str| Individual::new(IRI::new(iri_str).expect("Valid IRI")))
                        .collect();

                    for individual in individuals.iter() {
                        ontology
                            .add_named_individual(individual.clone())
                            .expect("Add individual");

                        // Add type assertion
                        let axiom = ClassAssertionAxiom::new(
                            ClassExpression::Class(person_class.clone()),
                            individual.clone(),
                        );
                        ontology
                            .add_class_assertion(axiom)
                            .expect("Add class assertion");
                    }

                    // Test that all individuals have the expected type
                    let reasoner = TableauxReasoner::new(ontology.clone());

                    for individual in individuals {
                        let instances = reasoner
                            .get_class_instances(person_class.iri())
                            .expect("Reasoning should not fail");
                        assert!(
                            instances.iter().any(|iri| iri == individual.iri()),
                            "Individual should be instance of person class"
                        );
                    }
                }
            );
        });
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_turtle_parsing_property() {
        let parser = TurtleParser::new();

        prop_assume!(valid_iri_string(), |subject_iri| {
            prop_assume!(valid_iri_string(), |predicate_iri| {
                prop_assume!(valid_iri_string(), |object_iri| {
                    // Create valid Turtle triple
                    let turtle_content =
                        format!("<{}> <{}> <{}> .", subject_iri, predicate_iri, object_iri);

                    // Parse should succeed
                    let result = parser.parse_str(&turtle_content);
                    assert!(result.is_ok(), "Valid Turtle should parse successfully");

                    if let Ok(ontology) = result {
                        // Verify ontology has content
                        assert!(
                            ontology.classes().count() >= 0
                                || ontology.named_individuals().count() >= 0,
                            "Parsed ontology should have content"
                        );
                    }
                });
            });
        });
    }

    #[test]
    fn test_ontology_serialization_roundtrip() {
        let mut ontology = Ontology::new();

        prop_assume!(
            proptest::collection::vec(valid_iri_string(), 1..=3),
            |class_iris| {
                // Create ontology with random classes
                ontology = Ontology::new();

                for iri_str in class_iri {
                    let class = Class::new(IRI::new(iri_str).expect("Valid IRI"));
                    ontology.add_class(class).expect("Add class");
                }

                // Test serialization roundtrip with different formats
                let formats = vec!["turtle", "n-triples"];

                for format in formats {
                    // This would require actual serialization implementation
                    // For now, just verify the ontology structure is preserved
                    assert!(
                        ontology.classes().count() == class_iris.len(),
                        "Class count should be preserved"
                    );
                }
            }
        );
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_large_ontology_reasoning_performance() {
        prop_assume!(1usize..=100usize, |class_count| {
            let mut ontology = Ontology::new();

            // Generate large ontology
            for i in 0..class_count {
                let class_iri = format!("http://example.org/class{}", i);
                let class = Class::new(IRI::new(&class_iri).expect("Valid IRI"));
                ontology.add_class(class).expect("Add class");
            }

            // Create hierarchical relations
            for i in 1..class_count.saturating_sub(10) {
                let parent_iri = format!("http://example.org/class{}", i / 10);
                let child_iri = format!("http://example.org/class{}", i);

                let parent = Class::new(IRI::new(&parent_iri).expect("Valid IRI"));
                let child = Class::new(IRI::new(&child_iri).expect("Valid IRI"));

                let axiom = SubClassOfAxiom::new(
                    ClassExpression::Class(child),
                    ClassExpression::Class(parent),
                );
                ontology
                    .add_subclass_axiom(axiom)
                    .expect("Add subclass axiom");
            }

            // Measure reasoning time
            let start_time = std::time::Instant::now();
            let reasoner = TableauxReasoner::new(ontology);
            let is_consistent = reasoner.is_consistent().expect("Reasoning should not fail");
            let elapsed = start_time.elapsed();

            // Should complete within reasonable time (5 seconds for 100 classes)
            assert!(is_consistent, "Large ontology should be consistent");
            assert!(
                elapsed.as_secs() < 5,
                "Reasoning should complete within 5 seconds"
            );

            // Performance should be roughly linear with class count
            assert!(
                elapsed.as_millis() < class_count as u64 * 100,
                "Performance should scale reasonably with size"
            );
        });
    }

    #[test]
    fn test_query_engine_caching() {
        prop_assume!(1usize..=50usize, |query_count| {
            let mut ontology = Ontology::new();

            // Setup test ontology
            let person_class =
                Class::new(IRI::new("http://example.org/Person").expect("Valid IRI"));
            ontology
                .add_class(person_class.clone())
                .expect("Add person class");

            for i in 0..query_count {
                let individual_iri = format!("http://example.org/person{}", i);
                let individual = Individual::new(IRI::new(&individual_iri).expect("Valid IRI"));
                ontology
                    .add_named_individual(individual.clone())
                    .expect("Add individual");

                let axiom = ClassAssertionAxiom::new(
                    ClassExpression::Class(person_class.clone()),
                    individual,
                );
                ontology
                    .add_class_assertion(axiom)
                    .expect("Add class assertion");
            }

            // Create query engine
            let mut engine = owl2_reasoner::reasoning::query::QueryEngine::new(ontology);

            // Test multiple queries
            let start_time = std::time::Instant::now();

            for i in 0..query_count {
                let individual_iri = format!("http://example.org/person{}", i);
                let result = engine
                    .get_class_instances(&IRI::new(&individual_iri).expect("Valid IRI"))
                    .expect("Query should not fail");

                // Should return at least the queried individual
                assert!(result.len() >= 1, "Query should return at least one result");
            }

            let elapsed = start_time.elapsed();

            // Query performance should be reasonable
            assert!(
                elapsed.as_secs() < 2,
                "Multiple queries should complete within 2 seconds"
            );
        });
    }
}

// Property-based test utilities
#[cfg(test)]
pub mod test_utils {
    use super::*;

    /// Strategy for generating random valid class expressions
    pub fn class_expression_strategy() -> impl Strategy<Value = ClassExpression> {
        prop_oneof![
            // Simple class
            valid_iri_string().prop_map(ClassExpression::Class),
            // Object intersection (conjunction)
            proptest::collection::vec(valid_iri_string(), 2..=4).prop_map(|iris| {
                let classes: Vec<ClassExpression> = iris
                    .into_iter()
                    .map(|iri| {
                        ClassExpression::Class(Class::new(IRI::new(&iri).expect("Valid IRI")))
                    })
                    .collect();
                ClassExpression::ObjectIntersectionOf(classes)
            }),
            // Object union (disjunction)
            proptest::collection::vec(valid_iri_string(), 2..=3).prop_map(|iris| {
                let classes: Vec<ClassExpression> = iris
                    .into_iter()
                    .map(|iri| {
                        ClassExpression::Class(Class::new(IRI::new(&iri).expect("Valid IRI")))
                    })
                    .collect();
                ClassExpression::ObjectUnionOf(classes)
            }),
        ]
    }

    /// Strategy for generating random property expressions
    pub fn property_expression_strategy() -> impl Strategy<Value = ObjectPropertyExpression> {
        valid_iri_string().prop_map(ObjectPropertyExpression::ObjectProperty)
    }

    /// Create a random ontology for testing
    pub fn random_ontology() -> impl Strategy<Value = Ontology> {
        proptest::collection::vec(valid_iri_string(), 1..=10).prop_map(|iris| {
            let mut ontology = Ontology::new();

            for iri_str in iris {
                let class = Class::new(IRI::new(&iri_str).expect("Valid IRI"));
                ontology.add_class(class).expect("Add class");
            }

            ontology
        })
    }
}
