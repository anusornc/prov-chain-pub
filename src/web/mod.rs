//! Web interface module for Phase 2 implementation
//! Provides REST API and web server functionality

pub mod server;
pub mod handlers;
pub mod auth;
pub mod models;
pub mod websocket;

pub use server::WebServer;
pub use websocket::{WebSocketState, BlockchainEventBroadcaster, websocket_handler};
