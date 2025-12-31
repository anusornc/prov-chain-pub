//! Property-based tests for memory management
//!
//! Tests memory safety, allocation patterns, and resource cleanup
//! across random inputs to catch memory-related bugs.

use owl2_reasoner::*;
use proptest::prelude::*;
use std::sync::Arc;

#[cfg(test)]
mod arena_allocator_tests {
    use super::*;

    #[test]
    fn test_arena_allocation_safety_property() {
        prop_assume!(proptest::collection::vec("test", 1..=100), |strings| {
            let arena = crate::parser::arena::ParserArena::with_capacity(strings.len());

            // Test allocation of various string types
            for (i, s) in strings.iter().enumerate() {
                let allocated = arena.alloc_str(s);
                assert_eq!(allocated, s, "Allocated string should match input");
                assert_eq!(allocated.len(), s.len(), "Length should be preserved");

                // Test that allocated strings are accessible throughout arena lifetime
                assert_eq!(
                    arena.get_string(allocated).unwrap(),
                    s,
                    "String should be retrievable from arena"
                );
            }

            // Test that all allocated strings are still accessible
            for s in &strings {
                assert!(
                    arena.get_string(s.as_str()).is_some(),
                    "All allocated strings should be accessible"
                );
            }
        });
    }

    #[test]
    fn test_arena_capacity_growth_property() {
        prop_assume!(1usize..=1000usize, |capacity| {
            let arena = crate::parser::arena::ParserArena::with_capacity(capacity);

            let mut allocated_count = 0;

            // Allocate until we hit the initial capacity
            for i in 0..=capacity {
                let test_str = format!("test_string_{}", i);
                if arena.get_string(&test_str).is_none() {
                    let _allocated = arena.alloc_str(&test_str);
                    allocated_count += 1;
                } else {
                    break;
                }
            }

            // Should be able to allocate at least the initial capacity
            assert!(
                allocated_count >= capacity.saturating_sub(1),
                "Should be able to allocate at least capacity-1 strings"
            );
        });
    }

    #[test]
    fn test_arena_vector_allocation_property() {
        prop_assume!(
            proptest::collection::vec(1i32..=100i32, 1..=50),
            |numbers| {
                let arena = crate::parser::arena::ParserArena::new();

                // Allocate vector and test mutable access
                let vec_slice = arena.alloc_vec(numbers);

                assert_eq!(vec_slice.len(), numbers.len(), "Slice length should match");

                // Test that we can modify the slice (this tests the unsafe conversion)
                for (i, &num) in numbers.iter().enumerate() {
                    assert_eq!(vec_slice[i], *num, "Vector elements should match");
                }

                // Verify no duplicate values (test that slice is a proper copy)
                let unique_numbers: std::collections::HashSet<_> = numbers.iter().collect();
                let slice_unique_numbers: std::collections::HashSet<_> = vec_slice.iter().collect();
                assert_eq!(
                    unique_numbers, slice_unique_numbers,
                    "Slice should contain same elements as original"
                );
            }
        );
    }
}

#[cfg(test)]
mod tableaux_memory_tests {
    use super::*;
    use owl2_reasoner::reasoning::tableaux::{MemoryManager, NodeId, TableauxNode};

    #[test]
    fn test_memory_manager_node_allocation_property() {
        let memory_manager = MemoryManager::new();

        prop_assume!(1u32..=1000u32, |node_id_num| {
            let node_id = NodeId::new(node_id_num);
            let node = TableauxNode::new(node_id);

            // Allocate node
            let allocated_node = memory_manager
                .allocate_node(node)
                .expect("Node allocation should succeed");

            // Verify node properties are preserved
            assert_eq!(allocated_node.id(), node_id);
            assert!(
                allocated_node.class_expressions().is_empty(),
                "New node should have no class expressions"
            );
        });
    }

    #[test]
    fn test_memory_manager_mutation_tracking_property() {
        let memory_manager = MemoryManager::with_tracking();

        prop_assume!(1u32..=100u32, |node_id_num| {
            let node_id = NodeId::new(node_id_num);
            let node = TableauxNode::new(node_id);

            // Allocate node
            let allocated_node = memory_manager
                .allocate_node(node)
                .expect("Node allocation should succeed");

            // Verify mutation tracking is enabled
            assert!(
                memory_manager.is_tracking_enabled(),
                "Tracking should be enabled"
            );

            // Get mutation stats
            let stats = memory_manager
                .get_mutation_stats()
                .expect("Should get stats");
            assert_eq!(
                stats.nodes_allocated,
                (node_id_num + 1) as u32,
                "Node allocation count should match"
            );

            // Get change log
            let change_log = memory_manager
                .get_change_log()
                .expect("Should get change log");
            assert!(
                change_log.len() >= (node_id_num + 1) as usize,
                "Change log should contain all allocations"
            );
        });
    }

    #[test]
    fn test_memory_manager_checkpoint_rollback_property() {
        let memory_manager = MemoryManager::with_tracking();

        prop_assume!(2u32..=20u32, |checkpoint_count| {
            // Create initial checkpoint
            let initial_checkpoint = memory_manager
                .create_checkpoint()
                .expect("Should create initial checkpoint");

            let mut allocated_nodes = Vec::new();

            // Allocate nodes
            for i in 0..checkpoint_count {
                let node_id = NodeId::new(i + 1);
                let node = TableauxNode::new(node_id);
                let allocated_node = memory_manager
                    .allocate_node(node)
                    .expect("Node allocation should succeed");
                allocated_nodes.push((node_id, allocated_node));
            }

            // Create checkpoint
            let middle_checkpoint = memory_manager
                .create_checkpoint()
                .expect("Should create middle checkpoint");

            // Verify intermediate state
            let middle_stats = memory_manager
                .get_mutation_stats()
                .expect("Should get stats");
            assert!(middle_stats.nodes_allocated >= checkpoint_count as u32);

            // Roll back to middle checkpoint
            memory_manager
                .rollback_to_checkpoint(middle_checkpoint)
                .expect("Rollback should succeed");

            // Verify state after rollback
            let rollback_stats = memory_manager
                .get_mutation_stats()
                .expect("Should get stats after rollback");
            assert_eq!(
                rollback_stats.nodes_allocated, checkpoint_count as u32,
                "Should have checkpoint_count allocations after rollback"
            );

            // Nodes allocated after rollback should still exist
            for i in 0..checkpoint_count {
                let node_id = NodeId::new(i + 1);
                assert!(
                    memory_manager.get_node(node_id).is_some(),
                    "Nodes allocated before checkpoint should still exist"
                );
            }
        });
    }

    #[test]
    fn test_memory_manager_interning_property() {
        let memory_manager = MemoryManager::with_tracking();

        prop_assume!(
            proptest::collection::vec("test_string_".to_string(), 1..=100),
            |strings| {
                let mut interned_strings = Vec::new();

                // Intern strings
                for s in &strings {
                    let interned = memory_manager
                        .intern_string(s)
                        .expect("String interning should succeed");
                    interned_strings.push(interned);
                }

                // Test that identical strings return the same interned value
                for (i, s) in strings.iter().enumerate() {
                    assert_eq!(
                        interned_strings[i], *s,
                        "Interned string should match original"
                    );

                    // Test that interning same string again returns same value
                    let interned_again = memory_manager
                        .intern_string(s)
                        .expect("Re-interning should succeed");
                    assert_eq!(
                        interned_again, interned_strings[i],
                        "Re-interning should return same value"
                    );
                }

                // Test that different strings return different values
                for (i, s1) in strings.iter().enumerate() {
                    for (j, s2) in strings.iter().enumerate() {
                        if i != j && s1 != s2 {
                            assert_ne!(
                                interned_strings[i], interned_strings[j],
                                "Different strings should have different interned values"
                            );
                        }
                    }
                }
            }
        );
    }
}

#[cfg(test)]
mod concurrent_memory_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_concurrent_arena_access_property() {
        let arena = Arc::new(crate::parser::arena::ParserArena::new());
        let test_strings = vec![
            "string1".to_string(),
            "string2".to_string(),
            "string3".to_string(),
        ];

        // Test concurrent string allocation
        let handles: Vec<_> = test_strings
            .into_iter()
            .map(|s| {
                let arena_clone = arena.clone();
                thread::spawn(move || {
                    let allocated = arena_clone.alloc_str(&s);
                    (s.clone(), allocated)
                })
            })
            .collect();

        // Wait for all threads and collect results
        let mut results = Vec::new();
        for handle in handles {
            let (original, allocated) = handle.join().expect("Thread should complete");
            assert_eq!(
                original, allocated,
                "Allocated string should match original"
            );
            results.push((original, allocated));
        }

        // Verify all strings are accessible
        for (original, allocated) in &results {
            assert_eq!(
                arena.get_string(allocated).unwrap(),
                original,
                "All allocated strings should be accessible"
            );
        }
    }

    #[test]
    fn test_concurrent_tableaux_reasoning_property() {
        prop_assume!(1usize..=10usize, |thread_count| {
            let mut ontology = Ontology::new();

            // Add basic class structure
            let person_class =
                Class::new(IRI::new("http://example.org/Person").expect("Valid IRI"));
            let animal_class = Class::new(IRI_new("http://example.org/Animal").expect("Valid IRI"));

            ontology
                .add_class(person_class.clone())
                .expect("Add person class");
            ontology
                .add_class(animal_class.clone())
                .expect("Add animal class");

            // Add subclass relation
            let subclass_axiom = SubClassOfAxiom::new(
                ClassExpression::Class(animal_class),
                ClassExpression::Class(person_class),
            );
            ontology
                .add_subclass_axiom(subclass_axiom)
                .expect("Add subclass axiom");

            let ontology = Arc::new(ontology);

            // Create multiple reasoning threads
            let handles: Vec<_> = (0..thread_count)
                .map(|i| {
                    let ontology_clone = ontology.clone();
                    thread::spawn(move || {
                        let reasoner = TableauxReasoner::new((*ontology_clone).clone());

                        // Test consistency checking
                        let start_time = std::time::Instant::now();
                        let is_consistent =
                            reasoner.is_consistent().expect("Reasoning should not fail");
                        let elapsed = start_time.elapsed();

                        // Test subclass reasoning
                        let is_subclass = reasoner
                            .is_subclass_of(
                                IRI::new("http://example.org/Animal").expect("Valid IRI"),
                                IRI::new("http://example.org/Person").expect("Valid IRI"),
                            )
                            .expect("Reasoning should not fail");

                        (i, is_consistent, is_subclass, elapsed)
                    })
                })
                .collect();

            // Wait for all threads
            for handle in handles {
                let (thread_id, is_consistent, is_subclass, elapsed) =
                    handle.join().expect("Thread should complete");

                // All threads should agree on results
                assert!(is_consistent, "All threads should agree on consistency");
                assert!(is_subclass, "All threads should agree on subclass relation");

                // Performance should be reasonable
                assert!(
                    elapsed.as_secs() < 5,
                    "Reasoning should complete within 5 seconds"
                );
            }
        });
    }
}

#[cfg(test)]
mod memory_stress_tests {
    use super::*;

    #[test]
    fn test_large_ontology_memory_usage_property() {
        prop_assume!(100usize..=1000usize, |class_count| {
            let memory_manager = MemoryManager::new();
            let mut ontology = Ontology::new();

            let mut allocated_count = 0;

            // Create large ontology
            for i in 0..class_count {
                let class_iri = format!("http://example.org/class{}", i);
                let class = Class::new(IRI::new(&class_iri).expect("Valid IRI"));
                ontology.add_class(class).expect("Add class");

                // Allocate individuals for each class
                for j in 0..=5 {
                    let individual_iri = format!("http://example.org/individual{}_{}", i, j);
                    let individual = Individual::new(IRI::new(&individual_iri).expect("Valid IRI"));
                    ontology
                        .add_named_individual(individual)
                        .expect("Add individual");

                    let axiom = ClassAssertionAxiom::new(
                        ClassExpression::Class(Class::new(
                            IRI::new(&class_iri).expect("Valid IRI"),
                        )),
                        individual.clone(),
                    );
                    ontology
                        .add_class_assertion(axiom)
                        .expect("Add class assertion");

                    allocated_count += 1;
                }
            }

            // Test reasoning performance with large ontology
            let start_time = std::time::Instant::now();
            let reasoner = TableauxReasoner::new(ontology);
            let is_consistent = reasoner.is_consistent().expect("Reasoning should not fail");
            let elapsed = start_time.elapsed();

            assert!(is_consistent, "Large ontology should be consistent");

            // Performance should be reasonable for the size
            let expected_time_per_entity = allocated_count as f64 / 1000.0; // 1ms per 1000 entities
            assert!(
                elapsed.as_millis() as f64 <= expected_time_per_entity * 1000.0 * 10.0,
                "Performance should be reasonable for ontology size"
            );

            // Memory usage should be proportional to entity count
            let stats = memory_manager.get_mutation_stats();
            if let Ok(stats) = stats {
                assert!(stats.nodes_allocated > 0, "Should have allocated nodes");
                assert!(stats.total_allocations > 0, "Should have some allocations");
            }
        });
    }

    #[test]
    fn test_memory_leak_prevention_property() {
        prop_assume!(50usize..=200usize, |allocation_count| {
            let memory_manager = MemoryManager::new();

            // Create and reason with large ontologies repeatedly
            for iteration in 0..5 {
                let mut ontology = Ontology::new();

                // Create test ontology
                for i in 0..allocation_count {
                    let class_iri = format!("http://example.org/class{}_{}", iteration, i);
                    let class = Class::new(IRI::new(&class_iri).expect("Valid IRI"));
                    ontology.add_class(class).expect("Add class");

                    for j in 0..=3 {
                        let individual_iri =
                            format!("http://example.org/individual{}_{}_{}", iteration, i, j);
                        let individual =
                            Individual::new(IRI::new(&individual_iri).expect("Valid IRI"));
                        ontology
                            .add_named_individual(individual)
                            .expect("Add individual");
                    }
                }

                // Test reasoning
                let reasoner = TableauxReasoner::new(ontology);
                assert!(reasoner.is_consistent().expect("Reasoning should not fail"));

                // Get memory stats
                if let Ok(stats) = memory_manager.get_mutation_stats() {
                    // Memory usage should not grow unboundedly
                    assert!(
                        stats.total_allocations < allocation_count as u64 * iteration as u64 * 10,
                        "Memory usage should be reasonable"
                    );
                }
            }
        });
    }
}
