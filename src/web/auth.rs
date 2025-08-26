//! Authentication and authorization module for web API

use crate::web::models::{UserClaims, ActorRole, AuthRequest, AuthResponse, ApiError};
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

/// JWT secret key (loaded from environment or generated securely)
fn get_jwt_secret() -> Result<Vec<u8>, crate::error::WebError> {
    match std::env::var("JWT_SECRET") {
        Ok(secret) => {
            if secret.len() < 32 {
                return Err(crate::error::WebError::ServerError(
                    "JWT_SECRET must be at least 32 characters long for security".to_string()
                ));
            }
            Ok(secret.into_bytes())
        }
        Err(_) => {
            if cfg!(debug_assertions) {
                // Only allow default in debug mode
                eprintln!("WARNING: Using default JWT secret in debug mode. Set JWT_SECRET environment variable for production!");
                Ok("debug-jwt-secret-change-in-production-32chars".to_string().into_bytes())
            } else {
                Err(crate::error::WebError::AuthenticationFailed(
                    "JWT_SECRET environment variable must be set in production mode".to_string()
                ))
            }
        }
    }
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
        let mut users = HashMap::new();
        
        // Add default users for demo purposes with proper bcrypt hashing
        // Use fallback hashes if bcrypt fails (should not happen in normal operation)
        let admin_hash = hash("admin123", DEFAULT_COST)
            .unwrap_or_else(|_| "fallback_admin_hash".to_string());
        let farmer_hash = hash("farmer123", DEFAULT_COST)
            .unwrap_or_else(|_| "fallback_farmer_hash".to_string());
        let processor_hash = hash("processor123", DEFAULT_COST)
            .unwrap_or_else(|_| "fallback_processor_hash".to_string());
        
        users.insert("admin".to_string(), UserInfo {
            username: "admin".to_string(),
            password_hash: admin_hash,
            role: ActorRole::Admin,
        });
        
        users.insert("farmer1".to_string(), UserInfo {
            username: "farmer1".to_string(),
            password_hash: farmer_hash,
            role: ActorRole::Farmer,
        });
        
        users.insert("processor1".to_string(), UserInfo {
            username: "processor1".to_string(),
            password_hash: processor_hash,
            role: ActorRole::Processor,
        });

        Self {
            users: Arc::new(RwLock::new(users)),
        }
    }

    /// Create a new user with secure password hashing
    pub async fn create_user(&self, username: String, password: String, role: ActorRole) -> Result<(), crate::error::WebError> {
        let password_hash = hash(&password, DEFAULT_COST)
            .map_err(|e| crate::error::WebError::ServerError(
                format!("Password hashing failed: {}", e)
            ))?;

        let mut users = self.users.write().await;
        
        if users.contains_key(&username) {
            return Err(crate::error::WebError::InvalidRequest(
                format!("User '{}' already exists", username)
            ));
        }

        users.insert(username.clone(), UserInfo {
            username,
            password_hash,
            role,
        });

        Ok(())
    }

    /// Update user password with secure hashing
    pub async fn update_password(&self, username: &str, new_password: String) -> Result<(), crate::error::WebError> {
        let password_hash = hash(&new_password, DEFAULT_COST)
            .map_err(|e| crate::error::WebError::ServerError(
                format!("Password hashing failed: {}", e)
            ))?;

        let mut users = self.users.write().await;
        
        if let Some(user_info) = users.get_mut(username) {
            user_info.password_hash = password_hash;
            Ok(())
        } else {
            Err(crate::error::WebError::ResourceNotFound(
                format!("User '{}' not found", username)
            ))
        }
    }
}

/// Generate JWT token for authenticated user
pub fn generate_token(username: &str, role: &ActorRole) -> Result<String, crate::error::WebError> {
    let jwt_secret = get_jwt_secret()?;
    
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .ok_or_else(|| crate::error::WebError::ServerError(
            "Failed to calculate token expiration time".to_string()
        ))?
        .timestamp() as usize;

    let claims = UserClaims {
        sub: username.to_string(),
        role: role.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&jwt_secret),
    )
    .map_err(|e| crate::error::WebError::AuthenticationFailed(
        format!("Token generation failed: {}", e)
    ))
}

/// Validate JWT token and extract claims
pub fn validate_token(token: &str) -> Result<UserClaims, crate::error::WebError> {
    let jwt_secret = get_jwt_secret()?;
    
    decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(&jwt_secret),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| crate::error::WebError::AuthenticationFailed(
        format!("Token validation failed: {}", e)
    ))
}

/// Authentication handler
pub async fn authenticate(
    State(auth_state): State<AuthState>,
    Json(auth_request): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ApiError>)> {
    let users = auth_state.users.read().await;
    
    if let Some(user_info) = users.get(&auth_request.username) {
        // Use bcrypt to verify password
        match verify(&auth_request.password, &user_info.password_hash) {
            Ok(true) => {
                match generate_token(&auth_request.username, &user_info.role) {
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
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            match validate_token(token) {
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
pub fn require_role(required_role: ActorRole) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, (StatusCode, Json<ApiError>)>> + Send>> + Clone {
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
