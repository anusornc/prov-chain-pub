//! Error types for ontology management and SHACL validation

use std::fmt;

/// Errors that can occur during ontology operations
#[derive(Debug)]
pub enum OntologyError {
    /// Ontology file not found at the specified path
    OntologyNotFound { path: String },
    /// Invalid ontology file path
    InvalidOntologyPath { path: String, reason: String },
    /// Error loading ontology file
    OntologyLoadError {
        path: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// Ontology parsing error
    OntologyParseError { path: String, message: String },
    /// Network consistency error - participants using different ontologies
    ConsistencyError {
        local_hash: String,
        network_hash: String,
        message: String,
    },
}

impl fmt::Display for OntologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OntologyError::OntologyNotFound { path } => {
                write!(f, "Ontology file not found: {}", path)
            }
            OntologyError::InvalidOntologyPath { path, reason } => {
                write!(f, "Invalid ontology path '{}': {}", path, reason)
            }
            OntologyError::OntologyLoadError { path, source } => {
                write!(f, "Failed to load ontology from '{}': {}", path, source)
            }
            OntologyError::OntologyParseError { path, message } => {
                write!(f, "Failed to parse ontology from '{}': {}", path, message)
            }
            OntologyError::ConsistencyError {
                local_hash,
                network_hash,
                message,
            } => {
                write!(
                    f,
                    "Ontology consistency error: local hash {} != network hash {}. {}",
                    local_hash, network_hash, message
                )
            }
        }
    }
}

impl std::error::Error for OntologyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OntologyError::OntologyLoadError { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

/// Errors that can occur during SHACL validation
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Human-readable error message
    pub message: String,
    /// Specific SHACL shape violations
    pub shape_violations: Vec<ShapeViolation>,
    /// Transaction ID if available
    pub transaction_id: Option<String>,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(message: String) -> Self {
        Self {
            message,
            shape_violations: Vec::new(),
            transaction_id: None,
        }
    }

    /// Create a validation error with shape violations
    pub fn with_violations(message: String, violations: Vec<ShapeViolation>) -> Self {
        Self {
            message,
            shape_violations: violations,
            transaction_id: None,
        }
    }

    /// Add a transaction ID to the error
    pub fn with_transaction_id(mut self, transaction_id: String) -> Self {
        self.transaction_id = Some(transaction_id);
        self
    }

    /// Add a shape violation to the error
    pub fn add_violation(&mut self, violation: ShapeViolation) {
        self.shape_violations.push(violation);
    }

    /// Check if this error has any shape violations
    pub fn has_violations(&self) -> bool {
        !self.shape_violations.is_empty()
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SHACL Validation Error: {}", self.message)?;

        if let Some(tx_id) = &self.transaction_id {
            write!(f, " (Transaction: {})", tx_id)?;
        }

        if !self.shape_violations.is_empty() {
            write!(f, "\nViolations:")?;
            for violation in &self.shape_violations {
                write!(f, "\n  - {}", violation)?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for ValidationError {}

/// Specific SHACL shape violation details
#[derive(Debug, Clone)]
pub struct ShapeViolation {
    /// The SHACL shape that was violated
    pub shape_id: String,
    /// The property path that caused the violation
    pub property_path: Option<String>,
    /// The value that violated the constraint
    pub value: Option<String>,
    /// The specific constraint that was violated
    pub constraint_type: ConstraintType,
    /// Human-readable violation message
    pub message: String,
    /// Severity level of the violation
    pub severity: ViolationSeverity,
}

impl ShapeViolation {
    /// Create a new shape violation
    pub fn new(shape_id: String, constraint_type: ConstraintType, message: String) -> Self {
        Self {
            shape_id,
            property_path: None,
            value: None,
            constraint_type,
            message,
            severity: ViolationSeverity::Violation,
        }
    }

    /// Set the property path for this violation
    pub fn with_property_path(mut self, path: String) -> Self {
        self.property_path = Some(path);
        self
    }

    /// Set the violating value
    pub fn with_value(mut self, value: String) -> Self {
        self.value = Some(value);
        self
    }

    /// Set the severity level
    pub fn with_severity(mut self, severity: ViolationSeverity) -> Self {
        self.severity = severity;
        self
    }
}

impl fmt::Display for ShapeViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.severity, self.shape_id, self.message)?;

        if let Some(path) = &self.property_path {
            write!(f, " (Property: {})", path)?;
        }

        if let Some(value) = &self.value {
            write!(f, " (Value: {})", value)?;
        }

        Ok(())
    }
}

/// Types of SHACL constraints that can be violated
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    /// Required property is missing
    MinCount,
    /// Too many values for a property
    MaxCount,
    /// Value doesn't match required datatype
    Datatype,
    /// Value not in allowed list
    In,
    /// Value doesn't match required class
    Class,
    /// Value doesn't match required node kind
    NodeKind,
    /// Value doesn't match pattern
    Pattern,
    /// Value outside allowed range
    MinInclusive,
    /// Value outside allowed range
    MaxInclusive,
    /// Custom constraint violation
    Custom(String),
}

impl fmt::Display for ConstraintType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstraintType::MinCount => write!(f, "MinCount"),
            ConstraintType::MaxCount => write!(f, "MaxCount"),
            ConstraintType::Datatype => write!(f, "Datatype"),
            ConstraintType::In => write!(f, "In"),
            ConstraintType::Class => write!(f, "Class"),
            ConstraintType::NodeKind => write!(f, "NodeKind"),
            ConstraintType::Pattern => write!(f, "Pattern"),
            ConstraintType::MinInclusive => write!(f, "MinInclusive"),
            ConstraintType::MaxInclusive => write!(f, "MaxInclusive"),
            ConstraintType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// Severity levels for SHACL violations
#[derive(Debug, Clone, PartialEq)]
pub enum ViolationSeverity {
    /// Information level - not blocking
    Info,
    /// Warning level - not blocking but should be addressed
    Warning,
    /// Violation level - blocking in strict mode
    Violation,
}

impl fmt::Display for ViolationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ViolationSeverity::Info => write!(f, "INFO"),
            ViolationSeverity::Warning => write!(f, "WARNING"),
            ViolationSeverity::Violation => write!(f, "VIOLATION"),
        }
    }
}

/// Network consistency error for ontology mismatches
#[derive(Debug, Clone)]
pub struct ConsistencyError {
    /// Local ontology hash
    pub local_hash: String,
    /// Network/expected ontology hash
    pub network_hash: String,
    /// Detailed error message
    pub message: String,
}

impl ConsistencyError {
    /// Create a new consistency error
    pub fn new(local_hash: String, network_hash: String, message: String) -> Self {
        Self {
            local_hash,
            network_hash,
            message,
        }
    }
}

impl fmt::Display for ConsistencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Ontology consistency error: local hash {} != network hash {}. {}",
            self.local_hash, self.network_hash, self.message
        )
    }
}

impl std::error::Error for ConsistencyError {}

/// Convert OntologyError to ValidationError for unified error handling
impl From<OntologyError> for ValidationError {
    fn from(error: OntologyError) -> Self {
        ValidationError::new(format!("Ontology error: {}", error))
    }
}

/// Convert ConsistencyError to ValidationError
impl From<ConsistencyError> for ValidationError {
    fn from(error: ConsistencyError) -> Self {
        ValidationError::new(format!("Consistency error: {}", error))
    }
}

/// Result of SHACL validation with detailed information
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the validation passed
    pub is_valid: bool,
    /// List of shape violations found
    pub violations: Vec<ShapeViolation>,
    /// Number of constraints that were checked
    pub constraints_checked: u32,
    /// Validation execution time in milliseconds
    pub execution_time_ms: Option<u64>,
    /// Additional metadata about the validation
    pub metadata: std::collections::HashMap<String, String>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn success(constraints_checked: u32) -> Self {
        Self {
            is_valid: true,
            violations: Vec::new(),
            constraints_checked,
            execution_time_ms: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a failed validation result with violations
    pub fn failure(violations: Vec<ShapeViolation>, constraints_checked: u32) -> Self {
        Self {
            is_valid: false,
            violations,
            constraints_checked,
            execution_time_ms: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add execution time to the result
    pub fn with_execution_time(mut self, execution_time_ms: u64) -> Self {
        self.execution_time_ms = Some(execution_time_ms);
        self
    }

    /// Add metadata to the result
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get the number of violations
    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }

    /// Check if there are any violations of a specific severity
    pub fn has_violations_of_severity(&self, severity: ViolationSeverity) -> bool {
        self.violations.iter().any(|v| v.severity == severity)
    }
}

impl fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_valid {
            write!(
                f,
                "Validation PASSED: {} constraints checked",
                self.constraints_checked
            )?;
        } else {
            write!(
                f,
                "Validation FAILED: {} violations found, {} constraints checked",
                self.violations.len(),
                self.constraints_checked
            )?;
        }

        if let Some(time) = self.execution_time_ms {
            write!(f, " ({}ms)", time)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_creation() {
        let error = ValidationError::new("Test error".to_string());
        assert_eq!(error.message, "Test error");
        assert!(!error.has_violations());
        assert!(error.transaction_id.is_none());
    }

    #[test]
    fn test_shape_violation_creation() {
        let violation = ShapeViolation::new(
            "TestShape".to_string(),
            ConstraintType::MinCount,
            "Required property missing".to_string(),
        )
        .with_property_path("ex:name".to_string())
        .with_value("".to_string());

        assert_eq!(violation.shape_id, "TestShape");
        assert_eq!(violation.property_path, Some("ex:name".to_string()));
        assert_eq!(violation.constraint_type, ConstraintType::MinCount);
    }

    #[test]
    fn test_validation_error_with_violations() {
        let violation = ShapeViolation::new(
            "TestShape".to_string(),
            ConstraintType::Datatype,
            "Invalid datatype".to_string(),
        );

        let error =
            ValidationError::with_violations("Multiple violations".to_string(), vec![violation]);

        assert!(error.has_violations());
        assert_eq!(error.shape_violations.len(), 1);
    }

    #[test]
    fn test_error_display() {
        let error = OntologyError::OntologyNotFound {
            path: "test.owl".to_string(),
        };
        assert_eq!(error.to_string(), "Ontology file not found: test.owl");
    }
}
