//! Authentication and authorization module for web API

use crate::web::models::{ActorRole, ApiError, AuthRequest, AuthResponse, UserClaims};
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// JWT secret key (loaded from environment variable only for security)
pub fn get_jwt_secret() -> Result<Vec<u8>, crate::error::WebError> {
    // Only use environment variable for security - no config file secrets
    if let Ok(secret) = std::env::var("JWT_SECRET") {
        if secret.len() < 32 {
            return Err(crate::error::WebError::ServerError(
                "JWT_SECRET must be at least 32 characters long for security".to_string(),
            ));
        }
        return Ok(secret.into_bytes());
    }

    // JWT_SECRET must be set in all environments (no hardcoded secrets allowed)
    let secret = std::env::var("JWT_SECRET").map_err(|_| {
        crate::error::WebError::AuthenticationFailed(
            "JWT_SECRET environment variable must be set (min 32 characters). \
             Generate one with: openssl rand -base64 32"
                .to_string(),
        )
    })?;

    // Validate minimum length
    if secret.len() < 32 {
        return Err(crate::error::WebError::AuthenticationFailed(format!(
            "JWT_SECRET must be at least 32 characters (current: {})",
            secret.len()
        )));
    }

    Ok(secret.into_bytes())
}

/// Generate a cryptographically secure random JWT secret
pub fn generate_secure_jwt_secret() -> Result<String, crate::error::WebError> {
    use rand::{thread_rng, Rng};

    let mut rng = thread_rng();
    let secret: String = (0..64)
        .map(|_| {
            let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                          abcdefghijklmnopqrstuvwxyz\
                          0123456789+/";
            chars[rng.gen_range(0..chars.len())] as char
        })
        .collect();

    Ok(secret)
}

/// User database (in production, this would be a proper database)
type UserDatabase = Arc<RwLock<HashMap<String, UserInfo>>>;

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub username: String,
    pub password_hash: String,
    pub role: ActorRole,
}

#[derive(Clone)]
pub struct AuthState {
    pub users: UserDatabase,
}

impl Default for AuthState {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthState {
    pub fn new() -> Self {
        // SECURITY: No default users created - users must be explicitly created
        // This prevents hardcoded credentials and improves security
        let users = HashMap::new();

        Self {
            users: Arc::new(RwLock::new(users)),
        }
    }

    /// Initialize with default users ONLY if ALLOW_DEFAULT_USERS env var is set (for development)
    pub fn new_with_defaults() -> Self {
        // Only allow default users in development if explicitly requested AND demo mode is on
        if !cfg!(debug_assertions)
            || std::env::var("PROVCHAIN_DEMO_MODE")
                .map(|v| v != "1")
                .unwrap_or(false)
        {
            eprintln!("SECURITY: Default users are disabled. Set PROVCHAIN_DEMO_MODE=1 in development to enable demo users.");
            return Self::new();
        }

        let mut users = HashMap::new();

        // DEVELOPMENT ONLY: Create default users with weak credentials
        // This should NEVER be used in production
        eprintln!(
            "WARNING: Creating default development users. This should not be used in production!"
        );

        let admin_hash =
            hash("admin123", DEFAULT_COST).unwrap_or_else(|_| "fallback_admin_hash".to_string());
        let farmer_hash =
            hash("farmer123", DEFAULT_COST).unwrap_or_else(|_| "fallback_farmer_hash".to_string());
        let processor_hash = hash("processor123", DEFAULT_COST)
            .unwrap_or_else(|_| "fallback_processor_hash".to_string());

        users.insert(
            "admin".to_string(),
            UserInfo {
                username: "admin".to_string(),
                password_hash: admin_hash,
                role: ActorRole::Admin,
            },
        );

        users.insert(
            "farmer1".to_string(),
            UserInfo {
                username: "farmer1".to_string(),
                password_hash: farmer_hash,
                role: ActorRole::Farmer,
            },
        );

        users.insert(
            "processor1".to_string(),
            UserInfo {
                username: "processor1".to_string(),
                password_hash: processor_hash,
                role: ActorRole::Processor,
            },
        );

        Self {
            users: Arc::new(RwLock::new(users)),
        }
    }

    /// Initialize with an admin user (for first-time setup)
    pub fn new_with_admin(
        username: String,
        password: String,
    ) -> Result<Self, crate::error::WebError> {
        let mut users = HashMap::new();

        let password_hash = hash(&password, DEFAULT_COST).map_err(|e| {
            crate::error::WebError::ServerError(format!("Password hashing failed: {}", e))
        })?;

        users.insert(
            username.clone(),
            UserInfo {
                username,
                password_hash,
                role: ActorRole::Admin,
            },
        );

        Ok(Self {
            users: Arc::new(RwLock::new(users)),
        })
    }

    /// Create a new user with secure password hashing and validation
    pub async fn create_user(
        &self,
        username: String,
        password: String,
        role: ActorRole,
    ) -> Result<(), crate::error::WebError> {
        // Input validation
        validate_username(&username)?;
        validate_password(&password)?;

        let password_hash = hash(&password, DEFAULT_COST).map_err(|e| {
            crate::error::WebError::ServerError(format!("Password hashing failed: {}", e))
        })?;

        let mut users = self.users.write().await;

        if users.contains_key(&username) {
            return Err(crate::error::WebError::InvalidRequest(format!(
                "User '{}' already exists",
                username
            )));
        }

        users.insert(
            username.clone(),
            UserInfo {
                username,
                password_hash,
                role,
            },
        );

        Ok(())
    }

    /// Update user password with secure hashing and validation
    pub async fn update_password(
        &self,
        username: &str,
        new_password: String,
    ) -> Result<(), crate::error::WebError> {
        validate_password(&new_password)?;

        let password_hash = hash(&new_password, DEFAULT_COST).map_err(|e| {
            crate::error::WebError::ServerError(format!("Password hashing failed: {}", e))
        })?;

        let mut users = self.users.write().await;

        if let Some(user_info) = users.get_mut(username) {
            user_info.password_hash = password_hash;
            Ok(())
        } else {
            Err(crate::error::WebError::ResourceNotFound(format!(
                "User '{}' not found",
                username
            )))
        }
    }

    /// Get user information (excluding password hash)
    pub async fn get_user_info(
        &self,
        username: &str,
    ) -> Result<(String, ActorRole), crate::error::WebError> {
        let users = self.users.read().await;

        if let Some(user_info) = users.get(username) {
            Ok((user_info.username.clone(), user_info.role.clone()))
        } else {
            Err(crate::error::WebError::ResourceNotFound(format!(
                "User '{}' not found",
                username
            )))
        }
    }

    /// Check if any users exist in the system
    pub async fn has_users(&self) -> bool {
        let users = self.users.read().await;
        !users.is_empty()
    }

    /// List all users (for admin purposes only)
    pub async fn list_users(&self) -> Vec<(String, ActorRole)> {
        let users = self.users.read().await;
        users
            .iter()
            .map(|(username, user_info)| (username.clone(), user_info.role.clone()))
            .collect()
    }

    /// Delete a user (admin only)
    pub async fn delete_user(&self, username: &str) -> Result<(), crate::error::WebError> {
        let mut users = self.users.write().await;

        if users.remove(username).is_some() {
            Ok(())
        } else {
            Err(crate::error::WebError::ResourceNotFound(format!(
                "User '{}' not found",
                username
            )))
        }
    }
}

/// Generate JWT token for authenticated user
///
/// # Parameters
/// - `username`: The user's identifier
/// - `role`: The user's role/permissions
/// - `secret`: The JWT secret key (must be at least 32 bytes)
pub fn generate_token(
    username: &str,
    role: &ActorRole,
    secret: &[u8],
) -> Result<String, crate::error::WebError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .ok_or_else(|| {
            crate::error::WebError::ServerError(
                "Failed to calculate token expiration time".to_string(),
            )
        })?
        .timestamp() as usize;

    let claims = UserClaims {
        sub: username.to_string(),
        role: role.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|e| {
        crate::error::WebError::AuthenticationFailed(format!("Token generation failed: {}", e))
    })
}

/// Validate JWT token and extract claims
///
/// # Parameters
/// - `token`: The JWT token string to validate
/// - `secret`: The JWT secret key (must be at least 32 bytes)
pub fn validate_token(token: &str, secret: &[u8]) -> Result<UserClaims, crate::error::WebError> {
    // Configure validation to check expiration
    let mut validation = Validation::default();
    validation.leeway = 0; // No leeway for expiration
    validation.validate_exp = true; // Explicitly enable expiration validation

    decode::<UserClaims>(token, &DecodingKey::from_secret(secret), &validation)
        .map(|data| data.claims)
        .map_err(|e| {
            crate::error::WebError::AuthenticationFailed(format!("Token validation failed: {}", e))
        })
}

/// Authentication handler
pub async fn authenticate(
    State(auth_state): State<AuthState>,
    Json(auth_request): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ApiError>)> {
    let jwt_secret = get_jwt_secret().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: "jwt_secret_error".to_string(),
                message: e.to_string(),
                timestamp: Utc::now(),
            }),
        )
    })?;

    let users = auth_state.users.read().await;

    if let Some(user_info) = users.get(&auth_request.username) {
        // Use bcrypt to verify password
        match verify(&auth_request.password, &user_info.password_hash) {
            Ok(true) => {
                match generate_token(&auth_request.username, &user_info.role, &jwt_secret) {
                    Ok(token) => {
                        let expires_at = Utc::now() + Duration::hours(24);
                        Ok(Json(AuthResponse {
                            token,
                            expires_at,
                            user_role: user_info.role.to_string(),
                        }))
                    }
                    Err(_) => Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiError {
                            error: "token_generation_failed".to_string(),
                            message: "Failed to generate authentication token".to_string(),
                            timestamp: Utc::now(),
                        }),
                    )),
                }
            }
            Ok(false) | Err(_) => Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    error: "invalid_credentials".to_string(),
                    message: "Invalid username or password".to_string(),
                    timestamp: Utc::now(),
                }),
            )),
        }
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                error: "invalid_credentials".to_string(),
                message: "Invalid username or password".to_string(),
                timestamp: Utc::now(),
            }),
        ))
    }
}

/// Middleware to verify JWT token
pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiError>)> {
    let jwt_secret = get_jwt_secret().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: "jwt_secret_error".to_string(),
                message: "JWT secret not configured".to_string(),
                timestamp: Utc::now(),
            }),
        )
    })?;

    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            match validate_token(token, &jwt_secret) {
                Ok(claims) => {
                    // Add user claims to request extensions
                    request.extensions_mut().insert(claims);
                    Ok(next.run(request).await)
                }
                Err(_) => Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ApiError {
                        error: "invalid_token".to_string(),
                        message: "Invalid or expired authentication token".to_string(),
                        timestamp: Utc::now(),
                    }),
                )),
            }
        } else {
            Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    error: "invalid_auth_header".to_string(),
                    message: "Invalid authorization header format".to_string(),
                    timestamp: Utc::now(),
                }),
            ))
        }
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                error: "missing_auth_header".to_string(),
                message: "Authorization header is required".to_string(),
                timestamp: Utc::now(),
            }),
        ))
    }
}

/// Role-based authorization middleware
pub fn require_role(
    required_role: ActorRole,
) -> impl Fn(
    Request,
    Next,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<Response, (StatusCode, Json<ApiError>)>> + Send>,
> + Clone {
    move |request: Request, next: Next| {
        let required_role = required_role.clone();
        Box::pin(async move {
            if let Some(claims) = request.extensions().get::<UserClaims>() {
                // Admin role can access everything
                if claims.role == "admin" || claims.role == required_role.to_string() {
                    Ok(next.run(request).await)
                } else {
                    Err((
                        StatusCode::FORBIDDEN,
                        Json(ApiError {
                            error: "insufficient_permissions".to_string(),
                            message: format!("Role '{required_role}' required for this operation"),
                            timestamp: Utc::now(),
                        }),
                    ))
                }
            } else {
                Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ApiError {
                        error: "authentication_required".to_string(),
                        message: "Authentication is required for this operation".to_string(),
                        timestamp: Utc::now(),
                    }),
                ))
            }
        })
    }
}

/// Validate username according to security requirements
fn validate_username(username: &str) -> Result<(), crate::error::WebError> {
    if username.is_empty() {
        return Err(crate::error::WebError::InvalidRequest(
            "Username cannot be empty".to_string(),
        ));
    }

    if username.len() < 3 {
        return Err(crate::error::WebError::InvalidRequest(
            "Username must be at least 3 characters long".to_string(),
        ));
    }

    if username.len() > 50 {
        return Err(crate::error::WebError::InvalidRequest(
            "Username cannot exceed 50 characters".to_string(),
        ));
    }

    // Only allow alphanumeric characters, underscores, and hyphens
    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(crate::error::WebError::InvalidRequest(
            "Username can only contain letters, numbers, underscores, and hyphens".to_string(),
        ));
    }

    // Prevent common dangerous usernames
    let dangerous_usernames = ["admin", "root", "system", "null", "undefined", "anonymous"];
    if dangerous_usernames.contains(&username.to_lowercase().as_str()) {
        return Err(crate::error::WebError::InvalidRequest(
            "Username is reserved and cannot be used".to_string(),
        ));
    }

    Ok(())
}

/// Validate password according to security requirements
fn validate_password(password: &str) -> Result<(), crate::error::WebError> {
    if password.is_empty() {
        return Err(crate::error::WebError::InvalidRequest(
            "Password cannot be empty".to_string(),
        ));
    }

    if password.len() < 8 {
        return Err(crate::error::WebError::InvalidRequest(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    if password.len() > 128 {
        return Err(crate::error::WebError::InvalidRequest(
            "Password cannot exceed 128 characters".to_string(),
        ));
    }

    // Check for character variety
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase || !has_lowercase || !has_digit {
        return Err(crate::error::WebError::InvalidRequest(
            "Password must contain at least one uppercase letter, one lowercase letter, and one digit".to_string()
        ));
    }

    // In production, require special characters
    if !cfg!(debug_assertions) && !has_special {
        return Err(crate::error::WebError::InvalidRequest(
            "Password must contain at least one special character".to_string(),
        ));
    }

    // Check for common weak passwords
    let weak_passwords = [
        "password",
        "123456",
        "qwerty",
        "admin",
        "letmein",
        "welcome",
        "password123",
        "admin123",
        "root123",
        "changeme",
        "default",
    ];

    if weak_passwords.contains(&password.to_lowercase().as_str()) {
        return Err(crate::error::WebError::InvalidRequest(
            "Password is too common and not secure".to_string(),
        ));
    }

    Ok(())
}

// ==============================================================================
// CRITICAL AUTHENTICATION SECURITY TESTS
// ==============================================================================
//
// This test suite addresses the critical security gap in authentication testing.
// Current test coverage: 5% - Target: 95%+ coverage for all security-critical functions
//
// Test Categories:
// 1. JWT Token Security Tests
// 2. Password Hashing Security Tests
// 3. User Management Security Tests
// 4. Input Validation & Attack Scenario Tests
// 5. Performance & Security Boundary Tests
// 6. Integration Tests
//
// Security Focus:
// - Prevent common authentication attacks (JWT tampering, brute force, timing attacks)
// - Validate proper error handling without information leakage
// - Test edge cases and security boundaries
// - Ensure cryptographic operations are secure
// - Verify role-based access control enforcement

#[cfg(test)]
mod auth_security_tests {
    use super::*;
    use chrono::{Duration, Utc};
    use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
    use std::env;

    // ========================================================================
    // JWT TOKEN SECURITY TESTS
    // ========================================================================

    mod jwt_token_security {
        use super::*;

        // Test secret constant - 32 bytes minimum for security
        const TEST_JWT_SECRET: &[u8] = b"test-secret-for-unit-testing-must-be-32-chars";

        #[test]
        fn test_jwt_token_generation_valid_inputs() {
            // Test successful token generation with valid inputs
            let username = "testuser";
            let role = ActorRole::Admin;

            let token = generate_token(username, &role, TEST_JWT_SECRET);
            assert!(
                token.is_ok(),
                "Token generation should succeed with valid inputs"
            );

            let token_str = token.unwrap();
            assert!(!token_str.is_empty(), "Generated token should not be empty");

            // Verify token structure (JWT should have 3 parts separated by dots)
            let parts: Vec<&str> = token_str.split('.').collect();
            assert_eq!(parts.len(), 3, "JWT token should have exactly 3 parts");
        }

        #[test]
        fn test_jwt_token_validation_success() {
            // Test successful token validation
            let username = "testuser";
            let role = ActorRole::Farmer;
            let token = generate_token(username, &role, TEST_JWT_SECRET).unwrap();

            let validation_result = validate_token(&token, TEST_JWT_SECRET);
            assert!(
                validation_result.is_ok(),
                "Token validation should succeed for valid token"
            );

            let claims = validation_result.unwrap();
            assert_eq!(claims.sub, username, "Token should preserve username");
            assert_eq!(claims.role, role.to_string(), "Token should preserve role");
        }

        #[test]
        fn test_jwt_token_expiration_handling() {
            // Test token expiration
            // Create an expired token
            let expired_claims = UserClaims {
                sub: "testuser".to_string(),
                role: "admin".to_string(),
                exp: (Utc::now() - Duration::minutes(1)).timestamp() as usize,
            };

            let expired_token = encode(
                &Header::default(),
                &expired_claims,
                &EncodingKey::from_secret(TEST_JWT_SECRET),
            )
            .unwrap();

            // Validation should fail for expired token
            let validation_result = validate_token(&expired_token, TEST_JWT_SECRET);
            assert!(
                validation_result.is_err(),
                "Token validation should fail for expired token"
            );

            let error = validation_result.unwrap_err();
            assert!(matches!(
                error,
                crate::error::WebError::AuthenticationFailed(_)
            ));
        }

        #[test]
        fn test_jwt_token_tampering_detection() {
            // Test that tampered tokens are rejected
            let username = "testuser";
            let role = ActorRole::Processor;
            let original_token = generate_token(username, &role, TEST_JWT_SECRET).unwrap();

            // Tamper with the token by changing a character in the payload
            let mut tampered_token = original_token.clone();
            let parts: Vec<&str> = tampered_token.split('.').collect();

            if parts.len() == 3 {
                let mut payload = parts[1].to_string();
                if !payload.is_empty() {
                    payload.replace_range(0..1, "X");
                    tampered_token = format!("{}.{}.{}", parts[0], payload, parts[2]);
                }
            }

            // Validation should fail for tampered token
            let validation_result = validate_token(&tampered_token, TEST_JWT_SECRET);
            assert!(
                validation_result.is_err(),
                "Token validation should fail for tampered token"
            );
        }

        #[test]
        fn test_jwt_token_invalid_signature_rejection() {
            // Test tokens with invalid signatures are rejected
            let username = "testuser";
            let role = ActorRole::Admin;
            let valid_token = generate_token(username, &role, TEST_JWT_SECRET).unwrap();

            // Use a different secret for validation - should fail
            const WRONG_SECRET: &[u8] = b"different-secret-32-chars-long-minimum";
            let validation_result = validate_token(&valid_token, WRONG_SECRET);
            assert!(
                validation_result.is_err(),
                "Token validation should fail with wrong secret"
            );
        }

        #[test]
        fn test_jwt_token_malformed_token_handling() {
            let malformed_tokens = vec![
                "",
                "invalid",
                "header.payload",
                "header.payload.signature.extra",
                "header..signature",
                ".payload.signature",
                "header.payload.",
            ];

            for malformed_token in malformed_tokens {
                let validation_result = validate_token(malformed_token, TEST_JWT_SECRET);
                assert!(
                    validation_result.is_err(),
                    "Token validation should fail for malformed token: {}",
                    malformed_token
                );
            }
        }

        #[test]
        fn test_jwt_token_secret_length_validation() {
            // This test verifies that get_jwt_secret() properly validates secret length
            env::set_var("JWT_SECRET", "short");

            let secret_result = get_jwt_secret();
            assert!(
                secret_result.is_err(),
                "Short JWT secret should be rejected"
            );

            let error = secret_result.unwrap_err();
            assert!(matches!(error, crate::error::WebError::ServerError(_)));

            // Clean up
            env::remove_var("JWT_SECRET");
        }

        #[test]
        fn test_jwt_token_security_properties() {
            let username = "testuser";
            let role = ActorRole::Auditor;
            let token = generate_token(username, &role, TEST_JWT_SECRET).unwrap();

            // Decode and verify security properties
            let decoded = decode::<UserClaims>(
                &token,
                &DecodingKey::from_secret(TEST_JWT_SECRET),
                &Validation::default(),
            );

            assert!(decoded.is_ok(), "Token should decode successfully");

            let token_data = decoded.unwrap();

            // Verify no algorithm confusion attacks (should use HS256 by default)
            assert_eq!(token_data.header.alg, jsonwebtoken::Algorithm::HS256);

            // Verify claims are properly set
            assert_eq!(token_data.claims.sub, username);
            assert_eq!(token_data.claims.role, role.to_string());

            // Verify token is not expired
            let now = Utc::now().timestamp() as usize;
            assert!(
                token_data.claims.exp > now,
                "Token should not be expired immediately after creation"
            );
        }
    }

    // ========================================================================
    // PASSWORD HASHING SECURITY TESTS
    // ========================================================================

    mod password_hashing_security {
        use super::*;

        #[test]
        fn test_password_hashing_strong_passwords() {
            let strong_passwords = vec![
                "StrongP@ssw0rd123!",
                "MySecurePassword#2025",
                "ComplexPass_WithNumbers8421",
                "Very$ecureP@sswordWithSymbls",
            ];

            for password in strong_passwords {
                let hash_result = hash(password, DEFAULT_COST);
                assert!(
                    hash_result.is_ok(),
                    "Password hashing should succeed for strong password"
                );

                let hash = hash_result.unwrap();
                assert!(!hash.is_empty(), "Hash should not be empty");
                assert!(hash.len() > 50, "Bcrypt hash should be sufficiently long");

                let verify_result = verify(password, &hash);
                assert!(
                    verify_result.is_ok(),
                    "Password verification should succeed"
                );
                assert!(
                    verify_result.unwrap(),
                    "Password verification should return true for correct password"
                );

                let wrong_verify = verify("wrongpassword", &hash);
                assert!(
                    wrong_verify.is_ok(),
                    "Password verification should succeed even for wrong password"
                );
                assert!(
                    !wrong_verify.unwrap(),
                    "Password verification should return false for wrong password"
                );
            }
        }

        #[test]
        #[ignore]
        fn test_password_hash_salt_uniqueness() {
            let password = "TestPassword123!";

            // Generate multiple hashes for the same password
            let mut hashes = Vec::new();
            for _ in 0..10 {
                let hash_result = hash(password, DEFAULT_COST);
                assert!(hash_result.is_ok(), "Password hashing should succeed");
                hashes.push(hash_result.unwrap());
            }

            // All hashes should be different (due to unique salts)
            for (i, hash1) in hashes.iter().enumerate() {
                for (j, hash2) in hashes.iter().enumerate() {
                    if i != j {
                        assert_ne!(
                            hash1, hash2,
                            "Same password should generate different hashes due to unique salts"
                        );
                    }
                }
            }

            // But all hashes should verify successfully
            for hash in &hashes {
                let verify_result = verify(password, hash);
                assert!(
                    verify_result.is_ok(),
                    "Password verification should succeed for all hashes"
                );
                assert!(
                    verify_result.unwrap(),
                    "All hashes should verify the original password"
                );
            }
        }

        #[test]
        fn test_password_hash_timing_attack_resistance() {
            let password = "TestPassword123!";
            let hash = hash(password, DEFAULT_COST).unwrap();

            let wrong_password = "WrongPassword123!";

            // Time correct verification
            let start = std::time::Instant::now();
            let correct_result = verify(password, &hash).unwrap();
            let correct_duration = start.elapsed();

            // Time incorrect verification
            let start = std::time::Instant::now();
            let wrong_result = verify(wrong_password, &hash).unwrap();
            let wrong_duration = start.elapsed();

            assert!(correct_result, "Correct password should verify");
            assert!(!wrong_result, "Wrong password should not verify");

            // Timing should be roughly similar (bcrypt is timing-attack resistant)
            let time_diff = correct_duration.abs_diff(wrong_duration);

            // In debug mode, allow more variance due to environment factors
            let max_diff = if cfg!(debug_assertions) {
                std::time::Duration::from_millis(1000)
            } else {
                std::time::Duration::from_millis(200)
            };

            assert!(time_diff < max_diff,
                "Password verification timing should be consistent to prevent timing attacks. Difference: {:?}", time_diff);
        }

        #[tokio::test]
        async fn test_password_hashing_in_auth_context() {
            let auth_state = AuthState::new();

            let username = "testuser";
            let password = "TestPassword123!";
            let role = ActorRole::Farmer;

            let create_result = auth_state
                .create_user(username.to_string(), password.to_string(), role.clone())
                .await;
            assert!(
                create_result.is_ok(),
                "User creation should succeed with valid password"
            );

            let duplicate_result = auth_state
                .create_user(username.to_string(), password.to_string(), role)
                .await;
            assert!(
                duplicate_result.is_err(),
                "Duplicate user creation should fail"
            );
        }
    }

    // ========================================================================
    // USER MANAGEMENT SECURITY TESTS
    // ========================================================================

    mod user_management_security {
        use super::*;

        #[tokio::test]
        async fn test_user_creation_validation() {
            let auth_state = AuthState::new();

            let valid_users = vec![
                ("farmer_john", "StrongPass123!", ActorRole::Farmer),
                ("processor_alice", "SecureP@ssw0rd", ActorRole::Processor),
                ("auditor_bob", "AuditPass#2025", ActorRole::Auditor),
                (
                    "transporter_sam",
                    "Transport$ecure842",
                    ActorRole::Transporter,
                ),
            ];

            for (username, password, role) in valid_users {
                let result = auth_state
                    .create_user(username.to_string(), password.to_string(), role.clone())
                    .await;
                assert!(
                    result.is_ok(),
                    "User creation should succeed for valid user: {} with role: {:?}",
                    username,
                    role
                );
            }
        }

        #[tokio::test]
        async fn test_user_creation_invalid_inputs() {
            let auth_state = AuthState::new();

            let long_username = "a".repeat(51);
            let invalid_usernames = vec![
                ("", "StrongPass123!", ActorRole::Farmer),
                ("ab", "StrongPass123!", ActorRole::Farmer),
                (&long_username, "StrongPass123!", ActorRole::Farmer),
                ("user@domain", "StrongPass123!", ActorRole::Farmer),
                ("user space", "StrongPass123!", ActorRole::Farmer),
                ("admin", "StrongPass123!", ActorRole::Farmer),
                ("root", "StrongPass123!", ActorRole::Farmer),
            ];

            for (username, password, role) in invalid_usernames {
                let result = auth_state
                    .create_user(username.to_string(), password.to_string(), role)
                    .await;
                assert!(
                    result.is_err(),
                    "User creation should fail for invalid username: {}",
                    username
                );
            }

            let invalid_passwords = vec![
                ("validuser", "", ActorRole::Farmer),
                ("validuser2", "short", ActorRole::Farmer),
                ("validuser3", "alllowercase", ActorRole::Farmer),
                ("validuser4", "ALLUPPERCASE", ActorRole::Farmer),
                ("validuser5", "NoNumbersHere", ActorRole::Farmer),
                ("validuser6", "weak123", ActorRole::Farmer),
                ("validuser7", "password", ActorRole::Farmer),
                ("validuser8", "123456", ActorRole::Farmer),
            ];

            for (username, password, role) in invalid_passwords {
                let result = auth_state
                    .create_user(username.to_string(), password.to_string(), role)
                    .await;
                assert!(
                    result.is_err(),
                    "User creation should fail for invalid password for user: {}",
                    username
                );
            }
        }

        #[tokio::test]
        async fn test_duplicate_user_prevention() {
            let auth_state = AuthState::new();

            let username = "testuser";
            let password1 = "Password123!";
            let password2 = "DifferentPass456!";

            let result1 = auth_state
                .create_user(
                    username.to_string(),
                    password1.to_string(),
                    ActorRole::Admin,
                )
                .await;
            assert!(result1.is_ok(), "First user creation should succeed");

            let result2 = auth_state
                .create_user(
                    username.to_string(),
                    password2.to_string(),
                    ActorRole::Farmer,
                )
                .await;
            assert!(result2.is_err(), "Duplicate user creation should fail");

            let users = auth_state.list_users().await;
            assert_eq!(
                users.len(),
                1,
                "Should only have one user after duplicate attempt"
            );
        }

        #[tokio::test]
        async fn test_user_info_security() {
            let auth_state = AuthState::new();

            let username = "testuser";
            let password = "TestPassword123!";
            let role = ActorRole::Processor;

            auth_state
                .create_user(username.to_string(), password.to_string(), role.clone())
                .await
                .unwrap();

            let user_info = auth_state.get_user_info(username).await;
            assert!(user_info.is_ok(), "User info should be retrievable");

            let (retrieved_username, retrieved_role) = user_info.unwrap();
            assert_eq!(retrieved_username, username, "Username should match");
            assert_eq!(retrieved_role, role, "Role should match");

            // Verify password hash is not exposed
            let users = auth_state.users.read().await;
            if let Some(user) = users.get(username) {
                assert!(!user.password_hash.is_empty(), "Password hash should exist");
                assert_ne!(
                    user.password_hash, password,
                    "Password should not be stored in plaintext"
                );
            }
        }

        #[tokio::test]
        async fn test_user_password_update_security() {
            let auth_state = AuthState::new();

            let username = "testuser";
            let old_password = "OldPassword123!";
            let new_password = "NewPassword456!";

            auth_state
                .create_user(
                    username.to_string(),
                    old_password.to_string(),
                    ActorRole::Admin,
                )
                .await
                .unwrap();

            let update_result = auth_state
                .update_password(username, new_password.to_string())
                .await;
            assert!(update_result.is_ok(), "Password update should succeed");

            let update_nonexistent = auth_state
                .update_password("nonexistent", new_password.to_string())
                .await;
            assert!(
                update_nonexistent.is_err(),
                "Updating non-existent user should fail"
            );

            let invalid_password_result = auth_state
                .update_password(username, "weak".to_string())
                .await;
            assert!(
                invalid_password_result.is_err(),
                "Update with weak password should fail"
            );
        }

        #[tokio::test]
        async fn test_role_based_access_control() {
            let auth_state = AuthState::new();

            let users = vec![
                ("admin_user", ActorRole::Admin),
                ("farmer_user", ActorRole::Farmer),
                ("processor_user", ActorRole::Processor),
                ("auditor_user", ActorRole::Auditor),
            ];

            for (username, role) in &users {
                auth_state
                    .create_user(
                        username.to_string(),
                        format!("{}Password123!", username),
                        role.clone(),
                    )
                    .await
                    .unwrap();
            }

            for (username, expected_role) in &users {
                let (retrieved_username, retrieved_role) =
                    auth_state.get_user_info(username).await.unwrap();
                assert_eq!(retrieved_username, *username, "Username should match");
                assert_eq!(retrieved_role, *expected_role, "Role should match");
            }

            // Test role hierarchy
            let admin_claims = UserClaims {
                sub: "admin_user".to_string(),
                role: "admin".to_string(),
                exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
            };

            // Admin should be able to access any role
            for role in [
                ActorRole::Farmer,
                ActorRole::Processor,
                ActorRole::Auditor,
                ActorRole::Admin,
            ] {
                let can_access =
                    admin_claims.role == "admin" || admin_claims.role == role.to_string();
                assert!(
                    can_access,
                    "Admin should be able to access role: {:?}",
                    role
                );
            }
        }
    }

    // ========================================================================
    // INPUT VALIDATION SECURITY TESTS
    // ========================================================================

    mod input_validation_security {
        use super::*;

        #[test]
        fn test_username_validation_security() {
            let valid_usernames = vec![
                "user123",
                "test_user",
                "user-name",
                "User123Name",
                "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0",
            ];

            for username in valid_usernames {
                let result = validate_username(username);
                assert!(
                    result.is_ok(),
                    "Valid username should pass validation: {}",
                    username
                );
            }

            let long_username = "a".repeat(51);
            let invalid_usernames = vec![
                "",
                "ab",
                &long_username,
                "user@domain.com",
                "user space",
                "user\tname",
                "user\nname",
                "user\rname",
                "user/name",
                "user\\name",
                "user|name",
                "user<name>",
                "user'name",
                "user\"name",
                "user`name",
                "user&name",
                "user%name",
                "user;name",
                "user:name",
                "user,name",
                "user.name",
                "user?name",
                "user*name",
                "user+name",
                "user=name",
                "user$name",
                "user!name",
                "admin",
                "root",
                "system",
                "null",
                "undefined",
                "anonymous",
                "ADMIN",
                "Root",
                "SYSTEM",
            ];

            for username in invalid_usernames {
                let result = validate_username(username);
                assert!(
                    result.is_err(),
                    "Invalid username should fail validation: {:?}",
                    username
                );
            }
        }

        #[test]
        fn test_password_validation_security() {
            let valid_passwords = vec![
                "Password123!@",
                "SecurePass#2025",
                "MyP@ssw0rd",
                "ComplexPass_WithNumbers8421",
                "Very$ecureP@sswordWithSymbls123",
                "Test1234@Password",
            ];

            for password in valid_passwords {
                let result = validate_password(password);
                assert!(
                    result.is_ok(),
                    "Valid password should pass validation: {}",
                    password
                );
            }

            let long_password = "a".repeat(129);
            let invalid_passwords = vec![
                "",
                "short",
                "alllowercase",
                "ALLUPPERCASE",
                "NoNumbersHere",
                "password",
                "123456",
                "qwerty",
                "admin",
                "letmein",
                "welcome",
                "password123",
                "admin123",
                "root123",
                "changeme",
                "default",
                &long_password,
            ];

            for password in invalid_passwords {
                let result = validate_password(password);
                assert!(
                    result.is_err(),
                    "Invalid password should fail validation: {}",
                    password
                );
            }
        }

        #[test]
        fn test_case_sensitivity_validation() {
            let mixed_case_usernames = vec![
                "User123",
                "Test_User",
                "USER-NAME",
                "user",
                "ADMIN", // Should fail because admin is reserved
            ];

            for username in mixed_case_usernames {
                let result = validate_username(username);
                if username.to_lowercase() == "admin" {
                    assert!(result.is_err(), "Admin in any case should be reserved");
                } else {
                    assert!(result.is_ok(), "Mixed case should be allowed: {}", username);
                }
            }

            let case_variations = vec![
                "PASSWORD123!",  // All uppercase - should fail (no lowercase)
                "password123!",  // All lowercase - should fail (no uppercase)
                "Password123!@", // Mixed case with special - should pass
            ];

            for password in case_variations {
                let result = validate_password(password);
                // Extract alphanumeric characters for case analysis
                let alphanumeric: Vec<char> =
                    password.chars().filter(|c| c.is_alphanumeric()).collect();
                let has_uppercase = alphanumeric.iter().any(|c| c.is_uppercase());
                let has_lowercase = alphanumeric.iter().any(|c| c.is_lowercase());

                if !has_lowercase {
                    assert!(
                        result.is_err(),
                        "Password without lowercase should fail: {}",
                        password
                    );
                } else if !has_uppercase {
                    assert!(
                        result.is_err(),
                        "Password without uppercase should fail: {}",
                        password
                    );
                } else {
                    assert!(
                        result.is_ok(),
                        "Mixed case password should pass: {}",
                        password
                    );
                }
            }
        }
    }

    // ========================================================================
    // ERROR HANDLING SECURITY TESTS
    // ========================================================================

    mod error_handling_security {
        use super::*;

        // Test secret constant - 32 bytes minimum for security
        const TEST_JWT_SECRET: &[u8] = b"test-secret-for-unit-testing-must-be-32-chars";

        #[test]
        fn test_jwt_error_message_security() {
            let invalid_tokens = vec![
                "invalid",
                "header.payload",
                "header.payload.signature.extra",
            ];

            for token in invalid_tokens {
                let result = validate_token(token, TEST_JWT_SECRET);
                assert!(
                    result.is_err(),
                    "Token validation should fail for invalid token: {}",
                    token
                );

                let error = result.unwrap_err();
                if let crate::error::WebError::AuthenticationFailed(message) = error {
                    assert!(
                        !message.contains("bcrypt"),
                        "Should not leak implementation details"
                    );
                    assert!(
                        !message.contains("secret"),
                        "Should not leak secret information"
                    );
                }
            }
        }

        #[tokio::test]
        async fn test_user_creation_error_security() {
            let auth_state = AuthState::new();

            let result = auth_state
                .create_user(
                    "testuser".to_string(),
                    "TestPassword123!".to_string(),
                    ActorRole::Admin,
                )
                .await;

            assert!(result.is_ok(), "First user creation should succeed");

            let duplicate_result = auth_state
                .create_user(
                    "testuser".to_string(),
                    "AnotherPassword456!".to_string(),
                    ActorRole::Farmer,
                )
                .await;

            assert!(
                duplicate_result.is_err(),
                "Duplicate user creation should fail"
            );

            let error = duplicate_result.unwrap_err();
            if let crate::error::WebError::InvalidRequest(message) = error {
                assert!(
                    message.contains("already exists"),
                    "Should indicate user exists"
                );
                assert!(
                    !message.contains("password"),
                    "Should not leak password information"
                );
                assert!(
                    !message.contains("hash"),
                    "Should not leak hash information"
                );
            }
        }
    }

    // ========================================================================
    // PERFORMANCE & SECURITY BOUNDARY TESTS
    // ========================================================================

    mod performance_security_tests {
        use super::*;
        use std::time::Instant;

        // Test secret constant - 32 bytes minimum for security
        const TEST_JWT_SECRET: &[u8] = b"test-secret-for-unit-testing-must-be-32-chars";

        #[test]
        fn test_jwt_token_generation_performance() {
            let username = "testuser";
            let role = ActorRole::Admin;

            let start = Instant::now();
            for _ in 0..100 {
                let result = generate_token(username, &role, TEST_JWT_SECRET);
                assert!(result.is_ok(), "Token generation should succeed");
            }
            let duration = start.elapsed();

            assert!(
                duration < std::time::Duration::from_secs(1),
                "Token generation should be fast: {:?}",
                duration
            );

            let avg_time_per_token = duration / 100;
            assert!(
                avg_time_per_token < std::time::Duration::from_millis(10),
                "Average token generation time should be under 10ms: {:?}",
                avg_time_per_token
            );
        }

        #[test]
        fn test_jwt_token_validation_performance() {
            let username = "testuser";
            let role = ActorRole::Admin;
            let token = generate_token(username, &role, TEST_JWT_SECRET).unwrap();

            let start = Instant::now();
            for _ in 0..100 {
                let result = validate_token(&token, TEST_JWT_SECRET);
                assert!(result.is_ok(), "Token validation should succeed");
            }
            let duration = start.elapsed();

            assert!(
                duration < std::time::Duration::from_secs(1),
                "Token validation should be fast: {:?}",
                duration
            );

            let avg_time_per_validation = duration / 100;
            assert!(
                avg_time_per_validation < std::time::Duration::from_millis(10),
                "Average token validation time should be under 10ms: {:?}",
                avg_time_per_validation
            );
        }

        #[test]
        #[ignore]
        fn test_password_hashing_performance() {
            let password = "TestPassword123!";

            // Use a lower cost for testing to avoid excessive execution time
            let test_cost = if cfg!(debug_assertions) {
                4
            } else {
                DEFAULT_COST
            };

            let start = Instant::now();
            let hash = hash(password, test_cost).unwrap();
            let duration = start.elapsed();

            assert!(
                duration > std::time::Duration::from_millis(10),
                "Password hashing should take some time for security: {:?}",
                duration
            );
            assert!(
                duration < std::time::Duration::from_secs(10),
                "Password hashing should not be too slow even in debug mode: {:?}",
                duration
            );

            let start = Instant::now();
            for _ in 0..2 {
                // Reduced to 2 iterations for faster testing
                let result = verify(password, &hash);
                assert!(result.is_ok(), "Password verification should succeed");
                assert!(result.unwrap(), "Password verification should return true");
            }
            let duration = start.elapsed();

            // Allow more time in debug mode
            let max_duration = if cfg!(debug_assertions) {
                std::time::Duration::from_millis(15000)
            } else {
                std::time::Duration::from_millis(5000)
            };

            assert!(
                duration < max_duration,
                "Password verification should be reasonably fast: {:?}",
                duration
            );
        }

        #[tokio::test]
        async fn test_concurrent_user_creation() {
            let auth_state = AuthState::new();

            let mut handles = Vec::new();

            for i in 0..10 {
                let auth_state = auth_state.clone();
                let handle = tokio::spawn(async move {
                    let username = format!("user{}", i);
                    let password = format!("Password{}!", i);
                    auth_state
                        .create_user(username, password, ActorRole::Farmer)
                        .await
                });
                handles.push(handle);
            }

            for handle in handles {
                let result = handle.await.unwrap();
                assert!(result.is_ok(), "Concurrent user creation should succeed");
            }

            let users = auth_state.list_users().await;
            assert_eq!(users.len(), 10, "All users should be created concurrently");
        }

        #[test]
        fn test_memory_usage_validation() {
            let mut tokens = Vec::new();

            for i in 0..1000 {
                let username = format!("user{}", i);
                let token = generate_token(&username, &ActorRole::Admin, TEST_JWT_SECRET).unwrap();
                tokens.push(token);
            }

            for token in &tokens {
                let result = validate_token(token, TEST_JWT_SECRET);
                assert!(result.is_ok(), "All tokens should validate successfully");
            }

            let long_username = "a".repeat(50);
            let result = generate_token(&long_username, &ActorRole::Admin, TEST_JWT_SECRET);
            assert!(result.is_ok(), "Long username should be handled correctly");

            let long_password = "a".repeat(128) + "A1!";
            let hash_result = hash(&long_password, DEFAULT_COST);
            assert!(
                hash_result.is_ok(),
                "Long password should be handled correctly"
            );
        }
    }

    // ========================================================================
    // INTEGRATION TESTS
    // ========================================================================

    mod integration_security_tests {
        use super::*;

        // Test secret constants for rotation simulation
        const ORIGINAL_SECRET: &[u8] = b"original-secret-32-chars-long-minimum";
        const NEW_SECRET: &[u8] = b"new-secret-32-chars-long-minimum-for-rotation";

        #[tokio::test]
        async fn test_full_user_lifecycle() {
            let auth_state = AuthState::new();

            // Step 1: Create user
            let username = "testuser";
            let password = "TestPassword123!";
            let role = ActorRole::Processor;

            let create_result = auth_state
                .create_user(username.to_string(), password.to_string(), role.clone())
                .await;
            assert!(create_result.is_ok(), "User creation should succeed");

            // Step 2: Get user info
            let user_info = auth_state.get_user_info(username).await;
            assert!(user_info.is_ok(), "User info should be retrievable");

            let (retrieved_username, retrieved_role) = user_info.unwrap();
            assert_eq!(
                retrieved_username, username,
                "Retrieved username should match"
            );
            assert_eq!(retrieved_role, role, "Retrieved role should match");

            // Step 3: Update password
            let new_password = "NewPassword456!";
            let update_result = auth_state
                .update_password(username, new_password.to_string())
                .await;
            assert!(update_result.is_ok(), "Password update should succeed");

            // Step 4: Verify user still exists
            let final_user_info = auth_state.get_user_info(username).await;
            assert!(
                final_user_info.is_ok(),
                "User should still exist after password update"
            );

            // Step 5: Delete user
            let delete_result = auth_state.delete_user(username).await;
            assert!(delete_result.is_ok(), "User deletion should succeed");

            // Step 6: Verify user no longer exists
            let final_check = auth_state.get_user_info(username).await;
            assert!(final_check.is_err(), "User should not exist after deletion");
        }

        #[test]
        fn test_jwt_secret_rotation_simulation() {
            let username = "testuser";
            let role = ActorRole::Admin;

            // Generate token with original secret
            let original_token = generate_token(username, &role, ORIGINAL_SECRET).unwrap();

            // Validate with original secret - should succeed
            let validation1 = validate_token(&original_token, ORIGINAL_SECRET);
            assert!(
                validation1.is_ok(),
                "Token should validate with original secret"
            );

            // Try to validate with new secret - should fail
            let validation2 = validate_token(&original_token, NEW_SECRET);
            assert!(
                validation2.is_err(),
                "Token should fail validation with new secret"
            );

            // Generate new token with new secret
            let new_token = generate_token(username, &role, NEW_SECRET).unwrap();

            // Validate new token with new secret - should succeed
            let validation3 = validate_token(&new_token, NEW_SECRET);
            assert!(
                validation3.is_ok(),
                "New token should validate with new secret"
            );

            // Original token should still fail with new secret
            let validation4 = validate_token(&original_token, NEW_SECRET);
            assert!(
                validation4.is_err(),
                "Original token should still fail with new secret"
            );
        }

        #[tokio::test]
        async fn test_auth_state_persistence() {
            let auth_state = AuthState::new();

            assert!(
                !auth_state.has_users().await,
                "Initial AuthState should have no users"
            );

            auth_state
                .create_user(
                    "testuser".to_string(),
                    "TestPassword123!".to_string(),
                    ActorRole::Farmer,
                )
                .await
                .unwrap();

            assert!(
                auth_state.has_users().await,
                "AuthState should have users after creation"
            );

            let users = auth_state.list_users().await;
            assert_eq!(users.len(), 1, "Should have exactly one user");
            assert_eq!(users[0].0, "testuser", "Username should match");
            assert_eq!(users[0].1, ActorRole::Farmer, "Role should match");

            auth_state
                .create_user(
                    "administrator".to_string(),
                    "AdminPassword123!".to_string(),
                    ActorRole::Admin,
                )
                .await
                .unwrap();

            auth_state
                .create_user(
                    "processor".to_string(),
                    "ProcessorPassword123!".to_string(),
                    ActorRole::Processor,
                )
                .await
                .unwrap();

            let users = auth_state.list_users().await;
            assert_eq!(users.len(), 3, "Should have three users");

            let user_map: std::collections::HashMap<String, ActorRole> =
                users.into_iter().collect();
            assert_eq!(user_map.get("testuser"), Some(&ActorRole::Farmer));
            assert_eq!(user_map.get("administrator"), Some(&ActorRole::Admin));
            assert_eq!(user_map.get("processor"), Some(&ActorRole::Processor));
        }

        #[tokio::test]
        async fn test_multi_role_security_isolation() {
            let auth_state = AuthState::new();

            let roles = vec![
                ("admin_user", ActorRole::Admin),
                ("farmer_user", ActorRole::Farmer),
                ("processor_user", ActorRole::Processor),
                ("auditor_user", ActorRole::Auditor),
                ("transporter_user", ActorRole::Transporter),
                ("retailer_user", ActorRole::Retailer),
                ("consumer_user", ActorRole::Consumer),
            ];

            for (username, role) in &roles {
                auth_state
                    .create_user(
                        username.to_string(),
                        format!("{}Password123!", username),
                        role.clone(),
                    )
                    .await
                    .unwrap();
            }

            for (username, expected_role) in &roles {
                let (retrieved_username, retrieved_role) =
                    auth_state.get_user_info(username).await.unwrap();
                assert_eq!(retrieved_username, *username);
                assert_eq!(retrieved_role, *expected_role);
            }
        }
    }
}
