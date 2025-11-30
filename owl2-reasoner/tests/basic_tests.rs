// Basic tests for OWL2 Reasoner
// Clean, minimal test suite to avoid memory issues

use owl2_reasoner::*;

#[test]
fn test_iri_creation() {
    let iri = IRI::new("http://example.org/test").unwrap();
    assert_eq!(iri.as_str(), "http://example.org/test");
}

#[test]
fn test_ontology_creation() {
    let mut ontology = Ontology::new();
    let iri = IRI::new("http://example.org/ontology").unwrap();
    ontology.set_iri(iri);

    assert_eq!(
        ontology.iri().unwrap().as_str(),
        "http://example.org/ontology"
    );
}

#[test]
fn test_class_creation() {
    let iri = IRI::new("http://example.org/Person").unwrap();
    let class = Class::new(iri);

    assert_eq!(class.iri().as_str(), "http://example.org/Person");
}

#[test]
fn test_simple_reasoner_creation() {
    let ontology = Ontology::new();
    let reasoner = SimpleReasoner::new(ontology);

    // Test creation succeeded
    assert_eq!(reasoner.ontology.iri(), None);
}
