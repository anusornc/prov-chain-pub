//! Integration module for testing owl2_rs library with ProvChainOrg

use anyhow::Result;
use owl2_rs::model::{Axiom, Class, ClassExpression, Individual, Ontology};

/// Test the basic functionality of owl2_rs integration
pub fn test_owl2_integration() -> Result<()> {
    println!("Testing owl2_rs integration with ProvChainOrg...");

    // Create a new ontology
    let mut ontology = Ontology::new(Some("http://provchain.org/ontology".to_string()));

    // Create a class
    let person_class = Class::new("http://provchain.org/ontology#Person".to_string());
    let class_expr = ClassExpression::Class(person_class.clone());

    // Create an individual
    let individual = Individual::named("http://provchain.org/ontology#Alice".to_string());

    // Create an axiom
    let axiom = Axiom::ClassAssertion(owl2_rs::model::axiom::ClassAssertionAxiom {
        class: class_expr,
        individual,
        annotations: vec![],
    });

    // Add the axiom to the ontology
    ontology.add_axiom(axiom);

    println!(
        "Successfully created ontology with {} axioms",
        ontology.axiom_count()
    );

    // Test that we have one axiom
    assert_eq!(ontology.axiom_count(), 1);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owl2_integration_function() -> Result<()> {
        test_owl2_integration()
    }
}
