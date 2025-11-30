# Getting Started

This guide will help you get up and running with the OWL2 Reasoner quickly.

## Installation

Add the OWL2 Reasoner to your `Cargo.toml`:

```toml
[dependencies]
owl2-reasoner = "0.1.0"
```

Or install it directly with cargo:

```bash
cargo add owl2-reasoner
```

## Basic Usage

### Creating an Ontology

```rust
use owl2_reasoner::{Ontology, Class, ObjectProperty, NamedIndividual};

// Create a new empty ontology
let mut ontology = Ontology::new();

// Set ontology IRI
ontology.set_iri("http://example.org/family");

// Add classes
let person_class = Class::new("http://example.org/Person");
let parent_class = Class::new("http://example.org/Parent");
ontology.add_class(person_class.clone())?;
ontology.add_class(parent_class.clone())?;

// Add properties
let has_child = ObjectProperty::new("http://example.org/hasChild");
ontology.add_object_property(has_child)?;

// Add individuals
let john = NamedIndividual::new("http://example.org/John");
let mary = NamedIndividual::new("http://example.org/Mary");
ontology.add_named_individual(john)?;
ontology.add_named_individual(mary)?;
```

### Adding Axioms

```rust
use owl2_reasoner::{
    SubClassOfAxiom, ClassAssertionAxiom, PropertyAssertionAxiom,
    ClassExpression
};

// Add subclass relationship
let subclass_axiom = SubClassOfAxiom::new(
    ClassExpression::from(parent_class.clone()),
    ClassExpression::from(person_class.clone()),
);
ontology.add_subclass_axiom(subclass_axiom)?;

// Add class assertions
let john_person = ClassAssertionAxiom::new(
    ClassExpression::from(person_class.clone()),
    john.clone(),
);
ontology.add_class_assertion(john_person)?;

// Add property assertions
let john_has_mary = PropertyAssertionAxiom::new(
    has_child.clone(),
    john.clone(),
    mary.clone(),
);
ontology.add_property_assertion(john_has_mary)?;
```

### Basic Reasoning

```rust
use owl2_reasoner::SimpleReasoner;

// Create a reasoner
let reasoner = SimpleReasoner::new(ontology);

// Check consistency
let is_consistent = reasoner.is_consistent()?;
println!("Ontology is consistent: {}", is_consistent);

// Check subclass relationships
let is_parent_subclass_of_person = reasoner.is_subclass_of(&parent_class, &person_class)?;
println!("Parent ⊑ Person: {}", is_parent_subclass_of_person);

// Get instances of a class
let person_instances = reasoner.get_instances(&person_class)?;
println!("Persons: {:?}", person_instances);
```

### Querying the Ontology

```rust
use owl2_reasoner::query::{QueryEngine, QueryPattern, QueryValue};

// Create a query engine
let mut query_engine = QueryEngine::new(&reasoner.ontology);

// Simple pattern query
let pattern = QueryPattern::Basic {
    subject: Some(QueryValue::IRI(john.clone())),
    predicate: Some(QueryValue::IRI(has_child.clone())),
    object: None, // Find all children
};

let results = query_engine.query_pattern(&pattern)?;
for result in results {
    println!("John has child: {:?}", result);
}

// Complex query with filters
let complex_pattern = QueryPattern::And(vec![
    QueryPattern::Basic {
        subject: None,
        predicate: Some(QueryValue::IRI(IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")?)),
        object: Some(QueryValue::IRI(person_class.clone())),
    },
    QueryPattern::Basic {
        subject: None,
        predicate: Some(QueryValue::IRI(has_child.clone())),
        object: None,
    },
]);

let parents_with_children = query_engine.query_pattern(&complex_pattern)?;
println!("Parents with children: {:?}", parents_with_children);
```

## Working with Files

### Loading from Turtle Format

```rust
use owl2_reasoner::parser::{Parser, TurtleParser};
use std::fs::File;

let file = File::open("ontology.ttl")?;
let mut parser = TurtleParser::new();
let ontology = parser.parse(&mut file)?;
```

### Loading from RDF/XML Format

```rust
use owl2_reasoner::parser::{Parser, RdfXmlParser};
use std::fs::File;

let file = File::open("ontology.rdf")?;
let mut parser = RdfXmlParser::new();
let ontology = parser.parse(&mut file)?;
```

### Loading from OWL/XML Format

```rust
use owl2_reasoner::parser::{Parser, OwlXmlParser};
use std::fs::File;

let file = File::open("ontology.owl")?;
let mut parser = OwlXmlParser::new();
let ontology = parser.parse(&mut file)?;
```

### Loading from N-Triples Format

```rust
use owl2_reasoner::parser::{Parser, NTriplesParser};
use std::fs::File;

let file = File::open("ontology.nt")?;
let mut parser = NTriplesParser::new();
let ontology = parser.parse(&mut file)?;
```

### Auto-detecting Format

```rust
use owl2_reasoner::parser::ParserFactory;
use std::fs::File;

let file = File::open("ontology.ttl")?;
let parser = ParserFactory::auto_detect_from_file(&file)?;
let ontology = parser.parse(&mut file)?;
```

### Saving to Turtle Format

```rust
use owl2_reasoner::parser::TurtleWriter;
use std::fs::File;

let file = File::create("output.ttl")?;
let writer = TurtleWriter::new();
writer.write(&mut file, &ontology)?;
```

## Advanced Features

### Property Characteristics

```rust
use owl2_reasoner::{ObjectProperty, ObjectPropertyCharacteristic};

let mut has_parent = ObjectProperty::new("http://example.org/hasParent");

// Add characteristics
has_parent.add_characteristic(ObjectPropertyCharacteristic::Transitive);
has_parent.add_characteristic(ObjectPropertyCharacteristic::Asymmetric);
has_parent.add_characteristic(ObjectPropertyCharacteristic::Irreflexive);
```

### Complex Class Expressions

```rust
use owl2_reasoner::{ClassExpression, ObjectPropertyExpression};

// Intersection: Person ⊓ Adult
let person_adult = ClassExpression::ObjectIntersectionOf(vec![
    ClassExpression::from(person_class),
    ClassExpression::from(adult_class),
]);

// Union: Parent ⊓ Child
let parent_or_child = ClassExpression::ObjectUnionOf(vec![
    ClassExpression::from(parent_class),
    ClassExpression::from(child_class),
]);

// Existential restriction: ∃hasChild.Person
let has_child_person = ClassExpression::ObjectSomeValuesFrom(
    Box::new(ObjectPropertyExpression::ObjectProperty(has_child)),
    Box::new(ClassExpression::from(person_class)),
);
```

### Performance Optimization

```rust
use owl2_reasoner::SimpleReasoner;

let reasoner = SimpleReasoner::new(ontology);

// Cache is automatically managed, but you can clear it if needed
reasoner.clear_caches();

// Get cache statistics
let stats = reasoner.cache_stats();
println!("Cache stats: {:?}", stats);
```

## Error Handling

The OWL2 Reasoner provides comprehensive error handling:

```rust
use owl2_reasoner::OwlError;

match ontology.add_class(invalid_class) {
    Ok(_) => println!("Class added successfully"),
    Err(OwlError::InvalidIRI(msg)) => println!("Invalid IRI: {}", msg),
    Err(OwlError::ParseError(msg)) => println!("Parse error: {}", msg),
    Err(e) => println!("Other error: {}", e),
}
```

## Next Steps

- [🛡️ Memory-Safe Testing Guide](memory-safety/testing.md) - Learn about memory-safe testing patterns
- [User Guide](user-guide/basic-usage.md) - Detailed usage patterns
- [Examples](examples/) - Real-world examples
- [API Reference](api/) - Complete API documentation
- [Performance Guide](user-guide/performance.md) - Optimization tips

## Troubleshooting

### Common Issues

1. **Compilation Errors**: Ensure you're using a stable Rust version (1.70+)
2. **Memory Usage**: Large ontologies may require increasing memory limits
3. **Performance**: Use indexed storage and caching for better performance
4. **Import Errors**: Check that imported ontologies are accessible

### Getting Help

- Check the [Memory Safety Documentation](memory-safety/README.md) for testing guidelines
- Check the [API Documentation](api/)
- Review the [Examples](examples/)
- Open an issue on GitHub for bug reports or feature requests