use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put, delete},
    Router, Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    auth::middleware::AuthenticatedUser,
    database::Database,
    error::Result,
    server::AppState,
};

/// Docker Compose Stack Management
/// Allows users to save, share, and deploy Docker Compose stacks

/// Stack definition
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stack {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub compose_content: String,
    pub version: String,
    pub author: String,
    pub author_email: String,
    pub tags: Vec<String>,
    pub is_public: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub download_count: u64,
    pub star_count: u64,
}

/// Stack creation request
#[derive(Debug, Deserialize)]
pub struct CreateStackRequest {
    pub name: String,
    pub description: Option<String>,
    pub compose_content: String,
    pub tags: Vec<String>,
    pub is_public: bool,
}

/// Stack update request
#[derive(Debug, Deserialize)]
pub struct UpdateStackRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub compose_content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_public: Option<bool>,
}

/// Stack query parameters
#[derive(Debug, Deserialize)]
pub struct StackQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub search: Option<String>,
    pub tags: Option<String>,
    pub author: Option<String>,
    pub public_only: Option<bool>,
}

/// Stack import from URL request
#[derive(Debug, Deserialize)]
pub struct ImportStackRequest {
    pub url: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_public: Option<bool>,
}

/// Stack routes
pub fn stack_routes() -> Router<AppState> {
    Router::new()
        // Stack CRUD operations  
        .route("/api/stacks", get(list_stacks).post(create_stack))
        .route("/api/stacks/:id", get(get_stack).put(update_stack).delete(delete_stack))
        
        // Stack sharing and discovery
        .route("/api/stacks/:id/star", post(star_stack))
        .route("/api/stacks/:id/unstar", post(unstar_stack))
        .route("/api/stacks/:id/download", get(download_stack))
        .route("/api/stacks/:id/raw", get(get_stack_raw))
        
        // Stack import/export
        .route("/api/stacks/import", post(import_stack_from_url))
        .route("/api/stacks/:id/export", get(export_stack))
        
        // Stack deployment
        .route("/api/stacks/:id/deploy", post(deploy_stack))
        .route("/api/stacks/:id/undeploy", post(undeploy_stack))
        .route("/api/stacks/:id/status", get(get_deployment_status))
        
        // Public stack registry
        .route("/api/registry/stacks", get(list_public_stacks))
        .route("/api/registry/stacks/featured", get(list_featured_stacks))
        .route("/api/registry/stacks/popular", get(list_popular_stacks))
}

/// List stacks for the authenticated user
async fn list_stacks(
    Query(query): Query<StackQuery>,
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    
    // Build filter conditions
    let mut conditions = vec![];
    let mut params = vec![];
    
    if !query.public_only.unwrap_or(false) {
        conditions.push("(author = ? OR is_public = true)");
        params.push(user.id.clone());
    } else {
        conditions.push("is_public = true");
    }
    
    if let Some(search) = &query.search {
        conditions.push("(name LIKE ? OR description LIKE ?)");
        let search_pattern = format!("%{}%", search);
        params.push(search_pattern.clone());
        params.push(search_pattern);
    }
    
    if let Some(tags) = &query.tags {
        let tag_list: Vec<&str> = tags.split(',').collect();
        for tag in tag_list {
            conditions.push("tags LIKE ?");
            params.push(format!("%{}%", tag));
        }
    }
    
    if let Some(author) = &query.author {
        conditions.push("author = ?");
        params.push(author.clone());
    }
    
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };
    
    // TODO: Execute database query
    let stacks: Vec<Stack> = vec![]; // Placeholder
    
    Ok(Json(serde_json::json!({
        "stacks": stacks,
        "total": 0,
        "limit": limit,
        "offset": offset
    })))
}

/// Create a new stack
#[axum::debug_handler]
async fn create_stack(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<CreateStackRequest>,
) -> Result<impl IntoResponse> {
    // Validate compose content
    if let Err(validation_error) = validate_compose_content(&request.compose_content) {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid Docker Compose content",
                "details": validation_error
            }))
        ).into_response());
    }
    
    let stack = Stack {
        id: Uuid::new_v4().to_string(),
        name: request.name,
        description: request.description,
        compose_content: request.compose_content,
        version: "1.0.0".to_string(),
        author: user.id.clone(),
        author_email: user.email.clone(),
        tags: request.tags,
        is_public: request.is_public,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        download_count: 0,
        star_count: 0,
    };
    
    // TODO: Save stack to database
    
    Ok((StatusCode::CREATED, Json(&stack)).into_response())
}

/// Get a specific stack
async fn get_stack(
    Path(id): Path<String>,
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse> {
    // TODO: Get stack from database
    // Check if user has access (owner or public)
    
    let stack = Stack {
        id: id.clone(),
        name: "Example Stack".to_string(),
        description: Some("An example Docker Compose stack".to_string()),
        compose_content: "version: '3.8'\nservices:\n  web:\n    image: nginx:latest".to_string(),
        version: "1.0.0".to_string(),
        author: user.id.clone(),
        author_email: user.email.clone(),
        tags: vec!["web".to_string(), "nginx".to_string()],
        is_public: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        download_count: 42,
        star_count: 5,
    };
    
    Ok(Json(stack))
}

/// Update a stack
async fn update_stack(
    Path(id): Path<String>,
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<UpdateStackRequest>,
) -> Result<impl IntoResponse> {
    // TODO: Update stack in database
    // Check if user is the owner
    
    if let Some(compose_content) = &request.compose_content {
        if let Err(validation_error) = validate_compose_content(compose_content) {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid Docker Compose content",
                    "details": validation_error
                }))
            ).into_response());
        }
    }
    
    Ok((StatusCode::OK, Json(serde_json::json!({"message": "Stack updated successfully"}))).into_response())
}

/// Delete a stack
async fn delete_stack(
    Path(id): Path<String>,
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse> {
    // TODO: Delete stack from database
    // Check if user is the owner
    
    Ok(StatusCode::NO_CONTENT)
}

/// Star a stack
async fn star_stack(
    Path(id): Path<String>,
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse> {
    // TODO: Add star to database
    
    Ok(Json(serde_json::json!({"message": "Stack starred successfully"})))
}

/// Unstar a stack
async fn unstar_stack(
    Path(id): Path<String>,
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse> {
    // TODO: Remove star from database
    
    Ok(Json(serde_json::json!({"message": "Stack unstarred successfully"})))
}

/// Download a stack (increment download counter)
async fn download_stack(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    // TODO: Increment download counter in database
    // Return the stack content
    
    let stack = Stack {
        id: id.clone(),
        name: "Example Stack".to_string(),
        description: Some("An example Docker Compose stack".to_string()),
        compose_content: "version: '3.8'\nservices:\n  web:\n    image: nginx:latest".to_string(),
        version: "1.0.0".to_string(),
        author: "user123".to_string(),
        author_email: "user@example.com".to_string(),
        tags: vec!["web".to_string(), "nginx".to_string()],
        is_public: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        download_count: 43,
        star_count: 5,
    };
    
    Ok(Json(stack))
}

/// Get raw stack content
async fn get_stack_raw(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    // TODO: Get stack from database
    
    let compose_content = "version: '3.8'\nservices:\n  web:\n    image: nginx:latest\n    ports:\n      - \"80:80\"";
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/yaml")
        .header("Content-Disposition", format!("attachment; filename=\"{}.yml\"", id))
        .body(axum::body::Body::from(compose_content))
        .unwrap())
}

/// Import stack from URL
async fn import_stack_from_url(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<ImportStackRequest>,
) -> Result<impl IntoResponse> {
    // Validate URL
    if !is_valid_compose_url(&request.url) {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid URL or unsupported source"
            }))
        ).into_response());
    }
    
    // Fetch compose content from URL
    let client = reqwest::Client::new();
    let response = client.get(&request.url).send().await
        .map_err(|e| crate::error::Error::from(anyhow::anyhow!("Failed to fetch URL: {}", e)))?;
    
    if !response.status().is_success() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Failed to fetch compose file from URL"
            }))
        ).into_response());
    }
    
    let compose_content = response.text().await
        .map_err(|e| crate::error::Error::from(anyhow::anyhow!("Failed to read response: {}", e)))?;
    
    // Validate compose content
    if let Err(validation_error) = validate_compose_content(&compose_content) {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid Docker Compose content",
                "details": validation_error
            }))
        ).into_response());
    }
    
    // Extract name from URL if not provided
    let stack_name = request.name.unwrap_or_else(|| {
        extract_name_from_url(&request.url).unwrap_or_else(|| "imported-stack".to_string())
    });
    
    let stack = Stack {
        id: Uuid::new_v4().to_string(),
        name: stack_name,
        description: request.description.or_else(|| Some("Imported from URL".to_string())),
        compose_content,
        version: "1.0.0".to_string(),
        author: user.id.clone(),
        author_email: user.email.clone(),
        tags: request.tags.unwrap_or_default(),
        is_public: request.is_public.unwrap_or(false),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        download_count: 0,
        star_count: 0,
    };
    
    // TODO: Save stack to database
    
    Ok((StatusCode::CREATED, Json(&stack)).into_response())
}

/// Export stack
async fn export_stack(
    Path(id): Path<String>,
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse> {
    // TODO: Get stack from database and create export package
    
    let export_data = serde_json::json!({
        "format": "ghostdock-stack-v1",
        "exported_at": chrono::Utc::now(),
        "exported_by": user.email,
        "stack": {
            "name": "example-stack",
            "description": "Example stack",
            "compose_content": "version: '3.8'\nservices:\n  web:\n    image: nginx:latest"
        }
    });
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Content-Disposition", format!("attachment; filename=\"{}.json\"", id))
        .body(axum::body::Body::from(export_data.to_string()))
        .unwrap())
}

/// Deploy stack
async fn deploy_stack(
    Path(id): Path<String>,
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse> {
    // TODO: Implement stack deployment using Docker Compose
    // This would require integration with Docker daemon
    
    Ok(Json(serde_json::json!({
        "message": "Stack deployment initiated",
        "deployment_id": Uuid::new_v4().to_string(),
        "status": "deploying"
    })))
}

/// Undeploy stack
async fn undeploy_stack(
    Path(id): Path<String>,
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse> {
    // TODO: Implement stack undeployment
    
    Ok(Json(serde_json::json!({
        "message": "Stack undeployment initiated",
        "status": "undeploying"
    })))
}

/// Get deployment status
async fn get_deployment_status(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    // TODO: Get actual deployment status
    
    Ok(Json(serde_json::json!({
        "stack_id": id,
        "status": "running",
        "services": [
            {
                "name": "web",
                "status": "running",
                "replicas": "1/1"
            }
        ],
        "last_updated": chrono::Utc::now()
    })))
}

/// List public stacks
async fn list_public_stacks(
    Query(query): Query<StackQuery>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    // TODO: Implement public stack listing
    
    Ok(Json(serde_json::json!({
        "stacks": [],
        "total": 0
    })))
}

/// List featured stacks
async fn list_featured_stacks(
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    // TODO: Implement featured stack listing
    
    Ok(Json(serde_json::json!({
        "stacks": []
    })))
}

/// List popular stacks
async fn list_popular_stacks(
    Query(query): Query<StackQuery>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    // TODO: Implement popular stack listing (sorted by stars/downloads)
    
    Ok(Json(serde_json::json!({
        "stacks": []
    })))
}

/// Helper functions

fn validate_compose_content(content: &str) -> std::result::Result<(), String> {
    // Basic YAML validation
    match serde_yaml::from_str::<serde_yaml::Value>(content) {
        Ok(parsed) => {
            // Check for required compose fields
            if let Some(obj) = parsed.as_mapping() {
                if !obj.contains_key(&serde_yaml::Value::String("version".to_string())) {
                    return Err("Missing 'version' field".to_string());
                }
                if !obj.contains_key(&serde_yaml::Value::String("services".to_string())) {
                    return Err("Missing 'services' field".to_string());
                }
                Ok(())
            } else {
                Err("Invalid YAML structure".to_string())
            }
        }
        Err(e) => Err(format!("YAML parsing error: {}", e)),
    }
}

fn is_valid_compose_url(url: &str) -> bool {
    // Check if URL is valid and from allowed sources
    if let Ok(parsed_url) = url::Url::parse(url) {
        match parsed_url.scheme() {
            "http" | "https" => {
                // Allow GitHub raw URLs, GitLab raw URLs, etc.
                let host = parsed_url.host_str().unwrap_or("");
                matches!(host, 
                    "raw.githubusercontent.com" |
                    "gist.githubusercontent.com" |
                    "gitlab.com" |
                    "bitbucket.org" |
                    "gist.github.com"
                ) || host.ends_with(".github.io")
            }
            _ => false,
        }
    } else {
        false
    }
}

fn extract_name_from_url(url: &str) -> Option<String> {
    if let Ok(parsed_url) = url::Url::parse(url) {
        let path = parsed_url.path();
        if let Some(filename) = path.split('/').last() {
            let name = filename.trim_end_matches(".yml").trim_end_matches(".yaml");
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_compose_content() {
        let valid_compose = r#"
version: '3.8'
services:
  web:
    image: nginx:latest
    ports:
      - "80:80"
"#;
        assert!(validate_compose_content(valid_compose).is_ok());

        let invalid_compose = "invalid yaml {[}";
        assert!(validate_compose_content(invalid_compose).is_err());

        let missing_version = r#"
services:
  web:
    image: nginx:latest
"#;
        assert!(validate_compose_content(missing_version).is_err());
    }

    #[test]
    fn test_is_valid_compose_url() {
        assert!(is_valid_compose_url("https://raw.githubusercontent.com/user/repo/main/docker-compose.yml"));
        assert!(is_valid_compose_url("https://gist.githubusercontent.com/user/id/raw/docker-compose.yml"));
        assert!(!is_valid_compose_url("https://malicious-site.com/compose.yml"));
        assert!(!is_valid_compose_url("ftp://example.com/compose.yml"));
    }

    #[test]
    fn test_extract_name_from_url() {
        assert_eq!(
            extract_name_from_url("https://raw.githubusercontent.com/user/repo/main/docker-compose.yml"),
            Some("docker-compose".to_string())
        );
        assert_eq!(
            extract_name_from_url("https://example.com/my-stack.yaml"),
            Some("my-stack".to_string())
        );
    }
}
