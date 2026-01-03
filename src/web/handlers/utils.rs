use regex::Regex;
use chrono::{Utc, NaiveDate};

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

/// Validate string length
pub fn validate_string_length(s: &str, max_len: usize, field_name: &str) -> Result<(), String> {
    if s.len() > max_len {
        return Err(format!("{} exceeds maximum length of {}", field_name, max_len));
    }
    Ok(())
}

/// Validate block index is within reasonable bounds
pub fn validate_block_index(index: u64) -> Result<(), String> {
    const MAX_BLOCK_INDEX: u64 = 1_000_000_000;
    if index > MAX_BLOCK_INDEX {
        return Err(format!("Block index {} exceeds maximum {}", index, MAX_BLOCK_INDEX));
    }
    Ok(())
}

/// Validate product ID format
pub fn validate_product_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 256 {
        return Err("Product ID must be 1-256 characters".to_string());
    }

    // Only allow alphanumeric, hyphens, underscores
    let id_regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
    if !id_regex.is_match(id) {
        return Err("Product ID contains invalid characters (only a-zA-Z0-9_- allowed)".to_string());
    }

    Ok(())
}

// Re-export the robust SPARQL validator
pub use crate::web::sparql_validator::validate_sparql_query;

/// Validate and parse a date string in YYYY-MM-DD format
/// Returns the parsed date or an error message
pub fn validate_date_format(date_str: &str) -> Result<NaiveDate, String> {
    if date_str.is_empty() {
        return Err("Date cannot be empty".to_string());
    }

    // Check length
    if date_str.len() > 10 {
        return Err("Date format invalid (expected YYYY-MM-DD)".to_string());
    }

    // Check for dangerous patterns that could indicate injection attempts
    let dangerous_patterns = [
        "'", "\"", ";", "--", "/*", "*/", "UNION", "SELECT", "DROP",
        "OR", "AND", "<script", "javascript:", "xp_", "declare",
    ];
    let date_upper = date_str.to_uppercase();
    for pattern in dangerous_patterns {
        if date_upper.contains(pattern) {
            return Err("Date contains invalid characters".to_string());
        }
    }

    // Parse as YYYY-MM-DD
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| "Date format invalid (expected YYYY-MM-DD)".to_string())
}

/// Validate date range is reasonable
pub fn validate_date_range(start_date: &str, end_date: &str) -> Result<(NaiveDate, NaiveDate), String> {
    let start = validate_date_format(start_date)
        .map_err(|e| format!("Invalid start_date: {}", e))?;

    let end = validate_date_format(end_date)
        .map_err(|e| format!("Invalid end_date: {}", e))?;

    // Ensure start <= end
    if start > end {
        return Err("start_date must be before or equal to end_date".to_string());
    }

    // Reasonable range limits
    let now = Utc::now().date_naive();
    let max_future = now + chrono::Duration::days(365); // Max 1 year in future
    let max_past = now - chrono::Duration::days(365 * 5); // Max 5 years in past

    if start > max_future {
        return Err("start_date too far in the future (max 1 year)".to_string());
    }

    if end < max_past {
        return Err("end_date too far in the past (max 5 years)".to_string());
    }

    // Check range duration is reasonable
    let duration = end.signed_duration_since(start);
    if duration.num_days() > 365 * 2 {
        return Err("Date range too large (max 2 years)".to_string());
    }

    Ok((start, end))
}

/// Sanitize date string by extracting only YYYY-MM-DD portion
pub fn sanitize_date_string(date_str: &str) -> String {
    date_str
        .split('T')
        .next()
        .unwrap_or("")
        .to_string()
}
