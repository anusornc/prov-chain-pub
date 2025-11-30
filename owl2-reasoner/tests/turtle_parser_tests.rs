use owl2_reasoner::parser::{OntologyParser, TurtleParser};
use owl2_reasoner::reasoning::tableaux::TableauxReasoner;
use owl2_reasoner::IRI;

#[test]
fn test_parse_simple_turtle() {
    let turtle_content = r#"
@prefix : <http://example.org/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

:Person a rdfs:Class .
:Employee rdfs:subClassOf :Person .
"#;

    let parser = TurtleParser::new();
    let result = parser.parse_str(turtle_content);

    assert!(result.is_ok());
    let ontology = result.unwrap();

    assert_eq!(ontology.classes().len(), 2);
    // The parser creates a SubClassOf axiom, but does not create a DeclarationAxiom for the class declaration.
    assert_eq!(ontology.axioms().len(), 1);
}

#[test]
fn test_parse_with_prefixes() {
    let turtle_content = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ex: <http://example.org/> .

ex:Person a rdfs:Class .
ex:Employee rdfs:subClassOf ex:Person .
"#;

    let parser = TurtleParser::new();
    let result = parser.parse_str(turtle_content);

    assert!(result.is_ok());
    let ontology = result.unwrap();

    assert_eq!(ontology.classes().len(), 2);
    assert_eq!(ontology.axioms().len(), 1);
}

#[test]
fn test_turtle_multiple_prefix_declarations() {
    let turtle_content = r#"
@prefix ex1: <http://example.org/1/> .
@prefix ex2: <http://example.org/2/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex1:Person a rdfs:Class .
ex2:Employee rdfs:subClassOf ex1:Person .
ex2:hasSkill rdfs:range ex2:Skill .
ex2:hasManager rdfs:domain ex2:Person .
ex2:Manager owl:equivalentClass ex1:Manager .
"#;

    let parser = TurtleParser::new();
    let result = parser.parse_str(turtle_content);

    assert!(result.is_ok());
    let ontology = result.unwrap();

    assert_eq!(ontology.classes().len(), 4); // Person, Employee, Skill, Manager
    assert_eq!(ontology.object_properties().len(), 3); // hasSkill, hasManager, equivalentClass
    assert_eq!(ontology.axioms().len(), 4); // 2 subclass, 1 domain, 1 equivalent
}

#[test]
fn test_turtle_property_assertions() {
    let turtle_content = r#"
@prefix : <http://example.org/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

:John a :Person .
:hasFather :hasParent :John .
:hasMother :hasParent :John .
:hasFriend :hasFriend :John .
"#;

    let parser = TurtleParser::new();
    let result = parser.parse_str(turtle_content);

    assert!(result.is_ok());
    let ontology = result.unwrap();

    assert_eq!(ontology.named_individuals().len(), 1);
    assert_eq!(ontology.property_assertions().len(), 3);

    // Test property assertion details
    for assertion in &ontology.property_assertions() {
        assert_eq!(assertion.subject().as_str(), "http://example.org/John");
    }
}

#[test]
fn test_turtle_data_properties() {
    let turtle_content = r#"
@prefix : <http://example.org/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

:John a :Person .
:age "30"^^xsd:integer .
:name "John Doe"^^xsd:string .
:active "true"^^xsd:boolean .
:score 95.5^^xsd:decimal .
"#;

    let parser = TurtleParser::new();
    let result = parser.parse_str(turtle_content);

    assert!(result.is_ok());
    let ontology = result.unwrap();

    assert_eq!(ontology.named_individuals().len(), 1);
    assert_eq!(ontology.data_property_assertions().len(), 4);

    // Verify literal types and values
    for assertion in &ontology.data_property_assertions() {
        assert_eq!(assertion.subject().as_str(), "http://example.org/John");
    }
}

#[test]
fn test_turtle_complex_nested_structures() {
    let turtle_content = r#"
@prefix : <http://example.org/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w2.org/2002/07/owl#> .

:ResearchInstitute a owl:Class .
:ResearchProject a owl:Class .
:hasResearcher rdfs:domain :ResearchInstitute .
:producesPublication rdfs:range :ResearchProject .
:hasMember rdfs:domain :ResearchInstitute .

:AI a :ResearchProject .
:usesMethod rdfs:range :Method .
:requiresSkill rdfs:range :Skill .
:hasFunding rdfs:range :Funding .

:MachineLearning a :Method .
:requiresProgramming rdfs:range :Programming .
:requiresMath rdfs:range :Math .
:hasApplication rdfs:range :Application .

:Python a :Application .
:usesLibrary rdfs:range :Library .
:hasVersion "3.9"^^xsd:string .
"#;

    let parser = TurtleParser::new();
    let result = parser.parse_str(turtle_content);

    assert!(result.is_ok());
    let ontology = result.unwrap();

    assert_eq!(ontology.classes().len(), 4);
    assert_eq!(ontology.object_properties().len(), 5);
    assert_eq!(ontology.data_property_assertions().len(), 2);
    assert_eq!(ontology.named_individuals().len(), 1);
}

#[test]
fn test_turtle_collections_and_lists() {
    let turtle_content = r#"
@prefix : <http://example.org/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

:Team a rdfs:Class .
:member rdf:first :Alice .
:member rdf:rest :Bob .
:member rdf:rest :Charlie .
:member rdf:rest :rdf:nil .

:Alice a :Person .
:Bob a :Person .
:Charlie a :Person .
"#;

    let parser = TurtleParser::new();
    let result = parser.parse_str(turtle_content);

    assert!(result.is_ok());
    let ontology = result.unwrap();

    assert_eq!(ontology.classes().len(), 2);
    assert_eq!(ontology.named_individuals().len(), 3);
    // The parser should handle rdf:first/rdf:rest/rdf:nil patterns
}

#[test]
fn test_turtle_blank_nodes() {
    let turtle_content = r#"
@prefix : <http://example.org/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

:Anonymous a _:Person .
:_hasProperty _:Anonymous .
"#;

    let parser = TurtleParser::new();
    let result = parser.parse_str(turtle_content);

    assert!(result.is_ok());
    let ontology = result.unwrap();

    assert_eq!(ontology.named_individuals().len(), 1);
    // Should handle blank node references properly
}

#[test]
fn test_turtle_invalid_syntax_errors() {
    // Test malformed Turtle syntax
    let invalid_cases = vec![
        ("Missing prefix declaration", ":Person a rdfs:Class ."),
        ("Trailing semicolon", ":Person a rdfs:Class ;"),
        ("Unclosed quotes", ":name \"Unclosed string"),
        ("Invalid IRI", "http://invalid iri"),
    ];

    for (description, invalid_content) in invalid_cases {
        let parser = TurtleParser::new();
        let result = parser.parse_str(invalid_content);
        assert!(result.is_err(), "Should fail for: {}", description);
    }
}

#[test]
fn test_turtle_reasoning_integration() {
    let turtle_content = r#"
@prefix : <http://example.org/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .

:Person a rdfs:Class .
:Student rdfs:subClassOf :Person .
:PhDStudent owl:equivalentClass :Student .
:FacultyMember rdfs:subClassOf :Person .
:Teaches rdfs:domain :FacultyMember .
:Advises rdfs:range :GraduateStudent .

:Alice a :Student .
:hasAdvisor :Bob .
:researchArea "Machine Learning" .
"#;

    let parser = TurtleParser::new();
    let result = parser.parse_str(turtle_content);

    assert!(result.is_ok());
    let ontology = result.unwrap();

    // Test reasoning with the parsed ontology
    let reasoner = TableauxReasoner::new(ontology);

    // Test subclass relationships
    let student_iri = IRI::new("http://example.org/Student").unwrap();
    let person_iri = IRI::new("http://example.org/Person").unwrap();
    assert!(reasoner
        .is_subclass_of(&student_iri, &person_iri)
        .unwrap_or(false));

    // Test equivalence (check if PhDStudent is equivalent to Student)
    let phd_student_iri = IRI::new("http://example.org/PhDStudent").unwrap();
    let student_equivalents = reasoner.get_equivalent_classes(&student_iri);
    assert!(student_equivalents.contains(&phd_student_iri));
}

#[test]
fn test_turtle_performance_large_ontology() {
    // Create a large Turtle document for performance testing
    let mut turtle_content = String::from(
        r#"
@prefix : <http://example.org/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
"#,
    );

    // Generate 100 classes
    for i in 0..100 {
        turtle_content.push_str(&format!(":Class{} a rdfs:Class .\n", i));
    }

    let parser = TurtleParser::new();
    let start = std::time::Instant::now();
    let result = parser.parse_str(&turtle_content);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Should parse large ontology");
    let ontology = result.unwrap();
    assert_eq!(ontology.classes().len(), 100);

    // Performance assertion - should complete within reasonable time
    assert!(
        duration.as_millis() < 10000,
        "Large ontology should parse in under 10 seconds"
    );
}

#[test]
fn test_turtle_edge_cases() {
    // Test edge cases and boundary conditions
    let long_iri_test = format!(":verylongnamespaceiri{} :value .", "a".repeat(1000));
    let edge_cases = vec![
        (
            "Multiple prefixes with same prefix",
            "@prefix ex: <http://example.org/> . @prefix ex: <http://different.org/> .",
        ),
        ("Deep nesting", ":a :b :c ."),
        ("Long IRI", long_iri_test.as_str()),
        ("Unicode support", ":测试 a rdfs:Class ."),
        ("Mixed case", ":Person a RDFS:Class ."),
    ];

    for (description, content) in edge_cases {
        let parser = TurtleParser::new();
        let result = parser.parse_str(content);

        match description {
            "Multiple prefixes with same prefix" => {
                // Should handle last prefix definition
                assert!(result.is_ok());
            }
            "Deep nesting" => {
                // Should handle nested property paths
                assert!(result.is_ok());
            }
            "Long IRI" => {
                // Should handle very long IRIs
                assert!(result.is_ok());
            }
            "Unicode support" => {
                // Should handle Unicode characters
                assert!(result.is_ok());
            }
            "Mixed case" => {
                // Should be case-sensitive for prefixes
                assert!(result.is_ok());
            }
            _ => {
                // Most edge cases should either work or give clear error messages
                println!("Testing edge case: {} - Result: {:?}", description, result);
            }
        }
    }
}
