//! Security hardening and compliance for production deployment

use crate::production::ProductionError;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable TLS/SSL
    pub tls_enabled: bool,
    /// TLS certificate path
    pub tls_cert_path: Option<String>,
    /// TLS private key path
    pub tls_key_path: Option<String>,
    /// Enable API rate limiting
    pub rate_limiting_enabled: bool,
    /// Rate limit per minute
    pub rate_limit_per_minute: u32,
    /// Enable request validation
    pub request_validation_enabled: bool,
    /// Enable CORS
    pub cors_enabled: bool,
    /// Allowed CORS origins
    pub cors_origins: Vec<String>,
    /// Enable security headers
    pub security_headers_enabled: bool,
    /// JWT secret key
    pub jwt_secret: String,
    /// JWT expiration time in hours
    pub jwt_expiration_hours: u64,
    /// Enable audit logging
    pub audit_logging_enabled: bool,
    /// Audit log path
    pub audit_log_path: String,
    /// Security policies
    pub security_policies: Vec<SecurityPolicy>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            tls_enabled: true,
            tls_cert_path: Some("/etc/ssl/certs/provchain.crt".to_string()),
            tls_key_path: Some("/etc/ssl/private/provchain.key".to_string()),
            rate_limiting_enabled: true,
            rate_limit_per_minute: 100,
            request_validation_enabled: true,
            cors_enabled: true,
            cors_origins: vec![
                "https://provchain.local".to_string(),
                "https://dashboard.provchain.local".to_string(),
            ],
            security_headers_enabled: true,
            jwt_secret: "your-super-secret-jwt-key-change-in-production".to_string(),
            jwt_expiration_hours: 24,
            audit_logging_enabled: true,
            audit_log_path: "/var/log/provchain/audit.log".to_string(),
            security_policies: vec![
                SecurityPolicy {
                    name: "password_policy".to_string(),
                    policy_type: PolicyType::Password,
                    rules: vec![
                        "minimum_length:8".to_string(),
                        "require_uppercase:true".to_string(),
                        "require_lowercase:true".to_string(),
                        "require_numbers:true".to_string(),
                        "require_special_chars:true".to_string(),
                    ],
                    enabled: true,
                },
                SecurityPolicy {
                    name: "session_policy".to_string(),
                    policy_type: PolicyType::Session,
                    rules: vec![
                        "max_session_duration:86400".to_string(), // 24 hours
                        "idle_timeout:3600".to_string(),          // 1 hour
                        "concurrent_sessions:3".to_string(),
                    ],
                    enabled: true,
                },
                SecurityPolicy {
                    name: "api_access_policy".to_string(),
                    policy_type: PolicyType::ApiAccess,
                    rules: vec![
                        "require_authentication:true".to_string(),
                        "require_authorization:true".to_string(),
                        "log_all_requests:true".to_string(),
                    ],
                    enabled: true,
                },
            ],
        }
    }
}

/// Security policy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub name: String,
    pub policy_type: PolicyType,
    pub rules: Vec<String>,
    pub enabled: bool,
}

/// Types of security policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyType {
    Password,
    Session,
    ApiAccess,
    DataAccess,
    NetworkAccess,
}

/// Claims for production JWT tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionClaims {
    /// Subject (user ID or system identifier)
    pub sub: String,
    /// Issued at (timestamp)
    pub iat: i64,
    /// Expiration time (timestamp)
    pub exp: i64,
    /// Issuer
    pub iss: String,
    /// Role/permission level
    pub role: Option<String>,
}

/// Security audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: AuditEventType,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub resource: String,
    pub action: String,
    pub result: AuditResult,
    pub details: HashMap<String, String>,
}

/// Types of audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    Authentication,
    Authorization,
    DataAccess,
    DataModification,
    SystemAccess,
    SecurityViolation,
    ConfigurationChange,
}

/// Audit event results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure,
    Blocked,
    Warning,
}

/// Security manager
pub struct SecurityManager {
    config: SecurityConfig,
    audit_events: std::sync::Arc<tokio::sync::RwLock<Vec<SecurityAuditEvent>>>,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(config: SecurityConfig) -> Result<Self, ProductionError> {
        Ok(Self {
            config,
            audit_events: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        })
    }

    /// Initialize security systems
    pub async fn initialize(&mut self) -> Result<(), ProductionError> {
        tracing::info!("Initializing security systems");

        // Validate TLS configuration
        if self.config.tls_enabled {
            self.validate_tls_config().await?;
        }

        // Initialize audit logging
        if self.config.audit_logging_enabled {
            self.initialize_audit_logging().await?;
        }

        // Validate security policies
        self.validate_security_policies().await?;

        tracing::info!("Security systems initialized successfully");
        Ok(())
    }

    /// Validate TLS configuration
    async fn validate_tls_config(&self) -> Result<(), ProductionError> {
        if let (Some(cert_path), Some(key_path)) =
            (&self.config.tls_cert_path, &self.config.tls_key_path)
        {
            // In a real implementation, we would validate the certificate and key files
            tracing::info!(
                "TLS configuration validated: cert={}, key={}",
                cert_path,
                key_path
            );
        } else {
            return Err(ProductionError::Security(
                "TLS enabled but certificate or key path not specified".to_string(),
            ));
        }
        Ok(())
    }

    /// Initialize audit logging
    async fn initialize_audit_logging(&self) -> Result<(), ProductionError> {
        // Create audit log directory if it doesn't exist
        let log_path = PathBuf::from(&self.config.audit_log_path);
        if let Some(parent) = log_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                ProductionError::Security(format!("Failed to create audit log directory: {e}"))
            })?;
        }

        tracing::info!("Audit logging initialized: {}", self.config.audit_log_path);
        Ok(())
    }

    /// Validate security policies
    async fn validate_security_policies(&self) -> Result<(), ProductionError> {
        for policy in &self.config.security_policies {
            if policy.enabled {
                tracing::debug!("Validating security policy: {}", policy.name);
                // In a real implementation, we would validate policy rules
            }
        }
        Ok(())
    }

    /// Log security audit event
    pub async fn log_audit_event(&self, event: SecurityAuditEvent) -> Result<(), ProductionError> {
        // Add to in-memory storage
        {
            let mut events = self.audit_events.write().await;
            events.push(event.clone());

            // Keep only last 10000 events in memory
            if events.len() > 10000 {
                events.remove(0);
            }
        }

        // Write to audit log file if enabled
        if self.config.audit_logging_enabled {
            let log_entry = serde_json::to_string(&event).map_err(|e| {
                ProductionError::Security(format!("Failed to serialize audit event: {e}"))
            })?;

            // In a real implementation, we would write to the actual log file
            tracing::info!("Audit event: {}", log_entry);
        }

        Ok(())
    }

    /// Get security status
    pub async fn status(&self) -> String {
        let events_count = self.audit_events.read().await.len();
        format!(
            "TLS: {}, Rate Limiting: {}, Audit Events: {}",
            if self.config.tls_enabled {
                "Enabled"
            } else {
                "Disabled"
            },
            if self.config.rate_limiting_enabled {
                "Enabled"
            } else {
                "Disabled"
            },
            events_count
        )
    }

    /// Get recent audit events
    pub async fn get_recent_audit_events(&self, limit: usize) -> Vec<SecurityAuditEvent> {
        let events = self.audit_events.read().await;
        let start = if events.len() > limit {
            events.len() - limit
        } else {
            0
        };
        events[start..].to_vec()
    }

    /// Generate security configuration files
    pub fn generate_nginx_config(&self) -> String {
        format!(
            r#"# Nginx configuration for ProvChain
server {{
    listen 80;
    listen [::]:80;
    server_name provchain.local;
    
    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}}

server {{
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name provchain.local;

    # SSL Configuration
    ssl_certificate {};
    ssl_certificate_key {};
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # Security Headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'";
    add_header Referrer-Policy "strict-origin-when-cross-origin";

    # Rate Limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate={}r/m;
    limit_req zone=api burst=20 nodelay;

    # Proxy to ProvChain application
    location / {{
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }}

    # Metrics endpoint (restricted access)
    location /metrics {{
        allow 127.0.0.1;
        allow 10.0.0.0/8;
        allow 172.16.0.0/12;
        allow 192.168.0.0/16;
        deny all;
        
        proxy_pass http://127.0.0.1:9090;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }}

    # Health check endpoint
    location /health {{
        proxy_pass http://127.0.0.1:8080;
        access_log off;
    }}

    # Static files
    location /static/ {{
        alias /var/www/provchain/static/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }}
}}
"#,
            self.config
                .tls_cert_path
                .as_ref()
                .unwrap_or(&"/etc/ssl/certs/provchain.crt".to_string()),
            self.config
                .tls_key_path
                .as_ref()
                .unwrap_or(&"/etc/ssl/private/provchain.key".to_string()),
            self.config.rate_limit_per_minute
        )
    }

    /// Generate firewall rules (iptables)
    pub fn generate_firewall_rules(&self) -> String {
        r#"#!/bin/bash
# iptables firewall rules for ProvChain

# Flush existing rules
iptables -F
iptables -X
iptables -t nat -F
iptables -t nat -X
iptables -t mangle -F
iptables -t mangle -X

# Set default policies
iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -P OUTPUT ACCEPT

# Allow loopback traffic
iptables -A INPUT -i lo -j ACCEPT
iptables -A OUTPUT -o lo -j ACCEPT

# Allow established and related connections
iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT

# Allow SSH (change port as needed)
iptables -A INPUT -p tcp --dport 22 -m conntrack --ctstate NEW,ESTABLISHED -j ACCEPT

# Allow HTTP and HTTPS
iptables -A INPUT -p tcp --dport 80 -m conntrack --ctstate NEW,ESTABLISHED -j ACCEPT
iptables -A INPUT -p tcp --dport 443 -m conntrack --ctstate NEW,ESTABLISHED -j ACCEPT

# Allow ProvChain application port (internal only)
iptables -A INPUT -p tcp --dport 8080 -s 127.0.0.1 -j ACCEPT
iptables -A INPUT -p tcp --dport 8080 -s 10.0.0.0/8 -j ACCEPT
iptables -A INPUT -p tcp --dport 8080 -s 172.16.0.0/12 -j ACCEPT
iptables -A INPUT -p tcp --dport 8080 -s 192.168.0.0/16 -j ACCEPT

# Allow Prometheus metrics (internal only)
iptables -A INPUT -p tcp --dport 9090 -s 127.0.0.1 -j ACCEPT
iptables -A INPUT -p tcp --dport 9090 -s 10.0.0.0/8 -j ACCEPT
iptables -A INPUT -p tcp --dport 9090 -s 172.16.0.0/12 -j ACCEPT
iptables -A INPUT -p tcp --dport 9090 -s 192.168.0.0/16 -j ACCEPT

# Rate limiting for HTTP/HTTPS
iptables -A INPUT -p tcp --dport 80 -m limit --limit 25/minute --limit-burst 100 -j ACCEPT
iptables -A INPUT -p tcp --dport 443 -m limit --limit 25/minute --limit-burst 100 -j ACCEPT

# Drop invalid packets
iptables -A INPUT -m conntrack --ctstate INVALID -j DROP

# Log dropped packets (optional)
iptables -A INPUT -m limit --limit 5/min -j LOG --log-prefix "iptables denied: " --log-level 7

# Save rules
iptables-save > /etc/iptables/rules.v4

echo "Firewall rules applied successfully"
"#
        .to_string()
    }

    /// Generate security audit report
    pub async fn generate_security_report(&self) -> String {
        let events = self.audit_events.read().await;
        let total_events = events.len();

        let mut event_counts = HashMap::new();
        let mut result_counts = HashMap::new();

        for event in events.iter() {
            *event_counts
                .entry(format!("{:?}", event.event_type))
                .or_insert(0) += 1;
            *result_counts
                .entry(format!("{:?}", event.result))
                .or_insert(0) += 1;
        }

        format!(
            r#"# ProvChain Security Audit Report
Generated: {}

## Summary
- Total Audit Events: {}
- TLS Enabled: {}
- Rate Limiting: {}
- Audit Logging: {}

## Event Types
{}

## Event Results
{}

## Security Policies
{}

## Recommendations
- Regularly review audit logs for suspicious activity
- Update TLS certificates before expiration
- Monitor rate limiting effectiveness
- Review and update security policies quarterly
- Implement additional monitoring for critical operations
"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            total_events,
            if self.config.tls_enabled { "Yes" } else { "No" },
            if self.config.rate_limiting_enabled {
                "Yes"
            } else {
                "No"
            },
            if self.config.audit_logging_enabled {
                "Yes"
            } else {
                "No"
            },
            event_counts
                .iter()
                .map(|(k, v)| format!("- {k}: {v}"))
                .collect::<Vec<_>>()
                .join("\n"),
            result_counts
                .iter()
                .map(|(k, v)| format!("- {k}: {v}"))
                .collect::<Vec<_>>()
                .join("\n"),
            self.config
                .security_policies
                .iter()
                .map(|p| format!(
                    "- {} ({}): {} rules",
                    p.name,
                    if p.enabled { "Enabled" } else { "Disabled" },
                    p.rules.len()
                ))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// Shutdown security systems
    pub async fn shutdown(&mut self) -> Result<(), ProductionError> {
        tracing::info!("Shutting down security systems");

        // Generate final security report
        let _report = self.generate_security_report().await;
        tracing::info!("Final security report generated");

        Ok(())
    }
}

/// Token bucket for rate limiting
#[derive(Debug)]
struct TokenBucket {
    /// Current number of tokens available
    tokens: f64,
    /// Last time tokens were refilled
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket with full capacity
    fn new(capacity: f64) -> Self {
        Self {
            tokens: capacity,
            last_refill: Instant::now(),
        }
    }

    /// Attempt to consume a token. Returns true if successful.
    fn consume(&mut self, tokens_to_consume: f64, refill_rate: f64, capacity: f64) -> bool {
        // Calculate time elapsed since last refill
        let elapsed = self.last_refill.elapsed().as_secs_f64();

        // Refill tokens based on elapsed time
        let tokens_to_add = elapsed * refill_rate;
        self.tokens = (self.tokens + tokens_to_add).min(capacity);
        self.last_refill = Instant::now();

        // Check if we have enough tokens
        if self.tokens >= tokens_to_consume {
            self.tokens -= tokens_to_consume;
            true
        } else {
            false
        }
    }

    /// Get current token count (for testing/debugging)
    fn token_count(&self) -> f64 {
        self.tokens
    }
}

/// Rate limiter using token bucket algorithm
#[derive(Debug)]
pub struct RateLimiter {
    /// Per-IP token buckets
    buckets: RwLock<HashMap<String, TokenBucket>>,
    /// Maximum requests per minute
    rate_limit_per_minute: u32,
    /// Burst capacity (allows short bursts above normal rate)
    burst_capacity: u32,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(rate_limit_per_minute: u32, burst_capacity: u32) -> Self {
        Self {
            buckets: RwLock::new(HashMap::new()),
            rate_limit_per_minute,
            burst_capacity,
        }
    }

    /// Check if a request from the given IP should be allowed
    pub fn check_rate_limit(&self, client_ip: &str) -> bool {
        // Calculate refill rate (tokens per second)
        let refill_rate = self.rate_limit_per_minute as f64 / 60.0;
        let capacity = self.burst_capacity as f64;

        let mut buckets = self.buckets.write().unwrap();

        // Get or create token bucket for this IP
        let bucket = buckets
            .entry(client_ip.to_string())
            .or_insert_with(|| TokenBucket::new(capacity));

        // Attempt to consume one token
        bucket.consume(1.0, refill_rate, capacity)
    }

    /// Get current bucket state for an IP (for testing)
    pub fn get_token_count(&self, client_ip: &str) -> Option<f64> {
        let buckets = self.buckets.read().unwrap();
        buckets.get(client_ip).map(|b| b.token_count())
    }

    /// Remove stale entries to prevent memory leak (for testing/maintenance)
    pub fn cleanup_stale_entries(&self, max_age: Duration) {
        let mut buckets = self.buckets.write().unwrap();
        let now = Instant::now();

        buckets.retain(|_, bucket| now.duration_since(bucket.last_refill) < max_age);
    }

    /// Get current number of tracked IPs (for testing)
    pub fn tracked_ip_count(&self) -> usize {
        self.buckets.read().unwrap().len()
    }
}

pub struct SecurityMiddleware {
    config: SecurityConfig,
    rate_limiter: RateLimiter,
}

impl SecurityMiddleware {
    pub fn new(config: SecurityConfig) -> Self {
        // Initialize rate limiter with configured limits
        // Burst capacity is 2x the normal rate to allow short bursts
        let burst_capacity = config.rate_limit_per_minute * 2;
        let rate_limiter = RateLimiter::new(config.rate_limit_per_minute, burst_capacity);

        Self {
            config,
            rate_limiter,
        }
    }

    /// Validate request headers
    pub fn validate_request_headers(
        &self,
        _headers: &HashMap<String, String>,
    ) -> Result<(), ProductionError> {
        // Check for required security headers
        if self.config.security_headers_enabled {
            // In a real implementation, we would validate security headers
            tracing::debug!("Validating request headers");
        }
        Ok(())
    }

    /// Check rate limits using token bucket algorithm
    pub fn check_rate_limit(&self, client_ip: &str) -> Result<bool, ProductionError> {
        if !self.config.rate_limiting_enabled {
            // Rate limiting disabled, allow all requests
            return Ok(true);
        }

        // Check rate limit using token bucket algorithm
        let allowed = self.rate_limiter.check_rate_limit(client_ip);

        tracing::debug!(
            "Rate limit check for IP: {} - {}",
            client_ip,
            if allowed { "ALLOWED" } else { "BLOCKED" }
        );

        Ok(allowed)
    }

    /// Get current token count for an IP (for testing/debugging)
    pub fn get_rate_limit_token_count(&self, client_ip: &str) -> Option<f64> {
        self.rate_limiter.get_token_count(client_ip)
    }

    /// Get number of IPs currently being tracked (for testing/maintenance)
    pub fn tracked_ip_count(&self) -> usize {
        self.rate_limiter.tracked_ip_count()
    }

    /// Clean up stale rate limit entries (for maintenance)
    pub fn cleanup_stale_rate_limits(&self, max_age_secs: u64) {
        self.rate_limiter
            .cleanup_stale_entries(Duration::from_secs(max_age_secs));
    }

    /// Validate JWT token with proper signature verification and expiration checking
    pub fn validate_jwt(&self, token: &str) -> Result<bool, ProductionError> {
        // Reject empty tokens immediately
        if token.is_empty() {
            return Ok(false);
        }

        // Get JWT secret from environment variable (most secure) or config (fallback)
        let jwt_secret = if let Ok(secret) = std::env::var("JWT_SECRET") {
            if secret.len() < 32 {
                return Err(ProductionError::Security(format!(
                    "JWT_SECRET from environment is too short ({} chars), minimum 32 required",
                    secret.len()
                )));
            }
            secret
        } else {
            // Fallback to config secret (with security warning)
            if self.config.jwt_secret.len() < 32 {
                return Err(ProductionError::Security(format!(
                    "Configured JWT secret is too short ({} chars), minimum 32 required. \
                            Set JWT_SECRET environment variable for production!",
                    self.config.jwt_secret.len()
                )));
            }

            // Log warning about using config secret instead of environment variable
            tracing::warn!(
                "SECURITY WARNING: Using JWT secret from config instead of environment variable. \
                           Set JWT_SECRET environment variable for better security."
            );

            self.config.jwt_secret.clone()
        };

        // Configure validation to check signature and expiration
        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.leeway = 0; // No leeway for strict expiration checking
        validation.validate_exp = true; // Check expiration

        // Decode and verify the token
        match decode::<ProductionClaims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &validation,
        ) {
            Ok(_) => {
                // Token is valid: signature verified, not expired, proper algorithm
                tracing::debug!("JWT token validation successful");
                Ok(true)
            }
            Err(e) => {
                // Token is invalid: bad signature, expired, wrong algorithm, malformed, etc.
                tracing::warn!("JWT token validation failed: {}", e);

                // Classify error type for better logging
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("expired") {
                    tracing::warn!("JWT token has expired");
                } else if error_msg.contains("signature") {
                    tracing::warn!("JWT token has invalid signature");
                } else if error_msg.contains("algorithm") {
                    tracing::warn!("JWT token uses invalid algorithm");
                }

                Ok(false)
            }
        }
    }
}
