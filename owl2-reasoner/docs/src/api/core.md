# Core Types

This chapter covers the fundamental types and structures that form the foundation of the OWL2 Reasoner.

## Overview

The core types module provides:

- **IRI management** with efficient caching and sharing
- **OWL2 entities** (classes, properties, individuals)
- **Error handling** with comprehensive error types
- **Result types** for idiomatic Rust error handling

## IRI (Internationalized Resource Identifier)

### Basic IRI Usage

```rust
use owl2_reasoner::IRI;

// Create a new IRI
let iri = IRI::new("http://example.org/Person")?;

// Get string representation
let iri_string = iri.as_str();
assert_eq!(iri_string, "http://example.org/Person");

// Get components
assert_eq!(iri.namespace(), "http://example.org/");
assert_eq!(iri.local_name(), "Person");
assert_eq!(iri.prefix(), Some("example"));
```

### Shared IRIs

```rust
use owl2_reasoner::{IRI, cache_manager};

// Use shared IRI for memory efficiency
let shared_iri = cache_manager::get_or_create_iri("http://example.org/Person")?;

// Multiple references to the same IRI share memory
let another_ref = cache_manager::get_or_create_iri("http://example.org/Person")?;
assert!(Arc::ptr_eq(&shared_iri, &another_ref));
```

### IRI Operations

```rust
// Check if IRI is in OWL namespace
let owl_thing = IRI::new("http://www.w3.org/2002/07/owl#Thing")?;
assert!(owl_thing.is_owl());

// Check if IRI is in RDF namespace
let rdf_type = IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")?;
assert!(rdf_type.is_rdf());

// Check if IRI is in XSD namespace
let xsd_string = IRI::new("http://www.w3.org/2001/XMLSchema#string")?;
assert!(xsd_string.is_xsd());

// Resolve relative IRI
let base = IRI::new("http://example.org/ontology/")?;
let relative = IRI::new("#Person")?;
let absolute = base.resolve(&relative)?;
assert_eq!(absolute.as_str(), "http://example.org/ontology/#Person");
```

## OWL2 Entities

### Classes

```rust
use owl2_reasoner::Class;

// Create a new class
let person_class = Class::new("http://example.org/Person");

// Create a shared class (memory efficient)
let shared_person = Class::new_shared("http://example.org/Person")?;

// Get class IRI
let iri = person_class.iri();
println!("Person class IRI: {}", iri);

// Check if class is built-in
let owl_thing = Class::new("http://www.w3.org/2002/07/owl#Thing");
assert!(owl_thing.is_builtin());
assert!(owl_thing.is_thing());

// Add annotations
let mut person_class = Class::new("http://example.org/Person");
let annotation = Annotation::new(
    IRI::new("http://www.w3.org/2000/01/rdf-schema#label")?,
    Literal::string("Person class")
);
person_class.add_annotation(annotation);
```

### Object Properties

```rust
use owl2_reasoner::ObjectProperty;

// Create an object property
let has_parent = ObjectProperty::new("http://example.org/hasParent");

// Create a shared object property
let shared_has_parent = ObjectProperty::new_shared("http://example.org/hasParent")?;

// Get property IRI
let iri = has_parent.iri();
println!("hasParent property IRI: {}", iri);

// Add domain and range annotations
let mut has_parent = ObjectProperty::new("http://example.org/hasParent");
let domain_annotation = Annotation::new(
    IRI::new("http://www.w3.org/2000/01/rdf-schema#domain")?,
    Literal::string("http://example.org/Person")
);
has_parent.add_annotation(domain_annotation);
```

### Data Properties

```rust
use owl2_reasoner::DataProperty;

// Create a data property
let has_age = DataProperty::new("http://example.org/hasAge");

// Create a shared data property
let shared_has_age = DataProperty::new_shared("http://example.org/hasAge")?;

// Get property IRI and add annotations
let mut has_age = DataProperty::new("http://example.org/hasAge");
let range_annotation = Annotation::new(
    IRI::new("http://www.w3.org/2000/01/rdf-schema#range")?,
    Literal::string("http://www.w3.org/2001/XMLSchema#integer")
);
has_age.add_annotation(range_annotation);
```

### Named Individuals

```rust
use owl2_reasoner::NamedIndividual;

// Create a named individual
let john = NamedIndividual::new("http://example.org/John");

// Create a shared named individual
let shared_john = NamedIndividual::new_shared("http://example.org/John")?;

// Get individual IRI
let iri = john.iri();
println!("John individual IRI: {}", iri);

// Add annotations
let mut john = NamedIndividual::new("http://example.org/John");
let label_annotation = Annotation::new(
    IRI::new("http://www.w3.org/2000/01/rdf-schema#label")?,
    Literal::string("John Doe")
);
john.add_annotation(label_annotation);
```

## Annotations

### Creating Annotations

```rust
use owl2_reasoner::{Annotation, Literal, IRI};

// Create a simple annotation
let label = Annotation::new(
    IRI::new("http://www.w3.org/2000/01/rdf-schema#label")?,
    Literal::string("Person")
);

// Create annotation with language tag
let spanish_label = Annotation::new(
    IRI::new("http://www.w3.org/2000/01/rdf-schema#label")?,
    Literal::lang_string("Persona", "es")
);

// Create annotation with typed literal
let description = Annotation::new(
    IRI::new("http://www.w3.org/2000/01/rdf-schema#comment")?,
    Literal::typed("A human being", "http://www.w3.org/2001/XMLSchema#string")
);
```

## Literals

### Creating Literals

```rust
use owl2_reasoner::Literal;

// String literals
let string_lit = Literal::string("Hello World");
let lang_string = Literal::lang_string("Hello", "en");

// Numeric literals
let int_lit = Literal::integer(42);
let float_lit = Literal::float(3.14159);
let decimal_lit = Literal::decimal("123.45");

// Boolean literals
let bool_lit = Literal::boolean(true);

// Date and time literals
let date_lit = Literal::date("2023-01-01")?;
let datetime_lit = Literal::datetime("2023-01-01T12:00:00")?;

// Custom typed literals
let custom_lit = Literal::typed("custom value", "http://example.org/customType")?;
```

### Literal Operations

```rust
// Get literal value and datatype
let int_lit = Literal::integer(42);
assert_eq!(int_lex.value(), "42");
assert_eq!(int_lit.datatype(), "http://www.w3.org/2001/XMLSchema#integer");

// Get language tag (for language-tagged strings)
let lang_lit = Literal::lang_string("Hello", "en");
assert_eq!(lang_lit.language(), Some("en"));

// Check literal types
assert!(int_lit.is_integer());
assert!(Literal::boolean(true).is_boolean());
assert!(Literal::string("test").is_string());
```

## Error Handling

### OwlError Types

```rust
use owl2_reasoner::OwlError;

// Handle different error types
match some_operation() {
    Ok(result) => println!("Success: {:?}", result),
    Err(OwlError::IRIError(msg)) => {
        eprintln!("IRI error: {}", msg);
    }
    Err(OwlError::ParseError(msg)) => {
        eprintln!("Parse error: {}", msg);
    }
    Err(OwlError::ReasoningError(msg)) => {
        eprintln!("Reasoning error: {}", msg);
    }
    Err(OwlError::ValidationError(msg)) => {
        eprintln!("Validation error: {}", msg);
    }
    Err(OwlError::CacheError(msg)) => {
        eprintln!("Cache error: {}", msg);
    }
    Err(OwlError::IOError(io_err)) => {
        eprintln!("IO error: {}", io_err);
    }
}
```

### OwlResult Type

```rust
use owl2_reasoner::OwlResult;

// Function returning OwlResult
fn create_class(iri_str: &str) -> OwlResult<Class> {
    let iri = IRI::new(iri_str)?;
    Ok(Class::new(iri))
}

// Using the function
match create_class("http://example.org/Person") {
    Ok(class) => println!("Created class: {}", class.iri()),
    Err(e) => eprintln!("Failed to create class: {}", e),
}
```

## Entity Trait

The `Entity` trait provides common functionality for all OWL2 entities:

```rust
use owl2_reasoner::{Entity, Class, ObjectProperty};

fn print_entity_info<E: Entity>(entity: &E) {
    println!("Entity IRI: {}", entity.iri());
    println!("Annotations: {}", entity.annotations().len());

    // Check if entity has specific annotation
    for annotation in entity.annotations() {
        if annotation.property().as_str().contains("label") {
            println!("Label: {}", annotation.value());
        }
    }
}

// Use with different entity types
let person_class = Class::new("http://example.org/Person");
let has_parent = ObjectProperty::new("http://example.org/hasParent");

print_entity_info(&person_class);
print_entity_info(&has_parent);
```

## Constants

### Well-Known IRIs

```rust
use owl2_reasoner::constants::*;

// OWL2 vocabulary
assert_eq!(OWL_THING, "http://www.w3.org/2002/07/owl#Thing");
assert_eq!(OWL_NOTHING, "http://www.w3.org/2002/07/owl#Nothing");
assert_eq!(OWL_CLASS, "http://www.w3.org/2002/07/owl#Class");

// RDF vocabulary
assert_eq!(RDF_TYPE, "http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
assert_eq!(RDFS_LABEL, "http://www.w3.org/2000/01/rdf-schema#label");
assert_eq!(RDFS_COMMENT, "http://www.w3.org/2000/01/rdf-schema#comment");

// XSD datatypes
assert_eq!(XSD_STRING, "http://www.w3.org/2001/XMLSchema#string");
assert_eq!(XSD_INTEGER, "http://www.w3.org/2001/XMLSchema#integer");
assert_eq!(XSD_BOOLEAN, "http://www.w3.org/2001/XMLSchema#boolean");
```

## Memory Management

### Shared References

```rust
use std::sync::Arc;
use owl2_reasoner::Class;

// Classes use Arc<IRI> internally for memory efficiency
let class1 = Class::new("http://example.org/Person");
let class2 = Class::new("http://example.org/Person");

// Even with separate instances, the underlying IRI might be shared
println!("Class 1 IRI ptr: {:?}", class1.iri().as_ptr());
println!("Class 2 IRI ptr: {:?}", class2.iri().as_ptr());
```

### Cache Management

```rust
use owl2_reasoner::cache_manager;

// Get global cache statistics
let stats = cache_manager::global_cache_stats();
println!("IRI cache size: {}", stats.iri_cache_size);
println!("Hit rate: {:.2}%", stats.iri_cache_hit_rate * 100.0);

// Clear cache if needed
cache_manager::clear_global_iri_cache()?;

// Force cache eviction
let evicted = cache_manager::force_global_entity_cache_eviction(100)?;
println!("Evicted {} cache entries", evicted);
```

## Type Safety

The core types leverage Rust's type system for safety:

```rust
// Type-safe entity creation
fn add_entity_to_ontology<E: Entity>(ontology: &mut Ontology, entity: E) -> OwlResult<()> {
    match entity.iri().as_str() {
        iri if iri.contains("Class") => {
            // This would be handled by specific add_* methods in real code
            println!("Adding class: {}", iri);
        }
        iri if iri.contains("Property") => {
            println!("Adding property: {}", iri);
        }
        iri if iri.contains("NamedIndividual") => {
            println!("Adding individual: {}", iri);
        }
        _ => {
            println!("Adding entity: {}", iri);
        }
    }
    Ok(())
}
```

## Best Practices

1. **Use shared constructors**: Prefer `Class::new_shared()` over `Class::new()` for better memory efficiency
2. **Cache IRIs**: Use the global cache manager for frequently used IRIs
3. **Handle errors properly**: Use the `?` operator with `OwlResult`
4. **Validate inputs**: Check IRI validity before creating entities
5. **Use constants**: Prefer well-known constants over hardcoded strings
6. **Leverage type system**: Use trait bounds for generic entity operations

## Summary

The core types provide the foundation for working with OWL2 ontologies:

- **IRI management** with efficient caching and sharing
- **OWL2 entities** with type-safe constructors and methods
- **Annotations and literals** for metadata and data values
- **Comprehensive error handling** with specific error types
- **Memory-efficient design** using Arc-based sharing

These core types are used throughout the OWL2 Reasoner and form the basis for more complex operations like reasoning and querying.