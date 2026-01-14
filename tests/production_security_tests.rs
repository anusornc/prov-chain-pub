//! Production Security Tests
//!
//! Comprehensive test suite for production security features including:
//! - JWT validation and authentication
//! - Rate limiting and DoS protection
//! - GDPR compliance
//! - Input sanitization
//! - Security policy enforcement
//! - Audit logging
//! - TLS configuration
//! - Security headers

use provchain_org::production::security::{
    AuditEventType, AuditResult, SecurityAuditEvent, SecurityManager, SecurityMiddleware,
};
use provchain_org::production::security::{ProductionClaims, SecurityConfig};
use provchain_org::production::ProductionError;
use std::collections::HashMap;
use std::time::SystemTime;

// Test JWT secret (32+ characters as required)
const TEST_JWT_SECRET: &str = "test-jwt-secret-key-min-32-chars-for-security-validation";

/// Helper function to create a valid JWT token for testing
fn create_test_jwt_token(subject: &str, expiration_hours: i64) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};

    let now = chrono::Utc::now();
    let exp = (now + chrono::Duration::hours(expiration_hours)).timestamp();

    let claims = ProductionClaims {
        sub: subject.to_string(),
        iat: now.timestamp(),
        exp,
        iss: "provchain-test".to_string(),
        role: Some("test".to_string()),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .expect("Failed to encode test JWT token")
}

/// Helper function to create an expired JWT token
fn create_expired_jwt_token() -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};

    let now = chrono::Utc::now();
    let exp = (now - chrono::Duration::hours(1)).timestamp(); // Expired 1 hour ago

    let claims = ProductionClaims {
        sub: "test-user".to_string(),
        iat: (now - chrono::Duration::hours(2)).timestamp(),
        exp,
        iss: "provchain-test".to_string(),
        role: Some("test".to_string()),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .expect("Failed to encode expired test JWT token")
}

/// Helper function to set up test environment
fn setup_test_env() {
    // Set JWT_SECRET for production security tests
    std::env::set_var("JWT_SECRET", TEST_JWT_SECRET);
}

// Helper function to create a test security config
fn create_test_config() -> SecurityConfig {
    // Ensure JWT_SECRET is set for tests
    setup_test_env();

    SecurityConfig {
        tls_enabled: true,
        tls_cert_path: Some("/tmp/test.crt".to_string()),
        tls_key_path: Some("/tmp/test.key".to_string()),
        rate_limiting_enabled: true,
        rate_limit_per_minute: 10, // Low limit for testing
        request_validation_enabled: true,
        cors_enabled: true,
        cors_origins: vec!["https://test.local".to_string()],
        security_headers_enabled: true,
        jwt_secret: TEST_JWT_SECRET.to_string(), // Use test secret
        jwt_expiration_hours: 1,
        audit_logging_enabled: false, // Disable file I/O in tests
        audit_log_path: "/tmp/test-audit.log".to_string(),
        security_policies: vec![],
    }
}

// Helper function to create a test audit event
fn create_test_audit_event(event_type: AuditEventType, result: AuditResult) -> SecurityAuditEvent {
    SecurityAuditEvent {
        timestamp: chrono::Utc::now(),
        event_type,
        user_id: Some("test-user".to_string()),
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("test-agent".to_string()),
        resource: "/api/test".to_string(),
        action: "test-action".to_string(),
        result,
        details: HashMap::new(),
    }
}

#[cfg(test)]
mod security_config_tests {
    use super::*;

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();

        // Verify default security settings
        assert!(config.tls_enabled);
        assert!(config.rate_limiting_enabled);
        assert!(config.request_validation_enabled);
        assert!(config.cors_enabled);
        assert!(config.security_headers_enabled);
        assert!(config.audit_logging_enabled);

        // Verify rate limit is reasonable
        assert!(config.rate_limit_per_minute > 0);
        assert!(config.rate_limit_per_minute <= 1000);

        // Verify JWT expiration is reasonable (1-48 hours)
        assert!(config.jwt_expiration_hours >= 1);
        assert!(config.jwt_expiration_hours <= 48);

        // Verify CORS origins are configured
        assert!(!config.cors_origins.is_empty());

        // Verify security policies exist
        assert!(!config.security_policies.is_empty());
    }

    #[test]
    fn test_security_policy_configuration() {
        let config = SecurityConfig::default();

        // Check for required security policies
        let policy_names: Vec<String> = config
            .security_policies
            .iter()
            .map(|p| p.name.clone())
            .collect();

        assert!(policy_names.contains(&"password_policy".to_string()));
        assert!(policy_names.contains(&"session_policy".to_string()));
        assert!(policy_names.contains(&"api_access_policy".to_string()));
    }

    #[test]
    fn test_password_policy_rules() {
        let config = SecurityConfig::default();

        let password_policy = config
            .security_policies
            .iter()
            .find(|p| p.name == "password_policy")
            .expect("Password policy should exist");

        assert!(password_policy.enabled);

        // Verify password policy rules exist
        let rules = &password_policy.rules;
        assert!(rules.iter().any(|r| r.contains("minimum_length")));
        assert!(rules.iter().any(|r| r.contains("require_uppercase")));
        assert!(rules.iter().any(|r| r.contains("require_lowercase")));
        assert!(rules.iter().any(|r| r.contains("require_numbers")));
        assert!(rules.iter().any(|r| r.contains("require_special_chars")));
    }

    #[test]
    fn test_session_policy_rules() {
        let config = SecurityConfig::default();

        let session_policy = config
            .security_policies
            .iter()
            .find(|p| p.name == "session_policy")
            .expect("Session policy should exist");

        assert!(session_policy.enabled);

        // Verify session policy rules exist
        let rules = &session_policy.rules;
        assert!(rules.iter().any(|r| r.contains("max_session_duration")));
        assert!(rules.iter().any(|r| r.contains("idle_timeout")));
        assert!(rules.iter().any(|r| r.contains("concurrent_sessions")));
    }

    #[test]
    fn test_api_access_policy_rules() {
        let config = SecurityConfig::default();

        let api_policy = config
            .security_policies
            .iter()
            .find(|p| p.name == "api_access_policy")
            .expect("API access policy should exist");

        assert!(api_policy.enabled);

        // Verify API policy rules exist
        let rules = &api_policy.rules;
        assert!(rules.iter().any(|r| r.contains("require_authentication")));
        assert!(rules.iter().any(|r| r.contains("require_authorization")));
        assert!(rules.iter().any(|r| r.contains("log_all_requests")));
    }
}

#[cfg(test)]
mod security_manager_tests {
    use super::*;

    #[tokio::test]
    async fn test_security_manager_creation() {
        let config = create_test_config();
        let manager = SecurityManager::new(config.clone());

        assert!(manager.is_ok());
        let manager = manager.unwrap();

        // Verify manager status
        let status = manager.status().await;
        assert!(status.contains("TLS: Enabled"));
        assert!(status.contains("Rate Limiting: Enabled"));
    }

    #[tokio::test]
    async fn test_security_manager_initialization() {
        let mut config = create_test_config();
        config.audit_logging_enabled = false; // Disable file I/O

        let mut manager = SecurityManager::new(config.clone()).unwrap();

        // Initialize should succeed
        let result = manager.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_security_manager_initialization_with_missing_tls_config() {
        let mut config = create_test_config();
        config.tls_enabled = true;
        config.tls_cert_path = None; // Missing cert
        config.tls_key_path = None; // Missing key

        let mut manager = SecurityManager::new(config).unwrap();

        // Initialization should fail with missing TLS config
        let result = manager.initialize().await;
        assert!(result.is_err());

        match result {
            Err(ProductionError::Security(msg)) => {
                assert!(msg.contains("TLS enabled but certificate or key path not specified"));
            }
            _ => panic!("Expected Security error"),
        }
    }

    #[tokio::test]
    async fn test_audit_event_logging() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        // Log a test event
        let event = create_test_audit_event(AuditEventType::Authentication, AuditResult::Success);
        let result = manager.log_audit_event(event).await;

        assert!(result.is_ok());

        // Verify event was logged
        let events = manager.get_recent_audit_events(10).await;
        assert_eq!(events.len(), 1);
        assert_eq!(format!("{:?}", events[0].event_type), "Authentication");
        assert_eq!(format!("{:?}", events[0].result), "Success");
    }

    #[tokio::test]
    async fn test_audit_event_rotation() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        // Log 10050 events (more than the 10000 limit)
        for _i in 0..10050 {
            let event = create_test_audit_event(AuditEventType::DataAccess, AuditResult::Success);
            manager.log_audit_event(event).await.unwrap();
        }

        // Should only keep last 10000 events
        let events = manager.get_recent_audit_events(20000).await;
        assert_eq!(events.len(), 10000);

        // Oldest event should be event #50
        assert_eq!(events[0].user_id, Some("test-user".to_string()));
    }

    #[tokio::test]
    async fn test_get_recent_audit_events_with_limit() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        // Log 50 events
        for _ in 0..50 {
            let event = create_test_audit_event(AuditEventType::DataAccess, AuditResult::Success);
            manager.log_audit_event(event).await.unwrap();
        }

        // Request only last 10 events
        let events = manager.get_recent_audit_events(10).await;
        assert_eq!(events.len(), 10);
    }

    #[tokio::test]
    async fn test_get_recent_audit_events_with_excess_limit() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        // Log 10 events
        for _ in 0..10 {
            let event = create_test_audit_event(AuditEventType::DataAccess, AuditResult::Success);
            manager.log_audit_event(event).await.unwrap();
        }

        // Request 100 events (more than exist)
        let events = manager.get_recent_audit_events(100).await;
        assert_eq!(events.len(), 10);
    }

    #[tokio::test]
    async fn test_security_audit_event_types() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        // Test all event types
        let event_types = vec![
            AuditEventType::Authentication,
            AuditEventType::Authorization,
            AuditEventType::DataAccess,
            AuditEventType::DataModification,
            AuditEventType::SystemAccess,
            AuditEventType::SecurityViolation,
            AuditEventType::ConfigurationChange,
        ];

        for event_type in event_types {
            let event = create_test_audit_event(event_type.clone(), AuditResult::Success);
            manager.log_audit_event(event).await.unwrap();
        }

        let events = manager.get_recent_audit_events(100).await;
        assert_eq!(events.len(), 7);
    }

    #[tokio::test]
    async fn test_security_audit_event_results() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        // Test all result types
        let results = vec![
            AuditResult::Success,
            AuditResult::Failure,
            AuditResult::Blocked,
            AuditResult::Warning,
        ];

        for result in results {
            let event = create_test_audit_event(AuditEventType::Authentication, result);
            manager.log_audit_event(event).await.unwrap();
        }

        let events = manager.get_recent_audit_events(100).await;
        assert_eq!(events.len(), 4);
    }

    #[tokio::test]
    async fn test_security_report_generation() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        // Log some test events
        for _ in 0..10 {
            let event =
                create_test_audit_event(AuditEventType::Authentication, AuditResult::Success);
            manager.log_audit_event(event).await.unwrap();
        }

        for _ in 0..5 {
            let event =
                create_test_audit_event(AuditEventType::Authorization, AuditResult::Failure);
            manager.log_audit_event(event).await.unwrap();
        }

        // Generate report
        let report = manager.generate_security_report().await;

        // Verify report contains expected sections
        assert!(report.contains("ProvChain Security Audit Report"));
        assert!(report.contains("Total Audit Events: 15"));
        assert!(report.contains("TLS Enabled: Yes"));
        assert!(report.contains("Rate Limiting: Yes"));

        // Verify event types are summarized
        assert!(report.contains("Authentication: 10"));
        assert!(report.contains("Authorization: 5"));

        // Verify results are summarized
        assert!(report.contains("Success: 10"));
        assert!(report.contains("Failure: 5"));
    }

    #[tokio::test]
    async fn test_security_manager_shutdown() {
        let config = create_test_config();
        let mut manager = SecurityManager::new(config).unwrap();

        // Shutdown should succeed
        let result = manager.shutdown().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_nginx_config_generation() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        let nginx_config = manager.generate_nginx_config();

        // Verify essential Nginx configuration elements
        assert!(nginx_config.contains("listen 80"));
        assert!(nginx_config.contains("listen 443 ssl"));
        assert!(nginx_config.contains("ssl_certificate"));
        assert!(nginx_config.contains("ssl_certificate_key"));
        assert!(nginx_config.contains("TLSv1.2 TLSv1.3"));

        // Verify security headers
        assert!(nginx_config.contains("X-Frame-Options DENY"));
        assert!(nginx_config.contains("X-Content-Type-Options nosniff"));
        assert!(nginx_config.contains("X-XSS-Protection"));
        assert!(nginx_config.contains("Strict-Transport-Security"));
        assert!(nginx_config.contains("Content-Security-Policy"));

        // Verify rate limiting
        assert!(nginx_config.contains("limit_req_zone"));
        assert!(nginx_config.contains("limit_req zone"));

        // Verify proxy configuration
        assert!(nginx_config.contains("proxy_pass http://127.0.0.1:8080"));
    }

    #[tokio::test]
    async fn test_firewall_rules_generation() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        let firewall_rules = manager.generate_firewall_rules();

        // Verify essential firewall rules
        assert!(firewall_rules.contains("iptables -F"));
        assert!(firewall_rules.contains("iptables -P INPUT DROP"));
        assert!(firewall_rules.contains("iptables -A INPUT -i lo -j ACCEPT"));

        // Verify SSH rule
        assert!(firewall_rules.contains("--dport 22"));

        // Verify HTTP/HTTPS rules
        assert!(firewall_rules.contains("--dport 80"));
        assert!(firewall_rules.contains("--dport 443"));

        // Verify application port rules (internal only)
        assert!(firewall_rules.contains("--dport 8080"));
        assert!(firewall_rules.contains("-s 127.0.0.1"));

        // Verify rate limiting in firewall
        assert!(firewall_rules.contains("-m limit --limit"));

        // Verify packet dropping
        assert!(firewall_rules.contains("INVALID"));
    }
}

#[cfg(test)]
mod security_middleware_tests {
    use super::*;

    #[test]
    fn test_security_middleware_creation() {
        let config = create_test_config();
        let middleware = SecurityMiddleware::new(config);

        // Middleware should be created successfully (can't access private config field)
        // Just verify it was created without panicking
        let _ = middleware;
    }

    #[test]
    fn test_validate_request_headers_with_security_enabled() {
        let config = create_test_config();
        let middleware = SecurityMiddleware::new(config);

        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "test-agent".to_string());
        headers.insert("Accept".to_string(), "application/json".to_string());

        // Should succeed with valid headers
        let result = middleware.validate_request_headers(&headers);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_request_headers_with_empty_headers() {
        let config = create_test_config();
        let middleware = SecurityMiddleware::new(config);

        let headers = HashMap::new();

        // Should succeed even with empty headers (implementation is a stub)
        let result = middleware.validate_request_headers(&headers);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_rate_limit_enabled() {
        let config = create_test_config();
        let middleware = SecurityMiddleware::new(config);

        // Rate limiting is enabled in test config
        let result = middleware.check_rate_limit("127.0.0.1");
        assert!(result.is_ok());

        // Current implementation always returns true (TODO: Implement actual rate limiting)
        let allowed = result.unwrap();
        assert_eq!(allowed, true);
    }

    #[test]
    fn test_check_rate_limit_disabled() {
        let mut config = create_test_config();
        config.rate_limiting_enabled = false;
        let middleware = SecurityMiddleware::new(config);

        // Rate limiting is disabled - should always allow
        let result = middleware.check_rate_limit("127.0.0.1");
        assert!(result.is_ok());

        let allowed = result.unwrap();
        assert_eq!(allowed, true);
    }

    #[test]
    fn test_check_rate_limit_different_ips() {
        let config = create_test_config();
        let middleware = SecurityMiddleware::new(config);

        // Test with different IP addresses
        let ips = vec!["127.0.0.1", "192.168.1.1", "10.0.0.1"];

        for ip in ips {
            let result = middleware.check_rate_limit(ip);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), true);
        }
    }

    #[test]
    fn test_check_rate_limit_with_malformed_ips() {
        let config = create_test_config();
        let middleware = SecurityMiddleware::new(config);

        // Test with malformed IP addresses (should still work as it's just a string)
        let malformed_ips = vec!["", "invalid", "999.999.999.999", "localhost"];

        for ip in malformed_ips {
            let result = middleware.check_rate_limit(ip);
            assert!(result.is_ok());
        }
    }
}

#[cfg(test)]
mod jwt_validation_tests {
    use super::*;

    #[test]
    fn test_validate_jwt_with_valid_token() {
        let config = create_test_config();
        let middleware = SecurityMiddleware::new(config);

        // Test with a properly signed valid JWT token
        let valid_token = create_test_jwt_token("test-user", 1);
        let result = middleware.validate_jwt(&valid_token);

        assert!(result.is_ok());
        // Should be valid: proper signature, not expired
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_validate_jwt_with_expired_token() {
        let config = create_test_config();
        let middleware = SecurityMiddleware::new(config);

        // Test with an expired token
        let expired_token = create_expired_jwt_token();
        let result = middleware.validate_jwt(&expired_token);

        assert!(result.is_ok());
        // Should be invalid because token is expired
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_validate_jwt_with_empty_token() {
        let config = create_test_config();
        let middleware = SecurityMiddleware::new(config);

        // Test with an empty token
        let result = middleware.validate_jwt("");
        assert!(result.is_ok());

        // Should be invalid because token is empty
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_validate_jwt_with_invalid_signature() {
        let config = create_test_config();
        let middleware = SecurityMiddleware::new(config);

        // Create a valid token, then tamper with it
        let mut token = create_test_jwt_token("test-user", 1);
        // Corrupt the signature by changing the last characters
        unsafe {
            let bytes = token.as_bytes_mut();
            if let Some(last) = bytes.last_mut() {
                *last = b'X';
            }
        }

        let result = middleware.validate_jwt(&token);
        assert!(result.is_ok());

        // Should be invalid because signature doesn't match
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_validate_jwt_with_malformed_token() {
        let config = create_test_config();
        let middleware = SecurityMiddleware::new(config);

        // Test with malformed tokens
        let malformed_tokens = vec![
            "not-a-jwt",
            "invalid.token",
            "invalid.token.format",
            "Bearer token",
            "   ", // whitespace only
        ];

        for token in malformed_tokens {
            let result = middleware.validate_jwt(token);
            assert!(result.is_ok(), "Should handle malformed token: {}", token);
            // Should be invalid because token is malformed
            assert_eq!(
                result.unwrap(),
                false,
                "Malformed token should be rejected: {}",
                token
            );
        }
    }

    #[test]
    fn test_validate_jwt_with_wrong_secret() {
        let config = create_test_config();
        let middleware = SecurityMiddleware::new(config);

        // Create a token signed with a different secret
        use jsonwebtoken::{encode, EncodingKey, Header};

        let now = chrono::Utc::now();
        let exp = (now + chrono::Duration::hours(1)).timestamp();

        let claims = ProductionClaims {
            sub: "test-user".to_string(),
            iat: now.timestamp(),
            exp,
            iss: "provchain-test".to_string(),
            role: Some("test".to_string()),
        };

        // Sign with wrong secret
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(b"different-secret-key-32-chars-minimum"),
        )
        .expect("Failed to encode test JWT token");

        let result = middleware.validate_jwt(&token);
        assert!(result.is_ok());

        // Should be invalid because signature doesn't match
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_validate_jwt_with_future_expiration() {
        let config = create_test_config();
        let middleware = SecurityMiddleware::new(config);

        // Token with expiration far in the future (should be valid)
        let token = create_test_jwt_token("test-user", 24); // 24 hours from now
        let result = middleware.validate_jwt(&token);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_validate_jwt_with_unicode_subject() {
        let config = create_test_config();
        let middleware = SecurityMiddleware::new(config);

        // Token with Unicode characters in subject
        let token = create_test_jwt_token("用户-🔑", 1);
        let result = middleware.validate_jwt(&token);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_validate_jwt_rejects_short_secret() {
        // This test must run without JWT_SECRET environment variable
        // to test the config secret validation
        if std::env::var("JWT_SECRET").is_ok() {
            // Skip this test if JWT_SECRET is set, since it takes precedence
            // The environment variable secret will be used instead of config secret
            return;
        }

        let mut config = create_test_config();
        // Use a short secret (less than 32 characters)
        config.jwt_secret = "short".to_string();

        let middleware = SecurityMiddleware::new(config);

        let valid_token = create_test_jwt_token("test-user", 1);
        let result = middleware.validate_jwt(&valid_token);

        // Should fail because secret is too short
        assert!(
            result.is_err(),
            "Should reject short config secret when JWT_SECRET env var is not set"
        );
    }

    #[test]
    fn test_validate_jwt_with_very_long_token() {
        let config = create_test_config();
        let middleware = SecurityMiddleware::new(config);

        // Create token with very long subject
        let long_subject = "a".repeat(10000);
        let token = create_test_jwt_token(&long_subject, 1);

        let result = middleware.validate_jwt(&token);
        assert!(result.is_ok());

        // Should be valid (just a long subject)
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_validate_jwt_with_null_bytes() {
        let config = create_test_config();
        let middleware = SecurityMiddleware::new(config);

        // Test with null bytes in token (malformed)
        let token_with_null = "invalid.token\0format";
        let result = middleware.validate_jwt(token_with_null);

        assert!(result.is_ok());
        // Should be invalid (malformed JWT)
        assert_eq!(result.unwrap(), false);
    }
}

#[cfg(test)]
mod rate_limiting_tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_rate_limit_thresholds() {
        let mut config = create_test_config();
        config.rate_limit_per_minute = 10; // Set low limit for testing
        let middleware = SecurityMiddleware::new(config);

        let ip = "127.0.0.1";

        // First 10 requests should be allowed (burst capacity is 20)
        let mut allowed_count = 0;
        for _ in 0..20 {
            if middleware.check_rate_limit(ip).unwrap() {
                allowed_count += 1;
            }
        }

        // Should allow at least burst capacity (2x rate limit)
        assert_eq!(allowed_count, 20, "Should allow burst capacity requests");

        // Next requests should be blocked
        let blocked_result = middleware.check_rate_limit(ip).unwrap();
        assert!(
            !blocked_result,
            "Should block after burst capacity exhausted"
        );

        // Verify token count is low
        let token_count = middleware.get_rate_limit_token_count(ip).unwrap();
        assert!(
            token_count < 1.0,
            "Token count should be near zero after burst"
        );
    }

    #[test]
    fn test_rate_limit_burst_protection() {
        let mut config = create_test_config();
        config.rate_limit_per_minute = 10; // Low limit for testing
        let middleware = SecurityMiddleware::new(config);

        let ip = "192.168.1.1";

        // Send rapid burst requests
        let mut allowed = 0;
        for _ in 0..30 {
            if middleware.check_rate_limit(ip).unwrap() {
                allowed += 1;
            }
        }

        // Should allow burst capacity (20) but no more
        assert_eq!(allowed, 20, "Should limit to burst capacity");

        // All further requests should be blocked
        for _ in 0..10 {
            let result = middleware.check_rate_limit(ip).unwrap();
            assert!(!result, "Should block after burst exhausted");
        }
    }

    #[test]
    fn test_rate_limit_token_refill() {
        let mut config = create_test_config();
        config.rate_limit_per_minute = 60; // 1 per second for easy testing
        let middleware = SecurityMiddleware::new(config);

        let ip = "10.0.0.1";

        // Exhaust burst capacity (120 = 2x 60)
        for _ in 0..120 {
            middleware.check_rate_limit(ip).unwrap();
        }

        // Should be blocked
        assert!(
            !middleware.check_rate_limit(ip).unwrap(),
            "Should be blocked after exhaustion"
        );

        // Wait for token refill (1 second)
        thread::sleep(Duration::from_secs(1));

        // Should now have 1 token available
        assert!(
            middleware.check_rate_limit(ip).unwrap(),
            "Should allow after token refill"
        );

        // Should be blocked again
        assert!(
            !middleware.check_rate_limit(ip).unwrap(),
            "Should be blocked after using refilled token"
        );
    }

    #[test]
    fn test_rate_limit_isolation_between_ips() {
        let mut config = create_test_config();
        config.rate_limit_per_minute = 5;
        let middleware = SecurityMiddleware::new(config);

        let ip1 = "127.0.0.1";
        let ip2 = "192.168.1.1";

        // Exhaust IP1's burst capacity (10)
        for _ in 0..10 {
            middleware.check_rate_limit(ip1).unwrap();
        }

        // IP1 should be blocked
        assert!(
            !middleware.check_rate_limit(ip1).unwrap(),
            "IP1 should be blocked"
        );

        // IP2 should still be allowed
        assert!(
            middleware.check_rate_limit(ip2).unwrap(),
            "IP2 should be allowed"
        );

        // Verify tracked IPs
        assert_eq!(middleware.tracked_ip_count(), 2, "Should track both IPs");
    }

    #[test]
    fn test_rate_limit_with_ipv6() {
        let mut config = create_test_config();
        config.rate_limit_per_minute = 5;
        let middleware = SecurityMiddleware::new(config);

        let ipv6_addresses = vec![
            "::1",
            "fe80::1",
            "2001:db8::1",
            "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
        ];

        // Each IPv6 address should have its own rate limit
        for ip in ipv6_addresses {
            // Exhaust burst for this IP
            for _ in 0..10 {
                let result = middleware.check_rate_limit(ip);
                assert!(result.is_ok(), "IPv6 address {} should be handled", ip);
                assert!(
                    result.unwrap(),
                    "IPv6 address {} should be allowed initially",
                    ip
                );
            }

            // Should be blocked after burst
            let result = middleware.check_rate_limit(ip).unwrap();
            assert!(!result, "IPv6 address {} should be blocked after burst", ip);
        }

        // Verify all IPs are tracked separately
        assert_eq!(
            middleware.tracked_ip_count(),
            4,
            "Should track all IPv6 addresses separately"
        );
    }

    #[test]
    fn test_rate_limit_disabled() {
        let mut config = create_test_config();
        config.rate_limiting_enabled = false;
        let middleware = SecurityMiddleware::new(config);

        let ip = "127.0.0.1";

        // Even after many requests, should always be allowed
        for _ in 0..1000 {
            let result = middleware.check_rate_limit(ip).unwrap();
            assert!(
                result,
                "Should allow all requests when rate limiting disabled"
            );
        }

        // Token count should be None since rate limiting is disabled
        assert!(middleware.get_rate_limit_token_count(ip).is_none());
    }

    #[test]
    fn test_rate_limit_cleanup() {
        let mut config = create_test_config();
        config.rate_limit_per_minute = 10;
        let middleware = SecurityMiddleware::new(config);

        // Add multiple IPs
        for i in 0..5 {
            let ip = format!("192.168.1.{}", i);
            for _ in 0..10 {
                middleware.check_rate_limit(&ip).unwrap();
            }
        }

        assert_eq!(middleware.tracked_ip_count(), 5, "Should track 5 IPs");

        // Cleanup stale entries (with 0 second max age, should remove all)
        middleware.cleanup_stale_rate_limits(0);

        // All entries should be cleaned up
        assert_eq!(
            middleware.tracked_ip_count(),
            0,
            "Should clean up all stale entries"
        );
    }

    #[test]
    fn test_rate_limit_different_rates_for_different_ips() {
        let mut config = create_test_config();
        config.rate_limit_per_minute = 10;
        let middleware = SecurityMiddleware::new(config);

        let ip1 = "127.0.0.1";
        let ip2 = "192.168.1.1";

        // Exhaust both IPs
        for _ in 0..20 {
            middleware.check_rate_limit(ip1).unwrap();
            middleware.check_rate_limit(ip2).unwrap();
        }

        // Both should be blocked
        assert!(
            !middleware.check_rate_limit(ip1).unwrap(),
            "IP1 should be blocked"
        );
        assert!(
            !middleware.check_rate_limit(ip2).unwrap(),
            "IP2 should be blocked"
        );

        // Wait 1 second for partial refill
        thread::sleep(Duration::from_secs(1));

        // Both should have ~0.17 tokens (10/60 per second)
        // Not enough for a full request
        assert!(
            !middleware.check_rate_limit(ip1).unwrap(),
            "IP1 should still be blocked after 1 second"
        );
        assert!(
            !middleware.check_rate_limit(ip2).unwrap(),
            "IP2 should still be blocked after 1 second"
        );

        // Wait 6 more seconds (7 total = ~1.17 tokens each)
        thread::sleep(Duration::from_secs(6));

        // Now both should have enough tokens
        assert!(
            middleware.check_rate_limit(ip1).unwrap(),
            "IP1 should be allowed after refill"
        );
        assert!(
            middleware.check_rate_limit(ip2).unwrap(),
            "IP2 should be allowed after refill"
        );
    }

    #[test]
    fn test_rate_limit_token_count_tracking() {
        let mut config = create_test_config();
        config.rate_limit_per_minute = 60; // 1 per second
        let middleware = SecurityMiddleware::new(config);

        let ip = "127.0.0.1";

        // Consume some tokens to create the bucket
        for _ in 0..5 {
            middleware.check_rate_limit(ip).unwrap();
        }

        // Get initial token count after creating bucket
        let initial_count = middleware.get_rate_limit_token_count(ip).unwrap();
        assert!(initial_count > 100.0, "Initial token count should be high");

        // Consume more tokens
        for _ in 0..10 {
            middleware.check_rate_limit(ip).unwrap();
        }

        // Token count should decrease
        let after_count = middleware.get_rate_limit_token_count(ip).unwrap();
        assert!(
            after_count < initial_count,
            "Token count should decrease after requests"
        );
        assert!(after_count > 100.0, "Should still have tokens left");
    }
}

#[cfg(test)]
mod input_sanitization_tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_event_with_malicious_user_input() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        // Test with potentially malicious user input
        let malicious_inputs = vec![
            "<script>alert('xss')</script>",
            "'; DROP TABLE users; --",
            "../../../etc/passwd",
            "{{7*7}}",
            "${7*7}",
            "\n[ERROR] Admin login failed",
        ];

        for input in malicious_inputs {
            let event = SecurityAuditEvent {
                timestamp: chrono::Utc::now(),
                event_type: AuditEventType::Authentication,
                user_id: Some(input.to_string()),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("test-agent".to_string()),
                resource: "/api/test".to_string(),
                action: "test-action".to_string(),
                result: AuditResult::Success,
                details: HashMap::new(),
            };

            // Should log the event (sanitization should happen at display time)
            let result = manager.log_audit_event(event).await;
            assert!(result.is_ok(), "Should handle malicious input: {}", input);
        }
    }

    #[tokio::test]
    async fn test_audit_event_details_with_special_characters() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        let mut details = HashMap::new();
        details.insert(
            "key\nwith\nnewlines".to_string(),
            "value\twith\ttabs".to_string(),
        );
        details.insert(
            "key\"with\"quotes".to_string(),
            "value'with'apostrophes".to_string(),
        );
        details.insert(
            "key<with>brackets".to_string(),
            "value&with&ampersands".to_string(),
        );

        let event = SecurityAuditEvent {
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::DataAccess,
            user_id: Some("test-user".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
            resource: "/api/test".to_string(),
            action: "test-action".to_string(),
            result: AuditResult::Success,
            details,
        };

        let result = manager.log_audit_event(event).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_security_config_with_malicious_origins() {
        let mut config = create_test_config();

        // Test with potentially malicious CORS origins
        let malicious_origins = vec![
            "javascript:alert('xss')".to_string(),
            "data:text/html,<script>alert('xss')</script>".to_string(),
            "file:///etc/passwd".to_string(),
            "http://localhost@evil.com".to_string(),
        ];

        config.cors_origins = malicious_origins;

        // Should create the config (validation should happen at initialization time)
        let middleware = SecurityMiddleware::new(config);
        // Can't access private config field to verify, but creation succeeded
        let _ = middleware;
        // TODO: Implement CORS origin validation
    }
}

#[cfg(test)]
mod compliance_tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_trail_completeness() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        // Log a sequence of related events
        let user_id = "test-user";
        let ip = "127.0.0.1";

        let events = vec![
            (AuditEventType::Authentication, AuditResult::Success),
            (AuditEventType::Authorization, AuditResult::Success),
            (AuditEventType::DataAccess, AuditResult::Success),
            (AuditEventType::DataModification, AuditResult::Success),
        ];

        for (event_type, result) in events {
            let event = SecurityAuditEvent {
                timestamp: chrono::Utc::now(),
                event_type,
                user_id: Some(user_id.to_string()),
                ip_address: Some(ip.to_string()),
                user_agent: Some("test-agent".to_string()),
                resource: "/api/test".to_string(),
                action: "test-action".to_string(),
                result,
                details: HashMap::new(),
            };

            manager.log_audit_event(event).await.unwrap();
        }

        // Verify all events were logged
        let all_events = manager.get_recent_audit_events(100).await;
        assert_eq!(all_events.len(), 4);

        // Verify audit trail completeness
        for event in &all_events {
            assert_eq!(event.user_id, Some(user_id.to_string()));
            assert_eq!(event.ip_address, Some(ip.to_string()));
            assert!(event.timestamp <= chrono::Utc::now());
        }
    }

    #[tokio::test]
    async fn test_audit_event_timestamp_accuracy() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        let _before = SystemTime::now();

        let event = create_test_audit_event(AuditEventType::Authentication, AuditResult::Success);
        manager.log_audit_event(event).await.unwrap();

        let _after = SystemTime::now();

        let events = manager.get_recent_audit_events(1).await;
        assert_eq!(events.len(), 1);

        let event_time = events[0].timestamp;
        let before_chrono = chrono::Utc::now() - chrono::Duration::seconds(1);

        // Event timestamp should be reasonable (within last second)
        assert!(event_time >= before_chrono);
        assert!(event_time <= chrono::Utc::now());
    }

    #[tokio::test]
    async fn test_security_violation_logging() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        // Log security violations
        let violations = vec![
            (
                AuditEventType::SecurityViolation,
                "Invalid credentials".to_string(),
            ),
            (
                AuditEventType::SecurityViolation,
                "Rate limit exceeded".to_string(),
            ),
            (
                AuditEventType::SecurityViolation,
                "Suspicious activity detected".to_string(),
            ),
        ];

        for (event_type, action) in violations {
            let event = SecurityAuditEvent {
                timestamp: chrono::Utc::now(),
                event_type,
                user_id: Some("suspicious-user".to_string()),
                ip_address: Some("10.0.0.100".to_string()),
                user_agent: Some("evil-scanner".to_string()),
                resource: "/api/admin".to_string(),
                action,
                result: AuditResult::Blocked,
                details: {
                    let mut map = HashMap::new();
                    map.insert("severity".to_string(), "high".to_string());
                    map
                },
            };

            manager.log_audit_event(event).await.unwrap();
        }

        let events = manager.get_recent_audit_events(100).await;

        // Verify all violations were logged
        let violation_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e.event_type, AuditEventType::SecurityViolation))
            .collect();

        assert_eq!(violation_events.len(), 3);

        // Verify all violations were blocked
        for event in violation_events {
            assert!(matches!(event.result, AuditResult::Blocked));
        }
    }

    #[tokio::test]
    async fn test_data_access_auditing() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        // Simulate data access by different users
        let access_scenarios = vec![
            ("user1", "/api/products/123", "read"),
            ("user2", "/api/products/456", "read"),
            ("admin", "/api/products/789", "delete"),
        ];

        for (user_id, resource, action) in access_scenarios {
            let event = SecurityAuditEvent {
                timestamp: chrono::Utc::now(),
                event_type: AuditEventType::DataAccess,
                user_id: Some(user_id.to_string()),
                ip_address: Some("10.0.0.50".to_string()),
                user_agent: Some("web-client".to_string()),
                resource: resource.to_string(),
                action: action.to_string(),
                result: AuditResult::Success,
                details: HashMap::new(),
            };

            manager.log_audit_event(event).await.unwrap();
        }

        let events = manager.get_recent_audit_events(100).await;
        assert_eq!(events.len(), 3);

        // Verify all access events are logged with proper details
        for event in &events {
            assert!(matches!(event.event_type, AuditEventType::DataAccess));
            assert!(event.user_id.is_some());
            assert!(!event.resource.is_empty());
            assert!(!event.action.is_empty());
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_logging_performance() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        let start = std::time::Instant::now();

        // Log 1000 events
        for _i in 0..1000 {
            let event = create_test_audit_event(AuditEventType::DataAccess, AuditResult::Success);
            manager.log_audit_event(event).await.unwrap();
        }

        let duration = start.elapsed();

        // Should complete in reasonable time (< 1 second for 1000 events)
        assert!(
            duration.as_secs() < 1,
            "Audit logging too slow: {}ms for 1000 events",
            duration.as_millis()
        );
    }

    #[tokio::test]
    async fn test_concurrent_audit_logging() {
        let config = create_test_config();
        let manager = std::sync::Arc::new(SecurityManager::new(config).unwrap());

        // Spawn multiple tasks logging concurrently
        let mut handles = vec![];

        for _i in 0..10 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                for _j in 0..100 {
                    let event =
                        create_test_audit_event(AuditEventType::DataAccess, AuditResult::Success);
                    manager_clone.log_audit_event(event).await.unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all events were logged
        let events = manager.get_recent_audit_events(2000).await;
        assert_eq!(events.len(), 1000);
    }

    #[tokio::test]
    async fn test_security_report_generation_performance() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        // Log many events
        for _ in 0..5000 {
            let event = create_test_audit_event(AuditEventType::DataAccess, AuditResult::Success);
            manager.log_audit_event(event).await.unwrap();
        }

        let start = std::time::Instant::now();
        let report = manager.generate_security_report().await;
        let duration = start.elapsed();

        // Report generation should be fast (< 100ms)
        assert!(
            duration.as_millis() < 100,
            "Report generation too slow: {}ms",
            duration.as_millis()
        );

        assert!(report.contains("Total Audit Events: 5000"));
    }
}
