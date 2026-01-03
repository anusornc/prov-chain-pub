//! Web server implementation using Axum

use crate::config::{Config, CorsConfig};
use crate::core::blockchain::Blockchain;
use crate::web::{
    auth::{auth_middleware, authenticate, AuthState},
    handlers::{
        add_triple,
        create_participant,
        create_transaction,
        delete_sparql_query,
        execute_sparql_query,
        get_analytics,
        get_block,
        get_block_rdf_summary,
        get_blockchain_status,
        get_blocks,
        get_enhanced_product_trace,
        get_knowledge_graph,

        get_product_analytics,
        get_product_by_id,
        get_product_provenance,
        get_product_trace,
        get_product_trace_path,
        get_products,
        get_products_by_participant,
        get_products_by_type,
        get_recent_transactions,
        get_related_items,
        get_saved_sparql_queries,
        // SPARQL helper endpoints
        get_sparql_config,
        health_check,
        register_wallet,
        save_sparql_query,
        sign_transaction,
        submit_transaction,
        toggle_favorite_sparql_query,
        trace_path_api,
        validate_blockchain,
        validate_item,
        validate_sparql_endpoint,
        AppState,
    },
    websocket::{websocket_handler, BlockchainEventBroadcaster, WebSocketState},
};
use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir, set_header::SetResponseHeaderLayer};
use tracing::{error, info};

/// Web server for the blockchain API
pub struct WebServer {
    app_state: AppState,
    auth_state: AuthState,
    websocket_state: WebSocketState,
    event_broadcaster: BlockchainEventBroadcaster,
    config: Config,
}

impl WebServer {
    /// Create a new web server instance
    pub fn new(blockchain: Blockchain, config: Config) -> Self {
        let blockchain_arc = Arc::new(Mutex::new(blockchain.clone()));
        let websocket_state = WebSocketState::new(blockchain_arc);
        let event_broadcaster = BlockchainEventBroadcaster::new(websocket_state.clone());

        Self {
            app_state: AppState::new(blockchain),
            auth_state: AuthState::new(),
            websocket_state,
            event_broadcaster,
            config,
        }
    }

    /// Create a new web server with a specific port (helper for tests/benchmarks)
    pub fn new_with_port(port: u16) -> Self {
        let mut config = Config::default();
        config.web.port = port;
        let blockchain = Blockchain::new();
        Self::new(blockchain, config)
    }

    /// Access the underlying blockchain (for tests/benchmarks)
    pub fn get_blockchain(&self) -> Arc<tokio::sync::RwLock<Blockchain>> {
        self.app_state.blockchain.clone()
    }

    /// Get the event broadcaster for blockchain operations
    pub fn event_broadcaster(&self) -> &BlockchainEventBroadcaster {
        &self.event_broadcaster
    }

    /// Build CORS layer from configuration
    fn build_cors_layer(&self, cors_config: &CorsConfig) -> CorsLayer {
        if !cors_config.enabled {
            return CorsLayer::permissive();
        }

        // Convert origins to HeaderValue vector
        let origins: Vec<http::HeaderValue> = cors_config
            .allowed_origins
            .iter()
            .filter_map(|origin| origin.parse().ok())
            .collect();

        // Convert methods to Method vector
        let methods: Vec<http::Method> = cors_config
            .allowed_methods
            .iter()
            .filter_map(|method| method.parse().ok())
            .collect();

        // Convert headers to HeaderName vector
        let headers: Vec<http::HeaderName> = cors_config
            .allowed_headers
            .iter()
            .filter_map(|header| header.parse().ok())
            .collect();

        let mut cors = CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(methods)
            .allow_headers(headers);

        // Set credentials
        if cors_config.allow_credentials {
            cors = cors.allow_credentials(true);
        }

        // Set max age if specified
        if let Some(max_age) = cors_config.max_age {
            cors = cors.max_age(std::time::Duration::from_secs(max_age));
        }

        cors
    }

    /// Build the router with all routes and middleware
    fn build_router(&self) -> Router {
        // Static file serving
        let static_service = ServeDir::new("static").append_index_html_on_directories(true);

        // WebSocket routes (no authentication required for WebSocket upgrade)
        let websocket_routes = Router::new()
            .route("/ws", get(websocket_handler))
            .with_state(self.websocket_state.clone());

        // Public routes (no authentication required)
        let public_routes = Router::new()
            .route("/health", get(health_check))
            .route("/auth/login", post(authenticate))
            .with_state(self.auth_state.clone());

        // Protected routes (authentication required)
        let protected_routes = Router::new()
            .route("/api/trace", get(trace_path_api))
            .route("/api/knowledge-graph", get(get_knowledge_graph))
            .route("/api/blockchain/status", get(get_blockchain_status))
            .route("/api/blockchain/blocks", get(get_blocks))
            .route("/api/blockchain/blocks/:index", get(get_block))
            .route(
                "/api/blockchain/blocks/:index/rdf-summary",
                get(get_block_rdf_summary),
            )
            .route("/api/blockchain/validate", get(validate_blockchain))
            .route("/api/transactions/recent", get(get_recent_transactions))
            .route("/api/analytics", get(get_analytics))
            .route("/api/sparql/query", post(execute_sparql_query))
            .route("/api/sparql/config", get(get_sparql_config))
            .route("/api/sparql/validate", post(validate_sparql_endpoint))
            .route("/api/sparql/queries", get(get_saved_sparql_queries))
            .route("/api/sparql/queries", post(save_sparql_query))
            .route("/api/sparql/queries/:id", delete(delete_sparql_query))
            .route(
                "/api/sparql/queries/:id/favorite",
                post(toggle_favorite_sparql_query),
            )
            .route("/api/products/trace", get(get_product_trace))
            .route(
                "/api/products/trace/enhanced",
                get(get_enhanced_product_trace),
            )
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
            .route("/api/products/by-type/:type", get(get_products_by_type))
            .route(
                "/api/products/by-participant/:participantId",
                get(get_products_by_participant),
            )
            .route("/api/products/:id/related", get(get_related_items))
            .route("/api/products/:id/validate", get(validate_item))
            .route("/api/participants", post(create_participant))
            .layer(middleware::from_fn(auth_middleware))
            .with_state(self.app_state.clone());

        // Configure CORS using configuration
        let cors_config = self.config.get_development_cors();
        let cors_layer = self.build_cors_layer(&cors_config);

        Router::new()
            .merge(websocket_routes)
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
                        http::HeaderValue::from_static(
                            "default-src 'self'; \
                             script-src 'self'; \
                             style-src 'self' 'unsafe-inline'; \
                             img-src 'self' data:; \
                             connect-src 'self' ws: wss:; \
                             font-src 'self'; \
                             object-src 'none'; \
                             base-uri 'self'; \
                             form-action 'self'; \
                             frame-ancestors 'none'"
                        ),
                    ))
                    .into_inner()
            )
    }

    /// Start the web server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let app = self.build_router();
        let addr = SocketAddr::from(([0, 0, 0, 0], self.config.web.port));

        info!("Starting ProvChain web server on {}", addr);
        info!(
            "Web UI available at: http://localhost:{}",
            self.config.web.port
        );
        info!(
            "WebSocket endpoint available at: ws://localhost:{}/ws",
            self.config.web.port
        );
        info!("API endpoints available:");
        info!("  GET  /health - Health check");
        info!("  GET  /ws - WebSocket connection for real-time updates");
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
        info!("Real-time features: Block creation, transaction updates, integrity alerts");

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
        self.config.web.port
    }
}

/// Create and configure the web server
pub async fn create_web_server(
    blockchain: Blockchain,
    config: Option<Config>,
) -> Result<WebServer, anyhow::Error> {
    let server_config = config.unwrap_or_else(|| Config::load_or_default("config.toml"));
    let server = WebServer::new(blockchain, server_config.clone());

    info!("Web server configured on port {}", server_config.web.port);
    Ok(server)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blockchain::Blockchain;

    #[tokio::test]
    async fn test_server_creation() {
        let blockchain = Blockchain::new();
        let config = Config::default();
        let server = WebServer::new(blockchain, config);
        assert_eq!(server.port(), 8080);
    }
}
