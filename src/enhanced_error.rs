use std::fmt;
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::json;
use tracing::{error, warn, info, debug};

/// Enhanced error types for better debugging and monitoring
#[derive(Debug)]
pub enum GhostDockError {
    Database(sqlx::Error),
    Storage(std::io::Error),
    Authentication(String),
    Authorization(String),
    Validation(String),
    NotFound(String),
    Conflict(String),
    RateLimit(String),
    Internal(String),
    External(reqwest::Error),
    Serialization(serde_json::Error),
}

impl fmt::Display for GhostDockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GhostDockError::Database(e) => write!(f, "Database error: {}", e),
            GhostDockError::Storage(e) => write!(f, "Storage error: {}", e),
            GhostDockError::Authentication(e) => write!(f, "Authentication error: {}", e),
            GhostDockError::Authorization(e) => write!(f, "Authorization error: {}", e),
            GhostDockError::Validation(e) => write!(f, "Validation error: {}", e),
            GhostDockError::NotFound(e) => write!(f, "Not found: {}", e),
            GhostDockError::Conflict(e) => write!(f, "Conflict: {}", e),
            GhostDockError::RateLimit(e) => write!(f, "Rate limit exceeded: {}", e),
            GhostDockError::Internal(e) => write!(f, "Internal error: {}", e),
            GhostDockError::External(e) => write!(f, "External service error: {}", e),
            GhostDockError::Serialization(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for GhostDockError {}

impl IntoResponse for GhostDockError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            GhostDockError::Database(_) => {
                error!("Database error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", "Internal database error")
            },
            GhostDockError::Storage(_) => {
                error!("Storage error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "STORAGE_ERROR", "Storage system error")
            },
            GhostDockError::Authentication(_) => {
                warn!("Authentication error: {}", self);
                (StatusCode::UNAUTHORIZED, "AUTH_ERROR", "Authentication required")
            },
            GhostDockError::Authorization(_) => {
                warn!("Authorization error: {}", self);
                (StatusCode::FORBIDDEN, "AUTHZ_ERROR", "Insufficient permissions")
            },
            GhostDockError::Validation(msg) => {
                info!("Validation error: {}", msg);
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.as_str())
            },
            GhostDockError::NotFound(resource) => {
                debug!("Resource not found: {}", resource);
                (StatusCode::NOT_FOUND, "NOT_FOUND", "Resource not found")
            },
            GhostDockError::Conflict(msg) => {
                info!("Conflict error: {}", msg);
                (StatusCode::CONFLICT, "CONFLICT", msg.as_str())
            },
            GhostDockError::RateLimit(msg) => {
                warn!("Rate limit exceeded: {}", msg);
                (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMIT", "Rate limit exceeded")
            },
            GhostDockError::Internal(_) => {
                error!("Internal error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal server error")
            },
            GhostDockError::External(_) => {
                error!("External service error: {}", self);
                (StatusCode::BAD_GATEWAY, "EXTERNAL_ERROR", "External service error")
            },
            GhostDockError::Serialization(_) => {
                error!("Serialization error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "SERIALIZATION_ERROR", "Data processing error")
            },
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "request_id": uuid::Uuid::new_v4().to_string()
            }
        }));

        (status, body).into_response()
    }
}

// Conversion implementations
impl From<sqlx::Error> for GhostDockError {
    fn from(err: sqlx::Error) -> Self {
        GhostDockError::Database(err)
    }
}

impl From<std::io::Error> for GhostDockError {
    fn from(err: std::io::Error) -> Self {
        GhostDockError::Storage(err)
    }
}

impl From<reqwest::Error> for GhostDockError {
    fn from(err: reqwest::Error) -> Self {
        GhostDockError::External(err)
    }
}

impl From<serde_json::Error> for GhostDockError {
    fn from(err: serde_json::Error) -> Self {
        GhostDockError::Serialization(err)
    }
}

pub type Result<T> = std::result::Result<T, GhostDockError>;

/// Enhanced logging with structured data
pub mod enhanced_logging {
    use tracing::{info, error, warn, debug};
    use std::time::Instant;
    
    pub struct RequestLogger {
        pub start_time: Instant,
        pub request_id: String,
        pub method: String,
        pub path: String,
        pub user_agent: Option<String>,
        pub client_ip: String,
    }

    impl RequestLogger {
        pub fn new(method: String, path: String, client_ip: String) -> Self {
            Self {
                start_time: Instant::now(),
                request_id: uuid::Uuid::new_v4().to_string(),
                method,
                path,
                user_agent: None,
                client_ip,
            }
        }

        pub fn log_request_start(&self) {
            info!(
                request_id = %self.request_id,
                method = %self.method,
                path = %self.path,
                client_ip = %self.client_ip,
                user_agent = %self.user_agent.as_deref().unwrap_or("unknown"),
                "Request started"
            );
        }

        pub fn log_request_end(&self, status_code: u16, response_size: Option<u64>) {
            let duration = self.start_time.elapsed();
            
            // Log based on status code
            match status_code {
                200..=299 => tracing::info!(
                    method = ?self.method,
                    path = %self.path,
                    status = status_code,
                    duration_ms = duration.as_millis(),
                    response_size = ?response_size,
                    "Request completed"
                ),
                400..=499 => tracing::warn!(
                    method = ?self.method,
                    path = %self.path,
                    status = status_code,
                    duration_ms = duration.as_millis(),
                    response_size = ?response_size,
                    "Request completed with warning"
                ),
                500..=599 => tracing::error!(
                    method = ?self.method,
                    path = %self.path,
                    status = status_code,
                    duration_ms = duration.as_millis(),
                    response_size = ?response_size,
                    "Request completed with error"
                ),
                _ => tracing::debug!(
                    method = ?self.method,
                    path = %self.path,
                    status = status_code,
                    duration_ms = duration.as_millis(),
                    response_size = ?response_size,
                    "Request completed"
                ),
            }
        }

        pub fn log_error(&self, error: &dyn std::error::Error) {
            error!(
                request_id = %self.request_id,
                method = %self.method,
                path = %self.path,
                client_ip = %self.client_ip,
                error = %error,
                "Request failed"
            );
        }
    }

    /// Structured logging for registry operations
    pub fn log_registry_operation(operation: &str, repository: &str, reference: &str, size: Option<u64>) {
        info!(
            operation = operation,
            repository = repository,
            reference = reference,
            size = size,
            timestamp = %chrono::Utc::now().to_rfc3339(),
            "Registry operation"
        );
    }

    /// Log authentication events
    pub fn log_auth_event(event: &str, user_id: Option<&str>, provider: Option<&str>, success: bool) {
        if success {
            info!(
                event = event,
                user_id = user_id,
                provider = provider,
                timestamp = %chrono::Utc::now().to_rfc3339(),
                "Authentication event"
            );
        } else {
            warn!(
                event = event,
                user_id = user_id,
                provider = provider,
                timestamp = %chrono::Utc::now().to_rfc3339(),
                "Authentication failed"
            );
        }
    }

    /// Performance monitoring
    pub async fn monitor_performance<F, T>(operation: &str, future: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let start = Instant::now();
        let span = tracing::info_span!("performance_monitor", operation = operation);
        let _guard = span.enter();
        
        let result = future.await;
        let duration = start.elapsed();
        
        if duration.as_millis() > 1000 {
            warn!(
                operation = operation,
                duration_ms = duration.as_millis(),
                "Slow operation detected"
            );
        } else {
            debug!(
                operation = operation,
                duration_ms = duration.as_millis(),
                "Operation completed"
            );
        }
        
        result
    }
}

/// Metrics collection
pub mod metrics {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use dashmap::DashMap;

    #[derive(Default)]
    pub struct Metrics {
        pub requests_total: AtomicU64,
        pub requests_by_status: DashMap<u16, AtomicU64>,
        pub registry_pulls: AtomicU64,
        pub registry_pushes: AtomicU64,
        pub bytes_transferred: AtomicU64,
        pub active_connections: AtomicU64,
    }

    impl Metrics {
        pub fn new() -> Arc<Self> {
            Arc::new(Self::default())
        }

        pub fn record_request(&self, status_code: u16) {
            self.requests_total.fetch_add(1, Ordering::Relaxed);
            self.requests_by_status
                .entry(status_code)
                .or_insert_with(|| AtomicU64::new(0))
                .fetch_add(1, Ordering::Relaxed);
        }

        pub fn record_pull(&self, bytes: u64) {
            self.registry_pulls.fetch_add(1, Ordering::Relaxed);
            self.bytes_transferred.fetch_add(bytes, Ordering::Relaxed);
        }

        pub fn record_push(&self, bytes: u64) {
            self.registry_pushes.fetch_add(1, Ordering::Relaxed);
            self.bytes_transferred.fetch_add(bytes, Ordering::Relaxed);
        }

        pub fn connection_opened(&self) {
            self.active_connections.fetch_add(1, Ordering::Relaxed);
        }

        pub fn connection_closed(&self) {
            self.active_connections.fetch_sub(1, Ordering::Relaxed);
        }

        pub fn export_prometheus(&self) -> String {
            let mut output = String::new();
            
            output.push_str(&format!(
                "ghostdock_requests_total {}\n",
                self.requests_total.load(Ordering::Relaxed)
            ));
            
            output.push_str(&format!(
                "ghostdock_registry_pulls_total {}\n",
                self.registry_pulls.load(Ordering::Relaxed)
            ));
            
            output.push_str(&format!(
                "ghostdock_registry_pushes_total {}\n",
                self.registry_pushes.load(Ordering::Relaxed)
            ));
            
            output.push_str(&format!(
                "ghostdock_bytes_transferred_total {}\n",
                self.bytes_transferred.load(Ordering::Relaxed)
            ));
            
            output.push_str(&format!(
                "ghostdock_active_connections {}\n",
                self.active_connections.load(Ordering::Relaxed)
            ));

            for entry in &self.requests_by_status {
                output.push_str(&format!(
                    "ghostdock_requests_by_status{{status=\"{}\"}} {}\n",
                    entry.key(),
                    entry.value().load(Ordering::Relaxed)
                ));
            }
            
            output
        }
    }
}
