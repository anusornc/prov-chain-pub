# Working with Ontologies

This chapter covers how to create, modify, and work with OWL2 ontologies using the OWL2 Reasoner.

## Creating an Ontology

### Basic Ontology Creation

```rust
use owl2_reasoner::{Ontology, Class, ObjectProperty, DataProperty};

// Create a new empty ontology
let mut ontology = Ontology::new();

// Set ontology IRI (recommended)
ontology.set_iri("http://example.org/my-ontology");

// Set version IRI
ontology.set_version_iri("http://example.org/my-ontology/1.0");
```

### Adding Classes

```rust
// Create named classes
let person_class = Class::new("http://example.org/Person");
let animal_class = Class::new("http://example.org/Animal");
let mammal_class = Class::new("http://example.org/Mammal");

// Add classes to ontology
ontology.add_class(person_class.clone())?;
ontology.add_class(animal_class.clone())?;
ontology.add_class(mammal_class.clone())?;

// Create anonymous classes (combinations)
let person_or_animal = ClassExpression::union([
    ClassExpression::from(person_class.clone()),
    ClassExpression::from(animal_class.clone()),
]);
```

### Adding Properties

```rust
// Object properties (relations between individuals)
let has_parent = ObjectProperty::new("http://example.org/hasParent");
let has_child = ObjectProperty::new("http://example.org/hasChild");
let eats = ObjectProperty::new("http://example.org/eats");

// Data properties (relations to data values)
let has_age = DataProperty::new("http://example.org/hasAge");
let has_name = DataProperty::new("http://example.org/hasName");

// Add properties to ontology
ontology.add_object_property(has_parent.clone())?;
ontology.add_object_property(has_child.clone())?;
ontology.add_data_property(has_age.clone())?;
```

## Class Expressions

### Basic Class Expressions

```rust
use owl2_reasoner::ClassExpression;

// Named class
let person = ClassExpression::from(Class::new("http://example.org/Person"));

// Class union (OR)
let person_or_animal = ClassExpression::union([
    person.clone(),
    ClassExpression::from(Class::new("http://example.org/Animal")),
]);

// Class intersection (AND)
let person_and_adult = ClassExpression::intersection([
    person.clone(),
    ClassExpression::from(Class::new("http://example.org/Adult")),
]);

// Complement (NOT)
let non_person = ClassExpression::complement(person.clone());
```

### Restrictions

```rust
use owl2_reasoner::{ObjectRestriction, DataRestriction};

// Object property restrictions
let has_child_restriction = ObjectRestriction::some_values_from(
    has_child.clone(),
    ClassExpression::from(Class::new("http://example.org/Person")),
);

let has_parent_restriction = ObjectRestriction::all_values_from(
    has_parent.clone(),
    ClassExpression::from(Class::new("http://example.org/Person")),
);

// Data property restrictions
let has_age_restriction = DataRestriction::some_values_from(
    has_age.clone(),
    DataRange::datatype(IRI::new("http://www.w3.org/2001/XMLSchema#integer")?),
);
```

## Adding Axioms

### Subclass Axioms

```rust
use owl2_reasoner::{SubClassOfAxiom, ClassExpression};

// Subclass relationship: Parent ⊑ Person
let subclass_axiom = SubClassOfAxiom::new(
    ClassExpression::from(Class::new("http://example.org/Parent")),
    ClassExpression::from(Class::new("http://example.org/Person")),
);

ontology.add_subclass_axiom(subclass_axiom)?;
```

### Equivalent Classes

```rust
use owl2_reasoner::EquivalentClassesAxiom;

// Equivalent classes: Human ≡ Person
let equivalent_axiom = EquivalentClassesAxiom::new([
    ClassExpression::from(Class::new("http://example.org/Human")),
    ClassExpression::from(Class::new("http://example.org/Person")),
]);

ontology.add_equivalent_classes_axiom(equivalent_axiom)?;
```

### Disjoint Classes

```rust
use owl2_reasoner::DisjointClassesAxiom;

// Disjoint classes: Animal ⊓ Plant = ⊥
let disjoint_axiom = DisjointClassesAxiom::new([
    ClassExpression::from(Class::new("http://example.org/Animal")),
    ClassExpression::from(Class::new("http://example.org/Plant")),
]);

ontology.add_disjoint_classes_axiom(disjoint_axiom)?;
```

### Property Characteristics

```rust
use owl2_reasoner::{
    TransitiveObjectPropertyAxiom, SymmetricObjectPropertyAxiom,
    FunctionalObjectPropertyAxiom, InverseFunctionalObjectPropertyAxiom
};

// Transitive property: hasAncestor
let transitive_axiom = TransitiveObjectPropertyAxiom::new(
    ObjectProperty::new("http://example.org/hasAncestor")
);

// Symmetric property: hasSibling
let symmetric_axiom = SymmetricObjectPropertyAxiom::new(
    ObjectProperty::new("http://example.org/hasSibling")
);

// Functional property: hasMother
let functional_axiom = FunctionalObjectPropertyAxiom::new(
    ObjectProperty::new("http://example.org/hasMother")
);

ontology.add_transitive_object_property_axiom(transitive_axiom)?;
ontology.add_symmetric_object_property_axiom(symmetric_axiom)?;
ontology.add_functional_object_property_axiom(functional_axiom)?;
```

## Working with Individuals

```rust
use owl2_reasoner::{
    NamedIndividual, ClassAssertionAxiom, ObjectPropertyAssertionAxiom,
    DataPropertyAssertionAxiom, ClassExpression
};

// Create individuals
let john = NamedIndividual::new("http://example.org/John");
let mary = NamedIndividual::new("http://example.org/Mary");
let susan = NamedIndividual::new("http://example.org/Susan");

// Class assertions
ontology.add_individual(john.clone())?;
ontology.add_individual(mary.clone())?;
ontology.add_individual(susan.clone())?;

// Assert John is a Person
let john_person_assertion = ClassAssertionAxiom::new(
    ClassExpression::from(Class::new("http://example.org/Person")),
    john.clone(),
);
ontology.add_class_assertion_axiom(john_person_assertion)?;

// Object property assertions: John hasParent Mary
let parent_assertion = ObjectPropertyAssertionAxiom::new(
    has_parent.clone(),
    john.clone(),
    mary.clone(),
);
ontology.add_object_property_assertion_axiom(parent_assertion)?;

// Data property assertions: John hasAge 30
let age_assertion = DataPropertyAssertionAxiom::new(
    has_age.clone(),
    john.clone(),
    Literal::integer(30),
);
ontology.add_data_property_assertion_axiom(age_assertion)?;
```

## Ontology Statistics

```rust
// Get basic statistics
let class_count = ontology.classes().len();
let individual_count = ontology.individuals().len();
let axiom_count = ontology.axiom_count();

println!("Ontology contains:");
println!("  {} classes", class_count);
println!("  {} individuals", individual_count);
println!("  {} axioms", axiom_count);

// Get specific axiom counts
let subclass_axioms = ontology.subclass_axioms().len();
let equivalent_axioms = ontology.equivalent_classes_axioms().len();
let disjoint_axioms = ontology.disjoint_classes_axioms().len();

println!("  {} subclass axioms", subclass_axioms);
println!("  {} equivalent class axioms", equivalent_axioms);
println!("  {} disjoint class axioms", disjoint_axioms);
```

## Import Management

```rust
// Import other ontologies
let imported_iri = IRI::new("http://example.org/imported-ontology")?;
ontology.add_import(imported_iri)?;

// Get all imports
for import_iri in ontology.imports() {
    println!("Imported: {}", import_iri);
}
```

## Ontology Validation

```rust
use owl2_reasoner::ValidationError;

// Validate ontology structure
match ontology.validate() {
    Ok(()) => println!("Ontology is valid"),
    Err(errors) => {
        for error in errors {
            match error {
                ValidationError::UndefinedEntity(iri) => {
                    println!("Undefined entity: {}", iri);
                }
                ValidationError::CyclicHierarchy(class_iri) => {
                    println!("Cyclic hierarchy involving: {}", class_iri);
                }
                ValidationError::InvalidAxiom(axiom) => {
                    println!("Invalid axiom: {:?}", axiom);
                }
            }
        }
    }
}
```

## Best Practices

1. **Use meaningful IRIs**: Create descriptive, persistent IRIs for your entities
2. **Organize imports**: Import external ontologies at the beginning
3. **Add annotations**: Use rdfs:label and rdfs:comment for documentation
4. **Validate early**: Check ontology validity after major changes
5. **Use profiles**: Consider OWL2 profiles for better performance
6. **Test reasoning**: Verify reasoner results match expectations

## Summary

This chapter covered the essential operations for creating and manipulating OWL2 ontologies:

- Creating and configuring ontologies
- Adding classes, properties, and individuals
- Working with complex class expressions
- Adding various types of axioms
- Managing ontology imports
- Validating ontology structure

**Next**: [Reasoning](reasoning.md) - Learn how to use reasoning capabilities to infer new knowledge.