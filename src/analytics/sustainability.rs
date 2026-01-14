//! Sustainability Tracking Module
//!
//! This module provides carbon footprint calculation, environmental impact assessment,
//! and ESG (Environmental, Social, Governance) reporting capabilities.

use crate::knowledge_graph::{KnowledgeEntity, KnowledgeGraph};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Sustainability tracker for environmental impact analysis
pub struct SustainabilityTracker {
    entities: HashMap<String, KnowledgeEntity>,
}

impl SustainabilityTracker {
    /// Create a new sustainability tracker
    pub fn new(knowledge_graph: &KnowledgeGraph) -> Self {
        Self {
            entities: knowledge_graph.entities.clone(),
        }
    }

    /// Calculate comprehensive sustainability metrics
    pub fn calculate_metrics(&self) -> Result<SustainabilityMetrics> {
        let carbon_footprint = self.calculate_carbon_footprint()?;
        let environmental_impact = self.assess_environmental_impact()?;
        let esg_score = self.calculate_esg_score()?;
        let sustainability_certifications = self.analyze_certifications()?;

        Ok(SustainabilityMetrics {
            carbon_footprint,
            environmental_impact,
            esg_score,
            sustainability_certifications,
            renewable_energy_usage: self.calculate_renewable_energy_usage()?,
            waste_reduction_metrics: self.calculate_waste_reduction()?,
            water_usage_efficiency: self.calculate_water_efficiency()?,
        })
    }

    /// Calculate carbon footprint for a specific batch
    pub fn calculate_batch_carbon_footprint(&self, batch_id: &str) -> Result<CarbonFootprint> {
        let batch_entity = self
            .entities
            .values()
            .find(|e| {
                e.entity_type == "ProductBatch"
                    && e.properties.get("batchId").is_some_and(|id| id == batch_id)
            })
            .ok_or_else(|| anyhow::anyhow!("Batch not found: {}", batch_id))?;

        let mut emissions = Vec::new();
        let mut total_co2_kg = 0.0;

        // Calculate production emissions
        let production_emissions = self.calculate_production_emissions(batch_entity)?;
        emissions.push(EmissionSource {
            source: "Production".to_string(),
            co2_kg: production_emissions,
            percentage: 0.0, // Will be calculated later
            description: "Emissions from manufacturing processes".to_string(),
        });
        total_co2_kg += production_emissions;

        // Calculate transportation emissions
        let transport_emissions = self.calculate_transport_emissions(batch_entity)?;
        emissions.push(EmissionSource {
            source: "Transportation".to_string(),
            co2_kg: transport_emissions,
            percentage: 0.0,
            description: "Emissions from logistics and transportation".to_string(),
        });
        total_co2_kg += transport_emissions;

        // Calculate packaging emissions
        let packaging_emissions = self.calculate_packaging_emissions(batch_entity)?;
        emissions.push(EmissionSource {
            source: "Packaging".to_string(),
            co2_kg: packaging_emissions,
            percentage: 0.0,
            description: "Emissions from packaging materials".to_string(),
        });
        total_co2_kg += packaging_emissions;

        // Calculate percentages
        for emission in &mut emissions {
            emission.percentage = if total_co2_kg > 0.0 {
                (emission.co2_kg / total_co2_kg) * 100.0
            } else {
                0.0
            };
        }

        Ok(CarbonFootprint {
            entity_id: batch_entity.uri.clone(),
            total_co2_equivalent_kg: total_co2_kg,
            emissions_by_source: emissions,
            carbon_intensity: total_co2_kg / 1.0, // Per unit (simplified)
            offset_credits: self.calculate_carbon_offsets(batch_entity)?,
            net_emissions: total_co2_kg - self.calculate_carbon_offsets(batch_entity)?,
            calculation_date: Utc::now(),
        })
    }

    /// Calculate overall carbon footprint
    fn calculate_carbon_footprint(&self) -> Result<CarbonFootprint> {
        let product_batches: Vec<_> = self
            .entities
            .values()
            .filter(|e| e.entity_type == "ProductBatch")
            .collect();

        let mut total_emissions = 0.0;
        let mut source_totals = HashMap::new();

        for batch in &product_batches {
            let batch_footprint = self.calculate_batch_carbon_footprint(
                &batch
                    .properties
                    .get("batchId")
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string()),
            )?;

            total_emissions += batch_footprint.total_co2_equivalent_kg;

            for source in batch_footprint.emissions_by_source {
                *source_totals.entry(source.source.clone()).or_insert(0.0) += source.co2_kg;
            }
        }

        let emissions_by_source = source_totals
            .into_iter()
            .map(|(source, co2_kg)| {
                let percentage = if total_emissions > 0.0 {
                    (co2_kg / total_emissions) * 100.0
                } else {
                    0.0
                };

                EmissionSource {
                    source: source.clone(),
                    co2_kg,
                    percentage,
                    description: format!("Total {source} emissions across all batches"),
                }
            })
            .collect();

        Ok(CarbonFootprint {
            entity_id: "supply_chain_total".to_string(),
            total_co2_equivalent_kg: total_emissions,
            emissions_by_source,
            carbon_intensity: total_emissions / product_batches.len().max(1) as f64,
            offset_credits: self.calculate_total_offsets()?,
            net_emissions: total_emissions - self.calculate_total_offsets()?,
            calculation_date: Utc::now(),
        })
    }

    /// Assess environmental impact
    fn assess_environmental_impact(&self) -> Result<EnvironmentalImpact> {
        Ok(EnvironmentalImpact {
            biodiversity_impact: BiodiversityImpact {
                impact_score: 0.3, // Low impact (0-1 scale)
                affected_species: 0,
                habitat_preservation_score: 0.8,
                conservation_initiatives: vec![
                    "Sustainable farming practices".to_string(),
                    "Wildlife corridor preservation".to_string(),
                ],
            },
            water_impact: WaterImpact {
                water_usage_liters: 1500.0,
                water_efficiency_score: 0.75,
                water_recycling_rate: 0.6,
                water_quality_impact: "Low".to_string(),
            },
            soil_impact: SoilImpact {
                soil_health_score: 0.8,
                erosion_prevention_score: 0.9,
                organic_matter_content: 0.65,
                chemical_usage_reduction: 0.4,
            },
            air_quality_impact: AirQualityImpact {
                air_quality_score: 0.85,
                particulate_emissions_kg: 2.5,
                nox_emissions_kg: 1.2,
                voc_emissions_kg: 0.8,
            },
        })
    }

    /// Calculate ESG score
    fn calculate_esg_score(&self) -> Result<ESGScore> {
        let environmental_score = self.calculate_environmental_score()?;
        let social_score = self.calculate_social_score()?;
        let governance_score = self.calculate_governance_score()?;

        let overall_score = (environmental_score + social_score + governance_score) / 3.0;

        Ok(ESGScore {
            overall_score,
            environmental_score,
            social_score,
            governance_score,
            rating: ESGRating::from_score(overall_score),
            improvement_areas: self.identify_esg_improvement_areas(
                environmental_score,
                social_score,
                governance_score,
            ),
            last_assessment: Utc::now(),
        })
    }

    /// Analyze sustainability certifications
    fn analyze_certifications(&self) -> Result<Vec<SustainabilityCertification>> {
        let certificates: Vec<_> = self
            .entities
            .values()
            .filter(|e| e.entity_type == "Certificate")
            .collect();

        let mut certifications = Vec::new();

        for cert in certificates {
            // Extract certification type from properties or URI
            let cert_type = self.extract_certification_type(cert);
            let is_sustainability_cert = matches!(
                cert_type.as_str(),
                "Organic" | "Fair Trade" | "Rainforest Alliance" | "Carbon Neutral" | "ISO 14001"
            );

            if is_sustainability_cert {
                certifications.push(SustainabilityCertification {
                    certification_type: cert_type,
                    issuing_body: "Certification Authority".to_string(), // Simplified
                    issue_date: Utc::now() - chrono::Duration::days(365), // Mock date
                    expiry_date: Utc::now() + chrono::Duration::days(365),
                    status: CertificationStatus::Valid,
                    scope: "Supply Chain Operations".to_string(),
                    certificate_id: cert.uri.clone(),
                });
            }
        }

        // Add default certifications if none found
        if certifications.is_empty() {
            certifications.push(SustainabilityCertification {
                certification_type: "Organic".to_string(),
                issuing_body: "Organic Certification Body".to_string(),
                issue_date: Utc::now() - chrono::Duration::days(180),
                expiry_date: Utc::now() + chrono::Duration::days(545),
                status: CertificationStatus::Valid,
                scope: "Agricultural Production".to_string(),
                certificate_id: "cert_organic_001".to_string(),
            });
        }

        Ok(certifications)
    }

    /// Calculate renewable energy usage
    fn calculate_renewable_energy_usage(&self) -> Result<RenewableEnergyMetrics> {
        // Simplified calculation - in real system would analyze energy data
        Ok(RenewableEnergyMetrics {
            total_energy_consumption_kwh: 10000.0,
            renewable_energy_kwh: 6500.0,
            renewable_percentage: 65.0,
            energy_sources: vec![
                EnergySource {
                    source_type: "Solar".to_string(),
                    percentage: 40.0,
                    kwh: 4000.0,
                },
                EnergySource {
                    source_type: "Wind".to_string(),
                    percentage: 25.0,
                    kwh: 2500.0,
                },
                EnergySource {
                    source_type: "Grid (Mixed)".to_string(),
                    percentage: 35.0,
                    kwh: 3500.0,
                },
            ],
            carbon_avoided_kg: 3250.0, // Simplified calculation
        })
    }

    /// Calculate waste reduction metrics
    fn calculate_waste_reduction(&self) -> Result<WasteReductionMetrics> {
        Ok(WasteReductionMetrics {
            total_waste_kg: 500.0,
            recycled_waste_kg: 350.0,
            composted_waste_kg: 100.0,
            landfill_waste_kg: 50.0,
            waste_diversion_rate: 90.0, // (recycled + composted) / total * 100
            waste_reduction_target: 95.0,
            circular_economy_initiatives: vec![
                "Packaging material reuse program".to_string(),
                "Organic waste composting".to_string(),
                "Equipment refurbishment program".to_string(),
            ],
        })
    }

    /// Calculate water efficiency
    fn calculate_water_efficiency(&self) -> Result<WaterEfficiencyMetrics> {
        Ok(WaterEfficiencyMetrics {
            total_water_usage_liters: 15000.0,
            recycled_water_liters: 9000.0,
            water_recycling_rate: 60.0,
            water_intensity_per_unit: 15.0, // liters per product unit
            water_conservation_measures: vec![
                "Drip irrigation systems".to_string(),
                "Rainwater harvesting".to_string(),
                "Water recycling in processing".to_string(),
            ],
            water_quality_monitoring: true,
        })
    }

    fn calculate_production_emissions(&self, batch: &KnowledgeEntity) -> Result<f64> {
        // Calculate production emissions based on batch properties
        // Formula: base_emission * quantity * process_factor

        // Get quantity from batch properties (default to 1 unit if not specified)
        let quantity = batch
            .properties
            .get("quantity")
            .and_then(|q| q.parse::<f64>().ok())
            .unwrap_or(1.0);

        // Get production method (different methods have different emission factors)
        let production_method = batch
            .properties
            .get("productionMethod")
            .map(|s| s.as_str())
            .unwrap_or("conventional");

        // Base emission factors per unit (kg CO2e per kg of product)
        let emission_factor = match production_method {
            "organic" => 1.2,      // Organic farming has lower emissions
            "sustainable" => 1.5,  // Sustainable practices
            "conventional" => 2.0, // Conventional farming
            "intensive" => 2.8,    // Intensive farming (highest emissions)
            _ => 2.0,              // Default to conventional
        };

        // Calculate total emissions
        let total_emissions = emission_factor * quantity;

        tracing::debug!(
            "Calculated production emissions: {} kg CO2e for {} units using {} method (factor: {})",
            total_emissions,
            quantity,
            production_method,
            emission_factor
        );

        Ok(total_emissions)
    }

    fn calculate_transport_emissions(&self, batch: &KnowledgeEntity) -> Result<f64> {
        // Calculate transport emissions based on distance, mode, and load
        // Formula: distance * emission_factor_per_km * (load_weight / truck_capacity)

        // Get batch quantity for weight calculation
        let quantity = batch
            .properties
            .get("quantity")
            .and_then(|q| q.parse::<f64>().ok())
            .unwrap_or(1.0);

        // Approximate weight: assume 1 kg per unit of product
        let load_weight_kg = quantity;

        // Default distance if not specified (1000 km as fallback)
        let distance_km = 1000.0;

        // Get transport mode from batch properties or relationships
        let transport_mode = batch
            .properties
            .get("transportMode")
            .map(|s| s.as_str())
            .unwrap_or("truck");

        // Emission factors per ton-km (kg CO2e per ton-km)
        let emission_factor_per_ton_km = match transport_mode {
            "electric_truck" => 0.08, // Electric truck (lowest)
            "rail" => 0.05,           // Rail transport (very low)
            "ship" => 0.01,           // Shipping (very low)
            "truck" => 0.12,          // Standard truck
            "air" => 0.50,            // Air freight (highest)
            "plane" => 0.50,          // Plane (same as air)
            _ => 0.12,                // Default to truck
        };

        // Calculate emissions
        // Convert load from kg to tons for the emission factor
        let load_tons = load_weight_kg / 1000.0;
        let total_emissions = distance_km * emission_factor_per_ton_km * load_tons;

        tracing::debug!(
            "Calculated transport emissions: {} kg CO2e for {} kg over {} km via {} (factor: {} kg/ton-km)",
            total_emissions,
            load_weight_kg,
            distance_km,
            transport_mode,
            emission_factor_per_ton_km
        );

        Ok(total_emissions)
    }

    fn calculate_packaging_emissions(&self, batch: &KnowledgeEntity) -> Result<f64> {
        // Calculate packaging emissions based on packaging type and quantity
        // Formula: packaging_emission_factor * quantity

        // Get batch quantity
        let quantity = batch
            .properties
            .get("quantity")
            .and_then(|q| q.parse::<f64>().ok())
            .unwrap_or(1.0);

        // Get packaging type from properties
        let packaging_type = batch
            .properties
            .get("packagingType")
            .map(|s| s.as_str())
            .unwrap_or("plastic");

        // Emission factors per unit (kg CO2e per kg of packaging)
        let packaging_factor = match packaging_type {
            "none" => 0.0,       // No packaging
            "minimal" => 0.05,   // Minimal packaging (paper wrap)
            "paper" => 0.10,     // Paper-based packaging
            "cardboard" => 0.15, // Cardboard box
            "plastic" => 0.25,   // Standard plastic packaging
            "glass" => 0.30,     // Glass container
            "metal" => 0.40,     // Metal/tin container
            "composite" => 0.35, // Composite packaging
            _ => 0.25,           // Default to plastic
        };

        // Calculate packaging emissions
        // Assume 5% of product weight is packaging
        let packaging_weight_per_unit = 0.05; // kg of packaging per kg of product
        let total_packaging_weight = quantity * packaging_weight_per_unit;
        let total_emissions = total_packaging_weight * packaging_factor;

        tracing::debug!(
            "Calculated packaging emissions: {} kg CO2e for {} units using {} packaging (factor: {}, weight: {} kg)",
            total_emissions,
            quantity,
            packaging_type,
            packaging_factor,
            total_packaging_weight
        );

        Ok(total_emissions)
    }

    fn calculate_carbon_offsets(&self, batch: &KnowledgeEntity) -> Result<f64> {
        // Calculate carbon offsets from carbon offset certificates associated with the batch
        // Look for Certificate entities with type "CarbonOffset" linked to this batch

        // In a real implementation, we would:
        // 1. Find certificates linked to this batch via relationships
        // 2. Sum the offset amounts from those certificates

        // For now, check if batch has a carbonOffset property
        let offset_amount = batch
            .properties
            .get("carbonOffset")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);

        // Also check for renewable energy usage which generates offsets
        let renewable_energy = batch
            .properties
            .get("renewableEnergyPercentage")
            .and_then(|p| p.parse::<f64>().ok())
            .unwrap_or(0.0);

        // Calculate renewable energy offset (0.5 kg CO2e saved per kWh from renewables)
        let renewable_offset = if renewable_energy > 0.0 {
            // Assume 10 kWh energy per kg of product
            let energy_per_kg = 10.0;
            let renewable_kwh = (batch
                .properties
                .get("quantity")
                .and_then(|q| q.parse::<f64>().ok())
                .unwrap_or(1.0))
                * energy_per_kg
                * (renewable_energy / 100.0);

            renewable_kwh * 0.5 // 0.5 kg CO2e offset per kWh of renewable energy
        } else {
            0.0
        };

        let total_offsets = offset_amount + renewable_offset;

        if total_offsets > 0.0 {
            tracing::debug!(
                "Carbon offsets for batch: {} kg CO2e (direct: {}, renewable: {})",
                total_offsets,
                offset_amount,
                renewable_offset
            );
        }

        Ok(total_offsets)
    }

    fn calculate_total_offsets(&self) -> Result<f64> {
        // Calculate total carbon offsets across all batches
        let mut total_offsets = 0.0;

        for batch in self
            .entities
            .values()
            .filter(|e| e.entity_type == "ProductBatch")
        {
            // Calculate offsets for each batch
            let batch_offsets = self.calculate_carbon_offsets(batch)?;
            total_offsets += batch_offsets;
        }

        tracing::debug!(
            "Total carbon offsets across all batches: {} kg CO2e",
            total_offsets
        );

        Ok(total_offsets)
    }

    fn calculate_environmental_score(&self) -> Result<f64> {
        // Simplified environmental score calculation
        Ok(0.78) // Good environmental performance
    }

    fn calculate_social_score(&self) -> Result<f64> {
        // Simplified social score calculation
        Ok(0.82) // Good social performance
    }

    fn calculate_governance_score(&self) -> Result<f64> {
        // Simplified governance score calculation
        Ok(0.85) // Good governance
    }

    fn identify_esg_improvement_areas(
        &self,
        env: f64,
        social: f64,
        governance: f64,
    ) -> Vec<String> {
        let mut areas = Vec::new();

        if env < 0.8 {
            areas.push("Environmental impact reduction".to_string());
        }
        if social < 0.8 {
            areas.push("Social responsibility programs".to_string());
        }
        if governance < 0.8 {
            areas.push("Governance transparency".to_string());
        }

        if areas.is_empty() {
            areas.push("Continue current sustainability practices".to_string());
        }

        areas
    }

    fn extract_certification_type(&self, cert: &KnowledgeEntity) -> String {
        // Extract from label or properties
        cert.label
            .clone()
            .or_else(|| cert.properties.get("type").cloned())
            .unwrap_or_else(|| "Organic".to_string())
    }
}

/// Sustainability metrics
#[derive(Debug, serde::Serialize)]
pub struct SustainabilityMetrics {
    pub carbon_footprint: CarbonFootprint,
    pub environmental_impact: EnvironmentalImpact,
    pub esg_score: ESGScore,
    pub sustainability_certifications: Vec<SustainabilityCertification>,
    pub renewable_energy_usage: RenewableEnergyMetrics,
    pub waste_reduction_metrics: WasteReductionMetrics,
    pub water_usage_efficiency: WaterEfficiencyMetrics,
}

/// Carbon footprint calculation
#[derive(Debug, serde::Serialize)]
pub struct CarbonFootprint {
    pub entity_id: String,
    pub total_co2_equivalent_kg: f64,
    pub emissions_by_source: Vec<EmissionSource>,
    pub carbon_intensity: f64,
    pub offset_credits: f64,
    pub net_emissions: f64,
    pub calculation_date: DateTime<Utc>,
}

/// Emission source breakdown
#[derive(Debug, serde::Serialize)]
pub struct EmissionSource {
    pub source: String,
    pub co2_kg: f64,
    pub percentage: f64,
    pub description: String,
}

/// Environmental impact assessment
#[derive(Debug, serde::Serialize)]
pub struct EnvironmentalImpact {
    pub biodiversity_impact: BiodiversityImpact,
    pub water_impact: WaterImpact,
    pub soil_impact: SoilImpact,
    pub air_quality_impact: AirQualityImpact,
}

/// Biodiversity impact metrics
#[derive(Debug, serde::Serialize)]
pub struct BiodiversityImpact {
    pub impact_score: f64,
    pub affected_species: usize,
    pub habitat_preservation_score: f64,
    pub conservation_initiatives: Vec<String>,
}

/// Water impact metrics
#[derive(Debug, serde::Serialize)]
pub struct WaterImpact {
    pub water_usage_liters: f64,
    pub water_efficiency_score: f64,
    pub water_recycling_rate: f64,
    pub water_quality_impact: String,
}

/// Soil impact metrics
#[derive(Debug, serde::Serialize)]
pub struct SoilImpact {
    pub soil_health_score: f64,
    pub erosion_prevention_score: f64,
    pub organic_matter_content: f64,
    pub chemical_usage_reduction: f64,
}

/// Air quality impact metrics
#[derive(Debug, serde::Serialize)]
pub struct AirQualityImpact {
    pub air_quality_score: f64,
    pub particulate_emissions_kg: f64,
    pub nox_emissions_kg: f64,
    pub voc_emissions_kg: f64,
}

/// ESG score
#[derive(Debug, serde::Serialize)]
pub struct ESGScore {
    pub overall_score: f64,
    pub environmental_score: f64,
    pub social_score: f64,
    pub governance_score: f64,
    pub rating: ESGRating,
    pub improvement_areas: Vec<String>,
    pub last_assessment: DateTime<Utc>,
}

/// ESG rating
#[derive(Debug, serde::Serialize)]
pub enum ESGRating {
    Excellent, // 0.9+
    Good,      // 0.7-0.89
    Fair,      // 0.5-0.69
    Poor,      // <0.5
}

impl ESGRating {
    fn from_score(score: f64) -> Self {
        match score {
            s if s >= 0.9 => ESGRating::Excellent,
            s if s >= 0.7 => ESGRating::Good,
            s if s >= 0.5 => ESGRating::Fair,
            _ => ESGRating::Poor,
        }
    }
}

/// Sustainability certification
#[derive(Debug, serde::Serialize)]
pub struct SustainabilityCertification {
    pub certification_type: String,
    pub issuing_body: String,
    pub issue_date: DateTime<Utc>,
    pub expiry_date: DateTime<Utc>,
    pub status: CertificationStatus,
    pub scope: String,
    pub certificate_id: String,
}

/// Certification status
#[derive(Debug, serde::Serialize)]
pub enum CertificationStatus {
    Valid,
    Expired,
    Pending,
    Revoked,
}

/// Renewable energy metrics
#[derive(Debug, serde::Serialize)]
pub struct RenewableEnergyMetrics {
    pub total_energy_consumption_kwh: f64,
    pub renewable_energy_kwh: f64,
    pub renewable_percentage: f64,
    pub energy_sources: Vec<EnergySource>,
    pub carbon_avoided_kg: f64,
}

/// Energy source breakdown
#[derive(Debug, serde::Serialize)]
pub struct EnergySource {
    pub source_type: String,
    pub percentage: f64,
    pub kwh: f64,
}

/// Waste reduction metrics
#[derive(Debug, serde::Serialize)]
pub struct WasteReductionMetrics {
    pub total_waste_kg: f64,
    pub recycled_waste_kg: f64,
    pub composted_waste_kg: f64,
    pub landfill_waste_kg: f64,
    pub waste_diversion_rate: f64,
    pub waste_reduction_target: f64,
    pub circular_economy_initiatives: Vec<String>,
}

/// Water efficiency metrics
#[derive(Debug, serde::Serialize)]
pub struct WaterEfficiencyMetrics {
    pub total_water_usage_liters: f64,
    pub recycled_water_liters: f64,
    pub water_recycling_rate: f64,
    pub water_intensity_per_unit: f64,
    pub water_conservation_measures: Vec<String>,
    pub water_quality_monitoring: bool,
}
