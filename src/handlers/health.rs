use crate::{error::Result, server::AppState, types::HealthResponse};
use axum::{extract::State, response::IntoResponse, Json};
use std::time::{SystemTime, UNIX_EPOCH};

/// Health check endpoint
pub async fn health_check(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Check database connectivity
    let db_status = match sqlx::query("SELECT 1").fetch_optional(&state.database.pool).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    // Check storage backend
    let storage_status = match tokio::fs::metadata(&state.config.storage.path).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    let health = HealthResponse {
        status: if db_status == "healthy" && storage_status == "healthy" {
            "healthy".to_string()
        } else {
            "unhealthy".to_string()
        },
        version: crate::VERSION.to_string(),
        uptime,
        database: db_status.to_string(),
        storage: storage_status.to_string(),
    };

    Ok(Json(health))
}

/// Metrics endpoint (Prometheus-compatible)
pub async fn metrics(State(state): State<AppState>) -> Result<impl IntoResponse> {
    // Get basic metrics from database
    let repo_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM repositories")
        .fetch_one(&state.database.pool)
        .await?;

    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.database.pool)
        .await?;

    let total_pulls: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(pull_count), 0) FROM repositories")
        .fetch_one(&state.database.pool)
        .await?;

    let total_pushes: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(push_count), 0) FROM repositories")
        .fetch_one(&state.database.pool)
        .await?;

    // Calculate storage usage
    let storage_usage = calculate_storage_usage(&state.config.storage.path).await.unwrap_or(0);

    let metrics = format!(
        r#"# HELP ghostdock_repositories_total Total number of repositories
# TYPE ghostdock_repositories_total counter
ghostdock_repositories_total {}

# HELP ghostdock_users_total Total number of users
# TYPE ghostdock_users_total counter
ghostdock_users_total {}

# HELP ghostdock_pulls_total Total number of image pulls
# TYPE ghostdock_pulls_total counter
ghostdock_pulls_total {}

# HELP ghostdock_pushes_total Total number of image pushes
# TYPE ghostdock_pushes_total counter
ghostdock_pushes_total {}

# HELP ghostdock_storage_bytes Storage usage in bytes
# TYPE ghostdock_storage_bytes gauge
ghostdock_storage_bytes {}

# HELP ghostdock_version_info Version information
# TYPE ghostdock_version_info gauge
ghostdock_version_info{{version="{}"}} 1
"#,
        repo_count,
        user_count,
        total_pulls,
        total_pushes,
        storage_usage,
        crate::VERSION
    );

    Ok((
        [("content-type", "text/plain; version=0.0.4")],
        metrics,
    ))
}

fn calculate_storage_usage(path: &std::path::Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64>> + Send + '_>> {
    Box::pin(async move {
        let mut total_size = 0u64;
        
        if !path.exists() {
            return Ok(0);
        }

        let mut dir = tokio::fs::read_dir(path).await?;
        while let Some(entry) = dir.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                total_size += calculate_storage_usage(&entry.path()).await?;
            }
        }

        Ok(total_size)
    })
}
