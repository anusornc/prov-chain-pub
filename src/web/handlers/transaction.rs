use crate::core::blockchain::BlockAdmissionTimings;
use crate::security::encryption::PrivacyManager;
use crate::transaction::transaction::{
    ComplianceInfo, EnvironmentalConditions, QualityData, Transaction, TransactionInput,
    TransactionMetadata, TransactionOutput, TransactionPayload, TransactionSignature,
    TransactionType,
};
use crate::wallet::{ContactInfo, Participant, ParticipantType};
use crate::web::handlers::utils::{validate_literal, validate_uri};
use crate::web::handlers::AppState;
use crate::web::models::{
    AddTripleBatchRequest, AddTripleRequest, ApiError, CreateTransactionRequest,
    CreateTransactionResponse, PolicyCheckRequest, PolicyCheckResponse, SignTransactionRequest,
    SignTransactionResponse, SubmitTransactionRequest, SubmitTransactionResponse,
    TurtleImportRequest, UserClaims, WalletRegistrationRequest, WalletRegistrationResponse,
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use oxigraph::{io::RdfFormat, store::Store};
use std::io::Cursor;
use std::time::Instant;
use uuid::Uuid;

fn emit_stage_timings() -> bool {
    std::env::var("PROVCHAIN_BENCHMARK_STAGE_TIMINGS")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn max_turtle_import_bytes() -> usize {
    std::env::var("PROVCHAIN_TURTLE_IMPORT_MAX_BYTES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(10 * 1024 * 1024)
}

fn build_handler_timings(
    request_validation_ms: f64,
    blockchain_lock_wait_ms: f64,
    turtle_materialization_ms: f64,
    block_admission_ms: f64,
    handler_total_ms: f64,
    admission_timings: Option<&BlockAdmissionTimings>,
    timing_scope: &str,
) -> serde_json::Value {
    let mut timings = serde_json::Map::new();
    timings.insert(
        "request_validation".to_string(),
        serde_json::json!(request_validation_ms),
    );
    timings.insert(
        "blockchain_lock_wait".to_string(),
        serde_json::json!(blockchain_lock_wait_ms),
    );
    timings.insert(
        "turtle_materialization".to_string(),
        serde_json::json!(turtle_materialization_ms),
    );
    timings.insert(
        "block_admission".to_string(),
        serde_json::json!(block_admission_ms),
    );
    timings.insert(
        "handler_total".to_string(),
        serde_json::json!(handler_total_ms),
    );
    timings.insert("timing_scope".to_string(), serde_json::json!(timing_scope));

    if let Some(admission_timings) = admission_timings {
        if let Ok(serde_json::Value::Object(details)) = serde_json::to_value(admission_timings) {
            for (key, value) in details {
                timings.insert(format!("block_admission_detail_{}", key), value);
            }
        }
    }

    serde_json::Value::Object(timings)
}

fn api_error(
    status: StatusCode,
    error: &str,
    message: impl Into<String>,
) -> (StatusCode, Json<ApiError>) {
    (
        status,
        Json(ApiError {
            error: error.to_string(),
            message: message.into(),
            timestamp: Utc::now(),
        }),
    )
}

fn decode_privacy_secret(
    key_id: &str,
    secret: &str,
) -> Result<[u8; 32], (StatusCode, Json<ApiError>)> {
    let key_bytes = hex::decode(secret).map_err(|_| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "privacy_key_invalid",
            format!(
                "Stored privacy key '{}' is not valid hex-encoded key material",
                key_id
            ),
        )
    })?;

    key_bytes.try_into().map_err(|bytes: Vec<u8>| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "privacy_key_invalid",
            format!(
                "Stored privacy key '{}' has invalid length: expected 32 bytes, got {}",
                key_id,
                bytes.len()
            ),
        )
    })
}

fn authorize_participant_signer(
    claims: &UserClaims,
    participant_id: Uuid,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let Ok(claim_subject) = Uuid::parse_str(&claims.sub) else {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            "signer_not_authorized",
            "Authenticated user is not authorized to sign for this participant",
        ));
    };

    if claim_subject != participant_id {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            "signer_not_authorized",
            "Authenticated user is not authorized to sign for this participant",
        ));
    }

    Ok(())
}

async fn load_privacy_key_for_claim(
    app_state: &AppState,
    claims: &UserClaims,
    key_id: &str,
) -> Result<[u8; 32], (StatusCode, Json<ApiError>)> {
    let participant_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        api_error(
            StatusCode::FORBIDDEN,
            "privacy_wallet_unavailable",
            "Encrypted triples require an authenticated participant UUID with a wallet",
        )
    })?;

    let mut wallet_manager = app_state.wallet_manager.write().await;
    if wallet_manager.get_wallet(participant_id).is_none() {
        let _ = wallet_manager.load_wallet(participant_id);
    }

    let secret = wallet_manager
        .get_wallet(participant_id)
        .and_then(|wallet| wallet.get_secret(key_id))
        .cloned()
        .ok_or_else(|| {
            api_error(
                StatusCode::FORBIDDEN,
                "privacy_key_not_found",
                format!(
                    "Privacy key '{}' is not available for participant '{}'",
                    key_id, participant_id
                ),
            )
        })?;

    decode_privacy_secret(key_id, &secret)
}

fn object_is_uri(object: &str) -> bool {
    object.starts_with("http://") || object.starts_with("https://")
}

fn validate_add_triple_request(
    request: &AddTripleRequest,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    if let Err(e) = validate_uri(&request.subject) {
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
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "invalid_predicate".to_string(),
                message: format!("Invalid predicate URI: {}", e),
                timestamp: Utc::now(),
            }),
        ));
    }

    if object_is_uri(&request.object) {
        if let Err(e) = validate_uri(&request.object) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: "invalid_object_uri".to_string(),
                    message: format!("Invalid object URI: {}", e),
                    timestamp: Utc::now(),
                }),
            ));
        }
    } else if let Err(e) = validate_literal(&request.object) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "invalid_object_literal".to_string(),
                message: format!("Invalid object literal: {}", e),
                timestamp: Utc::now(),
            }),
        ));
    }

    Ok(())
}

fn triple_to_turtle_line(request: &AddTripleRequest) -> String {
    if object_is_uri(&request.object) {
        format!(
            "<{}> <{}> <{}> .",
            request.subject, request.predicate, request.object
        )
    } else {
        format!(
            "<{}> <{}> \"{}\" .",
            request.subject, request.predicate, request.object
        )
    }
}

fn validate_turtle_import_request(
    request: &TurtleImportRequest,
) -> Result<usize, (StatusCode, Json<ApiError>)> {
    let trimmed = request.turtle_data.trim();
    if trimmed.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "empty_turtle_import".to_string(),
                message: "Turtle import data must not be empty".to_string(),
                timestamp: Utc::now(),
            }),
        ));
    }

    let byte_count = request.turtle_data.len();
    let max_bytes = max_turtle_import_bytes();
    if byte_count > max_bytes {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(ApiError {
                error: "turtle_import_too_large".to_string(),
                message: format!(
                    "Turtle import payload is {byte_count} bytes; maximum is {max_bytes} bytes"
                ),
                timestamp: Utc::now(),
            }),
        ));
    }

    let store = Store::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: "rdf_store_error".to_string(),
                message: format!("Failed to create RDF parser store: {}", e),
                timestamp: Utc::now(),
            }),
        )
    })?;
    let reader = Cursor::new(request.turtle_data.as_bytes());
    store
        .load_from_reader(RdfFormat::Turtle, reader)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: "invalid_turtle_import".to_string(),
                    message: format!("Invalid Turtle import payload: {}", e),
                    timestamp: Utc::now(),
                }),
            )
        })?;

    Ok(store.len().unwrap_or(0))
}

/// Import a Turtle dataset as one blockchain block.
pub async fn import_turtle_dataset(
    State(app_state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
    Json(request): Json<TurtleImportRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let request_start = Instant::now();
    let emit_stage_timings = emit_stage_timings();

    let validation_start = Instant::now();
    let triple_count = validate_turtle_import_request(&request)?;
    let request_validation_ms = validation_start.elapsed().as_secs_f64() * 1000.0;
    let dataset_bytes = request.turtle_data.len();

    let lock_start = Instant::now();
    let mut blockchain = app_state.blockchain.write().await;
    let blockchain_lock_wait_ms = lock_start.elapsed().as_secs_f64() * 1000.0;

    let block_admission_start = Instant::now();
    let block_admission_result = if emit_stage_timings {
        blockchain
            .add_block_with_timings(request.turtle_data)
            .map(Some)
    } else {
        blockchain.add_block(request.turtle_data).map(|_| None)
    };

    match block_admission_result {
        Ok(admission_timings) => {
            let block_admission_ms = block_admission_start.elapsed().as_secs_f64() * 1000.0;
            let block_hash = blockchain
                .chain
                .last()
                .map(|b| b.hash.clone())
                .unwrap_or_else(|| "unknown".to_string());

            let mut response = serde_json::json!({
                "success": true,
                "block_hash": block_hash,
                "block_index": blockchain.chain.len() - 1,
                "added_by": claims.sub,
                "timestamp": Utc::now(),
                "validation_status": "passed",
                "import_mode": "bulk_turtle_single_block",
                "dataset_bytes": dataset_bytes,
                "triple_count": triple_count,
                "block_count": 1
            });
            if emit_stage_timings {
                response["timings_ms"] = build_handler_timings(
                    request_validation_ms,
                    blockchain_lock_wait_ms,
                    0.0,
                    block_admission_ms,
                    request_start.elapsed().as_secs_f64() * 1000.0,
                    admission_timings.as_ref(),
                    "bulk_turtle_single_block",
                );
            }

            Ok(Json(response))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: "blockchain_error".to_string(),
                message: format!("Failed to import Turtle dataset: {}", e),
                timestamp: Utc::now(),
            }),
        )),
    }
}

/// Add new triple to blockchain with SHACL validation
pub async fn add_triple(
    State(app_state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
    Json(request): Json<AddTripleRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let request_start = Instant::now();
    let emit_stage_timings = emit_stage_timings();
    let validation_start = Instant::now();
    eprintln!("Add triple request: {:?}", request);

    // Validate inputs
    validate_add_triple_request(&request)?;
    let request_validation_ms = validation_start.elapsed().as_secs_f64() * 1000.0;

    let privacy_key = if let Some(key_id) = &request.privacy_key_id {
        Some((
            key_id.clone(),
            load_privacy_key_for_claim(&app_state, &claims, key_id).await?,
        ))
    } else {
        None
    };

    let lock_start = Instant::now();
    let mut blockchain = app_state.blockchain.write().await;
    let blockchain_lock_wait_ms = lock_start.elapsed().as_secs_f64() * 1000.0;

    // Create proper RDF triple data in Turtle format
    let turtle_start = Instant::now();
    let triple_data = triple_to_turtle_line(&request);
    let turtle_materialization_ms = turtle_start.elapsed().as_secs_f64() * 1000.0;

    eprintln!("Adding triple data: {}", triple_data);

    // Check for privacy request
    if let Some((key_id, key)) = privacy_key.as_ref() {
        match PrivacyManager::encrypt(&triple_data, key, key_id) {
            Ok(encrypted) => {
                let encrypted_json = serde_json::to_string(&encrypted).unwrap_or_default();

                // Clone validator public key before borrowing blockchain mutably
                let validator_public_key = blockchain.validator_public_key.clone();

                // Create a block with encrypted payload
                // We use a placeholder for the public data to indicate it's encrypted
                match blockchain.create_block_proposal(
                    format!("@prefix prov: <http://provchain.org/core#> . prov:EncryptedData prov:hasKeyId \"{}\" .", key_id),
                    Some(encrypted_json),
                    validator_public_key
                ) {
                    Ok(mut block) => {
                        // Sign the block
                        use ed25519_dalek::Signer;
                        let signature = blockchain.signing_key.sign(block.hash.as_bytes());
                        block.signature = hex::encode(signature.to_bytes());

                        match blockchain.submit_signed_block(block) {
                            Ok(()) => {
                                let block_hash = blockchain.chain.last().map(|b| b.hash.clone()).unwrap_or_default();
                                let response = serde_json::json!({
                                    "success": true,
                                    "block_hash": block_hash,
                                    "block_index": blockchain.chain.len() - 1,
                                    "added_by": claims.sub,
                                    "timestamp": Utc::now(),
                                    "validation_status": "encrypted",
                                    "encryption_status": "secured"
                                });
                                return Ok(Json(response));
                            },
                            Err(e) => {
                                return Err((
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    Json(ApiError {
                                        error: "blockchain_error".to_string(),
                                        message: format!("Failed to submit encrypted block: {}", e),
                                        timestamp: Utc::now(),
                                    }),
                                ));
                            }
                        }
                    },
                    Err(e) => {
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiError {
                                error: "block_creation_error".to_string(),
                                message: format!("Failed to create block proposal: {}", e),
                                timestamp: Utc::now(),
                            }),
                        ));
                    }
                }
            }
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError {
                        error: "encryption_error".to_string(),
                        message: format!("Failed to encrypt data: {}", e),
                        timestamp: Utc::now(),
                    }),
                ));
            }
        }
        // If we processed the encrypted transaction successfully, we return early
        // This prevents falling through to the unencrypted logic
        // Note: The return is inside the Ok(encrypted) match arm above
    }

    // Standard unencrypted flow
    // STEP 9: Add to blockchain with SHACL validation (this also adds to the internal RDF store)
    let block_admission_start = Instant::now();
    let block_admission_result = if emit_stage_timings {
        blockchain.add_block_with_timings(triple_data).map(Some)
    } else {
        blockchain.add_block(triple_data).map(|_| None)
    };
    match block_admission_result {
        Ok(admission_timings) => {
            let block_admission_ms = block_admission_start.elapsed().as_secs_f64() * 1000.0;
            let block_hash = blockchain
                .chain
                .last()
                .map(|b| b.hash.clone())
                .unwrap_or_else(|| "unknown".to_string());

            let mut response = serde_json::json!({
                "success": true,
                "block_hash": block_hash,
                "block_index": blockchain.chain.len() - 1,
                "added_by": claims.sub,
                "timestamp": Utc::now(),
                "validation_status": "passed"
            });
            if emit_stage_timings {
                response["timings_ms"] = build_handler_timings(
                    request_validation_ms,
                    blockchain_lock_wait_ms,
                    turtle_materialization_ms,
                    block_admission_ms,
                    request_start.elapsed().as_secs_f64() * 1000.0,
                    admission_timings.as_ref(),
                    "single_block",
                );
            }

            eprintln!("Add triple response: {}", response);
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Failed to add triple to blockchain: {}", e);

            // Check if this is a SHACL validation error
            let error_msg = e.to_string();
            if error_msg.contains("Transaction validation failed")
                || error_msg.contains("SHACL validation")
            {
                // SHACL validation failure - return detailed validation error
                Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(ApiError {
                        error: "shacl_validation_failed".to_string(),
                        message: format!(
                            "Transaction rejected due to SHACL validation failure: {}",
                            error_msg
                        ),
                        timestamp: Utc::now(),
                    }),
                ))
            } else {
                // Other blockchain errors
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError {
                        error: "blockchain_error".to_string(),
                        message: format!("Failed to add transaction to blockchain: {}", e),
                        timestamp: Utc::now(),
                    }),
                ))
            }
        }
    }
}

/// Add multiple triples as a single blockchain block.
pub async fn add_triples_batch(
    State(app_state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
    Json(request): Json<AddTripleBatchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let request_start = Instant::now();
    let emit_stage_timings = emit_stage_timings();
    let validation_start = Instant::now();

    if request.triples.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "empty_batch".to_string(),
                message: "Batch must include at least one triple".to_string(),
                timestamp: Utc::now(),
            }),
        ));
    }

    if request.triples.len() > 1000 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "batch_too_large".to_string(),
                message: "Batch is limited to 1000 triples".to_string(),
                timestamp: Utc::now(),
            }),
        ));
    }

    for triple in &request.triples {
        if triple.privacy_key_id.is_some() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: "privacy_batch_unsupported".to_string(),
                    message: "Encrypted triples must be submitted individually".to_string(),
                    timestamp: Utc::now(),
                }),
            ));
        }
        validate_add_triple_request(triple)?;
    }
    let request_validation_ms = validation_start.elapsed().as_secs_f64() * 1000.0;

    let turtle_start = Instant::now();
    let turtle_data = request
        .triples
        .iter()
        .map(triple_to_turtle_line)
        .collect::<Vec<_>>()
        .join("\n");
    let turtle_materialization_ms = turtle_start.elapsed().as_secs_f64() * 1000.0;

    let lock_start = Instant::now();
    let mut blockchain = app_state.blockchain.write().await;
    let blockchain_lock_wait_ms = lock_start.elapsed().as_secs_f64() * 1000.0;

    let block_admission_start = Instant::now();
    let block_admission_result = if emit_stage_timings {
        blockchain.add_block_with_timings(turtle_data).map(Some)
    } else {
        blockchain.add_block(turtle_data).map(|_| None)
    };
    match block_admission_result {
        Ok(admission_timings) => {
            let block_admission_ms = block_admission_start.elapsed().as_secs_f64() * 1000.0;
            let block_hash = blockchain
                .chain
                .last()
                .map(|b| b.hash.clone())
                .unwrap_or_else(|| "unknown".to_string());

            let mut response = serde_json::json!({
                "success": true,
                "block_hash": block_hash,
                "block_index": blockchain.chain.len() - 1,
                "added_by": claims.sub,
                "timestamp": Utc::now(),
                "validation_status": "passed",
                "triple_count": request.triples.len(),
                "block_count": 1
            });
            if emit_stage_timings {
                response["timings_ms"] = build_handler_timings(
                    request_validation_ms,
                    blockchain_lock_wait_ms,
                    turtle_materialization_ms,
                    block_admission_ms,
                    request_start.elapsed().as_secs_f64() * 1000.0,
                    admission_timings.as_ref(),
                    "batch_block",
                );
            }

            Ok(Json(response))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: "blockchain_error".to_string(),
                message: format!("Failed to add batch transaction to blockchain: {}", e),
                timestamp: Utc::now(),
            }),
        )),
    }
}

/// Check benchmark policy semantics for governance/policy parity workloads.
pub async fn check_policy(
    Extension(claims): Extension<UserClaims>,
    Json(request): Json<PolicyCheckRequest>,
) -> Result<Json<PolicyCheckResponse>, (StatusCode, Json<ApiError>)> {
    if request.record_id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "invalid_record_id".to_string(),
                message: "record_id is required".to_string(),
                timestamp: Utc::now(),
            }),
        ));
    }

    if request.actor_org.trim().is_empty() || request.owner_org.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "invalid_policy_actor".to_string(),
                message: "actor_org and owner_org are required".to_string(),
                timestamp: Utc::now(),
            }),
        ));
    }

    let action = request.action.trim().to_ascii_lowercase();
    if action != "read" && action != "write" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "invalid_policy_action".to_string(),
                message: "action must be read or write".to_string(),
                timestamp: Utc::now(),
            }),
        ));
    }

    let visibility = request.visibility.trim().to_ascii_lowercase();
    let policy_start = Instant::now();
    let authorized = match visibility.as_str() {
        "public" => true,
        "restricted" => request.actor_org == request.owner_org || request.actor_org == "AuditorMSP",
        "private" => request.actor_org == request.owner_org,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: "invalid_policy_visibility".to_string(),
                    message: "visibility must be public, restricted, or private".to_string(),
                    timestamp: Utc::now(),
                }),
            ));
        }
    };

    Ok(Json(PolicyCheckResponse {
        authorized,
        policy_latency_ms: policy_start.elapsed().as_secs_f64() * 1000.0,
        policy_engine: "provchain-benchmark-policy-v1".to_string(),
        evaluated_by: claims.sub,
        user_role: claims.role,
    }))
}

/// Create a new transaction
pub async fn create_transaction(
    State(app_state): State<AppState>,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<Json<CreateTransactionResponse>, (StatusCode, Json<ApiError>)> {
    if request.rdf_data.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "empty_rdf_data".to_string(),
                message: "Transaction RDF data must not be empty".to_string(),
                timestamp: Utc::now(),
            }),
        ));
    }

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
        environmental_conditions: request.metadata.environmental_conditions.map(|ec| {
            EnvironmentalConditions {
                temperature: ec.temperature,
                humidity: ec.humidity,
                pressure: ec.pressure,
                timestamp: ec.timestamp,
                sensor_id: ec.sensor_id,
            }
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
    let inputs = request
        .inputs
        .into_iter()
        .map(|input| TransactionInput {
            prev_tx_id: input.prev_tx_id,
            output_index: input.output_index,
            signature: None,
            public_key: None,
        })
        .collect();

    let outputs = request
        .outputs
        .into_iter()
        .map(|output| {
            let owner = uuid::Uuid::parse_str(&output.owner).map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        error: "invalid_output_owner".to_string(),
                        message: format!("Invalid output owner UUID: {}", output.owner),
                        timestamp: Utc::now(),
                    }),
                )
            })?;

            Ok(TransactionOutput {
                id: output.id,
                owner,
                asset_type: output.asset_type,
                value: output.value,
                metadata: output.metadata,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Create transaction
    let transaction = Transaction::new(
        tx_type,
        inputs,
        outputs,
        request.rdf_data,
        None,
        metadata,
        TransactionPayload::RdfData(String::new()),
    );

    let tx_id = transaction.id.clone();

    let mut pending_transactions = app_state.pending_transactions.write().await;
    pending_transactions.insert(tx_id.clone(), transaction);

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
    State(app_state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
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

    authorize_participant_signer(&claims, participant_id)?;

    let mut pending_transactions = app_state.pending_transactions.write().await;
    let transaction = pending_transactions.get_mut(&tx_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "transaction_not_found".to_string(),
                message: format!("Pending transaction '{}' was not found", tx_id),
                timestamp: Utc::now(),
            }),
        )
    })?;

    let hash = transaction.calculate_hash().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "transaction_hash_failed".to_string(),
                message: format!("Failed to calculate transaction hash: {}", e),
                timestamp: Utc::now(),
            }),
        )
    })?;

    let mut wallet_manager = app_state.wallet_manager.write().await;
    if wallet_manager.get_wallet(participant_id).is_none() {
        let _ = wallet_manager.load_wallet(participant_id);
    }

    let (signature, public_key, wallet_snapshot) = {
        let wallet = wallet_manager
            .get_wallet_mut(participant_id)
            .ok_or_else(|| {
                (
                    StatusCode::NOT_FOUND,
                    Json(ApiError {
                        error: "wallet_not_found".to_string(),
                        message: format!(
                            "Wallet for participant '{}' was not found",
                            participant_id
                        ),
                        timestamp: Utc::now(),
                    }),
                )
            })?;

        let signature = wallet.sign(hash.as_bytes()).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "transaction_signing_failed".to_string(),
                    message: format!("Failed to sign transaction: {}", e),
                    timestamp: Utc::now(),
                }),
            )
        })?;
        let public_key = wallet.public_key;
        wallet.update_activity();
        (signature, public_key, wallet.clone())
    };

    wallet_manager.save_wallet(&wallet_snapshot).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: "wallet_persistence_failed".to_string(),
                message: format!("Failed to persist wallet activity: {}", e),
                timestamp: Utc::now(),
            }),
        )
    })?;

    transaction.signatures.push(TransactionSignature {
        signature,
        public_key,
        signer_id: participant_id,
        timestamp: Utc::now(),
    });

    let response = SignTransactionResponse {
        tx_id: tx_id.clone(),
        signatures: transaction
            .signatures
            .iter()
            .map(|signature| crate::web::models::TransactionSignatureInfo {
                signer_id: signature.signer_id.to_string(),
                timestamp: signature.timestamp,
            })
            .collect(),
        message: "Transaction signed successfully".to_string(),
        timestamp: Utc::now(),
    };

    println!(
        "Signed transaction {} with participant {}",
        tx_id, participant_id
    );

    Ok(Json(response))
}

/// Submit a signed transaction to the blockchain
pub async fn submit_transaction(
    State(app_state): State<AppState>,
    Json(request): Json<SubmitTransactionRequest>,
) -> Result<Json<SubmitTransactionResponse>, (StatusCode, Json<ApiError>)> {
    let tx_id = request.tx_id;

    let transaction = {
        let pending_transactions = app_state.pending_transactions.read().await;
        pending_transactions.get(&tx_id).cloned().ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ApiError {
                    error: "transaction_not_found".to_string(),
                    message: format!("Pending transaction '{}' was not found", tx_id),
                    timestamp: Utc::now(),
                }),
            )
        })?
    };

    transaction.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "transaction_validation_failed".to_string(),
                message: format!("Transaction validation failed: {}", e),
                timestamp: Utc::now(),
            }),
        )
    })?;

    let block_index = {
        let mut blockchain = app_state.blockchain.write().await;
        blockchain.add_block(transaction.to_rdf()).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "blockchain_submission_failed".to_string(),
                    message: format!("Failed to submit transaction to blockchain: {}", e),
                    timestamp: Utc::now(),
                }),
            )
        })?;

        blockchain.chain.last().map(|block| block.index as usize)
    };

    let mut pending_transactions = app_state.pending_transactions.write().await;
    pending_transactions.remove(&tx_id);

    let response = SubmitTransactionResponse {
        tx_id: tx_id.clone(),
        block_index,
        message: "Transaction submitted successfully".to_string(),
        timestamp: Utc::now(),
    };

    println!("Submitted transaction {} to blockchain", tx_id);

    Ok(Json(response))
}

/// Register a new wallet for a participant
pub async fn register_wallet(
    State(app_state): State<AppState>,
    Json(request): Json<WalletRegistrationRequest>,
) -> Result<Json<WalletRegistrationResponse>, (StatusCode, Json<ApiError>)> {
    let _blockchain = app_state.blockchain.write().await;

    let participant_type = match request.participant_type.to_lowercase().as_str() {
        "producer" => ParticipantType::Producer,
        "manufacturer" => ParticipantType::Manufacturer,
        "logistics" => ParticipantType::LogisticsProvider,
        "quality" => ParticipantType::QualityLab,
        "auditor" => ParticipantType::Auditor,
        "retailer" => ParticipantType::Retailer,
        _ => ParticipantType::Producer, // Default
    };

    let participant = Participant {
        id: uuid::Uuid::new_v4(),
        name: request.name,
        participant_type,
        contact_info: request
            .contact_info
            .map(|c| ContactInfo {
                email: c.email,
                phone: c.phone,
                address: c.address,
                website: c.website,
            })
            .unwrap_or(ContactInfo {
                email: None,
                phone: None,
                address: None,
                website: None,
            }),
        location: request.location,
        permissions: crate::wallet::ParticipantPermissions::for_type(
            &crate::wallet::ParticipantType::Producer,
        ), // Simplified
        certificates: vec![],
        registered_at: Utc::now(),
        last_activity: None,
        reputation: 1.0,
        metadata: std::collections::HashMap::new(),
    };

    let participant_id = participant.id.to_string();

    // In a real implementation, we'd add to a wallet manager
    // For now, we simulate success

    Ok(Json(WalletRegistrationResponse {
        participant_id,
        public_key: "SIMULATED_PUBLIC_KEY".to_string(),
        message: "Wallet registered successfully".to_string(),
        timestamp: Utc::now(),
    }))
}

/// Create a new participant (legacy/alternative endpoint)
pub async fn create_participant(
    State(_app_state): State<AppState>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    // Simplified simulation
    Ok(Json(serde_json::json!({
        "success": true,
        "participant_id": uuid::Uuid::new_v4().to_string(),
        "message": "Participant created successfully"
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RuntimeMode;
    use crate::core::blockchain::Blockchain;
    use crate::wallet::WalletManager;
    use std::collections::HashMap;
    use std::sync::{atomic::AtomicU64, Arc};
    use tempfile::{tempdir, TempDir};
    use tokio::sync::RwLock;

    fn test_state_with_wallet_manager(wallet_manager: WalletManager) -> AppState {
        AppState {
            blockchain: Arc::new(RwLock::new(Blockchain::new())),
            network_peers: Arc::new(AtomicU64::new(0)),
            wallet_manager: Arc::new(RwLock::new(wallet_manager)),
            pending_transactions: Arc::new(RwLock::new(HashMap::new())),
            runtime_mode: RuntimeMode::Production,
        }
    }

    #[test]
    fn validates_non_empty_turtle_import_payload() {
        let request = TurtleImportRequest {
            turtle_data: r#"
                @prefix ex: <http://example.com/> .
                ex:batch1 ex:status "ok" .
            "#
            .to_string(),
        };

        let count = validate_turtle_import_request(&request)
            .expect("valid Turtle import payload should pass");

        assert_eq!(count, 1);
    }

    #[test]
    fn rejects_empty_turtle_import_payload() {
        let request = TurtleImportRequest {
            turtle_data: "   \n\t  ".to_string(),
        };

        let (status, Json(error)) = validate_turtle_import_request(&request)
            .expect_err("empty Turtle import payload should fail");

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(error.error, "empty_turtle_import");
    }

    #[test]
    fn rejects_invalid_turtle_import_payload() {
        let request = TurtleImportRequest {
            turtle_data: "@prefix ex: <http://example.com/> . ex:batch1 ex:status".to_string(),
        };

        let (status, Json(error)) = validate_turtle_import_request(&request)
            .expect_err("invalid Turtle import payload should fail");

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(error.error, "invalid_turtle_import");
    }

    #[tokio::test]
    async fn privacy_key_lookup_requires_registered_wallet_secret() {
        let state = test_state_with_wallet_manager(
            WalletManager::new(tempdir().unwrap().path(), None).unwrap(),
        );
        let claims = UserClaims {
            sub: Uuid::new_v4().to_string(),
            role: "user".to_string(),
            exp: usize::MAX,
        };

        let (status, Json(error)) = load_privacy_key_for_claim(&state, &claims, "key-1")
            .await
            .expect_err("missing wallet/key should fail closed");

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(error.error, "privacy_key_not_found");
    }

    fn minimal_transaction(rdf_data: &str) -> Transaction {
        Transaction::new(
            TransactionType::Production,
            vec![],
            vec![],
            rdf_data.to_string(),
            None,
            TransactionMetadata {
                location: None,
                environmental_conditions: None,
                compliance_info: None,
                quality_data: None,
                custom_fields: HashMap::new(),
            },
            TransactionPayload::RdfData(String::new()),
        )
    }

    fn test_claims(sub: impl Into<String>) -> UserClaims {
        UserClaims {
            sub: sub.into(),
            role: "user".to_string(),
            exp: usize::MAX,
        }
    }

    async fn state_with_wallet_and_pending_transaction() -> (AppState, Uuid, String, TempDir) {
        let wallet_dir = tempdir().unwrap();
        let mut wallet_manager = WalletManager::new(wallet_dir.path(), None).unwrap();
        let participant = Participant::new_farmer("test farm".to_string(), "test".to_string());
        let participant_id = participant.id;
        wallet_manager.create_wallet(participant).unwrap();
        let state = test_state_with_wallet_manager(wallet_manager);
        let transaction = minimal_transaction(
            r#"@prefix ex: <http://example.com/> .
ex:tx ex:status "created" ."#,
        );
        let tx_id = transaction.id.clone();
        state
            .pending_transactions
            .write()
            .await
            .insert(tx_id.clone(), transaction);
        (state, participant_id, tx_id, wallet_dir)
    }

    #[tokio::test]
    async fn privacy_key_lookup_uses_wallet_secret_without_generating_key() {
        let wallet_dir = tempdir().unwrap();
        let mut wallet_manager = WalletManager::new(wallet_dir.path(), None).unwrap();
        let participant = Participant::new_farmer("test farm".to_string(), "test".to_string());
        let participant_id = participant.id;
        wallet_manager.create_wallet(participant).unwrap();
        wallet_manager
            .get_wallet_mut(participant_id)
            .unwrap()
            .add_secret("key-1".to_string(), hex::encode([7u8; 32]));

        let state = test_state_with_wallet_manager(wallet_manager);
        let claims = UserClaims {
            sub: participant_id.to_string(),
            role: "user".to_string(),
            exp: usize::MAX,
        };

        let key = load_privacy_key_for_claim(&state, &claims, "key-1")
            .await
            .expect("registered wallet secret should be accepted");

        assert_eq!(key, [7u8; 32]);
    }

    #[tokio::test]
    async fn sign_transaction_rejects_participant_mismatch_before_signing() {
        let (state, participant_id, tx_id, _wallet_dir) =
            state_with_wallet_and_pending_transaction().await;
        let other_participant_id = Uuid::new_v4();

        let (status, Json(error)) = sign_transaction(
            State(state),
            Extension(test_claims(other_participant_id.to_string())),
            Json(SignTransactionRequest {
                tx_id,
                participant_id: participant_id.to_string(),
            }),
        )
        .await
        .expect_err("mismatched participant signer should fail closed");

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(error.error, "signer_not_authorized");
    }

    #[tokio::test]
    async fn sign_transaction_rejects_unauthorized_nonexistent_tx_without_oracle() {
        let (state, participant_id, _tx_id, _wallet_dir) =
            state_with_wallet_and_pending_transaction().await;
        let other_participant_id = Uuid::new_v4();

        let (status, Json(error)) = sign_transaction(
            State(state),
            Extension(test_claims(other_participant_id.to_string())),
            Json(SignTransactionRequest {
                tx_id: "missing-transaction".to_string(),
                participant_id: participant_id.to_string(),
            }),
        )
        .await
        .expect_err("authorization must happen before transaction lookup");

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(error.error, "signer_not_authorized");
    }

    #[tokio::test]
    async fn sign_transaction_accepts_matching_participant_claim() {
        let (state, participant_id, tx_id, _wallet_dir) =
            state_with_wallet_and_pending_transaction().await;

        let response = sign_transaction(
            State(state),
            Extension(test_claims(participant_id.to_string())),
            Json(SignTransactionRequest {
                tx_id,
                participant_id: participant_id.to_string(),
            }),
        )
        .await
        .expect("matching participant claim should sign");

        assert_eq!(response.0.signatures.len(), 1);
        assert_eq!(
            response.0.signatures[0].signer_id,
            participant_id.to_string()
        );
    }

    #[tokio::test]
    async fn sign_transaction_rejects_non_uuid_claim_subject() {
        let (state, participant_id, tx_id, _wallet_dir) =
            state_with_wallet_and_pending_transaction().await;

        let (status, Json(error)) = sign_transaction(
            State(state),
            Extension(test_claims("username-subject")),
            Json(SignTransactionRequest {
                tx_id,
                participant_id: participant_id.to_string(),
            }),
        )
        .await
        .expect_err("username subject must not imply participant authority");

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(error.error, "signer_not_authorized");
    }
}
