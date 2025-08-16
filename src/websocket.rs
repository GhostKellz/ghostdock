use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::{
    auth::{jwt::validate_token, middleware::AuthenticatedUser},
    error::Result,
};

/// WebSocket connection manager for real-time updates
#[derive(Clone)]
pub struct WebSocketState {
    /// Broadcast channel for sending updates to all connected clients
    pub broadcaster: broadcast::Sender<BroadcastMessage>,
    /// Active WebSocket connections
    pub connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
}

/// Information about an active WebSocket connection
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub user_id: String,
    pub user_email: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub subscriptions: Vec<String>,
}

/// Message types that can be broadcast to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BroadcastMessage {
    /// Registry activity updates
    RegistryActivity {
        activity: RegistryActivity,
    },
    /// Stack deployment updates
    StackDeployment {
        stack_id: String,
        status: DeploymentStatus,
        message: String,
    },
    /// System metrics updates
    SystemMetrics {
        metrics: SystemMetrics,
    },
    /// User notifications
    Notification {
        user_id: String,
        notification: Notification,
    },
    /// Live logs from deployments
    DeploymentLogs {
        stack_id: String,
        deployment_id: String,
        logs: String,
    },
}

/// Registry activity events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryActivity {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_id: String,
    pub user_email: String,
    pub action: ActivityAction,
    pub repository: String,
    pub tag: Option<String>,
    pub size: Option<u64>,
}

/// Types of registry activities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityAction {
    Push,
    Pull,
    Delete,
    ListTags,
    GetManifest,
}

/// Stack deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentStatus {
    Pending,
    Deploying,
    Running,
    Failed,
    Stopping,
    Stopped,
}

/// System metrics for real-time monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_rx: u64,
    pub network_tx: u64,
    pub active_connections: usize,
    pub registry_operations_per_minute: u64,
    pub storage_size: u64,
}

/// User notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub severity: NotificationSeverity,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub read: bool,
}

/// Notification severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationSeverity {
    Info,
    Warning,
    Error,
    Success,
}

/// WebSocket message from client
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Subscribe to specific event types
    Subscribe {
        topics: Vec<String>,
    },
    /// Unsubscribe from event types
    Unsubscribe {
        topics: Vec<String>,
    },
    /// Ping to keep connection alive
    Ping,
    /// Authentication token
    Auth {
        token: String,
    },
}

/// WebSocket message to client
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Welcome message after successful connection
    Welcome {
        connection_id: String,
        user_id: String,
        available_topics: Vec<String>,
    },
    /// Pong response to ping
    Pong,
    /// Error message
    Error {
        message: String,
    },
    /// Subscription confirmation
    Subscribed {
        topics: Vec<String>,
    },
    /// Unsubscription confirmation
    Unsubscribed {
        topics: Vec<String>,
    },
    /// Broadcast message
    Broadcast {
        message: BroadcastMessage,
    },
}

impl WebSocketState {
    /// Create a new WebSocket state
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(1000);
        Self {
            broadcaster: tx,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Broadcast a message to all connected clients
    pub async fn broadcast(&self, message: BroadcastMessage) {
        if let Err(e) = self.broadcaster.send(message) {
            eprintln!("Failed to broadcast message: {}", e);
        }
    }

    /// Get the number of active connections
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Remove a connection
    pub async fn remove_connection(&self, connection_id: &str) {
        self.connections.write().await.remove(connection_id);
    }
}

/// WebSocket routes
pub fn websocket_routes() -> Router<WebSocketState> {
    Router::new()
        .route("/ws", get(websocket_handler))
        .route("/ws/metrics", get(metrics_websocket_handler))
}

/// Main WebSocket handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

/// Metrics-specific WebSocket handler
async fn metrics_websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_metrics_websocket(socket, state))
}

/// Handle a WebSocket connection
async fn handle_websocket(socket: WebSocket, state: WebSocketState) {
    let connection_id = Uuid::new_v4().to_string();
    let mut authenticated_user: Option<AuthenticatedUser> = None;
    let mut subscriptions: Vec<String> = Vec::new();
    
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.broadcaster.subscribe();
    
    // Send welcome message
    let welcome_msg = ServerMessage::Welcome {
        connection_id: connection_id.clone(),
        user_id: "anonymous".to_string(),
        available_topics: vec![
            "registry_activity".to_string(),
            "stack_deployments".to_string(),
            "system_metrics".to_string(),
            "notifications".to_string(),
            "deployment_logs".to_string(),
        ],
    };
    
    if let Ok(msg_text) = serde_json::to_string(&welcome_msg) {
        if sender.send(Message::Text(msg_text)).await.is_err() {
            return;
        }
    }
    
    // Handle incoming messages and broadcast events concurrently
    loop {
        tokio::select! {
            // Handle incoming WebSocket messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                            match handle_client_message(
                                client_msg,
                                &mut authenticated_user,
                                &mut subscriptions,
                                &mut sender,
                                &connection_id,
                                &state,
                            ).await {
                                Ok(should_continue) => {
                                    if !should_continue {
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        break;
                    }
                    Some(Err(_)) => {
                        break;
                    }
                    _ => {}
                }
            }
            
            // Handle broadcast messages
            broadcast_msg = rx.recv() => {
                match broadcast_msg {
                    Ok(msg) => {
                        // Check if user should receive this message based on subscriptions
                        if should_receive_message(&msg, &subscriptions, &authenticated_user) {
                            let server_msg = ServerMessage::Broadcast { message: msg };
                            if let Ok(msg_text) = serde_json::to_string(&server_msg) {
                                if sender.send(Message::Text(msg_text)).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Broadcast channel closed
                        break;
                    }
                }
            }
        }
    }
    
    // Clean up connection
    state.remove_connection(&connection_id).await;
}

/// Handle metrics-specific WebSocket connection
async fn handle_metrics_websocket(socket: WebSocket, state: WebSocketState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.broadcaster.subscribe();
    
    // Send initial metrics
    let initial_metrics = SystemMetrics {
        timestamp: chrono::Utc::now(),
        cpu_usage: 25.5,
        memory_usage: 60.2,
        disk_usage: 45.0,
        network_rx: 1024 * 1024,
        network_tx: 512 * 1024,
        active_connections: state.connection_count().await,
        registry_operations_per_minute: 150,
        storage_size: 1024 * 1024 * 1024 * 5, // 5GB
    };
    
    let welcome_msg = ServerMessage::Broadcast {
        message: BroadcastMessage::SystemMetrics {
            metrics: initial_metrics,
        },
    };
    
    if let Ok(msg_text) = serde_json::to_string(&welcome_msg) {
        if sender.send(Message::Text(msg_text)).await.is_err() {
            return;
        }
    }
    
    // Handle metrics updates
    loop {
        tokio::select! {
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
            
            broadcast_msg = rx.recv() => {
                match broadcast_msg {
                    Ok(BroadcastMessage::SystemMetrics { metrics }) => {
                        let server_msg = ServerMessage::Broadcast {
                            message: BroadcastMessage::SystemMetrics { metrics },
                        };
                        if let Ok(msg_text) = serde_json::to_string(&server_msg) {
                            if sender.send(Message::Text(msg_text)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Ok(_) => {
                        // Ignore non-metrics messages
                    }
                    Err(_) => break,
                }
            }
        }
    }
}

/// Handle a client message
async fn handle_client_message(
    message: ClientMessage,
    authenticated_user: &mut Option<AuthenticatedUser>,
    subscriptions: &mut Vec<String>,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    connection_id: &str,
    state: &WebSocketState,
) -> Result<bool> {
    match message {
        ClientMessage::Auth { token } => {
            // We need a JwtConfig instance - for now create a default one
            let jwt_config = crate::auth::jwt::JwtConfig::new(
                std::env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string())
            );
            
            match validate_token(&token, &jwt_config) {
                Ok(claims) => {
                    *authenticated_user = Some(AuthenticatedUser {
                        id: claims.sub,
                        name: claims.name,
                        email: claims.email,
                        scopes: claims.scope,
                    });
                    
                    let user = authenticated_user.as_ref().unwrap();
                    
                    // Store connection info
                    let connection_info = ConnectionInfo {
                        user_id: user.id.clone(),
                        user_email: user.email.clone(),
                        connected_at: chrono::Utc::now(),
                        subscriptions: subscriptions.clone(),
                    };
                    
                    state.connections.write().await.insert(
                        connection_id.to_string(),
                        connection_info,
                    );
                    
                    let welcome_msg = ServerMessage::Welcome {
                        connection_id: connection_id.to_string(),
                        user_id: user.id.clone(),
                        available_topics: vec![
                            "registry_activity".to_string(),
                            "stack_deployments".to_string(),
                            "system_metrics".to_string(),
                            "notifications".to_string(),
                            "deployment_logs".to_string(),
                        ],
                    };
                    
                    if let Ok(msg_text) = serde_json::to_string(&welcome_msg) {
                        sender.send(Message::Text(msg_text)).await?;
                    }
                }
                Err(_) => {
                    let error_msg = ServerMessage::Error {
                        message: "Invalid authentication token".to_string(),
                    };
                    
                    if let Ok(msg_text) = serde_json::to_string(&error_msg) {
                        sender.send(Message::Text(msg_text)).await?;
                    }
                }
            }
        }
        
        ClientMessage::Subscribe { topics } => {
            for topic in &topics {
                if !subscriptions.contains(topic) {
                    subscriptions.push(topic.clone());
                }
            }
            
            let response = ServerMessage::Subscribed {
                topics: topics.clone(),
            };
            
            if let Ok(msg_text) = serde_json::to_string(&response) {
                sender.send(Message::Text(msg_text)).await?;
            }
        }
        
        ClientMessage::Unsubscribe { topics } => {
            for topic in &topics {
                subscriptions.retain(|s| s != topic);
            }
            
            let response = ServerMessage::Unsubscribed {
                topics: topics.clone(),
            };
            
            if let Ok(msg_text) = serde_json::to_string(&response) {
                sender.send(Message::Text(msg_text)).await?;
            }
        }
        
        ClientMessage::Ping => {
            let pong_msg = ServerMessage::Pong;
            if let Ok(msg_text) = serde_json::to_string(&pong_msg) {
                sender.send(Message::Text(msg_text)).await?;
            }
        }
    }
    
    Ok(true)
}

/// Check if a user should receive a specific broadcast message
fn should_receive_message(
    message: &BroadcastMessage,
    subscriptions: &[String],
    authenticated_user: &Option<AuthenticatedUser>,
) -> bool {
    match message {
        BroadcastMessage::RegistryActivity { .. } => {
            subscriptions.contains(&"registry_activity".to_string())
        }
        BroadcastMessage::StackDeployment { .. } => {
            subscriptions.contains(&"stack_deployments".to_string())
        }
        BroadcastMessage::SystemMetrics { .. } => {
            subscriptions.contains(&"system_metrics".to_string())
        }
        BroadcastMessage::Notification { user_id, .. } => {
            if let Some(user) = authenticated_user {
                subscriptions.contains(&"notifications".to_string()) && user.id == *user_id
            } else {
                false
            }
        }
        BroadcastMessage::DeploymentLogs { .. } => {
            subscriptions.contains(&"deployment_logs".to_string())
        }
    }
}

/// Helper functions for broadcasting different types of events

impl WebSocketState {
    /// Broadcast registry activity
    pub async fn broadcast_registry_activity(
        &self,
        user_id: String,
        user_email: String,
        action: ActivityAction,
        repository: String,
        tag: Option<String>,
        size: Option<u64>,
    ) {
        let activity = RegistryActivity {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            user_id,
            user_email,
            action,
            repository,
            tag,
            size,
        };
        
        self.broadcast(BroadcastMessage::RegistryActivity { activity }).await;
    }
    
    /// Broadcast stack deployment update
    pub async fn broadcast_stack_deployment(
        &self,
        stack_id: String,
        status: DeploymentStatus,
        message: String,
    ) {
        self.broadcast(BroadcastMessage::StackDeployment {
            stack_id,
            status,
            message,
        }).await;
    }
    
    /// Broadcast system metrics
    pub async fn broadcast_system_metrics(&self, metrics: SystemMetrics) {
        self.broadcast(BroadcastMessage::SystemMetrics { metrics }).await;
    }
    
    /// Broadcast user notification
    pub async fn broadcast_notification(&self, user_id: String, notification: Notification) {
        self.broadcast(BroadcastMessage::Notification {
            user_id,
            notification,
        }).await;
    }
    
    /// Broadcast deployment logs
    pub async fn broadcast_deployment_logs(
        &self,
        stack_id: String,
        deployment_id: String,
        logs: String,
    ) {
        self.broadcast(BroadcastMessage::DeploymentLogs {
            stack_id,
            deployment_id,
            logs,
        }).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_receive_message() {
        let user = Some(AuthenticatedUser {
            id: "user123".to_string(),
            email: "user@example.com".to_string(),
            scopes: vec![],
        });
        
        let subscriptions = vec!["registry_activity".to_string(), "notifications".to_string()];
        
        // Should receive registry activity
        let registry_msg = BroadcastMessage::RegistryActivity {
            activity: RegistryActivity {
                id: "1".to_string(),
                timestamp: chrono::Utc::now(),
                user_id: "user123".to_string(),
                user_email: "user@example.com".to_string(),
                action: ActivityAction::Push,
                repository: "test/repo".to_string(),
                tag: Some("latest".to_string()),
                size: Some(1024),
            },
        };
        assert!(should_receive_message(&registry_msg, &subscriptions, &user));
        
        // Should receive notification for same user
        let notification_msg = BroadcastMessage::Notification {
            user_id: "user123".to_string(),
            notification: Notification {
                id: "1".to_string(),
                title: "Test".to_string(),
                message: "Test message".to_string(),
                severity: NotificationSeverity::Info,
                timestamp: chrono::Utc::now(),
                read: false,
            },
        };
        assert!(should_receive_message(&notification_msg, &subscriptions, &user));
        
        // Should not receive notification for different user
        let other_notification_msg = BroadcastMessage::Notification {
            user_id: "otheruser".to_string(),
            notification: Notification {
                id: "2".to_string(),
                title: "Test".to_string(),
                message: "Test message".to_string(),
                severity: NotificationSeverity::Info,
                timestamp: chrono::Utc::now(),
                read: false,
            },
        };
        assert!(!should_receive_message(&other_notification_msg, &subscriptions, &user));
        
        // Should not receive system metrics without subscription
        let metrics_msg = BroadcastMessage::SystemMetrics {
            metrics: SystemMetrics {
                timestamp: chrono::Utc::now(),
                cpu_usage: 50.0,
                memory_usage: 60.0,
                disk_usage: 70.0,
                network_rx: 1024,
                network_tx: 1024,
                active_connections: 5,
                registry_operations_per_minute: 100,
                storage_size: 1024 * 1024 * 1024,
            },
        };
        assert!(!should_receive_message(&metrics_msg, &subscriptions, &user));
    }
}
