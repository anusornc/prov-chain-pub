//! WebSocket server implementation for real-time blockchain events

use crate::core::blockchain::Blockchain;
use crate::error::WebError;
use crate::web::auth::validate_token;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State, Query,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// WebSocket client information
#[derive(Debug, Clone)]
pub struct WebSocketClient {
    pub id: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub last_ping: chrono::DateTime<chrono::Utc>,
}

/// Blockchain events that can be broadcast to WebSocket clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum BlockchainEvent {
    /// New block was created and added to the blockchain
    BlockCreated {
        block_index: u64,
        block_hash: String,
        timestamp: String,
        transaction_count: usize,
    },
    /// Transaction was submitted to the transaction pool
    TransactionSubmitted {
        transaction_id: String,
        transaction_type: String,
        participant: String,
        timestamp: String,
    },
    /// Transaction was processed and included in a block
    TransactionProcessed {
        transaction_id: String,
        block_index: u64,
        status: String,
    },
    /// Blockchain validation completed
    ValidationComplete {
        block_index: u64,
        is_valid: bool,
        validation_time_ms: u64,
    },
    /// Integrity monitoring alert
    IntegrityAlert {
        level: String,
        message: String,
        timestamp: String,
        block_index: Option<u64>,
    },
    /// System status update
    SystemStatus {
        blockchain_height: u64,
        total_transactions: u64,
        active_participants: u64,
        system_health: String,
    },
    /// Real-time metrics update
    MetricsUpdate {
        blocks_per_minute: f64,
        transactions_per_minute: f64,
        average_block_time: f64,
        validation_performance: String,
    },
}

/// WebSocket message types for client-server communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WebSocketMessage {
    /// Client subscribes to specific event types
    Subscribe { events: Vec<String> },
    /// Client unsubscribes from event types
    Unsubscribe { events: Vec<String> },
    /// Heartbeat/ping message
    Ping { timestamp: String },
    /// Pong response to ping
    Pong { timestamp: String },
    /// Blockchain event notification
    Event(BlockchainEvent),
    /// Error message
    Error { message: String },
    /// Connection acknowledgment
    Connected { client_id: String },
}

/// WebSocket server state
#[derive(Clone)]
pub struct WebSocketState {
    pub clients: Arc<Mutex<HashMap<String, WebSocketClient>>>,
    pub event_sender: broadcast::Sender<BlockchainEvent>,
    pub blockchain: Arc<Mutex<Blockchain>>,
}

impl WebSocketState {
    /// Create new WebSocket state
    pub fn new(blockchain: Arc<Mutex<Blockchain>>) -> Self {
        let (event_sender, _) = broadcast::channel(1000);

        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            event_sender,
            blockchain,
        }
    }

    /// Broadcast event to all connected clients
    pub fn broadcast_event(&self, event: BlockchainEvent) {
        match self.event_sender.send(event.clone()) {
            Ok(receiver_count) => {
                debug!(
                    "Broadcasted event to {} clients: {:?}",
                    receiver_count, event
                );
            }
            Err(e) => {
                warn!("Failed to broadcast event: {}", e);
            }
        }
    }

    /// Get current client count
    pub fn client_count(&self) -> usize {
        self.clients.lock().unwrap().len()
    }

    /// Add new client
    pub fn add_client(&self, client_id: String) {
        let client = WebSocketClient {
            id: client_id.clone(),
            connected_at: chrono::Utc::now(),
            last_ping: chrono::Utc::now(),
        };

        self.clients
            .lock()
            .unwrap()
            .insert(client_id.clone(), client);
        info!("WebSocket client connected: {}", client_id);

        // Broadcast system status to new client
        if let Ok(blockchain) = self.blockchain.lock() {
            let status_event = BlockchainEvent::SystemStatus {
                blockchain_height: blockchain.get_latest_block_index(),
                total_transactions: blockchain.get_transaction_count() as u64,
                active_participants: blockchain.get_participant_count() as u64,
                system_health: "healthy".to_string(),
            };
            self.broadcast_event(status_event);
        }
    }

    /// Remove client
    pub fn remove_client(&self, client_id: &str) {
        self.clients.lock().unwrap().remove(client_id);
        info!("WebSocket client disconnected: {}", client_id);
    }

    /// Update client ping time
    pub fn update_client_ping(&self, client_id: &str) {
        if let Some(client) = self.clients.lock().unwrap().get_mut(client_id) {
            client.last_ping = chrono::Utc::now();
        }
    }
}

/// WebSocket upgrade handler with JWT authentication
#[derive(Debug, Deserialize)]
pub struct WebSocketAuthQuery {
    token: Option<String>,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketState>,
    Query(query): Query<WebSocketAuthQuery>,
) -> Response {
    // Extract token from query parameter
    let token = if let Some(token) = query.token {
        token
    } else {
        // Try to get token from Authorization header
        return ws.on_upgrade(move |socket| handle_websocket_unauthenticated(socket, state));
    };

    // Validate JWT token
    match validate_token(&token) {
        Ok(_claims) => {
            // Token is valid, proceed with WebSocket connection
            ws.on_upgrade(move |socket| handle_websocket(socket, state))
        }
        Err(e) => {
            // Token is invalid, return error
            return (StatusCode::UNAUTHORIZED, format!("Invalid JWT token: {}", e)).into_response();
        }
    }
}

/// Handle unauthorized WebSocket connection (sends disconnect message)
async fn handle_websocket_unauthenticated(socket: WebSocket, state: WebSocketState) {
    let client_id = Uuid::new_v4().to_string();

    // Split socket into sender and receiver
    let (mut sender, mut _receiver) = socket.split();

    // Send authentication failure message
    let error_msg = WebSocketMessage::Error {
        message: "Authentication required. Provide valid JWT token via ?token= query parameter.".to_string(),
    };

    if let Ok(msg_text) = serde_json::to_string(&error_msg) {
        let _ = sender.send(Message::Text(msg_text)).await;
    }

    // Close the connection
    let _ = sender.close().await;

    state.remove_client(&client_id);
}

/// Handle individual WebSocket connection
async fn handle_websocket(socket: WebSocket, state: WebSocketState) {
    let client_id = Uuid::new_v4().to_string();

    // Add client to state
    state.add_client(client_id.clone());

    // Split socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to blockchain events
    let mut event_receiver = state.event_sender.subscribe();

    // Send connection acknowledgment
    let connect_msg = WebSocketMessage::Connected {
        client_id: client_id.clone(),
    };

    if let Ok(msg_text) = serde_json::to_string(&connect_msg) {
        if sender.send(Message::Text(msg_text)).await.is_err() {
            error!(
                "Failed to send connection acknowledgment to client {}",
                client_id
            );
            state.remove_client(&client_id);
            return;
        }
    }

    // Spawn task to handle outgoing messages (events to client)
    let outgoing_client_id = client_id.clone();
    let outgoing_state = state.clone();
    let outgoing_task = tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            let message = WebSocketMessage::Event(event);

            if let Ok(msg_text) = serde_json::to_string(&message) {
                if sender.send(Message::Text(msg_text)).await.is_err() {
                    debug!(
                        "Client {} disconnected during event send",
                        outgoing_client_id
                    );
                    break;
                }
            }
        }

        outgoing_state.remove_client(&outgoing_client_id);
    });

    // Handle incoming messages from client
    let incoming_client_id = client_id.clone();
    let incoming_state = state.clone();
    let incoming_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) =
                        handle_client_message(&text, &incoming_client_id, &incoming_state).await
                    {
                        warn!("Error handling client message: {}", e);
                    }
                }
                Ok(Message::Binary(_)) => {
                    debug!(
                        "Received binary message from client {}, ignoring",
                        incoming_client_id
                    );
                }
                Ok(Message::Ping(_data)) => {
                    debug!("Received ping from client {}", incoming_client_id);
                    incoming_state.update_client_ping(&incoming_client_id);
                    // Pong is automatically sent by axum
                }
                Ok(Message::Pong(_)) => {
                    debug!("Received pong from client {}", incoming_client_id);
                    incoming_state.update_client_ping(&incoming_client_id);
                }
                Ok(Message::Close(_)) => {
                    info!("Client {} requested connection close", incoming_client_id);
                    break;
                }
                Err(e) => {
                    warn!("WebSocket error for client {}: {}", incoming_client_id, e);
                    break;
                }
            }
        }

        incoming_state.remove_client(&incoming_client_id);
    });

    // Wait for either task to complete
    tokio::select! {
        _ = outgoing_task => {
            debug!("Outgoing task completed for client {}", client_id);
        }
        _ = incoming_task => {
            debug!("Incoming task completed for client {}", client_id);
        }
    }

    // Ensure client is removed
    state.remove_client(&client_id);
}

/// Handle incoming message from WebSocket client
async fn handle_client_message(
    message: &str,
    client_id: &str,
    state: &WebSocketState,
) -> Result<(), WebError> {
    let parsed_message: WebSocketMessage = serde_json::from_str(message)
        .map_err(|e| WebError::BadRequest(format!("Invalid message format: {}", e)))?;

    match parsed_message {
        WebSocketMessage::Subscribe { events } => {
            debug!("Client {} subscribed to events: {:?}", client_id, events);
            // In a more complex implementation, we would track per-client subscriptions
            // For now, all clients receive all events
        }
        WebSocketMessage::Unsubscribe { events } => {
            debug!(
                "Client {} unsubscribed from events: {:?}",
                client_id, events
            );
            // In a more complex implementation, we would update per-client subscriptions
        }
        WebSocketMessage::Ping { timestamp } => {
            debug!("Received ping from client {} at {}", client_id, timestamp);
            state.update_client_ping(client_id);

            // Send pong response
            let _pong_msg = WebSocketMessage::Pong {
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            // Note: In a real implementation, we'd need to send this back to the specific client
            // This would require maintaining per-client senders
        }
        _ => {
            warn!(
                "Unexpected message type from client {}: {:?}",
                client_id, parsed_message
            );
        }
    }

    Ok(())
}

/// Blockchain event broadcaster - integrates with existing blockchain operations
pub struct BlockchainEventBroadcaster {
    websocket_state: WebSocketState,
}

impl BlockchainEventBroadcaster {
    /// Create new event broadcaster
    pub fn new(websocket_state: WebSocketState) -> Self {
        Self { websocket_state }
    }

    /// Broadcast block creation event
    pub fn broadcast_block_created(
        &self,
        block_index: u64,
        block_hash: String,
        transaction_count: usize,
    ) {
        let event = BlockchainEvent::BlockCreated {
            block_index,
            block_hash,
            timestamp: chrono::Utc::now().to_rfc3339(),
            transaction_count,
        };

        self.websocket_state.broadcast_event(event);
    }

    /// Broadcast transaction submission event
    pub fn broadcast_transaction_submitted(
        &self,
        transaction_id: String,
        transaction_type: String,
        participant: String,
    ) {
        let event = BlockchainEvent::TransactionSubmitted {
            transaction_id,
            transaction_type,
            participant,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.websocket_state.broadcast_event(event);
    }

    /// Broadcast validation completion event
    pub fn broadcast_validation_complete(
        &self,
        block_index: u64,
        is_valid: bool,
        validation_time_ms: u64,
    ) {
        let event = BlockchainEvent::ValidationComplete {
            block_index,
            is_valid,
            validation_time_ms,
        };

        self.websocket_state.broadcast_event(event);
    }

    /// Broadcast integrity alert
    pub fn broadcast_integrity_alert(
        &self,
        level: String,
        message: String,
        block_index: Option<u64>,
    ) {
        let event = BlockchainEvent::IntegrityAlert {
            level,
            message,
            timestamp: chrono::Utc::now().to_rfc3339(),
            block_index,
        };

        self.websocket_state.broadcast_event(event);
    }

    /// Broadcast system metrics update
    pub fn broadcast_metrics_update(
        &self,
        blocks_per_minute: f64,
        transactions_per_minute: f64,
        average_block_time: f64,
        validation_performance: String,
    ) {
        let event = BlockchainEvent::MetricsUpdate {
            blocks_per_minute,
            transactions_per_minute,
            average_block_time,
            validation_performance,
        };

        self.websocket_state.broadcast_event(event);
    }

    /// Get current client count
    pub fn client_count(&self) -> usize {
        self.websocket_state.client_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blockchain::Blockchain;

    #[tokio::test]
    async fn test_websocket_state_creation() {
        let blockchain = Arc::new(Mutex::new(Blockchain::new()));
        let state = WebSocketState::new(blockchain);

        assert_eq!(state.client_count(), 0);
    }

    #[tokio::test]
    async fn test_client_management() {
        let blockchain = Arc::new(Mutex::new(Blockchain::new()));
        let state = WebSocketState::new(blockchain);

        let client_id = "test-client-123".to_string();
        state.add_client(client_id.clone());

        assert_eq!(state.client_count(), 1);

        state.remove_client(&client_id);
        assert_eq!(state.client_count(), 0);
    }

    #[tokio::test]
    async fn test_event_broadcasting() {
        let blockchain = Arc::new(Mutex::new(Blockchain::new()));
        let state = WebSocketState::new(blockchain);

        let event = BlockchainEvent::BlockCreated {
            block_index: 1,
            block_hash: "test-hash".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            transaction_count: 5,
        };

        // This should not panic even with no subscribers
        state.broadcast_event(event);
    }

    #[test]
    fn test_blockchain_event_serialization() {
        let event = BlockchainEvent::BlockCreated {
            block_index: 1,
            block_hash: "test-hash".to_string(),
            timestamp: "2025-08-31T13:00:00Z".to_string(),
            transaction_count: 5,
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: BlockchainEvent = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            BlockchainEvent::BlockCreated { block_index, .. } => {
                assert_eq!(block_index, 1);
            }
            _ => panic!("Unexpected event type"),
        }
    }

    #[test]
    fn test_websocket_message_serialization() {
        let message = WebSocketMessage::Connected {
            client_id: "test-123".to_string(),
        };

        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            WebSocketMessage::Connected { client_id } => {
                assert_eq!(client_id, "test-123");
            }
            _ => panic!("Unexpected message type"),
        }
    }
}
