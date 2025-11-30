//! Web interface module for Phase 2 implementation
//! Provides REST API and web server functionality

pub mod auth;
pub mod handlers;
pub mod models;
pub mod server;
pub mod websocket;

pub use server::WebServer;
pub use websocket::{websocket_handler, BlockchainEventBroadcaster, WebSocketState};
