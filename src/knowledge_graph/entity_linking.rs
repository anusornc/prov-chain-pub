//! Entity Linking and Resolution System
//!
//! This module provides automatic entity resolution, deduplication,
//! and confidence scoring for entity relationships.

use super::{KnowledgeEntity, KnowledgeGraph};
use anyhow::Result;
use lazy_static::lazy_static;
use std::collections::HashMap;

/// Entity linking system for resolving and deduplicating entities
pub struct EntityLinker {
    similarity_threshold: f64,
    string_matchers: Vec<Box<dyn StringMatcher>>,
    external_resolvers: Vec<Box<dyn ExternalResolver>>,
}

impl EntityLinker {
    /// Create a new entity linker with default configuration
    pub fn new() -> Self {
        let mut linker = Self {
            similarity_threshold: 0.8,
            string_matchers: Vec::new(),
            external_resolvers: Vec::new(),
        };

        // Register default matchers and resolvers
        linker.register_default_components();
        linker
    }

    /// Register default string matchers and external resolvers
    fn register_default_components(&mut self) {
        self.string_matchers.push(Box::new(ExactMatcher));
        self.string_matchers.push(Box::new(LevenshteinMatcher));
        self.string_matchers.push(Box::new(TokenMatcher));
        self.string_matchers.push(Box::new(PhoneticMatcher));

        self.external_resolvers.push(Box::new(GeoNamesResolver));
        self.external_resolvers.push(Box::new(CompanyResolver));
    }

    /// Resolve and deduplicate entities in a knowledge graph
    pub fn resolve_entities(&self, kg: &mut KnowledgeGraph) -> Result<EntityResolutionReport> {
        let mut report = EntityResolutionReport::new();
        let mut entity_clusters: Vec<std::collections::HashSet<String>> = Vec::new();
        let entities: Vec<KnowledgeEntity> = kg.entities.values().cloned().collect();

        // Find similar entities and group them into clusters
        for (i, entity1) in entities.iter().enumerate() {
            for entity2 in entities.iter().skip(i + 1) {
                if self.are_entities_similar(entity1, entity2)? {
                    // Find or create cluster
                    let mut found_cluster = false;
                    for cluster in &mut entity_clusters {
                        if cluster.contains(&entity1.uri) || cluster.contains(&entity2.uri) {
                            cluster.insert(entity1.uri.clone());
                            cluster.insert(entity2.uri.clone());
                            found_cluster = true;
                            break;
                        }
                    }

                    if !found_cluster {
                        let mut new_cluster = std::collections::HashSet::new();
                        new_cluster.insert(entity1.uri.clone());
                        new_cluster.insert(entity2.uri.clone());
                        entity_clusters.push(new_cluster);
                    }
                }
            }
        }

        // Merge entities in each cluster
        for cluster in entity_clusters {
            if cluster.len() > 1 {
                let merged_entity = self.merge_entities(kg, &cluster)?;
                report.merged_entities.push(MergedEntityInfo {
                    canonical_uri: merged_entity.uri.clone(),
                    merged_uris: cluster.into_iter().collect(),
                    confidence_score: merged_entity.confidence_score,
                });
            }
        }

        // Enrich entities with external data
        for entity in kg.entities.values_mut() {
            if let Some(enriched) = self.enrich_entity(entity)? {
                *entity = enriched;
                report.enriched_entities += 1;
            }
        }

        Ok(report)
    }

    /// Check if two entities are similar enough to be considered the same
    fn are_entities_similar(
        &self,
        entity1: &KnowledgeEntity,
        entity2: &KnowledgeEntity,
    ) -> Result<bool> {
        // Must be the same type
        if entity1.entity_type != entity2.entity_type {
            return Ok(false);
        }

        // Calculate similarity scores
        let mut max_similarity = 0.0;

        // Compare URIs
        for matcher in &self.string_matchers {
            let similarity = matcher.calculate_similarity(&entity1.uri, &entity2.uri)?;
            max_similarity = f64::max(max_similarity, similarity);
        }

        // Compare labels if available
        if let (Some(label1), Some(label2)) = (&entity1.label, &entity2.label) {
            for matcher in &self.string_matchers {
                let similarity = matcher.calculate_similarity(label1, label2)?;
                max_similarity = f64::max(max_similarity, similarity);
            }
        }

        // Compare properties
        for (key, value1) in &entity1.properties {
            if let Some(value2) = entity2.properties.get(key) {
                for matcher in &self.string_matchers {
                    let similarity = matcher.calculate_similarity(value1, value2)?;
                    max_similarity = f64::max(max_similarity, similarity);
                }
            }
        }

        Ok(max_similarity >= self.similarity_threshold)
    }

    /// Merge multiple entities into a single canonical entity
    fn merge_entities(
        &self,
        kg: &mut KnowledgeGraph,
        entity_uris: &std::collections::HashSet<String>,
    ) -> Result<KnowledgeEntity> {
        let entities: Vec<KnowledgeEntity> = entity_uris
            .iter()
            .filter_map(|uri| kg.entities.get(uri))
            .cloned()
            .collect();

        if entities.is_empty() {
            return Err(anyhow::anyhow!("No entities to merge"));
        }

        // Choose canonical entity (highest confidence score)
        let canonical = entities
            .iter()
            .max_by(|a, b| a.confidence_score.partial_cmp(&b.confidence_score).unwrap())
            .unwrap();

        // Merge properties from all entities
        let mut merged_properties = canonical.properties.clone();
        for entity in &entities {
            for (key, value) in &entity.properties {
                if !merged_properties.contains_key(key) {
                    merged_properties.insert(key.clone(), value.clone());
                }
            }
        }

        // Create merged entity
        let merged_entity = KnowledgeEntity {
            uri: canonical.uri.clone(),
            entity_type: canonical.entity_type.clone(),
            label: canonical.label.clone(),
            properties: merged_properties,
            confidence_score: (entities.iter().map(|e| e.confidence_score).sum::<f64>()
                / entities.len() as f64)
                .min(1.0),
        };

        // Remove old entities and add merged entity
        for uri in entity_uris {
            if uri != &canonical.uri {
                kg.entities.remove(uri);
                // Update relationships to point to canonical entity
                for relationship in &mut kg.relationships {
                    if relationship.subject == *uri {
                        relationship.subject = canonical.uri.clone();
                    }
                    if relationship.object == *uri {
                        relationship.object = canonical.uri.clone();
                    }
                }
            }
        }

        kg.entities
            .insert(canonical.uri.clone(), merged_entity.clone());
        Ok(merged_entity)
    }

    /// Enrich an entity with external data sources
    fn enrich_entity(&self, entity: &KnowledgeEntity) -> Result<Option<KnowledgeEntity>> {
        let mut enriched = entity.clone();
        let mut was_enriched = false;

        for resolver in &self.external_resolvers {
            if let Some(external_data) = resolver.resolve_entity(entity)? {
                // Merge external data
                for (key, value) in external_data.properties {
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        enriched.properties.entry(key)
                    {
                        e.insert(value);
                        was_enriched = true;
                    }
                }

                if enriched.label.is_none() && external_data.label.is_some() {
                    enriched.label = external_data.label;
                    was_enriched = true;
                }

                // Update confidence score
                enriched.confidence_score =
                    (enriched.confidence_score + external_data.confidence_score) / 2.0;
            }
        }

        if was_enriched {
            Ok(Some(enriched))
        } else {
            Ok(None)
        }
    }
}

/// Trait for string similarity matching
pub trait StringMatcher: Send + Sync {
    fn calculate_similarity(&self, str1: &str, str2: &str) -> Result<f64>;
}

/// Trait for external entity resolution
pub trait ExternalResolver: Send + Sync {
    fn resolve_entity(&self, entity: &KnowledgeEntity) -> Result<Option<ExternalEntityData>>;
}

/// External entity data from resolution
#[derive(Debug, Clone)]
pub struct ExternalEntityData {
    pub label: Option<String>,
    pub properties: HashMap<String, String>,
    pub confidence_score: f64,
}

/// Report of entity resolution activities
#[derive(Debug)]
pub struct EntityResolutionReport {
    pub merged_entities: Vec<MergedEntityInfo>,
    pub enriched_entities: usize,
}

impl EntityResolutionReport {
    fn new() -> Self {
        Self {
            merged_entities: Vec::new(),
            enriched_entities: 0,
        }
    }
}

/// Information about merged entities
#[derive(Debug)]
pub struct MergedEntityInfo {
    pub canonical_uri: String,
    pub merged_uris: Vec<String>,
    pub confidence_score: f64,
}

/// Exact string matcher
pub struct ExactMatcher;

impl StringMatcher for ExactMatcher {
    fn calculate_similarity(&self, str1: &str, str2: &str) -> Result<f64> {
        if str1 == str2 {
            Ok(1.0)
        } else {
            Ok(0.0)
        }
    }
}

/// Levenshtein distance-based matcher
pub struct LevenshteinMatcher;

impl StringMatcher for LevenshteinMatcher {
    fn calculate_similarity(&self, str1: &str, str2: &str) -> Result<f64> {
        let distance = levenshtein_distance(str1, str2);
        let max_len = str1.len().max(str2.len());

        if max_len == 0 {
            Ok(1.0)
        } else {
            Ok(1.0 - (distance as f64 / max_len as f64))
        }
    }
}

/// Token-based matcher (Jaccard similarity)
pub struct TokenMatcher;

impl StringMatcher for TokenMatcher {
    fn calculate_similarity(&self, str1: &str, str2: &str) -> Result<f64> {
        let tokens1: std::collections::HashSet<&str> = str1.split_whitespace().collect();
        let tokens2: std::collections::HashSet<&str> = str2.split_whitespace().collect();

        let intersection = tokens1.intersection(&tokens2).count();
        let union = tokens1.union(&tokens2).count();

        if union == 0 {
            Ok(1.0)
        } else {
            Ok(intersection as f64 / union as f64)
        }
    }
}

/// Phonetic matcher using Soundex algorithm
pub struct PhoneticMatcher;

impl StringMatcher for PhoneticMatcher {
    fn calculate_similarity(&self, str1: &str, str2: &str) -> Result<f64> {
        let soundex1 = soundex(str1);
        let soundex2 = soundex(str2);

        if soundex1 == soundex2 {
            Ok(0.8) // High but not perfect similarity for phonetic matches
        } else {
            Ok(0.0)
        }
    }
}

/// GeoNames resolver for geographic entities
pub struct GeoNamesResolver;

impl ExternalResolver for GeoNamesResolver {
    fn resolve_entity(&self, entity: &KnowledgeEntity) -> Result<Option<ExternalEntityData>> {
        // Mock implementation - in real system would call GeoNames API
        if entity.entity_type == "Location" || entity.uri.contains("geo") {
            let mut properties = HashMap::new();
            properties.insert("country".to_string(), "Unknown".to_string());
            properties.insert("latitude".to_string(), "0.0".to_string());
            properties.insert("longitude".to_string(), "0.0".to_string());

            Ok(Some(ExternalEntityData {
                label: Some("Geographic Location".to_string()),
                properties,
                confidence_score: 0.7,
            }))
        } else {
            Ok(None)
        }
    }
}

/// Company resolver for business entities
pub struct CompanyResolver;

impl ExternalResolver for CompanyResolver {
    fn resolve_entity(&self, entity: &KnowledgeEntity) -> Result<Option<ExternalEntityData>> {
        // Mock implementation - in real system would call business directory APIs
        if entity.entity_type == "Manufacturer" || entity.entity_type == "Farmer" {
            let mut properties = HashMap::new();
            properties.insert("industry".to_string(), "Agriculture".to_string());
            properties.insert("founded".to_string(), "Unknown".to_string());

            Ok(Some(ExternalEntityData {
                label: None,
                properties,
                confidence_score: 0.6,
            }))
        } else {
            Ok(None)
        }
    }
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(str1: &str, str2: &str) -> usize {
    let chars1: Vec<char> = str1.chars().collect();
    let chars2: Vec<char> = str2.chars().collect();
    let len1 = chars1.len();
    let len2 = chars2.len();

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[len1][len2]
}

/// Simple Soundex implementation
fn soundex(s: &str) -> String {
    lazy_static! {
        static ref SOUNDEX_MAP: HashMap<char, char> = {
            let mut m = HashMap::new();
            for c in "BFPV".chars() {
                m.insert(c, '1');
            }
            for c in "CGJKQSXZ".chars() {
                m.insert(c, '2');
            }
            for c in "DT".chars() {
                m.insert(c, '3');
            }
            for c in "L".chars() {
                m.insert(c, '4');
            }
            for c in "MN".chars() {
                m.insert(c, '5');
            }
            for c in "R".chars() {
                m.insert(c, '6');
            }
            m
        };
    }

    let s = s.to_uppercase();
    let chars: Vec<char> = s.chars().filter(|c| c.is_alphabetic()).collect();

    if chars.is_empty() {
        return "0000".to_string();
    }

    let mut result = String::new();
    result.push(chars[0]);

    let mut prev_code = SOUNDEX_MAP.get(&chars[0]).copied().unwrap_or('0');

    for &c in chars.iter().skip(1) {
        if let Some(&code) = SOUNDEX_MAP.get(&c) {
            if code != prev_code && code != '0' {
                result.push(code);
                if result.len() == 4 {
                    break;
                }
            }
            prev_code = code;
        } else {
            prev_code = '0';
        }
    }

    while result.len() < 4 {
        result.push('0');
    }

    result
}

impl Default for EntityLinker {
    fn default() -> Self {
        Self::new()
    }
}
