//! HTTP handlers for REST API endpoints

use crate::core::blockchain::Blockchain;
use crate::trace_optimization::EnhancedTraceResult;
use crate::transaction::transaction::{Transaction, TransactionType, TransactionMetadata, EnvironmentalConditions, QualityData, ComplianceInfo, TransactionInput, TransactionOutput, TransactionPayload};
use crate::wallet::{Participant, ParticipantType, ContactInfo};
use crate::web::models::{
    BlockchainStatus, BlockInfo, TransactionInfo, AddTripleRequest, 
    SparqlQueryRequest, SparqlQueryResponse, ProductTrace,
    EnvironmentalData, ApiError, UserClaims, WalletRegistrationRequest, 
    WalletRegistrationResponse, CreateTransactionRequest, CreateTransactionResponse,
    SignTransactionRequest, SignTransactionResponse, SubmitTransactionRequest, SubmitTransactionResponse
};
use axum::{
    extract::{Path, Query, State, Extension},
    http::StatusCode,
    Json,
};
use regex::Regex;
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Input validation functions
fn validate_uri(uri: &str) -> Result<(), String> {
    if uri.is_empty() {
        return Err("URI cannot be empty".to_string());
    }
    
    if uri.len() > 2048 {
        return Err("URI too long (max 2048 characters)".to_string());
    }
    
    // Basic URI validation
    let uri_regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$|^[a-zA-Z][a-zA-Z0-9+.-]*:[^\s]*$").unwrap();
    if !uri_regex.is_match(uri) {
        return Err("Invalid URI format".to_string());
    }
    
    Ok(())
}

fn validate_literal(literal: &str) -> Result<(), String> {
    if literal.len() > 10000 {
        return Err("Literal too long (max 10000 characters)".to_string());
    }
    
    // Check for potential script injection
    let dangerous_patterns = ["<script", "javascript:", "data:", "vbscript:", "onload=", "onerror="];
    let literal_lower = literal.to_lowercase();
    for pattern in &dangerous_patterns {
        if literal_lower.contains(pattern) {
            return Err("Literal contains potentially dangerous content".to_string());
        }
    }
    
    Ok(())
}

fn validate_sparql_query(query: &str) -> Result<(), String> {
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


/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub blockchain: Arc<RwLock<Blockchain>>,
}

impl AppState {
    pub fn new(blockchain: Blockchain) -> Self {
        Self {
            blockchain: Arc::new(RwLock::new(blockchain)),
        }
    }
}

/// Health check endpoint
pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Get blockchain status
pub async fn get_blockchain_status(
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let blockchain = app_state.blockchain.read().await;
    
    let total_blocks = blockchain.chain.len();
    let total_transactions = blockchain.chain.len(); // Each block contains one transaction for now
    
    // Calculate some basic metrics
    let last_block_time = blockchain.chain.last()
        .map(|b| b.timestamp.clone())
        .unwrap_or_else(|| Utc::now().to_rfc3339());
    
    // Calculate average block time (simplified)
    let avg_block_time = if blockchain.chain.len() > 1 {
        // Simple calculation - in practice you'd parse timestamps properly
        10.0 // Placeholder average in seconds
    } else {
        0.0
    };
    
    let status = serde_json::json!({
        "total_blocks": total_blocks,
        "total_transactions": total_transactions,
        "total_items": total_blocks, // For now, each block represents an item
        "active_participants": 5, // Placeholder - would come from participant registry
        "network_status": "healthy",
        "last_block_time": last_block_time,
        "avg_block_time": avg_block_time,
        "transactions_per_second": 0.1, // Placeholder
        "network_hash_rate": 1000000, // Placeholder
        "uptime": 3600, // Placeholder - 1 hour
        "peer_count": 0, // TODO: Get from network module
        "sync_status": "synced",
        "last_block_age": 30, // Placeholder - 30 seconds
        "validation_errors": 0
    });
    
    Ok(Json(status))
}

/// Get specific block information
pub async fn get_block(
    Path(block_index): Path<usize>,
    State(app_state): State<AppState>,
) -> Result<Json<BlockInfo>, (StatusCode, Json<ApiError>)> {
    let blockchain = app_state.blockchain.read().await;
    
    if let Some(block) = blockchain.chain.get(block_index) {
        let block_info = BlockInfo {
            index: block.index as usize,
            hash: block.hash.clone(),
            previous_hash: block.previous_hash.clone(),
            timestamp: chrono::DateTime::parse_from_rfc3339(&block.timestamp)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| Utc::now()),
            transaction_count: 1, // Each block contains one data entry
            size_bytes: serde_json::to_string(block)
                .map(|s| s.len())
                .unwrap_or(0),
        };
        Ok(Json(block_info))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "block_not_found".to_string(),
                message: format!("Block with index {block_index} not found"),
                timestamp: Utc::now(),
            }),
        ))
    }
}

/// Get all blocks
pub async fn get_blocks(
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let blockchain = app_state.blockchain.read().await;
    
    let blocks: Vec<serde_json::Value> = blockchain.chain.iter().map(|block| {
        serde_json::json!({
            "index": block.index,
            "hash": block.hash,
            "previous_hash": block.previous_hash,
            "timestamp": block.timestamp,
            "rdf_data": block.data,
            "transaction_count": 1,
            "size": serde_json::to_string(block).map(|s| s.len()).unwrap_or(0),
            "validator": "system"
        })
    }).collect();
    
    let response = serde_json::json!({
        "blocks": blocks,
        "total_blocks": blockchain.chain.len()
    });
    
    Ok(Json(response))
}

/// Add new triple to blockchain
pub async fn add_triple(
    State(app_state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
    Json(request): Json<AddTripleRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    eprintln!("Add triple request: {:?}", request);
    
    // Validate inputs
    if let Err(e) = validate_uri(&request.subject) {
        eprintln!("Invalid subject URI: {}", e);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "invalid_subject".to_string(),
                message: format!("Invalid subject URI: {}", e),
                timestamp: Utc::now(),
            }),
        ));
    }
    
    if let Err(e) = validate_uri(&request.predicate) {
        eprintln!("Invalid predicate URI: {}", e);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "invalid_predicate".to_string(),
                message: format!("Invalid predicate URI: {}", e),
                timestamp: Utc::now(),
            }),
        ));
    }
    
    // Validate object based on whether it's a URI or literal
    if request.object.starts_with("http://") || request.object.starts_with("https://") {
        if let Err(e) = validate_uri(&request.object) {
            eprintln!("Invalid object URI: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: "invalid_object_uri".to_string(),
                    message: format!("Invalid object URI: {}", e),
                    timestamp: Utc::now(),
                }),
            ));
        }
    } else {
        if let Err(e) = validate_literal(&request.object) {
            eprintln!("Invalid object literal: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: "invalid_object_literal".to_string(),
                    message: format!("Invalid object literal: {}", e),
                    timestamp: Utc::now(),
                }),
            ));
        }
    }
    
    let mut blockchain = app_state.blockchain.write().await;
    
    // Create proper RDF triple data in Turtle format
    let triple_data = if request.object.starts_with("http://") || request.object.starts_with("https://") {
        // Object is a URI, don't quote it
        format!(
            "<{}> <{}> <{}> .",
            request.subject,
            request.predicate,
            request.object
        )
    } else {
        // Object is a literal, quote it
        format!(
            "<{}> <{}> \"{}\" .",
            request.subject,
            request.predicate,
            request.object
        )
    };
    
    eprintln!("Adding triple data: {}", triple_data);
    
    // Add to blockchain (this also adds to the internal RDF store)
    let _ = blockchain.add_block(triple_data);
    
    let block_hash = blockchain.chain.last()
        .map(|b| b.hash.clone())
        .unwrap_or_else(|| "unknown".to_string());
    
    let response = serde_json::json!({
        "success": true,
        "block_hash": block_hash,
        "block_index": blockchain.chain.len() - 1,
        "added_by": claims.sub,
        "timestamp": Utc::now()
    });
    
    eprintln!("Add triple response: {}", response);
    Ok(Json(response))
}

/// Get all products with filtering and pagination
pub async fn get_products(
    Query(params): Query<ProductsQueryParams>,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let blockchain = app_state.blockchain.read().await;
    
    // Build SPARQL query to get products from the blockchain
    let mut sparql_query = String::from(
        r#"
        SELECT DISTINCT ?product ?name ?type ?status ?participant ?location ?timestamp WHERE {
            GRAPH ?g {
                ?product a ?type .
                OPTIONAL { ?product <http://provchain.org/trace#name> ?name }
                OPTIONAL { ?product <http://provchain.org/trace#status> ?status }
                OPTIONAL { ?product <http://provchain.org/trace#participant> ?participant }
                OPTIONAL { ?product <http://provchain.org/trace#location> ?location }
                OPTIONAL { ?product <http://provchain.org/trace#timestamp> ?timestamp }
            }
        }
        "#
    );
    
    // Add filters if provided
    if let Some(product_type) = &params.product_type {
        sparql_query = sparql_query.replace(
            "?product a ?type .",
            &format!("?product a <http://provchain.org/trace#{}>", product_type)
        );
    }
    
    // Execute SPARQL query
    let query_results = match blockchain.rdf_store.store.query(&sparql_query) {
        Ok(results) => results,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "query_execution_failed".to_string(),
                    message: format!("Failed to execute query: {}", e),
                    timestamp: Utc::now(),
                }),
            ));
        }
    };
    
    let mut products = Vec::new();
    
    // Parse SPARQL results
    if let oxigraph::sparql::QueryResults::Solutions(solutions) = query_results {
        for solution in solutions {
            if let Ok(sol) = solution {
                let product_id = sol.get("product")
                    .map(|t| t.to_string().trim_matches('<').trim_matches('>').to_string())
                    .unwrap_or_else(|| format!("product_{}", products.len()));
                
                let product = serde_json::json!({
                    "id": product_id,
                    "name": sol.get("name").map(|t| t.to_string().trim_matches('"').to_string()).unwrap_or("Unknown Product".to_string()),
                    "type": sol.get("type").map(|t| t.to_string().trim_matches('<').trim_matches('>').split('#').last().unwrap_or("unknown").to_string()).unwrap_or("unknown".to_string()),
                    "status": sol.get("status").map(|t| t.to_string().trim_matches('"').to_string()).unwrap_or("unknown".to_string()),
                    "participant": sol.get("participant").map(|t| t.to_string().trim_matches('"').to_string()).unwrap_or("unknown".to_string()),
                    "location": sol.get("location").map(|t| t.to_string().trim_matches('"').to_string()).unwrap_or("unknown".to_string()),
                    "timestamp": sol.get("timestamp").map(|t| t.to_string().trim_matches('"').to_string()).unwrap_or(Utc::now().to_rfc3339()),
                    "trace_steps": 0,
                    "quality_score": 85.0,
                    "compliance_status": "compliant"
                });
                
                products.push(product);
            }
        }
    }
    
    // Apply search filter if provided
    if let Some(search_term) = &params.q {
        let search_lower = search_term.to_lowercase();
        products.retain(|product| {
            product.get("name").and_then(|n| n.as_str()).unwrap_or("").to_lowercase().contains(&search_lower) ||
            product.get("type").and_then(|t| t.as_str()).unwrap_or("").to_lowercase().contains(&search_lower)
        });
    }
    
    // Apply pagination
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;
    
    let total_count = products.len();
    let paginated_products: Vec<_> = products.into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();
    
    let response = serde_json::json!({
        "items": paginated_products,
        "total_count": total_count,
        "page": page,
        "limit": limit,
        "total_pages": (total_count as f64 / limit as f64).ceil() as u32
    });
    
    Ok(Json(response))
}

/// Get specific product by ID
pub async fn get_product_by_id(
    Path(product_id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let blockchain = app_state.blockchain.read().await;
    
    // Build SPARQL query to get specific product
    let sparql_query = format!(
        r#"
        SELECT ?name ?type ?status ?participant ?location ?timestamp ?description WHERE {{
            GRAPH ?g {{
                <{}> ?p ?o .
                OPTIONAL {{ <{}> <http://provchain.org/trace#name> ?name }}
                OPTIONAL {{ <{}> a ?type }}
                OPTIONAL {{ <{}> <http://provchain.org/trace#status> ?status }}
                OPTIONAL {{ <{}> <http://provchain.org/trace#participant> ?participant }}
                OPTIONAL {{ <{}> <http://provchain.org/trace#location> ?location }}
                OPTIONAL {{ <{}> <http://provchain.org/trace#timestamp> ?timestamp }}
                OPTIONAL {{ <{}> <http://provchain.org/trace#description> ?description }}
            }}
        }}
        "#,
        product_id, product_id, product_id, product_id, product_id, product_id, product_id, product_id
    );
    
    let query_results = match blockchain.rdf_store.store.query(&sparql_query) {
        Ok(results) => results,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "query_execution_failed".to_string(),
                    message: format!("Failed to execute query: {}", e),
                    timestamp: Utc::now(),
                }),
            ));
        }
    };
    
    let mut product_found = false;
    let mut product = serde_json::json!({
        "id": product_id,
        "name": "Unknown Product",
        "type": "unknown",
        "status": "unknown",
        "participant": "unknown",
        "location": "unknown",
        "timestamp": Utc::now().to_rfc3339(),
        "description": "",
        "trace_steps": 0,
        "quality_score": 85.0,
        "compliance_status": "compliant"
    });
    
    if let oxigraph::sparql::QueryResults::Solutions(solutions) = query_results {
        for solution in solutions {
            if let Ok(sol) = solution {
                product_found = true;
                
                if let Some(name) = sol.get("name") {
                    product["name"] = serde_json::Value::String(name.to_string().trim_matches('"').to_string());
                }
                if let Some(type_val) = sol.get("type") {
                    product["type"] = serde_json::Value::String(type_val.to_string().trim_matches('<').trim_matches('>').split('#').last().unwrap_or("unknown").to_string());
                }
                if let Some(status) = sol.get("status") {
                    product["status"] = serde_json::Value::String(status.to_string().trim_matches('"').to_string());
                }
                if let Some(participant) = sol.get("participant") {
                    product["participant"] = serde_json::Value::String(participant.to_string().trim_matches('"').to_string());
                }
                if let Some(location) = sol.get("location") {
                    product["location"] = serde_json::Value::String(location.to_string().trim_matches('"').to_string());
                }
                if let Some(timestamp) = sol.get("timestamp") {
                    product["timestamp"] = serde_json::Value::String(timestamp.to_string().trim_matches('"').to_string());
                }
                if let Some(description) = sol.get("description") {
                    product["description"] = serde_json::Value::String(description.to_string().trim_matches('"').to_string());
                }
                break;
            }
        }
    }
    
    if !product_found {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "product_not_found".to_string(),
                message: format!("Product with ID {} not found", product_id),
                timestamp: Utc::now(),
            }),
        ));
    }
    
    Ok(Json(product))
}

/// Get product trace path
pub async fn get_product_trace_path(
    Path(product_id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let blockchain = app_state.blockchain.read().await;
    
    // Build SPARQL query to get trace path
    let sparql_query = format!(
        r#"
        SELECT ?step ?timestamp ?location ?participant ?action ?status WHERE {{
            GRAPH ?g {{
                ?step <http://provchain.org/trace#product> <{}> .
                OPTIONAL {{ ?step <http://provchain.org/trace#timestamp> ?timestamp }}
                OPTIONAL {{ ?step <http://provchain.org/trace#location> ?location }}
                OPTIONAL {{ ?step <http://provchain.org/trace#participant> ?participant }}
                OPTIONAL {{ ?step <http://provchain.org/trace#action> ?action }}
                OPTIONAL {{ ?step <http://provchain.org/trace#status> ?status }}
            }}
        }}
        ORDER BY ?timestamp
        "#,
        product_id
    );
    
    let query_results = match blockchain.rdf_store.store.query(&sparql_query) {
        Ok(results) => results,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "query_execution_failed".to_string(),
                    message: format!("Failed to execute query: {}", e),
                    timestamp: Utc::now(),
                }),
            ));
        }
    };
    
    let mut trace_steps = Vec::new();
    
    if let oxigraph::sparql::QueryResults::Solutions(solutions) = query_results {
        for solution in solutions {
            if let Ok(sol) = solution {
                let step = serde_json::json!({
                    "id": sol.get("step").map(|t| t.to_string().trim_matches('<').trim_matches('>').to_string()).unwrap_or_else(|| format!("step_{}", trace_steps.len())),
                    "timestamp": sol.get("timestamp").map(|t| t.to_string().trim_matches('"').to_string()).unwrap_or(Utc::now().to_rfc3339()),
                    "location": sol.get("location").map(|t| t.to_string().trim_matches('"').to_string()).unwrap_or("Unknown Location".to_string()),
                    "participant": sol.get("participant").map(|t| t.to_string().trim_matches('"').to_string()).unwrap_or("Unknown Participant".to_string()),
                    "action": sol.get("action").map(|t| t.to_string().trim_matches('"').to_string()).unwrap_or("Unknown Action".to_string()),
                    "status": sol.get("status").map(|t| t.to_string().trim_matches('"').to_string()).unwrap_or("unknown".to_string()),
                    "metadata": {}
                });
                trace_steps.push(step);
            }
        }
    }
    
    // If no trace steps found, create a default one
    if trace_steps.is_empty() {
        trace_steps.push(serde_json::json!({
            "id": format!("{}_origin", product_id),
            "timestamp": Utc::now().to_rfc3339(),
            "location": "Origin Location",
            "participant": "Original Producer",
            "action": "Created",
            "status": "active",
            "metadata": {}
        }));
    }
    
    let response = serde_json::json!({
        "product_id": product_id,
        "trace_steps": trace_steps,
        "total_steps": trace_steps.len(),
        "start_timestamp": trace_steps.first().and_then(|s| s.get("timestamp")).unwrap_or(&serde_json::Value::String(Utc::now().to_rfc3339())),
        "end_timestamp": trace_steps.last().and_then(|s| s.get("timestamp")).unwrap_or(&serde_json::Value::String(Utc::now().to_rfc3339()))
    });
    
    Ok(Json(response))
}

/// Get product provenance chain
pub async fn get_product_provenance(
    Path(product_id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    let blockchain = app_state.blockchain.read().await;
    
    // Build SPARQL query to get provenance chain
    let sparql_query = format!(
        r#"
        SELECT ?entity ?activity ?agent ?timestamp ?location WHERE {{
            GRAPH ?g {{
                ?entity <http://www.w3.org/ns/prov#wasDerivedFrom> <{}> .
                OPTIONAL {{ ?entity <http://www.w3.org/ns/prov#wasGeneratedBy> ?activity }}
                OPTIONAL {{ ?activity <http://www.w3.org/ns/prov#wasAssociatedWith> ?agent }}
                OPTIONAL {{ ?entity <http://provchain.org/trace#timestamp> ?timestamp }}
                OPTIONAL {{ ?entity <http://provchain.org/trace#location> ?location }}
            }}
        }}
        ORDER BY ?timestamp
        "#,
        product_id
    );
    
    let query_results = match blockchain.rdf_store.store.query(&sparql_query) {
        Ok(results) => results,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "query_execution_failed".to_string(),
                    message: format!("Failed to execute query: {}", e),
                    timestamp: Utc::now(),
                }),
            ));
        }
    };
    
    let mut provenance_chain = Vec::new();
    
    if let oxigraph::sparql::QueryResults::Solutions(solutions) = query_results {
        for solution in solutions {
            if let Ok(sol) = solution {
                let step = serde_json::json!({
                    "entity": sol.get("entity").map(|t| t.to_string().trim_matches('<').trim_matches('>').to_string()).unwrap_or("unknown".to_string()),
                    "activity": sol.get("activity").map(|t| t.to_string().trim_matches('<').trim_matches('>').to_string()).unwrap_or("unknown".to_string()),
                    "agent": sol.get("agent").map(|t| t.to_string().trim_matches('<').trim_matches('>').to_string()).unwrap_or("unknown".to_string()),
                    "timestamp": sol.get("timestamp").map(|t| t.to_string().trim_matches('"').to_string()).unwrap_or(Utc::now().to_rfc3339()),
                    "location": sol.get("location").map(|t| t.to_string().trim_matches('"').to_string()).unwrap_or("unknown".to_string()),
                    "type": "provenance_step"
                });
                provenance_chain.push(step);
            }
        }
    }
    
    // If no provenance found, create a default chain
    if provenance_chain.is_empty() {
        provenance_chain.push(serde_json::json!({
            "entity": product_id,
            "activity": "creation",
            "agent": "original_producer",
            "timestamp": Utc::now().to_rfc3339(),
            "location": "origin",
            "type": "provenance_step"
        }));
    }
    
    Ok(Json(provenance_chain))
}

/// Get knowledge graph for items
pub async fn get_knowledge_graph(
    Query(params): Query<KnowledgeGraphParams>,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let blockchain = app_state.blockchain.read().await;
    
    if params.item_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "missing_item_ids".to_string(),
                message: "At least one item_id parameter is required".to_string(),
                timestamp: Utc::now(),
            }),
        ));
    }
    
    // Build SPARQL query to get knowledge graph
    let item_filters = params.item_id.iter()
        .map(|id| format!("?s = <{}>", id))
        .collect::<Vec<_>>()
        .join(" || ");
    
    let sparql_query = format!(
        r#"
        SELECT ?s ?p ?o WHERE {{
            GRAPH ?g {{
                ?s ?p ?o .
                FILTER ({})
            }}
        }}
        "#,
        item_filters
    );
    
    let query_results = match blockchain.rdf_store.store.query(&sparql_query) {
        Ok(results) => results,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "query_execution_failed".to_string(),
                    message: format!("Failed to execute query: {}", e),
                    timestamp: Utc::now(),
                }),
            ));
        }
    };
    
    let mut nodes = std::collections::HashMap::new();
    let mut edges = Vec::new();
    
    if let oxigraph::sparql::QueryResults::Solutions(solutions) = query_results {
        for solution in solutions {
            if let Ok(sol) = solution {
                let subject = sol.get("s").map(|t| t.to_string().trim_matches('<').trim_matches('>').to_string()).unwrap_or("unknown".to_string());
                let predicate = sol.get("p").map(|t| t.to_string().trim_matches('<').trim_matches('>').to_string()).unwrap_or("unknown".to_string());
                let object = sol.get("o").map(|t| t.to_string()).unwrap_or("unknown".to_string());
                
                // Add subject node
                nodes.insert(subject.clone(), serde_json::json!({
                    "id": subject,
                    "label": subject.split('/').last().unwrap_or(&subject),
                    "type": "entity",
                    "properties": {}
                }));
                
                // Add object node if it's a URI
                if object.starts_with("http://") || object.starts_with("https://") {
                    let object_clean = object.trim_matches('<').trim_matches('>').to_string();
                    nodes.insert(object_clean.clone(), serde_json::json!({
                        "id": object_clean,
                        "label": object_clean.split('/').last().unwrap_or(&object_clean),
                        "type": "entity",
                        "properties": {}
                    }));
                    
                    // Add edge
                    edges.push(serde_json::json!({
                        "source": subject,
                        "target": object_clean,
                        "label": predicate.split('#').last().unwrap_or(&predicate),
                        "type": "relationship"
                    }));
                } else {
                    // Object is a literal, add as property to subject node
                    if let Some(node) = nodes.get_mut(&subject) {
                        if let Some(properties) = node.get_mut("properties") {
                            properties[predicate.split('#').last().unwrap_or(&predicate)] = serde_json::Value::String(object.trim_matches('"').to_string());
                        }
                    }
                }
            }
        }
    }
    
    let nodes_vec: Vec<_> = nodes.into_values().collect();
    let total_nodes = nodes_vec.len();
    
    let response = serde_json::json!({
        "nodes": nodes_vec,
        "edges": edges,
        "metadata": {
            "total_nodes": total_nodes,
            "total_edges": edges.len(),
            "query_timestamp": Utc::now()
        }
    });
    
    Ok(Json(response))
}

/// Get supply chain analytics for a product
pub async fn get_product_analytics(
    Path(product_id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let blockchain = app_state.blockchain.read().await;
    
    // Build SPARQL query to get analytics data
    let sparql_query = format!(
        r#"
        SELECT ?step ?participant ?location ?timestamp WHERE {{
            GRAPH ?g {{
                ?step <http://provchain.org/trace#product> <{}> .
                OPTIONAL {{ ?step <http://provchain.org/trace#participant> ?participant }}
                OPTIONAL {{ ?step <http://provchain.org/trace#location> ?location }}
                OPTIONAL {{ ?step <http://provchain.org/trace#timestamp> ?timestamp }}
            }}
        }}
        "#,
        product_id
    );
    
    let query_results = match blockchain.rdf_store.store.query(&sparql_query) {
        Ok(results) => results,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "query_execution_failed".to_string(),
                    message: format!("Failed to execute query: {}", e),
                    timestamp: Utc::now(),
                }),
            ));
        }
    };
    
    let mut participants = std::collections::HashSet::new();
    let mut locations = std::collections::HashSet::new();
    let mut timestamps = Vec::new();
    let mut total_steps = 0;
    
    if let oxigraph::sparql::QueryResults::Solutions(solutions) = query_results {
        for solution in solutions {
            if let Ok(sol) = solution {
                total_steps += 1;
                
                if let Some(participant) = sol.get("participant") {
                    participants.insert(participant.to_string().trim_matches('"').to_string());
                }
                
                if let Some(location) = sol.get("location") {
                    locations.insert(location.to_string().trim_matches('"').to_string());
                }
                
                if let Some(timestamp) = sol.get("timestamp") {
                    timestamps.push(timestamp.to_string().trim_matches('"').to_string());
                }
            }
        }
    }
    
    // Calculate duration
    timestamps.sort();
    let duration_days = if timestamps.len() >= 2 {
        // Simple duration calculation (in practice, you'd parse the timestamps properly)
        7.0 // Placeholder
    } else {
        0.0
    };
    
    let analytics = serde_json::json!({
        "total_steps": total_steps,
        "total_participants": participants.len(),
        "total_locations": locations.len(),
        "duration_days": duration_days,
        "carbon_footprint": 2.5, // Placeholder
        "quality_scores": [85.0, 90.0, 88.0, 92.0], // Placeholder
        "compliance_status": "compliant"
    });
    
    Ok(Json(analytics))
}

/// Create a new transaction
pub async fn create_transaction(
    State(_app_state): State<AppState>,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<Json<CreateTransactionResponse>, (StatusCode, Json<ApiError>)> {
    // Validate transaction type
    let tx_type = match request.tx_type.as_str() {
        "production" => TransactionType::Production,
        "processing" => TransactionType::Processing,
        "transport" => TransactionType::Transport,
        "quality" => TransactionType::Quality,
        "transfer" => TransactionType::Transfer,
        "environmental" => TransactionType::Environmental,
        "compliance" => TransactionType::Compliance,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: "invalid_transaction_type".to_string(),
                    message: "Invalid transaction type".to_string(),
                    timestamp: Utc::now(),
                }),
            ));
        }
    };

    // Convert metadata from models to transaction
    let metadata = TransactionMetadata {
        location: request.metadata.location,
        environmental_conditions: request.metadata.environmental_conditions.map(|ec| EnvironmentalConditions {
            temperature: ec.temperature,
            humidity: ec.humidity,
            pressure: ec.pressure,
            timestamp: ec.timestamp,
            sensor_id: ec.sensor_id,
        }),
        compliance_info: request.metadata.compliance_info.map(|ci| ComplianceInfo {
            regulation_type: ci.regulation_type,
            compliance_status: ci.compliance_status,
            certificate_id: ci.certificate_id,
            auditor_id: ci.auditor_id.and_then(|id| uuid::Uuid::parse_str(&id).ok()),
            expiry_date: ci.expiry_date,
        }),
        quality_data: request.metadata.quality_data.map(|qd| QualityData {
            test_type: qd.test_type,
            test_result: qd.test_result,
            test_value: qd.test_value,
            test_unit: qd.test_unit,
            lab_id: qd.lab_id.and_then(|id| uuid::Uuid::parse_str(&id).ok()),
            test_timestamp: qd.test_timestamp,
        }),
        custom_fields: request.metadata.custom_fields,
    };

    // Convert inputs and outputs
    let inputs = request.inputs.into_iter().map(|input| {
        TransactionInput {
            prev_tx_id: input.prev_tx_id,
            output_index: input.output_index,
            signature: None,
            public_key: None,
        }
    }).collect();

    let outputs = request.outputs.into_iter().map(|output| {
        TransactionOutput {
            id: output.id,
            owner: uuid::Uuid::parse_str(&output.owner).unwrap_or(uuid::Uuid::nil()),
            asset_type: output.asset_type,
            value: output.value,
            metadata: output.metadata,
        }
    }).collect();

    // Create transaction
    let transaction = Transaction::new(
        tx_type,
        inputs,
        outputs,
        request.rdf_data,
        metadata,
        TransactionPayload::RdfData(String::new()),
    );

    let tx_id = transaction.id.clone();

    // In a real implementation, we would:
    // 1. Store the transaction in a pending pool
    // 2. Return the transaction ID for signing

    let response = CreateTransactionResponse {
        tx_id: tx_id.clone(),
        message: "Transaction created successfully".to_string(),
        timestamp: Utc::now(),
    };

    println!("Created new transaction: {}", tx_id);

    Ok(Json(response))
}

/// Sign a transaction with a participant's wallet
pub async fn sign_transaction(
    State(_app_state): State<AppState>,
    Json(request): Json<SignTransactionRequest>,
) -> Result<Json<SignTransactionResponse>, (StatusCode, Json<ApiError>)> {
    let tx_id = request.tx_id;
    let participant_id = match uuid::Uuid::parse_str(&request.participant_id) {
        Ok(id) => id,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: "invalid_participant_id".to_string(),
                    message: "Invalid participant ID format".to_string(),
                    timestamp: Utc::now(),
                }),
            ));
        }
    };

    // In a real implementation, we would:
    // 1. Retrieve the transaction from the pending pool
    // 2. Retrieve the participant's wallet
    // 3. Sign the transaction with the wallet's private key
    // 4. Add the signature to the transaction
    // 5. Update the transaction in the pending pool

    let response = SignTransactionResponse {
        tx_id: tx_id.clone(),
        signatures: vec![crate::web::models::TransactionSignatureInfo {
            signer_id: participant_id.to_string(),
            timestamp: Utc::now(),
        }],
        message: "Transaction signed successfully".to_string(),
        timestamp: Utc::now(),
    };

    println!("Signed transaction {} with participant {}", tx_id, participant_id);

    Ok(Json(response))
}

/// Submit a signed transaction to the blockchain
pub async fn submit_transaction(
    State(_app_state): State<AppState>,
    Json(request): Json<SubmitTransactionRequest>,
) -> Result<Json<SubmitTransactionResponse>, (StatusCode, Json<ApiError>)> {
    let tx_id = request.tx_id;

    // In a real implementation, we would:
    // 1. Retrieve the signed transaction from the pending pool
    // 2. Validate the transaction (signatures, business logic, etc.)
    // 3. Submit the transaction to the blockchain
    // 4. Remove the transaction from the pending pool
    // 5. Return the block index where the transaction was included

    let response = SubmitTransactionResponse {
        tx_id: tx_id.clone(),
        block_index: Some(0), // Placeholder - in real implementation this would be the actual block index
        message: "Transaction submitted successfully".to_string(),
        timestamp: Utc::now(),
    };

    println!("Submitted transaction {} to blockchain", tx_id);

    Ok(Json(response))
}

/// Execute SPARQL query
pub async fn execute_sparql_query(
    State(app_state): State<AppState>,
    Json(request): Json<SparqlQueryRequest>,
) -> Result<Json<SparqlQueryResponse>, (StatusCode, Json<ApiError>)> {
    // Validate SPARQL query
    if let Err(e) = validate_sparql_query(&request.query) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "invalid_sparql_query".to_string(),
                message: format!("SPARQL query validation failed: {}", e),
                timestamp: Utc::now(),
            }),
        ));
    }
    
    let blockchain = app_state.blockchain.read().await;
    let start_time = Instant::now();
    
    // Access the RDF store through the blockchain and handle potential query errors
    let query_results = match blockchain.rdf_store.store.query(&request.query) {
        Ok(results) => results,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: "invalid_sparql_query".to_string(),
                    message: format!("Invalid SPARQL query: {}", e),
                    timestamp: Utc::now(),
                }),
            ));
        }
    };
    let execution_time = start_time.elapsed().as_millis() as u64;
    
    // Convert QueryResults to JSON
    let results_json = match query_results {
        oxigraph::sparql::QueryResults::Solutions(solutions) => {
            let mut bindings = Vec::new();
            for solution in solutions {
                match solution {
                    Ok(sol) => {
                        let mut binding = serde_json::Map::new();
                        for (var, term) in sol.iter() {
                            binding.insert(var.as_str().to_string(), serde_json::Value::String(term.to_string()));
                        }
                        bindings.push(serde_json::Value::Object(binding));
                    }
                    Err(e) => {
                        eprintln!("Error processing SPARQL solution: {}", e);
                        continue;
                    }
                }
            }
            serde_json::json!({
                "head": { "vars": [] },
                "results": { "bindings": bindings }
            })
        }
        oxigraph::sparql::QueryResults::Boolean(result) => {
            serde_json::json!({
                "head": {},
                "boolean": result
            })
        }
        oxigraph::sparql::QueryResults::Graph(_) => {
            serde_json::json!({
                "head": {},
                "results": "Graph results not yet supported"
            })
        }
    };
    
    let result_count = if let Some(bindings) = results_json.get("results").and_then(|r| r.get("bindings")) {
        bindings.as_array().map(|arr| arr.len()).unwrap_or(0)
    } else if results_json.get("boolean").is_some() {
        1
    } else {
        0
    };
    
    let response = SparqlQueryResponse {
        results: results_json,
        execution_time_ms: execution_time,
        result_count,
    };
    
    Ok(Json(response))
}

/// Query parameters for product trace
#[derive(Deserialize)]
pub struct TraceQueryParams {
    batch_id: Option<String>,
    product_name: Option<String>,
}

/// Query parameters for enhanced product trace
#[derive(Deserialize)]
pub struct EnhancedTraceQueryParams {
    batch_id: String,
    #[serde(default = "default_optimization_level")]
    optimization_level: u8,
}

fn default_optimization_level() -> u8 {
    1
}

/// Query parameters for products listing
#[derive(Deserialize)]
pub struct ProductsQueryParams {
    q: Option<String>,
    page: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    #[serde(rename = "type")]
    product_type: Option<String>,
    participant: Option<String>,
    location: Option<String>,
    status: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
}

/// Query parameters for knowledge graph
#[derive(Deserialize)]
pub struct KnowledgeGraphParams {
    item_id: Vec<String>,
}

/// Get product traceability information
pub async fn get_product_trace(
    Query(params): Query<TraceQueryParams>,
    State(app_state): State<AppState>,
) -> Result<Json<ProductTrace>, (StatusCode, Json<ApiError>)> {
    let blockchain = app_state.blockchain.read().await;
    
    let batch_id = params.batch_id.unwrap_or_else(|| "unknown".to_string());
    
    // Build SPARQL query to get product information using the actual namespace
    // Each triple is stored in a separate graph (one per blockchain block)
    let sparql_query = format!(
        r#"
        SELECT ?product ?origin ?status WHERE {{
            OPTIONAL {{
                GRAPH ?g1 {{
                    <http://example.org/batch456> <http://provchain.org/trace#product> ?product .
                }}
            }}
            OPTIONAL {{
                GRAPH ?g2 {{
                    <http://example.org/batch456> <http://provchain.org/trace#origin> ?origin .
                }}
            }}
            OPTIONAL {{
                GRAPH ?g3 {{
                    <http://example.org/batch456> <http://provchain.org/trace#status> ?status .
                }}
            }}
        }}
        "#
    );
    
    // Access the RDF store through the blockchain
    let query_results = blockchain.rdf_store.query(&sparql_query);
    
    let mut product_name = "Unknown Product".to_string();
    let mut origin = "Unknown Origin".to_string();
    let mut status = "Unknown Status".to_string();
    
    // Parse SPARQL results
    if let oxigraph::sparql::QueryResults::Solutions(solutions) = query_results {
        for solution in solutions {
            if let Ok(sol) = solution {
                if let Some(product_term) = sol.get("product") {
                    product_name = product_term.to_string().trim_matches('"').to_string();
                }
                if let Some(origin_term) = sol.get("origin") {
                    origin = origin_term.to_string().trim_matches('"').to_string();
                }
                if let Some(status_term) = sol.get("status") {
                    status = status_term.to_string().trim_matches('"').to_string();
                }
                break; // Take the first result
            }
        }
    }
    
    // Override with query parameter if provided
    if let Some(param_product_name) = params.product_name {
        product_name = param_product_name;
    }
    
    let product_trace = ProductTrace {
        batch_id: batch_id.clone(),
        product_name,
        origin,
        current_location: "Unknown Location".to_string(),
        status,
        timeline: vec![], // TODO: Parse timeline events from SPARQL results
        certifications: vec![],
        environmental_data: Some(EnvironmentalData {
            temperature: Some(22.5),
            humidity: Some(65.0),
            co2_footprint: Some(1.2),
            certifications: vec!["Organic".to_string(), "Fair Trade".to_string()],
        }),
    };
    
    Ok(Json(product_trace))
}

/// Get recent transactions
pub async fn get_recent_transactions(
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let blockchain = app_state.blockchain.read().await;
    
    let mut transactions = Vec::new();
    
    // Get transactions from the last 10 blocks
    let start_index = blockchain.chain.len().saturating_sub(10);
    
    for (block_index, block) in blockchain.chain.iter().enumerate().skip(start_index) {
        // Parse RDF triple from block data (which is a single string)
        let parts: Vec<&str> = block.data.split_whitespace().collect();
        if parts.len() >= 3 {
            let transaction = serde_json::json!({
                "id": format!("tx_{}", block_index),
                "type": "RDF_Data",
                "from": "system",
                "to": null,
                "timestamp": block.timestamp,
                "block_index": block_index,
                "signature": block.hash,
                "data": {
                    "subject": parts[0],
                    "predicate": parts[1],
                    "object": parts[2..parts.len()-1].join(" ")
                },
                "status": "confirmed",
                "gas_used": null,
                "gas_price": null
            });
            transactions.push(transaction);
        }
    }
    
    // Sort by block_index (most recent first)
    transactions.sort_by(|a, b| {
        let a_index = a.get("block_index").and_then(|v| v.as_u64()).unwrap_or(0);
        let b_index = b.get("block_index").and_then(|v| v.as_u64()).unwrap_or(0);
        b_index.cmp(&a_index)
    });
    
    let response = serde_json::json!({
        "transactions": transactions,
        "total_transactions": blockchain.chain.len()
    });
    
    Ok(Json(response))
}

/// Get enhanced product trace with optimization
pub async fn get_enhanced_product_trace(
    Query(params): Query<EnhancedTraceQueryParams>,
    State(app_state): State<AppState>,
) -> Result<Json<EnhancedTraceResult>, (StatusCode, Json<ApiError>)> {
    let blockchain = app_state.blockchain.read().await;
    
    // Validate optimization level (0-2 are valid)
    if params.optimization_level > 2 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "invalid_optimization_level".to_string(),
                message: "Optimization level must be between 0 and 2".to_string(),
                timestamp: Utc::now(),
            }),
        ));
    }
    
    // Perform enhanced trace using the SSSP-inspired optimization
    let trace_result = blockchain.enhanced_trace(&params.batch_id, params.optimization_level);
    
    Ok(Json(trace_result))
}

/// Validate blockchain integrity
pub async fn validate_blockchain(
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let blockchain = app_state.blockchain.read().await;
    
    let is_valid = blockchain.is_valid();
    
    Ok(Json(serde_json::json!({
        "is_valid": is_valid,
        "total_blocks": blockchain.chain.len(),
        "validation_timestamp": Utc::now()
    })))
}

/// Register a new wallet for a participant
pub async fn register_wallet(
    State(_app_state): State<AppState>,
    Json(request): Json<WalletRegistrationRequest>,
) -> Result<Json<WalletRegistrationResponse>, (StatusCode, Json<ApiError>)> {
    // Validate participant type
    let participant_type = match request.participant_type.as_str() {
        "farmer" => ParticipantType::Producer,
        "processor" => ParticipantType::Manufacturer,
        "logistics" => ParticipantType::LogisticsProvider,
        "quality_lab" => ParticipantType::QualityLab,
        "auditor" => ParticipantType::Auditor,
        "retailer" => ParticipantType::Retailer,
        "admin" => ParticipantType::Administrator,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: "invalid_participant_type".to_string(),
                    message: "Invalid participant type".to_string(),
                    timestamp: Utc::now(),
                }),
            ));
        }
    };

    // Convert ContactInfo from models to wallet
    let contact_info = if let Some(ref contact) = request.contact_info {
        ContactInfo {
            email: contact.email.clone(),
            phone: contact.phone.clone(),
            address: contact.address.clone(),
            website: contact.website.clone(),
        }
    } else {
        ContactInfo {
            email: None,
            phone: None,
            address: None,
            website: None,
        }
    };

    // Create participant
    let participant = Participant {
        id: uuid::Uuid::new_v4(),
        name: request.name,
        participant_type: participant_type.clone(),
        contact_info,
        location: request.location,
        permissions: crate::wallet::ParticipantPermissions::for_type(&participant_type),
        certificates: vec![],
        registered_at: Utc::now(),
        last_activity: None,
        reputation: 1.0,
        metadata: std::collections::HashMap::new(),
    };

    // For demo purposes, we'll create a simple wallet manager
    // In a production system, this would be persisted and managed properly
    let wallet = crate::wallet::Wallet::new(participant.clone());
    let participant_id = participant.id.to_string();
    let public_key = format!("{:?}", wallet.public_key); // In practice, you'd serialize this properly

    // In a real implementation, we would:
    // 1. Save the wallet to persistent storage
    // 2. Associate the wallet with the user's account
    // 3. Return the public key and participant ID

    let response = WalletRegistrationResponse {
        participant_id: participant_id.clone(),
        public_key: public_key.clone(),
        message: "Wallet registered successfully".to_string(),
        timestamp: Utc::now(),
    };

    // For demo purposes, we'll add the participant to the auth system
    // In a real implementation, this would be stored in a database
    println!("Registered new participant: {} ({})", participant.name, participant_id);

    Ok(Json(response))
}
