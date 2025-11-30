//! OWL2 EL Profile Optimizer
#![allow(clippy::only_used_in_recursion)]
//!
//! This module implements optimizations for ontologies to make them compliant
//! with the OWL2 EL profile. It provides suggestions and transformations for:
//! - Removing disallowed constructs
//! - Simplifying complex expressions
//! - Restructuring hierarchies

use crate::axioms::ClassExpression;
use crate::error::OwlResult;
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::profiles::common::{OptimizationHint, OptimizationType, ProfileViolation};
use std::sync::Arc;

/// EL Profile Optimizer
pub struct ElOptimizer {
    ontology: Arc<Ontology>,
}

impl ElOptimizer {
    /// Create a new EL profile optimizer
    pub fn new(ontology: Arc<Ontology>) -> Self {
        Self { ontology }
    }

    /// Analyze ontology and generate optimization hints for EL compliance
    pub fn analyze_optimization_opportunities(&self) -> OwlResult<Vec<OptimizationHint>> {
        let mut hints = Vec::new();

        // Check for disjoint classes axioms (not allowed in EL)
        if !self.ontology.disjoint_classes_axioms().is_empty() {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::RemoveUnsupportedConstructs,
                description: format!(
                    "Remove {} disjoint classes axioms (not allowed in EL profile)",
                    self.ontology.disjoint_classes_axioms().len()
                ),
                estimated_impact: "High - required for EL compliance".to_string(),
            });
        }

        // Check for complex equivalent classes
        let complex_equiv_classes = self.count_complex_equivalent_classes()?;
        if complex_equiv_classes > 0 {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::SimplifyExpressions,
                description: format!(
                    "Simplify {} complex equivalent classes axioms (EL allows only simple cases)",
                    complex_equiv_classes
                ),
                estimated_impact: "Medium - improves EL compliance".to_string(),
            });
        }

        // Check for complex property restrictions
        let complex_restrictions = self.count_complex_property_restrictions()?;
        if complex_restrictions > 0 {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::SimplifyExpressions,
                description: format!(
                    "Simplify {} complex property restrictions (EL allows only existential restrictions)",
                    complex_restrictions
                ),
                estimated_impact: "High - critical for EL compliance".to_string(),
            });
        }

        // Check for complex data property ranges
        let complex_data_ranges = self.count_complex_data_ranges()?;
        if complex_data_ranges > 0 {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::SimplifyExpressions,
                description: format!(
                    "Simplify {} complex data property ranges (EL allows only basic datatypes)",
                    complex_data_ranges
                ),
                estimated_impact: "Medium - improves EL compliance".to_string(),
            });
        }

        Ok(hints)
    }

    /// Generate detailed optimization report with specific transformations
    pub fn generate_optimization_report(&self) -> OwlResult<ElOptimizationReport> {
        let violations = self.identify_el_violations()?;
        let hints = self.analyze_optimization_opportunities()?;

        Ok(ElOptimizationReport {
            total_violations: violations.len(),
            violations_by_type: self.categorize_violations(&violations),
            optimization_hints: hints,
            estimated_effort: self.estimate_optimization_effort(&violations),
            can_be_fully_optimized: self.can_be_fully_optimized(&violations),
        })
    }

    /// Identify specific EL violations
    fn identify_el_violations(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // Check for disjoint classes axioms
        for axiom in self.ontology.disjoint_classes_axioms() {
            violations.push(ProfileViolation {
                violation_type: crate::profiles::common::ProfileViolationType::DisjointClassesAxiom,
                message: "Disjoint classes axioms are not allowed in EL profile".to_string(),
                affected_entities: axiom.classes().iter().map(|iri| (**iri).clone()).collect(),
                severity: crate::profiles::common::ViolationSeverity::Error,
            });
        }

        // Check for complex equivalent classes
        for axiom in self.ontology.equivalent_classes_axioms() {
            if axiom.classes().len() > 2 {
                violations.push(ProfileViolation {
                    violation_type:
                        crate::profiles::common::ProfileViolationType::EquivalentClassesAxiom,
                    message: "Complex equivalent classes axioms are not allowed in EL profile"
                        .to_string(),
                    affected_entities: axiom.classes().iter().map(|iri| (**iri).clone()).collect(),
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }
        }

        // Check for complex property restrictions in subclass axioms
        for axiom in self.ontology.subclass_axioms() {
            violations.extend(self.check_subclass_axiom_restrictions(axiom)?);
        }

        Ok(violations)
    }

    /// Check subclass axiom for EL property restrictions
    fn check_subclass_axiom_restrictions(
        &self,
        axiom: &crate::axioms::SubClassOfAxiom,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // Check super class for complex restrictions
        violations.extend(self.check_class_expression_for_el(axiom.super_class())?);

        // Check sub class for complex restrictions
        violations.extend(self.check_class_expression_for_el(axiom.sub_class())?);

        Ok(violations)
    }

    /// Check class expression for EL compliance
    fn check_class_expression_for_el(
        &self,
        expr: &ClassExpression,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        match expr {
            // Allowed in EL
            ClassExpression::Class(_) => {}
            ClassExpression::ObjectSomeValuesFrom(_, _) => {}
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    violations.extend(self.check_class_expression_for_el(class_expr)?);
                }
            }

            // Not allowed in EL
            ClassExpression::ObjectAllValuesFrom(_, _) => {
                violations.push(ProfileViolation {
                    violation_type:
                        crate::profiles::common::ProfileViolationType::ComplexPropertyRestrictions,
                    message: "Universal restrictions are not allowed in EL profile".to_string(),
                    affected_entities: self.extract_entities_from_expression(expr)?,
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectHasValue(_, _) => {
                violations.push(ProfileViolation {
                    violation_type:
                        crate::profiles::common::ProfileViolationType::ComplexPropertyRestrictions,
                    message: "Has-value restrictions are not allowed in EL profile".to_string(),
                    affected_entities: self.extract_entities_from_expression(expr)?,
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectMinCardinality(_, _)
            | ClassExpression::ObjectMaxCardinality(_, _)
            | ClassExpression::ObjectExactCardinality(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: crate::profiles::common::ProfileViolationType::ComplexCardinalityRestrictions,
                    message: "Cardinality restrictions are not allowed in EL profile".to_string(),
                    affected_entities: self.extract_entities_from_expression(expr)?,
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectUnionOf(_) => {
                violations.push(ProfileViolation {
                    violation_type:
                        crate::profiles::common::ProfileViolationType::ComplexClassExpressions,
                    message: "Union of classes is not allowed in EL profile".to_string(),
                    affected_entities: self.extract_entities_from_expression(expr)?,
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectComplementOf(_) => {
                violations.push(ProfileViolation {
                    violation_type:
                        crate::profiles::common::ProfileViolationType::ComplexClassExpressions,
                    message: "Object complement is not allowed in EL profile".to_string(),
                    affected_entities: self.extract_entities_from_expression(expr)?,
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectOneOf(_) => {
                violations.push(ProfileViolation {
                    violation_type:
                        crate::profiles::common::ProfileViolationType::ComplexClassExpressions,
                    message: "Enumeration of individuals is not allowed in EL profile".to_string(),
                    affected_entities: self.extract_entities_from_expression(expr)?,
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectHasSelf(_) => {
                violations.push(ProfileViolation {
                    violation_type:
                        crate::profiles::common::ProfileViolationType::ComplexPropertyRestrictions,
                    message: "Has-self restrictions are not allowed in EL profile".to_string(),
                    affected_entities: self.extract_entities_from_expression(expr)?,
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }

            // Data property restrictions
            ClassExpression::DataSomeValuesFrom(_, _) => {} // Allowed
            ClassExpression::DataAllValuesFrom(_, _) => {
                violations.push(ProfileViolation {
                    violation_type:
                        crate::profiles::common::ProfileViolationType::ComplexPropertyRestrictions,
                    message: "Universal data restrictions are not allowed in EL profile"
                        .to_string(),
                    affected_entities: self.extract_entities_from_expression(expr)?,
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }
            ClassExpression::DataHasValue(_, _) => {
                violations.push(ProfileViolation {
                    violation_type:
                        crate::profiles::common::ProfileViolationType::ComplexPropertyRestrictions,
                    message: "Has-value data restrictions are not allowed in EL profile"
                        .to_string(),
                    affected_entities: self.extract_entities_from_expression(expr)?,
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }
            ClassExpression::DataMinCardinality(_, _)
            | ClassExpression::DataMaxCardinality(_, _)
            | ClassExpression::DataExactCardinality(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: crate::profiles::common::ProfileViolationType::ComplexCardinalityRestrictions,
                    message: "Data cardinality restrictions are not allowed in EL profile".to_string(),
                    affected_entities: self.extract_entities_from_expression(expr)?,
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }
        }

        Ok(violations)
    }

    /// Extract entities from class expression
    fn extract_entities_from_expression(&self, expr: &ClassExpression) -> OwlResult<Vec<IRI>> {
        let mut entities = Vec::new();

        match expr {
            ClassExpression::Class(class) => {
                entities.push((*class.iri()).clone().into());
            }
            ClassExpression::ObjectSomeValuesFrom(prop, class_expr) => {
                entities.extend(self.extract_iri_from_property_expression(prop)?);
                entities.extend(self.extract_entities_from_expression(class_expr)?);
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    entities.extend(self.extract_entities_from_expression(class_expr)?);
                }
            }
            // Add more cases as needed
            _ => {}
        }

        Ok(entities)
    }

    /// Extract IRI from ObjectPropertyExpression
    #[allow(clippy::only_used_in_recursion)]
    fn extract_iri_from_property_expression(
        &self,
        prop: &crate::axioms::property_expressions::ObjectPropertyExpression,
    ) -> OwlResult<Vec<IRI>> {
        use crate::axioms::property_expressions::ObjectPropertyExpression;

        match prop {
            ObjectPropertyExpression::ObjectProperty(obj_prop) => {
                Ok(vec![(*obj_prop.iri()).clone().into()])
            }
            ObjectPropertyExpression::ObjectInverseOf(obj_prop) => {
                self.extract_iri_from_property_expression(obj_prop)
            }
        }
    }

    /// Count complex equivalent classes axioms
    fn count_complex_equivalent_classes(&self) -> OwlResult<usize> {
        let mut count = 0;
        for axiom in self.ontology.equivalent_classes_axioms() {
            if axiom.classes().len() > 2 {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Count complex property restrictions
    fn count_complex_property_restrictions(&self) -> OwlResult<usize> {
        let mut count = 0;

        for axiom in self.ontology.subclass_axioms() {
            if self.has_complex_restrictions(axiom.super_class())? {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Check if class expression has complex restrictions not allowed in EL
    #[allow(clippy::only_used_in_recursion)]
    fn has_complex_restrictions(&self, expr: &ClassExpression) -> OwlResult<bool> {
        match expr {
            ClassExpression::Class(_) => Ok(false),
            ClassExpression::ObjectSomeValuesFrom(_, _) => Ok(false),
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    if self.has_complex_restrictions(class_expr)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            ClassExpression::ObjectAllValuesFrom(_, _)
            | ClassExpression::ObjectHasValue(_, _)
            | ClassExpression::ObjectMinCardinality(_, _)
            | ClassExpression::ObjectMaxCardinality(_, _)
            | ClassExpression::ObjectExactCardinality(_, _)
            | ClassExpression::ObjectHasSelf(_)
            | ClassExpression::ObjectUnionOf(_)
            | ClassExpression::ObjectComplementOf(_)
            | ClassExpression::ObjectOneOf(_) => Ok(true),
            // Data property restrictions
            ClassExpression::DataSomeValuesFrom(_, _) => Ok(false),
            ClassExpression::DataAllValuesFrom(_, _)
            | ClassExpression::DataHasValue(_, _)
            | ClassExpression::DataMinCardinality(_, _)
            | ClassExpression::DataMaxCardinality(_, _)
            | ClassExpression::DataExactCardinality(_, _) => Ok(true),
        }
    }

    /// Count complex data property ranges
    fn count_complex_data_ranges(&self) -> OwlResult<usize> {
        // For now, return a conservative estimate
        // Detailed data range analysis would require more sophisticated logic
        Ok(0)
    }

    /// Categorize violations by type
    fn categorize_violations(
        &self,
        violations: &[ProfileViolation],
    ) -> std::collections::HashMap<String, usize> {
        let mut categories = std::collections::HashMap::new();

        for violation in violations {
            let category = format!("{:?}", violation.violation_type);
            *categories.entry(category).or_insert(0) += 1;
        }

        categories
    }

    /// Estimate optimization effort
    fn estimate_optimization_effort(&self, violations: &[ProfileViolation]) -> OptimizationEffort {
        let high_effort_violations = violations
            .iter()
            .filter(|v| {
                matches!(
                    v.violation_type,
                    crate::profiles::common::ProfileViolationType::ComplexPropertyRestrictions
                        | crate::profiles::common::ProfileViolationType::ComplexClassExpressions
                )
            })
            .count();

        if high_effort_violations > 10 {
            OptimizationEffort::High
        } else if high_effort_violations > 3 {
            OptimizationEffort::Medium
        } else {
            OptimizationEffort::Low
        }
    }

    /// Check if ontology can be fully optimized for EL
    fn can_be_fully_optimized(&self, violations: &[ProfileViolation]) -> bool {
        // For now, assume most violations can be resolved
        // In practice, some semantic constraints might prevent full optimization
        !violations.is_empty()
    }
}

/// Optimization effort levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptimizationEffort {
    Low,    // Simple changes, minor restructuring
    Medium, // Moderate restructuring required
    High,   // Major restructuring, semantic changes needed
}

/// EL Optimization Report
#[derive(Debug, Clone)]
pub struct ElOptimizationReport {
    pub total_violations: usize,
    pub violations_by_type: std::collections::HashMap<String, usize>,
    pub optimization_hints: Vec<OptimizationHint>,
    pub estimated_effort: OptimizationEffort,
    pub can_be_fully_optimized: bool,
}
