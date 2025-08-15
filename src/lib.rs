//! # GhostDock
//!
//! A next-generation, self-hosted Docker registry with advanced management capabilities.
//! Built with Rust for performance, reliability, and safety.

pub mod api;
pub mod auth;
pub mod cli;
pub mod config;
pub mod database;
pub mod error;
pub mod handlers;
pub mod models;
pub mod server;
pub mod storage;
pub mod types;
pub mod utils;
pub mod web;

pub use config::Config;
pub use error::{Error, Result};

/// Current version of GhostDock
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default registry port
pub const DEFAULT_REGISTRY_PORT: u16 = 5000;

/// Default web UI port
pub const DEFAULT_WEB_PORT: u16 = 8080;
