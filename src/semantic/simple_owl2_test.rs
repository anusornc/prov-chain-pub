//! Simple OWL2 integration test

use anyhow::Result;
use owl2_rs::model::{Ontology, Class, ClassExpression, Individual, Axiom};

/// Simple integration test
pub fn simple_owl2_integration_test() -> Result<()> {
    println!("=== Simple OWL2 Integration Test ===");
    
    // Create a new ontology
    let mut ontology = Ontology::new(Some("http://provchain.org/test".to_string()));
    
    // Create a class
    let product_class = Class::new("http://provchain.org/test#Product".to_string());
    let class_expr = ClassExpression::Class(product_class);
    
    // Create an individual
    let product_instance = Individual::named("http://provchain.org/test#Product001".to_string());
    
    // Create an axiom
    let axiom = Axiom::ClassAssertion(owl2_rs::model::axiom::ClassAssertionAxiom {
        class: class_expr,
        individual: product_instance,
        annotations: vec![],
    });
    
    // Add the axiom to the ontology
    ontology.add_axiom(axiom);
    
    println!("Successfully created ontology with {} axioms", ontology.axiom_count());
    
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