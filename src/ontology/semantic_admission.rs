//! Package-selected, fail-closed semantic admission for asserted public state.
//!
//! This module is deliberately a deep boundary: package activation compiles the
//! bounded SHACL subset and initializes SPACL once, while Final Admission asks
//! it one question about a complete staged public-state transition.

use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::io::Cursor;
use std::sync::Arc;

use chrono::{DateTime, NaiveDate, NaiveDateTime};
use owl2_reasoner::{Class, ClassExpression, ParserFactory, SimpleReasoner, SubClassOfAxiom, IRI};
use oxigraph::io::RdfFormat;
use oxigraph::model::{Literal, NamedNode, Quad, Subject, Term, Triple};
use oxigraph::store::Store;
use oxsdatatypes::{
    Boolean as XsdBoolean, DateTime as XsdDateTime, Double as XsdDouble, Float as XsdFloat,
};
use regexml::Regex;
use thiserror::Error;

use crate::ontology::package::OntologyPackageManifest;
#[cfg(test)]
use crate::ontology::package::SEMANTIC_EXECUTION_PROFILE_V1;

const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDF_FIRST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first";
const RDF_REST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest";
const RDF_NIL: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#nil";
const RDFS_SUBCLASS_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
const SH: &str = "http://www.w3.org/ns/shacl#";
const SH_NODE_SHAPE: &str = "http://www.w3.org/ns/shacl#NodeShape";
const SH_PROPERTY_SHAPE: &str = "http://www.w3.org/ns/shacl#PropertyShape";
const SH_TARGET_CLASS: &str = "http://www.w3.org/ns/shacl#targetClass";
const SH_PROPERTY: &str = "http://www.w3.org/ns/shacl#property";
const SH_PATH: &str = "http://www.w3.org/ns/shacl#path";
const SH_MIN_COUNT: &str = "http://www.w3.org/ns/shacl#minCount";
const SH_MAX_COUNT: &str = "http://www.w3.org/ns/shacl#maxCount";
const SH_DATATYPE: &str = "http://www.w3.org/ns/shacl#datatype";
const SH_CLASS: &str = "http://www.w3.org/ns/shacl#class";
const SH_IN: &str = "http://www.w3.org/ns/shacl#in";
const SH_PATTERN: &str = "http://www.w3.org/ns/shacl#pattern";
const SH_MIN_INCLUSIVE: &str = "http://www.w3.org/ns/shacl#minInclusive";
const SH_MAX_INCLUSIVE: &str = "http://www.w3.org/ns/shacl#maxInclusive";
const SH_HAS_VALUE: &str = "http://www.w3.org/ns/shacl#hasValue";
const SH_MESSAGE: &str = "http://www.w3.org/ns/shacl#message";
const SH_NAME: &str = "http://www.w3.org/ns/shacl#name";
const SH_DESCRIPTION: &str = "http://www.w3.org/ns/shacl#description";
const XSD: &str = "http://www.w3.org/2001/XMLSchema#";
const XSD_BOOLEAN: &str = "http://www.w3.org/2001/XMLSchema#boolean";
const XSD_DATETIME: &str = "http://www.w3.org/2001/XMLSchema#dateTime";
const XSD_DOUBLE: &str = "http://www.w3.org/2001/XMLSchema#double";
const XSD_FLOAT: &str = "http://www.w3.org/2001/XMLSchema#float";
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const OWL_THING: &str = "http://www.w3.org/2002/07/owl#Thing";

const MAX_SHAPE_GRAPH_TRIPLES: usize = 16_384;
const MAX_NODE_SHAPES: usize = 1_024;
const MAX_PROPERTY_SHAPES: usize = 4_096;
const MAX_RDF_LIST_ITEMS: usize = 4_096;
const MAX_VALIDATION_STEPS: usize = 1_000_000;

fn is_validating_property_predicate(predicate: &str) -> bool {
    matches!(
        predicate,
        SH_MIN_COUNT
            | SH_MAX_COUNT
            | SH_DATATYPE
            | SH_CLASS
            | SH_IN
            | SH_PATTERN
            | SH_MIN_INCLUSIVE
            | SH_MAX_INCLUSIVE
            | SH_HAS_VALUE
    )
}

fn is_annotation_predicate(predicate: &str) -> bool {
    matches!(predicate, SH_MESSAGE | SH_NAME | SH_DESCRIPTION)
}

fn is_node_shape_predicate(predicate: &str) -> bool {
    matches!(predicate, SH_TARGET_CLASS | SH_PROPERTY) || is_annotation_predicate(predicate)
}

fn is_property_shape_predicate(predicate: &str) -> bool {
    predicate == SH_PATH
        || is_validating_property_predicate(predicate)
        || is_annotation_predicate(predicate)
}

fn is_allowed_shacl_predicate(predicate: &str) -> bool {
    is_node_shape_predicate(predicate) || is_property_shape_predicate(predicate)
}

/// Package activation failed before a node could expose admission capability.
#[derive(Debug, Error)]
#[error("semantic package activation failed: {0}")]
pub struct SemanticActivationError(String);

impl SemanticActivationError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

/// A runtime failure means the node cannot produce a globally valid verdict.
#[derive(Debug, Error)]
pub enum SemanticAdmissionError {
    /// The active node cannot completely execute the bound semantic contract.
    #[error("semantic admission engine unavailable: {0}")]
    NodeIncapacity(String),
}

/// Deterministic result of applying the active package to one transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticAdmissionVerdict {
    /// The full staged union conforms and the candidate contributes a focus subject.
    Conforms,
    /// The candidate deterministically violates the active semantic contract.
    Rejected {
        /// Stable reason suitable for an admission rejection.
        reason: String,
    },
}

#[derive(Clone)]
struct PropertyShape {
    id: String,
    path: NamedNode,
    min_count: Option<usize>,
    max_count: Option<usize>,
    datatype: Option<NamedNode>,
    class: Option<NamedNode>,
    in_values: Option<Vec<Term>>,
    pattern: Option<Arc<Regex>>,
    min_inclusive: Option<Literal>,
    max_inclusive: Option<Literal>,
    has_value: Option<Term>,
}

#[derive(Clone)]
struct NodeShape {
    id: String,
    target_classes: Vec<NamedNode>,
    properties: Vec<PropertyShape>,
}

#[derive(Clone)]
enum SubclassReasoner {
    Spacl(Arc<SimpleReasoner>),
    #[cfg(test)]
    FaultInjected,
}

impl SubclassReasoner {
    fn is_subclass_of(&self, actual: &IRI, required: &IRI) -> Result<bool, String> {
        match self {
            Self::Spacl(reasoner) => reasoner
                .is_subclass_of(actual, required)
                .map_err(|error| format!("SPACL subclass execution failed: {error}")),
            #[cfg(test)]
            Self::FaultInjected => Err("fault-injected SPACL execution failure".to_string()),
        }
    }
}

struct SemanticAssetSnapshot {
    core_ontology: String,
    domain_ontology: String,
    core_shapes: Vec<u8>,
    domain_shapes: Vec<u8>,
    package_hash: String,
}

impl SemanticAssetSnapshot {
    fn load(manifest: &OntologyPackageManifest) -> Result<Self, SemanticActivationError> {
        manifest
            .validate_metadata()
            .map_err(|error| SemanticActivationError::new(error.to_string()))?;

        let core_ontology = read_package_asset(&manifest.core_ontology_path)?;
        let domain_ontology = read_package_asset(&manifest.domain_ontology_path)?;
        let core_shapes = read_package_asset(&manifest.core_shacl_path)?;
        let domain_shapes = read_package_asset(&manifest.domain_shacl_path)?;
        let mapping_assets = manifest
            .mappings
            .iter()
            .map(|path| read_package_asset(path))
            .collect::<Result<Vec<_>, _>>()?;
        let package_hash = manifest
            .compute_package_hash_from_assets(
                [
                    &core_ontology,
                    &domain_ontology,
                    &core_shapes,
                    &domain_shapes,
                ],
                &mapping_assets,
            )
            .map_err(|error| SemanticActivationError::new(error.to_string()))?;
        if let Some(expected_hash) = &manifest.package_hash {
            if expected_hash != &package_hash {
                return Err(SemanticActivationError::new(format!(
                    "ontology package hash mismatch: expected {expected_hash}, computed {package_hash}"
                )));
            }
        }

        Ok(Self {
            core_ontology: String::from_utf8(core_ontology).map_err(|error| {
                SemanticActivationError::new(format!(
                    "{} is not UTF-8 ontology text: {error}",
                    manifest.core_ontology_path
                ))
            })?,
            domain_ontology: String::from_utf8(domain_ontology).map_err(|error| {
                SemanticActivationError::new(format!(
                    "{} is not UTF-8 ontology text: {error}",
                    manifest.domain_ontology_path
                ))
            })?,
            core_shapes,
            domain_shapes,
            package_hash,
        })
    }
}

fn read_package_asset(path: &str) -> Result<Vec<u8>, SemanticActivationError> {
    fs::read(path).map_err(|error| SemanticActivationError::new(format!("{path}: {error}")))
}

/// Fully compiled package runtime bound to an exact package digest and profile.
#[derive(Clone)]
pub struct ActivatedSemanticPackage {
    package_id: String,
    package_version: String,
    package_hash: String,
    semantic_execution_profile_id: String,
    shapes: Arc<Vec<NodeShape>>,
    reasoner: SubclassReasoner,
}

impl fmt::Debug for ActivatedSemanticPackage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ActivatedSemanticPackage")
            .field("package_id", &self.package_id)
            .field("package_version", &self.package_version)
            .field("package_hash", &self.package_hash)
            .field(
                "semantic_execution_profile_id",
                &self.semantic_execution_profile_id,
            )
            .field("node_shape_count", &self.shapes.len())
            .finish_non_exhaustive()
    }
}

impl ActivatedSemanticPackage {
    /// Read, authenticate, compile, and capability-check every declared asset.
    pub fn activate(manifest: &OntologyPackageManifest) -> Result<Self, SemanticActivationError> {
        let snapshot = SemanticAssetSnapshot::load(manifest)?;
        let shapes = compile_shapes(manifest, &snapshot)?;
        let reasoner = activate_reasoner(manifest, &snapshot)?;
        let consistent = reasoner
            .is_consistent()
            .map_err(|error| SemanticActivationError::new(format!("SPACL failed: {error}")))?;
        if !consistent {
            return Err(SemanticActivationError::new(
                "SPACL reports inconsistent package ontologies",
            ));
        }

        Ok(Self {
            package_id: manifest.package_id.clone(),
            package_version: manifest.package_version.clone(),
            package_hash: snapshot.package_hash,
            semantic_execution_profile_id: manifest.semantic_execution_profile_id.clone(),
            shapes: Arc::new(shapes),
            reasoner: SubclassReasoner::Spacl(Arc::new(reasoner)),
        })
    }

    /// Stable package identifier selected by the active profile.
    pub fn package_id(&self) -> &str {
        &self.package_id
    }

    /// Exact package version selected by the active profile.
    pub fn package_version(&self) -> &str {
        &self.package_version
    }

    /// Path-independent content digest of the activated package.
    pub fn package_hash(&self) -> &str {
        &self.package_hash
    }

    /// Bounded execution profile implemented by this runtime.
    pub fn semantic_execution_profile_id(&self) -> &str {
        &self.semantic_execution_profile_id
    }

    #[cfg(test)]
    fn inject_reasoner_execution_failure_for_test(&mut self) {
        self.reasoner = SubclassReasoner::FaultInjected;
    }

    /// Validate a committed parent without applying the candidate-focus rule.
    pub fn validate_committed_state(
        &self,
        public_state: &[Quad],
    ) -> Result<(), SemanticAdmissionError> {
        let graph = union_graph(public_state, &[]);
        if let Some(reason) = self.graph_bound_violation(&graph) {
            return Err(SemanticAdmissionError::NodeIncapacity(format!(
                "committed parent state exceeds the semantic profile: {reason}"
            )));
        }
        match self.evaluate_graph(&graph) {
            Ok(None) => Ok(()),
            Ok(Some(reason)) => Err(SemanticAdmissionError::NodeIncapacity(format!(
                "committed parent state is nonconforming: {reason}"
            ))),
            Err(error) => Err(SemanticAdmissionError::NodeIncapacity(error)),
        }
    }

    /// Validate the complete staged union and require a candidate-touched focus node.
    pub fn validate_transition(
        &self,
        parent_public_state: &[Quad],
        candidate_public_state: &[Quad],
    ) -> Result<SemanticAdmissionVerdict, SemanticAdmissionError> {
        self.validate_committed_state(parent_public_state)?;

        let graph = union_graph(parent_public_state, candidate_public_state);
        if let Some(reason) = self.graph_bound_violation(&graph) {
            return Ok(SemanticAdmissionVerdict::Rejected { reason });
        }
        let focus = self
            .focus_nodes(&graph)
            .map_err(SemanticAdmissionError::NodeIncapacity)?;
        let candidate_subjects: HashSet<Subject> = candidate_public_state
            .iter()
            .map(|quad| quad.subject.clone())
            .collect();
        if focus
            .iter()
            .all(|(_, subject)| !candidate_subjects.contains(subject))
        {
            return Ok(SemanticAdmissionVerdict::Rejected {
                reason: "ordinary candidate contributes no package-selected focus-node subject"
                    .to_string(),
            });
        }

        match self.evaluate_with_focus(&graph, &focus) {
            Ok(None) => Ok(SemanticAdmissionVerdict::Conforms),
            Ok(Some(reason)) => Ok(SemanticAdmissionVerdict::Rejected { reason }),
            Err(error) => Err(SemanticAdmissionError::NodeIncapacity(error)),
        }
    }

    fn evaluate_graph(&self, graph: &HashSet<Triple>) -> Result<Option<String>, String> {
        let focus = self.focus_nodes(graph)?;
        self.evaluate_with_focus(graph, &focus)
    }

    fn graph_bound_violation(&self, graph: &HashSet<Triple>) -> Option<String> {
        if graph.len() > MAX_VALIDATION_STEPS {
            return Some(format!(
                "semantic validation graph exceeds profile bound {MAX_VALIDATION_STEPS}"
            ));
        }
        let subject_count = graph
            .iter()
            .map(|triple| &triple.subject)
            .collect::<HashSet<_>>()
            .len();
        let target_count = self
            .shapes
            .iter()
            .map(|shape| shape.target_classes.len())
            .sum::<usize>();
        let focus_work = subject_count.checked_mul(target_count);
        if !matches!(focus_work, Some(work) if work <= MAX_VALIDATION_STEPS) {
            return Some(format!(
                "semantic focus selection exceeds profile work bound {MAX_VALIDATION_STEPS}"
            ));
        }
        None
    }

    fn focus_nodes(&self, graph: &HashSet<Triple>) -> Result<Vec<(usize, Subject)>, String> {
        let mut focus = Vec::new();
        let mut subjects: Vec<Subject> =
            graph.iter().map(|triple| triple.subject.clone()).collect();
        subjects.sort_by_key(ToString::to_string);
        subjects.dedup();

        for (shape_index, shape) in self.shapes.iter().enumerate() {
            for subject in &subjects {
                let mut selected = false;
                for target in &shape.target_classes {
                    if self.subject_has_class(graph, subject, target)? {
                        selected = true;
                        break;
                    }
                }
                if selected {
                    focus.push((shape_index, subject.clone()));
                    if focus.len() > MAX_VALIDATION_STEPS {
                        return Err("semantic focus selection exceeded execution bound".to_string());
                    }
                }
            }
        }
        Ok(focus)
    }

    fn subject_has_class(
        &self,
        graph: &HashSet<Triple>,
        subject: &Subject,
        required: &NamedNode,
    ) -> Result<bool, String> {
        // OWL's top class contains every RDF node; this remains a package-selected
        // target and is useful for packages that intentionally govern all subjects.
        if required.as_str() == OWL_THING {
            return Ok(true);
        }
        for triple in graph {
            if &triple.subject != subject || triple.predicate.as_str() != RDF_TYPE {
                continue;
            }
            let Term::NamedNode(actual) = &triple.object else {
                continue;
            };
            if actual == required || self.is_subclass(actual, required)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn is_subclass(&self, actual: &NamedNode, required: &NamedNode) -> Result<bool, String> {
        let actual = IRI::new(actual.as_str().to_string())
            .map_err(|error| format!("SPACL actual-class IRI error: {error}"))?;
        let required = IRI::new(required.as_str().to_string())
            .map_err(|error| format!("SPACL required-class IRI error: {error}"))?;
        self.reasoner.is_subclass_of(&actual, &required)
    }

    fn evaluate_with_focus(
        &self,
        graph: &HashSet<Triple>,
        focus: &[(usize, Subject)],
    ) -> Result<Option<String>, String> {
        let mut steps = 0usize;
        for (shape_index, subject) in focus {
            let shape = &self.shapes[*shape_index];
            for property in &shape.properties {
                steps = steps.saturating_add(graph.len());
                if steps > MAX_VALIDATION_STEPS {
                    return Ok(Some(format!(
                        "semantic validation exceeds profile work bound {MAX_VALIDATION_STEPS}"
                    )));
                }
                let mut values: Vec<&Term> = graph
                    .iter()
                    .filter(|triple| {
                        &triple.subject == subject && triple.predicate == property.path
                    })
                    .map(|triple| &triple.object)
                    .collect();
                values.sort_by_key(|term| term.to_string());

                if let Some(minimum) = property.min_count {
                    if values.len() < minimum {
                        return Ok(Some(violation(shape, property, subject, "minCount")));
                    }
                }
                if let Some(maximum) = property.max_count {
                    if values.len() > maximum {
                        return Ok(Some(violation(shape, property, subject, "maxCount")));
                    }
                }
                for value in &values {
                    if let Some(datatype) = &property.datatype {
                        let Term::Literal(literal) = value else {
                            return Ok(Some(violation(shape, property, subject, "datatype")));
                        };
                        if !literal_matches_datatype(literal, datatype) {
                            return Ok(Some(violation(shape, property, subject, "datatype")));
                        }
                    }
                    if let Some(class) = &property.class {
                        let Some(value_subject) = term_as_subject(value) else {
                            return Ok(Some(violation(shape, property, subject, "class")));
                        };
                        if !self.subject_has_class(graph, &value_subject, class)? {
                            return Ok(Some(violation(shape, property, subject, "class")));
                        }
                    }
                    if let Some(allowed) = &property.in_values {
                        if !allowed.contains(value) {
                            return Ok(Some(violation(shape, property, subject, "in")));
                        }
                    }
                    if let Some(pattern) = &property.pattern {
                        let Some(text) = sparql_string_form(value) else {
                            return Ok(Some(violation(shape, property, subject, "pattern")));
                        };
                        if !pattern.is_match(text) {
                            return Ok(Some(violation(shape, property, subject, "pattern")));
                        }
                    }
                    if let Some(minimum) = &property.min_inclusive {
                        if !compare_literals(value, minimum, Comparison::Minimum) {
                            return Ok(Some(violation(shape, property, subject, "minInclusive")));
                        }
                    }
                    if let Some(maximum) = &property.max_inclusive {
                        if !compare_literals(value, maximum, Comparison::Maximum) {
                            return Ok(Some(violation(shape, property, subject, "maxInclusive")));
                        }
                    }
                }
                if let Some(required) = &property.has_value {
                    if !values.contains(&required) {
                        return Ok(Some(violation(shape, property, subject, "hasValue")));
                    }
                }
            }
        }
        Ok(None)
    }
}

fn union_graph(parent: &[Quad], candidate: &[Quad]) -> HashSet<Triple> {
    parent
        .iter()
        .chain(candidate)
        .map(|quad| {
            Triple::new(
                quad.subject.clone(),
                quad.predicate.clone(),
                quad.object.clone(),
            )
        })
        .collect()
}

fn violation(
    shape: &NodeShape,
    property: &PropertyShape,
    focus: &Subject,
    component: &str,
) -> String {
    format!(
        "semantic constraint violation: shape={}, property={}, focus={}, component={component}",
        shape.id, property.id, focus
    )
}

fn term_as_subject(term: &Term) -> Option<Subject> {
    match term {
        Term::NamedNode(node) => Some(node.clone().into()),
        Term::BlankNode(node) => Some(node.clone().into()),
        Term::Literal(_) => None,
        Term::Triple(triple) => Some(triple.clone().into()),
    }
}

fn sparql_string_form(term: &Term) -> Option<&str> {
    match term {
        Term::NamedNode(node) => Some(node.as_str()),
        Term::Literal(literal) => Some(literal.value()),
        Term::BlankNode(_) | Term::Triple(_) => None,
    }
}

#[derive(Clone, Copy)]
enum Comparison {
    Minimum,
    Maximum,
}

fn compare_literals(value: &Term, bound: &Literal, comparison: Comparison) -> bool {
    let Term::Literal(value) = value else {
        return false;
    };
    let Some(value) = ComparableLiteral::parse(value) else {
        return false;
    };
    let Some(bound) = ComparableLiteral::parse(bound) else {
        return false;
    };
    match (value, bound) {
        (ComparableLiteral::Number(value), ComparableLiteral::Number(bound)) => value
            .compare(&bound)
            .is_some_and(|ordering| match comparison {
                Comparison::Minimum => ordering != Ordering::Less,
                Comparison::Maximum => ordering != Ordering::Greater,
            }),
        (ComparableLiteral::Date(value), ComparableLiteral::Date(bound)) => match comparison {
            Comparison::Minimum => value >= bound,
            Comparison::Maximum => value <= bound,
        },
        (ComparableLiteral::DateTime(value), ComparableLiteral::DateTime(bound)) => {
            match comparison {
                Comparison::Minimum => value >= bound,
                Comparison::Maximum => value <= bound,
            }
        }
        (ComparableLiteral::LocalDateTime(value), ComparableLiteral::LocalDateTime(bound)) => {
            match comparison {
                Comparison::Minimum => value >= bound,
                Comparison::Maximum => value <= bound,
            }
        }
        _ => false,
    }
}

fn literal_matches_datatype(literal: &Literal, required: &NamedNode) -> bool {
    let datatype = literal.datatype().as_str();
    if datatype != required.as_str() {
        return false;
    }

    let lexical = literal.value();
    match datatype {
        XSD_BOOLEAN => lexical.parse::<XsdBoolean>().is_ok(),
        XSD_DATETIME => lexical.parse::<XsdDateTime>().is_ok(),
        XSD_DOUBLE => lexical.parse::<XsdDouble>().is_ok(),
        XSD_FLOAT => lexical.parse::<XsdFloat>().is_ok(),
        XSD_STRING => lexical.chars().all(is_xml_character),
        datatype if is_numeric_datatype(datatype) => {
            NumericLiteral::parse(datatype, lexical).is_some()
        }
        _ => true,
    }
}

fn is_xml_character(character: char) -> bool {
    matches!(character, '\u{9}' | '\u{A}' | '\u{D}')
        || ('\u{20}'..='\u{D7FF}').contains(&character)
        || ('\u{E000}'..='\u{FFFD}').contains(&character)
        || ('\u{10000}'..='\u{10FFFF}').contains(&character)
}

enum ComparableLiteral {
    Number(NumericLiteral),
    Date(NaiveDate),
    DateTime(DateTime<chrono::FixedOffset>),
    LocalDateTime(NaiveDateTime),
}

impl ComparableLiteral {
    fn parse(literal: &Literal) -> Option<Self> {
        let datatype = literal.datatype().as_str();
        let lexical = literal.value();
        if is_numeric_datatype(datatype) {
            return NumericLiteral::parse(datatype, lexical).map(Self::Number);
        }
        match datatype {
            "http://www.w3.org/2001/XMLSchema#date" => {
                NaiveDate::parse_from_str(lexical, "%Y-%m-%d")
                    .ok()
                    .map(Self::Date)
            }
            "http://www.w3.org/2001/XMLSchema#dateTime" => DateTime::parse_from_rfc3339(lexical)
                .ok()
                .map(Self::DateTime)
                .or_else(|| {
                    NaiveDateTime::parse_from_str(lexical, "%Y-%m-%dT%H:%M:%S%.f")
                        .ok()
                        .map(Self::LocalDateTime)
                }),
            _ => None,
        }
    }
}

enum NumericLiteral {
    Exact(ExactDecimal),
    Float(f64),
}

impl NumericLiteral {
    fn parse(datatype: &str, lexical: &str) -> Option<Self> {
        let datatype = datatype.strip_prefix(XSD)?;
        match datatype {
            "float" | "double" => parse_xsd_float(lexical).map(Self::Float),
            "decimal" => ExactDecimal::parse(lexical, false).map(Self::Exact),
            "integer" | "nonPositiveInteger" | "negativeInteger" | "long" | "int" | "short"
            | "byte" | "nonNegativeInteger" | "unsignedLong" | "unsignedInt" | "unsignedShort"
            | "unsignedByte" | "positiveInteger" => {
                let value = ExactDecimal::parse(lexical, true)?;
                integer_value_is_in_datatype(&value, datatype).then_some(Self::Exact(value))
            }
            _ => None,
        }
    }

    fn compare(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Exact(left), Self::Exact(right)) => Some(left.cmp(right)),
            _ => self.as_f64().partial_cmp(&other.as_f64()),
        }
    }

    fn as_f64(&self) -> f64 {
        match self {
            Self::Exact(value) => value.as_f64(),
            Self::Float(value) => *value,
        }
    }
}

#[derive(Eq, PartialEq)]
struct ExactDecimal {
    negative: bool,
    integer: String,
    fractional: String,
}

impl ExactDecimal {
    fn parse(lexical: &str, integer_only: bool) -> Option<Self> {
        let (negative, unsigned) = if let Some(value) = lexical.strip_prefix('-') {
            (true, value)
        } else if let Some(value) = lexical.strip_prefix('+') {
            (false, value)
        } else {
            (false, lexical)
        };
        if unsigned.is_empty() {
            return None;
        }
        let mut pieces = unsigned.split('.');
        let integer = pieces.next()?;
        let fractional = pieces.next().unwrap_or_default();
        if pieces.next().is_some()
            || (integer.is_empty() && fractional.is_empty())
            || !integer.bytes().all(|byte| byte.is_ascii_digit())
            || !fractional.bytes().all(|byte| byte.is_ascii_digit())
            || (integer_only && unsigned.contains('.'))
        {
            return None;
        }
        let integer = integer.trim_start_matches('0');
        let integer = if integer.is_empty() { "0" } else { integer };
        let fractional = fractional.trim_end_matches('0');
        let is_zero = integer == "0" && fractional.is_empty();
        Some(Self {
            negative: negative && !is_zero,
            integer: integer.to_string(),
            fractional: fractional.to_string(),
        })
    }

    fn compare_absolute(&self, other: &Self) -> Ordering {
        self.integer
            .len()
            .cmp(&other.integer.len())
            .then_with(|| self.integer.cmp(&other.integer))
            .then_with(|| {
                let width = self.fractional.len().max(other.fractional.len());
                self.fractional
                    .bytes()
                    .chain(std::iter::repeat(b'0'))
                    .take(width)
                    .cmp(
                        other
                            .fractional
                            .bytes()
                            .chain(std::iter::repeat(b'0'))
                            .take(width),
                    )
            })
    }

    fn as_f64(&self) -> f64 {
        let sign = if self.negative { "-" } else { "" };
        let lexical = if self.fractional.is_empty() {
            format!("{sign}{}", self.integer)
        } else {
            format!("{sign}{}.{}", self.integer, self.fractional)
        };
        lexical.parse().unwrap_or(if self.negative {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        })
    }
}

impl Ord for ExactDecimal {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.negative, other.negative) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => self.compare_absolute(other),
            (true, true) => self.compare_absolute(other).reverse(),
        }
    }
}

impl PartialOrd for ExactDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn parse_xsd_float(lexical: &str) -> Option<f64> {
    match lexical {
        "INF" | "+INF" => Some(f64::INFINITY),
        "-INF" => Some(f64::NEG_INFINITY),
        "NaN" => None,
        value
            if value
                .bytes()
                .any(|byte| byte.is_ascii_alphabetic() && !matches!(byte, b'e' | b'E')) =>
        {
            None
        }
        value => value.parse::<f64>().ok().filter(|number| !number.is_nan()),
    }
}

fn integer_value_is_in_datatype(value: &ExactDecimal, datatype: &str) -> bool {
    let within = |minimum: &str, maximum: &str| {
        let minimum = ExactDecimal::parse(minimum, true).expect("valid integer lower bound");
        let maximum = ExactDecimal::parse(maximum, true).expect("valid integer upper bound");
        value >= &minimum && value <= &maximum
    };
    let zero = ExactDecimal::parse("0", true).expect("valid zero");
    match datatype {
        "integer" => true,
        "nonPositiveInteger" => value <= &zero,
        "negativeInteger" => value < &zero,
        "long" => within("-9223372036854775808", "9223372036854775807"),
        "int" => within("-2147483648", "2147483647"),
        "short" => within("-32768", "32767"),
        "byte" => within("-128", "127"),
        "nonNegativeInteger" => value >= &zero,
        "unsignedLong" => within("0", "18446744073709551615"),
        "unsignedInt" => within("0", "4294967295"),
        "unsignedShort" => within("0", "65535"),
        "unsignedByte" => within("0", "255"),
        "positiveInteger" => value > &zero,
        _ => false,
    }
}

fn is_numeric_datatype(datatype: &str) -> bool {
    matches!(
        datatype.strip_prefix(XSD),
        Some(
            "decimal"
                | "integer"
                | "nonPositiveInteger"
                | "negativeInteger"
                | "long"
                | "int"
                | "short"
                | "byte"
                | "nonNegativeInteger"
                | "unsignedLong"
                | "unsignedInt"
                | "unsignedShort"
                | "unsignedByte"
                | "positiveInteger"
                | "float"
                | "double"
        )
    )
}

fn activate_reasoner(
    manifest: &OntologyPackageManifest,
    snapshot: &SemanticAssetSnapshot,
) -> Result<SimpleReasoner, SemanticActivationError> {
    let artifacts = [
        (
            manifest.core_ontology_path.as_str(),
            snapshot.core_ontology.as_str(),
        ),
        (
            manifest.domain_ontology_path.as_str(),
            snapshot.domain_ontology.as_str(),
        ),
    ];
    let mut combined = None;
    let mut declared_subclasses = Vec::new();
    for (path, content) in artifacts {
        if content.trim().is_empty() {
            return Err(SemanticActivationError::new(format!(
                "ontology artifact is empty: {path}"
            )));
        }
        let parser = ParserFactory::auto_detect(content).ok_or_else(|| {
            SemanticActivationError::new(format!("cannot detect ontology syntax: {path}"))
        })?;
        let mut ontology = parser.parse_str(content).map_err(|error| {
            SemanticActivationError::new(format!("cannot parse ontology {path}: {error}"))
        })?;
        let subclasses = compile_declared_subclasses(content, path)?;
        for (subclass, superclass) in &subclasses {
            ontology
                .add_class(Class::new(subclass.clone()))
                .and_then(|()| ontology.add_class(Class::new(superclass.clone())))
                .and_then(|()| {
                    ontology.add_subclass_axiom(SubClassOfAxiom::new(
                        ClassExpression::Class(Class::new(subclass.clone())),
                        ClassExpression::Class(Class::new(superclass.clone())),
                    ))
                })
                .map_err(|error| {
                    SemanticActivationError::new(format!(
                        "cannot compile subclass entailment from {path}: {error}"
                    ))
                })?;
        }
        declared_subclasses.extend(subclasses);
        if let Some(target) = &mut combined {
            merge_ontology(target, &ontology)?;
        } else {
            combined = Some(ontology);
        }
    }
    let reasoner =
        SimpleReasoner::new(combined.ok_or_else(|| {
            SemanticActivationError::new("package declares no ontology artifacts")
        })?);
    for (subclass, superclass) in declared_subclasses {
        if !reasoner
            .is_subclass_of(&subclass, &superclass)
            .map_err(|error| {
                SemanticActivationError::new(format!(
                    "SPACL subclass capability check failed: {error}"
                ))
            })?
        {
            return Err(SemanticActivationError::new(format!(
                "SPACL could not execute declared subclass entailment {} -> {}",
                subclass.as_str(),
                superclass.as_str()
            )));
        }
    }
    Ok(reasoner)
}

fn compile_declared_subclasses(
    content: &str,
    path: &str,
) -> Result<Vec<(IRI, IRI)>, SemanticActivationError> {
    let store = Store::new().map_err(|error| SemanticActivationError::new(error.to_string()))?;
    let trimmed = content.trim_start();
    let format = if trimmed.starts_with("<?xml") || trimmed.starts_with("<rdf:RDF") {
        RdfFormat::RdfXml
    } else if content
        .lines()
        .filter(|line| {
            let line = line.trim();
            !line.is_empty() && !line.starts_with('#')
        })
        .all(|line| line.ends_with(" .") && line.starts_with('<'))
    {
        RdfFormat::NTriples
    } else {
        RdfFormat::Turtle
    };
    store
        .load_from_reader(format, Cursor::new(content.as_bytes()))
        .map_err(|error| {
            SemanticActivationError::new(format!(
                "cannot compile ontology entailment view {path}: {error}"
            ))
        })?;
    let statement_count = store
        .len()
        .map_err(|error| SemanticActivationError::new(error.to_string()))?;
    if statement_count == 0 {
        return Err(SemanticActivationError::new(format!(
            "ontology artifact contains no RDF statements: {path}"
        )));
    }
    let mut subclasses = Vec::new();
    for result in store.iter() {
        let quad = result.map_err(|error| SemanticActivationError::new(error.to_string()))?;
        if quad.predicate.as_str() != RDFS_SUBCLASS_OF {
            continue;
        }
        let Subject::NamedNode(subclass) = quad.subject else {
            return Err(SemanticActivationError::new(format!(
                "unsupported anonymous rdfs:subClassOf subject in {path}"
            )));
        };
        let Term::NamedNode(superclass) = quad.object else {
            return Err(SemanticActivationError::new(format!(
                "unsupported complex rdfs:subClassOf object in {path}"
            )));
        };
        subclasses.push((
            IRI::new(subclass.as_str().to_string()).map_err(|error| {
                SemanticActivationError::new(format!("invalid subclass IRI: {error}"))
            })?,
            IRI::new(superclass.as_str().to_string()).map_err(|error| {
                SemanticActivationError::new(format!("invalid superclass IRI: {error}"))
            })?,
        ));
    }
    Ok(subclasses)
}

fn merge_ontology(
    target: &mut owl2_reasoner::Ontology,
    source: &owl2_reasoner::Ontology,
) -> Result<(), SemanticActivationError> {
    for class in source.classes() {
        target.add_class((**class).clone()).map_err(|error| {
            SemanticActivationError::new(format!("cannot merge ontology class: {error}"))
        })?;
    }
    for property in source.object_properties() {
        target
            .add_object_property((**property).clone())
            .map_err(|error| {
                SemanticActivationError::new(format!(
                    "cannot merge ontology object property: {error}"
                ))
            })?;
    }
    for property in source.data_properties() {
        target
            .add_data_property((**property).clone())
            .map_err(|error| {
                SemanticActivationError::new(format!(
                    "cannot merge ontology data property: {error}"
                ))
            })?;
    }
    for axiom in source.axioms() {
        target.add_axiom((**axiom).clone()).map_err(|error| {
            SemanticActivationError::new(format!("cannot merge ontology axiom: {error}"))
        })?;
    }
    Ok(())
}

fn compile_shapes(
    manifest: &OntologyPackageManifest,
    snapshot: &SemanticAssetSnapshot,
) -> Result<Vec<NodeShape>, SemanticActivationError> {
    let store = Store::new().map_err(|error| SemanticActivationError::new(error.to_string()))?;
    let artifacts = [
        (
            manifest.core_shacl_path.as_str(),
            snapshot.core_shapes.as_slice(),
        ),
        (
            manifest.domain_shacl_path.as_str(),
            snapshot.domain_shapes.as_slice(),
        ),
    ];
    for (path, bytes) in artifacts {
        if bytes.iter().all(u8::is_ascii_whitespace) {
            return Err(SemanticActivationError::new(format!(
                "shapes artifact is empty: {path}"
            )));
        }
        let artifact =
            Store::new().map_err(|error| SemanticActivationError::new(error.to_string()))?;
        artifact
            .load_from_reader(RdfFormat::Turtle, Cursor::new(&bytes))
            .map_err(|error| {
                SemanticActivationError::new(format!("cannot parse shapes {path}: {error}"))
            })?;
        let artifact_len = artifact
            .len()
            .map_err(|error| SemanticActivationError::new(error.to_string()))?;
        if artifact_len == 0 {
            return Err(SemanticActivationError::new(format!(
                "shapes artifact contains no RDF statements: {path}"
            )));
        }
        for result in artifact.iter() {
            let quad = result.map_err(|error| SemanticActivationError::new(error.to_string()))?;
            store
                .insert(&quad)
                .map_err(|error| SemanticActivationError::new(error.to_string()))?;
        }
    }
    let graph_len = store
        .len()
        .map_err(|error| SemanticActivationError::new(error.to_string()))?;
    if graph_len > MAX_SHAPE_GRAPH_TRIPLES {
        return Err(SemanticActivationError::new(format!(
            "shapes graph exceeds profile bound {MAX_SHAPE_GRAPH_TRIPLES}"
        )));
    }
    inspect_shape_vocabulary(&store)?;

    let mut node_shape_ids = typed_subjects(&store, SH_NODE_SHAPE)?;
    node_shape_ids.sort_by_key(ToString::to_string);
    node_shape_ids.dedup();
    if node_shape_ids.is_empty() {
        return Err(SemanticActivationError::new(
            "package contains no executable active node shape",
        ));
    }
    if node_shape_ids.len() > MAX_NODE_SHAPES {
        return Err(SemanticActivationError::new(format!(
            "node-shape count exceeds profile bound {MAX_NODE_SHAPES}"
        )));
    }

    let node_shape_set: HashSet<Subject> = node_shape_ids.iter().cloned().collect();
    let mut attached_property_set = HashSet::new();
    for node_shape_id in &node_shape_ids {
        for property in objects(&store, node_shape_id, SH_PROPERTY)? {
            attached_property_set.insert(subject_term(property, "sh:property")?);
        }
    }
    validate_compilation_coverage(&store, &node_shape_set, &attached_property_set)?;

    let mut property_count = 0usize;
    let mut shapes = Vec::with_capacity(node_shape_ids.len());
    for node_shape_id in node_shape_ids {
        let targets = objects(&store, &node_shape_id, SH_TARGET_CLASS)?;
        if targets.is_empty() {
            return Err(SemanticActivationError::new(format!(
                "node shape {node_shape_id} has no sh:targetClass"
            )));
        }
        let mut target_classes = targets
            .into_iter()
            .map(|term| named_node(term, "sh:targetClass"))
            .collect::<Result<Vec<_>, _>>()?;
        target_classes.sort_by_key(|node| node.as_str().to_string());
        target_classes.dedup();

        let property_ids = objects(&store, &node_shape_id, SH_PROPERTY)?;
        let mut properties = Vec::with_capacity(property_ids.len());
        for property_term in property_ids {
            let property_id = subject_term(property_term, "sh:property")?;
            property_count += 1;
            if property_count > MAX_PROPERTY_SHAPES {
                return Err(SemanticActivationError::new(format!(
                    "property-shape count exceeds profile bound {MAX_PROPERTY_SHAPES}"
                )));
            }
            properties.push(compile_property_shape(&store, &property_id)?);
        }
        properties.sort_by(|left, right| left.id.cmp(&right.id));
        shapes.push(NodeShape {
            id: node_shape_id.to_string(),
            target_classes,
            properties,
        });
    }
    Ok(shapes)
}

fn validate_compilation_coverage(
    store: &Store,
    node_shapes: &HashSet<Subject>,
    attached_properties: &HashSet<Subject>,
) -> Result<(), SemanticActivationError> {
    for result in store.iter() {
        let quad = result.map_err(|error| SemanticActivationError::new(error.to_string()))?;
        let predicate = quad.predicate.as_str();
        if node_shapes.contains(&quad.subject) {
            let supported = if predicate == RDF_TYPE {
                matches!(&quad.object, Term::NamedNode(kind) if kind.as_str() == SH_NODE_SHAPE)
            } else {
                is_node_shape_predicate(predicate)
            };
            if !supported {
                return Err(SemanticActivationError::new(format!(
                    "unsupported node-shape statement on {}: <{predicate}> {}",
                    quad.subject, quad.object
                )));
            }
        }
        if attached_properties.contains(&quad.subject) {
            let supported = if predicate == RDF_TYPE {
                matches!(&quad.object, Term::NamedNode(kind) if kind.as_str() == SH_PROPERTY_SHAPE)
            } else {
                is_property_shape_predicate(predicate)
            };
            if !supported {
                return Err(SemanticActivationError::new(format!(
                    "unsupported property-shape statement on {}: <{predicate}> {}",
                    quad.subject, quad.object
                )));
            }
        }
        if matches!(predicate, SH_TARGET_CLASS | SH_PROPERTY)
            && !node_shapes.contains(&quad.subject)
        {
            return Err(SemanticActivationError::new(format!(
                "SHACL node-shape predicate <{predicate}> occurs on an uncompiled node {}",
                quad.subject
            )));
        }
        if (predicate == SH_PATH || is_validating_property_predicate(predicate))
            && !attached_properties.contains(&quad.subject)
        {
            return Err(SemanticActivationError::new(format!(
                "SHACL property predicate <{predicate}> occurs on an unattached property shape {}",
                quad.subject
            )));
        }
        if predicate == RDF_TYPE
            && matches!(&quad.object, Term::NamedNode(kind) if kind.as_str() == SH_PROPERTY_SHAPE)
            && !attached_properties.contains(&quad.subject)
        {
            return Err(SemanticActivationError::new(format!(
                "standalone property shape {} is unsupported by this execution profile",
                quad.subject
            )));
        }
    }
    Ok(())
}

fn inspect_shape_vocabulary(store: &Store) -> Result<(), SemanticActivationError> {
    for result in store.iter() {
        let quad = result.map_err(|error| SemanticActivationError::new(error.to_string()))?;
        let predicate = quad.predicate.as_str();
        if predicate.starts_with(SH) && !is_allowed_shacl_predicate(predicate) {
            return Err(SemanticActivationError::new(format!(
                "unsupported SHACL predicate <{predicate}>"
            )));
        }
        if predicate == RDF_TYPE {
            if let Term::NamedNode(kind) = &quad.object {
                if kind.as_str().starts_with(SH)
                    && !matches!(kind.as_str(), SH_NODE_SHAPE | SH_PROPERTY_SHAPE)
                {
                    return Err(SemanticActivationError::new(format!(
                        "unsupported SHACL type <{}>",
                        kind.as_str()
                    )));
                }
            }
        }
        if predicate.starts_with(RDF) && !matches!(predicate, RDF_TYPE | RDF_FIRST | RDF_REST) {
            return Err(SemanticActivationError::new(format!(
                "unsupported RDF structural predicate <{predicate}> in shapes graph"
            )));
        }
    }
    Ok(())
}

fn compile_property_shape(
    store: &Store,
    id: &Subject,
) -> Result<PropertyShape, SemanticActivationError> {
    let path = exactly_one(store, id, SH_PATH, true)?;
    let path = named_node(path, "simple IRI sh:path")?;
    let min_count = optional_count(store, id, SH_MIN_COUNT)?;
    let max_count = optional_count(store, id, SH_MAX_COUNT)?;
    if let (Some(minimum), Some(maximum)) = (min_count, max_count) {
        if minimum > maximum {
            return Err(SemanticActivationError::new(format!(
                "property shape {id} has minCount greater than maxCount"
            )));
        }
    }
    let datatype = optional_named(store, id, SH_DATATYPE)?;
    let class = optional_named(store, id, SH_CLASS)?;
    let in_values = optional_term(store, id, SH_IN)?
        .map(|head| parse_rdf_list(store, head))
        .transpose()?;
    let pattern = optional_literal(store, id, SH_PATTERN)?
        .map(|literal| {
            if literal.datatype().as_str() != XSD_STRING {
                return Err(SemanticActivationError::new(format!(
                    "property shape {id} requires an xsd:string sh:pattern parameter"
                )));
            }
            Regex::xpath(literal.value(), "")
                .map(Arc::new)
                .map_err(|error| {
                    SemanticActivationError::new(format!(
                        "property shape {id} has invalid XPath sh:pattern: {error:?}"
                    ))
                })
        })
        .transpose()?;
    let min_inclusive = optional_literal(store, id, SH_MIN_INCLUSIVE)?;
    let max_inclusive = optional_literal(store, id, SH_MAX_INCLUSIVE)?;
    for bound in [&min_inclusive, &max_inclusive].into_iter().flatten() {
        if ComparableLiteral::parse(bound).is_none() {
            return Err(SemanticActivationError::new(format!(
                "property shape {id} uses an unsupported or invalid range datatype"
            )));
        }
    }
    let has_value = optional_term(store, id, SH_HAS_VALUE)?;

    Ok(PropertyShape {
        id: id.to_string(),
        path,
        min_count,
        max_count,
        datatype,
        class,
        in_values,
        pattern,
        min_inclusive,
        max_inclusive,
        has_value,
    })
}

fn typed_subjects(store: &Store, type_iri: &str) -> Result<Vec<Subject>, SemanticActivationError> {
    let mut subjects = Vec::new();
    for result in store.iter() {
        let quad = result.map_err(|error| SemanticActivationError::new(error.to_string()))?;
        if quad.predicate.as_str() == RDF_TYPE
            && matches!(&quad.object, Term::NamedNode(node) if node.as_str() == type_iri)
        {
            subjects.push(quad.subject);
        }
    }
    Ok(subjects)
}

fn objects(
    store: &Store,
    subject: &Subject,
    predicate: &str,
) -> Result<Vec<Term>, SemanticActivationError> {
    let mut values = Vec::new();
    for result in store.iter() {
        let quad = result.map_err(|error| SemanticActivationError::new(error.to_string()))?;
        if &quad.subject == subject && quad.predicate.as_str() == predicate {
            values.push(quad.object);
        }
    }
    values.sort_by_key(ToString::to_string);
    Ok(values)
}

fn exactly_one(
    store: &Store,
    subject: &Subject,
    predicate: &str,
    required: bool,
) -> Result<Term, SemanticActivationError> {
    let mut values = objects(store, subject, predicate)?;
    match values.len() {
        1 => Ok(values.remove(0)),
        0 if required => Err(SemanticActivationError::new(format!(
            "{subject} requires exactly one <{predicate}>"
        ))),
        count => Err(SemanticActivationError::new(format!(
            "{subject} has {count} values for <{predicate}>"
        ))),
    }
}

fn optional_term(
    store: &Store,
    subject: &Subject,
    predicate: &str,
) -> Result<Option<Term>, SemanticActivationError> {
    let values = objects(store, subject, predicate)?;
    match values.as_slice() {
        [] => Ok(None),
        [value] => Ok(Some(value.clone())),
        _ => Err(SemanticActivationError::new(format!(
            "{subject} has multiple values for <{predicate}>"
        ))),
    }
}

fn optional_named(
    store: &Store,
    subject: &Subject,
    predicate: &str,
) -> Result<Option<NamedNode>, SemanticActivationError> {
    optional_term(store, subject, predicate)?
        .map(|term| named_node(term, predicate))
        .transpose()
}

fn optional_literal(
    store: &Store,
    subject: &Subject,
    predicate: &str,
) -> Result<Option<Literal>, SemanticActivationError> {
    optional_term(store, subject, predicate)?
        .map(|term| match term {
            Term::Literal(literal) => Ok(literal),
            _ => Err(SemanticActivationError::new(format!(
                "<{predicate}> on {subject} must be a literal"
            ))),
        })
        .transpose()
}

fn optional_count(
    store: &Store,
    subject: &Subject,
    predicate: &str,
) -> Result<Option<usize>, SemanticActivationError> {
    optional_literal(store, subject, predicate)?
        .map(|literal| {
            if literal.datatype().as_str() != XSD_INTEGER {
                return Err(SemanticActivationError::new(format!(
                    "<{predicate}> on {subject} must be an xsd:integer literal"
                )));
            }
            literal.value().parse::<usize>().map_err(|_| {
                SemanticActivationError::new(format!(
                    "<{predicate}> on {subject} must be a non-negative integer"
                ))
            })
        })
        .transpose()
}

fn named_node(term: Term, role: &str) -> Result<NamedNode, SemanticActivationError> {
    match term {
        Term::NamedNode(node) => Ok(node),
        _ => Err(SemanticActivationError::new(format!(
            "{role} must be an IRI"
        ))),
    }
}

fn subject_term(term: Term, role: &str) -> Result<Subject, SemanticActivationError> {
    term_as_subject(&term)
        .ok_or_else(|| SemanticActivationError::new(format!("{role} must identify an RDF node")))
}

fn parse_rdf_list(store: &Store, head: Term) -> Result<Vec<Term>, SemanticActivationError> {
    if matches!(&head, Term::NamedNode(node) if node.as_str() == RDF_NIL) {
        return Ok(Vec::new());
    }
    let mut current = subject_term(head, "sh:in list head")?;
    let mut visited = HashSet::new();
    let mut values = Vec::new();
    loop {
        if !visited.insert(current.clone()) {
            return Err(SemanticActivationError::new("cyclic sh:in RDF list"));
        }
        if visited.len() > MAX_RDF_LIST_ITEMS {
            return Err(SemanticActivationError::new(format!(
                "sh:in list exceeds profile bound {MAX_RDF_LIST_ITEMS}"
            )));
        }
        values.push(exactly_one(store, &current, RDF_FIRST, true)?);
        let rest = exactly_one(store, &current, RDF_REST, true)?;
        if matches!(&rest, Term::NamedNode(node) if node.as_str() == RDF_NIL) {
            break;
        }
        current = subject_term(rest, "rdf:rest")?;
    }
    Ok(values)
}

/// Build the permissive package used only by crate-local regression tests.
#[cfg(test)]
pub(crate) fn activated_test_package() -> ActivatedSemanticPackage {
    use std::sync::OnceLock;
    use tempfile::tempdir;

    static PACKAGE: OnceLock<ActivatedSemanticPackage> = OnceLock::new();
    PACKAGE
        .get_or_init(|| {
            let directory = tempdir().expect("semantic unit-test directory");
            let core = directory.path().join("core.ttl");
            let domain = directory.path().join("domain.ttl");
            let core_shapes = directory.path().join("core-shapes.ttl");
            let domain_shapes = directory.path().join("domain-shapes.ttl");
            let ontology = "@prefix owl: <http://www.w3.org/2002/07/owl#> .\nowl:Thing a owl:Class .\n";
            let shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> .\n@prefix owl: <http://www.w3.org/2002/07/owl#> .\n<urn:provchain:test:AllSubjectsShape> a sh:NodeShape ; sh:targetClass owl:Thing .\n";
            for (path, contents) in [
                (&core, ontology),
                (&domain, ontology),
                (&core_shapes, shapes),
                (&domain_shapes, shapes),
            ] {
                fs::write(path, contents).expect("write semantic unit-test asset");
            }
            let manifest = OntologyPackageManifest {
                package_id: "provchain.unit-test.semantic".to_string(),
                package_version: "1.0.0".to_string(),
                core_ontology_path: core.to_string_lossy().into_owned(),
                domain_ontology_path: domain.to_string_lossy().into_owned(),
                core_shacl_path: core_shapes.to_string_lossy().into_owned(),
                domain_shacl_path: domain_shapes.to_string_lossy().into_owned(),
                mappings: vec![],
                validation_mode: "strict".to_string(),
                semantic_execution_profile_id: SEMANTIC_EXECUTION_PROFILE_V1.to_string(),
                package_hash: None,
            };
            ActivatedSemanticPackage::activate(&manifest)
                .expect("activate semantic unit-test package")
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use oxigraph::model::{GraphName, Literal, NamedNode, Quad};

    use super::{
        activated_test_package, literal_matches_datatype, NodeShape, SemanticAdmissionError,
    };

    #[test]
    fn datatype_lexical_validation_preserves_valid_edge_forms() {
        for (datatype, lexical) in [
            (
                "http://www.w3.org/2001/XMLSchema#integer",
                "+000184467440737095516160",
            ),
            ("http://www.w3.org/2001/XMLSchema#double", "NaN"),
            ("http://www.w3.org/2001/XMLSchema#boolean", "1"),
            (
                "http://www.w3.org/2001/XMLSchema#dateTime",
                "2026-09-01T20:00:00+07:00",
            ),
            ("http://www.w3.org/2001/XMLSchema#string", "line\n\ttwo"),
            ("urn:provchain:test:custom-datatype", "provider-defined"),
        ] {
            let required = NamedNode::new(datatype).expect("datatype IRI");
            let literal = Literal::new_typed_literal(lexical, required.clone());
            assert!(
                literal_matches_datatype(&literal, &required),
                "expected valid lexical form for {datatype}: {lexical:?}"
            );
        }
    }

    #[test]
    fn reasoner_execution_failure_is_node_incapacity() {
        let mut package = activated_test_package();
        package.shapes = std::sync::Arc::new(vec![NodeShape {
            id: "urn:provchain:test:ReasonerFailureShape".to_string(),
            target_classes: vec![
                NamedNode::new("http://example.org/Record").expect("target class IRI")
            ],
            properties: vec![],
        }]);
        package.inject_reasoner_execution_failure_for_test();
        let candidate = vec![Quad::new(
            NamedNode::new("http://example.org/record").expect("subject IRI"),
            NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
                .expect("rdf:type IRI"),
            NamedNode::new("http://example.org/SpecialRecord").expect("class IRI"),
            GraphName::DefaultGraph,
        )];

        assert!(matches!(
            package.validate_transition(&[], &candidate),
            Err(SemanticAdmissionError::NodeIncapacity(reason))
                if reason.contains("fault-injected SPACL execution failure")
        ));
    }
}
