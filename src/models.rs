use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: Option<String>,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub provider: Option<String>, // google, github, microsoft
    pub provider_id: Option<String>,
    pub is_admin: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RepositoryModel {
    pub id: Uuid,
    pub name: String,
    pub namespace: Option<String>,
    pub description: Option<String>,
    pub is_public: bool,
    pub owner_id: Uuid,
    pub star_count: i32,
    pub pull_count: i64,
    pub push_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ManifestModel {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub digest: String,
    pub media_type: String,
    pub schema_version: i32,
    pub content: Vec<u8>, // JSON content
    pub size: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TagModel {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub name: String,
    pub manifest_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_pulled: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BlobModel {
    pub id: Uuid,
    pub digest: String,
    pub media_type: String,
    pub size: i64,
    pub content_type: Option<String>,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RepositoryBlobModel {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub blob_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UploadSessionModel {
    pub id: Uuid,
    pub uuid: Uuid,
    pub repository_id: Uuid,
    pub uploaded_size: i64,
    pub total_size: Option<i64>,
    pub digest: Option<String>,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ComposeStackModel {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub name: String,
    pub version: String,
    pub content: Vec<u8>, // YAML content
    pub description: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DockerfileModel {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub name: String,
    pub version: String,
    pub content: Vec<u8>, // Dockerfile content
    pub description: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AccessTokenModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub token_hash: String,
    pub scopes: String, // JSON array of scopes
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RepositoryPermissionModel {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub user_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub permission: String, // read, write, admin
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AuditLogModel {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WebhookModel {
    pub id: Uuid,
    pub repository_id: Option<Uuid>,
    pub url: String,
    pub secret: Option<String>,
    pub events: String, // JSON array of event types
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WebhookDeliveryModel {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// Helper structs for API requests
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: Option<String>,
    pub full_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRepositoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateComposeStackRequest {
    pub name: String,
    pub version: String,
    pub content: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDockerfileRequest {
    pub name: String,
    pub version: String,
    pub content: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccessTokenRequest {
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserModel,
    pub expires_at: DateTime<Utc>,
}
