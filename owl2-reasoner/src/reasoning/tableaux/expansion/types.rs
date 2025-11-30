//! Types and enums for tableaux expansion rules
//!
//! Contains the core data structures for representing expansion rules,
//! tasks, and related metadata.

/// Types of expansion rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpansionRule {
    /// Conjunction rule
    Conjunction,
    /// Disjunction rule
    Disjunction,
    /// Existential restriction rule
    ExistentialRestriction,
    /// Universal restriction rule
    UniversalRestriction,
    /// Nominal rule
    Nominal,
    /// Data range rule
    DataRange,
    /// Subclass axiom application rule
    SubclassAxiom,
    /// Transitive property rule
    TransitiveProperty,
    /// Symmetric property rule
    SymmetricProperty,
    /// Reflexive property rule
    ReflexiveProperty,
    /// Functional property rule (clash detection)
    FunctionalProperty,
    /// Inverse functional property rule (clash detection)
    InverseFunctionalProperty,
    /// Irreflexive property rule (clash detection)
    IrreflexiveProperty,
    /// Asymmetric property rule (clash detection)
    AsymmetricProperty,
    /// Property hierarchy rule (SubObjectPropertyOf)
    PropertyHierarchy,
    /// Property domain rule
    PropertyDomain,
    /// Property range rule
    PropertyRange,
    /// Inverse property rule
    InverseProperty,
    /// Property assertion rule
    PropertyAssertion,
    /// Negative property assertion rule (clash detection)
    NegativePropertyAssertion,
    /// Same individual rule (node merging)
    SameIndividual,
    /// Different individuals rule (inequality clash)
    DifferentIndividuals,
}

impl ExpansionRule {
    /// Get the priority of this rule (lower number = higher priority)
    pub fn priority(self) -> u8 {
        match self {
            ExpansionRule::Conjunction => 1,
            ExpansionRule::Disjunction => 2,
            ExpansionRule::ExistentialRestriction => 3,
            ExpansionRule::UniversalRestriction => 4,
            ExpansionRule::Nominal => 5,
            ExpansionRule::DataRange => 6,
            ExpansionRule::SubclassAxiom => 7,
            ExpansionRule::TransitiveProperty => 8,
            ExpansionRule::SymmetricProperty => 9,
            ExpansionRule::ReflexiveProperty => 10,
            ExpansionRule::FunctionalProperty => 11,
            ExpansionRule::InverseFunctionalProperty => 12,
            ExpansionRule::IrreflexiveProperty => 13,
            ExpansionRule::AsymmetricProperty => 14,
            ExpansionRule::PropertyHierarchy => 15,
            ExpansionRule::PropertyDomain => 16,
            ExpansionRule::PropertyRange => 17,
            ExpansionRule::InverseProperty => 18,
            ExpansionRule::PropertyAssertion => 19,
            ExpansionRule::NegativePropertyAssertion => 20,
            ExpansionRule::SameIndividual => 21,
            ExpansionRule::DifferentIndividuals => 22,
        }
    }

    /// Get the name of this rule for debugging
    pub fn name(self) -> &'static str {
        match self {
            ExpansionRule::Conjunction => "Conjunction",
            ExpansionRule::Disjunction => "Disjunction",
            ExpansionRule::ExistentialRestriction => "ExistentialRestriction",
            ExpansionRule::UniversalRestriction => "UniversalRestriction",
            ExpansionRule::Nominal => "Nominal",
            ExpansionRule::DataRange => "DataRange",
            ExpansionRule::SubclassAxiom => "SubclassAxiom",
            ExpansionRule::TransitiveProperty => "TransitiveProperty",
            ExpansionRule::SymmetricProperty => "SymmetricProperty",
            ExpansionRule::ReflexiveProperty => "ReflexiveProperty",
            ExpansionRule::FunctionalProperty => "FunctionalProperty",
            ExpansionRule::InverseFunctionalProperty => "InverseFunctionalProperty",
            ExpansionRule::IrreflexiveProperty => "IrreflexiveProperty",
            ExpansionRule::AsymmetricProperty => "AsymmetricProperty",
            ExpansionRule::PropertyHierarchy => "PropertyHierarchy",
            ExpansionRule::PropertyDomain => "PropertyDomain",
            ExpansionRule::PropertyRange => "PropertyRange",
            ExpansionRule::InverseProperty => "InverseProperty",
            ExpansionRule::PropertyAssertion => "PropertyAssertion",
            ExpansionRule::NegativePropertyAssertion => "NegativePropertyAssertion",
            ExpansionRule::SameIndividual => "SameIndividual",
            ExpansionRule::DifferentIndividuals => "DifferentIndividuals",
        }
    }

    /// Check if this rule creates a branching point (non-deterministic choice)
    pub fn is_branching(self) -> bool {
        matches!(self, ExpansionRule::Disjunction)
    }

    /// Check if this rule can create new nodes
    pub fn creates_nodes(self) -> bool {
        matches!(
            self,
            ExpansionRule::ExistentialRestriction | ExpansionRule::SameIndividual
        )
    }

    /// Check if this rule can detect clashes
    pub fn is_clash_detection(self) -> bool {
        matches!(
            self,
            ExpansionRule::FunctionalProperty
                | ExpansionRule::InverseFunctionalProperty
                | ExpansionRule::IrreflexiveProperty
                | ExpansionRule::AsymmetricProperty
                | ExpansionRule::NegativePropertyAssertion
                | ExpansionRule::DifferentIndividuals
        )
    }

    /// Check if this rule is a property-related rule
    pub fn is_property_rule(self) -> bool {
        matches!(
            self,
            ExpansionRule::TransitiveProperty
                | ExpansionRule::SymmetricProperty
                | ExpansionRule::ReflexiveProperty
                | ExpansionRule::FunctionalProperty
                | ExpansionRule::InverseFunctionalProperty
                | ExpansionRule::IrreflexiveProperty
                | ExpansionRule::AsymmetricProperty
                | ExpansionRule::PropertyHierarchy
                | ExpansionRule::PropertyDomain
                | ExpansionRule::PropertyRange
                | ExpansionRule::InverseProperty
                | ExpansionRule::PropertyAssertion
                | ExpansionRule::NegativePropertyAssertion
        )
    }

    /// Check if this rule is a class expression rule
    pub fn is_class_expression_rule(self) -> bool {
        matches!(
            self,
            ExpansionRule::Conjunction
                | ExpansionRule::Disjunction
                | ExpansionRule::ExistentialRestriction
                | ExpansionRule::UniversalRestriction
                | ExpansionRule::Nominal
                | ExpansionRule::DataRange
        )
    }
}

impl std::fmt::Display for ExpansionRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Expansion task for applying a specific rule
#[derive(Debug, Clone)]
pub struct ExpansionTask {
    /// Rule to apply
    pub rule: ExpansionRule,
    /// Target node ID
    pub node_id: crate::reasoning::tableaux::core::NodeId,
    /// Associated class expression (if applicable)
    pub class_expression: Option<crate::axioms::class_expressions::ClassExpression>,
    /// Associated property expression (if applicable)
    pub property_expression: Option<crate::axioms::ObjectPropertyExpression>,
    /// Associated IRI (if applicable)
    pub iri: Option<crate::iri::IRI>,
    /// Task priority
    pub priority: u8,
    /// Task creation depth
    pub depth: u32,
}

impl ExpansionTask {
    /// Create a new expansion task
    pub fn new(rule: ExpansionRule, node_id: crate::reasoning::tableaux::core::NodeId) -> Self {
        Self {
            priority: rule.priority(),
            rule,
            node_id,
            class_expression: None,
            property_expression: None,
            iri: None,
            depth: 0,
        }
    }

    /// Create a task with class expression
    pub fn with_class_expression(
        mut self,
        class_expression: crate::axioms::class_expressions::ClassExpression,
    ) -> Self {
        self.class_expression = Some(class_expression);
        self
    }

    /// Create a task with property expression
    pub fn with_property_expression(
        mut self,
        property_expression: crate::axioms::ObjectPropertyExpression,
    ) -> Self {
        self.property_expression = Some(property_expression);
        self
    }

    /// Create a task with IRI
    pub fn with_iri(mut self, iri: crate::iri::IRI) -> Self {
        self.iri = Some(iri);
        self
    }

    /// Create a task with specific depth
    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    /// Create a task with custom priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Get a summary of this task for debugging
    pub fn summary(&self) -> String {
        format!(
            "Task: {} on node {:?} (priority: {}, depth: {})",
            self.rule, self.node_id, self.priority, self.depth
        )
    }
}

impl PartialEq for ExpansionTask {
    fn eq(&self, other: &Self) -> bool {
        self.rule == other.rule
            && self.node_id == other.node_id
            && self.class_expression == other.class_expression
            && self.property_expression == other.property_expression
            && self.iri == other.iri
    }
}

impl Eq for ExpansionTask {}

impl std::hash::Hash for ExpansionTask {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.rule.hash(state);
        self.node_id.hash(state);
        // Note: We can't hash class_expression/property_expression directly
        // as they don't implement Hash. This is intentional for task deduplication.
        if let Some(ref iri) = self.iri {
            iri.hash(state);
        }
    }
}

/// Partial order for priority queue (reverse ordering for min-heap)
impl PartialOrd for ExpansionTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ExpansionTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Lower priority number = higher priority (comes first)
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| other.depth.cmp(&self.depth)) // Shallower depth first
            .then_with(|| self.node_id.cmp(&other.node_id))
    }
}
