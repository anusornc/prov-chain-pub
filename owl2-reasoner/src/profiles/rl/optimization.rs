//! OWL2 RL Profile Optimizer
//!
//! This module implements optimizations for ontologies to make them compliant
#![allow(clippy::only_used_in_recursion)]
//! with the OWL2 RL (Rule Language) profile. It provides suggestions and transformations for:
//! - Removing disallowed data range constructs
//! - Simplifying object expressions
//! - Optimizing for rule-based reasoning

use crate::axioms::{property_expressions::ObjectPropertyExpression, ClassExpression, DataRange};
use crate::error::OwlResult;
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::profiles::common::{OptimizationHint, OptimizationType, ProfileViolation};
use std::sync::Arc;

/// RL Profile Optimizer
pub struct RlOptimizer {
    ontology: Arc<Ontology>,
}

impl RlOptimizer {
    /// Create a new RL profile optimizer
    pub fn new(ontology: Arc<Ontology>) -> Self {
        Self { ontology }
    }

    /// Analyze ontology and generate optimization hints for RL compliance
    pub fn analyze_optimization_opportunities(&self) -> OwlResult<Vec<OptimizationHint>> {
        let mut hints = Vec::new();

        // Check for data complement of (not allowed in RL)
        let data_complement_count = self.count_data_complement_expressions()?;
        if data_complement_count > 0 {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::RemoveUnsupportedConstructs,
                description: format!(
                    "Remove {} data complement of expressions (not allowed in RL profile)",
                    data_complement_count
                ),
                estimated_impact: "High - required for RL compliance".to_string(),
            });
        }

        // Check for data one of (restricted in RL)
        let data_one_of_count = self.count_data_one_of_expressions()?;
        if data_one_of_count > 0 {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::SimplifyExpressions,
                description: format!(
                    "Simplify {} data one of expressions (restricted in RL profile)",
                    data_one_of_count
                ),
                estimated_impact: "Medium - improves RL compliance".to_string(),
            });
        }

        // Check for object complement of (not allowed in RL)
        let object_complement_count = self.count_object_complement_expressions()?;
        if object_complement_count > 0 {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::RemoveUnsupportedConstructs,
                description: format!(
                    "Remove {} object complement of expressions (not allowed in RL profile)",
                    object_complement_count
                ),
                estimated_impact: "High - required for RL compliance".to_string(),
            });
        }

        // Check for object has self (not allowed in RL)
        let object_has_self_count = self.count_object_has_self_expressions()?;
        if object_has_self_count > 0 {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::RemoveUnsupportedConstructs,
                description: format!(
                    "Remove {} object has self expressions (not allowed in RL profile)",
                    object_has_self_count
                ),
                estimated_impact: "High - required for RL compliance".to_string(),
            });
        }

        // Check for complex object one of expressions
        let complex_object_one_of_count = self.count_complex_object_one_of_expressions()?;
        if complex_object_one_of_count > 0 {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::SimplifyExpressions,
                description: format!(
                    "Simplify {} complex object one of expressions (RL has restrictions)",
                    complex_object_one_of_count
                ),
                estimated_impact: "Low - minor RL compliance improvement".to_string(),
            });
        }

        Ok(hints)
    }

    /// Generate detailed optimization report with specific transformations
    pub fn generate_optimization_report(&self) -> OwlResult<RlOptimizationReport> {
        let violations = self.identify_rl_violations()?;
        let hints = self.analyze_optimization_opportunities()?;

        Ok(RlOptimizationReport {
            total_violations: violations.len(),
            violations_by_type: self.categorize_violations(&violations),
            optimization_hints: hints,
            estimated_effort: self.estimate_optimization_effort(&violations),
            can_be_fully_optimized: self.can_be_fully_optimized(&violations),
        })
    }

    /// Identify specific RL violations
    fn identify_rl_violations(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // Check subclass axioms for RL violations
        for axiom in self.ontology.subclass_axioms() {
            violations.extend(self.check_subclass_axiom_for_rl(axiom)?);
        }

        // Check equivalent classes axioms for RL violations
        for axiom in self.ontology.equivalent_classes_axioms() {
            violations.extend(self.check_equivalent_classes_axiom_for_rl(axiom)?);
        }

        Ok(violations)
    }

    /// Check subclass axiom for RL compliance
    fn check_subclass_axiom_for_rl(
        &self,
        axiom: &crate::axioms::SubClassOfAxiom,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // Check super class for RL violations
        violations.extend(self.check_class_expression_for_rl(axiom.super_class())?);

        // Check sub class for RL violations
        violations.extend(self.check_class_expression_for_rl(axiom.sub_class())?);

        Ok(violations)
    }

    /// Check equivalent classes axiom for RL compliance
    fn check_equivalent_classes_axiom_for_rl(
        &self,
        axiom: &crate::axioms::EquivalentClassesAxiom,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        for class_iri in axiom.classes() {
            let class_expr = crate::axioms::ClassExpression::Class(crate::entities::Class::new(
                class_iri.as_str(),
            ));
            violations.extend(self.check_class_expression_for_rl(&class_expr)?);
        }

        Ok(violations)
    }

    /// Check class expression for RL compliance
    fn check_class_expression_for_rl(
        &self,
        expr: &ClassExpression,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        match expr {
            // Object expression violations
            ClassExpression::ObjectComplementOf(_) => {
                violations.push(ProfileViolation {
                    violation_type:
                        crate::profiles::common::ProfileViolationType::ObjectComplementOf,
                    message: "Object complement of is not allowed in RL profile".to_string(),
                    affected_entities: self.extract_entities_from_expression(expr)?,
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectHasSelf(_) => {
                violations.push(ProfileViolation {
                    violation_type: crate::profiles::common::ProfileViolationType::ObjectHasSelf,
                    message: "Object has self is not allowed in RL profile".to_string(),
                    affected_entities: self.extract_entities_from_expression(expr)?,
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectOneOf(individuals) => {
                if individuals.len() > 1 {
                    violations.push(ProfileViolation {
                        violation_type: crate::profiles::common::ProfileViolationType::ObjectOneOf,
                        message: format!(
                            "Object one of with multiple individuals ({}) may have restrictions in RL profile",
                            individuals.len()
                        ),
                        affected_entities: self.extract_entities_from_expression(expr)?,
                        severity: crate::profiles::common::ViolationSeverity::Warning,
                    });
                }
            }

            // Check data range violations in data property restrictions
            ClassExpression::DataSomeValuesFrom(_, data_range) => {
                violations.extend(self.check_data_range_for_rl(data_range)?);
            }
            ClassExpression::DataAllValuesFrom(_, data_range) => {
                violations.extend(self.check_data_range_for_rl(data_range)?);
            }

            // Recursively check nested expressions
            ClassExpression::ObjectSomeValuesFrom(_, class_expr) => {
                violations.extend(self.check_class_expression_for_rl(class_expr)?);
            }
            ClassExpression::ObjectAllValuesFrom(_, class_expr) => {
                violations.extend(self.check_class_expression_for_rl(class_expr)?);
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    violations.extend(self.check_class_expression_for_rl(class_expr)?);
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    violations.extend(self.check_class_expression_for_rl(class_expr)?);
                }
            }
            _ => {}
        }

        Ok(violations)
    }

    /// Check data range for RL compliance
    #[allow(clippy::only_used_in_recursion)]
    fn check_data_range_for_rl(&self, data_range: &DataRange) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        match data_range {
            DataRange::DataComplementOf(_) => {
                violations.push(ProfileViolation {
                    violation_type: crate::profiles::common::ProfileViolationType::DataComplementOf,
                    message: "Data complement of is not allowed in RL profile".to_string(),
                    affected_entities: vec![], // Simplified - would need data range entity extraction
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }
            DataRange::DataOneOf(_) => {
                violations.push(ProfileViolation {
                    violation_type: crate::profiles::common::ProfileViolationType::DataOneOf,
                    message: "Data one of is restricted in RL profile".to_string(),
                    affected_entities: vec![], // Simplified - would need data range entity extraction
                    severity: crate::profiles::common::ViolationSeverity::Warning,
                });
            }
            _ => {}
        }

        Ok(violations)
    }

    /// Extract entities from class expression
    #[allow(clippy::only_used_in_recursion)]
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
            ClassExpression::ObjectAllValuesFrom(prop, class_expr) => {
                entities.extend(self.extract_iri_from_property_expression(prop)?);
                entities.extend(self.extract_entities_from_expression(class_expr)?);
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    entities.extend(self.extract_entities_from_expression(class_expr)?);
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    entities.extend(self.extract_entities_from_expression(class_expr)?);
                }
            }
            _ => {}
        }

        Ok(entities)
    }

    /// Extract IRI from ObjectPropertyExpression
    fn extract_iri_from_property_expression(
        &self,
        prop: &ObjectPropertyExpression,
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

    /// Count data complement expressions
    fn count_data_complement_expressions(&self) -> OwlResult<usize> {
        let mut count = 0;

        for axiom in self.ontology.subclass_axioms() {
            count += self.count_data_complement_in_expression(axiom.super_class())?;
            count += self.count_data_complement_in_expression(axiom.sub_class())?;
        }

        for axiom in self.ontology.equivalent_classes_axioms() {
            for class_iri in axiom.classes() {
                let class_expr = crate::axioms::ClassExpression::Class(
                    crate::entities::Class::new(class_iri.as_str()),
                );
                count += self.count_data_complement_in_expression(&class_expr)?;
            }
        }

        Ok(count)
    }

    /// Count data complement in expression
    #[allow(clippy::only_used_in_recursion)]
    fn count_data_complement_in_expression(&self, expr: &ClassExpression) -> OwlResult<usize> {
        let mut count = 0;

        // Check for data complement in data ranges within the expression
        if let ClassExpression::DataSomeValuesFrom(_, data_range) = expr {
            if let DataRange::DataComplementOf(_) = **data_range {
                count += 1;
            }
        } else if let ClassExpression::DataAllValuesFrom(_, data_range) = expr {
            if let DataRange::DataComplementOf(_) = **data_range {
                count += 1;
            }
        }

        // Recursively count in nested expressions
        match expr {
            ClassExpression::ObjectSomeValuesFrom(_, class_expr) => {
                count += self.count_data_complement_in_expression(class_expr)?;
            }
            ClassExpression::ObjectAllValuesFrom(_, class_expr) => {
                count += self.count_data_complement_in_expression(class_expr)?;
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    count += self.count_data_complement_in_expression(class_expr)?;
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    count += self.count_data_complement_in_expression(class_expr)?;
                }
            }
            _ => {}
        }

        Ok(count)
    }

    /// Count data one of expressions
    fn count_data_one_of_expressions(&self) -> OwlResult<usize> {
        let mut count = 0;

        for axiom in self.ontology.subclass_axioms() {
            count += self.count_data_one_of_in_expression(axiom.super_class())?;
            count += self.count_data_one_of_in_expression(axiom.sub_class())?;
        }

        Ok(count)
    }

    /// Count data one of in expression
    #[allow(clippy::only_used_in_recursion)]
    fn count_data_one_of_in_expression(&self, expr: &ClassExpression) -> OwlResult<usize> {
        let mut count = 0;

        // Check for data one of in data ranges within the expression
        if let ClassExpression::DataSomeValuesFrom(_, data_range) = expr {
            if let DataRange::DataOneOf(_) = **data_range {
                count += 1;
            }
        } else if let ClassExpression::DataAllValuesFrom(_, data_range) = expr {
            if let DataRange::DataOneOf(_) = **data_range {
                count += 1;
            }
        }

        // Recursively count in nested expressions
        match expr {
            ClassExpression::ObjectSomeValuesFrom(_, class_expr) => {
                count += self.count_data_one_of_in_expression(class_expr)?;
            }
            ClassExpression::ObjectAllValuesFrom(_, class_expr) => {
                count += self.count_data_one_of_in_expression(class_expr)?;
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    count += self.count_data_one_of_in_expression(class_expr)?;
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    count += self.count_data_one_of_in_expression(class_expr)?;
                }
            }
            _ => {}
        }

        Ok(count)
    }

    /// Count object complement expressions
    fn count_object_complement_expressions(&self) -> OwlResult<usize> {
        let mut count = 0;

        for axiom in self.ontology.subclass_axioms() {
            count += self.count_object_complement_in_expression(axiom.super_class())?;
            count += self.count_object_complement_in_expression(axiom.sub_class())?;
        }

        for axiom in self.ontology.equivalent_classes_axioms() {
            for class_iri in axiom.classes() {
                let class_expr = crate::axioms::ClassExpression::Class(
                    crate::entities::Class::new(class_iri.as_str()),
                );
                count += self.count_object_complement_in_expression(&class_expr)?;
            }
        }

        Ok(count)
    }

    /// Count object complement in expression
    #[allow(clippy::only_used_in_recursion)]
    fn count_object_complement_in_expression(&self, expr: &ClassExpression) -> OwlResult<usize> {
        let mut count = 0;

        if let ClassExpression::ObjectComplementOf(_) = expr {
            count += 1;
        }

        // Recursively count in nested expressions
        match expr {
            ClassExpression::ObjectSomeValuesFrom(_, class_expr) => {
                count += self.count_object_complement_in_expression(class_expr)?;
            }
            ClassExpression::ObjectAllValuesFrom(_, class_expr) => {
                count += self.count_object_complement_in_expression(class_expr)?;
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    count += self.count_object_complement_in_expression(class_expr)?;
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    count += self.count_object_complement_in_expression(class_expr)?;
                }
            }
            _ => {}
        }

        Ok(count)
    }

    /// Count object has self expressions
    fn count_object_has_self_expressions(&self) -> OwlResult<usize> {
        let mut count = 0;

        for axiom in self.ontology.subclass_axioms() {
            count += self.count_object_has_self_in_expression(axiom.super_class())?;
            count += self.count_object_has_self_in_expression(axiom.sub_class())?;
        }

        for axiom in self.ontology.equivalent_classes_axioms() {
            for class_iri in axiom.classes() {
                let class_expr = crate::axioms::ClassExpression::Class(
                    crate::entities::Class::new(class_iri.as_str()),
                );
                count += self.count_object_has_self_in_expression(&class_expr)?;
            }
        }

        Ok(count)
    }

    /// Count object has self in expression
    #[allow(clippy::only_used_in_recursion)]
    fn count_object_has_self_in_expression(&self, expr: &ClassExpression) -> OwlResult<usize> {
        let mut count = 0;

        if let ClassExpression::ObjectHasSelf(_) = expr {
            count += 1;
        }

        // Recursively count in nested expressions
        match expr {
            ClassExpression::ObjectSomeValuesFrom(_, class_expr) => {
                count += self.count_object_has_self_in_expression(class_expr)?;
            }
            ClassExpression::ObjectAllValuesFrom(_, class_expr) => {
                count += self.count_object_has_self_in_expression(class_expr)?;
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    count += self.count_object_has_self_in_expression(class_expr)?;
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    count += self.count_object_has_self_in_expression(class_expr)?;
                }
            }
            _ => {}
        }

        Ok(count)
    }

    /// Count complex object one of expressions
    fn count_complex_object_one_of_expressions(&self) -> OwlResult<usize> {
        let mut count = 0;

        for axiom in self.ontology.subclass_axioms() {
            count += self.count_complex_object_one_of_in_expression(axiom.super_class())?;
            count += self.count_complex_object_one_of_in_expression(axiom.sub_class())?;
        }

        Ok(count)
    }

    /// Count complex object one of in expression
    fn count_complex_object_one_of_in_expression(
        &self,
        expr: &ClassExpression,
    ) -> OwlResult<usize> {
        let mut count = 0;

        if let ClassExpression::ObjectOneOf(individuals) = expr {
            if individuals.len() > 1 {
                count += 1;
            }
        }

        // Recursively count in nested expressions
        match expr {
            ClassExpression::ObjectSomeValuesFrom(_, class_expr) => {
                count += self.count_complex_object_one_of_in_expression(class_expr)?;
            }
            ClassExpression::ObjectAllValuesFrom(_, class_expr) => {
                count += self.count_complex_object_one_of_in_expression(class_expr)?;
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    count += self.count_complex_object_one_of_in_expression(class_expr)?;
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    count += self.count_complex_object_one_of_in_expression(class_expr)?;
                }
            }
            _ => {}
        }

        Ok(count)
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
    #[allow(clippy::only_used_in_recursion)]
    fn estimate_optimization_effort(&self, violations: &[ProfileViolation]) -> OptimizationEffort {
        let high_effort_violations = violations
            .iter()
            .filter(|v| {
                matches!(
                    v.violation_type,
                    crate::profiles::common::ProfileViolationType::DataComplementOf
                        | crate::profiles::common::ProfileViolationType::ObjectComplementOf
                )
            })
            .count();

        if high_effort_violations > 8 {
            OptimizationEffort::High
        } else if high_effort_violations > 3 {
            OptimizationEffort::Medium
        } else {
            OptimizationEffort::Low
        }
    }

    /// Check if ontology can be fully optimized for RL
    #[allow(clippy::only_used_in_recursion)]
    fn can_be_fully_optimized(&self, violations: &[ProfileViolation]) -> bool {
        // Most RL violations can be resolved through transformation
        // Some semantic constraints might prevent full optimization
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

/// RL Optimization Report
#[derive(Debug, Clone)]
pub struct RlOptimizationReport {
    pub total_violations: usize,
    pub violations_by_type: std::collections::HashMap<String, usize>,
    pub optimization_hints: Vec<OptimizationHint>,
    pub estimated_effort: OptimizationEffort,
    pub can_be_fully_optimized: bool,
}
