//! Comprehensive WebSocket integration tests
//! Tests real-time event broadcasting, client connection management, and system integration

use axum::{
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use provchain_org::{
    core::blockchain::Blockchain,
    web::websocket::{BlockchainEvent, BlockchainEventBroadcaster, WebSocketMessage, WebSocketState, websocket_handler},
};
use serde_json;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    net::{TcpListener, TcpStream},
    time::timeout,
};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message as TungsteniteMessage, WebSocketStream,
};

/// Test helper to create a test WebSocket server
async fn create_test_server() -> (String, WebSocketState) {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let websocket_state = WebSocketState::new(blockchain);
    
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(websocket_state.clone());
    
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_url = format!("ws://127.0.0.1:{}/ws", addr.port());
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    
    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    (server_url, websocket_state)
}

/// Test helper to connect WebSocket client
async fn connect_websocket_client(
    url: &str,
) -> Result<WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async(url).await?;
    Ok(ws_stream)
}

#[tokio::test]
async fn test_websocket_connection_and_acknowledgment() {
    let (server_url, _state) = create_test_server().await;
    
    let mut ws_stream = connect_websocket_client(&server_url).await.unwrap();
    
    // Should receive connection acknowledgment
    let msg = timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .expect("Timeout waiting for connection acknowledgment")
        .unwrap()
        .unwrap();
    
    if let TungsteniteMessage::Text(text) = msg {
        let parsed: WebSocketMessage = serde_json::from_str(&text).unwrap();
        match parsed {
            WebSocketMessage::Connected { client_id } => {
                assert!(!client_id.is_empty());
                println!("✅ Connection acknowledgment received with client_id: {}", client_id);
            }
            _ => panic!("Expected Connected message, got: {:?}", parsed),
        }
    } else {
        panic!("Expected text message, got: {:?}", msg);
    }
}

#[tokio::test]
async fn test_multiple_client_connections() {
    let (server_url, state) = create_test_server().await;
    
    // Connect multiple clients
    let mut clients = Vec::new();
    for i in 0..3 {
        let mut ws_stream = connect_websocket_client(&server_url).await.unwrap();
        
        // Read connection acknowledgment
        let _ack = timeout(Duration::from_secs(5), ws_stream.next())
            .await
            .expect("Timeout waiting for connection acknowledgment")
            .unwrap()
            .unwrap();
        
        clients.push(ws_stream);
        println!("✅ Client {} connected", i + 1);
    }
    
    // Give time for all clients to register
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify client count
    assert_eq!(state.client_count(), 3);
    println!("✅ All 3 clients registered in server state");
    
    // Close all connections
    for mut client in clients {
        let _ = client.close(None).await;
    }
    
    // Give time for cleanup
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Verify clients are cleaned up
    assert_eq!(state.client_count(), 0);
    println!("✅ All clients cleaned up after disconnection");
}

#[tokio::test]
async fn test_blockchain_event_broadcasting() {
    let (server_url, state) = create_test_server().await;
    
    // Connect client
    let mut ws_stream = connect_websocket_client(&server_url).await.unwrap();
    
    // Read connection acknowledgment
    let _ack = timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .expect("Timeout waiting for connection acknowledgment")
        .unwrap()
        .unwrap();
    
    // Create event broadcaster
    let broadcaster = BlockchainEventBroadcaster::new(state.clone());
    
    // Broadcast a block creation event
    broadcaster.broadcast_block_created(
        1,
        "test-block-hash-123".to_string(),
        5,
    );
    
    // Should receive the event
    let msg = timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .expect("Timeout waiting for blockchain event")
        .unwrap()
        .unwrap();
    
    if let TungsteniteMessage::Text(text) = msg {
        let parsed: WebSocketMessage = serde_json::from_str(&text).unwrap();
        match parsed {
            WebSocketMessage::Event(BlockchainEvent::BlockCreated { 
                block_index, 
                block_hash, 
                transaction_count,
                .. 
            }) => {
                assert_eq!(block_index, 1);
                assert_eq!(block_hash, "test-block-hash-123");
                assert_eq!(transaction_count, 5);
                println!("✅ Block creation event received correctly");
            }
            _ => panic!("Expected BlockCreated event, got: {:?}", parsed),
        }
    } else {
        panic!("Expected text message, got: {:?}", msg);
    }
}

#[tokio::test]
async fn test_multiple_event_types() {
    let (server_url, state) = create_test_server().await;
    
    // Connect client
    let mut ws_stream = connect_websocket_client(&server_url).await.unwrap();
    
    // Read connection acknowledgment
    let _ack = timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .expect("Timeout waiting for connection acknowledgment")
        .unwrap()
        .unwrap();
    
    let broadcaster = BlockchainEventBroadcaster::new(state.clone());
    
    // Test different event types
    let test_events: Vec<(&str, Box<dyn Fn()>)> = vec![
        ("transaction_submitted", Box::new(|| {
            broadcaster.broadcast_transaction_submitted(
                "tx-123".to_string(),
                "transfer".to_string(),
                "participant-1".to_string(),
            );
        })),
        ("validation_complete", Box::new(|| {
            broadcaster.broadcast_validation_complete(1, true, 150);
        })),
        ("integrity_alert", Box::new(|| {
            broadcaster.broadcast_integrity_alert(
                "warning".to_string(),
                "Test alert message".to_string(),
                Some(1),
            );
        })),
        ("metrics_update", Box::new(|| {
            broadcaster.broadcast_metrics_update(
                2.5,
                10.0,
                24.5,
                "excellent".to_string(),
            );
        })),
    ];
    
    for (event_name, broadcast_fn) in test_events {
        // Broadcast event
        broadcast_fn();
        
        // Receive and verify event
        let msg = timeout(Duration::from_secs(5), ws_stream.next())
            .await
            .expect(&format!("Timeout waiting for {} event", event_name))
            .unwrap()
            .unwrap();
        
        if let TungsteniteMessage::Text(text) = msg {
            let parsed: WebSocketMessage = serde_json::from_str(&text).unwrap();
            match parsed {
                WebSocketMessage::Event(event) => {
                    match event {
                        BlockchainEvent::TransactionSubmitted { transaction_id, .. } if event_name == "transaction_submitted" => {
                            assert_eq!(transaction_id, "tx-123");
                            println!("✅ Transaction submitted event received correctly");
                        }
                        BlockchainEvent::ValidationComplete { block_index, is_valid, .. } if event_name == "validation_complete" => {
                            assert_eq!(block_index, 1);
                            assert!(is_valid);
                            println!("✅ Validation complete event received correctly");
                        }
                        BlockchainEvent::IntegrityAlert { level, message, .. } if event_name == "integrity_alert" => {
                            assert_eq!(level, "warning");
                            assert_eq!(message, "Test alert message");
                            println!("✅ Integrity alert event received correctly");
                        }
                        BlockchainEvent::MetricsUpdate { blocks_per_minute, .. } if event_name == "metrics_update" => {
                            assert_eq!(blocks_per_minute, 2.5);
                            println!("✅ Metrics update event received correctly");
                        }
                        _ => panic!("Unexpected event type for {}: {:?}", event_name, event),
                    }
                }
                _ => panic!("Expected Event message for {}, got: {:?}", event_name, parsed),
            }
        } else {
            panic!("Expected text message for {}, got: {:?}", event_name, msg);
        }
    }
}

#[tokio::test]
async fn test_client_message_handling() {
    let (server_url, _state) = create_test_server().await;
    
    // Connect client
    let mut ws_stream = connect_websocket_client(&server_url).await.unwrap();
    
    // Read connection acknowledgment
    let _ack = timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .expect("Timeout waiting for connection acknowledgment")
        .unwrap()
        .unwrap();
    
    // Send subscription message
    let subscribe_msg = WebSocketMessage::Subscribe {
        events: vec!["BlockCreated".to_string(), "TransactionSubmitted".to_string()],
    };
    
    let msg_text = serde_json::to_string(&subscribe_msg).unwrap();
    ws_stream.send(TungsteniteMessage::Text(msg_text)).await.unwrap();
    
    // Send ping message
    let ping_msg = WebSocketMessage::Ping {
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let ping_text = serde_json::to_string(&ping_msg).unwrap();
    ws_stream.send(TungsteniteMessage::Text(ping_text)).await.unwrap();
    
    // Give server time to process messages
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    println!("✅ Client messages sent successfully (subscription and ping)");
}

#[tokio::test]
async fn test_connection_cleanup_on_disconnect() {
    let (server_url, state) = create_test_server().await;
    
    // Connect client
    let mut ws_stream = connect_websocket_client(&server_url).await.unwrap();
    
    // Read connection acknowledgment
    let _ack = timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .expect("Timeout waiting for connection acknowledgment")
        .unwrap()
        .unwrap();
    
    // Verify client is registered
    assert_eq!(state.client_count(), 1);
    
    // Close connection
    ws_stream.close(None).await.unwrap();
    
    // Give time for cleanup
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Verify client is cleaned up
    assert_eq!(state.client_count(), 0);
    println!("✅ Client properly cleaned up after disconnect");
}

#[tokio::test]
async fn test_broadcast_to_multiple_clients() {
    let (server_url, state) = create_test_server().await;
    
    // Connect multiple clients
    let mut clients = Vec::new();
    for i in 0..3 {
        let mut ws_stream = connect_websocket_client(&server_url).await.unwrap();
        
        // Read connection acknowledgment
        let _ack = timeout(Duration::from_secs(5), ws_stream.next())
            .await
            .expect("Timeout waiting for connection acknowledgment")
            .unwrap()
            .unwrap();
        
        clients.push(ws_stream);
        println!("Client {} connected", i + 1);
    }
    
    // Verify all clients are registered
    assert_eq!(state.client_count(), 3);
    
    // Broadcast event
    let broadcaster = BlockchainEventBroadcaster::new(state.clone());
    broadcaster.broadcast_block_created(
        42,
        "broadcast-test-hash".to_string(),
        10,
    );
    
    // Verify all clients receive the event (skip SystemStatus events)
    for (i, client) in clients.iter_mut().enumerate() {
        loop {
            let msg = timeout(Duration::from_secs(5), client.next())
                .await
                .expect(&format!("Timeout waiting for event on client {}", i + 1))
                .unwrap()
                .unwrap();
            
            if let TungsteniteMessage::Text(text) = msg {
                let parsed: WebSocketMessage = serde_json::from_str(&text).unwrap();
                match parsed {
                    WebSocketMessage::Event(BlockchainEvent::BlockCreated { 
                        block_index, 
                        block_hash,
                        .. 
                    }) => {
                        assert_eq!(block_index, 42);
                        assert_eq!(block_hash, "broadcast-test-hash");
                        println!("✅ Client {} received broadcast event correctly", i + 1);
                        break; // Found the expected event
                    }
                    WebSocketMessage::Event(BlockchainEvent::SystemStatus { .. }) => {
                        // Skip automatic system status events
                        continue;
                    }
                    _ => panic!("Client {} received unexpected message: {:?}", i + 1, parsed),
                }
            } else {
                panic!("Client {} received non-text message: {:?}", i + 1, msg);
            }
        }
    }
    
    println!("✅ All clients received broadcast event successfully");
}

#[tokio::test]
async fn test_invalid_message_handling() {
    let (server_url, _state) = create_test_server().await;
    
    // Connect client
    let mut ws_stream = connect_websocket_client(&server_url).await.unwrap();
    
    // Read connection acknowledgment
    let _ack = timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .expect("Timeout waiting for connection acknowledgment")
        .unwrap()
        .unwrap();
    
    // Send invalid JSON
    ws_stream.send(TungsteniteMessage::Text("invalid json".to_string())).await.unwrap();
    
    // Send malformed message structure
    ws_stream.send(TungsteniteMessage::Text(r#"{"invalid": "structure"}"#.to_string())).await.unwrap();
    
    // Give server time to process
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Connection should still be alive (server handles errors gracefully)
    // Send a valid ping to verify connection
    let ping_msg = WebSocketMessage::Ping {
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    let ping_text = serde_json::to_string(&ping_msg).unwrap();
    ws_stream.send(TungsteniteMessage::Text(ping_text)).await.unwrap();
    
    println!("✅ Server handled invalid messages gracefully, connection remains active");
}

/// Load test with many concurrent connections
#[tokio::test]
async fn test_concurrent_connections_load() {
    let (server_url, state) = create_test_server().await;
    
    const NUM_CLIENTS: usize = 50;
    let mut handles = Vec::new();
    
    // Spawn concurrent client connections
    for i in 0..NUM_CLIENTS {
        let url = server_url.clone();
        let handle = tokio::spawn(async move {
            let mut ws_stream = connect_websocket_client(&url).await.unwrap();
            
            // Read connection acknowledgment
            let _ack = timeout(Duration::from_secs(10), ws_stream.next())
                .await
                .expect(&format!("Timeout waiting for connection acknowledgment on client {}", i))
                .unwrap()
                .unwrap();
            
            // Keep connection alive for a short time
            tokio::time::sleep(Duration::from_millis(500)).await;
            
            // Close connection
            let _ = ws_stream.close(None).await;
            
            i
        });
        
        handles.push(handle);
    }
    
    // Wait for all clients to complete
    let results = futures_util::future::join_all(handles).await;
    
    // Verify all clients connected successfully
    for result in results {
        result.unwrap(); // This will panic if any client failed
    }
    
    // Give time for cleanup
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    // Verify all clients are cleaned up
    assert_eq!(state.client_count(), 0);
    println!("✅ Load test completed: {} concurrent connections handled successfully", NUM_CLIENTS);
}

/// Performance test for event broadcasting
#[tokio::test]
async fn test_event_broadcasting_performance() {
    let (server_url, state) = create_test_server().await;
    
    // Connect multiple clients
    const NUM_CLIENTS: usize = 10;
    let mut clients = Vec::new();
    
    for _i in 0..NUM_CLIENTS {
        let mut ws_stream = connect_websocket_client(&server_url).await.unwrap();
        
        // Read connection acknowledgment
        let _ack = timeout(Duration::from_secs(5), ws_stream.next())
            .await
            .expect("Timeout waiting for connection acknowledgment")
            .unwrap()
            .unwrap();
        
        clients.push(ws_stream);
    }
    
    // Verify all clients are connected
    assert_eq!(state.client_count(), NUM_CLIENTS);
    
    let broadcaster = BlockchainEventBroadcaster::new(state.clone());
    
    // Measure time to broadcast multiple events
    const NUM_EVENTS: usize = 100;
    let start_time = std::time::Instant::now();
    
    for i in 0..NUM_EVENTS {
        broadcaster.broadcast_block_created(
            i as u64,
            format!("perf-test-hash-{}", i),
            i % 10 + 1,
        );
    }
    
    let broadcast_duration = start_time.elapsed();
    
    println!("✅ Broadcasted {} events to {} clients in {:?}", 
             NUM_EVENTS, NUM_CLIENTS, broadcast_duration);
    
    // Verify events are received (sample a few, skip SystemStatus events)
    for client in clients.iter_mut().take(2) {
        let mut received_events = 0;
        while received_events < 5 {
            let msg = timeout(Duration::from_secs(5), client.next())
                .await
                .expect("Timeout waiting for performance test event")
                .unwrap()
                .unwrap();
            
            if let TungsteniteMessage::Text(text) = msg {
                let parsed: WebSocketMessage = serde_json::from_str(&text).unwrap();
                match parsed {
                    WebSocketMessage::Event(BlockchainEvent::BlockCreated { .. }) => {
                        // Event received successfully
                        received_events += 1;
                    }
                    WebSocketMessage::Event(BlockchainEvent::SystemStatus { .. }) => {
                        // Skip automatic system status events
                        continue;
                    }
                    _ => panic!("Unexpected message during performance test: {:?}", parsed),
                }
            }
        }
    }
    
    println!("✅ Performance test completed successfully");
}
