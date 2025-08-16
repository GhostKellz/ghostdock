use anyhow::Result;
use clap::Parser;
use ghostdock::{
    cli::Cli, 
    server::Server,
    websocket::WebSocketState,
};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with better formatting
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Parse CLI arguments
    let cli = Cli::parse();
    
    info!("🚀 Starting GhostDock Registry v{}", env!("CARGO_PKG_VERSION"));
    info!("📁 Config file: {:?}", cli.config);

    // Create shared state for WebSocket connections
    let websocket_state = Arc::new(WebSocketState::new());
    
    // Start background tasks
    let ws_state_metrics = Arc::clone(&websocket_state);
    tokio::spawn(async move {
        start_metrics_broadcaster(ws_state_metrics).await;
    });

    // Create and start server with enhanced features
    let server = Server::new(cli.config).await?;
    
    info!("🌐 Registry server starting...");
    info!("📊 Real-time WebSocket updates enabled");
    info!("🐳 Docker Compose stack management ready");
    info!("🎯 Enhanced web interface available");
    
    // Handle graceful shutdown
    let shutdown_signal = tokio::signal::ctrl_c();
    
    tokio::select! {
        result = server.run() => {
            match result {
                Ok(_) => info!("Server shutdown gracefully"),
                Err(e) => warn!("Server error: {}", e),
            }
        }
        _ = shutdown_signal => {
            info!("Received shutdown signal, stopping server...");
        }
    }

    info!("👻 GhostDock Registry stopped");
    Ok(())
}

/// Background task to broadcast system metrics
async fn start_metrics_broadcaster(websocket_state: Arc<WebSocketState>) {
    let mut interval = interval(Duration::from_secs(5));
    
    loop {
        interval.tick().await;
        
        // Collect system metrics
        let metrics = collect_system_metrics(&websocket_state).await;
        
        // Broadcast to all connected WebSocket clients
        websocket_state.broadcast_system_metrics(metrics).await;
    }
}

/// Collect current system metrics
async fn collect_system_metrics(websocket_state: &WebSocketState) -> ghostdock::websocket::SystemMetrics {
    use ghostdock::websocket::SystemMetrics;
    
    // In a real implementation, you would collect actual system metrics
    // For now, we'll simulate some realistic values
    let cpu_usage = simulate_cpu_usage();
    let memory_usage = simulate_memory_usage();
    let disk_usage = simulate_disk_usage();
    
    SystemMetrics {
        timestamp: chrono::Utc::now(),
        cpu_usage,
        memory_usage,
        disk_usage,
        network_rx: simulate_network_rx(),
        network_tx: simulate_network_tx(),
        active_connections: websocket_state.connection_count().await,
        registry_operations_per_minute: simulate_registry_ops(),
        storage_size: simulate_storage_size(),
    }
}

/// Simulate CPU usage (in production, use system metrics)
fn simulate_cpu_usage() -> f64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    // Simulate varying CPU usage between 10-80%
    rng.gen_range(10.0..80.0)
}

/// Simulate memory usage (in production, use system metrics)
fn simulate_memory_usage() -> f64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    // Simulate varying memory usage between 30-90%
    rng.gen_range(30.0..90.0)
}

/// Simulate disk usage (in production, use system metrics)
fn simulate_disk_usage() -> f64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    // Simulate disk usage growing slowly over time
    rng.gen_range(45.0..65.0)
}

/// Simulate network RX bytes
fn simulate_network_rx() -> u64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(1024 * 1024..10 * 1024 * 1024) // 1MB to 10MB
}

/// Simulate network TX bytes
fn simulate_network_tx() -> u64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(512 * 1024..5 * 1024 * 1024) // 512KB to 5MB
}

/// Simulate registry operations per minute
fn simulate_registry_ops() -> u64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(50..300)
}

/// Simulate storage size
fn simulate_storage_size() -> u64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(1024 * 1024 * 1024..50 * 1024 * 1024 * 1024) // 1GB to 50GB
}
