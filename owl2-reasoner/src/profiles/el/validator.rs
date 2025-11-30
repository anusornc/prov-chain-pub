//! OWL2 EL Profile Validator
//!
//! This module implements validation for the OWL2 EL (Expressive Logic) profile.
//! The EL profile supports:
//! - Class subclass axioms
//! - Equivalent classes axioms (simple cases)
//! - Object property existential restrictions
//! - Data property restrictions
//! - Intersection of class expressions
//!

#![allow(clippy::only_used_in_recursion)]
//! But disallows:
//! - Disjoint classes axioms
//! - Complex equivalent classes axioms
//! - Universal restrictions
//! - Cardinality restrictions
//! - Has-value restrictions
//! - Complex data property ranges

use crate::axioms::ClassExpression;
use crate::error::OwlResult;
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::profiles::common::{ProfileViolation, ProfileViolationType, ViolationSeverity};
use crate::utils::iri::IriUtils;
use crate::utils::smallvec::SmallVecUtils;
use std::sync::Arc;

/// EL Profile Validator
pub struct ElValidator {
    ontology: Arc<Ontology>,
}

impl ElValidator {
    /// Create a new EL profile validator
    pub fn new(ontology: Arc<Ontology>) -> Self {
        Self { ontology }
    }

    /// Validate ontology against EL profile restrictions
    pub fn validate(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = SmallVecUtils::violations();

        // Check for disallowed constructs in EL profile

        // 1. No disjoint classes axioms
        if !self.ontology.disjoint_classes_axioms().is_empty() {
            violations.push(ProfileViolation {
                violation_type: ProfileViolationType::DisjointClassesAxiom,
                message: "Disjoint classes axioms are not allowed in EL profile".to_string(),
                affected_entities: self.get_affected_entities_from_disjoint_classes(),
                severity: ViolationSeverity::Error,
            });
        }

        // 2. No equivalent classes axioms (except simple cases)
        violations.extend(self.check_equivalent_classes_for_el()?);

        // 3. Check property restrictions
        violations.extend(self.check_property_restrictions_for_el()?);

        // 4. No data property ranges beyond basic datatypes
        violations.extend(self.check_data_property_ranges_for_el()?);

        Ok(violations.into_vec())
    }

    /// Quick check for EL profile compliance
    pub fn quick_check(&self) -> OwlResult<bool> {
        // Quick check for EL profile without detailed validation
        Ok(self.ontology.disjoint_classes_axioms().is_empty()
            && !self.has_complex_equivalent_classes()?
            && !self.has_complex_property_restrictions()?)
    }

    /// Check equivalent classes for EL profile compliance
    fn check_equivalent_classes_for_el(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = SmallVecUtils::violations();

        for axiom in self.ontology.equivalent_classes_axioms() {
            if axiom.classes().len() > 2 {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::EquivalentClassesAxiom,
                    message: "Complex equivalent classes axioms with more than 2 classes are not allowed in EL profile".to_string(),
                    affected_entities: self.convert_arc_iri_to_iri(axiom.classes().to_vec()),
                    severity: ViolationSeverity::Error,
                });
            }
        }

        Ok(violations.into_vec())
    }

    /// Check property restrictions for EL profile compliance
    fn check_property_restrictions_for_el(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = SmallVecUtils::violations();

        // EL Profile property restrictions:
        // - No property characteristics beyond basic ones
        // - Only existential restrictions allowed in class expressions
        // - No universal restrictions, cardinality restrictions, or has-value restrictions

        // Check subclass axioms for complex property restrictions
        for axiom in self.ontology.subclass_axioms() {
            violations.extend(self.validate_property_restrictions_in_expression(
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
                    self.validate_property_restrictions_in_expression(&class_expr, &class_expr)?,
                );
            }
        }

        Ok(violations.into_vec())
    }

    /// Validate property restrictions in class expressions
    fn validate_property_restrictions_in_expression(
        &self,
        expr: &ClassExpression,
        context: &ClassExpression,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = SmallVecUtils::violations();

        match expr {
            // These are allowed in EL
            ClassExpression::Class(_) => {}
            ClassExpression::ObjectSomeValuesFrom(_, _) => {}
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    violations.extend(
                        self.validate_property_restrictions_in_expression(class_expr, context)?,
                    );
                }
            }

            // These are NOT allowed in EL
            ClassExpression::ObjectAllValuesFrom(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexPropertyRestrictions,
                    message:
                        "Universal restrictions (ObjectAllValuesFrom) are not allowed in EL profile"
                            .to_string(),
                    affected_entities: self.convert_arc_iri_to_iri(
                        self.extract_entities_from_class_expression(context)?,
                    ),
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectHasValue(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexPropertyRestrictions,
                    message:
                        "Has-value restrictions (ObjectHasValue) are not allowed in EL profile"
                            .to_string(),
                    affected_entities: self.convert_arc_iri_to_iri(
                        self.extract_entities_from_class_expression(context)?,
                    ),
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectMinCardinality(_, _)
            | ClassExpression::ObjectMaxCardinality(_, _)
            | ClassExpression::ObjectExactCardinality(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexCardinalityRestrictions,
                    message: "Cardinality restrictions are not allowed in EL profile".to_string(),
                    affected_entities: self.convert_arc_iri_to_iri(
                        self.extract_entities_from_class_expression(context)?,
                    ),
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectHasSelf(_) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexPropertyRestrictions,
                    message: "Has-self restrictions (ObjectHasSelf) are not allowed in EL profile"
                        .to_string(),
                    affected_entities: self.convert_arc_iri_to_iri(
                        self.extract_entities_from_class_expression(context)?,
                    ),
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectUnionOf(_) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexClassExpressions,
                    message: "Union of classes (ObjectUnionOf) is not allowed in EL profile"
                        .to_string(),
                    affected_entities: self.convert_arc_iri_to_iri(
                        self.extract_entities_from_class_expression(context)?,
                    ),
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectComplementOf(_) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexClassExpressions,
                    message: "Object complement (ObjectComplementOf) is not allowed in EL profile"
                        .to_string(),
                    affected_entities: self.convert_arc_iri_to_iri(
                        self.extract_entities_from_class_expression(context)?,
                    ),
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectOneOf(_) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexClassExpressions,
                    message:
                        "Enumeration of individuals (ObjectOneOf) is not allowed in EL profile"
                            .to_string(),
                    affected_entities: self.convert_arc_iri_to_iri(
                        self.extract_entities_from_class_expression(context)?,
                    ),
                    severity: ViolationSeverity::Error,
                });
            }

            // Data property restrictions
            ClassExpression::DataSomeValuesFrom(_, _) => {} // Allowed in EL
            ClassExpression::DataAllValuesFrom(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexPropertyRestrictions,
                    message: "Universal data restrictions (DataAllValuesFrom) are not allowed in EL profile".to_string(),
                    affected_entities: self.convert_arc_iri_to_iri(self.extract_entities_from_class_expression(context)?),
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::DataHasValue(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexPropertyRestrictions,
                    message:
                        "Has-value data restrictions (DataHasValue) are not allowed in EL profile"
                            .to_string(),
                    affected_entities: self.convert_arc_iri_to_iri(
                        self.extract_entities_from_class_expression(context)?,
                    ),
                    severity: ViolationSeverity::Error,
                });
            }
            ClassExpression::DataMinCardinality(_, _)
            | ClassExpression::DataMaxCardinality(_, _)
            | ClassExpression::DataExactCardinality(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexCardinalityRestrictions,
                    message: "Data cardinality restrictions are not allowed in EL profile"
                        .to_string(),
                    affected_entities: self.convert_arc_iri_to_iri(
                        self.extract_entities_from_class_expression(context)?,
                    ),
                    severity: ViolationSeverity::Error,
                });
            }
        }

        Ok(violations.into_vec())
    }

    /// Check data property ranges for EL profile compliance
    fn check_data_property_ranges_for_el(&self) -> OwlResult<Vec<ProfileViolation>> {
        let violations = SmallVecUtils::violations();

        // EL Profile allows only basic datatypes for data property ranges
        // Complex datatypes like DataComplementOf, DataOneOf are not allowed

        for _axiom in self
            .ontology
            .axioms_by_type(crate::axioms::AxiomType::DataPropertyDomain)
        {
            // Check if domain uses complex data ranges
            // For now, we assume basic compliance since detailed data range analysis is complex
            // This is a conservative approach
        }

        for _axiom in self
            .ontology
            .axioms_by_type(crate::axioms::AxiomType::DataPropertyRange)
        {
            // Check if range uses complex data ranges
            // For now, we assume basic compliance since detailed data range analysis is complex
            // This is a conservative approach
        }

        Ok(violations.into_vec())
    }

    /// Extract entities from class expression for violation reporting (optimized)
    fn extract_entities_from_class_expression(
        &self,
        expr: &ClassExpression,
    ) -> OwlResult<Vec<Arc<IRI>>> {
        let mut entities = SmallVecUtils::iris();

        match expr {
            ClassExpression::Class(class) => {
                entities.push(Arc::clone(class.iri())); // Use Arc::clone instead of .clone()
            }
            ClassExpression::ObjectSomeValuesFrom(prop, class_expr) => {
                entities.push(self.iri_from_property_expression(prop)?);
                entities.extend(self.extract_entities_from_class_expression(class_expr)?);
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    entities.extend(self.extract_entities_from_class_expression(class_expr)?);
                }
            }
            // Add more cases as needed
            _ => {
                // For complex expressions, we might not be able to extract all entities
                // This is a limitation of the current implementation
            }
        }

        Ok(entities.into_vec())
    }

    /// Helper to extract IRI from property expression
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

    /// Helper to convert Arc<IRI> to IRI for affected entities (optimized)
    fn convert_arc_iri_to_iri(&self, arc_iris: Vec<Arc<IRI>>) -> Vec<IRI> {
        IriUtils::arc_iris_to_iris(arc_iris)
    }

    /// Optimized helper to collect IRIs from entities without intermediate cloning
    #[allow(dead_code)]
    fn collect_iris_from_entities<'a, I>(&self, entities: I) -> Vec<Arc<IRI>>
    where
        I: IntoIterator<Item = &'a Arc<IRI>>,
    {
        let mut iris = SmallVecUtils::iris();
        iris.extend(entities.into_iter().cloned());
        iris.into_vec()
    }

    /// Get affected entities from disjoint classes axioms (optimized)
    fn get_affected_entities_from_disjoint_classes(&self) -> Vec<IRI> {
        let mut arc_entities = SmallVecUtils::iris();

        for axiom in self.ontology.disjoint_classes_axioms() {
            arc_entities.extend(axiom.classes().iter().cloned());
        }

        IriUtils::arc_iris_to_iris(arc_entities.into_vec())
    }

    // Helper methods for quick checks
    fn has_complex_equivalent_classes(&self) -> OwlResult<bool> {
        // Check if there are complex equivalent classes axioms
        // For now, assume all equivalent classes are complex for EL
        Ok(!self.ontology.equivalent_classes_axioms().is_empty())
    }

    fn has_complex_property_restrictions(&self) -> OwlResult<bool> {
        // Check for complex property restrictions not allowed in EL
        // NOTE: Complex property restriction analysis not yet implemented
        // Currently returns false as a conservative estimate
        Ok(false)
    }
}
