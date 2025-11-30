//! Simple OWL2 integration test

use anyhow::Result;
use owl2_reasoner::{Class, ClassAssertionAxiom, ClassExpression, Ontology, IRI};
use std::sync::Arc;

/// Simple integration test
pub fn simple_owl2_integration_test() -> Result<()> {
    println!("=== Simple OWL2 Integration Test ===");

    // Create a new ontology with IRI
    let mut ontology = Ontology::with_iri("http://provchain.org/test");

    // Create a class
    let product_class = Class::new("http://provchain.org/test#Product");
    let class_expr = ClassExpression::Class(product_class);

    // Create an individual IRI (returns Result, so we need to handle it)
    let product_iri = Arc::new(IRI::new("http://provchain.org/test#Product001")?);

    // Create a class assertion axiom
    let class_assertion = ClassAssertionAxiom::new(product_iri, class_expr);

    // Add the axiom to the ontology
    ontology.add_class_assertion(class_assertion)?;

    println!(
        "Successfully created ontology with {} axioms",
        ontology.axiom_count()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_owl2_integration() -> Result<()> {
        simple_owl2_integration_test()
    }
}
