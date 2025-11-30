//! OWL2 QL Profile Optimizer
//!
//! This module implements optimizations for ontologies to make them compliant
#![allow(clippy::only_used_in_recursion)]
//! with the OWL2 QL profile. It provides suggestions and transformations for:
//! - Removing disallowed property characteristics
//! - Simplifying cardinality restrictions
//! - Optimizing property chains

use crate::axioms::ClassExpression;
use crate::error::OwlResult;
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::profiles::common::{OptimizationHint, OptimizationType, ProfileViolation};
use std::sync::Arc;

/// QL Profile Optimizer
pub struct QlOptimizer {
    ontology: Arc<Ontology>,
}

impl QlOptimizer {
    /// Create a new QL profile optimizer
    pub fn new(ontology: Arc<Ontology>) -> Self {
        Self { ontology }
    }

    /// Analyze ontology and generate optimization hints for QL compliance
    pub fn analyze_optimization_opportunities(&self) -> OwlResult<Vec<OptimizationHint>> {
        let mut hints = Vec::new();

        // Check for transitive properties (restricted in QL)
        let transitive_count = self
            .ontology
            .axioms_by_type(crate::axioms::AxiomType::TransitiveProperty)
            .len();
        if transitive_count > 0 {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::RemoveUnsupportedConstructs,
                description: format!(
                    "Remove or modify {} transitive property axioms (restricted in QL profile)",
                    transitive_count
                ),
                estimated_impact: "High - affects query performance".to_string(),
            });
        }

        // Check for asymmetric properties (not allowed in QL)
        let asymmetric_count = self
            .ontology
            .axioms_by_type(crate::axioms::AxiomType::AsymmetricProperty)
            .len();
        if asymmetric_count > 0 {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::RemoveUnsupportedConstructs,
                description: format!(
                    "Remove {} asymmetric property axioms (not allowed in QL profile)",
                    asymmetric_count
                ),
                estimated_impact: "Medium - required for QL compliance".to_string(),
            });
        }

        // Check for irreflexive properties (not allowed in QL)
        let irreflexive_count = self
            .ontology
            .axioms_by_type(crate::axioms::AxiomType::IrreflexiveProperty)
            .len();
        if irreflexive_count > 0 {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::RemoveUnsupportedConstructs,
                description: format!(
                    "Remove {} irreflexive property axioms (not allowed in QL profile)",
                    irreflexive_count
                ),
                estimated_impact: "Medium - required for QL compliance".to_string(),
            });
        }

        // Check for complex cardinality restrictions
        let complex_cardinality = self.count_complex_cardinality_restrictions()?;
        if complex_cardinality > 0 {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::SimplifyExpressions,
                description: format!(
                    "Simplify {} complex cardinality restrictions (QL allows only simple ones)",
                    complex_cardinality
                ),
                estimated_impact: "High - critical for QL compliance".to_string(),
            });
        }

        // Check for complex property chains
        let complex_chains = self.count_complex_property_chains()?;
        if complex_chains > 0 {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::SimplifyExpressions,
                description: format!(
                    "Simplify {} complex property chains (QL allows only chains of length ≤ 2)",
                    complex_chains
                ),
                estimated_impact: "Medium - improves QL compliance".to_string(),
            });
        }

        Ok(hints)
    }

    /// Generate detailed optimization report with specific transformations
    pub fn generate_optimization_report(&self) -> OwlResult<QlOptimizationReport> {
        let violations = self.identify_ql_violations()?;
        let hints = self.analyze_optimization_opportunities()?;

        Ok(QlOptimizationReport {
            total_violations: violations.len(),
            violations_by_type: self.categorize_violations(&violations),
            optimization_hints: hints,
            estimated_effort: self.estimate_optimization_effort(&violations),
            can_be_fully_optimized: self.can_be_fully_optimized(&violations),
        })
    }

    /// Identify specific QL violations
    fn identify_ql_violations(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // Check transitive properties - simplified for now
        let transitive_count = self
            .ontology
            .axioms_by_type(crate::axioms::AxiomType::TransitiveProperty)
            .len();
        if transitive_count > 0 {
            violations.push(ProfileViolation {
                violation_type: crate::profiles::common::ProfileViolationType::TransitiveProperties,
                message: format!(
                    "Found {} transitive property axioms (restricted in QL profile)",
                    transitive_count
                ),
                affected_entities: vec![], // Simplified - no specific entities for now
                severity: crate::profiles::common::ViolationSeverity::Error,
            });
        }

        // Check asymmetric properties - simplified for now
        let asymmetric_count = self
            .ontology
            .axioms_by_type(crate::axioms::AxiomType::AsymmetricProperty)
            .len();
        if asymmetric_count > 0 {
            violations.push(ProfileViolation {
                violation_type: crate::profiles::common::ProfileViolationType::AsymmetricProperties,
                message: format!(
                    "Found {} asymmetric property axioms (not allowed in QL profile)",
                    asymmetric_count
                ),
                affected_entities: vec![], // Simplified - no specific entities for now
                severity: crate::profiles::common::ViolationSeverity::Error,
            });
        }

        // Check irreflexive properties - simplified for now
        let irreflexive_count = self
            .ontology
            .axioms_by_type(crate::axioms::AxiomType::IrreflexiveProperty)
            .len();
        if irreflexive_count > 0 {
            violations.push(ProfileViolation {
                violation_type:
                    crate::profiles::common::ProfileViolationType::IrreflexiveProperties,
                message: format!(
                    "Found {} irreflexive property axioms (not allowed in QL profile)",
                    irreflexive_count
                ),
                affected_entities: vec![], // Simplified - no specific entities for now
                severity: crate::profiles::common::ViolationSeverity::Error,
            });
        }

        // Check complex cardinality restrictions
        for axiom in self.ontology.subclass_axioms() {
            violations.extend(self.check_subclass_axiom_cardinality_restrictions(axiom)?);
        }

        // Check complex property chains - simplified for now
        let complex_chains = self.count_complex_property_chains()?;
        if complex_chains > 0 {
            violations.push(ProfileViolation {
                violation_type: crate::profiles::common::ProfileViolationType::PropertyChainAxioms,
                message: format!(
                    "Found {} complex property chains (not allowed in QL profile)",
                    complex_chains
                ),
                affected_entities: vec![], // Simplified - no specific entities for now
                severity: crate::profiles::common::ViolationSeverity::Error,
            });
        }

        Ok(violations)
    }

    /// Check subclass axiom for QL cardinality restrictions
    fn check_subclass_axiom_cardinality_restrictions(
        &self,
        axiom: &crate::axioms::SubClassOfAxiom,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // Check both super and sub classes for complex cardinality restrictions
        violations.extend(self.check_cardinality_restrictions_for_ql(axiom.super_class())?);
        violations.extend(self.check_cardinality_restrictions_for_ql(axiom.sub_class())?);

        Ok(violations)
    }

    /// Check class expression for QL cardinality restrictions
    fn check_cardinality_restrictions_for_ql(
        &self,
        expr: &ClassExpression,
    ) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        match expr {
            ClassExpression::ObjectMinCardinality(cardinality, _) => {
                if *cardinality > 1 {
                    violations.push(ProfileViolation {
                        violation_type: crate::profiles::common::ProfileViolationType::ComplexCardinalityRestrictions,
                        message: format!(
                            "Minimum cardinality > 1 is not allowed in QL profile (found: {})",
                            cardinality
                        ),
                        affected_entities: self.extract_entities_from_expression(expr)?,
                        severity: crate::profiles::common::ViolationSeverity::Error,
                    });
                }
            }
            ClassExpression::ObjectMaxCardinality(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: crate::profiles::common::ProfileViolationType::ComplexCardinalityRestrictions,
                    message: "Maximum cardinality restrictions are not allowed in QL profile".to_string(),
                    affected_entities: self.extract_entities_from_expression(expr)?,
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }
            ClassExpression::ObjectExactCardinality(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: crate::profiles::common::ProfileViolationType::ComplexCardinalityRestrictions,
                    message: "Exact cardinality restrictions are not allowed in QL profile".to_string(),
                    affected_entities: self.extract_entities_from_expression(expr)?,
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }
            ClassExpression::DataMinCardinality(cardinality, _) => {
                if *cardinality > 1 {
                    violations.push(ProfileViolation {
                        violation_type: crate::profiles::common::ProfileViolationType::ComplexCardinalityRestrictions,
                        message: format!(
                            "Data minimum cardinality > 1 is not allowed in QL profile (found: {})",
                            cardinality
                        ),
                        affected_entities: self.extract_entities_from_expression(expr)?,
                        severity: crate::profiles::common::ViolationSeverity::Error,
                    });
                }
            }
            ClassExpression::DataMaxCardinality(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: crate::profiles::common::ProfileViolationType::ComplexCardinalityRestrictions,
                    message: "Data maximum cardinality restrictions are not allowed in QL profile".to_string(),
                    affected_entities: self.extract_entities_from_expression(expr)?,
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }
            ClassExpression::DataExactCardinality(_, _) => {
                violations.push(ProfileViolation {
                    violation_type: crate::profiles::common::ProfileViolationType::ComplexCardinalityRestrictions,
                    message: "Data exact cardinality restrictions are not allowed in QL profile".to_string(),
                    affected_entities: self.extract_entities_from_expression(expr)?,
                    severity: crate::profiles::common::ViolationSeverity::Error,
                });
            }

            // Recursively check nested expressions
            ClassExpression::ObjectSomeValuesFrom(_, class_expr) => {
                violations.extend(self.check_cardinality_restrictions_for_ql(class_expr)?);
            }
            ClassExpression::ObjectAllValuesFrom(_, class_expr) => {
                violations.extend(self.check_cardinality_restrictions_for_ql(class_expr)?);
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    violations.extend(self.check_cardinality_restrictions_for_ql(class_expr)?);
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    violations.extend(self.check_cardinality_restrictions_for_ql(class_expr)?);
                }
            }
            ClassExpression::ObjectComplementOf(class_expr) => {
                violations.extend(self.check_cardinality_restrictions_for_ql(class_expr)?);
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

    /// Count complex cardinality restrictions
    fn count_complex_cardinality_restrictions(&self) -> OwlResult<usize> {
        let mut count = 0;

        for axiom in self.ontology.subclass_axioms() {
            count += self.count_cardinality_restrictions_in_expression(axiom.super_class())?;
            count += self.count_cardinality_restrictions_in_expression(axiom.sub_class())?;
        }

        Ok(count)
    }

    /// Count cardinality restrictions in expression
    fn count_cardinality_restrictions_in_expression(
        &self,
        expr: &ClassExpression,
    ) -> OwlResult<usize> {
        let mut count = 0;

        match expr {
            ClassExpression::ObjectMinCardinality(cardinality, _) => {
                if *cardinality > 1 {
                    count += 1;
                }
            }
            ClassExpression::ObjectMaxCardinality(_, _) => {
                count += 1;
            }
            ClassExpression::ObjectExactCardinality(_, _) => {
                count += 1;
            }
            ClassExpression::DataMinCardinality(cardinality, _) => {
                if *cardinality > 1 {
                    count += 1;
                }
            }
            ClassExpression::DataMaxCardinality(_, _) => {
                count += 1;
            }
            ClassExpression::DataExactCardinality(_, _) => {
                count += 1;
            }

            // Recursively count in nested expressions
            ClassExpression::ObjectSomeValuesFrom(_, class_expr) => {
                count += self.count_cardinality_restrictions_in_expression(class_expr)?;
            }
            ClassExpression::ObjectAllValuesFrom(_, class_expr) => {
                count += self.count_cardinality_restrictions_in_expression(class_expr)?;
            }
            ClassExpression::ObjectIntersectionOf(classes) => {
                for class_expr in classes {
                    count += self.count_cardinality_restrictions_in_expression(class_expr)?;
                }
            }
            ClassExpression::ObjectUnionOf(classes) => {
                for class_expr in classes {
                    count += self.count_cardinality_restrictions_in_expression(class_expr)?;
                }
            }
            ClassExpression::ObjectComplementOf(class_expr) => {
                count += self.count_cardinality_restrictions_in_expression(class_expr)?;
            }
            _ => {}
        }

        Ok(count)
    }

    /// Count complex property chains - simplified for now
    fn count_complex_property_chains(&self) -> OwlResult<usize> {
        // For now, return a conservative estimate
        // In practice, we would analyze SubObjectProperty axioms for complex chains
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
    #[allow(clippy::only_used_in_recursion)]
    fn estimate_optimization_effort(&self, violations: &[ProfileViolation]) -> OptimizationEffort {
        let high_effort_violations = violations
            .iter()
            .filter(|v| {
                matches!(
                    v.violation_type,
                    crate::profiles::common::ProfileViolationType::ComplexCardinalityRestrictions
                        | crate::profiles::common::ProfileViolationType::TransitiveProperties
                )
            })
            .count();

        if high_effort_violations > 5 {
            OptimizationEffort::High
        } else if high_effort_violations > 2 {
            OptimizationEffort::Medium
        } else {
            OptimizationEffort::Low
        }
    }

    /// Check if ontology can be fully optimized for QL
    #[allow(clippy::only_used_in_recursion)]
    fn can_be_fully_optimized(&self, violations: &[ProfileViolation]) -> bool {
        // Most QL violations can be resolved through transformation
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

/// QL Optimization Report
#[derive(Debug, Clone)]
pub struct QlOptimizationReport {
    pub total_violations: usize,
    pub violations_by_type: std::collections::HashMap<String, usize>,
    pub optimization_hints: Vec<OptimizationHint>,
    pub estimated_effort: OptimizationEffort,
    pub can_be_fully_optimized: bool,
}
