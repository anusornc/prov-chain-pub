//! Web server implementation using Axum

use crate::core::blockchain::Blockchain;
use crate::web::{
    auth::{AuthState, authenticate, auth_middleware},
    handlers::{
        AppState, health_check, get_blockchain_status, get_block, get_blocks,
        add_triple, execute_sparql_query, get_product_trace, get_recent_transactions,
        validate_blockchain, get_enhanced_product_trace, register_wallet,
        create_transaction, sign_transaction, submit_transaction,
        get_products, get_product_by_id, get_product_trace_path, get_product_provenance,
        get_product_analytics, get_knowledge_graph,
    },
};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
    set_header::SetResponseHeaderLayer,
};
use tracing::{info, error};

/// Web server for the blockchain API
pub struct WebServer {
    app_state: AppState,
    auth_state: AuthState,
    port: u16,
}

impl WebServer {
    /// Create a new web server instance
    pub fn new(blockchain: Blockchain, port: u16) -> Self {
        Self {
            app_state: AppState::new(blockchain),
            auth_state: AuthState::new(),
            port,
        }
    }

    /// Build the router with all routes and middleware
    fn build_router(&self) -> Router {
        // Static file serving
        let static_service = ServeDir::new("static").append_index_html_on_directories(true);

        // Public routes (no authentication required)
        let public_routes = Router::new()
            .route("/health", get(health_check))
            .route("/auth/login", post(authenticate))
            .with_state(self.auth_state.clone());

        // Protected routes (authentication required)
        let protected_routes = Router::new()
            .route("/api/blockchain/status", get(get_blockchain_status))
            .route("/api/blockchain/blocks", get(get_blocks))
            .route("/api/blockchain/blocks/:index", get(get_block))
            .route("/api/blockchain/validate", get(validate_blockchain))
            .route("/api/transactions/recent", get(get_recent_transactions))
            .route("/api/sparql/query", post(execute_sparql_query))
            .route("/api/products/trace", get(get_product_trace))
            .route("/api/products/trace/enhanced", get(get_enhanced_product_trace))
            .route("/api/blockchain/add-triple", post(add_triple))
            .route("/api/wallet/register", post(register_wallet))
            .route("/api/transactions/create", post(create_transaction))
            .route("/api/transactions/sign", post(sign_transaction))
            .route("/api/transactions/submit", post(submit_transaction))
            // New traceability API endpoints
            .route("/api/products", get(get_products))
            .route("/api/products/:id", get(get_product_by_id))
            .route("/api/products/:id/trace", get(get_product_trace_path))
            .route("/api/products/:id/provenance", get(get_product_provenance))
            .route("/api/products/:id/analytics", get(get_product_analytics))
            .route("/api/knowledge-graph", get(get_knowledge_graph))
            .layer(middleware::from_fn(auth_middleware))
            .with_state(self.app_state.clone());

        // Configure CORS - secure by default
        let cors_layer = if cfg!(debug_assertions) {
            // Development mode - allow localhost frontend on multiple ports
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse::<http::HeaderValue>().unwrap())
                .allow_origin("http://localhost:5174".parse::<http::HeaderValue>().unwrap())
                .allow_origin("http://localhost:5175".parse::<http::HeaderValue>().unwrap())
                .allow_credentials(true)
                .allow_methods([
                    http::Method::GET,
                    http::Method::POST,
                    http::Method::OPTIONS,
                ])
                .allow_headers([
                    http::header::AUTHORIZATION,
                    http::header::CONTENT_TYPE,
                    http::header::ACCEPT,
                ])
        } else {
            // Production mode - restrict to specific origins
            let allowed_origins = std::env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| {
                    eprintln!("WARNING: ALLOWED_ORIGINS not set, using default");
                    "https://yourdomain.com".to_string()
                });
            
            CorsLayer::new()
                .allow_origin(allowed_origins.parse::<http::HeaderValue>().unwrap())
                .allow_credentials(true)
                .allow_methods([
                    http::Method::GET,
                    http::Method::POST,
                    http::Method::OPTIONS,
                ])
                .allow_headers([
                    http::header::AUTHORIZATION,
                    http::header::CONTENT_TYPE,
                    http::header::ACCEPT,
                ])
        };

        Router::new()
            .merge(public_routes)
            .merge(protected_routes)
            .nest_service("/", static_service)
            .layer(
                ServiceBuilder::new()
                    .layer(cors_layer)
                    // Security headers
                    .layer(SetResponseHeaderLayer::if_not_present(
                        http::header::X_CONTENT_TYPE_OPTIONS,
                        http::HeaderValue::from_static("nosniff"),
                    ))
                    .layer(SetResponseHeaderLayer::if_not_present(
                        http::header::X_FRAME_OPTIONS,
                        http::HeaderValue::from_static("DENY"),
                    ))
                    .layer(SetResponseHeaderLayer::if_not_present(
                        http::header::X_XSS_PROTECTION,
                        http::HeaderValue::from_static("1; mode=block"),
                    ))
                    .layer(SetResponseHeaderLayer::if_not_present(
                        http::header::REFERRER_POLICY,
                        http::HeaderValue::from_static("strict-origin-when-cross-origin"),
                    ))
                    .layer(SetResponseHeaderLayer::if_not_present(
                        http::header::CONTENT_SECURITY_POLICY,
                        http::HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'"),
                    ))
                    .into_inner()
            )
    }

    /// Start the web server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let app = self.build_router();
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));

        info!("Starting ProvChain web server on {}", addr);
        info!("Web UI available at: http://localhost:{}", self.port);
        info!("API endpoints available:");
        info!("  GET  /health - Health check");
        info!("  POST /auth/login - Authentication");
        info!("  POST /api/wallet/register - Register new wallet");
        info!("  POST /api/transactions/create - Create new transaction");
        info!("  POST /api/transactions/sign - Sign transaction");
        info!("  POST /api/transactions/submit - Submit transaction to blockchain");
        info!("  GET  /api/blockchain/status - Blockchain status");
        info!("  GET  /api/blockchain/blocks - All blocks");
        info!("  GET  /api/blockchain/blocks/:index - Specific block");
        info!("  GET  /api/blockchain/validate - Validate blockchain");
        info!("  GET  /api/transactions/recent - Recent transactions");
        info!("  POST /api/sparql/query - Execute SPARQL query");
        info!("  GET  /api/products/trace - Product traceability");
        info!("  POST /api/blockchain/add-triple - Add new triple");
        info!("Static files served from: ./static/");

        let listener = tokio::net::TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;
        
        info!("Web server listening on {}", local_addr);
        
        match axum::serve(listener, app).await {
            Ok(_) => {
                info!("Web server started successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to start web server: {}", e);
                Err(Box::new(e))
            }
        }
    }

    /// Get the server port
    pub fn port(&self) -> u16 {
        self.port
    }
}

/// Create and configure the web server
pub async fn create_web_server(
    blockchain: Blockchain,
    port: Option<u16>,
) -> Result<WebServer, anyhow::Error> {
    let server_port = port.unwrap_or(8080);
    let server = WebServer::new(blockchain, server_port);
    
    info!("Web server configured on port {}", server_port);
    Ok(server)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blockchain::Blockchain;

    #[tokio::test]
    async fn test_server_creation() {
        let blockchain = Blockchain::new();
        let server = WebServer::new(blockchain, 8080);
        assert_eq!(server.port(), 8080);
    }
}
