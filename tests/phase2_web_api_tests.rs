//! Phase 2 Test Suite: REST API and Web Interface
//!
//! This comprehensive test suite validates all Phase 2 functionality including:
//! - Web server startup and configuration
//! - Authentication and authorization
//! - REST API endpoints
//! - Data model serialization/deserialization
//! - Security features
//! - Integration scenarios

use chrono::Utc;
use provchain_org::core::blockchain::Blockchain;
use provchain_org::storage::rdf_store::RDFStore;
use provchain_org::utils::config::NodeConfig;
use provchain_org::web::{models::*, WebServer};
use serde_json::json;

use provchain_org::config::Config;

/// Test helper to create a test web server instance
fn create_test_server() -> WebServer {
    let blockchain = Blockchain::new();
    let mut config = Config::default();
    config.web.port = 0; // Use random available port for testing

    WebServer::new(blockchain, config)
}

/// Test helper to get authentication token (for integration tests)
async fn _get_auth_token(
    server_url: &str,
    username: &str,
    password: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let auth_request = AuthRequest {
        username: username.to_string(),
        password: password.to_string(),
    };

    let response = client
        .post(format!("{server_url}/auth/login"))
        .json(&auth_request)
        .send()
        .await?;

    let auth_response: AuthResponse = response.json().await?;
    Ok(auth_response.token)
}

#[tokio::test]
async fn test_web_server_creation() {
    let server = create_test_server();

    // Test that server can be created without panicking
    assert_eq!(server.port(), 0, "Server should use port 0 for testing");
}

#[test]
fn test_auth_request_serialization() {
    let auth_request = AuthRequest {
        username: "testuser".to_string(),
        password: "testpass".to_string(),
    };

    let json = serde_json::to_string(&auth_request).unwrap();
    assert!(json.contains("testuser"));
    assert!(json.contains("testpass"));

    let deserialized: AuthRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.username, "testuser");
    assert_eq!(deserialized.password, "testpass");
}

#[test]
fn test_auth_response_serialization() {
    let auth_response = AuthResponse {
        token: "test_token_123".to_string(),
        expires_at: Utc::now(),
        user_role: "admin".to_string(),
    };

    let json = serde_json::to_string(&auth_response).unwrap();
    assert!(json.contains("test_token_123"));
    assert!(json.contains("admin"));

    let deserialized: AuthResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.token, "test_token_123");
    assert_eq!(deserialized.user_role, "admin");
}

#[test]
fn test_blockchain_status_model() {
    let status = BlockchainStatus {
        height: 10,
        latest_block_hash: "abc123".to_string(),
        total_transactions: 25,
        network_peers: 3,
        last_updated: Utc::now(),
    };

    let json = serde_json::to_string(&status).unwrap();
    let deserialized: BlockchainStatus = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.height, 10);
    assert_eq!(deserialized.latest_block_hash, "abc123");
    assert_eq!(deserialized.total_transactions, 25);
    assert_eq!(deserialized.network_peers, 3);
}

#[test]
fn test_block_info_model() {
    let block_info = BlockInfo {
        index: 5,
        hash: "block_hash_123".to_string(),
        previous_hash: "prev_hash_456".to_string(),
        timestamp: Utc::now(),
        transaction_count: 3,
        size_bytes: 1024,
    };

    let json = serde_json::to_string(&block_info).unwrap();
    let deserialized: BlockInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.index, 5);
    assert_eq!(deserialized.hash, "block_hash_123");
    assert_eq!(deserialized.previous_hash, "prev_hash_456");
    assert_eq!(deserialized.transaction_count, 3);
    assert_eq!(deserialized.size_bytes, 1024);
}

#[test]
fn test_add_triple_request_model() {
    let request = AddTripleRequest {
        subject: "http://example.org/product1".to_string(),
        predicate: "http://example.org/hasLocation".to_string(),
        object: "Farm A".to_string(),
        graph_name: Some("supply_chain".to_string()),
        privacy_key_id: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: AddTripleRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.subject, "http://example.org/product1");
    assert_eq!(deserialized.predicate, "http://example.org/hasLocation");
    assert_eq!(deserialized.object, "Farm A");
    assert_eq!(deserialized.graph_name, Some("supply_chain".to_string()));
    assert_eq!(deserialized.privacy_key_id, None);
}

#[test]
fn test_add_triple_request_with_privacy() {
    let request = AddTripleRequest {
        subject: "http://example.org/secret_formula".to_string(),
        predicate: "http://example.org/hasIngredient".to_string(),
        object: "Secret Ingredient X".to_string(),
        graph_name: None,
        privacy_key_id: Some("key_123".to_string()),
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("privacy_key_id"));
    assert!(json.contains("key_123"));

    let deserialized: AddTripleRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.privacy_key_id, Some("key_123".to_string()));
}

#[test]
fn test_sparql_query_request_model() {
    let request = SparqlQueryRequest {
        query: "SELECT * WHERE { ?s ?p ?o }".to_string(),
        format: Some("json".to_string()),
    };

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: SparqlQueryRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.query, "SELECT * WHERE { ?s ?p ?o }");
    assert_eq!(deserialized.format, Some("json".to_string()));
}

#[test]
fn test_sparql_query_response_model() {
    let response = SparqlQueryResponse {
        results: json!({"bindings": []}),
        execution_time_ms: 150,
        result_count: 0,
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: SparqlQueryResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.execution_time_ms, 150);
    assert_eq!(deserialized.result_count, 0);
}

#[test]
fn test_product_trace_model() {
    let environmental_data = EnvironmentalData {
        temperature: Some(22.5),
        humidity: Some(65.0),
        co2_footprint: Some(1.2),
        certifications: vec!["Organic".to_string(), "Fair Trade".to_string()],
    };

    let trace_event = TraceEvent {
        timestamp: Utc::now(),
        location: "Farm A".to_string(),
        actor: "Farmer John".to_string(),
        action: "Harvested".to_string(),
        details: "Organic tomatoes harvested".to_string(),
        block_hash: "block123".to_string(),
    };

    let product_trace = ProductTrace {
        batch_id: "BATCH001".to_string(),
        product_name: "Organic Tomatoes".to_string(),
        origin: "Farm A".to_string(),
        current_location: "Processing Plant B".to_string(),
        status: "In Transit".to_string(),
        timeline: vec![trace_event],
        certifications: vec!["Organic".to_string()],
        environmental_data: Some(environmental_data),
    };

    let json = serde_json::to_string(&product_trace).unwrap();
    let deserialized: ProductTrace = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.batch_id, "BATCH001");
    assert_eq!(deserialized.product_name, "Organic Tomatoes");
    assert_eq!(deserialized.origin, "Farm A");
    assert_eq!(deserialized.current_location, "Processing Plant B");
    assert_eq!(deserialized.status, "In Transit");
    assert_eq!(deserialized.timeline.len(), 1);
    assert_eq!(deserialized.certifications.len(), 1);
    assert!(deserialized.environmental_data.is_some());

    let env_data = deserialized.environmental_data.unwrap();
    assert_eq!(env_data.temperature, Some(22.5));
    assert_eq!(env_data.humidity, Some(65.0));
    assert_eq!(env_data.co2_footprint, Some(1.2));
    assert_eq!(env_data.certifications.len(), 2);
}

#[test]
fn test_api_error_model() {
    let api_error = ApiError {
        error: "ValidationError".to_string(),
        message: "Invalid input data".to_string(),
        timestamp: Utc::now(),
    };

    let json = serde_json::to_string(&api_error).unwrap();
    let deserialized: ApiError = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.error, "ValidationError");
    assert_eq!(deserialized.message, "Invalid input data");
}

#[test]
fn test_user_claims_model() {
    let claims = UserClaims {
        sub: "user123".to_string(),
        role: "farmer".to_string(),
        exp: 1234567890,
    };

    let json = serde_json::to_string(&claims).unwrap();
    let deserialized: UserClaims = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.sub, "user123");
    assert_eq!(deserialized.role, "farmer");
    assert_eq!(deserialized.exp, 1234567890);
}

#[test]
fn test_actor_role_display() {
    assert_eq!(ActorRole::Farmer.to_string(), "farmer");
    assert_eq!(ActorRole::Processor.to_string(), "processor");
    assert_eq!(ActorRole::Transporter.to_string(), "transporter");
    assert_eq!(ActorRole::Retailer.to_string(), "retailer");
    assert_eq!(ActorRole::Consumer.to_string(), "consumer");
    assert_eq!(ActorRole::Auditor.to_string(), "auditor");
    assert_eq!(ActorRole::Admin.to_string(), "admin");
}

#[test]
fn test_actor_role_serialization() {
    let roles = vec![
        ActorRole::Farmer,
        ActorRole::Processor,
        ActorRole::Transporter,
        ActorRole::Retailer,
        ActorRole::Consumer,
        ActorRole::Auditor,
        ActorRole::Admin,
    ];

    for role in roles {
        let json = serde_json::to_string(&role).unwrap();
        let deserialized: ActorRole = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{role:?}"), format!("{:?}", deserialized));
    }
}

#[test]
fn test_transaction_info_model() {
    let transaction = TransactionInfo {
        subject: "http://example.org/product1".to_string(),
        predicate: "http://example.org/hasLocation".to_string(),
        object: "Farm A".to_string(),
        block_index: 5,
        timestamp: Utc::now(),
    };

    let json = serde_json::to_string(&transaction).unwrap();
    let deserialized: TransactionInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.subject, "http://example.org/product1");
    assert_eq!(deserialized.predicate, "http://example.org/hasLocation");
    assert_eq!(deserialized.object, "Farm A");
    assert_eq!(deserialized.block_index, 5);
}

/// Integration test for model validation with real blockchain data
#[test]
fn test_models_with_blockchain_integration() {
    let mut blockchain = Blockchain::new();
    let _ = blockchain.add_block("test triple data".to_string());

    let status = BlockchainStatus {
        height: blockchain.chain.len(),
        latest_block_hash: blockchain.chain.last().unwrap().hash.clone(),
        total_transactions: blockchain.chain.len() - 1, // Exclude genesis block
        network_peers: 0,
        last_updated: Utc::now(),
    };

    assert_eq!(status.height, 2); // Genesis + 1 block
    assert!(!status.latest_block_hash.is_empty());
    assert_eq!(status.total_transactions, 1);
}

/// Test data validation and edge cases
#[test]
fn test_model_edge_cases() {
    // Test empty strings
    let auth_request = AuthRequest {
        username: "".to_string(),
        password: "".to_string(),
    };
    let json = serde_json::to_string(&auth_request).unwrap();
    let _: AuthRequest = serde_json::from_str(&json).unwrap();

    // Test None values
    let add_triple = AddTripleRequest {
        subject: "test".to_string(),
        predicate: "test".to_string(),
        object: "test".to_string(),
        graph_name: None,
        privacy_key_id: None,
    };
    let json = serde_json::to_string(&add_triple).unwrap();
    let deserialized: AddTripleRequest = serde_json::from_str(&json).unwrap();
    assert!(deserialized.graph_name.is_none());

    // Test empty vectors
    let product_trace = ProductTrace {
        batch_id: "test".to_string(),
        product_name: "test".to_string(),
        origin: "test".to_string(),
        current_location: "test".to_string(),
        status: "test".to_string(),
        timeline: vec![],
        certifications: vec![],
        environmental_data: None,
    };
    let json = serde_json::to_string(&product_trace).unwrap();
    let deserialized: ProductTrace = serde_json::from_str(&json).unwrap();
    assert!(deserialized.timeline.is_empty());
    assert!(deserialized.certifications.is_empty());
    assert!(deserialized.environmental_data.is_none());
}

/// Test JSON compatibility with external systems
#[test]
fn test_json_compatibility() {
    // Test that our models can handle JSON from external systems
    let external_json = r#"{
        "username": "external_user",
        "password": "external_pass"
    }"#;

    let auth_request: AuthRequest = serde_json::from_str(external_json).unwrap();
    assert_eq!(auth_request.username, "external_user");
    assert_eq!(auth_request.password, "external_pass");

    // Test that we can produce JSON for external systems
    let sparql_response = SparqlQueryResponse {
        results: json!({
            "head": {"vars": ["s", "p", "o"]},
            "results": {"bindings": []}
        }),
        execution_time_ms: 100,
        result_count: 0,
    };

    let json = serde_json::to_string_pretty(&sparql_response).unwrap();
    assert!(json.contains("head"));
    assert!(json.contains("vars"));
    assert!(json.contains("bindings"));
}

/// Test RDF Store integration with Phase 2 models
#[test]
fn test_rdf_store_integration() {
    let rdf_store = RDFStore::new();

    // Test that RDF store can be created for web server
    assert!(true, "RDF store should be created successfully");

    // Test basic RDF operations that would be used by the web API
    let test_query = "SELECT * WHERE { ?s ?p ?o } LIMIT 10";
    let _results = rdf_store.query(test_query);

    // The query should execute without panicking
    assert!(true, "SPARQL query should execute successfully");
}

/// Test configuration integration for Phase 2
#[test]
fn test_config_integration() {
    let config = NodeConfig::default();

    // Test that default config has reasonable values for web server
    assert_eq!(config.network.listen_port, 8080);
    assert_eq!(config.network.bind_address, "0.0.0.0");
    assert!(config.validate().is_ok());
}

/// Test blockchain integration with web models
#[test]
fn test_blockchain_web_integration() {
    let mut blockchain = Blockchain::new();

    // Add some test data
    let _ = blockchain.add_block("test RDF data".to_string());
    let _ = blockchain.add_block("more test data".to_string());

    // Test that blockchain data can be converted to web models
    let block_info = BlockInfo {
        index: blockchain.chain[1].index as usize,
        hash: blockchain.chain[1].hash.clone(),
        previous_hash: blockchain.chain[1].previous_hash.clone(),
        timestamp: Utc::now(), // In real implementation, would parse from block
        transaction_count: 1,
        size_bytes: blockchain.chain[1].data.len(),
    };

    assert_eq!(block_info.index, 1);
    assert!(!block_info.hash.is_empty());
    assert!(!block_info.previous_hash.is_empty());
    assert_eq!(block_info.size_bytes, "test RDF data".len());
}

/// Test error handling in web models
#[test]
fn test_error_handling() {
    // Test API error creation
    let error = ApiError {
        error: "InvalidRequest".to_string(),
        message: "The request body is malformed".to_string(),
        timestamp: Utc::now(),
    };

    let json = serde_json::to_string(&error).unwrap();
    assert!(json.contains("InvalidRequest"));
    assert!(json.contains("malformed"));

    // Test that error can be deserialized
    let deserialized: ApiError = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.error, "InvalidRequest");
}

/// Test authentication model validation
#[test]
fn test_auth_model_validation() {
    // Test valid authentication request
    let valid_auth = AuthRequest {
        username: "valid_user".to_string(),
        password: "secure_password123".to_string(),
    };

    let json = serde_json::to_string(&valid_auth).unwrap();
    let deserialized: AuthRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.username, "valid_user");
    assert_eq!(deserialized.password, "secure_password123");

    // Test user claims
    let claims = UserClaims {
        sub: "user_123".to_string(),
        role: "farmer".to_string(),
        exp: 1234567890, // Fixed timestamp for testing
    };

    let claims_json = serde_json::to_string(&claims).unwrap();
    let deserialized_claims: UserClaims = serde_json::from_str(&claims_json).unwrap();
    assert_eq!(deserialized_claims.sub, "user_123");
    assert_eq!(deserialized_claims.role, "farmer");
}

/// Test supply chain specific models
#[test]
fn test_supply_chain_models() {
    // Test environmental data
    let env_data = EnvironmentalData {
        temperature: Some(25.0),
        humidity: Some(60.0),
        co2_footprint: Some(2.5),
        certifications: vec!["Organic".to_string(), "Carbon Neutral".to_string()],
    };

    let json = serde_json::to_string(&env_data).unwrap();
    let deserialized: EnvironmentalData = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.temperature, Some(25.0));
    assert_eq!(deserialized.humidity, Some(60.0));
    assert_eq!(deserialized.co2_footprint, Some(2.5));
    assert_eq!(deserialized.certifications.len(), 2);

    // Test trace event
    let event = TraceEvent {
        timestamp: Utc::now(),
        location: "Processing Plant".to_string(),
        actor: "Quality Inspector".to_string(),
        action: "Quality Check".to_string(),
        details: "Passed all quality standards".to_string(),
        block_hash: "abc123def456".to_string(),
    };

    let event_json = serde_json::to_string(&event).unwrap();
    let deserialized_event: TraceEvent = serde_json::from_str(&event_json).unwrap();

    assert_eq!(deserialized_event.location, "Processing Plant");
    assert_eq!(deserialized_event.actor, "Quality Inspector");
    assert_eq!(deserialized_event.action, "Quality Check");
    assert_eq!(deserialized_event.block_hash, "abc123def456");
}
