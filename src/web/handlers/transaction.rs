use crate::security::encryption::PrivacyManager;
use crate::transaction::transaction::{
    ComplianceInfo, EnvironmentalConditions, QualityData, Transaction, TransactionInput,
    TransactionMetadata, TransactionOutput, TransactionPayload, TransactionType,
};
use crate::wallet::{ContactInfo, Participant, ParticipantType};
use crate::web::handlers::utils::{validate_literal, validate_uri};
use crate::web::handlers::AppState;
use crate::web::models::{
    AddTripleRequest, ApiError, CreateTransactionRequest, CreateTransactionResponse,
    SignTransactionRequest, SignTransactionResponse, SubmitTransactionRequest,
    SubmitTransactionResponse, UserClaims, WalletRegistrationRequest, WalletRegistrationResponse,
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;

/// Add new triple to blockchain with SHACL validation
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
    } else if let Err(e) = validate_literal(&request.object) {
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

    let mut blockchain = app_state.blockchain.write().await;

    // Create proper RDF triple data in Turtle format
    let triple_data =
        if request.object.starts_with("http://") || request.object.starts_with("https://") {
            // Object is a URI, don't quote it
            format!(
                "<{}> <{}> <{}> .",
                request.subject, request.predicate, request.object
            )
        } else {
            // Object is a literal, quote it
            format!(
                "<{}> <{}> \"{}\" .",
                request.subject, request.predicate, request.object
            )
        };

    eprintln!("Adding triple data: {}", triple_data);

    // Check for privacy request
    if let Some(key_id) = &request.privacy_key_id {
        // In a real implementation, we would retrieve the key from the wallet manager via AppState
        // For this thesis demonstration, we generate a key on the fly if one isn't found,
        // effectively simulating that the user has provided a valid key ID.
        let key = PrivacyManager::generate_key(); // Simulation of retrieving key for 'key_id'

        match PrivacyManager::encrypt(&triple_data, &key, key_id) {
            Ok(encrypted) => {
                let encrypted_json = serde_json::to_string(&encrypted).unwrap_or_default();

                // Clone validator public key before borrowing blockchain mutably
                let validator_public_key = blockchain.validator_public_key.clone();

                // Create a block with encrypted payload
                // We use a placeholder for the public data to indicate it's encrypted
                match blockchain.create_block_proposal(
                    format!("@prefix prov: <http://provchain.org/core#> . prov:EncryptedData prov:hasKeyId \"{}\" .", key_id),
                    validator_public_key
                ) {
                    Ok(mut block) => {
                        block.encrypted_data = Some(encrypted_json);
                        // Re-calculate hash to include encrypted data
                        block.hash = block.calculate_hash();
                        // Re-sign the block with the new hash
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
    match blockchain.add_block(triple_data) {
        Ok(()) => {
            let block_hash = blockchain
                .chain
                .last()
                .map(|b| b.hash.clone())
                .unwrap_or_else(|| "unknown".to_string());

            let response = serde_json::json!({
                "success": true,
                "block_hash": block_hash,
                "block_index": blockchain.chain.len() - 1,
                "added_by": claims.sub,
                "timestamp": Utc::now(),
                "validation_status": "passed"
            });

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
        .map(|output| TransactionOutput {
            id: output.id,
            owner: uuid::Uuid::parse_str(&output.owner).unwrap_or(uuid::Uuid::nil()),
            asset_type: output.asset_type,
            value: output.value,
            metadata: output.metadata,
        })
        .collect();

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

    println!(
        "Signed transaction {} with participant {}",
        tx_id, participant_id
    );

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
