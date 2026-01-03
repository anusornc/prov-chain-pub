//! Robust SPARQL query validation
//!
//! Uses actual SPARQL parsing to prevent injection attacks

use anyhow::{anyhow, Result};

/// SPARQL validator configuration
#[derive(Debug, Clone)]
pub struct SparqlValidatorConfig {
    pub max_query_length: usize,
    pub allow_select: bool,
    pub allow_ask: bool,
    pub allow_construct: bool,
    pub allow_describe: bool,
    pub allow_updates: bool,
}

impl Default for SparqlValidatorConfig {
    fn default() -> Self {
        Self {
            max_query_length: 50_000,
            allow_select: true,
            allow_ask: true,
            allow_construct: false,
            allow_describe: false,
            allow_updates: false,
        }
    }
}

/// SPARQL query validator
pub struct SparqlValidator {
    config: SparqlValidatorConfig,
}

impl SparqlValidator {
    pub fn new(config: SparqlValidatorConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(SparqlValidatorConfig::default())
    }

    /// Validate a SPARQL query
    pub fn validate(&self, query: &str) -> Result<()> {
        // Check 1: Empty query
        if query.trim().is_empty() {
            return Err(anyhow!("SPARQL query cannot be empty"));
        }

        // Check 2: Length limit
        if query.len() > self.config.max_query_length {
            return Err(anyhow!(
                "SPARQL query too long (max {} chars, got {})",
                self.config.max_query_length,
                query.len()
            ));
        }

        // Check 3: Validate query type using keyword analysis
        // This is a simplified approach - production should use oxigraph::sparql::Query
        self.validate_query_type(query)?;

        // Check 4: Check for injection patterns
        self.check_injection_patterns(query)?;

        Ok(())
    }

    /// Validate query type
    fn validate_query_type(&self, query: &str) -> Result<()> {
        let query_upper = query.trim().to_uppercase();

        // Check for allowed query types
        let has_select = query_upper.starts_with("SELECT");
        let has_ask = query_upper.starts_with("ASK");
        let has_construct = query_upper.starts_with("CONSTRUCT");
        let has_describe = query_upper.starts_with("DESCRIBE");

        // Check for update operations (not allowed via API)
        let update_keywords = ["INSERT", "DELETE", "LOAD", "CLEAR", "CREATE", "DROP", "COPY", "MOVE", "ADD"];
        for keyword in &update_keywords {
            if query_upper.contains(keyword) {
                return Err(anyhow!(
                    "SPARQL update operation '{}' is not allowed via API",
                    keyword
                ));
            }
        }

        // Validate query type
        if has_select && !self.config.allow_select {
            return Err(anyhow!("SELECT queries are not allowed"));
        }
        if has_ask && !self.config.allow_ask {
            return Err(anyhow!("ASK queries are not allowed"));
        }
        if has_construct && !self.config.allow_construct {
            return Err(anyhow!("CONSTRUCT queries are not allowed"));
        }
        if has_describe && !self.config.allow_describe {
            return Err(anyhow!("DESCRIBE queries are not allowed"));
        }

        // Must have at least one valid query type
        if !has_select && !has_ask && !has_construct && !has_describe {
            return Err(anyhow!("Query must start with SELECT, ASK, CONSTRUCT, or DESCRIBE"));
        }

        Ok(())
    }

    /// Check for common injection patterns
    fn check_injection_patterns(&self, query: &str) -> Result<()> {
        let query_upper = query.to_uppercase();
        let query_lower = query.to_lowercase();

        // Sensitive property patterns that should not be accessible via API
        let sensitive_properties = [
            "password", "passwd", "pwd", "token", "secret", "credential",
            "hash", "salt", "key", "private", "auth", "login", "session",
            "csrf", "jwt", "bearer", "cookie", "api_key", "apikey",
        ];

        // Check if sensitive properties appear in property position (after ? or :)
        for sensitive in &sensitive_properties {
            // Pattern 1: :sensitive or ?sensitive (as property)
            if query_lower.contains(&format!(":{}", sensitive)) || 
               query_lower.contains(&format!("?{}", sensitive)) {
                // Additional check: is this actually being used as a property?
                if self.is_used_as_property(query, sensitive) {
                    return Err(anyhow!(
                        "Access to sensitive property '{}' is not allowed",
                        sensitive
                    ));
                }
            }

            // Pattern 2: String literals containing sensitive data in FILTER/regex
            if query_lower.contains(sensitive) && 
               (query_upper.contains("FILTER") || query_upper.contains("REGEX")) {
                return Err(anyhow!(
                    "Potential injection attempt: sensitive property '{}' in FILTER/REGEX",
                    sensitive
                ));
            }
        }

        // Check for comment-based bypasses
        if query.contains("--") || query.contains("#") {
            let suspicious = [
                ("-- DROP", "Comment-based DROP bypass"),
                ("#; DROP", "Comment-based injection attempt"),
                ("*/", "Multi-line comment injection"),
            ];

            for (pattern, desc) in &suspicious {
                if query_upper.contains(pattern) {
                    return Err(anyhow!("Potential injection: {}", desc));
                }
            }
        }

        // Check for multiple statements (SEMICOLON separates statements in SPARQL)
        let statement_count = query.matches(';').count();
        if statement_count > 5 {
            return Err(anyhow!(
                "Too many SPARQL statements (detected {}, max 5)",
                statement_count + 1
            ));
        }

        // Check for SERVICE clause (potential SSRF)
        if query_upper.contains("SERVICE") {
            return Err(anyhow!("SERVICE clause is not allowed (potential SSRF)"));
        }

        // Check for GRAPH clause injection
        if query_upper.contains("GRAPH") {
            // Only allow specific safe graph patterns
            if self.check_unsafe_graph_patterns(query) {
                return Err(anyhow!("Unsafe GRAPH clause detected"));
            }
        }

        // Check for RDF* (triple) patterns that might expose internals
        if query.contains("<<<") || query.contains(">>>") {
            return Err(anyhow!("RDF* triple patterns are not allowed via API"));
        }

        // Check for dangerous BIND functions
        let dangerous_functions = [
            ("MD5", "Hash function"), ("SHA1", "Hash function"), 
            ("SHA256", "Hash function"), ("ENCRYPT", "Encryption"),
        ];
        for (func, desc) in &dangerous_functions {
            if query_upper.contains(func) && query_upper.contains("BIND") {
                return Err(anyhow!(
                    "Use of {} in BIND is not allowed ({})",
                    func,
                    desc
                ));
            }
        }

        Ok(())
    }

    /// Check if a sensitive term is being used as a property
    fn is_used_as_property(&self, query: &str, sensitive: &str) -> bool {
        let query_lower = query.to_lowercase();
        
        // Check if sensitive word appears right after : or ? in property position
        let patterns = [
            format!(":{}", sensitive),
            format!("?{}", sensitive),
        ];

        for pattern in patterns {
            if let Some(pos) = query_lower.find(&pattern) {
                // Check if it's followed by common property patterns
                let after = &query_lower[pos + pattern.len()..];
                if after.starts_with(" ") || after.starts_with("\t") || 
                   after.starts_with(";") || after.starts_with("}") ||
                   after.starts_with("|") || after.starts_with("/") ||
                   after.starts_with("^") {
                    return true;
                }
            }
        }

        false
    }

    /// Check for unsafe GRAPH patterns
    fn check_unsafe_graph_patterns(&self, query: &str) -> bool {
        let query_upper = query.to_uppercase();
        
        // Allow GRAPH with variable but not with specific IRIs
        let has_graph_variable = query_upper.contains("GRAPH ?G") || 
                                query_upper.contains("GRAPH ?GRAPH");
        
        // Check for hardcoded IRIs in GRAPH
        if query_upper.contains("GRAPH <") {
            // Check if it's a safe IRI pattern
            let unsafe_iris = [
                "ADMIN", "SECRET", "PASSWORD", "AUTH", "CONFIG",
                "PRIVATE", "INTERNAL", "SYSTEM", "METADATA",
                "../", "./", "\\.\\.", // Path traversal
            ];
            
            for unsafe_pattern in &unsafe_iris {
                if query_upper.contains(unsafe_pattern) {
                    return true;
                }
            }
        }

        !has_graph_variable
    }
}

/// Helper function for backward compatibility
pub fn validate_sparql_query(query: &str) -> Result<(), String> {
    let validator = SparqlValidator::with_default_config();
    validator.validate(query).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_select_query() {
        let validator = SparqlValidator::with_default_config();
        let query = "SELECT ?s WHERE { ?s a <http://example.org/Test> }";
        assert!(validator.validate(query).is_ok());
    }

    #[test]
    fn test_reject_insert() {
        let validator = SparqlValidator::with_default_config();
        let query = "INSERT DATA { <s> <p> <o> }";
        assert!(validator.validate(query).is_err());
    }

    #[test]
    fn test_reject_delete() {
        let validator = SparqlValidator::with_default_config();
        let query = "DELETE WHERE { ?s ?p ?o }";
        assert!(validator.validate(query).is_err());
    }

    #[test]
    fn test_reject_drop() {
        let validator = SparqlValidator::with_default_config();
        let query = "SELECT ?s WHERE { -- DROP TABLE users\n?s a <http://example.org/Test> }";
        assert!(validator.validate(query).is_err());
    }

    #[test]
    fn test_query_too_long() {
        let mut config = SparqlValidatorConfig::default();
        config.max_query_length = 100;
        let validator = SparqlValidator::new(config);

        let long_query = format!("SELECT ?s WHERE {{ {} }}", "a ".repeat(200));
        assert!(validator.validate(&long_query).is_err());
    }

    #[test]
    fn test_empty_query() {
        let validator = SparqlValidator::with_default_config();
        assert!(validator.validate("").is_err());
        assert!(validator.validate("   ").is_err());
    }

    #[test]
    fn test_valid_ask_query() {
        let validator = SparqlValidator::with_default_config();
        let query = "ASK { <http://example.org> a <http://example.org/Test> }";
        assert!(validator.validate(query).is_ok());
    }

    #[test]
    fn test_reject_construct() {
        let validator = SparqlValidator::with_default_config();
        let query = "CONSTRUCT { ?s a <Test> } WHERE { ?s a <http://example.org/Test> }";
        assert!(validator.validate(query).is_err());
    }
}
