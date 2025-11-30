//! OWL2 Axioms - Logical statements about entities
//!
//! This module defines all OWL2 axiom types that express logical relationships
//! between classes, properties, and individuals.

pub mod class_expressions;
pub mod property_expressions;

pub use crate::entities::{Annotation, AnonymousIndividual, Literal, ObjectProperty};
pub use class_expressions::*;
pub use property_expressions::*;

use crate::iri::IRI;
use crate::{constants::*, OwlError, OwlResult};
use std::sync::Arc;

/// Creates an IRI safely with proper error handling.
///
/// This function attempts to create an optimized IRI from the given string.
/// If the IRI creation fails, it returns an appropriate error.
///
/// # Parameters
/// - `iri_str`: The IRI string to create
///
/// # Returns
/// Returns an `OwlResult` containing the created IRI wrapped in an `Arc`, or an `OwlError::IriCreationError`.
fn create_iri_safe(iri_str: &str) -> OwlResult<Arc<IRI>> {
    IRI::new_optimized(iri_str).map_err(|_| OwlError::IriCreationError {
        iri_str: iri_str.to_string(),
    })
}

/// Creates a blank node IRI safely with proper error handling.
///
/// This function creates an IRI for a blank node by prefixing the node ID
/// with the standard blank node prefix and attempting to create an optimized IRI.
///
/// # Parameters
/// - `node_id`: The identifier for the blank node
///
/// # Returns
/// Returns an `OwlResult` containing the created blank node IRI wrapped in an `Arc`, or an `OwlError::IriCreationError`.
fn create_blank_node_iri(node_id: &str) -> OwlResult<Arc<IRI>> {
    IRI::new_optimized(format!("{}{}", BLANK_NODE_PREFIX, node_id)).map_err(|_| {
        OwlError::IriCreationError {
            iri_str: format!("{}{}", BLANK_NODE_PREFIX, node_id),
        }
    })
}

/// Object value for property assertions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PropertyAssertionObject {
    /// Named individual (IRI)
    Named(Arc<IRI>),
    /// Anonymous individual (blank node)
    Anonymous(Box<AnonymousIndividual>),
}

/// OWL2 Axiom type identifiers for indexing and classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AxiomType {
    SubClassOf,
    EquivalentClasses,
    DisjointClasses,
    ClassAssertion,
    PropertyAssertion,
    DataPropertyAssertion,
    SubObjectProperty,
    EquivalentObjectProperties,
    DisjointObjectProperties,
    FunctionalProperty,
    InverseFunctionalProperty,
    ReflexiveProperty,
    IrreflexiveProperty,
    SymmetricProperty,
    AsymmetricProperty,
    TransitiveProperty,
    SubPropertyChainOf,
    InverseObjectProperties,
    SubDataProperty,
    EquivalentDataProperties,
    DisjointDataProperties,
    FunctionalDataProperty,
    SameIndividual,
    DifferentIndividuals,
    HasKey,
    AnnotationAssertion,
    SubAnnotationPropertyOf,
    AnnotationPropertyDomain,
    AnnotationPropertyRange,
    ObjectMinQualifiedCardinality,
    ObjectMaxQualifiedCardinality,
    ObjectExactQualifiedCardinality,
    DataMinQualifiedCardinality,
    DataMaxQualifiedCardinality,
    DataExactQualifiedCardinality,
    ObjectPropertyDomain,
    ObjectPropertyRange,
    DataPropertyDomain,
    DataPropertyRange,
    NegativeObjectPropertyAssertion,
    NegativeDataPropertyAssertion,
    Import,
    Collection,
    Container,
    Reification,
}

/// OWL2 Axiom types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Axiom {
    /// Subclass axiom: C ⊑ D
    SubClassOf(Box<SubClassOfAxiom>),
    /// Equivalent classes axiom: C ≡ D
    EquivalentClasses(Box<EquivalentClassesAxiom>),
    /// Disjoint classes axiom: C ⊓ D ⊑ ⊥
    DisjointClasses(Box<DisjointClassesAxiom>),
    /// Class assertion: a ∈ C
    ClassAssertion(Box<ClassAssertionAxiom>),
    /// Property assertion: (a, b) ∈ P
    PropertyAssertion(Box<PropertyAssertionAxiom>),
    /// Data property assertion: (a, v) ∈ P where v is a literal
    DataPropertyAssertion(Box<DataPropertyAssertionAxiom>),
    /// Subproperty axiom: P ⊑ Q
    SubObjectProperty(Box<SubObjectPropertyAxiom>),
    /// Equivalent properties axiom: P ≡ Q
    EquivalentObjectProperties(Box<EquivalentObjectPropertiesAxiom>),
    /// Disjoint properties axiom: P ⊓ Q ⊑ ⊥
    DisjointObjectProperties(Box<DisjointObjectPropertiesAxiom>),
    /// Functional property axiom: ⊤ ⊑ ≤1P
    FunctionalProperty(Box<FunctionalPropertyAxiom>),
    /// Inverse functional property axiom: ⊤ ⊑ ≤1P⁻
    InverseFunctionalProperty(Box<InverseFunctionalPropertyAxiom>),
    /// Reflexive property axiom: ⊤ ⊑ ∃P.Self
    ReflexiveProperty(Box<ReflexivePropertyAxiom>),
    /// Irreflexive property axiom: ⊥ ⊑ ∃P.Self
    IrreflexiveProperty(Box<IrreflexivePropertyAxiom>),
    /// Symmetric property axiom: P ≡ P⁻
    SymmetricProperty(Box<SymmetricPropertyAxiom>),
    /// Asymmetric property axiom: P ⊓ P⁻ ⊑ ⊥
    AsymmetricProperty(Box<AsymmetricPropertyAxiom>),
    /// Transitive property axiom: P⁺ ⊑ P
    TransitiveProperty(Box<TransitivePropertyAxiom>),
    /// Property chain axiom: P₁ ∘ ... ∘ Pₙ ⊑ Q
    SubPropertyChainOf(Box<SubPropertyChainOfAxiom>),
    /// Inverse object properties axiom: P ≡ Q⁻
    InverseObjectProperties(Box<InverseObjectPropertiesAxiom>),
    /// Subdata property axiom: Q ⊑ P
    SubDataProperty(Box<SubDataPropertyAxiom>),
    /// Equivalent data properties axiom: P ≡ Q
    EquivalentDataProperties(Box<EquivalentDataPropertiesAxiom>),
    /// Disjoint data properties axiom: P ⊓ Q ⊑ ⊥
    DisjointDataProperties(Box<DisjointDataPropertiesAxiom>),
    /// Functional data property axiom: ⊤ ⊑ ≤1P
    FunctionalDataProperty(FunctionalDataPropertyAxiom),
    /// Same individual axiom: a = b
    SameIndividual(Box<SameIndividualAxiom>),
    /// Different individuals axiom: a ≠ b
    DifferentIndividuals(Box<DifferentIndividualsAxiom>),
    /// Has key axiom: P₁,...,Pₙ ⊑ Key(C)
    HasKey(Box<HasKeyAxiom>),
    /// Annotation assertion axiom: ⊤ ⊑ ∃r.{@a}
    AnnotationAssertion(Box<AnnotationAssertionAxiom>),
    /// Sub-annotation property axiom: P ⊑ Q
    SubAnnotationPropertyOf(SubAnnotationPropertyOfAxiom),
    /// Annotation property domain axiom: ∀P.C ⊑ D
    AnnotationPropertyDomain(AnnotationPropertyDomainAxiom),
    /// Annotation property range axiom: ∀P.C ⊑ D
    AnnotationPropertyRange(AnnotationPropertyRangeAxiom),
    /// Object minimum qualified cardinality: ⊤ ⊑ ≥n R.C
    ObjectMinQualifiedCardinality(Box<ObjectMinQualifiedCardinalityAxiom>),
    /// Object maximum qualified cardinality: ⊤ ⊑ ≤n R.C
    ObjectMaxQualifiedCardinality(Box<ObjectMaxQualifiedCardinalityAxiom>),
    /// Object exact qualified cardinality: ⊤ ⊑ =n R.C
    ObjectExactQualifiedCardinality(Box<ObjectExactQualifiedCardinalityAxiom>),
    /// Data minimum qualified cardinality: ⊤ ⊑ ≥n R.D
    DataMinQualifiedCardinality(Box<DataMinQualifiedCardinalityAxiom>),
    /// Data maximum qualified cardinality: ⊤ ⊑ ≤n R.D
    DataMaxQualifiedCardinality(Box<DataMaxQualifiedCardinalityAxiom>),
    /// Data exact qualified cardinality: ⊤ ⊑ =n R.D
    DataExactQualifiedCardinality(Box<DataExactQualifiedCardinalityAxiom>),
    /// Object property domain: ∀P.C ⊑ D
    ObjectPropertyDomain(Box<ObjectPropertyDomainAxiom>),
    /// Object property range: ∀P.D ⊑ C
    ObjectPropertyRange(Box<ObjectPropertyRangeAxiom>),
    /// Data property domain: ∀Q.C ⊑ D
    DataPropertyDomain(Box<DataPropertyDomainAxiom>),
    /// Data property range: ∃Q.l ⊑ D
    DataPropertyRange(Box<DataPropertyRangeAxiom>),
    /// Negative object property assertion: (a, b) ∉ P
    NegativeObjectPropertyAssertion(Box<NegativeObjectPropertyAssertionAxiom>),
    /// Negative data property assertion: (a, l) ∉ Q
    NegativeDataPropertyAssertion(Box<NegativeDataPropertyAssertionAxiom>),
    /// Import declaration: imports ontology with given IRI
    Import(ImportAxiom),
    /// RDF Collection axiom: represents ordered list using rdf:first, rdf:rest, rdf:nil
    Collection(Box<CollectionAxiom>),
    /// RDF Container axiom: represents Seq, Bag, or Alt containers
    Container(Box<ContainerAxiom>),
    /// RDF Reification axiom: represents statements about statements
    Reification(Box<ReificationAxiom>),
}

impl Axiom {
    /// Get the type of this axiom
    pub fn axiom_type(&self) -> AxiomType {
        // Macro to map axiom variants to their corresponding types
        macro_rules! axiom_type_map {
            ($($variant:ident => $type:ident),*) => {
                match self {
                    $(Axiom::$variant(_) => AxiomType::$type),*
                }
            };
        }

        axiom_type_map! {
            // Class expression axioms
            SubClassOf => SubClassOf,
            EquivalentClasses => EquivalentClasses,
            DisjointClasses => DisjointClasses,
            ClassAssertion => ClassAssertion,

            // Property assertion axioms
            PropertyAssertion => PropertyAssertion,
            DataPropertyAssertion => DataPropertyAssertion,

            // Object property axioms
            SubObjectProperty => SubObjectProperty,
            EquivalentObjectProperties => EquivalentObjectProperties,
            DisjointObjectProperties => DisjointObjectProperties,
            FunctionalProperty => FunctionalProperty,
            InverseFunctionalProperty => InverseFunctionalProperty,
            ReflexiveProperty => ReflexiveProperty,
            IrreflexiveProperty => IrreflexiveProperty,
            SymmetricProperty => SymmetricProperty,
            AsymmetricProperty => AsymmetricProperty,
            TransitiveProperty => TransitiveProperty,
            SubPropertyChainOf => SubPropertyChainOf,
            InverseObjectProperties => InverseObjectProperties,
            ObjectPropertyDomain => ObjectPropertyDomain,
            ObjectPropertyRange => ObjectPropertyRange,
            NegativeObjectPropertyAssertion => NegativeObjectPropertyAssertion,

            // Data property axioms
            SubDataProperty => SubDataProperty,
            EquivalentDataProperties => EquivalentDataProperties,
            DisjointDataProperties => DisjointDataProperties,
            FunctionalDataProperty => FunctionalDataProperty,
            DataPropertyDomain => DataPropertyDomain,
            DataPropertyRange => DataPropertyRange,
            NegativeDataPropertyAssertion => NegativeDataPropertyAssertion,

            // Individual axioms
            SameIndividual => SameIndividual,
            DifferentIndividuals => DifferentIndividuals,

            // Cardinality axioms
            HasKey => HasKey,
            ObjectMinQualifiedCardinality => ObjectMinQualifiedCardinality,
            ObjectMaxQualifiedCardinality => ObjectMaxQualifiedCardinality,
            ObjectExactQualifiedCardinality => ObjectExactQualifiedCardinality,
            DataMinQualifiedCardinality => DataMinQualifiedCardinality,
            DataMaxQualifiedCardinality => DataMaxQualifiedCardinality,
            DataExactQualifiedCardinality => DataExactQualifiedCardinality,

            // Annotation axioms
            AnnotationAssertion => AnnotationAssertion,
            SubAnnotationPropertyOf => SubAnnotationPropertyOf,
            AnnotationPropertyDomain => AnnotationPropertyDomain,
            AnnotationPropertyRange => AnnotationPropertyRange,

            // Special axioms
            Import => Import,
            Collection => Collection,
            Container => Container,
            Reification => Reification
        }
    }

    /// Get the signature IRIs of this axiom (main entities involved)
    pub fn signature(&self) -> Vec<Arc<IRI>> {
        // Simplified signature extraction - will be enhanced with proper axiom methods
        Vec::new() // Placeholder implementation
    }
}

/// Subclass axiom: C ⊑ D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubClassOfAxiom {
    sub_class: class_expressions::ClassExpression,
    super_class: class_expressions::ClassExpression,
}

impl SubClassOfAxiom {
    /// Create a new subclass axiom
    pub fn new(
        sub_class: class_expressions::ClassExpression,
        super_class: class_expressions::ClassExpression,
    ) -> Self {
        SubClassOfAxiom {
            sub_class,
            super_class,
        }
    }

    /// Get the subclass
    pub fn sub_class(&self) -> &class_expressions::ClassExpression {
        &self.sub_class
    }

    /// Get the superclass
    pub fn super_class(&self) -> &class_expressions::ClassExpression {
        &self.super_class
    }

    /// Check if this axiom involves a specific class
    pub fn involves_class(&self, class_iri: &IRI) -> bool {
        self.sub_class.contains_class(class_iri) || self.super_class.contains_class(class_iri)
    }
}

/// Equivalent classes axiom: C ≡ D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquivalentClassesAxiom {
    classes: Vec<Arc<IRI>>,
}

impl EquivalentClassesAxiom {
    /// Create a new equivalent classes axiom
    pub fn new(classes: Vec<Arc<IRI>>) -> Self {
        EquivalentClassesAxiom { classes }
    }

    /// Get the equivalent classes
    pub fn classes(&self) -> &Vec<Arc<IRI>> {
        &self.classes
    }
}

/// Disjoint classes axiom: C ⊓ D ⊑ ⊥
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisjointClassesAxiom {
    classes: Vec<Arc<IRI>>,
}

impl DisjointClassesAxiom {
    /// Create a new disjoint classes axiom
    pub fn new(classes: Vec<Arc<IRI>>) -> Self {
        DisjointClassesAxiom { classes }
    }

    /// Get the disjoint classes
    pub fn classes(&self) -> &Vec<Arc<IRI>> {
        &self.classes
    }
}

/// Class assertion axiom: a ∈ C
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassAssertionAxiom {
    individual: Arc<IRI>,
    class_expr: class_expressions::ClassExpression,
}

impl ClassAssertionAxiom {
    /// Create a new class assertion axiom
    pub fn new(individual: Arc<IRI>, class_expr: class_expressions::ClassExpression) -> Self {
        ClassAssertionAxiom {
            individual,
            class_expr,
        }
    }

    /// Get the individual
    pub fn individual(&self) -> &Arc<IRI> {
        &self.individual
    }

    /// Get the class expression
    pub fn class_expr(&self) -> &class_expressions::ClassExpression {
        &self.class_expr
    }
}

/// Property assertion axiom: (a, b) ∈ P
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyAssertionAxiom {
    subject: Arc<IRI>,
    property: Arc<IRI>,
    object: PropertyAssertionObject,
}

/// Data property assertion axiom: (a, v) ∈ P
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPropertyAssertionAxiom {
    subject: Arc<IRI>,
    property: Arc<IRI>,
    value: crate::entities::Literal,
}

impl DataPropertyAssertionAxiom {
    /// Create a new data property assertion axiom
    pub fn new(subject: Arc<IRI>, property: Arc<IRI>, value: crate::entities::Literal) -> Self {
        DataPropertyAssertionAxiom {
            subject,
            property,
            value,
        }
    }

    /// Get the subject
    pub fn subject(&self) -> &Arc<IRI> {
        &self.subject
    }

    /// Get the property
    pub fn property(&self) -> &Arc<IRI> {
        &self.property
    }

    /// Get the literal value
    pub fn value(&self) -> &crate::entities::Literal {
        &self.value
    }
}

impl PropertyAssertionAxiom {
    /// Create a new property assertion axiom with named individual (backward compatibility)
    pub fn new(subject: Arc<IRI>, property: Arc<IRI>, object: Arc<IRI>) -> Self {
        PropertyAssertionAxiom {
            subject,
            property,
            object: PropertyAssertionObject::Named(object),
        }
    }

    /// Create a new property assertion axiom with anonymous individual (blank node)
    pub fn new_with_anonymous(
        subject: Arc<IRI>,
        property: Arc<IRI>,
        object: AnonymousIndividual,
    ) -> Self {
        PropertyAssertionAxiom {
            subject,
            property,
            object: PropertyAssertionObject::Anonymous(Box::new(object)),
        }
    }

    /// Create a new property assertion axiom with property assertion object
    pub fn new_with_object(
        subject: Arc<IRI>,
        property: Arc<IRI>,
        object: PropertyAssertionObject,
    ) -> Self {
        PropertyAssertionAxiom {
            subject,
            property,
            object,
        }
    }

    /// Get the subject
    pub fn subject(&self) -> &Arc<IRI> {
        &self.subject
    }

    /// Get the property
    pub fn property(&self) -> &Arc<IRI> {
        &self.property
    }

    /// Get the object
    pub fn object(&self) -> &PropertyAssertionObject {
        &self.object
    }

    /// Get the object as IRI if it's a named individual, returns None for anonymous
    pub fn object_iri(&self) -> Option<&Arc<IRI>> {
        match &self.object {
            PropertyAssertionObject::Named(iri) => Some(iri),
            PropertyAssertionObject::Anonymous(_) => None,
        }
    }

    /// Get the object as anonymous individual if it's anonymous, returns None for named
    pub fn object_anonymous(&self) -> Option<&AnonymousIndividual> {
        match &self.object {
            PropertyAssertionObject::Named(_) => None,
            PropertyAssertionObject::Anonymous(individual) => Some(&**individual),
        }
    }
}

/// Subobject property axiom: P ⊑ Q
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubObjectPropertyAxiom {
    sub_property: Arc<IRI>,
    super_property: Arc<IRI>,
}

impl SubObjectPropertyAxiom {
    /// Create a new subobject property axiom
    pub fn new(sub_property: Arc<IRI>, super_property: Arc<IRI>) -> Self {
        SubObjectPropertyAxiom {
            sub_property,
            super_property,
        }
    }

    /// Get the subproperty
    pub fn sub_property(&self) -> &Arc<IRI> {
        &self.sub_property
    }

    /// Get the superproperty
    pub fn super_property(&self) -> &Arc<IRI> {
        &self.super_property
    }
}

/// Equivalent object properties axiom: P ≡ Q
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquivalentObjectPropertiesAxiom {
    properties: Vec<Arc<IRI>>,
}

impl EquivalentObjectPropertiesAxiom {
    /// Create a new equivalent object properties axiom
    pub fn new(properties: Vec<Arc<IRI>>) -> Self {
        EquivalentObjectPropertiesAxiom { properties }
    }

    /// Get the equivalent properties
    pub fn properties(&self) -> &Vec<Arc<IRI>> {
        &self.properties
    }
}

/// Disjoint object properties axiom: P ⊓ Q ⊑ ⊥
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisjointObjectPropertiesAxiom {
    properties: Vec<Arc<IRI>>,
}

impl DisjointObjectPropertiesAxiom {
    /// Create a new disjoint object properties axiom
    pub fn new(properties: Vec<Arc<IRI>>) -> Self {
        DisjointObjectPropertiesAxiom { properties }
    }

    /// Get the disjoint properties
    pub fn properties(&self) -> &Vec<Arc<IRI>> {
        &self.properties
    }
}

/// Functional property axiom: ⊤ ⊑ ≤1P
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionalPropertyAxiom {
    property: Arc<IRI>,
}

impl FunctionalPropertyAxiom {
    /// Create a new functional property axiom
    pub fn new(property: Arc<IRI>) -> Self {
        FunctionalPropertyAxiom { property }
    }

    /// Get the property
    pub fn property(&self) -> &Arc<IRI> {
        &self.property
    }
}

/// Inverse functional property axiom: ⊤ ⊑ ≤1P⁻
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InverseFunctionalPropertyAxiom {
    property: Arc<IRI>,
}

impl InverseFunctionalPropertyAxiom {
    /// Create a new inverse functional property axiom
    pub fn new(property: Arc<IRI>) -> Self {
        InverseFunctionalPropertyAxiom { property }
    }

    /// Get the property
    pub fn property(&self) -> &Arc<IRI> {
        &self.property
    }
}

/// Reflexive property axiom: ⊤ ⊑ ∃P.Self
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflexivePropertyAxiom {
    property: Arc<IRI>,
}

impl ReflexivePropertyAxiom {
    /// Create a new reflexive property axiom
    pub fn new(property: Arc<IRI>) -> Self {
        ReflexivePropertyAxiom { property }
    }

    /// Get the property
    pub fn property(&self) -> &Arc<IRI> {
        &self.property
    }
}

/// Irreflexive property axiom: ⊥ ⊑ ∃P.Self
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrreflexivePropertyAxiom {
    property: Arc<IRI>,
}

impl IrreflexivePropertyAxiom {
    /// Create a new irreflexive property axiom
    pub fn new(property: Arc<IRI>) -> Self {
        IrreflexivePropertyAxiom { property }
    }

    /// Get the property
    pub fn property(&self) -> &Arc<IRI> {
        &self.property
    }
}

/// Symmetric property axiom: P ≡ P⁻
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymmetricPropertyAxiom {
    property: Arc<IRI>,
}

impl SymmetricPropertyAxiom {
    /// Create a new symmetric property axiom
    pub fn new(property: Arc<IRI>) -> Self {
        SymmetricPropertyAxiom { property }
    }

    /// Get the property
    pub fn property(&self) -> &Arc<IRI> {
        &self.property
    }
}

/// Asymmetric property axiom: P ⊓ P⁻ ⊑ ⊥
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsymmetricPropertyAxiom {
    property: Arc<IRI>,
}

impl AsymmetricPropertyAxiom {
    /// Create a new asymmetric property axiom
    pub fn new(property: Arc<IRI>) -> Self {
        AsymmetricPropertyAxiom { property }
    }

    /// Get the property
    pub fn property(&self) -> &Arc<IRI> {
        &self.property
    }
}

/// Transitive property axiom: P⁺ ⊑ P
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitivePropertyAxiom {
    property: Arc<IRI>,
}

impl TransitivePropertyAxiom {
    /// Create a new transitive property axiom
    pub fn new(property: Arc<IRI>) -> Self {
        TransitivePropertyAxiom { property }
    }

    /// Get the property
    pub fn property(&self) -> &Arc<IRI> {
        &self.property
    }
}

/// Property chain axiom: P₁ ∘ ... ∘ Pₙ ⊑ Q
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubPropertyChainOfAxiom {
    property_chain: Vec<ObjectPropertyExpression>,
    super_property: ObjectPropertyExpression,
}

impl SubPropertyChainOfAxiom {
    /// Create a new property chain axiom
    pub fn new(
        property_chain: Vec<ObjectPropertyExpression>,
        super_property: ObjectPropertyExpression,
    ) -> Self {
        SubPropertyChainOfAxiom {
            property_chain,
            super_property,
        }
    }

    /// Get the property chain
    pub fn property_chain(&self) -> &[ObjectPropertyExpression] {
        &self.property_chain
    }

    /// Get the super property
    pub fn super_property(&self) -> &ObjectPropertyExpression {
        &self.super_property
    }
}

/// Inverse object properties axiom: P ≡ Q⁻
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InverseObjectPropertiesAxiom {
    property1: ObjectPropertyExpression,
    property2: ObjectPropertyExpression,
}

impl InverseObjectPropertiesAxiom {
    /// Create a new inverse object properties axiom
    pub fn new(property1: ObjectPropertyExpression, property2: ObjectPropertyExpression) -> Self {
        InverseObjectPropertiesAxiom {
            property1,
            property2,
        }
    }

    /// Get the first property
    pub fn property1(&self) -> &ObjectPropertyExpression {
        &self.property1
    }

    /// Get the second property
    pub fn property2(&self) -> &ObjectPropertyExpression {
        &self.property2
    }
}

/// Subdata property axiom: Q ⊑ P
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubDataPropertyAxiom {
    sub_property: Arc<IRI>,
    super_property: Arc<IRI>,
}

impl SubDataPropertyAxiom {
    /// Create a new subdata property axiom
    pub fn new(sub_property: Arc<IRI>, super_property: Arc<IRI>) -> Self {
        SubDataPropertyAxiom {
            sub_property,
            super_property,
        }
    }

    /// Get the subproperty
    pub fn sub_property(&self) -> &Arc<IRI> {
        &self.sub_property
    }

    /// Get the superproperty
    pub fn super_property(&self) -> &Arc<IRI> {
        &self.super_property
    }
}

/// Equivalent data properties axiom: P ≡ Q
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquivalentDataPropertiesAxiom {
    properties: Vec<Arc<IRI>>,
}

impl EquivalentDataPropertiesAxiom {
    /// Create a new equivalent data properties axiom
    pub fn new(properties: Vec<Arc<IRI>>) -> Self {
        EquivalentDataPropertiesAxiom { properties }
    }

    /// Get the equivalent properties
    pub fn properties(&self) -> &Vec<Arc<IRI>> {
        &self.properties
    }
}

/// Disjoint data properties axiom: P ⊓ Q ⊑ ⊥
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisjointDataPropertiesAxiom {
    properties: Vec<Arc<IRI>>,
}

impl DisjointDataPropertiesAxiom {
    /// Create a new disjoint data properties axiom
    pub fn new(properties: Vec<Arc<IRI>>) -> Self {
        DisjointDataPropertiesAxiom { properties }
    }

    /// Get the disjoint properties
    pub fn properties(&self) -> &Vec<Arc<IRI>> {
        &self.properties
    }
}

/// Functional data property axiom: ⊤ ⊑ ≤1P
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionalDataPropertyAxiom {
    property: Arc<IRI>,
}

impl FunctionalDataPropertyAxiom {
    /// Create a new functional data property axiom
    pub fn new(property: Arc<IRI>) -> Self {
        FunctionalDataPropertyAxiom { property }
    }

    /// Get the property
    pub fn property(&self) -> &Arc<IRI> {
        &self.property
    }
}

/// Same individual axiom: a = b
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SameIndividualAxiom {
    individuals: Vec<Arc<IRI>>,
}

impl SameIndividualAxiom {
    /// Create a new same individual axiom
    pub fn new(individuals: Vec<Arc<IRI>>) -> Self {
        SameIndividualAxiom { individuals }
    }

    /// Get the individuals
    pub fn individuals(&self) -> &[Arc<IRI>] {
        &self.individuals
    }
}

/// Different individuals axiom: a ≠ b
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DifferentIndividualsAxiom {
    individuals: Vec<Arc<IRI>>,
}

impl DifferentIndividualsAxiom {
    /// Create a new different individuals axiom
    pub fn new(individuals: Vec<Arc<IRI>>) -> Self {
        DifferentIndividualsAxiom { individuals }
    }

    /// Get the individuals
    pub fn individuals(&self) -> &[Arc<IRI>] {
        &self.individuals
    }
}

/// Has key axiom: P₁,...,Pₙ ⊑ Key(C)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HasKeyAxiom {
    class_expression: class_expressions::ClassExpression,
    properties: Vec<Arc<IRI>>,
}

impl HasKeyAxiom {
    /// Create a new has key axiom
    pub fn new(
        class_expression: class_expressions::ClassExpression,
        properties: Vec<Arc<IRI>>,
    ) -> Self {
        HasKeyAxiom {
            class_expression,
            properties,
        }
    }

    /// Get the class expression
    pub fn class_expression(&self) -> &class_expressions::ClassExpression {
        &self.class_expression
    }

    /// Get the properties
    pub fn properties(&self) -> &[Arc<IRI>] {
        &self.properties
    }
}

/// Annotation assertion axiom: ⊤ ⊑ ∃r.{@a}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationAssertionAxiom {
    annotation_property: Arc<IRI>,
    subject: Arc<IRI>,
    value: crate::entities::AnnotationValue,
}

impl AnnotationAssertionAxiom {
    /// Create a new annotation assertion axiom
    pub fn new(
        annotation_property: Arc<IRI>,
        subject: Arc<IRI>,
        value: crate::entities::AnnotationValue,
    ) -> Self {
        AnnotationAssertionAxiom {
            annotation_property,
            subject,
            value,
        }
    }

    /// Get the annotation property
    pub fn annotation_property(&self) -> &Arc<IRI> {
        &self.annotation_property
    }

    /// Get the subject
    pub fn subject(&self) -> &Arc<IRI> {
        &self.subject
    }

    /// Get the annotation value
    pub fn value(&self) -> &crate::entities::AnnotationValue {
        &self.value
    }
}

/// Sub-annotation property axiom: P ⊑ Q
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubAnnotationPropertyOfAxiom {
    sub_property: Arc<IRI>,
    super_property: Arc<IRI>,
}

impl SubAnnotationPropertyOfAxiom {
    /// Create a new sub-annotation property axiom
    pub fn new(sub_property: Arc<IRI>, super_property: Arc<IRI>) -> Self {
        SubAnnotationPropertyOfAxiom {
            sub_property,
            super_property,
        }
    }

    /// Get the sub-property
    pub fn sub_property(&self) -> &Arc<IRI> {
        &self.sub_property
    }

    /// Get the super-property
    pub fn super_property(&self) -> &Arc<IRI> {
        &self.super_property
    }
}

/// Annotation property domain axiom: ∀P.C ⊑ D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationPropertyDomainAxiom {
    property: Arc<IRI>,
    domain: Arc<IRI>,
}

impl AnnotationPropertyDomainAxiom {
    /// Create a new annotation property domain axiom
    pub fn new(property: Arc<IRI>, domain: Arc<IRI>) -> Self {
        AnnotationPropertyDomainAxiom { property, domain }
    }

    /// Get the property
    pub fn property(&self) -> &Arc<IRI> {
        &self.property
    }

    /// Get the domain
    pub fn domain(&self) -> &Arc<IRI> {
        &self.domain
    }
}

/// Annotation property range axiom: ∀P.C ⊑ D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationPropertyRangeAxiom {
    property: Arc<IRI>,
    range: Arc<IRI>,
}

impl AnnotationPropertyRangeAxiom {
    /// Create a new annotation property range axiom
    pub fn new(property: Arc<IRI>, range: Arc<IRI>) -> Self {
        AnnotationPropertyRangeAxiom { property, range }
    }

    /// Get the property
    pub fn property(&self) -> &Arc<IRI> {
        &self.property
    }

    /// Get the range
    pub fn range(&self) -> &Arc<IRI> {
        &self.range
    }
}

/// Object minimum qualified cardinality axiom: ⊤ ⊑ ≥n R.C
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectMinQualifiedCardinalityAxiom {
    cardinality: u32,
    property: ObjectPropertyExpression,
    filler: class_expressions::ClassExpression,
}

impl ObjectMinQualifiedCardinalityAxiom {
    /// Create a new object minimum qualified cardinality axiom
    pub fn new(
        cardinality: u32,
        property: ObjectPropertyExpression,
        filler: class_expressions::ClassExpression,
    ) -> Self {
        Self {
            cardinality,
            property,
            filler,
        }
    }

    /// Get the cardinality
    pub fn cardinality(&self) -> u32 {
        self.cardinality
    }

    /// Get the property
    pub fn property(&self) -> &ObjectPropertyExpression {
        &self.property
    }

    /// Get the filler class expression
    pub fn filler(&self) -> &class_expressions::ClassExpression {
        &self.filler
    }
}

/// Object maximum qualified cardinality axiom: ⊤ ⊑ ≤n R.C
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectMaxQualifiedCardinalityAxiom {
    cardinality: u32,
    property: ObjectPropertyExpression,
    filler: class_expressions::ClassExpression,
}

impl ObjectMaxQualifiedCardinalityAxiom {
    /// Create a new object maximum qualified cardinality axiom
    pub fn new(
        cardinality: u32,
        property: ObjectPropertyExpression,
        filler: class_expressions::ClassExpression,
    ) -> Self {
        Self {
            cardinality,
            property,
            filler,
        }
    }

    /// Get the cardinality
    pub fn cardinality(&self) -> u32 {
        self.cardinality
    }

    /// Get the property
    pub fn property(&self) -> &ObjectPropertyExpression {
        &self.property
    }

    /// Get the filler class expression
    pub fn filler(&self) -> &class_expressions::ClassExpression {
        &self.filler
    }
}

/// Object exact qualified cardinality axiom: ⊤ ⊑ =n R.C
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectExactQualifiedCardinalityAxiom {
    cardinality: u32,
    property: ObjectPropertyExpression,
    filler: class_expressions::ClassExpression,
}

impl ObjectExactQualifiedCardinalityAxiom {
    /// Create a new object exact qualified cardinality axiom
    pub fn new(
        cardinality: u32,
        property: ObjectPropertyExpression,
        filler: class_expressions::ClassExpression,
    ) -> Self {
        Self {
            cardinality,
            property,
            filler,
        }
    }

    /// Get the cardinality
    pub fn cardinality(&self) -> u32 {
        self.cardinality
    }

    /// Get the property
    pub fn property(&self) -> &ObjectPropertyExpression {
        &self.property
    }

    /// Get the filler class expression
    pub fn filler(&self) -> &class_expressions::ClassExpression {
        &self.filler
    }
}

/// Data minimum qualified cardinality axiom: ⊤ ⊑ ≥n R.D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataMinQualifiedCardinalityAxiom {
    cardinality: u32,
    property: ObjectPropertyExpression,
    filler: Arc<IRI>,
}

impl DataMinQualifiedCardinalityAxiom {
    /// Create a new data minimum qualified cardinality axiom
    pub fn new(cardinality: u32, property: ObjectPropertyExpression, filler: Arc<IRI>) -> Self {
        Self {
            cardinality,
            property,
            filler,
        }
    }

    /// Get the cardinality
    pub fn cardinality(&self) -> u32 {
        self.cardinality
    }

    /// Get the property
    pub fn property(&self) -> &ObjectPropertyExpression {
        &self.property
    }

    /// Get the filler datatype IRI
    pub fn filler(&self) -> &Arc<IRI> {
        &self.filler
    }
}

/// Data maximum qualified cardinality axiom: ⊤ ⊑ ≤n R.D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataMaxQualifiedCardinalityAxiom {
    cardinality: u32,
    property: ObjectPropertyExpression,
    filler: Arc<IRI>,
}

impl DataMaxQualifiedCardinalityAxiom {
    /// Create a new data maximum qualified cardinality axiom
    pub fn new(cardinality: u32, property: ObjectPropertyExpression, filler: Arc<IRI>) -> Self {
        Self {
            cardinality,
            property,
            filler,
        }
    }

    /// Get the cardinality
    pub fn cardinality(&self) -> u32 {
        self.cardinality
    }

    /// Get the property
    pub fn property(&self) -> &ObjectPropertyExpression {
        &self.property
    }

    /// Get the filler datatype IRI
    pub fn filler(&self) -> &Arc<IRI> {
        &self.filler
    }
}

/// Data exact qualified cardinality axiom: ⊤ ⊑ =n R.D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataExactQualifiedCardinalityAxiom {
    cardinality: u32,
    property: ObjectPropertyExpression,
    filler: Arc<IRI>,
}

impl DataExactQualifiedCardinalityAxiom {
    /// Create a new data exact qualified cardinality axiom
    pub fn new(cardinality: u32, property: ObjectPropertyExpression, filler: Arc<IRI>) -> Self {
        Self {
            cardinality,
            property,
            filler,
        }
    }

    /// Get the cardinality
    pub fn cardinality(&self) -> u32 {
        self.cardinality
    }

    /// Get the property
    pub fn property(&self) -> &ObjectPropertyExpression {
        &self.property
    }

    /// Get the filler datatype IRI
    pub fn filler(&self) -> &Arc<IRI> {
        &self.filler
    }
}

/// Object property domain axiom: ∀P.C ⊑ D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectPropertyDomainAxiom {
    property: Arc<IRI>,
    domain: class_expressions::ClassExpression,
}

impl ObjectPropertyDomainAxiom {
    /// Create a new object property domain axiom
    pub fn new(property: Arc<IRI>, domain: class_expressions::ClassExpression) -> Self {
        ObjectPropertyDomainAxiom { property, domain }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }

    /// Get the domain class expression
    pub fn domain(&self) -> &class_expressions::ClassExpression {
        &self.domain
    }
}

/// Object property range axiom: ∀P.D ⊑ C
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectPropertyRangeAxiom {
    property: IRI,
    range: class_expressions::ClassExpression,
}

impl ObjectPropertyRangeAxiom {
    /// Create a new object property range axiom
    pub fn new(property: IRI, range: class_expressions::ClassExpression) -> Self {
        ObjectPropertyRangeAxiom { property, range }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }

    /// Get the range class expression
    pub fn range(&self) -> &class_expressions::ClassExpression {
        &self.range
    }
}

/// Data property domain axiom: ∀Q.C ⊑ D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPropertyDomainAxiom {
    property: IRI,
    domain: class_expressions::ClassExpression,
}

impl DataPropertyDomainAxiom {
    /// Create a new data property domain axiom
    pub fn new(property: IRI, domain: class_expressions::ClassExpression) -> Self {
        DataPropertyDomainAxiom { property, domain }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }

    /// Get the domain class expression
    pub fn domain(&self) -> &class_expressions::ClassExpression {
        &self.domain
    }
}

/// Data property range axiom: ∃Q.l ⊑ D
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPropertyRangeAxiom {
    property: IRI,
    range: IRI,
}

impl DataPropertyRangeAxiom {
    /// Create a new data property range axiom
    pub fn new(property: IRI, range: IRI) -> Self {
        DataPropertyRangeAxiom { property, range }
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }

    /// Get the range datatype
    pub fn range(&self) -> &IRI {
        &self.range
    }
}

/// Negative object property assertion axiom: (a, b) ∉ P
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegativeObjectPropertyAssertionAxiom {
    subject: IRI,
    property: IRI,
    object: IRI,
}

impl NegativeObjectPropertyAssertionAxiom {
    /// Create a new negative object property assertion axiom
    pub fn new(subject: IRI, property: IRI, object: IRI) -> Self {
        NegativeObjectPropertyAssertionAxiom {
            subject,
            property,
            object,
        }
    }

    /// Get the subject individual
    pub fn subject(&self) -> &IRI {
        &self.subject
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }

    /// Get the object individual
    pub fn object(&self) -> &IRI {
        &self.object
    }
}

/// Negative data property assertion axiom: (a, l) ∉ Q
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegativeDataPropertyAssertionAxiom {
    subject: IRI,
    property: IRI,
    value: crate::entities::Literal,
}

impl NegativeDataPropertyAssertionAxiom {
    /// Create a new negative data property assertion axiom
    pub fn new(subject: IRI, property: IRI, value: crate::entities::Literal) -> Self {
        NegativeDataPropertyAssertionAxiom {
            subject,
            property,
            value,
        }
    }

    /// Get the subject individual
    pub fn subject(&self) -> &IRI {
        &self.subject
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }

    /// Get the negated literal value
    pub fn value(&self) -> &crate::entities::Literal {
        &self.value
    }
}

/// Import axiom: imports ontology with given IRI
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportAxiom {
    imported_ontology: Arc<IRI>,
}

impl ImportAxiom {
    /// Create a new import axiom
    pub fn new(imported_ontology: Arc<IRI>) -> Self {
        ImportAxiom { imported_ontology }
    }

    /// Get the imported ontology IRI
    pub fn imported_ontology(&self) -> &Arc<IRI> {
        &self.imported_ontology
    }
}

/// RDF Collection axiom representing ordered lists using rdf:first, rdf:rest, rdf:nil
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionAxiom {
    /// The subject that has the collection
    subject: Arc<IRI>,
    /// The property that relates the subject to the collection
    property: Arc<IRI>,
    /// The list of items in the collection
    items: Vec<CollectionItem>,
}

/// Individual item in a collection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectionItem {
    Named(Arc<IRI>),
    Anonymous(Box<AnonymousIndividual>),
    Literal(Literal),
}

impl CollectionAxiom {
    /// Create a new collection axiom
    pub fn new(subject: Arc<IRI>, property: Arc<IRI>, items: Vec<CollectionItem>) -> Self {
        CollectionAxiom {
            subject,
            property,
            items,
        }
    }

    /// Get the subject
    pub fn subject(&self) -> &Arc<IRI> {
        &self.subject
    }

    /// Get the property
    pub fn property(&self) -> &Arc<IRI> {
        &self.property
    }

    /// Get the items
    pub fn items(&self) -> &Vec<CollectionItem> {
        &self.items
    }

    /// Get the number of items in the collection
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Create property assertions from the collection
    pub fn to_property_assertions(&self) -> OwlResult<Vec<PropertyAssertionAxiom>> {
        let mut assertions = Vec::new();

        // Create a blank node for each collection node
        let _previous_node: Option<AnonymousIndividual> = None;

        // Process items in reverse order to build the linked list
        for (index, item) in self.items.iter().enumerate() {
            let node_id = format!(
                "{}_{}",
                self.subject
                    .as_str()
                    .replace("http://", "")
                    .replace("/", "_"),
                index
            );
            let _anon_node = AnonymousIndividual::new(&node_id);

            // Add rdf:first assertion
            let first_assertion = match item {
                CollectionItem::Named(iri) => PropertyAssertionAxiom::new(
                    create_blank_node_iri(&node_id)?,
                    IRI::new_optimized(rdf::first().as_str())?,
                    IRI::new_optimized(iri.as_str())?,
                ),
                CollectionItem::Anonymous(anon) => PropertyAssertionAxiom::new_with_anonymous(
                    create_blank_node_iri(&node_id)?,
                    IRI::new_optimized(rdf::first().as_str())?,
                    *(*anon).clone(),
                ),
                CollectionItem::Literal(_lit) => {
                    // For literals, we'd need to create a data property assertion
                    // This is a simplified version
                    PropertyAssertionAxiom::new(
                        create_blank_node_iri(&node_id)?,
                        IRI::new_optimized(rdf::first().as_str())?,
                        IRI::new_optimized("http://test.org/literal")?, // placeholder
                    )
                }
            };

            // Add rdf:rest assertion
            let rest_assertion = if index == self.items.len() - 1 {
                // Last item points to rdf:nil
                PropertyAssertionAxiom::new(
                    create_blank_node_iri(&node_id)?,
                    IRI::new_optimized(rdf::rest().as_str())?,
                    IRI::new_optimized(rdf::nil().as_str())?,
                )
            } else {
                // Points to next node
                let next_node_id = format!(
                    "{}_{}",
                    self.subject
                        .as_str()
                        .replace("http://", "")
                        .replace("/", "_"),
                    index + 1
                );
                PropertyAssertionAxiom::new(
                    create_blank_node_iri(&node_id)?,
                    IRI::new_optimized(rdf::rest().as_str())?,
                    create_blank_node_iri(&next_node_id)?,
                )
            };

            assertions.push(first_assertion);
            assertions.push(rest_assertion);

            // If this is the first item, connect it to the subject
            if index == 0 {
                let subject_assertion = PropertyAssertionAxiom::new(
                    self.subject.clone(),
                    self.property.clone(),
                    create_blank_node_iri(&node_id)?,
                );
                assertions.insert(0, subject_assertion);
            }

            // _previous_node is tracked but not used in current implementation
            // previous_node = Some(anon_node);
        }

        Ok(assertions)
    }
}

/// RDF Container types (Seq, Bag, Alt)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContainerType {
    /// Ordered sequence (rdf:Seq)
    Sequence,
    /// Unordered bag (rdf:Bag)
    Bag,
    /// Alternative (rdf:Alt)
    Alternative,
}

/// RDF Container axiom: represents Seq, Bag, or Alt containers
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerAxiom {
    /// The subject that has the container
    subject: IRI,
    /// The property that relates the subject to the container
    property: IRI,
    /// The type of container (Seq, Bag, Alt)
    container_type: ContainerType,
    /// The list of items in the container
    items: Vec<ContainerItem>,
}

/// Individual item in a container
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerItem {
    Named(IRI),
    Anonymous(Box<AnonymousIndividual>),
    Literal(Literal),
}

impl ContainerAxiom {
    /// Create a new container axiom
    pub fn new(
        subject: IRI,
        property: IRI,
        container_type: ContainerType,
        items: Vec<ContainerItem>,
    ) -> Self {
        ContainerAxiom {
            subject,
            property,
            container_type,
            items,
        }
    }

    /// Get the subject
    pub fn subject(&self) -> &IRI {
        &self.subject
    }

    /// Get the property
    pub fn property(&self) -> &IRI {
        &self.property
    }

    /// Get the container type
    pub fn container_type(&self) -> ContainerType {
        self.container_type
    }

    /// Get the items
    pub fn items(&self) -> &Vec<ContainerItem> {
        &self.items
    }

    /// Get the number of items in the container
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the container is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Create property assertions from the container
    pub fn to_property_assertions(&self) -> OwlResult<Vec<PropertyAssertionAxiom>> {
        let mut assertions = Vec::new();

        // Create the container resource
        let container_id = format!(
            "{}_container",
            self.subject
                .as_str()
                .replace("http://", "")
                .replace("/", "_")
        );
        let container_iri = create_blank_node_iri(&container_id)?;

        // Connect subject to container
        let subject_to_container = PropertyAssertionAxiom::new(
            IRI::new_optimized(self.subject.as_str())?,
            IRI::new_optimized(self.property.as_str())?,
            container_iri.clone(),
        );
        assertions.push(subject_to_container);

        // Add type assertion for the container
        let type_property = rdf::type_property();

        let type_value = match self.container_type {
            ContainerType::Sequence => rdf::seq(),
            ContainerType::Bag => rdf::bag(),
            ContainerType::Alternative => rdf::alt(),
        };

        let type_assertion = PropertyAssertionAxiom::new(
            container_iri.clone(),
            IRI::new_optimized(type_property.as_str())?,
            IRI::new_optimized(type_value.as_str())?,
        );
        assertions.push(type_assertion);

        // Add numbered elements (rdf:_1, rdf:_2, etc.)
        for (index, item) in self.items.iter().enumerate() {
            let element_property =
                create_iri_safe(&format!("{}{}", RDF_ELEMENT_PREFIX, index + 1))?;

            let element_assertion = match item {
                ContainerItem::Named(iri) => PropertyAssertionAxiom::new(
                    container_iri.clone(),
                    element_property,
                    IRI::new_optimized(iri.as_str())?,
                ),
                ContainerItem::Anonymous(anon) => PropertyAssertionAxiom::new(
                    container_iri.clone(),
                    element_property,
                    create_blank_node_iri(anon.node_id())?,
                ),
                ContainerItem::Literal(_lit) => {
                    // For literals, we need to use a data property assertion
                    // This is a simplification - in practice, containers typically use IRIs
                    PropertyAssertionAxiom::new(
                        container_iri.clone(),
                        element_property,
                        create_blank_node_iri(&format!("literal_{}", index))?,
                    )
                }
            };
            assertions.push(element_assertion);
        }

        Ok(assertions)
    }
}

/// RDF Reification axiom: represents statements about statements using rdf:subject, rdf:predicate, rdf:object
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReificationAxiom {
    /// The reified statement resource (blank node or named resource)
    reification_resource: Arc<IRI>,
    /// The subject of the original statement
    subject: Arc<IRI>,
    /// The predicate of the original statement
    predicate: Arc<IRI>,
    /// The object of the original statement
    object: ReificationObject,
    /// Additional properties about the reified statement
    properties: Vec<PropertyAssertionAxiom>,
}

/// Object in a reified statement
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReificationObject {
    Named(Arc<IRI>),
    Anonymous(Box<AnonymousIndividual>),
    Literal(Literal),
}

impl ReificationAxiom {
    /// Create a new reification axiom
    pub fn new(
        reification_resource: Arc<IRI>,
        subject: Arc<IRI>,
        predicate: Arc<IRI>,
        object: ReificationObject,
    ) -> Self {
        ReificationAxiom {
            reification_resource,
            subject,
            predicate,
            object,
            properties: Vec::new(),
        }
    }

    /// Create a new reification axiom with additional properties
    pub fn with_properties(
        reification_resource: Arc<IRI>,
        subject: Arc<IRI>,
        predicate: Arc<IRI>,
        object: ReificationObject,
        properties: Vec<PropertyAssertionAxiom>,
    ) -> Self {
        ReificationAxiom {
            reification_resource,
            subject,
            predicate,
            object,
            properties,
        }
    }

    /// Get the reification resource
    pub fn reification_resource(&self) -> &Arc<IRI> {
        &self.reification_resource
    }

    /// Get the subject of the original statement
    pub fn subject(&self) -> &Arc<IRI> {
        &self.subject
    }

    /// Get the predicate of the original statement
    pub fn predicate(&self) -> &Arc<IRI> {
        &self.predicate
    }

    /// Get the object of the original statement
    pub fn object(&self) -> &ReificationObject {
        &self.object
    }

    /// Get additional properties about the reified statement
    pub fn properties(&self) -> &Vec<PropertyAssertionAxiom> {
        &self.properties
    }

    /// Add a property to the reified statement
    pub fn add_property(&mut self, property: PropertyAssertionAxiom) {
        self.properties.push(property);
    }

    /// Create property assertions from the reification
    pub fn to_property_assertions(&self) -> OwlResult<Vec<PropertyAssertionAxiom>> {
        let mut assertions = Vec::new();

        // Add rdf:subject assertion
        let subject_assertion = PropertyAssertionAxiom::new(
            self.reification_resource.clone(),
            IRI::new_optimized(rdf::subject().as_str())?,
            self.subject.clone(),
        );
        assertions.push(subject_assertion);

        // Add rdf:predicate assertion
        let predicate_assertion = PropertyAssertionAxiom::new(
            self.reification_resource.clone(),
            IRI::new_optimized(rdf::predicate().as_str())?,
            self.predicate.clone(),
        );
        assertions.push(predicate_assertion);

        // Add rdf:object assertion
        let object_iri = match &self.object {
            ReificationObject::Named(iri) => IRI::new_optimized(iri.as_str())?,
            ReificationObject::Anonymous(anon) => create_blank_node_iri(anon.node_id())?,
            ReificationObject::Literal(lit) => {
                // For literals, create a temporary IRI (simplification)
                create_blank_node_iri(&format!("literal_{}", lit.lexical_form()))?
            }
        };

        let object_assertion = PropertyAssertionAxiom::new(
            self.reification_resource.clone(),
            IRI::new_optimized(rdf::object().as_str())?,
            object_iri,
        );
        assertions.push(object_assertion);

        // Add additional properties
        assertions.extend(self.properties.clone());

        // Add rdf:type assertion to identify as rdf:Statement
        let type_assertion = PropertyAssertionAxiom::new(
            self.reification_resource.clone(),
            IRI::new_optimized(rdf::type_property().as_str())?,
            IRI::new_optimized(rdf::statement().as_str())?,
        );
        assertions.push(type_assertion);

        Ok(assertions)
    }

    /// Get the original statement as a triple (subject, predicate, object)
    pub fn original_statement(&self) -> (&Arc<IRI>, &Arc<IRI>, &ReificationObject) {
        (&self.subject, &self.predicate, &self.object)
    }
}
