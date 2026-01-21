//! Comprehensive tests for analytics modules
//!
//! This test suite covers:
//! - Predictive analytics (forecasting, predictions, optimization)
//! - Supply chain analytics (risk, performance, quality, compliance)
//! - Sustainability analytics (carbon footprint, ESG, environmental impact)

use provchain_org::analytics::predictive::PredictiveAnalyzer;
use provchain_org::analytics::supply_chain::SupplyChainAnalyzer;
use provchain_org::analytics::sustainability::SustainabilityTracker;
use provchain_org::analytics::{AnalyticsEngine, ComplianceStatus, RiskLevel};
use provchain_org::knowledge_graph::{KnowledgeEntity, KnowledgeGraph};
use provchain_org::storage::rdf_store::RDFStore;

/// Create a test knowledge graph with sample entities
fn create_test_knowledge_graph() -> KnowledgeGraph {
    let mut graph = KnowledgeGraph::new();

    // Add a farmer entity
    let farmer = KnowledgeEntity {
        uri: "urn:provchain:entity:farmer:1".to_string(),
        entity_type: "Farmer".to_string(),
        label: Some("Green Valley Farms".to_string()),
        properties: vec![
            ("location".to_string(), "California".to_string()),
            ("certified".to_string(), "true".to_string()),
        ]
        .into_iter()
        .collect(),
        confidence_score: 1.0,
    };
    let _ = graph.add_entity(farmer);

    // Add a product batch
    let batch = KnowledgeEntity {
        uri: "urn:provchain:entity:batch:1".to_string(),
        entity_type: "ProductBatch".to_string(),
        label: Some("Batch 001 - Organic Tomatoes".to_string()),
        properties: vec![
            ("batchId".to_string(), "BATCH-001".to_string()),
            ("product".to_string(), "Organic Tomatoes".to_string()),
            ("quantity".to_string(), "1000".to_string()),
            ("productionDate".to_string(), "2025-01-01".to_string()),
        ]
        .into_iter()
        .collect(),
        confidence_score: 1.0,
    };
    let _ = graph.add_entity(batch);

    // Add a quality check
    let quality_check = KnowledgeEntity {
        uri: "urn:provchain:entity:quality:1".to_string(),
        entity_type: "QualityCheck".to_string(),
        label: Some("Quality Check for Batch 001".to_string()),
        properties: vec![
            ("checkDate".to_string(), "2025-01-02".to_string()),
            ("result".to_string(), "passed".to_string()),
            ("score".to_string(), "0.92".to_string()),
        ]
        .into_iter()
        .collect(),
        confidence_score: 1.0,
    };
    let _ = graph.add_entity(quality_check);

    // Add a transport activity
    let transport = KnowledgeEntity {
        uri: "urn:provchain:entity:transport:1".to_string(),
        entity_type: "TransportActivity".to_string(),
        label: Some("Transport to Distribution Center".to_string()),
        properties: vec![
            ("transportDate".to_string(), "2025-01-03".to_string()),
            ("from".to_string(), "California".to_string()),
            ("to".to_string(), "New York".to_string()),
            ("duration".to_string(), "48".to_string()),
        ]
        .into_iter()
        .collect(),
        confidence_score: 1.0,
    };
    let _ = graph.add_entity(transport);

    // Add a certificate
    let certificate = KnowledgeEntity {
        uri: "urn:provchain:entity:cert:1".to_string(),
        entity_type: "Certificate".to_string(),
        label: Some("Organic Certification".to_string()),
        properties: vec![
            ("issueDate".to_string(), "2024-01-01".to_string()),
            ("expiryDate".to_string(), "2025-12-31".to_string()),
            ("type".to_string(), "Organic".to_string()),
        ]
        .into_iter()
        .collect(),
        confidence_score: 1.0,
    };
    let _ = graph.add_entity(certificate);

    // Add relationships
    let rel1 = provchain_org::knowledge_graph::KnowledgeRelationship {
        subject: "urn:provchain:entity:farmer:1".to_string(),
        predicate: "produced".to_string(),
        object: "urn:provchain:entity:batch:1".to_string(),
        confidence_score: 1.0,
        temporal_info: None,
    };
    let _ = graph.add_relationship(rel1);

    let rel2 = provchain_org::knowledge_graph::KnowledgeRelationship {
        subject: "urn:provchain:entity:batch:1".to_string(),
        predicate: "validatedBy".to_string(),
        object: "urn:provchain:entity:quality:1".to_string(),
        confidence_score: 1.0,
        temporal_info: None,
    };
    let _ = graph.add_relationship(rel2);

    graph
}

mod predictive_analytics_tests {
    use super::*;

    #[test]
    fn test_predictive_analyzer_creation() {
        let kg = create_test_knowledge_graph();
        let analyzer = PredictiveAnalyzer::new(&kg);

        // Should successfully create analyzer
        let _ = analyzer;
    }

    #[test]
    fn test_demand_forecast_generation() {
        let kg = create_test_knowledge_graph();
        let analyzer = PredictiveAnalyzer::new(&kg);

        let forecast = analyzer.forecast_demand(30).unwrap();

        // Should generate forecast for 30 days
        assert_eq!(forecast.forecast_period_days, 30);
        assert_eq!(forecast.forecast_points.len(), 30);

        // Forecast should have reasonable values
        assert!(forecast.forecast_accuracy > 0.0);
        assert!(forecast.forecast_accuracy <= 1.0);
        assert!(!forecast.model_type.is_empty());
        assert!(!forecast.key_drivers.is_empty());
    }

    #[test]
    fn test_demand_forecast_points_have_confidence_intervals() {
        let kg = create_test_knowledge_graph();
        let analyzer = PredictiveAnalyzer::new(&kg);

        let forecast = analyzer.forecast_demand(7).unwrap();

        for point in &forecast.forecast_points {
            // Each forecast point should have a confidence interval
            assert!(point.confidence_interval.0 < point.predicted_demand);
            assert!(point.confidence_interval.1 > point.predicted_demand);
            assert!(!point.factors.is_empty());
        }
    }

    #[test]
    fn test_quality_predictions_generation() {
        let kg = create_test_knowledge_graph();
        let analyzer = PredictiveAnalyzer::new(&kg);

        let predictions = analyzer.predict_quality_issues().unwrap();

        // Verify predictions were returned (may be empty, which is valid)
        let _ = predictions.len(); // Acknowledge we received a result

        // If predictions exist, they should have valid structure
        for pred in &predictions {
            assert!(!pred.entity_id.is_empty());
            assert!(!pred.entity_type.is_empty());
            assert!(pred.probability >= 0.0);
            assert!(pred.probability <= 1.0);
            assert!(pred.confidence_score >= 0.0);
            assert!(pred.confidence_score <= 1.0);
        }
    }

    #[test]
    fn test_risk_predictions_generation() {
        let kg = create_test_knowledge_graph();
        let analyzer = PredictiveAnalyzer::new(&kg);

        let predictions = analyzer.predict_supply_chain_risks().unwrap();

        // Should generate risk predictions
        assert!(!predictions.is_empty());

        for pred in &predictions {
            assert!(!pred.risk_type.is_empty());
            assert!(pred.probability >= 0.0);
            assert!(pred.probability <= 1.0);
            assert!(!pred.predicted_timeframe.is_empty());
            assert!(!pred.affected_entities.is_empty());
            assert!(!pred.mitigation_strategies.is_empty());
            assert!(pred.confidence_score >= 0.0);
            assert!(pred.confidence_score <= 1.0);
        }
    }

    #[test]
    fn test_optimization_recommendations_generation() {
        let kg = create_test_knowledge_graph();
        let analyzer = PredictiveAnalyzer::new(&kg);

        let recommendations = analyzer.generate_optimization_recommendations().unwrap();

        // Should generate recommendations
        assert!(!recommendations.is_empty());

        for rec in &recommendations {
            assert!(!rec.category.is_empty());
            assert!(!rec.recommendation.is_empty());
            assert!(!rec.expected_benefit.is_empty());
            assert!(!rec.implementation_timeline.is_empty());
            assert!(rec.estimated_savings >= 0.0);
        }
    }

    #[test]
    fn test_comprehensive_predictive_insights() {
        let kg = create_test_knowledge_graph();
        let analyzer = PredictiveAnalyzer::new(&kg);

        let insights = analyzer.generate_insights().unwrap();

        // Should generate all components
        assert_eq!(insights.demand_forecast.forecast_period_days, 30);
        assert!(!insights.risk_predictions.is_empty());
        assert!(!insights.optimization_recommendations.is_empty());
        assert!(!insights.market_trends.is_empty());
        assert!(!insights.seasonal_patterns.is_empty());

        // Anomaly detection should have threshold set
        assert!(insights.anomaly_detection.threshold > 0.0);
    }

    #[test]
    fn test_seasonal_factor_calculation() {
        let kg = create_test_knowledge_graph();
        let analyzer = PredictiveAnalyzer::new(&kg);

        // Should successfully generate forecasts for any date
        let forecast = analyzer.forecast_demand(1).unwrap();
        assert!(!forecast.forecast_points.is_empty());
    }
}

mod supply_chain_analytics_tests {
    use super::*;

    #[test]
    fn test_supply_chain_analyzer_creation() {
        let kg = create_test_knowledge_graph();
        let analyzer = SupplyChainAnalyzer::new(&kg);

        // Should successfully create analyzer
        let _ = analyzer;
    }

    #[test]
    fn test_comprehensive_supply_chain_metrics() {
        let kg = create_test_knowledge_graph();
        let analyzer = SupplyChainAnalyzer::new(&kg);

        let metrics = analyzer.calculate_metrics().unwrap();

        // Should calculate all metric types (usize values are always >= 0)
        let _ = metrics.quality_metrics.total_quality_checks;
        let _ = metrics.compliance_status.total_checks;
        assert!(metrics.traceability_coverage.overall_coverage_percentage >= 0.0);
        assert!(metrics.efficiency_metrics.efficiency_score >= 0.0);
        assert!(metrics.visibility_score >= 0.0);
        assert!(metrics.visibility_score <= 1.0);
    }

    #[test]
    fn test_batch_risk_assessment() {
        let kg = create_test_knowledge_graph();
        let analyzer = SupplyChainAnalyzer::new(&kg);

        let assessment = analyzer.assess_batch_risk("BATCH-001").unwrap();

        assert_eq!(assessment.entity_id, "urn:provchain:entity:batch:1");
        assert!(assessment.overall_risk_score >= 0.0);
        assert!(assessment.overall_risk_score <= 1.0);
        assert!(!assessment.risk_factors.is_empty());
        assert!(!assessment.recommendations.is_empty());

        // Risk factors should have valid structure
        for factor in &assessment.risk_factors {
            assert!(!factor.category.is_empty());
            assert!(!factor.description.is_empty());
            assert!(factor.score >= 0.0);
            assert!(factor.score <= 1.0);
        }
    }

    #[test]
    fn test_risk_level_classification() {
        // Test RiskLevel::from_score
        assert!(matches!(RiskLevel::from_score(0.1), RiskLevel::Low));
        assert!(matches!(RiskLevel::from_score(0.3), RiskLevel::Medium));
        assert!(matches!(RiskLevel::from_score(0.6), RiskLevel::High));
        assert!(matches!(RiskLevel::from_score(0.8), RiskLevel::Critical));
    }

    #[test]
    fn test_quality_metrics_calculation() {
        let kg = create_test_knowledge_graph();
        let analyzer = SupplyChainAnalyzer::new(&kg);

        let quality = analyzer.calculate_quality_metrics().unwrap();

        assert!(quality.quality_pass_rate >= 0.0);
        assert!(quality.quality_pass_rate <= 1.0);
        assert!(quality.defect_rate >= 0.0);
        assert!(quality.defect_rate <= 1.0);
    }

    #[test]
    fn test_compliance_status_check() {
        let kg = create_test_knowledge_graph();
        let analyzer = SupplyChainAnalyzer::new(&kg);

        let compliance = analyzer.check_compliance_status().unwrap();

        assert!(compliance.overall_compliance_rate >= 0.0);
        assert!(compliance.overall_compliance_rate <= 1.0);

        // Each compliance check should have valid structure
        for check in &compliance.compliance_checks {
            assert!(!check.check_type.is_empty());
            assert!(!check.entity_id.is_empty());
            assert!(matches!(
                check.status,
                ComplianceStatus::Compliant
                    | ComplianceStatus::NonCompliant
                    | ComplianceStatus::Pending
                    | ComplianceStatus::Unknown
            ));
        }
    }

    #[test]
    fn test_traceability_coverage_calculation() {
        let kg = create_test_knowledge_graph();
        let analyzer = SupplyChainAnalyzer::new(&kg);

        let coverage = analyzer.calculate_traceability_coverage().unwrap();

        assert!(coverage.overall_coverage_percentage >= 0.0);
        assert!(coverage.overall_coverage_percentage <= 1.0);

        // Coverage details should have valid structure
        for detail in &coverage.coverage_details {
            assert!(!detail.batch_id.is_empty());
            assert!(detail.coverage_percentage >= 0.0);
            assert!(detail.coverage_percentage <= 1.0);
        }
    }

    #[test]
    fn test_efficiency_metrics_calculation() {
        let kg = create_test_knowledge_graph();
        let analyzer = SupplyChainAnalyzer::new(&kg);

        let metrics = analyzer.calculate_metrics().unwrap();
        let efficiency = metrics.efficiency_metrics;

        assert!(efficiency.average_processing_time_hours > 0.0);
        assert!(efficiency.average_transport_time_hours > 0.0);
        assert!(efficiency.total_cycle_time_hours > 0.0);
        assert!(efficiency.efficiency_score >= 0.0);
        assert!(efficiency.efficiency_score <= 1.0);
    }

    #[test]
    fn test_supplier_performance_analysis() {
        let kg = create_test_knowledge_graph();
        let analyzer = SupplyChainAnalyzer::new(&kg);

        let suppliers = analyzer.analyze_supplier_performance().unwrap();

        // Should analyze all supplier-type entities
        for supplier in &suppliers {
            assert!(!supplier.supplier_id.is_empty());
            assert!(!supplier.supplier_type.is_empty());
            assert!(supplier.overall_score >= 0.0);
            assert!(supplier.overall_score <= 1.0);
            assert!(supplier.quality_score >= 0.0);
            assert!(supplier.quality_score <= 1.0);
            assert!(supplier.delivery_performance >= 0.0);
            assert!(supplier.delivery_performance <= 1.0);
            assert!(supplier.compliance_score >= 0.0);
            assert!(supplier.compliance_score <= 1.0);
        }
    }

    #[test]
    fn test_visibility_score_calculation() {
        let kg = create_test_knowledge_graph();
        let analyzer = SupplyChainAnalyzer::new(&kg);

        let metrics = analyzer.calculate_metrics().unwrap();

        // Visibility score should be between 0 and 1
        assert!(metrics.visibility_score >= 0.0);
        assert!(metrics.visibility_score <= 1.0);
    }
}

mod sustainability_analytics_tests {
    use super::*;

    #[test]
    fn test_sustainability_tracker_creation() {
        let kg = create_test_knowledge_graph();
        let tracker = SustainabilityTracker::new(&kg);

        // Should successfully create tracker
        let _ = tracker;
    }

    #[test]
    fn test_comprehensive_sustainability_metrics() {
        let kg = create_test_knowledge_graph();
        let tracker = SustainabilityTracker::new(&kg);

        let metrics = tracker.calculate_metrics().unwrap();

        // Should calculate all sustainability components
        assert!(metrics.carbon_footprint.total_co2_equivalent_kg >= 0.0);
        assert!(!metrics.carbon_footprint.emissions_by_source.is_empty());

        assert!(metrics.esg_score.overall_score >= 0.0);
        assert!(metrics.esg_score.overall_score <= 1.0);

        assert!(!metrics.sustainability_certifications.is_empty());
        assert!(metrics.renewable_energy_usage.renewable_percentage >= 0.0);
        assert!(metrics.waste_reduction_metrics.waste_diversion_rate >= 0.0);
    }

    #[test]
    fn test_batch_carbon_footprint_calculation() {
        let kg = create_test_knowledge_graph();
        let tracker = SustainabilityTracker::new(&kg);

        let footprint = tracker
            .calculate_batch_carbon_footprint("BATCH-001")
            .unwrap();

        assert_eq!(footprint.entity_id, "urn:provchain:entity:batch:1");
        assert!(footprint.total_co2_equivalent_kg >= 0.0);
        assert!(!footprint.emissions_by_source.is_empty());

        // Verify emissions add up to total
        let calculated_total: f64 = footprint.emissions_by_source.iter().map(|e| e.co2_kg).sum();

        assert!((calculated_total - footprint.total_co2_equivalent_kg).abs() < 0.01);

        // Each emission source should have valid data
        for source in &footprint.emissions_by_source {
            assert!(!source.source.is_empty());
            assert!(source.co2_kg >= 0.0);
            assert!(source.percentage >= 0.0);
            assert!(source.percentage <= 100.0);
        }
    }

    #[test]
    fn test_carbon_intensity_calculation() {
        let kg = create_test_knowledge_graph();
        let tracker = SustainabilityTracker::new(&kg);

        let footprint = tracker
            .calculate_batch_carbon_footprint("BATCH-001")
            .unwrap();

        // Carbon intensity should be positive
        assert!(footprint.carbon_intensity > 0.0);

        // Net emissions should account for offsets
        assert!(footprint.net_emissions <= footprint.total_co2_equivalent_kg);
    }

    #[test]
    fn test_esg_score_calculation() {
        let kg = create_test_knowledge_graph();
        let tracker = SustainabilityTracker::new(&kg);

        let metrics = tracker.calculate_metrics().unwrap();
        let esg = metrics.esg_score;

        // All scores should be valid
        assert!(esg.environmental_score >= 0.0);
        assert!(esg.environmental_score <= 1.0);
        assert!(esg.social_score >= 0.0);
        assert!(esg.social_score <= 1.0);
        assert!(esg.governance_score >= 0.0);
        assert!(esg.governance_score <= 1.0);
    }

    #[test]
    fn test_environmental_impact_assessment() {
        let kg = create_test_knowledge_graph();
        let tracker = SustainabilityTracker::new(&kg);

        let metrics = tracker.calculate_metrics().unwrap();
        let impact = metrics.environmental_impact;

        // Biodiversity impact
        assert!(impact.biodiversity_impact.impact_score >= 0.0);
        assert!(impact.biodiversity_impact.impact_score <= 1.0);

        // Water impact
        assert!(impact.water_impact.water_usage_liters >= 0.0);
        assert!(impact.water_impact.water_efficiency_score >= 0.0);
        assert!(impact.water_impact.water_efficiency_score <= 1.0);
        assert!(impact.water_impact.water_recycling_rate >= 0.0);
        assert!(impact.water_impact.water_recycling_rate <= 1.0);

        // Air quality impact
        assert!(impact.air_quality_impact.air_quality_score >= 0.0);
        assert!(impact.air_quality_impact.air_quality_score <= 1.0);
        assert!(impact.air_quality_impact.particulate_emissions_kg >= 0.0);
    }

    #[test]
    fn test_renewable_energy_metrics() {
        let kg = create_test_knowledge_graph();
        let tracker = SustainabilityTracker::new(&kg);

        let metrics = tracker.calculate_metrics().unwrap();
        let energy = metrics.renewable_energy_usage;

        assert!(energy.total_energy_consumption_kwh > 0.0);
        assert!(energy.renewable_energy_kwh >= 0.0);
        assert!(energy.renewable_energy_kwh <= energy.total_energy_consumption_kwh);
        assert!(energy.renewable_percentage >= 0.0);
        assert!(energy.renewable_percentage <= 100.0);

        // Energy sources should add up to total
        let sources_total: f64 = energy.energy_sources.iter().map(|s| s.kwh).sum();
        assert!((sources_total - energy.total_energy_consumption_kwh).abs() < 0.1);
    }

    #[test]
    fn test_waste_reduction_metrics() {
        let kg = create_test_knowledge_graph();
        let tracker = SustainabilityTracker::new(&kg);

        let metrics = tracker.calculate_metrics().unwrap();
        let waste = metrics.waste_reduction_metrics;

        assert!(waste.total_waste_kg >= 0.0);
        assert!(waste.recycled_waste_kg >= 0.0);
        assert!(waste.composted_waste_kg >= 0.0);
        assert!(waste.landfill_waste_kg >= 0.0);

        // Verify waste accounting
        let accounted_for =
            waste.recycled_waste_kg + waste.composted_waste_kg + waste.landfill_waste_kg;
        assert!((accounted_for - waste.total_waste_kg).abs() < 0.01);

        // Verify diversion rate calculation
        let expected_diversion =
            ((waste.recycled_waste_kg + waste.composted_waste_kg) / waste.total_waste_kg) * 100.0;
        assert!((waste.waste_diversion_rate - expected_diversion).abs() < 0.1);
    }

    #[test]
    fn test_water_efficiency_metrics() {
        let kg = create_test_knowledge_graph();
        let tracker = SustainabilityTracker::new(&kg);

        let metrics = tracker.calculate_metrics().unwrap();
        let water = metrics.water_usage_efficiency;

        assert!(water.total_water_usage_liters > 0.0);
        assert!(water.recycled_water_liters >= 0.0);
        assert!(water.recycled_water_liters <= water.total_water_usage_liters);
        assert!(water.water_recycling_rate >= 0.0);
        assert!(water.water_recycling_rate <= 100.0);

        // Verify recycling rate calculation
        let expected_rate = (water.recycled_water_liters / water.total_water_usage_liters) * 100.0;
        assert!((water.water_recycling_rate - expected_rate).abs() < 0.1);

        assert!(!water.water_conservation_measures.is_empty());
    }
}

mod analytics_engine_tests {
    use super::*;

    #[test]
    fn test_analytics_engine_creation() {
        let kg = create_test_knowledge_graph();
        let rdf_store = RDFStore::new();

        let engine = AnalyticsEngine::new(kg, rdf_store);

        // Should create all analyzers
        let _ = engine;
    }

    #[test]
    fn test_comprehensive_analytics_report() {
        let kg = create_test_knowledge_graph();
        let rdf_store = RDFStore::new();

        let engine = AnalyticsEngine::new(kg, rdf_store);
        let report = engine.generate_comprehensive_report().unwrap();

        // Should generate all report sections
        assert!(
            report
                .sustainability_metrics
                .carbon_footprint
                .total_co2_equivalent_kg
                >= 0.0
        );
        assert_eq!(
            report
                .predictive_insights
                .demand_forecast
                .forecast_period_days,
            30
        );
        assert!(report.summary.total_entities > 0);
        assert!(!report.summary.key_insights.is_empty());
    }

    #[test]
    fn test_analytics_engine_updates() {
        let mut kg1 = create_test_knowledge_graph();
        let rdf_store = RDFStore::new();

        let mut engine = AnalyticsEngine::new(kg1.clone(), rdf_store);

        // Add new entity to update
        let new_entity = KnowledgeEntity {
            uri: "urn:provchain:entity:test:1".to_string(),
            entity_type: "TestEntity".to_string(),
            label: Some("Test".to_string()),
            properties: vec![("test".to_string(), "value".to_string())]
                .into_iter()
                .collect(),
            confidence_score: 1.0,
        };
        let _ = kg1.add_entity(new_entity);

        engine.update_knowledge_graph(kg1);

        // Should have updated entity count
        assert_eq!(engine.get_knowledge_graph().entities.len(), 6);
    }

    #[test]
    fn test_executive_summary_generation() {
        let kg = create_test_knowledge_graph();
        let rdf_store = RDFStore::new();

        let engine = AnalyticsEngine::new(kg, rdf_store);
        let report = engine.generate_comprehensive_report().unwrap();
        let summary = report.summary;

        assert!(summary.total_entities > 0);
        assert!(summary.total_relationships > 0);
        // product_batches and total_activities are usize, so always >= 0
        let _ = summary.product_batches;
        let _ = summary.total_activities;
        assert!(!summary.key_insights.is_empty());
    }
}
