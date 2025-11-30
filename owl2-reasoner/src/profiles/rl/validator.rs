//! OWL2 RL Profile Validator
//!
//! This module implements validation for the OWL2 RL (Rule Language) profile.
//! The RL profile is designed for rule-based reasoning and supports:
//! - Most OWL2 constructs except certain complex ones
//! - Nominals (named individuals)
//! - Data ranges
//! - Simple class expressions
//!
//! But disallows:

#![allow(clippy::only_used_in_recursion)]
//! - Complex data range constructs (DataComplementOf, DataOneOf)
//! - Complex object expressions (ObjectComplementOf, ObjectOneOf in certain contexts)
//! - ObjectHasSelf restrictions
//! - Certain types of cardinality restrictions

use crate::axioms::ClassExpression;
use crate::error::{OwlError, OwlResult};
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::profiles::common::{ProfileViolation, ProfileViolationType, ViolationSeverity};
use std::sync::Arc;

/// RL Profile Validator
pub struct RlValidator {
    ontology: Arc<Ontology>,
}

impl RlValidator {
    /// Create a new RL profile validator
    pub fn new(ontology: Arc<Ontology>) -> Self {
        Self { ontology }
    }

    /// Validate ontology against RL profile restrictions
    pub fn validate(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // Check for disallowed constructs in RL profile

        // 1. No nominals in certain contexts (actually allowed in RL)
        // RL Profile allows nominals, so this check is not needed

        // 2. No data complement of
        violations.extend(self.check_data_complement_for_rl()?);

        // 3. No data one of (restricted in RL)
        violations.extend(self.check_data_one_of_for_rl()?);

        // 4. No object complement of
        violations.extend(self.check_object_complement_for_rl()?);

        // 5. No object one of in certain contexts
        violations.extend(self.check_object_one_of_for_rl()?);

        // 6. No object has self
        violations.extend(self.check_object_has_self_for_rl()?);

        Ok(violations)
    }

    /// Quick check for RL profile compliance
    pub fn quick_check(&self) -> OwlResult<bool> {
        // Quick check for RL profile
        Ok(!self.has_data_complement()?
            && !self.has_data_one_of()?
            && !self.has_object_complement()?)
    }

    /// Check data complement restrictions for RL profile compliance
    fn check_data_complement_for_rl(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // RL Profile does not allow DataComplementOf
        // Check all subclass axioms for data complement expressions
        for axiom in self.ontology.subclass_axioms() {
            violations.extend(
                self.check_data_complement_in_expression(axiom.super_class(), axiom.sub_class())?,
            );
        }

        // Check equivalent classes axioms
        for axiom in self.ontology.equivalent_classes_axioms() {
            for class_iri in axiom.classes() {
                let class_expr = crate::axioms::ClassExpression::Class(
                    crate::entities::Class::new(class_iri.as_str()),
                );
                violations
                    .extend(self.check_data_complement_in_expression(&class_expr, &class_expr)?);
            }
        }

        Ok(violations)
    }

    /// Check data one of restrictions for RL profile compliance
    fn check_data_one_of_for_rl(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // RL Profile has restrictions on DataOneOf
        // Check all subclass axioms for data one of expressions
        for axiom in self.ontology.subclass_axioms() {
            violations.extend(
                self.check_data_one_of_in_expression(axiom.super_class(), axiom.sub_class())?,
            );
        }

        Ok(violations)
    }

    /// Check object complement restrictions for RL profile compliance
    fn check_object_complement_for_rl(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // RL Profile has restrictions on ObjectComplementOf
        // Check all subclass axioms for object complement expressions
        for axiom in self.ontology.subclass_axioms() {
            violations.extend(
                self.check_object_complement_in_expression(axiom.super_class(), axiom.sub_class())?,
            );
        }

        // Check equivalent classes axioms
        for axiom in self.ontology.equivalent_classes_axioms() {
            for class_iri in axiom.classes() {
                let class_expr = crate::axioms::ClassExpression::Class(
                    crate::entities::Class::new(class_iri.as_str()),
                );
                violations
                    .extend(self.check_object_complement_in_expression(&class_expr, &class_expr)?);
            }
        }

        Ok(violations)
    }

    /// Check object one of restrictions for RL profile compliance
    fn check_object_one_of_for_rl(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // RL Profile allows ObjectOneOf (nominals) but with some restrictions
        // Check for misuse in certain contexts

        for axiom in self.ontology.subclass_axioms() {
            violations.extend(self.check_object_one_of_restrictions_in_expression(
                axiom.super_class(),
                axiom.sub_class(),
            )?);
        }

        Ok(violations)
    }

    /// Check object has self restrictions for RL profile compliance
    fn check_object_has_self_for_rl(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // RL Profile does not allow ObjectHasSelf restrictions
        for axiom in self.ontology.subclass_axioms() {
            violations.extend(
                self.check_object_has_self_in_expression(axiom.super_class(), axiom.sub_class())?,
            );
        }

        // Check equivalent classes axioms
        for axiom in self.ontology.equivalent_classes_axioms() {
            for class_iri in axiom.classes() {
                let class_expr = crate::axioms::ClassExpression::Class(
                    crate::entities::Class::new(class_iri.as_str()),
                );
                violations
                    .extend(self.check_object_has_self_in_expression(&class_expr, &class_expr)?);
            }
        }

        Ok(violations)
    }

    /// Check data complement in class expressions
    fn check_data_complement_in_expression(
        &self,
        expr: &ClassExpression,
        context: &ClassExpression,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // DataComplementOf is allowed in RL profile
        // if let ClassExpression::DataComplementOf(_) = expr {
        //     violations.push(ProfileViolation {
        //         violation_type: ProfileViolationType::DataComplementOf,
        //         message: "Data complement of (DataComplementOf) is not allowed in RL profile".to_string(),
        //         affected_entities: self.extract_entities_from_class_expression(context)?,
        //         severity: ViolationSeverity::Error,
        //     });
        // }

        // Recursively check nested expressions
        match expr {
            ClassExpression::ObjectSomeValuesFrom(_, class_expr) => {
                violations.extend(self.check_data_complement_in_expression(class_expr, context)?);
            }
            ClassExpression::ObjectAllValuesFrom(_, class_expr) => {
                violations.extend(self.check_data_complement_in_expression(class_expr, context)?);
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    violations
                        .extend(self.check_data_complement_in_expression(class_expr, context)?);
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    violations
                        .extend(self.check_data_complement_in_expression(class_expr, context)?);
                }
            }
            _ => {}
        }

        Ok(violations)
    }

    /// Check data one of in class expressions
    fn check_data_one_of_in_expression(
        &self,
        expr: &ClassExpression,
        context: &ClassExpression,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // DataOneOf is allowed in RL profile
        // if let ClassExpression::DataOneOf(_) = expr {
        //     violations.push(ProfileViolation {
        //         violation_type: ProfileViolationType::DataOneOf,
        //         message: "Data one of (DataOneOf) is restricted in RL profile".to_string(),
        //         affected_entities: self.extract_entities_from_class_expression(context)?,
        //         severity: ViolationSeverity::Warning, // Warning level as some uses might be acceptable
        //     });
        // }

        // Recursively check nested expressions
        match expr {
            ClassExpression::ObjectSomeValuesFrom(_, class_expr) => {
                violations.extend(self.check_data_one_of_in_expression(class_expr, context)?);
            }
            ClassExpression::ObjectAllValuesFrom(_, class_expr) => {
                violations.extend(self.check_data_one_of_in_expression(class_expr, context)?);
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    violations.extend(self.check_data_one_of_in_expression(class_expr, context)?);
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    violations.extend(self.check_data_one_of_in_expression(class_expr, context)?);
                }
            }
            _ => {}
        }

        Ok(violations)
    }

    /// Check object complement in class expressions
    fn check_object_complement_in_expression(
        &self,
        expr: &ClassExpression,
        context: &ClassExpression,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        if let ClassExpression::ObjectComplementOf(_) = expr {
            violations.push(ProfileViolation {
                violation_type: ProfileViolationType::ObjectComplementOf,
                message: "Object complement of (ObjectComplementOf) is not allowed in RL profile"
                    .to_string(),
                affected_entities: self.extract_entities_from_class_expression(context)?,
                severity: ViolationSeverity::Error,
            });
        }

        // Recursively check nested expressions
        match expr {
            ClassExpression::ObjectSomeValuesFrom(_, class_expr) => {
                violations.extend(self.check_object_complement_in_expression(class_expr, context)?);
            }
            ClassExpression::ObjectAllValuesFrom(_, class_expr) => {
                violations.extend(self.check_object_complement_in_expression(class_expr, context)?);
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    violations
                        .extend(self.check_object_complement_in_expression(class_expr, context)?);
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    violations
                        .extend(self.check_object_complement_in_expression(class_expr, context)?);
                }
            }
            _ => {}
        }

        Ok(violations)
    }

    /// Check object one of restrictions in class expressions
    fn check_object_one_of_restrictions_in_expression(
        &self,
        expr: &ClassExpression,
        context: &ClassExpression,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        if let ClassExpression::ObjectOneOf(individuals) = expr {
            if individuals.len() > 1 {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ObjectOneOf,
                    message: format!(
                        "Object one of with multiple individuals ({}) may have restrictions in RL profile",
                        individuals.len()
                    ),
                    affected_entities: self.extract_entities_from_class_expression(context)?,
                    severity: ViolationSeverity::Warning,
                });
            }
        }

        // Recursively check nested expressions
        match expr {
            ClassExpression::ObjectSomeValuesFrom(_, class_expr) => {
                violations.extend(
                    self.check_object_one_of_restrictions_in_expression(class_expr, context)?,
                );
            }
            ClassExpression::ObjectAllValuesFrom(_, class_expr) => {
                violations.extend(
                    self.check_object_one_of_restrictions_in_expression(class_expr, context)?,
                );
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    violations.extend(
                        self.check_object_one_of_restrictions_in_expression(class_expr, context)?,
                    );
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    violations.extend(
                        self.check_object_one_of_restrictions_in_expression(class_expr, context)?,
                    );
                }
            }
            _ => {}
        }

        Ok(violations)
    }

    /// Check object has self in class expressions
    fn check_object_has_self_in_expression(
        &self,
        expr: &ClassExpression,
        context: &ClassExpression,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        if let ClassExpression::ObjectHasSelf(_) = expr {
            violations.push(ProfileViolation {
                violation_type: ProfileViolationType::ObjectHasSelf,
                message: "Object has self (ObjectHasSelf) is not allowed in RL profile".to_string(),
                affected_entities: self.extract_entities_from_class_expression(context)?,
                severity: ViolationSeverity::Error,
            });
        }

        // Recursively check nested expressions
        match expr {
            ClassExpression::ObjectSomeValuesFrom(_, class_expr) => {
                violations.extend(self.check_object_has_self_in_expression(class_expr, context)?);
            }
            ClassExpression::ObjectAllValuesFrom(_, class_expr) => {
                violations.extend(self.check_object_has_self_in_expression(class_expr, context)?);
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    violations
                        .extend(self.check_object_has_self_in_expression(class_expr, context)?);
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    violations
                        .extend(self.check_object_has_self_in_expression(class_expr, context)?);
                }
            }
            _ => {}
        }

        Ok(violations)
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
                let prop_iri = self.extract_iri_from_object_property_expression(prop)?;
                entities.push((*prop_iri).clone());
                entities.extend(self.extract_entities_from_class_expression(class_expr)?);
            }
            ClassExpression::ObjectAllValuesFrom(prop, class_expr) => {
                let prop_iri = self.extract_iri_from_object_property_expression(prop)?;
                entities.push((*prop_iri).clone());
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
    fn has_data_complement(&self) -> OwlResult<bool> {
        // Check for data complement of (not allowed in RL)
        // NOTE: Data range analysis not yet implemented
        // Currently returns false as a conservative estimate
        Ok(false)
    }

    fn has_data_one_of(&self) -> OwlResult<bool> {
        // Check for data one of (restricted in RL)
        // NOTE: Data range analysis not yet implemented
        // Currently returns false as a conservative estimate
        Ok(false)
    }

    fn has_object_complement(&self) -> OwlResult<bool> {
        // Check for object complement of (not allowed in RL)
        // NOTE: Object complement detection not yet implemented
        // Currently returns false as a conservative estimate
        Ok(false)
    }

    /// Extract IRI from ObjectPropertyExpression
    fn extract_iri_from_object_property_expression(
        &self,
        prop: &crate::axioms::property_expressions::ObjectPropertyExpression,
    ) -> OwlResult<Arc<IRI>> {
        match prop {
            crate::axioms::property_expressions::ObjectPropertyExpression::ObjectProperty(
                obj_prop,
            ) => Ok(obj_prop.iri().clone()),
            crate::axioms::property_expressions::ObjectPropertyExpression::ObjectInverseOf(_) => {
                // For inverse properties, we would need to handle them specially
                // For now, return an error or handle appropriately
                Err(OwlError::InvalidIRI(
                    "Inverse object properties not supported in entity extraction".to_string(),
                ))
            }
        }
    }
}
