//! Security middleware and utilities for ProvChain-Org web API

use crate::web::models::ApiError;
use axum::{
    extract::{Request, State},
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: usize,
    /// Time window for rate limiting
    pub window: Duration,
    /// Whether to apply rate limiting per IP or globally
    pub per_ip: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,        // 100 requests per window
            window: Duration::from_secs(60), // 1 minute window
            per_ip: true,
        }
    }
}

/// Rate limiter for authentication endpoints (more restrictive)
impl RateLimitConfig {
    pub fn auth_strict() -> Self {
        Self {
            max_requests: 5,          // 5 login attempts per window
            window: Duration::from_secs(300), // 5 minute window
            per_ip: true,
        }
    }

    pub fn api_standard() -> Self {
        Self {
            max_requests: 1000,       // 1000 requests per window
            window: Duration::from_secs(60), // 1 minute window
            per_ip: true,
        }
    }
}

/// Rate limiter state
#[derive(Debug)]
pub struct RateLimiter {
    /// Map of client IP to request history
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    /// Configuration
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Check if a request should be rate limited
    pub async fn check_rate_limit(&self, client_ip: &str) -> Result<(), RateLimitError> {
        let mut requests = self.requests.write().await;
        let now = Instant::now();

        // Get or create request history for this client
        let request_history = requests.entry(client_ip.to_string()).or_insert_with(Vec::new);

        // Remove old requests outside the time window
        request_history.retain(|&timestamp| now.duration_since(timestamp) < self.config.window);

        // Check if we've exceeded the rate limit
        if request_history.len() >= self.config.max_requests {
            return Err(RateLimitError::TooManyRequests {
                retry_after: self.config.window.as_secs(),
            });
        }

        // Add current request to history
        request_history.push(now);

        Ok(())
    }

    /// Clean up expired entries (should be called periodically)
    pub async fn cleanup_expired(&self) {
        let mut requests = self.requests.write().await;
        let now = Instant::now();

        requests.retain(|_, history| {
            history.retain(|&timestamp| now.duration_since(timestamp) < self.config.window);
            !history.is_empty()
        });
    }
}

/// Rate limiting errors
#[derive(Debug)]
pub enum RateLimitError {
    TooManyRequests { retry_after: u64 },
}

impl RateLimitError {
    pub fn into_response(self) -> (StatusCode, ApiError) {
        match self {
            RateLimitError::TooManyRequests { retry_after } => (
                StatusCode::TOO_MANY_REQUESTS,
                ApiError {
                    error: "rate_limit_exceeded".to_string(),
                    message: format!("Rate limit exceeded. Try again in {} seconds.", retry_after),
                    timestamp: chrono::Utc::now(),
                },
            ),
        }
    }
}

/// Security middleware for rate limiting
pub async fn rate_limit_middleware(
    State(rate_limiter): State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, ApiError)> {
    // Extract client IP from headers or connection info
    let client_ip = extract_client_ip(&request);

    // Check rate limit
    if let Err(rate_limit_error) = rate_limiter.check_rate_limit(&client_ip).await {
        return Err(rate_limit_error.into_response());
    }

    // Continue with the request
    Ok(next.run(request).await)
}

/// Extract client IP address from request
fn extract_client_ip(request: &Request) -> String {
    // Try to get IP from common headers first (for reverse proxy setups)
    let headers = [
        "x-forwarded-for",
        "x-real-ip",
        "cf-connecting-ip",
        "x-client-ip",
    ];

    for header_name in headers.iter() {
        if let Some(header_value) = request.headers().get(header_name) {
            if let Ok(ip_str) = header_value.to_str() {
                // X-Forwarded-For can contain multiple IPs, take the first one
                let ip = ip_str.split(',').next().unwrap_or("").trim();
                if !ip.is_empty() && ip != "unknown" {
                    return ip.to_string();
                }
            }
        }
    }

    // Fall back to connection info (this might not be available in all setups)
    "unknown".to_string()
}

/// Security headers middleware
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, ApiError)> {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Content Security Policy
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https:; \
             font-src 'self'; \
             connect-src 'self' ws: wss:; \
             frame-ancestors 'none'; \
             base-uri 'self'; \
             form-action 'self'"
        ),
    );

    // X-Content-Type-Options
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );

    // X-Frame-Options
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("DENY"),
    );

    // X-XSS-Protection
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer Policy
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Strict-Transport-Security (only in production with HTTPS)
    if !cfg!(debug_assertions) {
        headers.insert(
            "Strict-Transport-Security",
            HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
        );
    }

    // Permissions Policy
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static(
            "camera=(), microphone=(), geolocation=(), payment=(), usb=(), magnetometer=(), gyroscope=()"
        ),
    );

    Ok(response)
}

/// Input validation utilities
pub mod validation {
    use crate::error::WebError;

    /// Sanitize and validate username input
    pub fn sanitize_username(input: &str) -> Result<String, WebError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err(WebError::InvalidRequest("Username cannot be empty".to_string()));
        }

        if trimmed.len() > 100 {
            return Err(WebError::InvalidRequest("Username too long".to_string()));
        }

        // Remove any potentially dangerous characters
        let sanitized = trimmed
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
            .collect::<String>();

        if sanitized.is_empty() {
            return Err(WebError::InvalidRequest("Invalid username format".to_string()));
        }

        Ok(sanitized)
    }

    /// Validate password strength (basic checks, more comprehensive validation should be done at the auth layer)
    pub fn validate_password_strength(password: &str) -> Result<(), WebError> {
        if password.len() < 1 {
            return Err(WebError::InvalidRequest("Password cannot be empty".to_string()));
        }

        if password.len() > 1000 {
            return Err(WebError::InvalidRequest("Password too long".to_string()));
        }

        Ok(())
    }

    /// Sanitize string input to prevent injection attacks
    pub fn sanitize_string(input: &str) -> String {
        input
            .chars()
            .filter(|c| {
                // Allow printable ASCII characters and common Unicode characters
                *c as u32 >= 32 && *c as u32 <= 126
                || *c as u32 >= 160 && *c as u32 <= 0x10FFFF
            })
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// Validate SPARQL query to prevent injection
    pub fn validate_sparql_query(query: &str) -> Result<(), WebError> {
        if query.len() > 100_000 {
            return Err(WebError::InvalidRequest("Query too long".to_string()));
        }

        // Basic check for potentially dangerous keywords
        let dangerous_keywords = [
            "DELETE", "INSERT", "LOAD", "CLEAR", "CREATE", "DROP", "COPY", "MOVE", "ADD"
        ];

        let uppercase_query = query.to_uppercase();
        for keyword in dangerous_keywords.iter() {
            if uppercase_query.contains(keyword) {
                return Err(WebError::InvalidRequest(
                    format!("Query contains potentially dangerous keyword: {}", keyword)
                ));
            }
        }

        Ok(())
    }
}

/// Periodic cleanup task for rate limiter
pub async fn cleanup_task(rate_limiter: Arc<RateLimiter>) {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // Run every 5 minutes

    loop {
        interval.tick().await;
        rate_limiter.cleanup_expired().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_rate_limiting() {
        let config = RateLimitConfig {
            max_requests: 2,
            window: Duration::from_secs(1),
            per_ip: true,
        };

        let rate_limiter = RateLimiter::new(config);
        let client_ip = "192.168.1.1";

        // First request should succeed
        assert!(rate_limiter.check_rate_limit(client_ip).await.is_ok());

        // Second request should succeed
        assert!(rate_limiter.check_rate_limit(client_ip).await.is_ok());

        // Third request should fail
        assert!(rate_limiter.check_rate_limit(client_ip).await.is_err());

        // Wait for window to expire
        sleep(Duration::from_secs(1)).await;

        // Request should succeed again
        assert!(rate_limiter.check_rate_limit(client_ip).await.is_ok());
    }

    #[test]
    fn test_sanitize_username() {
        assert_eq!(sanitize_username("test_user").unwrap(), "test_user");
        assert_eq!(sanitize_username("  test-user  ").unwrap(), "test-user");
        assert_eq!(sanitize_username("test.user").unwrap(), "test.user");
        assert!(sanitize_username("").is_err());
        assert!(sanitize_username("test user").unwrap().contains("user"));
    }

    #[test]
    fn test_validate_sparql_query() {
        assert!(validate_sparql_query("SELECT ?s ?p ?o WHERE { ?s ?p ?o }").is_ok());
        assert!(validate_sparql_query("").is_ok());
        assert!(validate_sparql_query("DELETE WHERE { ?s ?p ?o }").is_err());
        assert!(validate_sparql_query(&"a".repeat(100_001)).is_err());
    }
}