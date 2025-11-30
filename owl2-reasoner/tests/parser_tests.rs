// Basic parser tests
use owl2_reasoner::*;

#[test]
fn test_turtle_parser_creation() {
    let _parser = parser::TurtleParser::new();

    // Test creation succeeded (parser is not a Result type)
    // Parser creation succeeded (no assertion needed as we would have panicked above)
}

#[test]
fn test_rdf_xml_parser_creation() {
    let _parser = parser::RdfXmlParser::new();

    // Test creation succeeded (parser is not a Result type)
    // Parser creation succeeded (no assertion needed as we would have panicked above)
}

#[test]
fn test_owl_xml_parser_creation() {
    let _parser = parser::OwlXmlParser::new();

    // Test creation succeeded (parser is not a Result type)
    // Parser creation succeeded (no assertion needed as we would have panicked above)
}
