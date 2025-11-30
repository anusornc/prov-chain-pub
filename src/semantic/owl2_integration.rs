//! Integration module for testing owl2-reasoner library with ProvChainOrg

use anyhow::Result;
use owl2_reasoner::{Class, ClassAssertionAxiom, ClassExpression, Ontology, IRI};
use std::sync::Arc;

/// Test the basic functionality of owl2-reasoner integration
pub fn test_owl2_integration() -> Result<()> {
    println!("Testing owl2-reasoner integration with ProvChainOrg...");

    // Create a new ontology with IRI
    let mut ontology = Ontology::with_iri("http://provchain.org/ontology");

    // Create a class
    let person_class = Class::new("http://provchain.org/ontology#Person");
    let class_expr = ClassExpression::Class(person_class.clone());

    // Create an individual IRI (returns Result, so we need to handle it)
    let individual_iri = Arc::new(IRI::new("http://provchain.org/ontology#Alice")?);

    // Create a class assertion axiom
    let class_assertion = ClassAssertionAxiom::new(individual_iri, class_expr);

    // Add the axiom to the ontology
    ontology.add_class_assertion(class_assertion)?;

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
