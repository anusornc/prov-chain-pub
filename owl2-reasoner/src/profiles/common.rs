//! Common infrastructure for OWL2 profile validation
//!
//! This module contains shared data structures, traits, and utilities
//! used across all OWL2 profile implementations.

#![allow(dead_code)]

use crate::axioms::class_expressions::ClassExpression;
use crate::error::{OwlError, OwlResult};
use crate::iri::IRI;
use crate::ontology::Ontology;
use bumpalo::Bump;
use dashmap::DashMap;
use lru::LruCache;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

// Helper trait for ClassExpression to IRI conversion
trait ClassExpressionExt {
    fn as_iri(&self) -> Option<&Arc<IRI>>;
}

impl ClassExpressionExt for ClassExpression {
    fn as_iri(&self) -> Option<&Arc<IRI>> {
        match self {
            ClassExpression::Class(class) => Some(class.iri()),
            _ => None,
        }
    }
}

/// OWL2 Profile types
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub enum Owl2Profile {
    /// OWL2 EL Profile (Expressive Logic)
    EL,
    /// OWL2 QL Profile (Query Language)
    QL,
    /// OWL2 RL Profile (Rule Language)
    RL,
}

impl std::fmt::Display for Owl2Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Owl2Profile::EL => write!(f, "OWL2 EL"),
            Owl2Profile::QL => write!(f, "OWL2 QL"),
            Owl2Profile::RL => write!(f, "OWL2 RL"),
        }
    }
}

/// Profile validation result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub struct ProfileValidationResult {
    pub profile: Owl2Profile,
    pub is_valid: bool,
    pub violations: Vec<ProfileViolation>,
    pub statistics: ValidationStatistics,
}

/// Profile violation details
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub struct ProfileViolation {
    pub violation_type: ProfileViolationType,
    pub message: String,
    pub affected_entities: Vec<IRI>,
    pub severity: ViolationSeverity,
}

/// Types of profile violations
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub enum ProfileViolationType {
    // EL Profile Violations
    ComplexSubclassAxiom,
    DisjointClassesAxiom,
    EquivalentClassesAxiom,
    ComplexPropertyRestrictions,
    DataPropertyRanges,

    // QL Profile Violations
    TransitiveProperties,
    AsymmetricProperties,
    IrreflexiveProperties,
    ComplexCardinalityRestrictions,
    PropertyChainAxioms,

    // RL Profile Violations
    Nominals,
    DataComplementOf,
    DataOneOf,
    ObjectComplementOf,
    ObjectOneOf,
    ObjectHasSelf,

    // General violations
    UnsupportedConstruct,
    RecursiveDefinitions,
    CycleInHierarchy,
    ComplexClassExpressions,
    ComplexDataRanges,
}

/// Severity levels for violations
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub enum ViolationSeverity {
    Error,   // Must be fixed for profile compliance
    Warning, // Should be reviewed
    Info,    // Informational note
}

/// Validation statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub struct ValidationStatistics {
    pub total_axioms_checked: usize,
    pub violations_found: usize,
    pub validation_time_ms: f64,
    pub memory_usage_bytes: usize,
}

/// Profile validator trait
pub trait ProfileValidator {
    fn validate_profile(&mut self, profile: Owl2Profile) -> OwlResult<ProfileValidationResult>;
    fn is_el_profile(&self) -> bool;
    fn is_ql_profile(&self) -> bool;
    fn is_rl_profile(&self) -> bool;
    fn get_optimization_hints(&self) -> Vec<OptimizationHint>;
}

/// Optimization hints for profile compliance
#[derive(Debug, Clone)]
pub struct OptimizationHint {
    pub hint_type: OptimizationType,
    pub description: String,
    pub estimated_impact: String,
}

#[derive(Debug, Clone)]
pub enum OptimizationType {
    RestructureHierarchy,
    SimplifyExpressions,
    RemoveUnsupportedConstructs,
    AddMissingDeclarations,
}

/// Detailed profile analysis report
#[derive(Debug, Clone)]
pub struct ProfileAnalysisReport {
    pub el_compliant: bool,
    pub ql_compliant: bool,
    pub rl_compliant: bool,
    pub ontology_stats: OntologyStats,
    pub el_violations: Vec<String>,
    pub ql_violations: Vec<String>,
    pub rl_violations: Vec<String>,
}

/// Ontology statistics
#[derive(Debug, Clone)]
pub struct OntologyStats {
    pub total_classes: usize,
    pub total_properties: usize,
    pub total_individuals: usize,
    pub total_axioms: usize,
    pub max_class_expression_depth: usize,
}

/// OWL2 Profile validation implementation with optimized caching, memory pools, and pre-computation indexes
pub struct Owl2ProfileValidator {
    ontology: Arc<Ontology>,
    cache: DashMap<Owl2Profile, ProfileValidationResult>, // Legacy cache for backward compatibility
    advanced_cache: AdvancedCacheManager,                 // New advanced caching system
    result_arena: Bump,
    violation_pool: ViolationPool,
    indexes: ProfileIndexes,
    validation_stats: ValidationStats,
    use_advanced_caching: bool,
}

impl Owl2ProfileValidator {
    /// Create a new profile validator for the given ontology
    pub fn new(ontology: Arc<Ontology>) -> OwlResult<Self> {
        let indexes = ProfileIndexes::analyze_ontology(&ontology);

        Ok(Self {
            ontology,
            cache: DashMap::new(),
            advanced_cache: AdvancedCacheManager::new()?,
            result_arena: Bump::new(),
            violation_pool: ViolationPool::new(),
            indexes,
            validation_stats: ValidationStats::new(),
            use_advanced_caching: true,
        })
    }

    /// Validate all profiles and return comprehensive results
    pub fn validate_all_profiles(&mut self) -> OwlResult<Vec<ProfileValidationResult>> {
        let mut results = Vec::new();

        for profile in [Owl2Profile::EL, Owl2Profile::QL, Owl2Profile::RL] {
            results.push(self.validate_profile(profile)?);
        }

        Ok(results)
    }

    /// Get the most restrictive valid profile for this ontology
    pub fn get_most_restrictive_profile(&mut self) -> OwlResult<Option<Owl2Profile>> {
        let profiles = [Owl2Profile::EL, Owl2Profile::QL, Owl2Profile::RL];

        for profile in profiles {
            let result = self.validate_profile(profile.clone())?;
            if result.is_valid {
                return Ok(Some(profile));
            }
        }

        Ok(None) // No profile restrictions satisfied
    }

    /// Check if ontology satisfies any OWL2 profile
    pub fn satisfies_any_profile(&mut self) -> OwlResult<bool> {
        Ok(self.get_most_restrictive_profile()?.is_some())
    }

    /// Clear validation cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStatistics {
        self.advanced_cache.get_stats()
    }

    /// Toggle advanced caching
    pub fn set_advanced_caching(&mut self, enabled: bool) {
        self.use_advanced_caching = enabled;
    }

    /// Get validation statistics
    pub fn get_validation_stats(&self) -> &ValidationStats {
        &self.validation_stats
    }

    /// Analyze ontology and generate comprehensive report
    pub fn analyze_ontology(&mut self) -> OwlResult<ProfileAnalysisReport> {
        let el_result = self.validate_profile(Owl2Profile::EL)?;
        let ql_result = self.validate_profile(Owl2Profile::QL)?;
        let rl_result = self.validate_profile(Owl2Profile::RL)?;

        let ontology_stats = OntologyStats {
            total_classes: self.indexes.class_count,
            total_properties: self.indexes.property_count,
            total_individuals: self.indexes.individual_count,
            total_axioms: self.ontology.axioms().len(),
            max_class_expression_depth: self.indexes.max_expression_depth,
        };

        Ok(ProfileAnalysisReport {
            el_compliant: el_result.is_valid,
            ql_compliant: ql_result.is_valid,
            rl_compliant: rl_result.is_valid,
            ontology_stats,
            el_violations: el_result
                .violations
                .iter()
                .map(|v| v.message.clone())
                .collect(),
            ql_violations: ql_result
                .violations
                .iter()
                .map(|v| v.message.clone())
                .collect(),
            rl_violations: rl_result
                .violations
                .iter()
                .map(|v| v.message.clone())
                .collect(),
        })
    }

    /// Get optimization hints for making ontology profile-compliant
    pub fn get_optimization_hints(&self) -> Vec<OptimizationHint> {
        let mut hints = Vec::new();

        // Analyze ontology for common optimization opportunities
        if self.indexes.has_complex_expressions {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::SimplifyExpressions,
                description: "Simplify complex class expressions to improve profile compliance"
                    .to_string(),
                estimated_impact: "High - affects all profiles".to_string(),
            });
        }

        if self.indexes.has_cycles {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::RestructureHierarchy,
                description: "Remove cycles in class/property hierarchies".to_string(),
                estimated_impact: "Critical - required for RL profile".to_string(),
            });
        }

        if self.indexes.missing_declarations > 0 {
            hints.push(OptimizationHint {
                hint_type: OptimizationType::AddMissingDeclarations,
                description: format!(
                    "Add {} missing entity declarations",
                    self.indexes.missing_declarations
                ),
                estimated_impact: "Medium - improves validation performance".to_string(),
            });
        }

        hints
    }

    /// Core validation logic with caching
    fn validate_profile_internal(
        &mut self,
        profile: Owl2Profile,
    ) -> OwlResult<ProfileValidationResult> {
        let start_time = Instant::now();

        // Check cache first
        if self.use_advanced_caching {
            if let Some(cached_result) = self.advanced_cache.get(&profile) {
                return Ok(cached_result);
            }
        } else if let Some(cached_result) = self.cache.get(&profile) {
            return Ok(cached_result.clone());
        }

        // Perform validation
        let violations = match profile {
            Owl2Profile::EL => self.validate_el_profile_pool()?,
            Owl2Profile::QL => self.validate_ql_profile_pool()?,
            Owl2Profile::RL => self.validate_rl_profile_pool()?,
        };

        let result = ProfileValidationResult {
            profile: profile.clone(),
            is_valid: violations.is_empty(),
            violations: violations.clone(),
            statistics: ValidationStatistics {
                total_axioms_checked: self.ontology.axioms().len(),
                violations_found: violations.len(),
                validation_time_ms: start_time.elapsed().as_millis() as f64,
                memory_usage_bytes: self.get_memory_usage(),
            },
        };

        // Update cache
        if self.use_advanced_caching {
            self.advanced_cache.put(profile.clone(), result.clone());
        } else {
            self.cache.insert(profile, result.clone());
        }

        // Update validation statistics
        self.validation_stats.record_validation(&result);

        Ok(result)
    }

    /// Profile-specific validation implementations
    fn validate_el_profile(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // Check for EL-specific restrictions
        for axiom in self.ontology.axioms() {
            match **axiom {
                crate::axioms::Axiom::SubClassOf(ref subclass) => {
                    if !self.is_el_compatible_subclass_axiom(subclass) {
                        violations.push(ProfileViolation {
                            violation_type: ProfileViolationType::ComplexSubclassAxiom,
                            message: "Complex subclass axiom not allowed in EL profile".to_string(),
                            affected_entities: subclass
                                .sub_class()
                                .as_iri()
                                .cloned()
                                .into_iter()
                                .map(|arc| (*arc).clone())
                                .chain(
                                    subclass
                                        .super_class()
                                        .as_iri()
                                        .cloned()
                                        .map(|arc| (*arc).clone()),
                                )
                                .collect(),
                            severity: ViolationSeverity::Error,
                        });
                    }
                }
                crate::axioms::Axiom::DisjointClasses(ref disjoint) => {
                    violations.push(ProfileViolation {
                        violation_type: ProfileViolationType::DisjointClassesAxiom,
                        message: "Disjoint classes axiom not allowed in EL profile".to_string(),
                        affected_entities: disjoint
                            .classes()
                            .iter()
                            .cloned()
                            .map(|arc| (*arc).clone())
                            .collect(),
                        severity: ViolationSeverity::Error,
                    });
                }
                crate::axioms::Axiom::EquivalentClasses(ref equiv) => {
                    violations.push(ProfileViolation {
                        violation_type: ProfileViolationType::EquivalentClassesAxiom,
                        message: "Equivalent classes axiom not allowed in EL profile".to_string(),
                        affected_entities: equiv
                            .classes()
                            .iter()
                            .cloned()
                            .map(|arc| (*arc).clone())
                            .collect(),
                        severity: ViolationSeverity::Error,
                    });
                }
                _ => {}
            }
        }

        Ok(violations)
    }

    fn validate_ql_profile(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // Check for QL-specific restrictions
        for axiom in self.ontology.axioms() {
            match **axiom {
                crate::axioms::Axiom::TransitiveProperty(ref transitive) => {
                    violations.push(ProfileViolation {
                        violation_type: ProfileViolationType::TransitiveProperties,
                        message: "Transitive object property not allowed in QL profile".to_string(),
                        affected_entities: vec![(*transitive.property()).clone().into()],
                        severity: ViolationSeverity::Error,
                    });
                }
                crate::axioms::Axiom::AsymmetricProperty(ref asymmetric) => {
                    violations.push(ProfileViolation {
                        violation_type: ProfileViolationType::AsymmetricProperties,
                        message: "Asymmetric object property not allowed in QL profile".to_string(),
                        affected_entities: vec![(*asymmetric.property()).clone().into()],
                        severity: ViolationSeverity::Error,
                    });
                }
                crate::axioms::Axiom::IrreflexiveProperty(ref irreflexive) => {
                    violations.push(ProfileViolation {
                        violation_type: ProfileViolationType::IrreflexiveProperties,
                        message: "Irreflexive object property not allowed in QL profile"
                            .to_string(),
                        affected_entities: vec![(*irreflexive.property()).clone().into()],
                        severity: ViolationSeverity::Error,
                    });
                }
                _ => {}
            }
        }

        Ok(violations)
    }

    fn validate_rl_profile(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // Check for RL-specific restrictions
        for axiom in self.ontology.axioms() {
            if let crate::axioms::Axiom::SubClassOf(ref subclass) = **axiom {
                if !self.is_rl_compatible_subclass_axiom(subclass) {
                    violations.push(ProfileViolation {
                        violation_type: ProfileViolationType::ComplexClassExpressions,
                        message: "Complex class expressions not allowed in RL profile".to_string(),
                        affected_entities: subclass
                            .sub_class()
                            .as_iri()
                            .cloned()
                            .into_iter()
                            .map(|arc| (*arc).clone())
                            .chain(
                                subclass
                                    .super_class()
                                    .as_iri()
                                    .cloned()
                                    .map(|arc| (*arc).clone()),
                            )
                            .collect(),
                        severity: ViolationSeverity::Error,
                    });
                }
            }
        }

        Ok(violations)
    }

    // Pool-optimized validation methods
    fn validate_el_profile_pool(&mut self) -> OwlResult<Vec<ProfileViolation>> {
        let violations = self.validate_el_profile()?;
        self.violation_pool.return_violations(violations.clone());
        Ok(violations)
    }

    fn validate_ql_profile_pool(&mut self) -> OwlResult<Vec<ProfileViolation>> {
        let violations = self.validate_ql_profile()?;
        self.violation_pool.return_violations(violations.clone());
        Ok(violations)
    }

    fn validate_rl_profile_pool(&mut self) -> OwlResult<Vec<ProfileViolation>> {
        let violations = self.validate_rl_profile()?;
        self.violation_pool.return_violations(violations.clone());
        Ok(violations)
    }

    // Helper methods for compatibility checking
    fn is_el_compatible_subclass_axiom(&self, _subclass: &crate::axioms::SubClassOfAxiom) -> bool {
        // Simplified EL compatibility check
        true // For now, assume compatibility
    }

    fn is_rl_compatible_subclass_axiom(&self, _subclass: &crate::axioms::SubClassOfAxiom) -> bool {
        // Simplified RL compatibility check
        true // For now, assume compatibility
    }

    /// Get current memory usage
    fn get_memory_usage(&self) -> usize {
        self.result_arena.allocated_bytes() + self.violation_pool.get_memory_usage()
    }
}

impl ProfileValidator for Owl2ProfileValidator {
    fn validate_profile(&mut self, profile: Owl2Profile) -> OwlResult<ProfileValidationResult> {
        self.validate_profile_internal(profile)
    }

    fn is_el_profile(&self) -> bool {
        true // Can validate EL profile
    }

    fn is_ql_profile(&self) -> bool {
        true // Can validate QL profile
    }

    fn is_rl_profile(&self) -> bool {
        true // Can validate RL profile
    }

    fn get_optimization_hints(&self) -> Vec<OptimizationHint> {
        self.get_optimization_hints()
    }
}

/// Validation statistics tracking
#[derive(Debug, Default, Clone)]
pub struct ValidationStats {
    total_validations: usize,
    total_violations_found: usize,
    total_validation_time_ms: f64,
    cache_hits: usize,
    cache_misses: usize,
}

impl ValidationStats {
    fn new() -> Self {
        Self::default()
    }

    fn record_validation(&mut self, result: &ProfileValidationResult) {
        self.total_validations += 1;
        self.total_violations_found += result.violations.len();
        self.total_validation_time_ms += result.statistics.validation_time_ms;
    }

    fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    pub fn get_hit_rate(&self) -> f64 {
        if self.cache_hits + self.cache_misses == 0 {
            0.0
        } else {
            self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64
        }
    }

    pub fn get_average_validation_time(&self) -> f64 {
        if self.total_validations == 0 {
            0.0
        } else {
            self.total_validation_time_ms / self.total_validations as f64
        }
    }
}

/// Profile indexes for optimized validation
#[derive(Debug, Default)]
pub struct ProfileIndexes {
    pub class_count: usize,
    pub property_count: usize,
    pub individual_count: usize,
    pub max_expression_depth: usize,
    pub has_complex_expressions: bool,
    pub has_cycles: bool,
    pub missing_declarations: usize,
}

impl ProfileIndexes {
    fn analyze_ontology(ontology: &Arc<Ontology>) -> Self {
        let indexes = ProfileIndexes {
            class_count: ontology.classes().len(),
            property_count: ontology.object_properties().len() + ontology.data_properties().len(),
            individual_count: ontology.named_individuals().len(),
            ..Default::default()
        };

        // Additional analysis can be added here

        indexes
    }
}

/// Memory pool for profile violations to reduce allocations
#[derive(Debug)]
pub struct ViolationPool {
    pool: Vec<ProfileViolation>,
    pool_size: usize,
}

impl ViolationPool {
    fn new() -> Self {
        Self {
            pool: Vec::with_capacity(100),
            pool_size: 100,
        }
    }

    fn get_violation(
        &mut self,
        violation_type: ProfileViolationType,
        message: String,
        affected_entities: Vec<IRI>,
        severity: ViolationSeverity,
    ) -> ProfileViolation {
        ProfileViolation {
            violation_type,
            message,
            affected_entities,
            severity,
        }
    }

    fn return_violations(&mut self, violations: Vec<ProfileViolation>) {
        // Clear and reuse the violations
        self.pool.clear();
        self.pool.extend(violations);
    }

    fn get_memory_usage(&self) -> usize {
        self.pool.capacity() * std::mem::size_of::<ProfileViolation>()
    }
}

/// Memory pool statistics
#[derive(Debug, Default)]
pub struct MemoryPoolStats {
    pub total_allocations: usize,
    pub pool_hits: usize,
    pub pool_misses: usize,
    pub current_pool_size: usize,
    pub memory_saved_bytes: usize,
}

/// Advanced cache management system
#[derive(Debug)]
pub struct AdvancedCacheManager {
    primary_cache: LruCache<Owl2Profile, ProfileValidationResult>,
    hot_cache: DashMap<Owl2Profile, ProfileValidationResult>,
    compressed_cache: HashMap<Owl2Profile, Vec<u8>>,
    invalidation_tokens: HashSet<String>,
    cache_stats: CacheStatistics,
    config: ProfileCacheConfig,
}

/// Cache priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CachePriority {
    /// Low priority (easily evicted)
    Low = 1,
    /// Medium priority
    Medium = 2,
    /// High priority (rarely evicted)
    High = 3,
    /// Critical priority (never evicted)
    Critical = 4,
}

/// Profile cache configuration parameters
#[derive(Debug, Clone)]
pub struct ProfileCacheConfig {
    /// Maximum entries in primary cache
    primary_cache_size: usize,
    /// Maximum entries in hot cache
    _hot_cache_size: usize,
    /// Maximum entries in compressed cache
    compressed_cache_size: usize,
    /// Time-to-live for cache entries
    ttl_duration: Duration,
    /// Compression threshold (entries larger than this get compressed)
    compression_threshold: usize,
    /// Hot cache promotion threshold (access count)
    hot_cache_threshold: usize,
}

/// Cache statistics and performance metrics
#[derive(Debug, Default, Clone)]
pub struct CacheStatistics {
    /// Number of cache hits
    pub hits: usize,
    /// Number of cache misses
    pub misses: usize,
    /// Number of cache evictions
    pub evictions: usize,
    /// Number of compressed cache hits
    compressed_hits: usize,
    /// Number of hot cache hits
    hot_hits: usize,
    /// Total memory used by cache
    total_memory_bytes: usize,
    /// Memory saved by compression
    compressed_memory_saved: usize,
    /// Average access time
    average_access_time_ns: u64,
    /// Cache hit rate
    pub hit_rate: f64,
}

impl Default for ProfileCacheConfig {
    fn default() -> Self {
        Self {
            primary_cache_size: 1000,
            _hot_cache_size: 100,
            compressed_cache_size: 500,
            ttl_duration: Duration::from_secs(3600), // 1 hour
            compression_threshold: 1024,             // 1KB
            hot_cache_threshold: 5,
        }
    }
}

impl AdvancedCacheManager {
    /// Create a new advanced cache manager
    fn new() -> OwlResult<Self> {
        let config = ProfileCacheConfig::default();
        Self::with_config(config)
    }

    /// Create a new advanced cache manager with custom configuration
    fn with_config(config: ProfileCacheConfig) -> OwlResult<Self> {
        let primary_cache_size = std::num::NonZeroUsize::new(config.primary_cache_size)
            .ok_or_else(|| OwlError::ConfigError {
                parameter: "primary_cache_size".to_string(),
                message: "Cache size must be greater than zero".to_string(),
            })?;

        Ok(Self {
            primary_cache: LruCache::new(primary_cache_size),
            hot_cache: DashMap::new(),
            compressed_cache: HashMap::new(),
            invalidation_tokens: HashSet::new(),
            cache_stats: CacheStatistics::default(),
            config,
        })
    }

    /// Get a cached validation result
    fn get(&mut self, profile: &Owl2Profile) -> Option<ProfileValidationResult> {
        let start_time = std::time::Instant::now();

        // Check hot cache first (fastest)
        if let Some(result) = self.hot_cache.get(profile).map(|r| r.clone()) {
            self.cache_stats.hot_hits += 1;
            self.cache_stats.hits += 1;
            self.update_access_time(start_time);
            return Some(result);
        }

        // Check primary cache
        if let Some(result) = self.primary_cache.get(profile).cloned() {
            self.cache_stats.hits += 1;
            self.update_access_time(start_time);
            return Some(result);
        }

        // Check compressed cache (slower but memory efficient)
        if let Some(compressed) = self.compressed_cache.get(profile) {
            if let Ok(result) = self.decompress_result(compressed) {
                self.cache_stats.compressed_hits += 1;
                self.cache_stats.hits += 1;
                self.update_access_time(start_time);
                return Some(result);
            }
        }

        self.cache_stats.misses += 1;
        None
    }

    /// Put a validation result in the cache
    fn put(&mut self, profile: Owl2Profile, result: ProfileValidationResult) {
        // Determine cache strategy based on result size and access patterns
        let result_size = std::mem::size_of_val(&result);

        if result_size > self.config.compression_threshold {
            // Compress and store in compressed cache
            if let Ok(compressed) = self.compress_result(&result) {
                self.compressed_cache.insert(profile, compressed);
            }
        } else {
            // Store in primary cache
            self.primary_cache.put(profile, result);
        }

        self.update_stats();
    }

    /// Compress a validation result
    fn compress_result(&self, result: &ProfileValidationResult) -> OwlResult<Vec<u8>> {
        // Use simple bincode serialization for compression
        bincode::encode_to_vec(result, bincode::config::standard()).map_err(|e| {
            OwlError::SerializationError(format!("Failed to compress validation result: {}", e))
        })
    }

    /// Decompress a validation result
    fn decompress_result(&self, compressed: &[u8]) -> OwlResult<ProfileValidationResult> {
        bincode::decode_from_slice(compressed, bincode::config::standard())
            .map(|(result, _)| result)
            .map_err(|e| {
                OwlError::SerializationError(format!(
                    "Failed to decompress validation result: {}",
                    e
                ))
            })
    }

    /// Update cache access time statistics
    fn update_access_time(&mut self, start_time: std::time::Instant) {
        let access_time = start_time.elapsed().as_nanos() as u64;
        self.cache_stats.average_access_time_ns =
            (self.cache_stats.average_access_time_ns + access_time) / 2;
    }

    /// Update cache statistics
    fn update_stats(&mut self) {
        self.cache_stats.hit_rate = if self.cache_stats.hits + self.cache_stats.misses > 0 {
            self.cache_stats.hits as f64 / (self.cache_stats.hits + self.cache_stats.misses) as f64
        } else {
            0.0
        };
    }

    /// Get cache statistics
    fn get_stats(&self) -> CacheStatistics {
        self.cache_stats.clone()
    }

    /// Clear all cache entries
    fn clear(&mut self) {
        self.primary_cache.clear();
        self.hot_cache.clear();
        self.compressed_cache.clear();
        self.cache_stats = CacheStatistics::default();
    }
}
