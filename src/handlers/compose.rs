use axum::{
    extract::{State, Path, Query},
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Docker Compose Stack Management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeStack {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub compose_content: String,
    pub registry_url: Option<String>, // URL where stack can be pulled from
    pub version: String,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub author: String,
    pub is_public: bool,
    pub download_count: u64,
    pub star_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StackManifest {
    pub apiVersion: String, // "compose.docker.com/v1"
    pub kind: String,       // "Stack"
    pub metadata: StackMetadata,
    pub spec: StackSpec,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StackMetadata {
    pub name: String,
    pub namespace: Option<String>,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StackSpec {
    pub compose: String, // Base64 encoded compose file
    pub services: Vec<ServiceReference>,
    pub networks: Vec<NetworkReference>,
    pub volumes: Vec<VolumeReference>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceReference {
    pub name: String,
    pub image: String,
    pub registry: Option<String>, // Which registry to pull from
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkReference {
    pub name: String,
    pub driver: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VolumeReference {
    pub name: String,
    pub driver: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStackRequest {
    pub name: String,
    pub description: Option<String>,
    pub compose_content: String,
    pub tags: Vec<String>,
    pub is_public: bool,
}

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub search: Option<String>,
    pub tags: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

pub fn routes() -> Router<crate::AppState> {
    Router::new()
        .route("/stacks", get(list_stacks).post(create_stack))
        .route("/stacks/:id", get(get_stack).put(update_stack).delete(delete_stack))
        .route("/stacks/:id/download", get(download_stack))
        .route("/stacks/:id/star", post(star_stack))
        .route("/stacks/:id/deploy", post(deploy_stack))
        .route("/stacks/import", post(import_from_url))
        .route("/stacks/search", get(search_stacks))
        .route("/registry/:registry/stacks", get(list_registry_stacks))
}

/// List all available stacks
async fn list_stacks(
    State(state): State<crate::AppState>,
    Query(params): Query<QueryParams>,
) -> Result<Json<Vec<ComposeStack>>, crate::error::GhostDockError> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    let mut query = "SELECT * FROM compose_stacks WHERE 1=1".to_string();
    let mut bind_params = Vec::new();

    if let Some(search) = &params.search {
        query.push_str(" AND (name LIKE ? OR description LIKE ?)");
        let search_pattern = format!("%{}%", search);
        bind_params.push(search_pattern.clone());
        bind_params.push(search_pattern);
    }

    if let Some(tags) = &params.tags {
        let tag_list: Vec<&str> = tags.split(',').collect();
        let placeholders = vec!["?"; tag_list.len()].join(",");
        query.push_str(&format!(" AND tags && ARRAY[{}]", placeholders));
        bind_params.extend(tag_list.iter().map(|s| s.to_string()));
    }

    query.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
    bind_params.push(limit.to_string());
    bind_params.push(offset.to_string());

    // Execute query (simplified - would use actual database)
    let stacks = vec![]; // TODO: Implement actual database query

    Ok(Json(stacks))
}

/// Create a new stack
async fn create_stack(
    State(state): State<crate::AppState>,
    Json(request): Json<CreateStackRequest>,
) -> Result<Json<ComposeStack>, crate::error::GhostDockError> {
    // Validate compose file
    validate_compose_content(&request.compose_content)?;

    let stack = ComposeStack {
        id: Uuid::new_v4(),
        name: request.name,
        description: request.description,
        compose_content: request.compose_content,
        registry_url: Some(format!("https://registry.ghostdock.com/stacks/{}", Uuid::new_v4())),
        version: "1.0.0".to_string(),
        tags: request.tags,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        author: "user".to_string(), // TODO: Get from auth context
        is_public: request.is_public,
        download_count: 0,
        star_count: 0,
    };

    // TODO: Save to database
    // TODO: Generate registry manifest

    Ok(Json(stack))
}

/// Import stack from external URL
async fn import_from_url(
    State(state): State<crate::AppState>,
    Json(url): Json<String>,
) -> Result<Json<ComposeStack>, crate::error::GhostDockError> {
    // Fetch compose file from URL
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    let compose_content = response.text().await?;

    // Parse and validate
    validate_compose_content(&compose_content)?;

    // Extract metadata from compose file
    let metadata = extract_compose_metadata(&compose_content)?;

    let stack = ComposeStack {
        id: Uuid::new_v4(),
        name: metadata.name.unwrap_or_else(|| "Imported Stack".to_string()),
        description: metadata.description,
        compose_content,
        registry_url: Some(url),
        version: metadata.version.unwrap_or_else(|| "1.0.0".to_string()),
        tags: metadata.tags.unwrap_or_default(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        author: "imported".to_string(),
        is_public: false,
        download_count: 0,
        star_count: 0,
    };

    // TODO: Save to database
    Ok(Json(stack))
}

/// Deploy stack to Docker daemon
async fn deploy_stack(
    State(state): State<crate::AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, crate::error::GhostDockError> {
    // TODO: Get stack from database
    // TODO: Parse compose file
    // TODO: Deploy using Docker API or docker-compose command

    Ok(Json(serde_json::json!({
        "status": "deployed",
        "deployment_id": Uuid::new_v4(),
        "services": ["web", "db", "redis"],
        "networks": ["app-network"],
        "volumes": ["app-data"]
    })))
}

/// Download stack as compose file
async fn download_stack(
    State(state): State<crate::AppState>,
    Path(id): Path<Uuid>,
) -> Result<String, crate::error::GhostDockError> {
    // TODO: Get stack from database
    // TODO: Increment download count
    
    // Return compose content with proper headers
    Ok("version: '3.8'\nservices:\n  web:\n    image: nginx:latest".to_string())
}

fn validate_compose_content(content: &str) -> Result<(), crate::error::GhostDockError> {
    // Parse YAML
    let _parsed: serde_yaml::Value = serde_yaml::from_str(content)
        .map_err(|e| crate::error::GhostDockError::Validation(format!("Invalid YAML: {}", e)))?;

    // TODO: Validate compose schema
    // TODO: Check for security issues
    // TODO: Validate image references

    Ok(())
}

#[derive(Debug, Deserialize)]
struct ComposeMetadata {
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
    tags: Option<Vec<String>>,
}

fn extract_compose_metadata(content: &str) -> Result<ComposeMetadata, crate::error::GhostDockError> {
    let parsed: serde_yaml::Value = serde_yaml::from_str(content)?;
    
    // Extract metadata from labels or x-metadata section
    let metadata = if let Some(x_metadata) = parsed.get("x-metadata") {
        serde_yaml::from_value(x_metadata.clone()).unwrap_or_default()
    } else {
        ComposeMetadata {
            name: None,
            description: None,
            version: None,
            tags: None,
        }
    };

    Ok(metadata)
}

async fn search_stacks(
    State(state): State<crate::AppState>,
    Query(params): Query<QueryParams>,
) -> Result<Json<Vec<ComposeStack>>, crate::error::GhostDockError> {
    // Similar to list_stacks but with enhanced search
    list_stacks(State(state), Query(params)).await
}

async fn get_stack(
    State(state): State<crate::AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ComposeStack>, crate::error::GhostDockError> {
    // TODO: Implement
    Err(crate::error::GhostDockError::NotFound("Stack not found".to_string()))
}

async fn update_stack(
    State(state): State<crate::AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<CreateStackRequest>,
) -> Result<Json<ComposeStack>, crate::error::GhostDockError> {
    // TODO: Implement
    Err(crate::error::GhostDockError::NotFound("Stack not found".to_string()))
}

async fn delete_stack(
    State(state): State<crate::AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, crate::error::GhostDockError> {
    // TODO: Implement
    Ok(Json(serde_json::json!({"deleted": true})))
}

async fn star_stack(
    State(state): State<crate::AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, crate::error::GhostDockError> {
    // TODO: Implement starring system
    Ok(Json(serde_json::json!({"starred": true})))
}

async fn list_registry_stacks(
    State(state): State<crate::AppState>,
    Path(registry): Path<String>,
) -> Result<Json<Vec<ComposeStack>>, crate::error::GhostDockError> {
    // TODO: Implement cross-registry stack discovery
    Ok(Json(vec![]))
}
