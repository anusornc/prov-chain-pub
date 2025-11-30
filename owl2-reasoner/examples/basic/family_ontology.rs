//! Simplified Family Ontology Example
//!
//! This example demonstrates basic OWL2 reasoning functionality.

use owl2_reasoner::*;

fn main() -> OwlResult<()> {
    println!("=== Simplified Family Ontology Example ===\n");

    // Create a new ontology
    let mut ontology = Ontology::new();
    ontology.set_iri("http://example.org/family");

    // Define classes
    let person = Class::new("http://example.org/Person");
    let parent = Class::new("http://example.org/Parent");

    // Add classes to ontology
    ontology.add_class(person.clone())?;
    ontology.add_class(parent.clone())?;

    println!("✓ Added {} classes", ontology.classes().len());

    // Define property
    let has_child = ObjectProperty::new("http://example.org/hasChild");
    ontology.add_object_property(has_child.clone())?;

    println!(
        "✓ Added {} object properties",
        ontology.object_properties().len()
    );

    // Add subclass relationship
    let subclass_axiom = SubClassOfAxiom::new(
        ClassExpression::from(parent.clone()),
        ClassExpression::from(person.clone()),
    );
    ontology.add_subclass_axiom(subclass_axiom)?;

    println!(
        "✓ Added {} subclass axioms",
        ontology.subclass_axioms().len()
    );

    // Add individuals
    let john = NamedIndividual::new("http://example.org/John");
    let mary = NamedIndividual::new("http://example.org/Mary");

    ontology.add_named_individual(john.clone())?;
    ontology.add_named_individual(mary.clone())?;

    println!(
        "✓ Added {} named individuals",
        ontology.named_individuals().len()
    );

    // Add class assertions
    let john_person =
        ClassAssertionAxiom::new(john.iri().clone(), ClassExpression::from(person.clone()));
    let mary_parent =
        ClassAssertionAxiom::new(mary.iri().clone(), ClassExpression::from(parent.clone()));

    ontology.add_class_assertion(john_person)?;
    ontology.add_class_assertion(mary_parent)?;

    println!(
        "✓ Added {} class assertions",
        ontology.class_assertions().len()
    );

    // Add property assertion
    let john_has_mary = PropertyAssertionAxiom::new(
        john.iri().clone(),
        has_child.iri().clone(),
        mary.iri().clone(),
    );
    ontology.add_property_assertion(john_has_mary)?;

    println!(
        "✓ Added {} property assertions",
        ontology.property_assertions().len()
    );

    // Create reasoner and perform reasoning
    println!("\n=== Reasoning Results ===");
    let reasoner = SimpleReasoner::new(ontology);

    // Check consistency
    let is_consistent = reasoner.is_consistent()?;
    println!("✓ Ontology is consistent: {}", is_consistent);

    // Check subclass relationships using IRIs
    let is_parent_subclass_of_person = reasoner.is_subclass_of(parent.iri(), person.iri())?;
    println!("✓ Parent ⊑ Person: {}", is_parent_subclass_of_person);

    // Get instances using IRIs
    let person_instances = reasoner.get_instances(person.iri())?;
    let parent_instances = reasoner.get_instances(parent.iri())?;

    println!("✓ Persons: {:?}", person_instances);
    println!("✓ Parents: {:?}", parent_instances);

    // Performance statistics
    println!("\n=== Performance Statistics ===");
    println!("✓ Total entities: {}", reasoner.ontology.entity_count());
    println!("✓ Total axioms: {}", reasoner.ontology.axiom_count());
    println!("✓ Cache stats: {:?}", reasoner.cache_stats());

    println!("\n=== Example Complete ===");
    println!("✓ Successfully demonstrated basic OWL2 reasoning capabilities");
    println!("✓ All operations completed without errors");

    Ok(())
}
