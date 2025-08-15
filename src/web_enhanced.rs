use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Html,
    Router,
    routing::get,
    extract::State,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct WebState {
    tx: broadcast::Sender<String>,
}

pub fn enhanced_routes() -> Router {
    let (tx, _rx) = broadcast::channel(100);
    let state = WebState { tx };

    Router::new()
        .route("/", get(enhanced_index))
        .route("/dashboard", get(enhanced_dashboard))
        .route("/api/stats", get(get_stats))
        .route("/ws", get(websocket_handler))
        .nest_service("/static", ServeDir::new("assets"))
        .with_state(state)
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebState>,
) -> axum::response::Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: WebState) {
    let mut rx = state.tx.subscribe();
    
    while let Ok(msg) = rx.recv().await {
        if socket.send(axum::extract::ws::Message::Text(msg)).await.is_err() {
            break;
        }
    }
}

async fn enhanced_index() -> Html<String> {
    Html(include_str!("../assets/enhanced_index.html").to_string())
}

async fn enhanced_dashboard() -> Html<String> {
    Html(include_str!("../assets/enhanced_dashboard.html").to_string())
}

async fn get_stats() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "repositories": 12,
        "images": 89,
        "storage_gb": 2.4,
        "pulls_today": 147,
        "pushes_today": 23
    }))
}
