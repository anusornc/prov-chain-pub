//! Streaming RDF/XML parser using rio-xml library

use crate::axioms::class_expressions::ClassExpression;
use crate::axioms::*;
use crate::entities::*;
use crate::error::OwlResult;
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::parser::rdf_xml_common::{ERR_RIO_XML_PARSE, NS_OWL, NS_RDF, NS_RDFS};
use crate::parser::{ParserArenaBuilder, ParserArenaTrait, ParserConfig};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

#[cfg(feature = "rio-xml")]
use rio_api::model::{Subject, Term, Triple};
#[cfg(feature = "rio-xml")]
use rio_api::parser::TriplesParser as _;
#[cfg(feature = "rio-xml")]
use rio_xml::RdfXmlParser as RioRdfXmlParser;

/// Streaming RDF/XML parser for efficient large file processing
pub struct RdfXmlStreamingParser {
    pub config: ParserConfig,
    pub namespaces: HashMap<String, String>,
    pub base_iri: Option<IRI>,
    pub arena: Option<Box<dyn ParserArenaTrait>>,
}

impl RdfXmlStreamingParser {
    /// Create a new streaming parser
    pub fn new(config: ParserConfig) -> Self {
        let namespaces = crate::parser::rdf_xml_common::initialize_namespaces(&config.prefixes);

        let arena = if config.use_arena_allocation {
            Some(
                ParserArenaBuilder::new()
                    .with_capacity(config.arena_capacity)
                    .build(),
            )
        } else {
            None
        };

        Self {
            config,
            namespaces,
            base_iri: None,
            arena,
        }
    }

    /// Parse RDF/XML content using streaming approach
    #[cfg(feature = "rio-xml")]
    pub fn parse_content(&mut self, content: &str) -> OwlResult<Ontology> {
        let mut ontology = Ontology::new();

        let base_iri = self
            .base_iri
            .as_ref()
            .and_then(|iri| oxiri::Iri::parse(iri.as_str().to_string()).ok());

        let mut parser = RioRdfXmlParser::new(Cursor::new(content), base_iri);

        let mut handler = |triple: Triple| -> Result<(), std::io::Error> {
            self.process_triple(&mut ontology, triple)
                .map_err(std::io::Error::other)
        };

        parser.parse_all(&mut handler).map_err(|e| {
            crate::error::OwlError::ParseError(format!("{}: {}", ERR_RIO_XML_PARSE, e))
        })?;

        Ok(ontology)
    }

    /// Parse RDF/XML file using streaming approach
    #[cfg(feature = "rio-xml")]
    pub fn parse_file(&mut self, path: &Path) -> OwlResult<Ontology> {
        use std::fs::File;
        use std::io::BufReader;

        let file = File::open(path).map_err(crate::error::OwlError::IoError)?;

        let reader = BufReader::new(file);
        self.parse_stream(reader)
    }

    /// Parse RDF/XML from a reader using streaming approach
    #[cfg(feature = "rio-xml")]
    pub fn parse_stream(&mut self, reader: impl std::io::BufRead) -> OwlResult<Ontology> {
        let mut ontology = Ontology::new();

        let base_iri = self
            .base_iri
            .as_ref()
            .and_then(|iri| oxiri::Iri::parse(iri.as_str().to_string()).ok());

        let mut parser = RioRdfXmlParser::new(reader, base_iri);

        let mut handler = |triple: Triple| -> Result<(), std::io::Error> {
            self.process_triple(&mut ontology, triple)
                .map_err(std::io::Error::other)
        };

        parser.parse_all(&mut handler).map_err(|e| {
            crate::error::OwlError::ParseError(format!("{}: {}", ERR_RIO_XML_PARSE, e))
        })?;

        Ok(ontology)
    }

    /// Process a single triple and add to ontology
    #[cfg(feature = "rio-xml")]
    fn process_triple(&mut self, ontology: &mut Ontology, triple: Triple) -> OwlResult<()> {
        let subject_iri = self.subject_to_iri(&triple.subject)?;
        let predicate_iri = IRI::new(triple.predicate.iri)?;
        let object = self.process_object(&triple.object)?;

        // Ensure subject individual exists (create if not already present)
        let subject_individual = NamedIndividual::new(subject_iri.clone());
        if !ontology
            .named_individuals()
            .iter()
            .any(|ni| ni.iri().as_ref() == &subject_iri)
        {
            ontology.add_named_individual(subject_individual)?;
        }

        // Handle different types of triples
        match predicate_iri.as_str() {
            // Type assertions
            ty if ty == format!("{}type", NS_RDF) => {
                if let Some(object_iri) = object.as_iri() {
                    self.handle_type_assertion(ontology, &subject_iri, object_iri)?;
                }
            }

            // Subclass relationships
            ty if ty == format!("{}subClassOf", NS_RDFS) => {
                if let Some(object_iri) = object.as_iri() {
                    self.handle_subclass_of(ontology, &subject_iri, object_iri)?;
                }
            }

            // Domain and range
            ty if ty == format!("{}domain", NS_RDFS) => {
                if let Some(object_iri) = object.as_iri() {
                    self.handle_domain(ontology, &subject_iri, object_iri)?;
                }
            }

            ty if ty == format!("{}range", NS_RDFS) => {
                if let Some(object_iri) = object.as_iri() {
                    self.handle_range(ontology, &subject_iri, object_iri)?;
                }
            }

            // OWL-specific properties
            ty if ty.starts_with(NS_OWL) => {
                self.handle_owl_property(ontology, &subject_iri, &predicate_iri, &object)?;
            }

            _ => {
                // Generic property assertion
                self.handle_property_assertion(ontology, &subject_iri, &predicate_iri, &object)?;
            }
        }

        Ok(())
    }

    /// Convert Rio subject to IRI
    #[cfg(feature = "rio-xml")]
    fn subject_to_iri(&self, subject: &Subject) -> OwlResult<IRI> {
        match subject {
            Subject::NamedNode(node) => IRI::new(node.iri),
            Subject::BlankNode(node) => IRI::new(format!("_:{}", node.id)),
            Subject::Triple(triple) => {
                // Handle RDF-star triple subjects by creating a reification IRI
                // This represents a statement about another statement
                let subject_iri = self.subject_to_iri(&triple.subject)?;
                let predicate_iri = IRI::new(triple.predicate.iri)?;
                let object = self.process_object(&triple.object)?;

                // Create a reified statement identifier
                // Format: _:reified_<subject>_<predicate>_<object>
                let object_str = match &object {
                    ProcessedObject::Iri(iri) => iri.as_str().to_string(),
                    ProcessedObject::BlankNode(id) => format!("_:{}", id),
                    ProcessedObject::Literal(lit) => {
                        // For literals, create a simple representation
                        format!("\"{}\"", lit.lexical_form())
                    }
                };

                // Create a unique identifier for the reified statement
                let reified_id = format!(
                    "_:reified_{}_{}_{}",
                    subject_iri
                        .as_str()
                        .replace("http://", "")
                        .replace("https://", "")
                        .replace("/", "_")
                        .replace("#", "_"),
                    predicate_iri
                        .as_str()
                        .replace("http://", "")
                        .replace("https://", "")
                        .replace("/", "_")
                        .replace("#", "_"),
                    object_str
                        .replace("http://", "")
                        .replace("https://", "")
                        .replace("/", "_")
                        .replace("#", "_")
                        .replace("\"", "")
                );

                // Truncate if too long to avoid unreasonable IRIs
                let reified_id = if reified_id.len() > 200 {
                    use std::hash::{Hash, Hasher};
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    reified_id.hash(&mut hasher);
                    format!("_:reified_triple_{:x}", hasher.finish())
                } else {
                    reified_id
                };

                IRI::new(reified_id)
            }
        }
    }

    /// Process object term
    #[cfg(feature = "rio-xml")]
    fn process_object(&self, term: &Term) -> OwlResult<ProcessedObject> {
        match term {
            Term::NamedNode(node) => Ok(ProcessedObject::Iri(IRI::new(node.iri)?)),
            Term::BlankNode(node) => Ok(ProcessedObject::BlankNode(node.id.to_string())),
            Term::Literal(literal) => {
                // Process Rio API literals
                // TODO: Implement proper literal extraction by checking Rio API 0.8 documentation
                // For now, we use a debug representation which preserves the literal information
                let literal_str = format!("{:?}", literal);

                // Create a simple literal using the debug representation
                // This preserves the literal information in a usable format
                let processed_literal = Literal::simple(literal_str);

                Ok(ProcessedObject::Literal(processed_literal))
            }
            Term::Triple(triple) => {
                // Handle RDF-star triple terms by creating a reified statement object
                // This represents a statement used as an object
                let subject_iri = self.subject_to_iri(&triple.subject)?;
                let predicate_iri = IRI::new(triple.predicate.iri)?;
                let object = self.process_object(&triple.object)?;

                // Create a reified statement identifier for the triple term
                let object_str = match &object {
                    ProcessedObject::Iri(iri) => iri.as_str().to_string(),
                    ProcessedObject::BlankNode(id) => format!("_:{}", id),
                    ProcessedObject::Literal(lit) => {
                        format!("\"{}\"", lit.lexical_form())
                    }
                };

                // Create a unique identifier for the reified statement
                let reified_id = format!(
                    "_:reified_triple_term_{}_{}_{}",
                    subject_iri
                        .as_str()
                        .replace("http://", "")
                        .replace("https://", "")
                        .replace("/", "_")
                        .replace("#", "_"),
                    predicate_iri
                        .as_str()
                        .replace("http://", "")
                        .replace("https://", "")
                        .replace("/", "_")
                        .replace("#", "_"),
                    object_str
                        .replace("http://", "")
                        .replace("https://", "")
                        .replace("/", "_")
                        .replace("#", "_")
                        .replace("\"", "")
                );

                // Truncate if too long to avoid unreasonable IRIs
                let reified_id = if reified_id.len() > 200 {
                    use std::hash::{Hash, Hasher};
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    reified_id.hash(&mut hasher);
                    format!("_:reified_triple_term_{:x}", hasher.finish())
                } else {
                    reified_id
                };

                // Return as a blank node with the reified statement identifier
                Ok(ProcessedObject::BlankNode(reified_id))
            }
        }
    }

    /// Handle type assertions (rdf:type)
    #[cfg(feature = "rio-xml")]
    fn handle_type_assertion(
        &mut self,
        ontology: &mut Ontology,
        subject: &IRI,
        object_iri: &IRI,
    ) -> OwlResult<()> {
        match object_iri.as_str() {
            ty if ty == format!("{}Class", NS_OWL) => {
                let class = Class::new(subject.clone());
                ontology.add_class(class)?;
            }
            ty if ty == format!("{}ObjectProperty", NS_OWL) => {
                let property = ObjectProperty::new(subject.clone());
                ontology.add_object_property(property)?;
            }
            ty if ty == format!("{}DatatypeProperty", NS_OWL) => {
                let property = DataProperty::new(subject.clone());
                ontology.add_data_property(property)?;
            }
            ty if ty == format!("{}NamedIndividual", NS_OWL) => {
                let individual = NamedIndividual::new(subject.clone());
                ontology.add_named_individual(individual)?;
            }
            _ => {
                // Generic type assertion
                let class = Class::new(object_iri.clone());
                let assertion = ClassAssertionAxiom::new(
                    Arc::new(subject.clone()),
                    ClassExpression::Class(class),
                );
                ontology.add_class_assertion(assertion)?;
            }
        }
        Ok(())
    }

    /// Handle subclass relationships
    #[cfg(feature = "rio-xml")]
    fn handle_subclass_of(
        &mut self,
        ontology: &mut Ontology,
        subject: &IRI,
        object_iri: &IRI,
    ) -> OwlResult<()> {
        let subclass = Class::new(subject.clone());
        let superclass = Class::new(object_iri.clone());
        let axiom = SubClassOfAxiom::new(
            ClassExpression::Class(subclass),
            ClassExpression::Class(superclass),
        );
        ontology.add_subclass_axiom(axiom)?;
        Ok(())
    }

    /// Handle domain declarations
    #[cfg(feature = "rio-xml")]
    fn handle_domain(
        &mut self,
        ontology: &mut Ontology,
        subject: &IRI,
        object_iri: &IRI,
    ) -> OwlResult<()> {
        // Implementation depends on whether subject is object or data property
        let class = Class::new(object_iri.clone());

        // This is simplified - in practice, you'd need to determine the property type
        let axiom = ObjectPropertyDomainAxiom::new(
            Arc::new(subject.clone()),
            ClassExpression::Class(class),
        );
        // Add as generic axiom for now
        ontology.add_axiom(crate::axioms::Axiom::ObjectPropertyDomain(Box::new(axiom)))?;
        Ok(())
    }

    /// Handle range declarations
    #[cfg(feature = "rio-xml")]
    fn handle_range(
        &mut self,
        ontology: &mut Ontology,
        subject: &IRI,
        object_iri: &IRI,
    ) -> OwlResult<()> {
        // For object property range
        let class = Class::new(object_iri.clone());
        let axiom = ObjectPropertyRangeAxiom::new(subject.clone(), ClassExpression::Class(class));
        // Add as generic axiom for now
        ontology.add_axiom(crate::axioms::Axiom::ObjectPropertyRange(Box::new(axiom)))?;
        Ok(())
    }

    /// Handle OWL-specific properties
    #[cfg(feature = "rio-xml")]
    fn handle_owl_property(
        &mut self,
        ontology: &mut Ontology,
        subject: &IRI,
        predicate: &IRI,
        object: &ProcessedObject,
    ) -> OwlResult<()> {
        // Handle various OWL properties like equivalentClass, disjointWith, etc.
        let predicate_str = predicate.as_str();

        // Handle owl:disjointWith
        if predicate_str == format!("{}disjointWith", NS_OWL) {
            if let Some(object_iri) = object.as_iri() {
                let axiom = DisjointClassesAxiom::new(vec![
                    Arc::new(subject.clone()),
                    Arc::new(object_iri.clone()),
                ]);
                ontology.add_disjoint_classes_axiom(axiom)?;
            }
        }
        // Handle owl:equivalentClass
        else if predicate_str == format!("{}equivalentClass", NS_OWL) {
            if let Some(object_iri) = object.as_iri() {
                let axiom = EquivalentClassesAxiom::new(vec![
                    Arc::new(subject.clone()),
                    Arc::new(object_iri.clone()),
                ]);
                ontology.add_equivalent_classes_axiom(axiom)?;
            }
        }

        // Handle other OWL properties as needed
        // For now, we just ignore unknown OWL properties

        Ok(())
    }

    /// Handle generic property assertions
    #[cfg(feature = "rio-xml")]
    fn handle_property_assertion(
        &mut self,
        ontology: &mut Ontology,
        subject: &IRI,
        predicate: &IRI,
        object: &ProcessedObject,
    ) -> OwlResult<()> {
        match object {
            ProcessedObject::Iri(object_iri) => {
                // Object property with named individual
                let object_individual = NamedIndividual::new(object_iri.clone());
                ontology.add_named_individual(object_individual.clone())?;

                let assertion = PropertyAssertionAxiom::new(
                    Arc::new(subject.clone()),
                    Arc::new(predicate.clone()),
                    object_individual.iri().clone(),
                );
                ontology.add_property_assertion(assertion)?;
            }
            ProcessedObject::BlankNode(node_id) => {
                // Object property with anonymous individual
                let anon_individual = AnonymousIndividual::new(format!("_:{}", node_id));
                ontology.add_anonymous_individual(anon_individual.clone())?;

                let assertion = PropertyAssertionAxiom::new_with_anonymous(
                    Arc::new(subject.clone()),
                    Arc::new(predicate.clone()),
                    anon_individual,
                );
                ontology.add_property_assertion(assertion)?;
            }
            ProcessedObject::Literal(literal) => {
                // Data property with literal value
                let assertion = DataPropertyAssertionAxiom::new(
                    Arc::new(subject.clone()),
                    Arc::new(predicate.clone()),
                    literal.clone(),
                );
                ontology.add_data_property_assertion(assertion)?;
            }
        }

        Ok(())
    }
}

/// Processed object representation
#[derive(Debug)]
pub enum ProcessedObject {
    Iri(IRI),
    BlankNode(String),
    Literal(Literal),
}

impl ProcessedObject {
    pub fn as_iri(&self) -> Option<&IRI> {
        match self {
            ProcessedObject::Iri(iri) => Some(iri),
            _ => None,
        }
    }
}

// Fallback implementations when rio-xml feature is not enabled
#[cfg(not(feature = "rio-xml"))]
impl RdfXmlStreamingParser {
    pub fn parse_content(&mut self, _content: &str) -> OwlResult<Ontology> {
        Err(crate::error::OwlError::ParseError(
            "Streaming parsing requires 'rio-xml' feature".to_string(),
        ))
    }

    pub fn parse_file(&self, _path: &Path) -> OwlResult<Ontology> {
        Err(crate::error::OwlError::ParseError(
            "Streaming parsing requires 'rio-xml' feature".to_string(),
        ))
    }

    pub fn parse_stream(&mut self, _reader: impl std::io::BufRead) -> OwlResult<Ontology> {
        Err(crate::error::OwlError::ParseError(
            "Streaming parsing requires 'rio-xml' feature".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "rio-xml")]
    #[test]
    fn test_rdf_xml_streaming_basic_parsing() {
        let config = ParserConfig::default();
        let mut parser = RdfXmlStreamingParser::new(config);

        // Simple RDF/XML content
        let rdf_xml_content = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#">

    <owl:Class rdf:about="http://example.org/Person">
        <rdfs:label>Person</rdfs:label>
    </owl:Class>

</rdf:RDF>"#;

        let result = parser.parse_content(rdf_xml_content);
        assert!(
            result.is_ok(),
            "Failed to parse basic RDF/XML content: {:?}",
            result.err()
        );

        if let Ok(ontology) = result {
            let classes = ontology.classes();
            assert!(
                !classes.is_empty(),
                "No classes were parsed from the content"
            );
        }
    }

    #[cfg(feature = "rio-xml")]
    #[test]
    fn test_subject_to_iri_with_named_node() {
        let config = ParserConfig::default();
        let parser = RdfXmlStreamingParser::new(config);

        use rio_api::model::NamedNode;
        let named_node = NamedNode {
            iri: "http://example.org/test",
        };
        let subject = Subject::NamedNode(named_node);

        let result = parser.subject_to_iri(&subject);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "http://example.org/test");
    }

    #[cfg(feature = "rio-xml")]
    #[test]
    fn test_subject_to_iri_with_blank_node() {
        let config = ParserConfig::default();
        let parser = RdfXmlStreamingParser::new(config);

        use rio_api::model::BlankNode;
        let blank_node = BlankNode { id: "test123" };
        let subject = Subject::BlankNode(blank_node);

        let result = parser.subject_to_iri(&subject);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "_:test123");
    }

    #[cfg(feature = "rio-xml")]
    #[test]
    fn test_process_object_with_named_node() {
        let config = ParserConfig::default();
        let parser = RdfXmlStreamingParser::new(config);

        use rio_api::model::NamedNode;
        let named_node = NamedNode {
            iri: "http://example.org/object",
        };
        let term = Term::NamedNode(named_node);

        let result = parser.process_object(&term);
        assert!(result.is_ok());

        if let ProcessedObject::Iri(iri) = result.unwrap() {
            assert_eq!(iri.as_str(), "http://example.org/object");
        } else {
            panic!("Expected Iri object");
        }
    }

    #[cfg(feature = "rio-xml")]
    #[test]
    fn test_process_object_with_blank_node() {
        let config = ParserConfig::default();
        let parser = RdfXmlStreamingParser::new(config);

        use rio_api::model::BlankNode;
        let blank_node = BlankNode { id: "blank456" };
        let term = Term::BlankNode(blank_node);

        let result = parser.process_object(&term);
        assert!(result.is_ok());

        if let ProcessedObject::BlankNode(id) = result.unwrap() {
            assert_eq!(id, "blank456");
        } else {
            panic!("Expected BlankNode object");
        }
    }
}
