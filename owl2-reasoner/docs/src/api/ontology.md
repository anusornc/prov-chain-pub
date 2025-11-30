# Ontology Management

This chapter covers the `Ontology` struct and related functionality for managing OWL2 ontologies.

## Overview

The `Ontology` struct provides:

- **Indexed storage** for O(1) access to axioms and entities
- **Memory-efficient management** with Arc-based sharing
- **Import support** for multi-ontology reasoning
- **Validation** for ontology structure and consistency
- **Statistics** and metadata management

## Creating an Ontology

### Basic Creation

```rust
use owl2_reasoner::Ontology;

// Create an empty ontology
let mut ontology = Ontology::new();

// Create with specific IRI
let mut ontology = Ontology::with_iri("http://example.org/my-ontology")?;

// Create with IRI and version
let mut ontology = Ontology::with_version(
    "http://example.org/my-ontology",
    "http://example.org/my-ontology/1.0"
)?;
```

### Configuration

```rust
use owl2_reasoner::OntologyConfig;

let config = OntologyConfig {
    enable_indexing: true,
    cache_size: 10000,
    enable_validation: true,
    auto_optimize: true,
};

let mut ontology = Ontology::with_config(config);
```

## Managing Ontology Metadata

### Basic Metadata

```rust
use owl2_reasoner::Ontology;

let mut ontology = Ontology::new();

// Set ontology IRI
ontology.set_iri("http://example.org/family-ontology")?;

// Set version IRI
ontology.set_version_iri("http://example.org/family-ontology/2.0")?;

// Get metadata
if let Some(iri) = ontology.iri() {
    println!("Ontology IRI: {}", iri);
}

if let Some(version_iri) = ontology.version_iri() {
    println!("Version IRI: {}", version_iri);
}
```

### Annotations

```rust
use owl2_reasoner::{Annotation, Literal, IRI};

let mut ontology = Ontology::new();

// Add ontology-level annotations
let title_annotation = Annotation::new(
    IRI::new("http://purl.org/dc/terms/title")?,
    Literal::string("Family Relationships Ontology")
);
ontology.add_annotation(title_annotation)?;

let description_annotation = Annotation::new(
    IRI::new("http://purl.org/dc/terms/description")?,
    Literal::string("An ontology describing family relationships and kinship")
);
ontology.add_annotation(description_annotation)?;

// Get all annotations
for annotation in ontology.annotations() {
    println!("{}: {}", annotation.property(), annotation.value());
}
```

## Entity Management

### Adding Classes

```rust
use owl2_reasoner::{Class, NamedIndividual};

// Create and add classes
let person_class = Class::new("http://example.org/Person");
let male_class = Class::new("http://example.org/Male");
let female_class = Class::new("http://example.org/Female");

ontology.add_class(person_class.clone())?;
ontology.add_class(male_class.clone())?;
ontology.add_class(female_class.clone())?;

// Check if class exists
assert!(ontology.contains_class(&person_class.iri()));

// Get class by IRI
if let Some(class) = ontology.get_class(&person_class.iri()) {
    println!("Found class: {}", class.iri());
}

// Get all classes
println!("Total classes: {}", ontology.classes().len());
for class in ontology.classes() {
    println!("  - {}", class.iri());
}
```

### Adding Properties

```rust
use owl2_reasoner::{ObjectProperty, DataProperty};

// Object properties
let has_parent = ObjectProperty::new("http://example.org/hasParent");
let has_child = ObjectProperty::new("http://example.org/hasChild");
let has_spouse = ObjectProperty::new("http://example.org/hasSpouse");

ontology.add_object_property(has_parent)?;
ontology.add_object_property(has_child)?;
ontology.add_object_property(has_spouse)?;

// Data properties
let has_age = DataProperty::new("http://example.org/hasAge");
let has_name = DataProperty::new("http://example.org/hasName");

ontology.add_data_property(has_age)?;
ontology.add_data_property(has_name)?;

// Get properties
println!("Object properties: {}", ontology.object_properties().len());
println!("Data properties: {}", ontology.data_properties().len());
```

### Adding Individuals

```rust
// Create and add individuals
let john = NamedIndividual::new("http://example.org/John");
let mary = NamedIndividual::new("http://example.org/Mary");
let susan = NamedIndividual::new("http://example.org/Susan");

ontology.add_individual(john)?;
ontology.add_individual(mary)?;
ontology.add_individual(susan)?;

// Get individuals
println!("Individuals: {}", ontology.individuals().len());
for individual in ontology.individuals() {
    println!("  - {}", individual.iri());
}
```

## Axiom Management

### Subclass Axioms

```rust
use owl2_reasoner::{SubClassOfAxiom, ClassExpression};

// Create subclass axioms
let male_subclass = SubClassOfAxiom::new(
    ClassExpression::from(male_class.clone()),
    ClassExpression::from(person_class.clone())
);

let female_subclass = SubClassOfAxiom::new(
    ClassExpression::from(female_class.clone()),
    ClassExpression::from(person_class.clone())
);

ontology.add_subclass_axiom(male_subclass)?;
ontology.add_subclass_axiom(female_subclass)?;

// Get subclass axioms
println!("Subclass axioms: {}", ontology.subclass_axioms().len());
for axiom in ontology.subclass_axioms() {
    println!("  {} ⊑ {}", axiom.sub_class(), axiom.super_class());
}
```

### Equivalent Classes Axioms

```rust
use owl2_reasoner::EquivalentClassesAxiom;

// Create equivalent classes axiom
let equivalent_axiom = EquivalentClassesAxiom::new([
    ClassExpression::from(Class::new("http://example.org/Human")),
    ClassExpression::from(Class::new("http://example.org/Person")),
]);

ontology.add_equivalent_classes_axiom(equivalent_axiom)?;

// Get equivalent class axioms
for axiom in ontology.equivalent_classes_axioms() {
    println!("Equivalent: {:?}", axiom.class_expressions());
}
```

### Disjoint Classes Axioms

```rust
use owl2_reasoner::DisjointClassesAxiom;

// Create disjoint classes axiom
let disjoint_axiom = DisjointClassesAxiom::new([
    ClassExpression::from(male_class.clone()),
    ClassExpression::from(female_class.clone()),
]);

ontology.add_disjoint_classes_axiom(disjoint_axiom)?;
```

### Property Axioms

```rust
use owl2_reasoner::{
    TransitiveObjectPropertyAxiom, SymmetricObjectPropertyAxiom,
    FunctionalObjectPropertyAxiom, InverseFunctionalObjectPropertyAxiom,
    ObjectProperty, DataProperty
};

// Transitive property
let transitive_axiom = TransitiveObjectPropertyAxiom::new(
    ObjectProperty::new("http://example.org/hasAncestor")
);
ontology.add_transitive_object_property_axiom(transitive_axiom)?;

// Symmetric property
let symmetric_axiom = SymmetricObjectPropertyAxiom::new(
    ObjectProperty::new("http://example.org/hasSibling")
);
ontology.add_symmetric_object_property_axiom(symmetric_axiom)?;

// Functional property
let functional_axiom = FunctionalObjectPropertyAxiom::new(
    ObjectProperty::new("http://example.org/hasMother")
);
ontology.add_functional_object_property_axiom(functional_axiom)?;

// Data property characteristics
let functional_data_axiom = FunctionalDataPropertyAxiom::new(
    DataProperty::new("http://example.org/hasAge")
);
ontology.add_functional_data_property_axiom(functional_data_axiom)?;
```

## Individual Assertions

### Class Assertions

```rust
use owl2_reasoner::{ClassAssertionAxiom, ClassExpression};

// Assert individuals belong to classes
let john_person = ClassAssertionAxiom::new(
    ClassExpression::from(person_class.clone()),
    john.clone()
);

let john_male = ClassAssertionAxiom::new(
    ClassExpression::from(male_class.clone()),
    john.clone()
);

ontology.add_class_assertion_axiom(john_person)?;
ontology.add_class_assertion_axiom(john_male)?;
```

### Property Assertions

```rust
use owl2_reasoner::{
    ObjectPropertyAssertionAxiom, DataPropertyAssertionAxiom,
    Literal
};

// Object property assertions
let parent_assertion = ObjectPropertyAssertionAxiom::new(
    ObjectProperty::new("http://example.org/hasParent"),
    john.clone(),
    mary.clone()
);
ontology.add_object_property_assertion_axiom(parent_assertion)?;

// Data property assertions
let age_assertion = DataPropertyAssertionAxiom::new(
    DataProperty::new("http://example.org/hasAge"),
    john.clone(),
    Literal::integer(30)
);
ontology.add_data_property_assertion_axiom(age_assertion)?;

let name_assertion = DataPropertyAssertionAxiom::new(
    DataProperty::new("http://example.org/hasName"),
    john.clone(),
    Literal::string("John Doe")
);
ontology.add_data_property_assertion_axiom(name_assertion)?;
```

## Import Management

### Adding Imports

```rust
use owl2_reasoner::IRI;

// Import external ontologies
let foaf_import = IRI::new("http://xmlns.com/foaf/spec/")?;
let schema_import = IRI::new("http://schema.org/")?;

ontology.add_import(foaf_import)?;
ontology.add_import(schema_import)?;

// Get all imports
for import_iri in ontology.imports() {
    println!("Imported: {}", import_iri);
}
```

### Import Resolution

```rust
use owl2_reasoner::parser::ImportResolver;

let resolver = ImportResolver::new();

// Resolve imports automatically
let resolved_ontology = resolver.resolve_imports(&ontology)?;

// Get import dependencies
let dependencies = resolver.get_dependencies(&ontology)?;
for dependency in dependencies {
    println!("Dependency: {}", dependency);
}
```

## Ontology Statistics

### Basic Statistics

```rust
// Get comprehensive statistics
let stats = ontology.statistics();
println!("Ontology Statistics:");
println!("  Classes: {}", stats.class_count);
println!("  Object Properties: {}", stats.object_property_count);
println!("  Data Properties: {}", stats.data_property_count);
println!("  Individuals: {}", stats.individual_count);
println!("  Axioms: {}", stats.axiom_count);
println!("  Annotations: {}", stats.annotation_count);
```

### Axiom Breakdown

```rust
// Get detailed axiom counts
println!("Axiom Breakdown:");
println!("  Subclass axioms: {}", ontology.subclass_axioms().len());
println!("  Equivalent class axioms: {}", ontology.equivalent_classes_axioms().len());
println!("  Disjoint class axioms: {}", ontology.disjoint_classes_axioms().len());
println!("  Class assertions: {}", ontology.class_assertion_axioms().len());
println!("  Object property assertions: {}", ontology.object_property_assertion_axioms().len());
println!("  Data property assertions: {}", ontology.data_property_assertion_axioms().len());
```

## Validation

### Structure Validation

```rust
use owl2_reasoner::ValidationError;

// Validate ontology structure
match ontology.validate() {
    Ok(()) => println!("Ontology structure is valid"),
    Err(errors) => {
        println!("Validation errors:");
        for error in errors {
            match error {
                ValidationError::UndefinedEntity(iri) => {
                    println!("  Undefined entity: {}", iri);
                }
                ValidationError::CyclicHierarchy(class_iri) => {
                    println!("  Cyclic hierarchy: {}", class_iri);
                }
                ValidationError::InvalidAxiom(axiom) => {
                    println!("  Invalid axiom: {:?}", axiom);
                }
                ValidationError::MissingImport(import_iri) => {
                    println!("  Missing import: {}", import_iri);
                }
            }
        }
    }
}
```

### Consistency Validation

```rust
// Check internal consistency
let consistency_report = ontology.check_consistency()?;

if consistency_report.is_consistent {
    println!("Ontology is internally consistent");
} else {
    println!("Consistency issues found:");
    for issue in consistency_report.issues {
        println!("  - {}", issue.description);
    }
}
```

## Query and Search

### Finding Entities

```rust
// Find classes by pattern
let person_classes = ontology.find_classes_by_pattern("Person")?;
for class in person_classes {
    println!("Found: {}", class.iri());
}

// Find individuals by type
let people = ontology.find_individuals_by_type(&person_class.iri())?;
for individual in people {
    println!("Person: {}", individual.iri());
}

// Find properties by domain/range
let person_properties = ontology.find_properties_with_domain(&person_class.iri())?;
for prop in person_properties {
    println!("Property with Person domain: {}", prop.iri());
}
```

### Axiom Queries

```rust
// Get all axioms involving a specific entity
let person_axioms = ontology.get_axioms_for_entity(&person_class.iri())?;
println!("Axioms involving Person: {}", person_axioms.len());

// Get subclass relationships
let superclasses = ontology.get_super_classes(&person_class.iri())?;
let subclasses = ontology.get_sub_classes(&person_class.iri())?;

println!("Superclasses of Person:");
for superclass in superclasses {
    println!("  - {}", superclass);
}

println!("Subclasses of Person:");
for subclass in subclasses {
    println!("  - {}", subclass);
}
```

## Serialization

### Export to Different Formats

```rust
use owl2_reasoner::parser::{ParserFactory, SerializationFormat};

// Export to Turtle
let turtle_parser = ParserFactory::create_parser(SerializationFormat::Turtle)?;
let turtle_output = turtle_parser.serialize_ontology(&ontology)?;
println!("Turtle output:\n{}", turtle_output);

// Export to RDF/XML
let rdfxml_parser = ParserFactory::create_parser(SerializationFormat::RdfXml)?;
let rdfxml_output = rdfxml_parser.serialize_ontology(&ontology)?;
println!("RDF/XML output:\n{}", rdfxml_output);

// Export to OWL/XML
let owlexml_parser = ParserFactory::create_parser(SerializationFormat::OwlXml)?;
let owlexml_output = owlexml_parser.serialize_ontology(&ontology)?;
println!("OWL/XML output:\n{}", owlexml_output);
```

## Performance Optimization

### Index Management

```rust
// Rebuild indexes for better performance
ontology.rebuild_indexes()?;

// Get index statistics
let index_stats = ontology.index_statistics();
println!("Index Statistics:");
println!("  Class index size: {}", index_stats.class_index_size);
println!("  Property index size: {}", index_stats.property_index_size);
println!("  Individual index size: {}", index_stats.individual_index_size);

// Optimize for specific access patterns
ontology.optimize_for_queries(&[
    "get_sub_classes",
    "get_super_classes",
    "find_individuals_by_type"
])?;
```

### Memory Management

```rust
// Clear caches to free memory
ontology.clear_caches()?;

// Get memory usage
let memory_usage = ontology.memory_usage();
println!("Memory usage: {} bytes", memory_usage.total_bytes);
println!("  Indexes: {} bytes", memory_usage.index_bytes);
println!("  Cache: {} bytes", memory_usage.cache_bytes);

// Compact memory
ontology.compact_memory()?;
```

## Best Practices

1. **Use shared entities**: Prefer shared constructors for better memory efficiency
2. **Validate early**: Check ontology validity after major changes
3. **Optimize indexes**: Rebuild indexes after large modifications
4. **Manage imports**: Resolve imports before reasoning
5. **Monitor memory**: Clear caches periodically for long-running applications
6. **Use appropriate formats**: Choose serialization format based on use case

## Summary

The `Ontology` struct provides comprehensive functionality for managing OWL2 ontologies:

- **Entity management** for classes, properties, and individuals
- **Axiom management** with indexed storage for fast access
- **Import support** for multi-ontology scenarios
- **Validation** for structure and consistency checking
- **Query capabilities** for finding entities and axioms
- **Serialization** support for multiple formats
- **Performance optimization** with indexing and caching

The ontology serves as the central data structure for all OWL2 Reasoner operations.