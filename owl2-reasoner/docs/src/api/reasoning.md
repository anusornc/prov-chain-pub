# Reasoning Engine

This chapter covers the reasoning engine components and APIs for performing OWL2 inference and reasoning tasks.

## Overview

The reasoning engine provides:

- **Multiple reasoner types** for different use cases and performance requirements
- **Tableaux-based reasoning** for complete OWL2 DL support
- **Profile-optimized reasoning** for EL, QL, and RL profiles
- **Caching and performance optimization** for repeated operations
- **Incremental reasoning** for dynamic ontologies

## Reasoner Types

### Simple Reasoner

The `SimpleReasoner` provides basic reasoning capabilities with good performance for most use cases.

```rust
use owl2_reasoner::{Ontology, SimpleReasoner};

// Create a simple reasoner
let mut reasoner = SimpleReasoner::new(ontology);

// Basic reasoning operations
let is_consistent = reasoner.is_consistent()?;
let is_subclass = reasoner.is_subclass_of(
    "http://example.org/Parent",
    "http://example.org/Person"
)?;

// Classification
reasoner.classify()?;

// Get classification results
let subclasses = reasoner.direct_subclasses_of("http://example.org/Person")?;
```

### Advanced Tableaux Reasoner

The `TableauxReasoner` provides complete OWL2 DL reasoning with SROIQ(D) description logic.

```rust
use owl2_reasoner::reasoning::tableaux::{TableauxReasoner, ReasoningConfig};
use std::time::Duration;

// Configure advanced reasoning
let config = ReasoningConfig {
    max_depth: 1000,
    timeout: Some(Duration::from_secs(60)),
    debug: false,
    parallel: true,
    cache_size: 10000,
    early_termination: true,
    incremental: true,
};

// Create advanced reasoner
let mut tableaux_reasoner = TableauxReasoner::with_config(&ontology, config);

// Advanced reasoning with explanations
let consistency_result = tableaux_reasoner.is_consistent()?;
if !consistency_result {
    let explanations = tableaux_reasoner.explain_inconsistency()?;
    for (i, explanation) in explanations.iter().enumerate() {
        println!("Explanation {}: {}", i + 1, explanation);
    }
}
```

### Profile-Optimized Reasoner

Profile-optimized reasoners provide specialized algorithms for OWL2 profiles.

```rust
use owl2_reasoner::reasoning::profile_optimized::ProfileOptimizedReasoner;
use owl2_reasoner::profiles::Owl2Profile;

// EL Profile Reasoner (polynomial time)
let mut el_reasoner = ProfileOptimizedReasoner::new(&ontology, Owl2Profile::EL);
let el_classification = el_reasoner.classify()?;

// QL Profile Reasoner (query rewriting)
let mut ql_reasoner = ProfileOptimizedReasoner::new(&ontology, Owl2Profile::QL);
let ql_results = ql_reasoner.query_class_instances("http://example.org/Person")?;

// RL Profile Reasoner (rule-based)
let mut rl_reasoner = ProfileOptimizedReasoner::new(&ontology, Owl2Profile::RL);
let rl_entailments = rl_reasoner.compute_entailments()?;
```

## Consistency Checking

### Basic Consistency

```rust
// Check overall ontology consistency
let is_consistent = reasoner.is_consistent()?;

if is_consistent {
    println!("Ontology is logically consistent");
} else {
    println!("Ontology contains contradictions");
}
```

### Consistency Explanations

```rust
// Get detailed explanations for inconsistency
if !reasoner.is_consistent()? {
    let explanations = reasoner.explain_inconsistency()?;

    println!("Inconsistency explanations:");
    for (i, explanation) in explanations.iter().enumerate() {
        println!("  Explanation {}:", i + 1);
        for axiom in explanation {
            println!("    - {}", axiom);
        }
    }
}
```

### Module-Based Consistency

```rust
// Check consistency of specific ontology modules
let module_iris = vec![
    "http://example.org/person-module",
    "http://example.org/location-module",
];

for module_iri in module_iris {
    let module = reasoner.extract_module(module_iri)?;
    let is_consistent = reasoner.is_module_consistent(&module)?;
    println!("Module {} is consistent: {}", module_iri, is_consistent);
}
```

## Classification

### Class Hierarchy Computation

```rust
// Compute complete class hierarchy
reasoner.classify()?;

// Get direct subclasses
let direct_subclasses = reasoner.direct_subclasses_of("http://example.org/Person")?;
println!("Direct subclasses of Person:");
for subclass in direct_subclasses {
    println!("  - {}", subclass);
}

// Get all subclasses (transitive closure)
let all_subclasses = reasoner.all_subclasses_of("http://example.org/Person")?;
println!("All subclasses of Person: {} classes", all_subclasses.len());
```

### Superclass Relationships

```rust
// Check subclass relationships
let is_subclass = reasoner.is_subclass_of(
    "http://example.org/Parent",
    "http://example.org/Person"
)?;

// Get direct superclasses
let direct_superclasses = reasoner.direct_superclasses_of("http://example.org/Person")?;
println!("Direct superclasses of Person:");
for superclass in direct_superclasses {
    println!("  - {}", superclass);
}

// Get equivalent classes
let equivalent_classes = reasoner.equivalent_classes_of("http://example.org/Person")?;
println!("Classes equivalent to Person:");
for equiv_class in equivalent_classes {
    if equiv_class != "http://example.org/Person" {
        println!("  - {}", equiv_class);
    }
}
```

### Disjointness Checking

```rust
// Check if classes are disjoint
let are_disjoint = reasoner.are_disjoint(
    "http://example.org/Male",
    "http://example.org/Female"
)?;

// Get all disjoint classes
let disjoint_classes = reasoner.disjoint_classes_of("http://example.org/Male")?;
println!("Classes disjoint with Male:");
for disjoint_class in disjoint_classes {
    println!("  - {}", disjoint_class);
}
```

## Satisfiability Testing

### Class Satisfiability

```rust
// Check if a class is satisfiable
let is_satisfiable = reasoner.is_class_satisfiable("http://example.org/Unicorn")?;

if is_satisfiable {
    println!("Unicorn is satisfiable (can have instances)");
} else {
    println!("Unicorn is unsatisfiable (contradictory definition)");
}
```

### Satisfiability Explanations

```rust
// Get explanations for unsatisfiability
if !reasoner.is_class_satisfiable("http://example.org/UnsatisfiableClass")? {
    let explanations = reasoner.explain_unsatisfiability("http://example.org/UnsatisfiableClass")?;

    println!("Unsatisfiability explanations:");
    for (i, explanation) in explanations.iter().enumerate() {
        println!("  Explanation {}:", i + 1);
        for axiom in explanation {
            println!("    - {}", axiom);
        }
    }
}
```

### Most Specific Concepts

```rust
// Find most specific classes for an individual
let individual_iri = "http://example.org/John";
let most_specific = reasoner.most_specific_classes(individual_iri)?;

println!("Most specific classes for {}:", individual_iri);
for class_iri in most_specific {
    println!("  - {}", class_iri);
}
```

## Property Reasoning

### Property Hierarchy

```rust
// Check property subsumption
let is_subproperty = reasoner.is_subproperty_of(
    "http://example.org/hasParent",
    "http://example.org/hasAncestor"
)?;

// Get sub-properties
let subproperties = reasoner.subproperties_of("http://example.org/hasAncestor")?;
println!("Sub-properties of hasAncestor:");
for prop_iri in subproperties {
    println!("  - {}", prop_iri);
}

// Get super-properties
let superproperties = reasoner.superproperties_of("http://example.org/hasParent")?;
println!("Super-properties of hasParent:");
for prop_iri in superproperties {
    println!("  - {}", prop_iri);
}
```

### Property Characteristics

```rust
// Check property characteristics
let property_iri = "http://example.org/hasParent";

let is_transitive = reasoner.is_transitive_property(property_iri)?;
let is_symmetric = reasoner.is_symmetric_property(property_iri)?;
let is_functional = reasoner.is_functional_property(property_iri)?;
let is_inverse_functional = reasoner.is_inverse_functional_property(property_iri)?;
let is_reflexive = reasoner.is_reflexive_property(property_iri)?;
let is_irreflexive = reasoner.is_irreflexive_property(property_iri)?;

println!("Property characteristics for hasParent:");
println!("  Transitive: {}", is_transitive);
println!("  Symmetric: {}", is_symmetric);
println!("  Functional: {}", is_functional);
println!("  Inverse Functional: {}", is_inverse_functional);
println!("  Reflexive: {}", is_reflexive);
println!("  Irreflexive: {}", is_irreflexive);
```

## Individual Reasoning

### Class Membership

```rust
// Check if an individual belongs to a class
let individual_iri = "http://example.org/John";
let class_iri = "http://example.org/Person";

let is_instance = reasoner.is_instance_of(individual_iri, class_iri)?;
if is_instance {
    println!("John is a Person");
}

// Get all classes an individual belongs to
let individual_classes = reasoner.types_of(individual_iri)?;
println!("Classes that John belongs to:");
for class_iri in individual_classes {
    println!("  - {}", class_iri);
}
```

### Same/Different Individuals

```rust
// Check if two individuals are the same
let john_iri = "http://example.org/John";
let john_doe_iri = "http://example.org/JohnDoe";

let are_same = reasoner.are_same_individuals(john_iri, john_doe_iri)?;
if are_same {
    println!("John and JohnDoe are the same individual");
}

// Check if two individuals are different
let mary_iri = "http://example.org/Mary";
let are_different = reasoner.are_different_individuals(john_iri, mary_iri)?;
if are_different {
    println!("John and Mary are different individuals");
}

// Get all same individuals
let same_individuals = reasoner.same_individuals_as(john_iri)?;
println!("Individuals same as John:");
for individual_iri in same_individuals {
    if individual_iri != john_iri {
        println!("  - {}", individual_iri);
    }
}
```

## Performance Optimization

### Caching Configuration

```rust
use owl2_reasoner::reasoning::ReasoningConfig;
use std::time::Duration;

// Configure caching for performance
let config = ReasoningConfig {
    cache_consistency_results: true,
    cache_classification_results: true,
    cache_satisfiability_results: true,
    cache_size: 10000,
    cache_ttl: Some(Duration::from_secs(300)), // 5 minutes
    ..Default::default()
};

let reasoner = SimpleReasoner::with_config(ontology, config);
```

### Cache Statistics

```rust
// Get cache performance statistics
let stats = reasoner.cache_stats()?;

println!("Cache Statistics:");
println!("  Hits: {}", stats.hits);
println!("  Misses: {}", stats.misses);
println!("  Hit Rate: {:.2}%", stats.hit_rate() * 100.0);
println!("  Size: {} entries", stats.size);
println!("  Memory Usage: {} bytes", stats.memory_usage);
```

### Parallel Reasoning

```rust
// Enable parallel reasoning for large ontologies
let config = ReasoningConfig {
    enable_parallel_reasoning: true,
    max_workers: Some(8), // Use 8 worker threads
    parallel_threshold: 1000, // Use parallel for ontologies with >1000 classes
    ..Default::default()
};

let parallel_reasoner = SimpleReasoner::with_config(ontology, config);
```

## Incremental Reasoning

### Adding Axioms Incrementally

```rust
use owl2_reasoner::reasoning::IncrementalReasoner;

let mut incremental_reasoner = IncrementalReasoner::new(ontology);

// Initial classification
incremental_reasoner.classify()?;

// Add new axiom incrementally
let new_axiom = SubClassOfAxiom::new(
    ClassExpression::from(Class::new("http://example.org/Student")),
    ClassExpression::from(Class::new("http://example.org/Person")),
);

// Only recompute affected parts
let update_time = incremental_reasoner.add_axiom_incremental(new_axiom)?;
println!("Incremental update took: {:?}", update_time);
```

### Change Tracking

```rust
// Track what changed during incremental updates
let changes = incremental_reasoner.get_pending_changes();

if !changes.is_empty() {
    println!("Changes to process:");
    for change in changes {
        match change {
            Change::AddedAxiom(axiom) => println!("  + Added: {}", axiom),
            Change::RemovedAxiom(axiom) => println!("  - Removed: {}", axiom),
            Change::ModifiedEntity(entity) => println!("  ~ Modified: {}", entity),
        }
    }

    incremental_reasoner.apply_changes()?;
}
```

## Error Handling

### Reasoning Errors

```rust
use owl2_reasoner::{OwlError, OwlResult};

fn perform_reasoning(reasoner: &SimpleReasoner) -> OwlResult<()> {
    match reasoner.is_consistent() {
        Ok(is_consistent) => {
            if is_consistent {
                println!("Ontology is consistent");
                reasoner.classify()?;
            } else {
                println!("Ontology is inconsistent");
                let explanations = reasoner.explain_inconsistency()?;
                println!("Found {} explanations", explanations.len());
            }
            Ok(())
        }
        Err(OwlError::ReasoningError(msg)) => {
            eprintln!("Reasoning failed: {}", msg);
            Err(OwlError::ReasoningError(msg))
        }
        Err(OwlError::TimeoutError) => {
            eprintln!("Reasoning timed out");
            Err(OwlError::TimeoutError)
        }
        Err(OwlError::MemoryError) => {
            eprintln!("Insufficient memory for reasoning");
            Err(OwlError::MemoryError)
        }
        Err(err) => {
            eprintln!("Other error: {}", err);
            Err(err)
        }
    }
}
```

## Best Practices

1. **Choose the right reasoner**:
   - Use `SimpleReasoner` for most applications
   - Use `TableauxReasoner` for complex reasoning needs
   - Use profile-optimized reasoners for known profiles

2. **Configure caching appropriately**:
   - Enable caching for repeated operations
   - Set appropriate cache sizes based on memory constraints
   - Use TTL for cache eviction

3. **Handle timeouts gracefully**:
   - Set reasonable timeouts for large ontologies
   - Provide fallback options for timeout scenarios

4. **Use incremental reasoning**:
   - For frequently changing ontologies
   - To avoid recomputing entire classification

5. **Monitor performance**:
   - Use cache statistics to optimize configuration
   - Profile reasoning times for bottlenecks

## Summary

The reasoning engine provides comprehensive OWL2 reasoning capabilities:

- **Multiple reasoner types** for different use cases
- **Complete reasoning operations** (consistency, classification, satisfiability)
- **Performance optimization** with caching and parallel processing
- **Incremental reasoning** for dynamic ontologies
- **Robust error handling** and timeout management
- **Profile optimization** for better performance

These reasoning capabilities form the core intelligence of the OWL2 Reasoner and enable powerful inference over semantic knowledge bases.