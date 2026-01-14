//! Supply Chain Analytics Module
//!
//! This module provides risk assessment algorithms, supplier performance analytics,
//! quality prediction models, and compliance monitoring capabilities.

use super::{ComplianceStatus, QualityScore, RiskLevel, TrendAnalysis};
use crate::knowledge_graph::{KnowledgeEntity, KnowledgeGraph, KnowledgeRelationship};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Supply chain analyzer for risk assessment and performance analytics
pub struct SupplyChainAnalyzer {
    entities: HashMap<String, KnowledgeEntity>,
    relationships: Vec<KnowledgeRelationship>,
}

impl SupplyChainAnalyzer {
    /// Create a new supply chain analyzer
    pub fn new(knowledge_graph: &KnowledgeGraph) -> Self {
        Self {
            entities: knowledge_graph.entities.clone(),
            relationships: knowledge_graph.relationships.clone(),
        }
    }

    /// Calculate comprehensive supply chain metrics
    pub fn calculate_metrics(&self) -> Result<SupplyChainMetrics> {
        let risk_assessment = self.assess_overall_risk()?;
        let supplier_performance = self.analyze_supplier_performance()?;
        let quality_metrics = self.calculate_quality_metrics()?;
        let compliance_status = self.check_compliance_status()?;
        let traceability_coverage = self.calculate_traceability_coverage()?;

        Ok(SupplyChainMetrics {
            risk_assessment,
            supplier_performance,
            quality_metrics,
            compliance_status,
            traceability_coverage,
            efficiency_metrics: self.calculate_efficiency_metrics()?,
            visibility_score: self.calculate_visibility_score()?,
        })
    }

    /// Assess risk for a specific batch
    pub fn assess_batch_risk(&self, batch_id: &str) -> Result<RiskAssessment> {
        // Find the batch entity
        let batch_entity = self
            .entities
            .values()
            .find(|e| {
                e.entity_type == "ProductBatch"
                    && e.properties.get("batchId").is_some_and(|id| id == batch_id)
            })
            .ok_or_else(|| anyhow::anyhow!("Batch not found: {}", batch_id))?;

        let mut risk_factors = Vec::new();
        let mut total_risk_score = 0.0;

        // Analyze supplier risk
        let supplier_risk = self.analyze_supplier_risk(batch_entity)?;
        risk_factors.push(RiskFactor {
            category: "Supplier".to_string(),
            description: "Supplier reliability and history".to_string(),
            score: supplier_risk,
            impact: if supplier_risk > 0.7 {
                "High".to_string()
            } else {
                "Medium".to_string()
            },
        });
        total_risk_score += supplier_risk * 0.3;

        // Analyze transportation risk
        let transport_risk = self.analyze_transport_risk(batch_entity)?;
        risk_factors.push(RiskFactor {
            category: "Transportation".to_string(),
            description: "Transportation conditions and delays".to_string(),
            score: transport_risk,
            impact: if transport_risk > 0.6 {
                "High".to_string()
            } else {
                "Low".to_string()
            },
        });
        total_risk_score += transport_risk * 0.2;

        // Analyze quality risk
        let quality_risk = self.analyze_quality_risk(batch_entity)?;
        risk_factors.push(RiskFactor {
            category: "Quality".to_string(),
            description: "Quality control and testing results".to_string(),
            score: quality_risk,
            impact: if quality_risk > 0.5 {
                "High".to_string()
            } else {
                "Low".to_string()
            },
        });
        total_risk_score += quality_risk * 0.3;

        // Analyze environmental risk
        let environmental_risk = self.analyze_environmental_risk(batch_entity)?;
        risk_factors.push(RiskFactor {
            category: "Environmental".to_string(),
            description: "Environmental conditions during production and transport".to_string(),
            score: environmental_risk,
            impact: if environmental_risk > 0.4 {
                "Medium".to_string()
            } else {
                "Low".to_string()
            },
        });
        total_risk_score += environmental_risk * 0.2;

        let recommendations = self.generate_risk_recommendations(total_risk_score, &risk_factors);

        Ok(RiskAssessment {
            entity_id: batch_entity.uri.clone(),
            overall_risk_score: total_risk_score,
            risk_level: RiskLevel::from_score(total_risk_score),
            risk_factors,
            recommendations,
            last_updated: Utc::now(),
        })
    }

    /// Analyze supplier performance
    pub fn analyze_supplier_performance(&self) -> Result<Vec<SupplierPerformance>> {
        let mut supplier_performance = Vec::new();

        // Get all supplier entities
        let suppliers: Vec<_> = self
            .entities
            .values()
            .filter(|e| {
                matches!(
                    e.entity_type.as_str(),
                    "Farmer" | "Manufacturer" | "LogisticsProvider"
                )
            })
            .collect();

        for supplier in suppliers {
            let performance = self.calculate_supplier_metrics(supplier)?;
            supplier_performance.push(performance);
        }

        // Sort by overall score (descending)
        supplier_performance.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap());

        Ok(supplier_performance)
    }

    /// Calculate quality metrics across the supply chain
    pub fn calculate_quality_metrics(&self) -> Result<QualityMetrics> {
        let quality_checks: Vec<_> = self
            .entities
            .values()
            .filter(|e| e.entity_type == "QualityCheck")
            .collect();

        let total_checks = quality_checks.len();
        let mut passed_checks = 0;
        let mut quality_scores = Vec::new();

        for check in &quality_checks {
            // Simulate quality check results (in real system would parse from properties)
            let score = self.extract_quality_score(check);
            quality_scores.push(score);

            if score >= 0.7 {
                passed_checks += 1;
            }
        }

        let pass_rate = if total_checks > 0 {
            passed_checks as f64 / total_checks as f64
        } else {
            1.0
        };

        let average_score = if !quality_scores.is_empty() {
            quality_scores.iter().sum::<f64>() / quality_scores.len() as f64
        } else {
            0.8 // Default score
        };

        Ok(QualityMetrics {
            overall_quality_score: QualityScore::from_score(average_score),
            quality_pass_rate: pass_rate,
            total_quality_checks: total_checks,
            failed_checks: total_checks - passed_checks,
            quality_trends: self.analyze_quality_trends()?,
            defect_rate: 1.0 - pass_rate,
        })
    }

    /// Check compliance status across the supply chain
    pub fn check_compliance_status(&self) -> Result<ComplianceReport> {
        let mut compliance_checks = Vec::new();
        let mut total_compliant = 0;
        let mut total_checks = 0;

        // Check certificate compliance
        let certificates: Vec<_> = self
            .entities
            .values()
            .filter(|e| e.entity_type == "Certificate")
            .collect();

        for cert in certificates {
            total_checks += 1;
            let is_compliant = self.check_certificate_validity(cert);
            if is_compliant {
                total_compliant += 1;
            }

            compliance_checks.push(ComplianceCheck {
                check_type: "Certificate".to_string(),
                entity_id: cert.uri.clone(),
                status: if is_compliant {
                    ComplianceStatus::Compliant
                } else {
                    ComplianceStatus::NonCompliant
                },
                details: "Certificate validity check".to_string(),
                last_checked: Utc::now(),
            });
        }

        // Check regulatory compliance for activities
        let activities: Vec<_> = self
            .entities
            .values()
            .filter(|e| {
                matches!(
                    e.entity_type.as_str(),
                    "ProcessingActivity" | "TransportActivity"
                )
            })
            .collect();

        for activity in activities {
            total_checks += 1;
            let is_compliant = self.check_activity_compliance(activity);
            if is_compliant {
                total_compliant += 1;
            }

            compliance_checks.push(ComplianceCheck {
                check_type: "Activity".to_string(),
                entity_id: activity.uri.clone(),
                status: if is_compliant {
                    ComplianceStatus::Compliant
                } else {
                    ComplianceStatus::NonCompliant
                },
                details: "Regulatory compliance check".to_string(),
                last_checked: Utc::now(),
            });
        }

        let compliance_rate = if total_checks > 0 {
            total_compliant as f64 / total_checks as f64
        } else {
            1.0
        };

        Ok(ComplianceReport {
            overall_compliance_rate: compliance_rate,
            total_checks,
            compliant_checks: total_compliant,
            non_compliant_checks: total_checks - total_compliant,
            compliance_checks,
            critical_violations: self.identify_critical_violations()?,
        })
    }

    /// Calculate traceability coverage
    pub fn calculate_traceability_coverage(&self) -> Result<TraceabilityCoverage> {
        let product_batches: Vec<_> = self
            .entities
            .values()
            .filter(|e| e.entity_type == "ProductBatch")
            .collect();

        let mut fully_traceable = 0;
        let mut partially_traceable = 0;
        let mut coverage_details = Vec::new();

        for batch in &product_batches {
            let coverage = self.calculate_batch_traceability(batch)?;
            coverage_details.push(coverage.clone());

            if coverage.coverage_percentage >= 0.9 {
                fully_traceable += 1;
            } else if coverage.coverage_percentage >= 0.5 {
                partially_traceable += 1;
            }
        }

        let total_batches = product_batches.len();
        let overall_coverage = if total_batches > 0 {
            coverage_details
                .iter()
                .map(|c| c.coverage_percentage)
                .sum::<f64>()
                / total_batches as f64
        } else {
            1.0
        };

        Ok(TraceabilityCoverage {
            overall_coverage_percentage: overall_coverage,
            fully_traceable_batches: fully_traceable,
            partially_traceable_batches: partially_traceable,
            non_traceable_batches: total_batches - fully_traceable - partially_traceable,
            coverage_details,
        })
    }

    /// Calculate efficiency metrics
    fn calculate_efficiency_metrics(&self) -> Result<EfficiencyMetrics> {
        let activities: Vec<_> = self
            .entities
            .values()
            .filter(|e| {
                matches!(
                    e.entity_type.as_str(),
                    "ProcessingActivity" | "TransportActivity"
                )
            })
            .collect();

        let mut processing_times = Vec::new();
        let mut transport_times = Vec::new();

        for activity in activities {
            if let Some(_recorded_at) = activity.properties.get("recordedAt") {
                // Simulate processing/transport time calculation
                let duration = self.calculate_activity_duration(activity);

                match activity.entity_type.as_str() {
                    "ProcessingActivity" => processing_times.push(duration),
                    "TransportActivity" => transport_times.push(duration),
                    _ => {}
                }
            }
        }

        let avg_processing_time = if !processing_times.is_empty() {
            processing_times.iter().sum::<f64>() / processing_times.len() as f64
        } else {
            24.0 // Default 24 hours
        };

        let avg_transport_time = if !transport_times.is_empty() {
            transport_times.iter().sum::<f64>() / transport_times.len() as f64
        } else {
            48.0 // Default 48 hours
        };

        Ok(EfficiencyMetrics {
            average_processing_time_hours: avg_processing_time,
            average_transport_time_hours: avg_transport_time,
            total_cycle_time_hours: avg_processing_time + avg_transport_time,
            efficiency_score: self
                .calculate_efficiency_score(avg_processing_time, avg_transport_time),
            bottlenecks: self.identify_bottlenecks()?,
        })
    }

    /// Calculate visibility score
    fn calculate_visibility_score(&self) -> Result<f64> {
        let total_entities = self.entities.len();
        let total_relationships = self.relationships.len();

        // Score based on data completeness and connectivity
        let completeness_score = self.calculate_data_completeness();
        let connectivity_score = if total_entities > 0 {
            total_relationships as f64 / total_entities as f64
        } else {
            0.0
        };

        // Normalize and combine scores
        let visibility_score =
            (completeness_score * 0.6 + connectivity_score.min(1.0) * 0.4).min(1.0);

        Ok(visibility_score)
    }

    fn analyze_supplier_risk(&self, batch: &KnowledgeEntity) -> Result<f64> {
        // Analyze supplier risk based on supplier history and characteristics
        // Find suppliers linked to this batch through relationships

        // For now, calculate risk based on batch properties
        let mut risk_score = 0.1; // Base risk (low)

        // Check if batch is from a new supplier (no historical data)
        let supplier_history = batch
            .properties
            .get("supplierHistory")
            .and_then(|h| h.parse::<u32>().ok())
            .unwrap_or(0);

        if supplier_history == 0 {
            risk_score += 0.3; // Increase risk for unknown suppliers
        } else if supplier_history < 10 {
            risk_score += 0.1; // Slight increase for low history
        }

        // Check if supplier has certifications
        let has_certifications = batch
            .properties
            .get("certified")
            .map(|c| c == "true")
            .unwrap_or(false);

        if !has_certifications {
            risk_score += 0.2; // Higher risk for uncertified suppliers
        }

        // Check supplier location (imported goods have higher risk)
        let is_imported = batch
            .properties
            .get("countryOfOrigin")
            .map(|country| country != "US" && country != "USA" && country != "local")
            .unwrap_or(false);

        if is_imported {
            risk_score += 0.1; // Slight increase for imports
        }

        // Normalize to 0-1 range
        let normalized_risk = (risk_score / 1.0_f64).min(1.0_f64);

        tracing::debug!(
            "Supplier risk for batch {}: {:.2} (history: {}, certified: {}, imported: {})",
            batch.uri,
            normalized_risk,
            supplier_history,
            has_certifications,
            is_imported
        );

        Ok(normalized_risk)
    }

    fn analyze_transport_risk(&self, batch: &KnowledgeEntity) -> Result<f64> {
        // Analyze transportation risk based on distance, mode, and conditions
        let mut risk_score = 0.15; // Base risk

        // Get transport mode
        let transport_mode = batch
            .properties
            .get("transportMode")
            .map(|s| s.as_str())
            .unwrap_or("truck");

        // Different transport modes have different risk levels
        match transport_mode {
            "rail" | "ship" => risk_score += 0.0,  // Low risk
            "truck" => risk_score += 0.05,         // Slight risk
            "air" | "plane" => risk_score += 0.25, // Higher risk
            _ => risk_score += 0.10,               // Unknown mode
        }

        // Check if there are temperature requirements
        let requires_temperature_control = batch.properties.get("temperatureMin").is_some();

        if requires_temperature_control {
            risk_score += 0.10; // Higher risk for temperature-sensitive goods
        }

        // Check transport duration (if available)
        if let Some(duration_str) = batch.properties.get("duration") {
            if let Ok(duration_hours) = duration_str.parse::<f64>() {
                // Longer durations increase risk
                if duration_hours > 48.0 {
                    risk_score += 0.10;
                } else if duration_hours > 24.0 {
                    risk_score += 0.05;
                }
            }
        }

        // Normalize to 0-1 range
        let normalized_risk = (risk_score / 1.0_f64).min(1.0_f64);

        tracing::debug!(
            "Transport risk for batch {}: {:.2} (mode: {}, temp_control: {})",
            batch.uri,
            normalized_risk,
            transport_mode,
            requires_temperature_control
        );

        Ok(normalized_risk)
    }

    fn analyze_quality_risk(&self, batch: &KnowledgeEntity) -> Result<f64> {
        // Analyze quality risk based on product type and historical quality data
        let mut risk_score = 0.1; // Base risk (low)

        // Check product type
        let product_type = batch
            .properties
            .get("product")
            .map(|p| p.to_lowercase())
            .unwrap_or_default();

        // Perishable products have higher quality risk
        if product_type.contains("fresh")
            || product_type.contains("dairy")
            || product_type.contains("meat")
        {
            risk_score += 0.15;
        } else if product_type.contains("frozen") {
            risk_score += 0.10;
        } else if product_type.contains("organic") {
            risk_score += 0.05; // Organic has slightly higher variability risk
        }

        // Check batch age
        if let Some(production_date_str) = batch.properties.get("productionDate") {
            if let Ok(production_date) = chrono::DateTime::parse_from_rfc3339(production_date_str) {
                // Convert to UTC and calculate age
                let production_date_utc = production_date.with_timezone(&chrono::Utc);
                let age_days = (chrono::Utc::now() - production_date_utc).num_days();

                if age_days > 30 {
                    risk_score += 0.15; // Older batches have higher quality risk
                } else if age_days > 14 {
                    risk_score += 0.08;
                } else if age_days < 1 {
                    risk_score += 0.02; // Very fresh batches have minimal risk
                }
            }
        }

        // Check if quality checks exist
        let has_quality_checks = self
            .entities
            .values()
            .filter(|e| e.entity_type == "QualityCheck")
            .any(|check| {
                check
                    .properties
                    .get("batchId")
                    .map(|id| id == batch.properties.get("batchId").unwrap_or(&String::new()))
                    .unwrap_or(false)
            });

        if !has_quality_checks {
            risk_score += 0.20; // Higher risk if no quality checks performed
        }

        // Normalize to 0-1 range
        let normalized_risk = (risk_score / 1.0_f64).min(1.0_f64);

        tracing::debug!(
            "Quality risk for batch {}: {:.2} (product: {}, checks: {})",
            batch.uri,
            normalized_risk,
            product_type,
            has_quality_checks
        );

        Ok(normalized_risk)
    }

    fn analyze_environmental_risk(&self, batch: &KnowledgeEntity) -> Result<f64> {
        // Analyze environmental risk based on production conditions and location
        let mut risk_score = 0.15; // Base risk

        // Check production location
        let location = batch
            .properties
            .get("location")
            .map(|l| l.to_lowercase())
            .unwrap_or_default();

        // Certain locations may have higher environmental risks
        if location.contains("flood") || location.contains("drought") {
            risk_score += 0.20; // Natural disaster risk areas
        } else if location.contains("tropical") {
            risk_score += 0.10; // Tropical regions (higher pest/decay risk)
        }

        // Check if organic (generally lower environmental risk from chemicals)
        let is_organic = batch
            .properties
            .get("productionMethod")
            .map(|m| m == "organic")
            .unwrap_or(false);

        if !is_organic {
            risk_score += 0.15; // Higher risk from chemical use
        }

        // Check storage conditions
        let has_storage_info = batch.properties.get("storageTemperature").is_some()
            || batch.properties.get("storageHumidity").is_some();

        if !has_storage_info {
            risk_score += 0.10; // Unknown storage conditions
        }

        // Normalize to 0-1 range
        let normalized_risk = (risk_score / 1.0_f64).min(1.0_f64);

        tracing::debug!(
            "Environmental risk for batch {}: {:.2} (location: {}, organic: {})",
            batch.uri,
            normalized_risk,
            location,
            is_organic
        );

        Ok(normalized_risk)
    }

    fn generate_risk_recommendations(
        &self,
        risk_score: f64,
        risk_factors: &[RiskFactor],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if risk_score > 0.7 {
            recommendations.push("Immediate attention required - high risk detected".to_string());
        }

        for factor in risk_factors {
            if factor.score > 0.6 {
                recommendations.push(format!(
                    "Address {} risk: {}",
                    factor.category, factor.description
                ));
            }
        }

        if recommendations.is_empty() {
            recommendations.push("Continue monitoring - risk levels acceptable".to_string());
        }

        recommendations
    }

    fn assess_overall_risk(&self) -> Result<RiskAssessment> {
        // Calculate overall supply chain risk
        let mut total_risk = 0.0;
        let mut risk_factors = Vec::new();

        // Supplier risk
        let supplier_risk = 0.2;
        risk_factors.push(RiskFactor {
            category: "Suppliers".to_string(),
            description: "Overall supplier reliability".to_string(),
            score: supplier_risk,
            impact: "Medium".to_string(),
        });
        total_risk += supplier_risk * 0.4;

        // Quality risk
        let quality_risk = 0.15;
        risk_factors.push(RiskFactor {
            category: "Quality".to_string(),
            description: "Quality control effectiveness".to_string(),
            score: quality_risk,
            impact: "Low".to_string(),
        });
        total_risk += quality_risk * 0.3;

        // Compliance risk
        let compliance_risk = 0.1;
        risk_factors.push(RiskFactor {
            category: "Compliance".to_string(),
            description: "Regulatory compliance status".to_string(),
            score: compliance_risk,
            impact: "Low".to_string(),
        });
        total_risk += compliance_risk * 0.3;

        let recommendations = self.generate_risk_recommendations(total_risk, &risk_factors);

        Ok(RiskAssessment {
            entity_id: "supply_chain_overall".to_string(),
            overall_risk_score: total_risk,
            risk_level: RiskLevel::from_score(total_risk),
            risk_factors,
            recommendations,
            last_updated: Utc::now(),
        })
    }

    fn calculate_supplier_metrics(
        &self,
        supplier: &KnowledgeEntity,
    ) -> Result<SupplierPerformance> {
        // Calculate supplier performance metrics from entity data and relationships
        let mut quality_score = 0.85; // Default score
        let mut delivery_score = 0.90;
        let mut compliance_score = 0.95;
        let mut total_orders = 10;
        let mut on_time_deliveries = 9;
        let mut quality_issues = 1;

        // Extract metrics from supplier properties if available
        if let Some(avg_quality) = supplier.properties.get("averageQualityScore") {
            quality_score = avg_quality.parse::<f64>().unwrap_or(0.85);
        }

        if let Some(on_time_rate) = supplier.properties.get("onTimeDeliveryRate") {
            delivery_score = on_time_rate.parse::<f64>().unwrap_or(0.90);
        }

        if let Some(compliance_rate) = supplier.properties.get("complianceRate") {
            compliance_score = compliance_rate.parse::<f64>().unwrap_or(0.95);
        }

        if let Some(orders) = supplier.properties.get("totalOrders") {
            total_orders = orders.parse::<usize>().unwrap_or(10);
        }

        if let Some(on_time) = supplier.properties.get("onTimeDeliveries") {
            on_time_deliveries = on_time
                .parse::<usize>()
                .unwrap_or((total_orders as f64 * delivery_score) as usize);
        }

        if let Some(issues) = supplier.properties.get("qualityIssues") {
            quality_issues = issues.parse::<usize>().unwrap_or(1);
        }

        // Calculate overall score as weighted average
        let overall_score = quality_score * 0.4 + delivery_score * 0.3 + compliance_score * 0.3;

        // Ensure on_time_deliveries doesn't exceed total_orders
        on_time_deliveries = on_time_deliveries.min(total_orders);

        tracing::debug!(
            "Supplier metrics for {}: quality={:.2}, delivery={:.2}, compliance={:.2}, overall={:.2}",
            supplier.uri,
            quality_score,
            delivery_score,
            compliance_score,
            overall_score
        );

        Ok(SupplierPerformance {
            supplier_id: supplier.uri.clone(),
            supplier_name: supplier
                .label
                .clone()
                .unwrap_or_else(|| "Unknown".to_string()),
            supplier_type: supplier.entity_type.clone(),
            overall_score,
            quality_score,
            delivery_performance: delivery_score,
            compliance_score,
            risk_level: RiskLevel::from_score(1.0 - overall_score),
            total_orders,
            on_time_deliveries,
            quality_issues,
        })
    }

    fn extract_quality_score(&self, check: &KnowledgeEntity) -> f64 {
        // Extract quality score from QualityCheck entity properties
        check
            .properties
            .get("score")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or_else(|| {
                // If no score property, try to derive from result
                check
                    .properties
                    .get("result")
                    .map(|result| match result.to_lowercase().as_str() {
                        "passed" | "pass" => 0.95,
                        "failed" | "fail" => 0.25,
                        "partial" | "conditional" => 0.60,
                        _ => 0.70, // Default mid-range if unclear
                    })
                    .unwrap_or(0.70)
            })
    }

    fn analyze_quality_trends(&self) -> Result<TrendAnalysis> {
        // Simplified trend analysis
        Ok(TrendAnalysis {
            direction: super::TrendDirection::Stable,
            strength: 0.1,
            confidence: 0.8,
            forecast: Vec::new(),
        })
    }

    fn check_certificate_validity(&self, cert: &KnowledgeEntity) -> bool {
        // Check certificate validity based on expiry date
        let now = chrono::Utc::now();

        cert.properties
            .get("expiryDate")
            .and_then(|expiry_str| chrono::DateTime::parse_from_rfc3339(expiry_str).ok())
            .map(|expiry_date| {
                let is_valid = expiry_date > now;

                if !is_valid {
                    tracing::warn!("Certificate {} expired on {}", cert.uri, expiry_date);
                }

                is_valid
            })
            .unwrap_or(true) // If no expiry date, assume valid (might be a permanent certificate)
    }

    fn check_activity_compliance(&self, activity: &KnowledgeEntity) -> bool {
        // Check activity compliance based on various requirements
        let mut is_compliant = true;

        // Check if activity has required documentation
        if activity.properties.get("documentationDate").is_none() {
            // Only require documentation for certain activity types
            if matches!(
                activity.entity_type.as_str(),
                "TransportActivity" | "ProcessingActivity"
            ) {
                tracing::warn!("Activity {} missing required documentation", activity.uri);
                is_compliant = false;
            }
        }

        // Check if activity has temperature controls (for temperature-sensitive products)
        if activity.properties.get("temperatureMin").is_none()
            && activity
                .properties
                .get("productType")
                .map(|t| t.contains("fresh") || t.contains("perishable"))
                .unwrap_or(false)
        {
            tracing::warn!(
                "Activity {} transporting perishable goods without temperature controls",
                activity.uri
            );
            is_compliant = false;
        }

        // Check regulatory compliance
        if let Some(regulatory_status) = activity.properties.get("regulatoryStatus") {
            if regulatory_status.to_lowercase() != "compliant" {
                tracing::warn!(
                    "Activity {} has non-compliant regulatory status: {}",
                    activity.uri,
                    regulatory_status
                );
                is_compliant = false;
            }
        }

        is_compliant
    }

    fn identify_critical_violations(&self) -> Result<Vec<String>> {
        // No critical violations in simplified implementation
        Ok(Vec::new())
    }

    fn calculate_batch_traceability(&self, batch: &KnowledgeEntity) -> Result<BatchTraceability> {
        // Simplified traceability calculation
        Ok(BatchTraceability {
            batch_id: batch
                .properties
                .get("batchId")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string()),
            coverage_percentage: 0.95,
            missing_data_points: vec!["Environmental sensor data".to_string()],
            trace_completeness: "High".to_string(),
        })
    }

    fn calculate_activity_duration(&self, _activity: &KnowledgeEntity) -> f64 {
        // Simplified duration calculation
        24.0 // 24 hours
    }

    fn calculate_efficiency_score(&self, processing_time: f64, transport_time: f64) -> f64 {
        // Simplified efficiency score calculation
        let total_time = processing_time + transport_time;
        let optimal_time = 48.0; // 48 hours optimal

        if total_time <= optimal_time {
            1.0
        } else {
            optimal_time / total_time
        }
    }

    fn identify_bottlenecks(&self) -> Result<Vec<String>> {
        // Simplified bottleneck identification
        Ok(vec!["Quality check processing".to_string()])
    }

    fn calculate_data_completeness(&self) -> f64 {
        // Calculate percentage of entities with complete data
        let mut complete_entities = 0;

        for entity in self.entities.values() {
            if !entity.properties.is_empty() {
                complete_entities += 1;
            }
        }

        if self.entities.is_empty() {
            1.0
        } else {
            complete_entities as f64 / self.entities.len() as f64
        }
    }
}

/// Supply chain metrics
#[derive(Debug, serde::Serialize)]
pub struct SupplyChainMetrics {
    pub risk_assessment: RiskAssessment,
    pub supplier_performance: Vec<SupplierPerformance>,
    pub quality_metrics: QualityMetrics,
    pub compliance_status: ComplianceReport,
    pub traceability_coverage: TraceabilityCoverage,
    pub efficiency_metrics: EfficiencyMetrics,
    pub visibility_score: f64,
}

/// Risk assessment result
#[derive(Debug, serde::Serialize)]
pub struct RiskAssessment {
    pub entity_id: String,
    pub overall_risk_score: f64,
    pub risk_level: RiskLevel,
    pub risk_factors: Vec<RiskFactor>,
    pub recommendations: Vec<String>,
    pub last_updated: DateTime<Utc>,
}

/// Individual risk factor
#[derive(Debug, serde::Serialize)]
pub struct RiskFactor {
    pub category: String,
    pub description: String,
    pub score: f64,
    pub impact: String,
}

/// Supplier performance metrics
#[derive(Debug, serde::Serialize)]
pub struct SupplierPerformance {
    pub supplier_id: String,
    pub supplier_name: String,
    pub supplier_type: String,
    pub overall_score: f64,
    pub quality_score: f64,
    pub delivery_performance: f64,
    pub compliance_score: f64,
    pub risk_level: RiskLevel,
    pub total_orders: usize,
    pub on_time_deliveries: usize,
    pub quality_issues: usize,
}

/// Quality metrics
#[derive(Debug, serde::Serialize)]
pub struct QualityMetrics {
    pub overall_quality_score: QualityScore,
    pub quality_pass_rate: f64,
    pub total_quality_checks: usize,
    pub failed_checks: usize,
    pub quality_trends: TrendAnalysis,
    pub defect_rate: f64,
}

/// Compliance report
#[derive(Debug, serde::Serialize)]
pub struct ComplianceReport {
    pub overall_compliance_rate: f64,
    pub total_checks: usize,
    pub compliant_checks: usize,
    pub non_compliant_checks: usize,
    pub compliance_checks: Vec<ComplianceCheck>,
    pub critical_violations: Vec<String>,
}

/// Individual compliance check
#[derive(Debug, serde::Serialize)]
pub struct ComplianceCheck {
    pub check_type: String,
    pub entity_id: String,
    pub status: ComplianceStatus,
    pub details: String,
    pub last_checked: DateTime<Utc>,
}

/// Traceability coverage
#[derive(Debug, serde::Serialize)]
pub struct TraceabilityCoverage {
    pub overall_coverage_percentage: f64,
    pub fully_traceable_batches: usize,
    pub partially_traceable_batches: usize,
    pub non_traceable_batches: usize,
    pub coverage_details: Vec<BatchTraceability>,
}

/// Batch traceability details
#[derive(Debug, Clone, serde::Serialize)]
pub struct BatchTraceability {
    pub batch_id: String,
    pub coverage_percentage: f64,
    pub missing_data_points: Vec<String>,
    pub trace_completeness: String,
}

/// Efficiency metrics
#[derive(Debug, serde::Serialize)]
pub struct EfficiencyMetrics {
    pub average_processing_time_hours: f64,
    pub average_transport_time_hours: f64,
    pub total_cycle_time_hours: f64,
    pub efficiency_score: f64,
    pub bottlenecks: Vec<String>,
}
