# Basic Usage

This chapter covers the fundamental usage patterns of the OWL2 Reasoner library.

## Creating Your First Ontology

Let's start by creating a simple family ontology:

```rust
use owl2_reasoner::{
    Ontology, Class, ObjectProperty, NamedIndividual,
    SubClassOfAxiom, ClassAssertionAxiom, PropertyAssertionAxiom,
    ClassExpression, IRI
};

fn main() -> OwlResult<()> {
    // Create a new ontology
    let mut ontology = Ontology::new();
    ontology.set_iri("http://example.org/family");
    
    // Define classes
    let person_class = Class::new("http://example.org/Person");
    let parent_class = Class::new("http://example.org/Parent");
    let child_class = Class::new("http://example.org/Child");
    
    // Add classes to ontology
    ontology.add_class(person_class.clone())?;
    ontology.add_class(parent_class.clone())?;
    ontology.add_class(child_class.clone())?;
    
    println!("Created ontology with {} classes", ontology.classes().len());
    Ok(())
}
```

## Adding Relationships

Now let's add some relationships between classes:

```rust
// Parent is a subclass of Person
let parent_subclass = SubClassOfAxiom::new(
    ClassExpression::from(parent_class.clone()),
    ClassExpression::from(person_class.clone()),
);
ontology.add_subclass_axiom(parent_subclass)?;

// Child is a subclass of Person
let child_subclass = SubClassOfAxiom::new(
    ClassExpression::from(child_class.clone()),
    ClassExpression::from(person_class.clone()),
);
ontology.add_subclass_axiom(child_subclass)?;

// Define properties
let has_parent = ObjectProperty::new("http://example.org/hasParent");
let has_child = ObjectProperty::new("http://example.org/hasChild");

ontology.add_object_property(has_parent.clone())?;
ontology.add_object_property(has_child.clone())?;
```

## Adding Individuals and Assertions

Let's add some family members and their relationships:

```rust
// Create individuals
let john = NamedIndividual::new("http://example.org/John");
let mary = NamedIndividual::new("http://example.org/Mary");
let alice = NamedIndividual::new("http://example.org/Alice");

// Add individuals to ontology
ontology.add_named_individual(john.clone())?;
ontology.add_named_individual(mary.clone())?;
ontology.add_named_individual(alice.clone())?;

// Add class assertions
ontology.add_class_assertion(ClassAssertionAxiom::new(
    ClassExpression::from(parent_class.clone()),
    john.clone(),
))?;

ontology.add_class_assertion(ClassAssertionAxiom::new(
    ClassExpression::from(parent_class.clone()),
    mary.clone(),
))?;

ontology.add_class_assertion(ClassAssertionAxiom::new(
    ClassExpression::from(child_class.clone()),
    alice.clone(),
))?;

// Add property assertions
ontology.add_property_assertion(PropertyAssertionAxiom::new(
    has_child.clone(),
    john.clone(),
    alice.clone(),
))?;

ontology.add_property_assertion(PropertyAssertionAxiom::new(
    has_child.clone(),
    mary.clone(),
    alice.clone(),
))?;

ontology.add_property_assertion(PropertyAssertionAxiom::new(
    has_parent.clone(),
    alice.clone(),
    john.clone(),
))?;
```

## Basic Reasoning

Now let's use the reasoner to infer new knowledge:

```rust
use owl2_reasoner::SimpleReasoner;

// Create a reasoner
let reasoner = SimpleReasoner::new(ontology);

// Check if the ontology is consistent
let is_consistent = reasoner.is_consistent()?;
println!("Ontology is consistent: {}", is_consistent);

// Check subclass relationships
let is_parent_subclass_of_person = reasoner.is_subclass_of(&parent_class, &person_class)?;
println!("Parent ⊑ Person: {}", is_parent_subclass_of_person);

// Get all instances of Person
let person_instances = reasoner.get_instances(&person_class)?;
println!("Persons: {:?}", person_instances);

// Get all instances of Parent
let parent_instances = reasoner.get_instances(&parent_class)?;
println!("Parents: {:?}", parent_instances);
```

## Querying the Knowledge Base

The query engine allows you to retrieve information using patterns:

```rust
use owl2_reasoner::query::{QueryEngine, QueryPattern, QueryValue};

// Create a query engine
let mut query_engine = QueryEngine::new(&reasoner.ontology);

// Find all parents
let parent_pattern = QueryPattern::Basic {
    subject: None,
    predicate: Some(QueryValue::IRI(IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")?)),
    object: Some(QueryValue::IRI(parent_class.clone())),
};

let parents = query_engine.query_pattern(&parent_pattern)?;
println!("Found {} parents", parents.len());

// Find all parent-child relationships
let family_pattern = QueryPattern::Basic {
    subject: None,
    predicate: Some(QueryValue::IRI(has_child.clone())),
    object: None,
};

let relationships = query_engine.query_pattern(&family_pattern)?;
println!("Found {} parent-child relationships", relationships.len());

// Complex query: Find all parents who have children
let complex_pattern = QueryPattern::And(vec![
    QueryPattern::Basic {
        subject: None,
        predicate: Some(QueryValue::IRI(IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")?)),
        object: Some(QueryValue::IRI(parent_class.clone())),
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

## Working with Property Characteristics

OWL2 properties can have various characteristics:

```rust
use owl2_reasoner::{ObjectPropertyCharacteristic, ObjectPropertyExpression};

// Create a property with characteristics
let mut has_ancestor = ObjectProperty::new("http://example.org/hasAncestor");

// Add characteristics
has_ancestor.add_characteristic(ObjectPropertyCharacteristic::Transitive);
has_ancestor.add_characteristic(ObjectPropertyCharacteristic::Asymmetric);
has_ancestor.add_characteristic(ObjectPropertyCharacteristic::Irreflexive);

ontology.add_object_property(has_ancestor)?;

// Add some ancestor relationships
ontology.add_property_assertion(PropertyAssertionAxiom::new(
    has_ancestor.clone(),
    alice.clone(),
    john.clone(),
))?;

ontology.add_property_assertion(PropertyAssertionAxiom::new(
    has_ancestor.clone(),
    john.clone(),
    mary.clone(),
))?;
```

## Complex Class Expressions

You can create complex class expressions using Boolean operators and restrictions:

```rust
// Intersection: Parent ⊓ Male
let male_class = Class::new("http://example.org/Male");
ontology.add_class(male_class.clone())?;

let father_class = ClassExpression::ObjectIntersectionOf(vec![
    ClassExpression::from(parent_class.clone()),
    ClassExpression::from(male_class.clone()),
]);

// Union: Parent ⊓ Child
let parent_or_child = ClassExpression::ObjectUnionOf(vec![
    ClassExpression::from(parent_class.clone()),
    ClassExpression::from(child_class.clone()),
]);

// Complement: ¬Child
let not_child = ClassExpression::ObjectComplementOf(Box::new(
    ClassExpression::from(child_class.clone()),
));

// Existential restriction: ∃hasChild.Person
let has_child_person = ClassExpression::ObjectSomeValuesFrom(
    Box::new(ObjectPropertyExpression::ObjectProperty(has_child.clone())),
    Box::new(ClassExpression::from(person_class.clone())),
);

// Universal restriction: ∀hasParent.Person
let all_parents_person = ClassExpression::ObjectAllValuesFrom(
    Box::new(ObjectPropertyExpression::ObjectProperty(has_parent.clone())),
    Box::new(ClassExpression::from(person_class.clone())),
);
```

## Error Handling

The library provides comprehensive error handling:

```rust
use owl2_reasoner::OwlError;

fn safe_ontology_operations() -> OwlResult<()> {
    // Try to create an invalid IRI
    let result = IRI::new("");
    match result {
        Ok(_) => println!("IRI created successfully"),
        Err(OwlError::InvalidIRI(msg)) => println!("Invalid IRI: {}", msg),
        Err(e) => println!("Other error: {}", e),
    }
    
    // Try to add duplicate class
    let mut ontology = Ontology::new();
    let class = Class::new("http://example.org/Test");
    ontology.add_class(class.clone())?;
    
    // This will not error but will be ignored (classes are unique)
    let result = ontology.add_class(class.clone());
    match result {
        Ok(_) => println!("Class handling completed"),
        Err(e) => println!("Error adding class: {}", e),
    }
    
    Ok(())
}
```

## Performance Tips

### 1. Use IRI Caching
```rust
// IRIs are automatically cached, but you can access the cache
use owl2_reasoner::{global_iri_cache_stats, clear_global_iri_cache};

let stats = global_iri_cache_stats();
println!("IRI cache hit rate: {:.2}%", stats.hit_rate() * 100.0);

// Clear cache if needed (use sparingly)
clear_global_iri_cache();
```

### 2. Manage Reasoner Caches
```rust
let reasoner = SimpleReasoner::new(ontology);

// Clear all caches
reasoner.clear_caches();

// Get cache statistics
let cache_stats = reasoner.cache_stats();
println!("Cache statistics: {:?}", cache_stats);
```

### 3. Batch Operations
```rust
// Batch operations are more efficient
let mut ontology = Ontology::new();
let mut axioms = Vec::new();

// Prepare axioms
for i in 0..1000 {
    let class = Class::new(&format!("http://example.org/Class{}", i));
    ontology.add_class(class)?;
    
    let subclass_axiom = SubClassOfAxiom::new(
        ClassExpression::from(class),
        ClassExpression::from(person_class.clone()),
    );
    axioms.push(subclass_axiom);
}

// Add all axioms at once
for axiom in axioms {
    ontology.add_subclass_axiom(axiom)?;
}
```

## Complete Example

Here's a complete example that puts everything together:

```rust
use owl2_reasoner::*;

fn create_family_ontology() -> OwlResult<()> {
    // Create ontology
    let mut ontology = Ontology::new();
    ontology.set_iri("http://example.org/family");
    
    // Define classes
    let person = Class::new("http://example.org/Person");
    let parent = Class::new("http://example.org/Parent");
    let child = Class::new("http://example.org/Child");
    let male = Class::new("http://example.org/Male");
    let female = Class::new("http://example.org/Female");
    
    // Add classes
    for class in &[person.clone(), parent.clone(), child.clone(), male.clone(), female.clone()] {
        ontology.add_class(class.clone())?;
    }
    
    // Define properties
    let has_parent = ObjectProperty::new("http://example.org/hasParent");
    let has_child = ObjectProperty::new("http://example.org/hasChild");
    let has_spouse = ObjectProperty::new("http://example.org/hasSpouse");
    
    // Add properties
    for prop in &[has_parent.clone(), has_child.clone(), has_spouse.clone()] {
        ontology.add_object_property(prop.clone())?;
    }
    
    // Add subclass relationships
    ontology.add_subclass_axiom(SubClassOfAxiom::new(
        ClassExpression::from(parent.clone()),
        ClassExpression::from(person.clone()),
    ))?;
    
    ontology.add_subclass_axiom(SubClassOfAxiom::new(
        ClassExpression::from(child.clone()),
        ClassExpression::from(person.clone()),
    ))?;
    
    // Add individuals
    let john = NamedIndividual::new("http://example.org/John");
    let mary = NamedIndividual::new("http://example.org/Mary");
    let alice = NamedIndividual::new("http://example.org/Alice");
    let bob = NamedIndividual::new("http://example.org/Bob");
    
    for individual in &[john.clone(), mary.clone(), alice.clone(), bob.clone()] {
        ontology.add_named_individual(individual.clone())?;
    }
    
    // Add class assertions
    ontology.add_class_assertion(ClassAssertionAxiom::new(
        ClassExpression::from(parent.clone()),
        john.clone(),
    ))?;
    
    ontology.add_class_assertion(ClassAssertionAxiom::new(
        ClassExpression::from(parent.clone()),
        mary.clone(),
    ))?;
    
    ontology.add_class_assertion(ClassAssertionAxiom::new(
        ClassExpression::from(child.clone()),
        alice.clone(),
    ))?;
    
    ontology.add_class_assertion(ClassAssertionAxiom::new(
        ClassExpression::from(child.clone()),
        bob.clone(),
    ))?;
    
    // Add property assertions
    ontology.add_property_assertion(PropertyAssertionAxiom::new(
        has_child.clone(),
        john.clone(),
        alice.clone(),
    ))?;
    
    ontology.add_property_assertion(PropertyAssertionAxiom::new(
        has_child.clone(),
        mary.clone(),
        alice.clone(),
    ))?;
    
    ontology.add_property_assertion(PropertyAssertionAxiom::new(
        has_child.clone(),
        john.clone(),
        bob.clone(),
    ))?;
    
    ontology.add_property_assertion(PropertyAssertionAxiom::new(
        has_child.clone(),
        mary.clone(),
        bob.clone(),
    ))?;
    
    ontology.add_property_assertion(PropertyAssertionAxiom::new(
        has_spouse.clone(),
        john.clone(),
        mary.clone(),
    ))?;
    
    // Reasoning
    let reasoner = SimpleReasoner::new(ontology);
    
    println!("Ontology is consistent: {}", reasoner.is_consistent()?);
    println!("Parent ⊑ Person: {}", reasoner.is_subclass_of(&parent, &person)?);
    println!("Child ⊑ Person: {}", reasoner.is_subclass_of(&child, &person)?);
    
    let parents = reasoner.get_instances(&parent)?;
    let children = reasoner.get_instances(&child)?;
    
    println!("Parents: {:?}", parents);
    println!("Children: {:?}", children);
    
    Ok(())
}

fn main() {
    if let Err(e) = create_family_ontology() {
        eprintln!("Error: {}", e);
    }
}
```

This example demonstrates the core functionality of the OWL2 Reasoner library. You can build upon this foundation to create more complex ontologies and reasoning applications.