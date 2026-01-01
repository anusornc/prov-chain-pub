use regex::Regex;

/// Input validation functions
pub fn validate_uri(uri: &str) -> Result<(), String> {
    if uri.is_empty() {
        return Err("URI cannot be empty".to_string());
    }

    if uri.len() > 2048 {
        return Err("URI too long (max 2048 characters)".to_string());
    }

    // Basic URI validation
    let uri_regex =
        Regex::new(r"^https?://[^\s/$.?#].[^\s]*$|^[a-zA-Z][a-zA-Z0-9+.-]*:[^\s]*$").unwrap();
    if !uri_regex.is_match(uri) {
        return Err("Invalid URI format".to_string());
    }

    Ok(())
}

pub fn validate_literal(literal: &str) -> Result<(), String> {
    if literal.len() > 10000 {
        return Err("Literal too long (max 10000 characters)".to_string());
    }

    // Check for potential script injection
    let dangerous_patterns = [
        "<script",
        "javascript:",
        "data:",
        "vbscript:",
        "onload=",
        "onerror=",
    ];
    let literal_lower = literal.to_lowercase();
    for pattern in &dangerous_patterns {
        if literal_lower.contains(pattern) {
            return Err("Literal contains potentially dangerous content".to_string());
        }
    }

    Ok(())
}

pub fn validate_sparql_query(query: &str) -> Result<(), String> {
    if query.is_empty() {
        return Err("SPARQL query cannot be empty".to_string());
    }

    if query.len() > 50000 {
        return Err("SPARQL query too long (max 50000 characters)".to_string());
    }

    // Check for potentially dangerous operations
    let query_upper = query.to_uppercase();
    let dangerous_operations = ["DROP", "CLEAR", "DELETE", "INSERT", "LOAD", "CREATE"];
    for operation in &dangerous_operations {
        if query_upper.contains(operation) {
            return Err(format!("SPARQL operation '{}' is not allowed", operation));
        }
    }

    Ok(())
}
