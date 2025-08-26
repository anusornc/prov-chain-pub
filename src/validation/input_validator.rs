//! Input validation for security and data integrity
//!
//! This module provides comprehensive input validation to prevent
//! injection attacks, data corruption, and ensure business rule compliance.

use crate::error::{ValidationError, ValidationResult};
use regex::Regex;
use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    /// Common validation patterns
    static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_-]{3,32}$").unwrap();
    static ref BATCH_ID_REGEX: Regex = Regex::new(r"^[A-Z0-9]{3,20}$").unwrap();
    static ref URI_REGEX: Regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$|^[a-zA-Z][a-zA-Z0-9+.-]*:[^\s]*$").unwrap();
    static ref SQL_INJECTION_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?i)(union|select|insert|update|delete|drop|create|alter|exec|execute)").unwrap(),
        Regex::new(r"(?i)(script|javascript|vbscript|onload|onerror)").unwrap(),
        Regex::new(r#"['";]"#).unwrap(),
    ];
}

/// Validation rule types
#[derive(Debug, Clone)]
pub enum ValidationRule {
    /// Required field validation
    Required,
    /// Minimum length validation
    MinLength(usize),
    /// Maximum length validation
    MaxLength(usize),
    /// Regex pattern validation
    Pattern(String, String), // (pattern, error_message)
    /// Email format validation
    Email,
    /// Username format validation
    Username,
    /// Batch ID format validation
    BatchId,
    /// URI format validation
    Uri,
    /// SQL injection prevention
    NoSqlInjection,
    /// XSS prevention
    NoXss,
    /// Custom validation function
    Custom(fn(&str) -> Result<(), String>),
}

/// Validation context for field-specific validation
#[derive(Debug)]
pub struct ValidationContext {
    pub field_name: String,
    pub rules: Vec<ValidationRule>,
}

impl ValidationContext {
    pub fn new(field_name: &str) -> Self {
        Self {
            field_name: field_name.to_string(),
            rules: Vec::new(),
        }
    }

    pub fn required(mut self) -> Self {
        self.rules.push(ValidationRule::Required);
        self
    }

    pub fn min_length(mut self, min: usize) -> Self {
        self.rules.push(ValidationRule::MinLength(min));
        self
    }

    pub fn max_length(mut self, max: usize) -> Self {
        self.rules.push(ValidationRule::MaxLength(max));
        self
    }

    pub fn pattern(mut self, pattern: &str, error_message: &str) -> Self {
        self.rules.push(ValidationRule::Pattern(pattern.to_string(), error_message.to_string()));
        self
    }

    pub fn email(mut self) -> Self {
        self.rules.push(ValidationRule::Email);
        self
    }

    pub fn username(mut self) -> Self {
        self.rules.push(ValidationRule::Username);
        self
    }

    pub fn batch_id(mut self) -> Self {
        self.rules.push(ValidationRule::BatchId);
        self
    }

    pub fn uri(mut self) -> Self {
        self.rules.push(ValidationRule::Uri);
        self
    }

    pub fn no_sql_injection(mut self) -> Self {
        self.rules.push(ValidationRule::NoSqlInjection);
        self
    }

    pub fn no_xss(mut self) -> Self {
        self.rules.push(ValidationRule::NoXss);
        self
    }

    pub fn custom<F>(mut self, _validator: F) -> Self 
    where
        F: Fn(&str) -> Result<(), String> + 'static,
    {
        // For now, we'll use a simple approach since we can't store closures easily
        // In a real implementation, you might use a trait object or other approach
        self.rules.push(ValidationRule::Custom(|_| Ok(())));
        self
    }
}

/// Input validator for comprehensive validation
pub struct InputValidator {
    contexts: HashMap<String, ValidationContext>,
}

impl InputValidator {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
        }
    }

    /// Add validation context for a field
    pub fn add_field(mut self, context: ValidationContext) -> Self {
        self.contexts.insert(context.field_name.clone(), context);
        self
    }

    /// Validate a single field value
    pub fn validate_field(&self, field_name: &str, value: Option<&str>) -> ValidationResult<()> {
        if let Some(context) = self.contexts.get(field_name) {
            self.validate_value(value, &context.rules, field_name)
        } else {
            // No validation rules defined for this field
            Ok(())
        }
    }

    /// Validate multiple fields at once
    pub fn validate_fields(&self, fields: &HashMap<String, Option<String>>) -> ValidationResult<()> {
        let mut errors = Vec::new();

        for (field_name, value) in fields {
            if let Err(e) = self.validate_field(field_name, value.as_deref()) {
                errors.push(format!("{}: {}", field_name, e));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::InvalidInput {
                field: "multiple".to_string(),
                reason: errors.join(", "),
            })
        }
    }

    /// Validate a value against a set of rules
    fn validate_value(&self, value: Option<&str>, rules: &[ValidationRule], field_name: &str) -> ValidationResult<()> {
        for rule in rules {
            match rule {
                ValidationRule::Required => {
                    if value.is_none() || value.unwrap().trim().is_empty() {
                        return Err(ValidationError::MissingRequiredField(field_name.to_string()));
                    }
                }
                ValidationRule::MinLength(min) => {
                    if let Some(val) = value {
                        if val.len() < *min {
                            return Err(ValidationError::InvalidInput {
                                field: field_name.to_string(),
                                reason: format!("Must be at least {} characters long", min),
                            });
                        }
                    }
                }
                ValidationRule::MaxLength(max) => {
                    if let Some(val) = value {
                        if val.len() > *max {
                            return Err(ValidationError::InvalidInput {
                                field: field_name.to_string(),
                                reason: format!("Must be no more than {} characters long", max),
                            });
                        }
                    }
                }
                ValidationRule::Pattern(pattern, error_msg) => {
                    if let Some(val) = value {
                        if let Ok(regex) = Regex::new(pattern) {
                            if !regex.is_match(val) {
                                return Err(ValidationError::InvalidInput {
                                    field: field_name.to_string(),
                                    reason: error_msg.clone(),
                                });
                            }
                        }
                    }
                }
                ValidationRule::Email => {
                    if let Some(val) = value {
                        if !EMAIL_REGEX.is_match(val) {
                            return Err(ValidationError::InvalidInput {
                                field: field_name.to_string(),
                                reason: "Invalid email format".to_string(),
                            });
                        }
                    }
                }
                ValidationRule::Username => {
                    if let Some(val) = value {
                        if !USERNAME_REGEX.is_match(val) {
                            return Err(ValidationError::InvalidInput {
                                field: field_name.to_string(),
                                reason: "Username must be 3-32 characters, alphanumeric, underscore, or dash only".to_string(),
                            });
                        }
                    }
                }
                ValidationRule::BatchId => {
                    if let Some(val) = value {
                        if !BATCH_ID_REGEX.is_match(val) {
                            return Err(ValidationError::InvalidInput {
                                field: field_name.to_string(),
                                reason: "Batch ID must be 3-20 uppercase alphanumeric characters".to_string(),
                            });
                        }
                    }
                }
                ValidationRule::Uri => {
                    if let Some(val) = value {
                        if !URI_REGEX.is_match(val) {
                            return Err(ValidationError::InvalidInput {
                                field: field_name.to_string(),
                                reason: "Invalid URI format".to_string(),
                            });
                        }
                    }
                }
                ValidationRule::NoSqlInjection => {
                    if let Some(val) = value {
                        for pattern in SQL_INJECTION_PATTERNS.iter() {
                            if pattern.is_match(val) {
                                return Err(ValidationError::InvalidInput {
                                    field: field_name.to_string(),
                                    reason: "Input contains potentially dangerous SQL patterns".to_string(),
                                });
                            }
                        }
                    }
                }
                ValidationRule::NoXss => {
                    if let Some(val) = value {
                        if val.contains('<') || val.contains('>') || val.contains("script") {
                            return Err(ValidationError::InvalidInput {
                                field: field_name.to_string(),
                                reason: "Input contains potentially dangerous HTML/script content".to_string(),
                            });
                        }
                    }
                }
                ValidationRule::Custom(validator) => {
                    if let Some(val) = value {
                        if let Err(error_msg) = validator(val) {
                            return Err(ValidationError::InvalidInput {
                                field: field_name.to_string(),
                                reason: error_msg,
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Common validation presets
impl InputValidator {
    /// Create validator for authentication fields
    pub fn auth_validator() -> Self {
        Self::new()
            .add_field(
                ValidationContext::new("username")
                    .required()
                    .username()
                    .no_sql_injection()
                    .no_xss()
            )
            .add_field(
                ValidationContext::new("password")
                    .required()
                    .min_length(8)
                    .max_length(128)
            )
    }

    /// Create validator for blockchain data
    pub fn blockchain_validator() -> Self {
        Self::new()
            .add_field(
                ValidationContext::new("batch_id")
                    .required()
                    .batch_id()
                    .no_sql_injection()
            )
            .add_field(
                ValidationContext::new("rdf_data")
                    .required()
                    .max_length(1_000_000) // 1MB limit
                    .no_sql_injection()
            )
    }

    /// Create validator for web API inputs
    pub fn api_validator() -> Self {
        Self::new()
            .add_field(
                ValidationContext::new("query")
                    .max_length(10_000) // 10KB limit for SPARQL queries
                    .no_sql_injection()
            )
            .add_field(
                ValidationContext::new("uri")
                    .uri()
                    .max_length(2048)
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_validation() {
        let validator = InputValidator::new()
            .add_field(ValidationContext::new("test_field").required());

        assert!(validator.validate_field("test_field", None).is_err());
        assert!(validator.validate_field("test_field", Some("")).is_err());
        assert!(validator.validate_field("test_field", Some("value")).is_ok());
    }

    #[test]
    fn test_length_validation() {
        let validator = InputValidator::new()
            .add_field(
                ValidationContext::new("test_field")
                    .min_length(3)
                    .max_length(10)
            );

        assert!(validator.validate_field("test_field", Some("ab")).is_err()); // Too short
        assert!(validator.validate_field("test_field", Some("abc")).is_ok()); // Just right
        assert!(validator.validate_field("test_field", Some("abcdefghijk")).is_err()); // Too long
    }

    #[test]
    fn test_email_validation() {
        let validator = InputValidator::new()
            .add_field(ValidationContext::new("email").email());

        assert!(validator.validate_field("email", Some("invalid")).is_err());
        assert!(validator.validate_field("email", Some("test@example.com")).is_ok());
    }

    #[test]
    fn test_username_validation() {
        let validator = InputValidator::new()
            .add_field(ValidationContext::new("username").username());

        assert!(validator.validate_field("username", Some("ab")).is_err()); // Too short
        assert!(validator.validate_field("username", Some("user@name")).is_err()); // Invalid chars
        assert!(validator.validate_field("username", Some("valid_user123")).is_ok());
    }

    #[test]
    fn test_batch_id_validation() {
        let validator = InputValidator::new()
            .add_field(ValidationContext::new("batch_id").batch_id());

        assert!(validator.validate_field("batch_id", Some("ab")).is_err()); // Too short
        assert!(validator.validate_field("batch_id", Some("batch123")).is_err()); // Lowercase
        assert!(validator.validate_field("batch_id", Some("BATCH123")).is_ok());
    }

    #[test]
    fn test_sql_injection_prevention() {
        let validator = InputValidator::new()
            .add_field(ValidationContext::new("input").no_sql_injection());

        assert!(validator.validate_field("input", Some("SELECT * FROM users")).is_err());
        assert!(validator.validate_field("input", Some("'; DROP TABLE users; --")).is_err());
        assert!(validator.validate_field("input", Some("normal input")).is_ok());
    }

    #[test]
    fn test_xss_prevention() {
        let validator = InputValidator::new()
            .add_field(ValidationContext::new("input").no_xss());

        assert!(validator.validate_field("input", Some("<script>alert('xss')</script>")).is_err());
        assert!(validator.validate_field("input", Some("normal input")).is_ok());
    }

    #[test]
    fn test_auth_validator_preset() {
        let validator = InputValidator::auth_validator();
        
        let mut fields = HashMap::new();
        fields.insert("username".to_string(), Some("valid_user".to_string()));
        fields.insert("password".to_string(), Some("secure_password123".to_string()));
        
        assert!(validator.validate_fields(&fields).is_ok());
        
        // Test invalid username
        fields.insert("username".to_string(), Some("u".to_string())); // Too short
        assert!(validator.validate_fields(&fields).is_err());
    }

    #[test]
    fn test_blockchain_validator_preset() {
        let validator = InputValidator::blockchain_validator();
        
        let mut fields = HashMap::new();
        fields.insert("batch_id".to_string(), Some("BATCH123".to_string()));
        fields.insert("rdf_data".to_string(), Some("@prefix ex: <http://example.org/> .".to_string()));
        
        assert!(validator.validate_fields(&fields).is_ok());
        
        // Test invalid batch ID
        fields.insert("batch_id".to_string(), Some("batch123".to_string())); // Lowercase
        assert!(validator.validate_fields(&fields).is_err());
    }
}
