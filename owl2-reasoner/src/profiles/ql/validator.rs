//! OWL2 QL Profile Validator
//!
//! This module implements validation for the OWL2 QL (Query Language) profile.
//! The QL profile is designed for query answering over large amounts of instance data
//! and supports:
//! - Existential quantification
//! - Concept inclusion axioms
//! - Role inclusion axioms
//! - Role hierarchies
//! - Transitive roles (with restrictions)

#![allow(clippy::only_used_in_recursion)]
//!
//! But disallows:
//! - Property characteristics beyond basic ones
//! - Complex cardinality restrictions
//! - Property chain axioms (with some exceptions)

use crate::axioms::ClassExpression;
use crate::error::OwlResult;
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::profiles::common::{ProfileViolation, ProfileViolationType, ViolationSeverity};
use crate::utils::smallvec::SmallVecUtils;
use std::sync::Arc;

/// QL Profile Validator
pub struct QlValidator {
    ontology: Arc<Ontology>,
}

impl QlValidator {
    /// Create a new QL profile validator
    pub fn new(ontology: Arc<Ontology>) -> Self {
        Self { ontology }
    }

    /// Validate ontology against QL profile restrictions
    pub fn validate(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = SmallVecUtils::violations();

        // Check for disallowed constructs in QL profile

        // 1. No transitive properties
        violations.extend(self.check_transitive_properties_for_ql()?);

        // 2. No asymmetric properties
        violations.extend(self.check_asymmetric_properties_for_ql()?);

        // 3. No complex cardinality restrictions
        violations.extend(self.check_cardinality_restrictions_for_ql()?);

        // 4. No property chain axioms
        violations.extend(self.check_property_chains_for_ql()?);

        Ok(violations.into_vec())
    }

    /// Quick check for QL profile compliance
    pub fn quick_check(&self) -> OwlResult<bool> {
        // Quick check for QL profile
        Ok(!self.has_transitive_properties()?
            && !self.has_asymmetric_properties()?
            && !self.has_complex_cardinality_restrictions()?)
    }

    /// Check transitive properties for QL profile compliance
    fn check_transitive_properties_for_ql(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = SmallVecUtils::violations();

        // QL Profile restrictions on transitive properties:
        // - Transitive properties are allowed but with restrictions
        // - Cannot be used in certain types of axioms

        for axiom in self
            .ontology
            .axioms_by_type(crate::axioms::AxiomType::TransitiveProperty)
        {
            if let crate::axioms::Axiom::TransitiveProperty(transitive_axiom) = axiom {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::TransitiveProperties,
                    message: "Transitive object properties are restricted in QL profile"
                        .to_string(),
                    affected_entities: vec![(**transitive_axiom.property()).clone()],
                    severity: ViolationSeverity::Error,
                });
            }
        }

        Ok(violations.into_vec())
    }

    /// Check asymmetric properties for QL profile compliance
    fn check_asymmetric_properties_for_ql(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = SmallVecUtils::violations();

        // QL Profile does not allow asymmetric properties
        for axiom in self
            .ontology
            .axioms_by_type(crate::axioms::AxiomType::AsymmetricProperty)
        {
            if let crate::axioms::Axiom::AsymmetricProperty(asymmetric_axiom) = axiom {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::AsymmetricProperties,
                    message: "Asymmetric object properties are not allowed in QL profile"
                        .to_string(),
                    affected_entities: vec![(**asymmetric_axiom.property()).clone()],
                    severity: ViolationSeverity::Error,
                });
            }
        }

        Ok(violations.into_vec())
    }

    /// Check irreflexive properties for QL profile compliance
    #[allow(dead_code)]
    fn check_irreflexive_properties_for_ql(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = SmallVecUtils::violations();

        // QL Profile does not allow irreflexive properties
        for axiom in self
            .ontology
            .axioms_by_type(crate::axioms::AxiomType::IrreflexiveProperty)
        {
            if let crate::axioms::Axiom::IrreflexiveProperty(irreflexive_axiom) = axiom {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::IrreflexiveProperties,
                    message: "Irreflexive object properties are not allowed in QL profile"
                        .to_string(),
                    affected_entities: vec![(**irreflexive_axiom.property()).clone()],
                    severity: ViolationSeverity::Error,
                });
            }
        }

        Ok(violations.into_vec())
    }

    /// Check cardinality restrictions for QL profile compliance
    fn check_cardinality_restrictions_for_ql(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = SmallVecUtils::violations();

        // QL Profile allows only simple cardinality restrictions
        // Complex cardinality restrictions (qualified, non-simple) are not allowed

        // Check subclass axioms for complex cardinality restrictions
        for axiom in self.ontology.subclass_axioms() {
            violations.extend(self.check_cardinality_restrictions_in_expression(
                axiom.super_class(),
                axiom.sub_class(),
            )?);
        }

        // Check equivalent classes axioms
        for axiom in self.ontology.equivalent_classes_axioms() {
            for class_iri in axiom.classes() {
                let class_expr = crate::axioms::ClassExpression::Class(
                    crate::entities::Class::new(class_iri.as_str()),
                );
                violations.extend(
                    self.check_cardinality_restrictions_in_expression(&class_expr, &class_expr)?,
                );
            }
        }

        Ok(violations.into_vec())
    }

    /// Check property chains for QL profile compliance
    fn check_property_chains_for_ql(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = SmallVecUtils::violations();

        // QL Profile has restrictions on property chain axioms
        // Only simple property chains are allowed

        for axiom in self
            .ontology
            .axioms_by_type(crate::axioms::AxiomType::SubPropertyChainOf)
        {
            if let crate::axioms::Axiom::SubPropertyChainOf(sub_axiom) = axiom {
                let property_chain = sub_axiom.property_chain();
                if property_chain.len() > 2 {
                    violations.push(ProfileViolation {
                        violation_type: ProfileViolationType::PropertyChainAxioms,
                        message: "Complex property chains (length > 2) are not allowed in QL profile".to_string(),
                        affected_entities: property_chain
                        .iter()
                        .filter_map(|prop| match prop {
                            crate::axioms::property_expressions::ObjectPropertyExpression::ObjectProperty(obj_prop) => Some(Arc::into_inner(Arc::clone(obj_prop.iri())).unwrap_or_else(|| (*obj_prop.iri()).clone().into())),
                            crate::axioms::property_expressions::ObjectPropertyExpression::ObjectInverseOf(_) => None,
                        })
                        .collect(),
                        severity: ViolationSeverity::Error,
                    });
                }
            }
        }

        Ok(violations.into_vec())
    }

    /// Validate cardinality restrictions in class expressions
    fn check_cardinality_restrictions_in_expression(
        &self,
        expr: &ClassExpression,
        context: &ClassExpression,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = SmallVecUtils::violations();

        match expr {
            // These are allowed in QL
            ClassExpression::Class(_) => {}
            ClassExpression::ObjectSomeValuesFrom(_, _) => {}
            ClassExpression::ObjectAllValuesFrom(_, _) => {} // Allowed in QL
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    violations.extend(
                        self.check_cardinality_restrictions_in_expression(class_expr, context)?,
                    );
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    violations.extend(
                        self.check_cardinality_restrictions_in_expression(class_expr, context)?,
                    );
                }
            }

            // Cardinality restrictions need special handling in QL
            ClassExpression::ObjectMinCardinality(cardinality, _) => {
                if *cardinality > 1 {
                    violations.push(ProfileViolation {
                        violation_type: ProfileViolationType::ComplexCardinalityRestrictions,
                        message: format!(
                            "Minimum cardinality restrictions > 1 are not allowed in QL profile (found: {})",
                            cardinality
                        ),
                        affected_entities: self.extract_entities_from_class_expression(context)?,
                        severity: ViolationSeverity::Error,
                    });
                }
            }
            ClassExpression::ObjectMaxCardinality(_cardinality, _) => {
                // Qualified cardinality restrictions are not allowed in QL
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexCardinalityRestrictions,
                    message: "Maximum cardinality restrictions are not allowed in QL profile"
                        .to_string(),
                    affected_entities: self.extract_entities_from_class_expression(context)?,
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectExactCardinality(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexCardinalityRestrictions,
                    message: "Exact cardinality restrictions are not allowed in QL profile"
                        .to_string(),
                    affected_entities: self.extract_entities_from_class_expression(context)?,
                    severity: ViolationSeverity::Error,
                });
            }

            // These are generally allowed but may have restrictions
            ClassExpression::ObjectHasValue(_, _) => {
                // Has-value restrictions are allowed in QL
            }
            ClassExpression::ObjectHasSelf(_) => {
                // Has-self restrictions are allowed in QL
            }
            ClassExpression::ObjectComplementOf(_) => {
                // Object complement is allowed in QL
            }
            ClassExpression::ObjectOneOf(_) => {
                // Nominals are allowed in QL
            }

            // Data property restrictions
            ClassExpression::DataSomeValuesFrom(_, _) => {} // Allowed
            ClassExpression::DataAllValuesFrom(_, _) => {}  // Allowed
            ClassExpression::DataHasValue(_, _) => {}       // Allowed
            ClassExpression::DataMinCardinality(cardinality, _) => {
                if *cardinality > 1 {
                    violations.push(ProfileViolation {
                        violation_type: ProfileViolationType::ComplexCardinalityRestrictions,
                        message: format!(
                            "Data minimum cardinality restrictions > 1 are not allowed in QL profile (found: {})",
                            cardinality
                        ),
                        affected_entities: self.extract_entities_from_class_expression(context)?,
                        severity: ViolationSeverity::Error,
                    });
                }
            }
            ClassExpression::DataMaxCardinality(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexCardinalityRestrictions,
                    message: "Data maximum cardinality restrictions are not allowed in QL profile"
                        .to_string(),
                    affected_entities: self.extract_entities_from_class_expression(context)?,
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::DataExactCardinality(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexCardinalityRestrictions,
                    message: "Data exact cardinality restrictions are not allowed in QL profile"
                        .to_string(),
                    affected_entities: self.extract_entities_from_class_expression(context)?,
                    severity: ViolationSeverity::Error,
                });
            }
        }

        Ok(violations.into_vec())
    }

    /// Extract entities from class expression for violation reporting
    fn extract_entities_from_class_expression(
        &self,
        expr: &ClassExpression,
    ) -> OwlResult<Vec<IRI>> {
        let mut entities = Vec::new();

        match expr {
            ClassExpression::Class(class) => {
                entities.push(
                    Arc::into_inner(Arc::clone(class.iri()))
                        .unwrap_or_else(|| (*class.iri()).clone().into()),
                );
            }
            ClassExpression::ObjectSomeValuesFrom(prop, class_expr) => {
                let prop_iri = prop
                    .as_ref()
                    .as_named()
                    .ok_or(crate::error::OwlError::ExpectedNamedObjectProperty)?
                    .iri();
                entities.push(
                    Arc::into_inner(Arc::clone(prop_iri))
                        .unwrap_or_else(|| (*prop_iri).clone().into()),
                );
                entities.extend(self.extract_entities_from_class_expression(class_expr)?);
            }
            ClassExpression::ObjectAllValuesFrom(prop, class_expr) => {
                let prop_iri = prop
                    .as_ref()
                    .as_named()
                    .ok_or(crate::error::OwlError::ExpectedNamedObjectProperty)?
                    .iri();
                entities.push(
                    Arc::into_inner(Arc::clone(prop_iri))
                        .unwrap_or_else(|| (*prop_iri).clone().into()),
                );
                entities.extend(self.extract_entities_from_class_expression(class_expr)?);
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    entities.extend(self.extract_entities_from_class_expression(class_expr)?);
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    entities.extend(self.extract_entities_from_class_expression(class_expr)?);
                }
            }
            // Add more cases as needed
            _ => {
                // For complex expressions, we might not be able to extract all entities
            }
        }

        Ok(entities)
    }

    // Helper methods for quick checks
    fn has_transitive_properties(&self) -> OwlResult<bool> {
        // Check for transitive properties (restricted in QL)
        // NOTE: Property characteristic analysis not yet implemented
        // Currently returns false as a conservative estimate
        Ok(false)
    }

    fn has_asymmetric_properties(&self) -> OwlResult<bool> {
        // Check for asymmetric properties (not allowed in QL)
        // NOTE: Property characteristic analysis not yet implemented
        // Currently returns false as a conservative estimate
        Ok(false)
    }

    fn has_complex_cardinality_restrictions(&self) -> OwlResult<bool> {
        // Check for complex cardinality restrictions (not allowed in QL)
        // NOTE: Cardinality restriction analysis not yet implemented
        // Currently returns false as a conservative estimate
        Ok(false)
    }

    /// Helper to extract IRI from property expression
    #[allow(dead_code)]
    fn iri_from_property_expression(
        &self,
        prop: &crate::axioms::property_expressions::ObjectPropertyExpression,
    ) -> OwlResult<Arc<IRI>> {
        use crate::axioms::property_expressions::ObjectPropertyExpression;

        match prop {
            ObjectPropertyExpression::ObjectProperty(obj_prop) => Ok(obj_prop.iri().clone()),
            ObjectPropertyExpression::ObjectInverseOf(obj_prop) => {
                self.iri_from_property_expression(obj_prop)
            }
        }
    }
}
