//! # GhostDock
//!
//! A next-generation, self-hosted Docker registry with advanced management capabilities.
//! Built with Rust for performance, reliability, and safety.
//!
//! ## Features
//! - Docker Registry v2 API compliance
//! - JWT-based authentication with role-based access control
//! - Docker Compose stack management and deployment
//! - Real-time WebSocket updates for monitoring
//! - Modern web interface with interactive dashboard
//! - Blob storage with configurable backends
//! - Production-ready with monitoring and metrics

pub mod api;
pub mod auth;
pub mod cli;
pub mod config;
pub mod database;
pub mod enhanced_error;
pub mod error;
pub mod handlers;
pub mod models;
pub mod performance;
pub mod server;
pub mod stack_management;
pub mod storage;
pub mod types;
pub mod utils;
pub mod web;
pub mod web_enhanced;
pub mod websocket;

pub use config::Config;
pub use error::{Error, Result};

/// Current version of GhostDock
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default registry port
pub const DEFAULT_REGISTRY_PORT: u16 = 5000;

/// Default web UI port
pub const DEFAULT_WEB_PORT: u16 = 8080;

/// Maximum blob size (2GB)
pub const MAX_BLOB_SIZE: u64 = 2 * 1024 * 1024 * 1024;

/// WebSocket ping interval (seconds)
pub const WEBSOCKET_PING_INTERVAL: u64 = 30;

/// Default JWT expiration time (24 hours)
pub const DEFAULT_JWT_EXPIRATION: u64 = 24 * 60 * 60;
