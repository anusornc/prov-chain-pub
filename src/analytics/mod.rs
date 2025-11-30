//! Analytics module for Phase 3
//!
//! This module provides advanced analytics and intelligence capabilities
//! including supply chain analytics, sustainability tracking, and predictive analytics.

pub mod predictive;
pub mod supply_chain;
pub mod sustainability;

use crate::knowledge_graph::KnowledgeGraph;
use crate::storage::rdf_store::RDFStore;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Main analytics engine
pub struct AnalyticsEngine {
    knowledge_graph: KnowledgeGraph,
    _rdf_store: RDFStore,
    supply_chain_analyzer: supply_chain::SupplyChainAnalyzer,
    sustainability_tracker: sustainability::SustainabilityTracker,
    predictive_analyzer: predictive::PredictiveAnalyzer,
}

impl AnalyticsEngine {
    /// Create a new analytics engine
    pub fn new(knowledge_graph: KnowledgeGraph, rdf_store: RDFStore) -> Self {
        Self {
            supply_chain_analyzer: supply_chain::SupplyChainAnalyzer::new(&knowledge_graph),
            sustainability_tracker: sustainability::SustainabilityTracker::new(&knowledge_graph),
            predictive_analyzer: predictive::PredictiveAnalyzer::new(&knowledge_graph),
            knowledge_graph,
            _rdf_store: rdf_store,
        }
    }

    /// Update the analytics engine with new knowledge graph data
    pub fn update_knowledge_graph(&mut self, knowledge_graph: KnowledgeGraph) {
        self.knowledge_graph = knowledge_graph;
        self.supply_chain_analyzer = supply_chain::SupplyChainAnalyzer::new(&self.knowledge_graph);
        self.sustainability_tracker =
            sustainability::SustainabilityTracker::new(&self.knowledge_graph);
        self.predictive_analyzer = predictive::PredictiveAnalyzer::new(&self.knowledge_graph);
    }

    /// Generate comprehensive analytics report
    pub fn generate_comprehensive_report(&self) -> Result<AnalyticsReport> {
        let supply_chain_metrics = self.supply_chain_analyzer.calculate_metrics()?;
        let sustainability_metrics = self.sustainability_tracker.calculate_metrics()?;
        let predictive_insights = self.predictive_analyzer.generate_insights()?;

        Ok(AnalyticsReport {
            timestamp: Utc::now(),
            supply_chain_metrics,
            sustainability_metrics,
            predictive_insights,
            summary: self.generate_executive_summary()?,
        })
    }

    /// Generate executive summary
    fn generate_executive_summary(&self) -> Result<ExecutiveSummary> {
        let total_entities = self.knowledge_graph.entities.len();
        let total_relationships = self.knowledge_graph.relationships.len();

        // Count entities by type
        let mut entity_type_counts = HashMap::new();
        for entity in self.knowledge_graph.entities.values() {
            *entity_type_counts
                .entry(entity.entity_type.clone())
                .or_insert(0) += 1;
        }

        let product_batches = entity_type_counts.get("ProductBatch").unwrap_or(&0);
        let activities = entity_type_counts.get("ProcessingActivity").unwrap_or(&0)
            + entity_type_counts.get("TransportActivity").unwrap_or(&0)
            + entity_type_counts.get("QualityCheck").unwrap_or(&0);

        Ok(ExecutiveSummary {
            total_entities,
            total_relationships,
            product_batches: *product_batches,
            total_activities: activities,
            key_insights: vec![
                format!(
                    "Knowledge graph contains {} entities across {} types",
                    total_entities,
                    entity_type_counts.len()
                ),
                format!(
                    "Tracking {} product batches through {} activities",
                    product_batches, activities
                ),
                "Supply chain visibility and traceability fully operational".to_string(),
            ],
        })
    }

    /// Get supply chain analyzer
    pub fn supply_chain_analyzer(&self) -> &supply_chain::SupplyChainAnalyzer {
        &self.supply_chain_analyzer
    }

    /// Get sustainability tracker
    pub fn sustainability_tracker(&self) -> &sustainability::SustainabilityTracker {
        &self.sustainability_tracker
    }

    /// Get predictive analyzer
    pub fn predictive_analyzer(&self) -> &predictive::PredictiveAnalyzer {
        &self.predictive_analyzer
    }

    /// Get the knowledge graph
    pub fn get_knowledge_graph(&self) -> &KnowledgeGraph {
        &self.knowledge_graph
    }
}

/// Comprehensive analytics report
#[derive(Debug, serde::Serialize)]
pub struct AnalyticsReport {
    pub timestamp: DateTime<Utc>,
    pub supply_chain_metrics: supply_chain::SupplyChainMetrics,
    pub sustainability_metrics: sustainability::SustainabilityMetrics,
    pub predictive_insights: predictive::PredictiveInsights,
    pub summary: ExecutiveSummary,
}

/// Executive summary of analytics
#[derive(Debug, serde::Serialize)]
pub struct ExecutiveSummary {
    pub total_entities: usize,
    pub total_relationships: usize,
    pub product_batches: usize,
    pub total_activities: usize,
    pub key_insights: Vec<String>,
}

/// Risk level enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    /// Convert risk score to risk level
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s < 0.25 => RiskLevel::Low,
            s if s < 0.5 => RiskLevel::Medium,
            s if s < 0.75 => RiskLevel::High,
            _ => RiskLevel::Critical,
        }
    }

    /// Get color code for visualization
    pub fn color_code(&self) -> &'static str {
        match self {
            RiskLevel::Low => "#28a745",      // Green
            RiskLevel::Medium => "#ffc107",   // Yellow
            RiskLevel::High => "#fd7e14",     // Orange
            RiskLevel::Critical => "#dc3545", // Red
        }
    }
}

/// Quality score enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum QualityScore {
    Excellent,
    Good,
    Fair,
    Poor,
}

impl QualityScore {
    /// Convert numeric score to quality score
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 0.9 => QualityScore::Excellent,
            s if s >= 0.7 => QualityScore::Good,
            s if s >= 0.5 => QualityScore::Fair,
            _ => QualityScore::Poor,
        }
    }
}

/// Compliance status enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    Pending,
    Unknown,
}

/// Time series data point
#[derive(Debug, Clone, serde::Serialize)]
pub struct TimeSeriesPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub metadata: HashMap<String, String>,
}

/// Trend analysis result
#[derive(Debug, serde::Serialize)]
pub struct TrendAnalysis {
    pub direction: TrendDirection,
    pub strength: f64,
    pub confidence: f64,
    pub forecast: Vec<TimeSeriesPoint>,
}

/// Trend direction enumeration
#[derive(Debug, serde::Serialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

/// Anomaly detection result
#[derive(Debug, serde::Serialize)]
pub struct AnomalyDetection {
    pub anomalies: Vec<AnomalyPoint>,
    pub threshold: f64,
    pub confidence_interval: (f64, f64),
}

/// Anomaly point
#[derive(Debug, serde::Serialize)]
pub struct AnomalyPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub severity: f64,
    pub description: String,
}
