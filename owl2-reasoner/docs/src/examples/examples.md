# Examples

This section provides practical examples of using the OWL2 Reasoner library with detailed explanations and performance considerations.

## Simple Example

The [simple example](simple_example.rs) demonstrates the basic functionality of the OWL2 Reasoner library:

### What it demonstrates:
- Creating an ontology with classes and properties
- Adding subclass relationships
- Creating individuals and class assertions
- Adding property assertions
- Basic reasoning operations

### Key concepts:
- **Ontology Management**: Creating and populating ontologies
- **Entity Creation**: Classes, properties, and individuals
- **Axiom Addition**: Subclass relationships and assertions
- **Reasoning**: Consistency checking and instance retrieval

### Running the example:
```bash
cargo run --example simple_example
```

### Expected output:
```
=== Simple OWL2 Reasoner Example ===

✓ Added 2 classes
✓ Added 1 object properties
✓ Added 1 subclass axioms
✓ Added 2 named individuals
✓ Added 2 class assertions
✓ Added 1 property assertions

=== Reasoning Results ===
✓ Ontology is consistent: true
✓ Parent ⊑ Person: true
✓ Persons: ["http://example.org/John", "http://example.org/Mary"]
✓ Parents: ["http://example.org/Mary"]

=== Performance Statistics ===
✓ Total entities: 5
✓ Total axioms: 4
✓ Cache stats: {"consistency": 1, "subclass": 1, "satisfiability": 0, "instances": 2}

=== Example Complete ===
✓ Successfully demonstrated basic OWL2 reasoning capabilities
✓ All operations completed without errors
```

## Family Ontology Example

The [family ontology example](family_ontology.rs) demonstrates more complex relationships and property characteristics:

### What it demonstrates:
- Complex class hierarchies
- Property characteristics (transitive, symmetric, asymmetric)
- Property hierarchies (subproperties)
- Multiple individuals and relationships
- Comprehensive reasoning operations

### Key concepts:
- **Property Characteristics**: Transitive, symmetric, asymmetric properties
- **Property Hierarchies**: Subproperty relationships
- **Complex Reasoning**: Multiple inference types
- **Query Operations**: Pattern-based queries

### Running the example:
```bash
cargo run --example family_ontology
```

## Biomedical Ontology Example

The [biomedical ontology example](biomedical_ontology.rs) demonstrates domain-specific ontology development:

### What it demonstrates:
- Biomedical domain modeling
- Complex class expressions
- Gene-disease associations
- Protein-protein interactions
- Equivalent class definitions

### Key concepts:
- **Domain Modeling**: Biomedical knowledge representation
- **Complex Class Expressions**: Intersection, union, restrictions
- **Equivalent Classes**: Definitional relationships
- **Biomedical Reasoning**: Domain-specific inference

### Running the example:
```bash
cargo run --example biomedical_ontology
```

## Performance Benchmarking Example

The [performance benchmarking example](performance_benchmarking.rs) demonstrates performance testing and analysis:

### What it demonstrates:
- Large ontology creation (10,000+ entities)
- Reasoning performance metrics
- Cache performance analysis
- Memory usage analysis
- Scaling performance evaluation

### Key concepts:
- **Performance Testing**: Timing and metrics
- **Cache Analysis**: Hit rates and speedup factors
- **Memory Analysis**: Component-wise memory usage
- **Scaling Studies**: Performance vs. ontology size

### Running the example:
```bash
cargo run --example performance_benchmarking
```

### Performance characteristics:
- **Creation Rate**: Thousands of entities per second
- **Reasoning Speed**: Sub-millisecond for small ontologies
- **Cache Speedup**: 2-10x improvement for cached operations
- **Memory Efficiency**: ~100 bytes per entity average

## Example Code Patterns

### Basic Ontology Creation
```rust
let mut ontology = Ontology::new();
ontology.set_iri("http://example.org/my-ontology");

// Add classes
let class1 = Class::new("http://example.org/Class1");
ontology.add_class(class1)?;

// Add properties
let prop1 = ObjectProperty::new("http://example.org/hasRelation");
ontology.add_object_property(prop1)?;
```

### Adding Axioms
```rust
// Subclass relationship
let subclass_axiom = SubClassOfAxiom::new(
    ClassExpression::from(sub_class),
    ClassExpression::from(super_class),
);
ontology.add_subclass_axiom(subclass_axiom)?;

// Class assertion
let assertion = ClassAssertionAxiom::new(
    individual.iri().clone(),
    ClassExpression::from(class),
);
ontology.add_class_assertion(assertion)?;
```

### Reasoning Operations
```rust
let reasoner = SimpleReasoner::new(ontology);

// Consistency checking
let is_consistent = reasoner.is_consistent()?;

// Subclass inference
let is_subclass = reasoner.is_subclass_of(&sub_iri, &sup_iri)?;

// Instance retrieval
let instances = reasoner.get_instances(&class_iri)?;
```

## Performance Considerations

### 1. Batch Operations
```rust
// Batch creation is more efficient
let mut entities = Vec::new();
for i in 0..1000 {
    entities.push(Class::new(&format!("http://example.org/Class{}", i)));
}

for entity in entities {
    ontology.add_class(entity)?;
}
```

### 2. Cache Management
```rust
let reasoner = SimpleReasoner::new(ontology);

// Clear caches if needed
reasoner.clear_caches();

// Monitor cache performance
let stats = reasoner.cache_stats();
```

### 3. Memory Efficiency
```rust
// Use IRI caching effectively
let iri = IRI::new("http://example.org/Entity")?;
// IRI is automatically cached and shared

// Use Arc-based sharing through the API
let class = Class::new("http://example.org/Class");
// Class uses Arc internally for efficient storage
```

## Common Patterns

### 1. Namespace Management
```rust
// Define namespace prefixes
let mut registry = IRIRegistry::new();
registry.register("ex", "http://example.org/")?;

// Create IRIs with prefixes
let class = registry.iri_with_prefix("ex", "Class")?;
```

### 2. Error Handling
```rust
fn safe_ontology_operations() -> OwlResult<()> {
    let mut ontology = Ontology::new();
    
    // Handle potential errors
    match ontology.add_class(valid_class) {
        Ok(_) => println!("Class added successfully"),
        Err(OwlError::InvalidIRI(msg)) => println!("Invalid IRI: {}", msg),
        Err(e) => println!("Other error: {}", e),
    }
    
    Ok(())
}
```

### 3. Validation Patterns
```rust
// Validate ontology before reasoning
if reasoner.is_consistent()? {
    println!("Ontology is consistent, proceeding with reasoning");
    // Perform reasoning operations
} else {
    println!("Ontology is inconsistent, check for contradictions");
}
```

## Best Practices

### 1. IRI Management
- Use consistent naming conventions
- Leverage the global IRI cache
- Use namespace prefixes for readability

### 2. Performance Optimization
- Batch operations when possible
- Monitor cache performance
- Use appropriate indexing strategies

### 3. Error Handling
- Handle all potential errors gracefully
- Provide meaningful error messages
- Use the `OwlResult` type consistently

### 4. Testing
- Test with small ontologies first
- Verify consistency before complex operations
- Monitor performance characteristics

## Troubleshooting

### Common Issues

1. **Compilation Errors**: Check API signatures and parameter types
2. **Runtime Errors**: Handle IRI validation and ontology consistency
3. **Performance Issues**: Monitor cache usage and memory allocation
4. **Memory Usage**: Use appropriate data structures and sharing

### Getting Help

- Check the [API Documentation](../api/)
- Review the [User Guide](../user-guide/)
- Examine the test cases in the source code
- Open an issue on GitHub for specific problems