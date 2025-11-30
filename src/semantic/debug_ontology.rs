//! Debug module to test ontology loading

use oxigraph::sparql::QueryResults;
use oxigraph::store::Store;
use std::io::Cursor;

pub fn debug_ontology_loading(ontology_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Debug: Loading ontology from {} ===", ontology_path);

    // Check if file exists
    if !std::path::Path::new(ontology_path).exists() {
        println!("ERROR: Ontology file does not exist: {}", ontology_path);
        return Err("Ontology file not found".into());
    }

    // Read file
    let ontology_data = std::fs::read_to_string(ontology_path)?;
    println!(
        "SUCCESS: Ontology file read, length: {}",
        ontology_data.len()
    );

    // Create store and load
    let store = Store::new()?;
    let reader = Cursor::new(ontology_data.as_bytes());

    match store.load_from_reader(oxigraph::io::RdfFormat::Turtle, reader) {
        Ok(_) => println!("SUCCESS: Ontology loaded into store"),
        Err(e) => {
            println!("ERROR: Failed to load ontology: {}", e);
            return Err(e.into());
        }
    }

    // Test hasKey query
    println!("\n=== Testing owl:hasKey query ===");
    let query = r#"
        PREFIX owl: <http://www.w3.org/2002/07/owl#>
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

        SELECT ?class ?keyList WHERE {
            ?class owl:hasKey ?keyList .
        }
    "#;

    match store.query(query) {
        Ok(QueryResults::Solutions(solutions)) => {
            println!("SUCCESS: Query executed");
            let mut count = 0;
            for sol in solutions.flatten() {
                count += 1;
                println!(
                    "Solution {}: class={:?}, keyList={:?}",
                    count,
                    sol.get("class"),
                    sol.get("keyList")
                );
            }
            if count == 0 {
                println!("INFO: No hasKey axioms found");
            }
        }
        Ok(_) => println!("INFO: Query returned no solutions"),
        Err(e) => println!("ERROR: Query failed: {}", e),
    }

    // Test property chain query
    println!("\n=== Testing owl:propertyChainAxiom query ===");
    let query2 = r#"
        PREFIX owl: <http://www.w3.org/2002/07/owl#>
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

        SELECT ?superProperty ?chainList WHERE {
            ?superProperty owl:propertyChainAxiom ?chainList .
        }
    "#;

    match store.query(query2) {
        Ok(QueryResults::Solutions(solutions)) => {
            println!("SUCCESS: Query executed");
            let mut count = 0;
            for sol in solutions.flatten() {
                count += 1;
                println!(
                    "Solution {}: superProperty={:?}, chainList={:?}",
                    count,
                    sol.get("superProperty"),
                    sol.get("chainList")
                );
            }
            if count == 0 {
                println!("INFO: No property chain axioms found");
            }
        }
        Ok(_) => println!("INFO: Query returned no solutions"),
        Err(e) => println!("ERROR: Query failed: {}", e),
    }

    // Test qualified cardinality query
    println!("\n=== Testing owl:qualifiedCardinality query ===");
    let query3 = r#"
        PREFIX owl: <http://www.w3.org/2002/07/owl#>
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?class ?property ?cardinality ?filler WHERE {
            ?restriction a owl:Restriction ;
                owl:onProperty ?property ;
                owl:qualifiedCardinality ?cardinality ;
                owl:onClass ?filler .
            ?class rdfs:subClassOf ?restriction .
        }
    "#;

    match store.query(query3) {
        Ok(QueryResults::Solutions(solutions)) => {
            println!("SUCCESS: Query executed");
            let mut count = 0;
            for sol in solutions.flatten() {
                count += 1;
                println!(
                    "Solution {}: class={:?}, property={:?}, cardinality={:?}, filler={:?}",
                    count,
                    sol.get("class"),
                    sol.get("property"),
                    sol.get("cardinality"),
                    sol.get("filler")
                );
            }
            if count == 0 {
                println!("INFO: No qualified cardinality restrictions found");
            }
        }
        Ok(_) => println!("INFO: Query returned no solutions"),
        Err(e) => println!("ERROR: Query failed: {}", e),
    }

    // Test minQualifiedCardinality too
    println!("\n=== Testing owl:minQualifiedCardinality query ===");
    let query4 = r#"
        PREFIX owl: <http://www.w3.org/2002/07/owl#>
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?class ?property ?cardinality ?filler WHERE {
            ?restriction a owl:Restriction ;
                owl:onProperty ?property ;
                owl:minQualifiedCardinality ?cardinality ;
                owl:onClass ?filler .
            ?class rdfs:subClassOf ?restriction .
        }
    "#;

    match store.query(query4) {
        Ok(QueryResults::Solutions(solutions)) => {
            println!("SUCCESS: Query executed");
            let mut count = 0;
            for sol in solutions.flatten() {
                count += 1;
                println!(
                    "Solution {}: class={:?}, property={:?}, minCardinality={:?}, filler={:?}",
                    count,
                    sol.get("class"),
                    sol.get("property"),
                    sol.get("cardinality"),
                    sol.get("filler")
                );
            }
            if count == 0 {
                println!("INFO: No minQualified cardinality restrictions found");
            }
        }
        Ok(_) => println!("INFO: Query returned no solutions"),
        Err(e) => println!("ERROR: Query failed: {}", e),
    }

    println!("\n=== Debug completed ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_ontology_loading() -> Result<(), Box<dyn std::error::Error>> {
        debug_ontology_loading("ontologies/test-owl2.owl")
    }
}
