# Reasoning

This chapter covers the reasoning capabilities of the OWL2 Reasoner, including consistency checking, classification, satisfiability testing, and inference.

## Overview of Reasoning Types

The OWL2 Reasoner provides several types of reasoning:

- **Consistency Checking**: Verify if an ontology is logically consistent
- **Classification**: Compute the class hierarchy and subclass relationships
- **Satisfiability**: Check if classes can have instances
- **Realization**: Find the most specific classes for individuals
- **Query Answering**: Retrieve information based on logical patterns

## Creating a Reasoner

### Simple Reasoner

```rust
use owl2_reasoner::{Ontology, SimpleReasoner};

// Create a reasoner with default configuration
let mut reasoner = SimpleReasoner::new(ontology);

// Check if the ontology is consistent
let is_consistent = reasoner.is_consistent()?;
println!("Ontology is consistent: {}", is_consistent);
```

### Advanced Tableaux Reasoner

```rust
use owl2_reasoner::reasoning::tableaux::{TableauxReasoner, ReasoningConfig};

// Configure advanced reasoning
let config = ReasoningConfig {
    max_depth: 1000,
    timeout: Some(std::time::Duration::from_secs(30)),
    debug: false,
    parallel: true,
    cache_size: 10000,
};

// Create advanced reasoner
let mut tableaux_reasoner = TableauxReasoner::with_config(&ontology, config);
```

### Profile-Optimized Reasoner

```rust
use owl2_reasoner::reasoning::profile_optimized::ProfileOptimizedReasoner;
use owl2_reasoner::profiles::Owl2Profile;

// Create reasoner optimized for specific OWL2 profile
let mut el_reasoner = ProfileOptimizedReasoner::new(&ontology, Owl2Profile::EL);
let mut ql_reasoner = ProfileOptimizedReasoner::new(&ontology, Owl2Profile::QL);
let mut rl_reasoner = ProfileOptimizedReasoner::new(&ontology, Owl2Profile::RL);
```

## Consistency Checking

### Basic Consistency

```rust
// Check overall ontology consistency
let is_consistent = reasoner.is_consistent()?;

if is_consistent {
    println!("The ontology is logically consistent");
} else {
    println!("The ontology contains contradictions");
}
```

### Consistency Explanations

```rust
// Get explanations for inconsistency
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
// Compute the complete class hierarchy
reasoner.classify()?;

// Get direct subclasses of a class
let person_iri = "http://example.org/Person";
let direct_subclasses = reasoner.direct_subclasses_of(person_iri)?;

println!("Direct subclasses of Person:");
for subclass_iri in direct_subclasses {
    println!("  - {}", subclass_iri);
}

// Get all subclasses (transitive closure)
let all_subclasses = reasoner.all_subclasses_of(person_iri)?;
println!("All subclasses of Person: {}", all_subclasses.len());
```

### Superclass Relationships

```rust
// Check if one class is a subclass of another
let parent_iri = "http://example.org/Parent";
let person_iri = "http://example.org/Person";

let is_subclass = reasoner.is_subclass_of(parent_iri, person_iri)?;
if is_subclass {
    println!("Parent ⊑ Person");
}

// Get direct superclasses
let direct_superclasses = reasoner.direct_superclasses_of(person_iri)?;
println!("Direct superclasses of Person:");
for superclass_iri in direct_superclasses {
    println!("  - {}", superclass_iri);
}
```

### Equivalent Classes

```rust
// Find equivalent classes
let class_iri = "http://example.org/Human";
let equivalent_classes = reasoner.equivalent_classes_of(class_iri)?;

println!("Classes equivalent to Human:");
for equiv_iri in equivalent_classes {
    if equiv_iri != class_iri {
        println!("  - {}", equiv_iri);
    }
}
```

### Disjoint Classes

```rust
// Check if two classes are disjoint
let animal_iri = "http://example.org/Animal";
let plant_iri = "http://example.org/Plant";

let are_disjoint = reasoner.are_disjoint(animal_iri, plant_iri)?;
if are_disjoint {
    println!("Animal and Plant are disjoint");
}

// Get all disjoint classes
let disjoint_classes = reasoner.disjoint_classes_of(animal_iri)?;
println!("Classes disjoint with Animal:");
for disjoint_iri in disjoint_classes {
    println!("  - {}", disjoint_iri);
}
```

## Satisfiability Testing

### Class Satisfiability

```rust
// Check if a class is satisfiable (can have instances)
let class_iri = "http://example.org/Unicorn";
let is_satisfiable = reasoner.is_class_satisfiable(class_iri)?;

if is_satisfiable {
    println!("Unicorn is satisfiable (can have instances)");
} else {
    println!("Unicorn is unsatisfiable (contradictory definition)");
}
```

### Unsatisfiability Explanations

```rust
// Get explanations for class unsatisfiability
if !reasoner.is_class_satisfiable(class_iri)? {
    let explanations = reasoner.explain_unsatisfiability(class_iri)?;

    println!("Unsatisfiability explanations for {}:", class_iri);
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
// Find most specific concepts for a given individual
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
let has_parent_iri = "http://example.org/hasParent";
let has_ancestor_iri = "http://example.org/hasAncestor";

let is_subproperty = reasoner.is_subproperty_of(has_parent_iri, has_ancestor_iri)?;
if is_subproperty {
    println!("hasParent ⊑ hasAncestor");
}

// Get sub-properties
let subproperties = reasoner.subproperties_of(has_ancestor_iri)?;
println!("Sub-properties of hasAncestor:");
for prop_iri in subproperties {
    println!("  - {}", prop_iri);
}
```

### Property Characteristics

```rust
// Check if a property has specific characteristics
let property_iri = "http://example.org/hasParent";

let is_transitive = reasoner.is_transitive_property(property_iri)?;
let is_symmetric = reasoner.is_symmetric_property(property_iri)?;
let is_functional = reasoner.is_functional_property(property_iri)?;
let is_inverse_functional = reasoner.is_inverse_functional_property(property_iri)?;

println!("Property characteristics for hasParent:");
println!("  Transitive: {}", is_transitive);
println!("  Symmetric: {}", is_symmetric);
println!("  Functional: {}", is_functional);
println!("  Inverse Functional: {}", is_inverse_functional);
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
```

## Reasoning with Caching

### Configuration

```rust
use owl2_reasoner::reasoning::ReasoningConfig;
use std::time::Duration;

let config = ReasoningConfig {
    cache_size: 10000,
    cache_ttl: Some(Duration::from_secs(300)), // 5 minutes
    enable_parallel_reasoning: true,
    max_workers: None, // Use all available cores
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
```

## Performance Optimization

### Profile Detection

```rust
use owl2_reasoner::profiles::ProfileValidator;

// Detect OWL2 profile of ontology
let validator = ProfileValidator::new();
let profile_result = validator.detect_profile(&ontology)?;

match profile_result.detected_profile {
    Some(profile) => {
        println!("Detected OWL2 profile: {:?}", profile);
        println!("Confidence: {:.2}%", profile_result.confidence * 100.0);
    }
    None => {
        println!("No specific OWL2 profile detected");
    }
}
```

### Incremental Reasoning

```rust
// Add new axiom and update reasoning incrementally
let new_axiom = SubClassOfAxiom::new(
    ClassExpression::from(Class::new("http://example.org/Student")),
    ClassExpression::from(Class::new("http://example.org/Person")),
);

reasoner.add_axiom_incremental(new_axiom)?;

// Reasoning results are automatically updated
let is_subclass = reasoner.is_subclass_of(
    "http://example.org/Student",
    "http://example.org/Person"
)?;
println!("Student ⊑ Person: {}", is_subclass);
```

## Error Handling

### Reasoning Errors

```rust
use owl2_reasoner::OwlError;

match reasoner.is_consistent() {
    Ok(is_consistent) => {
        println!("Consistency check result: {}", is_consistent);
    }
    Err(OwlError::ReasoningError(msg)) => {
        eprintln!("Reasoning failed: {}", msg);
    }
    Err(OwlError::TimeoutError) => {
        eprintln!("Reasoning timed out");
    }
    Err(OwlError::MemoryError) => {
        eprintln!("Insufficient memory for reasoning");
    }
    Err(err) => {
        eprintln!("Other error: {}", err);
    }
}
```

## Best Practices

1. **Choose the right reasoner**: Use SimpleReasoner for basic tasks, TableauxReasoner for complex reasoning
2. **Configure caching**: Enable caching with appropriate TTL for repeated queries
3. **Use profile optimization**: Detect and use appropriate OWL2 profiles when possible
4. **Handle timeouts**: Set reasonable timeouts for large ontologies
5. **Monitor performance**: Use cache statistics to optimize configuration
6. **Validate input**: Check ontology validity before reasoning
7. **Use incremental updates**: For dynamic ontologies, use incremental reasoning

## Summary

This chapter covered the comprehensive reasoning capabilities of the OWL2 Reasoner:

- Different types of reasoning and reasoner configurations
- Consistency checking and explanation generation
- Classification and hierarchy computation
- Satisfiability testing and analysis
- Property and individual reasoning
- Performance optimization and caching
- Error handling and best practices

**Next**: [Querying](querying.md) - Learn how to query ontologies using SPARQL-like patterns.