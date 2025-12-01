//! Peer connection management for GraphChain P2P networking
//!
//! This module handles individual peer connections using WebSockets,
//! including connection lifecycle, message sending/receiving, and
//! connection health monitoring.

use anyhow::Result;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::{
    accept_async, connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::messages::{P2PMessage, PeerInfo};

/// Represents a connection to a peer node
pub struct PeerConnection {
    /// Information about the peer
    pub info: PeerInfo,
    /// Channel for sending messages to the peer
    pub sender: mpsc::UnboundedSender<P2PMessage>,
    /// Handle to the connection task
    pub task_handle: tokio::task::JoinHandle<()>,
}

/// WebSocket connection wrapper
pub enum WebSocketConnection {
    Client(WebSocketStream<MaybeTlsStream<TcpStream>>),
    Server(WebSocketStream<TcpStream>),
}

impl PeerConnection {
    /// Create a new peer connection from an outgoing WebSocket connection
    pub async fn new_outgoing(
        peer_info: PeerInfo,
        ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        message_handler: Arc<dyn Fn(Uuid, P2PMessage) + Send + Sync>,
    ) -> Result<Self> {
        let (sender, receiver) = mpsc::unbounded_channel();
        let connection = WebSocketConnection::Client(ws_stream);

        let task_handle = tokio::spawn(Self::connection_task(
            peer_info.node_id,
            connection,
            receiver,
            message_handler,
        ));

        Ok(Self {
            info: peer_info,
            sender,
            task_handle,
        })
    }

    /// Create a new peer connection from an incoming WebSocket connection
    pub async fn new_incoming(
        peer_info: PeerInfo,
        ws_stream: WebSocketStream<TcpStream>,
        message_handler: Arc<dyn Fn(Uuid, P2PMessage) + Send + Sync>,
    ) -> Result<Self> {
        let (sender, receiver) = mpsc::unbounded_channel();
        let connection = WebSocketConnection::Server(ws_stream);

        let task_handle = tokio::spawn(Self::connection_task(
            peer_info.node_id,
            connection,
            receiver,
            message_handler,
        ));

        Ok(Self {
            info: peer_info,
            sender,
            task_handle,
        })
    }

    /// Send a message to the peer
    pub async fn send_message(&self, message: P2PMessage) -> Result<()> {
        self.sender
            .send(message)
            .map_err(|_| anyhow::anyhow!("Failed to send message to peer {}", self.info.node_id))?;
        Ok(())
    }

    /// Main connection task that handles message sending and receiving
    async fn connection_task(
        peer_id: Uuid,
        connection: WebSocketConnection,
        message_receiver: mpsc::UnboundedReceiver<P2PMessage>,
        message_handler: Arc<dyn Fn(Uuid, P2PMessage) + Send + Sync>,
    ) {
        debug!("Starting connection task for peer {}", peer_id);

        match connection {
            WebSocketConnection::Client(ws) => {
                Self::handle_connection_loop(peer_id, ws, message_receiver, message_handler).await;
            }
            WebSocketConnection::Server(ws) => {
                Self::handle_connection_loop(peer_id, ws, message_receiver, message_handler).await;
            }
        }

        info!("Connection task ended for peer {}", peer_id);
    }

    /// Generic connection loop that works with any WebSocket stream type
    async fn handle_connection_loop<S>(
        peer_id: Uuid,
        ws: WebSocketStream<S>,
        mut message_receiver: mpsc::UnboundedReceiver<P2PMessage>,
        message_handler: Arc<dyn Fn(Uuid, P2PMessage) + Send + Sync>,
    ) where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        let (mut ws_sender, mut ws_receiver) = ws.split();

        loop {
            tokio::select! {
                // Handle outgoing messages
                message = message_receiver.recv() => {
                    match message {
                        Some(msg) => {
                            if let Err(e) = Self::send_websocket_message_generic(&mut ws_sender, msg).await {
                                error!("Failed to send message to peer {}: {}", peer_id, e);
                                break;
                            }
                        }
                        None => {
                            debug!("Message channel closed for peer {}", peer_id);
                            break;
                        }
                    }
                }

                // Handle incoming messages
                ws_message = ws_receiver.next() => {
                    match ws_message {
                        Some(Ok(Message::Text(text))) => {
                            match P2PMessage::from_bytes(text.as_bytes()) {
                                Ok(message) => {
                                    debug!("Received {} from peer {}", message.message_type(), peer_id);
                                    message_handler(peer_id, message);
                                }
                                Err(e) => {
                                    warn!("Failed to parse message from peer {}: {}", peer_id, e);
                                }
                            }
                        }
                        Some(Ok(Message::Binary(data))) => {
                            match P2PMessage::from_bytes(&data) {
                                Ok(message) => {
                                    debug!("Received {} from peer {}", message.message_type(), peer_id);
                                    message_handler(peer_id, message);
                                }
                                Err(e) => {
                                    warn!("Failed to parse binary message from peer {}: {}", peer_id, e);
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("Peer {} closed connection", peer_id);
                            break;
                        }
                        Some(Ok(Message::Ping(data))) => {
                            if let Err(e) = ws_sender.send(Message::Pong(data)).await {
                                error!("Failed to send pong to peer {}: {}", peer_id, e);
                                break;
                            }
                        }
                        Some(Ok(Message::Pong(_))) => {
                            // Handle pong if needed for connection health
                        }
                        Some(Ok(Message::Frame(_))) => {
                            // Handle raw frames if needed
                            debug!("Received raw frame from peer {}", peer_id);
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error with peer {}: {}", peer_id, e);
                            break;
                        }
                        None => {
                            info!("WebSocket stream ended for peer {}", peer_id);
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Send a P2P message over WebSocket (generic version)
    async fn send_websocket_message_generic<S>(
        ws_sender: &mut SplitSink<WebSocketStream<S>, Message>,
        message: P2PMessage,
    ) -> Result<()>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        let bytes = message.to_bytes()?;
        let ws_message = Message::Text(String::from_utf8(bytes)?);
        ws_sender.send(ws_message).await?;
        Ok(())
    }
}

/// WebSocket server for accepting incoming peer connections
pub struct PeerServer {
    listener: TcpListener,
    message_handler: Arc<dyn Fn(Uuid, P2PMessage) + Send + Sync>,
    connection_handler: Arc<dyn Fn(PeerConnection) + Send + Sync>,
}

impl PeerServer {
    /// Create a new peer server
    pub async fn new(
        listen_addr: &str,
        message_handler: Arc<dyn Fn(Uuid, P2PMessage) + Send + Sync>,
        connection_handler: Arc<dyn Fn(PeerConnection) + Send + Sync>,
    ) -> Result<Self> {
        let listener = TcpListener::bind(listen_addr).await?;
        info!("WebSocket server listening on {}", listen_addr);

        Ok(Self {
            listener,
            message_handler,
            connection_handler,
        })
    }

    /// Start accepting incoming connections
    pub async fn start(&self) -> Result<()> {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New connection from {}", addr);
                    let message_handler = Arc::clone(&self.message_handler);
                    let connection_handler = Arc::clone(&self.connection_handler);

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_incoming_connection(
                            stream,
                            message_handler,
                            connection_handler,
                        )
                        .await
                        {
                            error!("Failed to handle incoming connection from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Handle a new incoming WebSocket connection
    async fn handle_incoming_connection(
        stream: TcpStream,
        message_handler: Arc<dyn Fn(Uuid, P2PMessage) + Send + Sync>,
        connection_handler: Arc<dyn Fn(PeerConnection) + Send + Sync>,
    ) -> Result<()> {
        let ws_stream = accept_async(stream).await?;
        info!("WebSocket connection established");

        // For now, create a temporary peer info
        // In a real implementation, this would be established through handshake
        let peer_info = PeerInfo::new(
            Uuid::new_v4(), // This should be received from peer discovery
            "unknown".to_string(),
            0,
            "unknown".to_string(),
            false,
        );

        let connection =
            PeerConnection::new_incoming(peer_info, ws_stream, message_handler).await?;

        // Pass the connection to the handler
        connection_handler(connection);

        Ok(())
    }
}

/// Client for connecting to remote peers
pub struct PeerClient;

impl PeerClient {
    /// Connect to a remote peer
    pub async fn connect(
        peer_address: &str,
        message_handler: Arc<dyn Fn(Uuid, P2PMessage) + Send + Sync>,
    ) -> Result<PeerConnection> {
        let url = format!("ws://{peer_address}");
        info!("Connecting to peer at {}", url);

        let (ws_stream, _) = connect_async(&url).await?;
        info!("Connected to peer at {}", peer_address);

        // For now, create a temporary peer info
        // In a real implementation, this would be established through handshake
        let peer_info = PeerInfo::new(
            Uuid::new_v4(), // This should be received from peer discovery
            peer_address
                .split(':')
                .next()
                .unwrap_or("unknown")
                .to_string(),
            peer_address
                .split(':')
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            "unknown".to_string(),
            false,
        );

        PeerConnection::new_outgoing(peer_info, ws_stream, message_handler).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_peer_connection_creation() {
        let message_count = Arc::new(AtomicUsize::new(0));
        let message_count_clone = Arc::clone(&message_count);

        let _handler = Arc::new(move |_peer_id: Uuid, _message: P2PMessage| {
            message_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        // This test would require setting up actual WebSocket connections
        // For now, we just test that the handler compiles correctly
        assert_eq!(message_count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_peer_info() {
        let peer_info = PeerInfo::new(
            Uuid::new_v4(),
            "127.0.0.1".to_string(),
            8080,
            "test-network".to_string(),
            false,
        );

        assert_eq!(peer_info.full_address(), "127.0.0.1:8080");
        assert!(!peer_info.is_authority);
    }
}
