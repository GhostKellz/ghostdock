use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Database model types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Blob {
    pub id: Uuid,
    pub digest: String,
    pub media_type: String,
    pub size: i64,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub digest: String,
    pub media_type: String,
    pub content: String,
    pub size: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub name: String,
    pub manifest_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UploadSession {
    pub id: Uuid,
    pub uuid: Uuid,
    pub repository_id: Uuid,
    pub uploaded_size: i64,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Docker Registry v2 API types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryError {
    pub code: String,
    pub message: String,
    pub detail: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryErrorResponse {
    pub errors: Vec<RegistryError>,
}

/// Manifest types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mediaType")]
pub enum ManifestType {
    #[serde(rename = "application/vnd.docker.distribution.manifest.v1+json")]
    V1(ManifestV1),
    #[serde(rename = "application/vnd.docker.distribution.manifest.v2+json")]
    V2(ManifestV2),
    #[serde(rename = "application/vnd.docker.distribution.manifest.list.v2+json")]
    List(ManifestList),
    #[serde(rename = "application/vnd.oci.image.manifest.v1+json")]
    Oci(OciManifest),
    #[serde(rename = "application/vnd.oci.image.index.v1+json")]
    OciIndex(OciIndex),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestV1 {
    pub name: String,
    pub tag: String,
    pub architecture: String,
    pub history: Vec<V1History>,
    #[serde(rename = "fsLayers")]
    pub fs_layers: Vec<V1Layer>,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub signatures: Vec<V1Signature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V1History {
    pub v1_compatibility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V1Layer {
    #[serde(rename = "blobSum")]
    pub blob_sum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V1Signature {
    pub header: V1SignatureHeader,
    pub signature: String,
    pub protected: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V1SignatureHeader {
    pub jwk: serde_json::Value,
    pub alg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestV2 {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub config: Descriptor,
    pub layers: Vec<Descriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestList {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub manifests: Vec<ManifestListEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestListEntry {
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub size: u64,
    pub digest: String,
    pub platform: Option<Platform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform {
    pub architecture: String,
    pub os: String,
    #[serde(rename = "os.version", skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
    #[serde(rename = "os.features", skip_serializing_if = "Option::is_none")]
    pub os_features: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub config: Descriptor,
    pub layers: Vec<Descriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciIndex {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub manifests: Vec<Descriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Descriptor {
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub size: u64,
    pub digest: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<Platform>,
}

/// User and authentication types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub is_admin: bool,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
    pub exp: usize,
    pub iat: usize,
}

/// API response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagsResponse {
    pub name: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryListResponse {
    pub repositories: Vec<Repository>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime: u64,
    pub database: String,
    pub storage: String,
}
