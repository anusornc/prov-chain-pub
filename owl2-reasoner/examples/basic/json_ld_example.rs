//! JSON-LD parsing example
//!
//! Demonstrates how to parse JSON-LD format ontologies using the OWL2 reasoner.

use owl2_reasoner::parser::{JsonLdParser, OntologyParser};
use owl2_reasoner::reasoning::SimpleReasoner;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🦀 OWL2 Reasoner JSON-LD Example\n");

    // Example JSON-LD content representing a simple ontology
    let json_ld_content = r#"
{
    "@context": {
        "@vocab": "http://example.org/ontology/",
        "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
        "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
        "owl": "http://www.w3.org/2002/07/owl#",
        "xsd": "http://www.w3.org/2001/XMLSchema#"
    },
    "@graph": [
        {
            "@id": "Person",
            "@type": "owl:Class",
            "rdfs:label": "Person",
            "rdfs:comment": "A human being"
        },
        {
            "@id": "Student",
            "@type": "owl:Class",
            "rdfs:label": "Student",
            "rdfs:comment": "A person who is studying"
        },
        {
            "@id": "Person",
            "rdfs:subClassOf": {
                "@id": "Student"
            }
        },
        {
            "@id": "hasName",
            "@type": "owl:DatatypeProperty",
            "rdfs:label": "has name",
            "rdfs:domain": {
                "@id": "Person"
            },
            "rdfs:range": {
                "@id": "xsd:string"
            }
        },
        {
            "@id": "studiesAt",
            "@type": "owl:ObjectProperty",
            "rdfs:label": "studies at",
            "rdfs:domain": {
                "@id": "Student"
            }
        },
        {
            "@id": "alice",
            "@type": ["Person", "Student"],
            "hasName": "Alice Smith",
            "rdfs:label": "Alice"
        },
        {
            "@id": "university1",
            "@type": "http://example.org/ontology/University",
            "rdfs:label": "Example University"
        },
        {
            "@id": "alice",
            "studiesAt": {
                "@id": "university1"
            }
        }
    ]
}
"#;

    println!("📄 Parsing JSON-LD content...");
    println!("───────────────────────────────────────");

    // Create JSON-LD parser
    let parser = JsonLdParser::new();

    // Parse the JSON-LD content
    let ontology = parser.parse_str(json_ld_content)?;
    println!("✅ JSON-LD parsed successfully!");

    // Display parsed entities
    println!("\n📊 Parsed Ontology Summary:");
    println!("  Classes: {}", ontology.classes().len());
    println!(
        "  Object Properties: {}",
        ontology.object_properties().len()
    );
    println!("  Data Properties: {}", ontology.data_properties().len());
    println!(
        "  Named Individuals: {}",
        ontology.named_individuals().len()
    );

    // List parsed classes
    println!("\n🏛️  Classes:");
    for class in ontology.classes() {
        println!("  • {}", class.iri().as_str());
    }

    // List parsed individuals
    println!("\n👤 Named Individuals:");
    for individual in ontology.named_individuals() {
        println!("  • {}", individual.iri().as_str());
    }

    // Perform reasoning
    println!("\n🧠 Performing reasoning...");
    let reasoner = SimpleReasoner::new(ontology);

    // Check consistency
    let is_consistent = reasoner.is_consistent()?;
    println!("✅ Ontology is consistent: {}", is_consistent);

    // Get cache statistics
    let cache_stats = reasoner.get_cache_stats()?;
    println!("📈 Cache statistics: {:?}", cache_stats);

    println!("\n🎉 JSON-LD example completed successfully!");

    Ok(())
}
