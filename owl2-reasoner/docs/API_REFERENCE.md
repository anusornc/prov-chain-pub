# OWL2 Reasoner API Reference

## Table of Contents

1. [Core Data Types](#core-data-types)
2. [Ontology Management](#ontology-management)
3. [Entities and Axioms](#entities-and-axioms)
4. [Reasoning Engine](#reasoning-engine)
5. [Parser Framework](#parser-framework)
6. [Query Engine](#query-engine)
7. [Error Handling](#error-handling)
8. [Examples and Use Cases](#examples-and-use-cases)

## Core Data Types

### IRI (Internationalized Resource Identifier)

```rust
use owl2_reasoner::iri::IRI;

// Create IRI from string
let iri = IRI::new("http://example.org/Person")?;

// Create IRI with namespace
let iri = IRI::with_namespace("ex", "Person")?;

// IRI operations
let namespace = iri.namespace();
let local_name = iri.local_name();
let full_str = iri.as_str();
```

### ClassExpression

```rust
use owl2_reasoner::entities::Class;
use owl2_reasoner::axioms::ClassExpression;

// Simple class
let person_class = Class::new("http://example.org/Person");
let class_expr = ClassExpression::Class(person_class);

// Object restrictions
let restriction = ClassExpression::ObjectSomeValuesFrom(
    ObjectPropertyExpression::ObjectProperty(has_child_property),
    Box::new(ClassExpression::Class(person_class))
);

// Complex expressions
let intersection = ClassExpression::ObjectIntersectionOf(vec![
    Box::new(ClassExpression::Class(person_class)),
    Box::new(ClassExpression::Class(adult_class))
]);
```

## Ontology Management

### Creating and Managing Ontologies

```rust
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::entities::Class;
use owl2_reasoner::axioms::{SubClassOfAxiom, ClassExpression};

// Create empty ontology
let mut ontology = Ontology::new();

// Add classes
let person = Class::new("http://example.org/Person");
let adult = Class::new("http://example.org/Adult");
ontology.add_class(person.clone())?;
ontology.add_class(adult.clone())?;

// Add subclass relationship
let subclass_axiom = SubClassOfAxiom::new(
    ClassExpression::Class(adult),
    ClassExpression::Class(person)
);
ontology.add_subclass_axiom(subclass_axiom)?;

// Query ontology
let classes = ontology.classes();
let axioms = ontology.subclass_axioms();
let individuals = ontology.individuals();
```

### Ontology Storage and Indexing

```rust
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::storage::StorageBackend;

// Create ontology with custom storage
let mut ontology = Ontology::with_storage(MyCustomStorage::new());

// Access indexed data
let class_count = ontology.class_count();
let axiom_count = ontology.axiom_count();
let individual_count = ontology.individual_count();

// Search operations
let person_axioms = ontology.find_axioms_for_class(&person_iri);
let property_axioms = ontology.find_axioms_for_property(&property_iri);
```

## Entities and Axioms

### Working with Classes

```rust
use owl2_reasoner::entities::Class;

// Create class
let person = Class::new("http://example.org/Person");

// Class operations
let iri = person.iri();
let annotation_properties = person.annotation_properties();

// Create equivalent classes
let human = Class::new("http://example.org/Human");
let equivalent_axiom = EquivalentClassesAxiom::new(vec![
    ClassExpression::Class(person),
    ClassExpression::Class(human)
]);
```

### Working with Properties

```rust
use owl2_reasoner::entities::{
    ObjectProperty, DataProperty,
    ObjectPropertyExpression, DataPropertyExpression
};

// Object properties
let has_child = ObjectProperty::new("http://example.org/hasChild");
let has_parent = ObjectProperty::new("http://example.org/hasParent");

// Property characteristics
let functional_axiom = FunctionalObjectPropertyAxiom::new(has_child.iri().clone());
let inverse_axiom = InverseObjectPropertiesAxiom::new(
    has_child.iri().clone(),
    has_parent.iri().clone()
);

// Data properties
let has_age = DataProperty::new("http://example.org/hasAge");
let range_axiom = DataPropertyRangeAxiom::new(
    has_age.iri().clone(),
    DataRange::Datatype(XSDDatatype::Integer)
);
```

### Working with Individuals

```rust
use owl2_reasoner::entities::NamedIndividual;
use owl2_reasoner::axioms::{ClassAssertionAxiom, ObjectPropertyAssertionAxiom};

// Create individuals
let john = NamedIndividual::new("http://example.org/John");
let mary = NamedIndividual::new("http://example.org/Mary");

// Class assertions
let person_class = Class::new("http://example.org/Person");
let class_assertion = ClassAssertionAxiom::new(
    ClassExpression::Class(person_class),
    john.iri().clone()
);

// Property assertions
let property_assertion = ObjectPropertyAssertionAxiom::new(
    has_child.iri().clone(),
    john.iri().clone(),
    mary.iri().clone()
);
```

## Reasoning Engine

### Simple Reasoner

```rust
use owl2_reasoner::reasoning::SimpleReasoner;

// Create reasoner
let reasoner = SimpleReasoner::new(ontology);

// Consistency checking
let is_consistent = reasoner.is_consistent()?;

// Class satisfiability
let is_satisfiable = reasoner.is_class_satisfiable(&class_iri)?;

// Subclass relationships
let is_subclass = reasoner.is_subclass_of(&subclass_iri, &superclass_iri)?;

// Equivalent classes
let are_equivalent = reasoner.are_equivalent_classes(&class1_iri, &class2_iri)?;

// Instance checking
let is_instance = reasoner.is_instance_of(&individual_iri, &class_iri)?;
```

### Advanced Reasoner (OwlReasoner)

```rust
use owl2_reasoner::reasoning::{
    OwlReasoner, ReasoningConfig,
    tableaux::ReasoningConfig as TableauxConfig
};

// Configure advanced reasoning
let tableaux_config = TableauxConfig {
    max_depth: 2000,
    debug: false,
    incremental: true,
    timeout: Some(45000),
};

let reasoning_config = ReasoningConfig {
    enable_reasoning: true,
    use_advanced_reasoning: true,
    tableaux_config,
};

// Create advanced reasoner
let mut reasoner = OwlReasoner::with_config(ontology, reasoning_config);

// Advanced reasoning capabilities
let is_consistent = reasoner.is_consistent()?;
let is_satisfiable = reasoner.is_class_satisfiable(&class_iri)?;
let classification_results = reasoner.classify()?;
```

### Tableaux Reasoner

```rust
use owl2_reasoner::reasoning::tableaux::TableauxReasoner;

// Create tableaux reasoner
let mut tableaux_reasoner = TableauxReasoner::new(ontology);

// Tableaux-specific operations
let tableau_result = tableaux_reasoner.build_tableau()?;
let satisfiable = tableaux_reasoner.is_satisfiable(&class_expression)?;
let classification = tableaux_reasoner.classify()?;
```

## Parser Framework

### Multi-format Parsing

```rust
use owl2_reasoner::parser::{
    OntologyParser, TurtleParser, RdfXmlParser,
    OwlFunctionalSyntaxParser
};

// Turtle parsing
let turtle_parser = TurtleParser::new();
let ontology = turtle_parser.parse_file("path/to/file.ttl")?;

// RDF/XML parsing
let rdf_parser = RdfXmlParser::new();
let ontology = rdf_parser.parse_file("path/to/file.rdf")?;

// OWL Functional Syntax parsing
let ofn_parser = OwlFunctionalSyntaxParser::new();
let ontology = ofn_parser.parse_file("path/to/file.ofn")?;

// Parsing from string
let ontology = turtle_parser.parse_str(turtle_content)?;
```

### Parser Configuration

```rust
use owl2_reasoner::parser::ParserConfig;

// Configure parser
let config = ParserConfig {
    strict_mode: true,
    validate_uris: true,
    base_iri: Some("http://example.org/base/".to_string()),
    namespaces: vec![
        ("ex".to_string(), "http://example.org/".to_string()),
        ("rdf".to_string(), "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string()),
    ],
};

let parser = TurtleParser::with_config(config);
let ontology = parser.parse_file("path/to/file.ttl")?;
```

## Query Engine

### SPARQL-like Querying

```rust
use owl2_reasoner::reasoning::query::{
    QueryEngine, QueryPattern, QueryResult
};

// Create query engine
let mut query_engine = QueryEngine::new(ontology);

// Basic graph pattern query
let pattern = QueryPattern::BasicGraphPattern(vec![
    TriplePattern {
        subject: Variable::new("s"),
        predicate: IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")?,
        object: IRI::new("http://example.org/Person")?,
    }
]);

// Execute query
let result = query_engine.execute_query(&pattern)?;

// Process results
for binding in result.bindings() {
    let subject = binding.get("s").unwrap();
    println!("Found person: {}", subject);
}
```

### Complex Queries

```rust
// Union query
let pattern = QueryPattern::Union(vec![
    QueryPattern::BasicGraphPattern(person_pattern),
    QueryPattern::BasicGraphPattern(organization_pattern),
]);

// Optional query
let pattern = QueryPattern::Optional(vec![
    QueryPattern::BasicGraphPattern(person_pattern),
    QueryPattern::BasicGraphPattern(name_pattern),
]);

// Filter query
let pattern = QueryPattern::Filter(
    Box::new(QueryPattern::BasicGraphPattern(pattern)),
    Expression::GreaterThan(
        Box::new(Expression::Variable("age")),
        Box::new(Expression::Literal("18"))
    )
);
```

## Error Handling

### Error Types

```rust
use owl2_reasoner::error::{OwlError, OwlResult};

// Result type alias
type OwlResult<T> = Result<T, OwlError>;

// Error handling
fn process_ontology() -> OwlResult<()> {
    let ontology = parse_ontology()?; // Returns OwlResult<Ontology>
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent()?;

    if is_consistent {
        println!("Ontology is consistent");
    } else {
        println!("Ontology is inconsistent");
    }

    Ok(())
}

// Error matching
match parse_ontology() {
    Ok(ontology) => println!("Successfully parsed ontology"),
    Err(OwlError::ParseError(msg)) => eprintln!("Parse error: {}", msg),
    Err(OwlError::IoError(e)) => eprintln!("I/O error: {}", e),
    Err(OwlError::ReasoningError(msg)) => eprintln!("Reasoning error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

### Custom Error Handling

```rust
use owl2_reasoner::error::OwlError;

// Custom error handling
fn safe_class_satisfiability(reasoner: &mut SimpleReasoner, class_iri: &IRI) -> bool {
    match reasoner.is_class_satisfiable(class_iri) {
        Ok(true) => true,
        Ok(false) => false,
        Err(e) => {
            eprintln!("Error checking satisfiability: {}", e);
            false // Default to false on error
        }
    }
}
```

## Examples and Use Cases

### Example 1: Family Relationships

```rust
use owl2_reasoner::*;

fn create_family_ontology() -> OwlResult<Ontology> {
    let mut ontology = Ontology::new();

    // Create classes
    let person = Class::new("http://example.org/family/Person");
    let parent = Class::new("http://example.org/family/Parent");
    let child = Class::new("http://example.org/family/Child");

    ontology.add_class(person.clone())?;
    ontology.add_class(parent.clone())?;
    ontology.add_class(child.clone())?;

    // Create properties
    let has_child = ObjectProperty::new("http://example.org/family/hasChild");
    let has_parent = ObjectProperty::new("http://example.org/family/hasParent");

    // Add axioms
    let parent_subclass = SubClassOfAxiom::new(
        ClassExpression::Class(parent),
        ClassExpression::Class(person)
    );
    let child_subclass = SubClassOfAxiom::new(
        ClassExpression::Class(child),
        ClassExpression::Class(person)
    );

    ontology.add_subclass_axiom(parent_subclass)?;
    ontology.add_subclass_axiom(child_subclass)?;

    Ok(ontology)
}

fn reason_about_family() -> OwlResult<()> {
    let ontology = create_family_ontology()?;
    let mut reasoner = SimpleReasoner::new(ontology);

    // Check consistency
    let is_consistent = reasoner.is_consistent()?;
    println!("Family ontology consistent: {}", is_consistent);

    // Check subclass relationships
    let parent_iri = IRI::new("http://example.org/family/Parent")?;
    let person_iri = IRI::new("http://example.org/family/Person")?;
    let is_subclass = reasoner.is_subclass_of(&parent_iri, &person_iri)?;
    println!("Parent ⊑ Person: {}", is_subclass);

    Ok(())
}
```

### Example 2: Biomedical Ontology

```rust
use owl2_reasoner::*;

fn create_biomedical_ontology() -> OwlResult<Ontology> {
    let mut ontology = Ontology::new();

    // Create disease and symptom classes
    let disease = Class::new("http://example.org/medical/Disease");
    let symptom = Class::new("http://example.org/medical/Symptom");
    let treatment = Class::new("http://example.org/medical/Treatment");

    // Create relationships
    let has_symptom = ObjectProperty::new("http://example.org/medical/hasSymptom");
    let has_treatment = ObjectProperty::new("http://example.org/medical/hasTreatment");

    // Add domain and range restrictions
    let symptom_domain = ObjectPropertyDomainAxiom::new(
        has_symptom.iri().clone(),
        disease.iri().clone()
    );
    let symptom_range = ObjectPropertyRangeAxiom::new(
        has_symptom.iri().clone(),
        symptom.iri().clone()
    );

    ontology.add_class(disease)?;
    ontology.add_class(symptom)?;
    ontology.add_class(treatment)?;
    ontology.add_object_property_domain_axiom(symptom_domain)?;
    ontology.add_object_property_range_axiom(symptom_range)?;

    Ok(ontology)
}

fn medical_reasoning() -> OwlResult<()> {
    let ontology = create_biomedical_ontology()?;
    let mut reasoner = OwlReasoner::new(ontology);

    // Use advanced reasoning for medical ontology
    let is_consistent = reasoner.is_consistent()?;
    println!("Medical ontology consistent: {}", is_consistent);

    // Check class satisfiability
    let disease_iri = IRI::new("http://example.org/medical/Disease")?;
    let is_satisfiable = reasoner.is_class_satisfiable(&disease_iri)?;
    println!("Disease class satisfiable: {}", is_satisfiable);

    Ok(())
}
```

### Example 3: Query Processing

```rust
use owl2_reasoner::*;

fn query_example() -> OwlResult<()> {
    let ontology = create_family_ontology()?;
    let mut query_engine = QueryEngine::new(ontology);

    // Find all parents
    let pattern = QueryPattern::BasicGraphPattern(vec![
        TriplePattern {
            subject: Variable::new("person"),
            predicate: IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")?,
            object: IRI::new("http://example.org/family/Parent")?,
        }
    ]);

    let result = query_engine.execute_query(&pattern)?;

    println!("Found {} parents:", result.bindings().len());
    for binding in result.bindings() {
        if let Some(person) = binding.get("person") {
            println!("  - {}", person);
        }
    }

    Ok(())
}
```

## Best Practices

### 1. Memory Management
- Use `Arc<Ontology>` for sharing ontologies between reasoners
- Clear caches when memory is constrained
- Use appropriate data structures for large ontologies

### 2. Performance Optimization
- Configure reasoners appropriately for your use case
- Use SimpleReasoner for basic reasoning needs
- Use OwlReasoner with advanced reasoning for complex ontologies
- Cache results when possible

### 3. Error Handling
- Always handle errors appropriately
- Use meaningful error messages
- Log errors for debugging purposes

### 4. Testing
- Test ontologies for consistency before reasoning
- Validate reasoning results with expected outcomes
- Use test suites for compliance checking

## Migration Guide

### From SimpleReasoner to OwlReasoner

```rust
// Before (SimpleReasoner)
let mut reasoner = SimpleReasoner::new(ontology);
let is_consistent = reasoner.is_consistent()?;

// After (OwlReasoner)
let mut reasoner = OwlReasoner::new(ontology);
let is_consistent = reasoner.is_consistent()?;

// The API remains compatible for basic operations
```

### From Legacy Parser to New Parser Framework

```rust
// Before (legacy parser)
let ontology = parse_ontology_old_way(content)?;

// After (new parser framework)
let parser = TurtleParser::new();
let ontology = parser.parse_str(content)?;
```

## Conclusion

This API reference provides comprehensive documentation for the OWL2 Reasoner library. The API is designed to be intuitive, flexible, and performant, supporting both basic and advanced OWL2 reasoning use cases.

For additional examples and tutorials, please refer to the [User Guide](USER_GUIDE.md) and [Examples](../examples/) directory.