use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, put, head, delete, patch},
    Router, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sha2::{Sha256, Digest};
use hex;

use crate::{
    auth::middleware::{AuthenticatedUser, get_authenticated_user},
    database::Database,
    storage::Storage,
    error::Result,
};

/// Docker Registry v2 API implementation
/// Follows the Docker Registry HTTP API V2 specification

#[derive(Clone)]
pub struct RegistryState {
    pub database: Arc<Database>,
    pub storage: Arc<Storage>,
}

/// Registry API routes
pub fn registry_routes() -> Router<RegistryState> {
    Router::new()
        // Base API endpoint
        .route("/v2/", get(check_api_version))
        
        // Blob endpoints
        .route("/v2/:name/blobs/:digest", get(get_blob))
        .route("/v2/:name/blobs/:digest", head(head_blob))
        .route("/v2/:name/blobs/:digest", delete(delete_blob))
        
        // Blob upload endpoints
        .route("/v2/:name/blobs/uploads/", post(initiate_blob_upload))
        .route("/v2/:name/blobs/uploads/:uuid", put(complete_blob_upload))
        .route("/v2/:name/blobs/uploads/:uuid", patch(upload_blob_chunk))
        .route("/v2/:name/blobs/uploads/:uuid", get(get_upload_status))
        .route("/v2/:name/blobs/uploads/:uuid", delete(cancel_upload))
        
        // Manifest endpoints
        .route("/v2/:name/manifests/:reference", get(get_manifest))
        .route("/v2/:name/manifests/:reference", put(put_manifest))
        .route("/v2/:name/manifests/:reference", head(head_manifest))
        .route("/v2/:name/manifests/:reference", delete(delete_manifest))
        
        // Tag listing
        .route("/v2/:name/tags/list", get(list_tags))
        
        // Catalog (repository listing)
        .route("/v2/_catalog", get(get_catalog))
}

/// Check API version - Docker Registry API base endpoint
async fn check_api_version() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Docker-Distribution-Api-Version", "registry/2.0")
        .body("{}".to_string())
        .unwrap()
}

/// Get a blob by digest
async fn get_blob(
    Path((name, digest)): Path<(String, String)>,
    State(state): State<RegistryState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse> {
    // Validate repository access
    if !validate_repository_access(&name, "read").await? {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    // Get blob from storage
    match state.storage.get_blob(&digest).await {
        Ok(Some(blob_data)) => {
            let mut response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/octet-stream")
                .header("Content-Length", blob_data.len().to_string())
                .header("Docker-Content-Digest", &digest);

            // Handle range requests
            if let Some(range) = headers.get("range") {
                if let Ok(range_str) = range.to_str() {
                    if let Some(range_data) = handle_range_request(&blob_data, range_str) {
                        response = response.status(StatusCode::PARTIAL_CONTENT)
                            .header("Content-Range", format!("bytes {}-{}/{}", 
                                range_data.start, range_data.end, blob_data.len()));
                        return Ok(response.body(Body::from(range_data.data)).unwrap().into_response());
                    }
                }
            }

            Ok(response.body(Body::from(blob_data)).unwrap().into_response())
        }
        Ok(None) => Ok(StatusCode::NOT_FOUND.into_response()),
        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    }
}

/// Head request for blob
async fn head_blob(
    Path((name, digest)): Path<(String, String)>,
    State(state): State<RegistryState>,
) -> Result<impl IntoResponse> {
    if !validate_repository_access(&name, "read").await? {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    match state.storage.blob_exists(&digest).await {
        Ok(true) => {
            let size = state.storage.blob_size(&digest).await.unwrap_or(0);
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Length", size.to_string())
                .header("Docker-Content-Digest", &digest)
                .body("".to_string())
                .unwrap()
                .into_response())
        }
        Ok(false) => Ok(StatusCode::NOT_FOUND.into_response()),
        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    }
}

/// Delete a blob
async fn delete_blob(
    Path((name, digest)): Path<(String, String)>,
    State(state): State<RegistryState>,
) -> Result<impl IntoResponse> {
    if !validate_repository_access(&name, "delete").await? {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    match state.storage.delete_blob(&digest).await {
        Ok(()) => Ok(StatusCode::ACCEPTED.into_response()),
        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    }
}

/// Initiate blob upload
async fn initiate_blob_upload(
    Path(name): Path<String>,
    State(state): State<RegistryState>,
) -> Result<impl IntoResponse> {
    if !validate_repository_access(&name, "write").await? {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    let upload_uuid = uuid::Uuid::new_v4().to_string();
    
    // Create upload session in database
    // TODO: Implement upload session creation
    
    Ok(Response::builder()
        .status(StatusCode::ACCEPTED)
        .header("Location", format!("/v2/{}/blobs/uploads/{}", name, upload_uuid))
        .header("Range", "bytes=0-0")
        .header("Content-Length", "0")
        .header("Docker-Upload-UUID", &upload_uuid)
        .body("".to_string())
        .unwrap()
        .into_response())
}

/// Complete blob upload
async fn complete_blob_upload(
    Path((name, uuid)): Path<(String, String)>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    State(state): State<RegistryState>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse> {
    if !validate_repository_access(&name, "write").await? {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    let digest = params.get("digest").ok_or_else(|| {
        crate::error::Error::from(anyhow::anyhow!("Missing digest parameter"))
    })?;

    // Store the blob
    match state.storage.store_blob(digest.clone(), &body).await {
        Ok(()) => {
            Ok(Response::builder()
                .status(StatusCode::CREATED)
                .header("Location", format!("/v2/{}/blobs/{}", name, digest))
                .header("Content-Length", "0")
                .header("Docker-Content-Digest", digest)
                .body("".to_string())
                .unwrap()
                .into_response())
        }
        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    }
}

/// Upload blob chunk (for chunked uploads)
async fn upload_blob_chunk(
    Path((name, uuid)): Path<(String, String)>,
    headers: HeaderMap,
    State(_state): State<RegistryState>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse> {
    if !validate_repository_access(&name, "write").await? {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    // Handle chunked upload
    // TODO: Implement chunked upload logic
    
    let content_range = headers.get("content-range")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("bytes 0-0/*");

    Ok(Response::builder()
        .status(StatusCode::ACCEPTED)
        .header("Location", format!("/v2/{}/blobs/uploads/{}", name, uuid))
        .header("Range", content_range)
        .header("Content-Length", "0")
        .header("Docker-Upload-UUID", &uuid)
        .body("".to_string())
        .unwrap()
        .into_response())
}

/// Get upload status
async fn get_upload_status(
    Path((name, uuid)): Path<(String, String)>,
    State(_state): State<RegistryState>,
) -> Result<impl IntoResponse> {
    if !validate_repository_access(&name, "read").await? {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    // TODO: Get actual upload status from database
    
    Ok(Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("Location", format!("/v2/{}/blobs/uploads/{}", name, uuid))
        .header("Range", "bytes=0-0")
        .header("Docker-Upload-UUID", &uuid)
        .body("".to_string())
        .unwrap()
        .into_response())
}

/// Cancel upload
async fn cancel_upload(
    Path((name, uuid)): Path<(String, String)>,
    State(_state): State<RegistryState>,
) -> Result<impl IntoResponse> {
    if !validate_repository_access(&name, "write").await? {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    // TODO: Cancel upload session
    
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Get manifest
async fn get_manifest(
    Path((name, reference)): Path<(String, String)>,
    headers: HeaderMap,
    State(state): State<RegistryState>,
) -> Result<impl IntoResponse> {
    if !validate_repository_access(&name, "read").await? {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    // Handle different manifest media types
    let accept_header = headers.get("accept")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("application/vnd.docker.distribution.manifest.v2+json");

    match state.storage.get_manifest(&name, &reference).await {
        Ok(Some(manifest)) => {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", accept_header)
                .header("Content-Length", manifest.len().to_string())
                .header("Docker-Content-Digest", format!("sha256:{}", hex::encode(Sha256::digest(&manifest))))
                .body(manifest)
                .unwrap()
                .into_response())
        }
        Ok(None) => Ok(StatusCode::NOT_FOUND.into_response()),
        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    }
}

/// Put manifest
async fn put_manifest(
    Path((name, reference)): Path<(String, String)>,
    headers: HeaderMap,
    State(state): State<RegistryState>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse> {
    if !validate_repository_access(&name, "write").await? {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    let manifest_str = String::from_utf8_lossy(&body);
    let digest = format!("sha256:{}", hex::encode(Sha256::digest(&body)));

    match state.storage.store_manifest(&name, &reference, &manifest_str).await {
        Ok(()) => {
            Ok(Response::builder()
                .status(StatusCode::CREATED)
                .header("Location", format!("/v2/{}/manifests/{}", name, digest))
                .header("Content-Length", "0")
                .header("Docker-Content-Digest", &digest)
                .body("".to_string())
                .unwrap()
                .into_response())
        }
        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    }
}

/// Head manifest
async fn head_manifest(
    Path((name, reference)): Path<(String, String)>,
    State(state): State<RegistryState>,
) -> Result<impl IntoResponse> {
    if !validate_repository_access(&name, "read").await? {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    match state.storage.manifest_exists(&name, &reference).await {
        Ok(true) => {
            // TODO: Get actual manifest size and digest
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/vnd.docker.distribution.manifest.v2+json")
                .header("Content-Length", "0")
                .body("".to_string())
                .unwrap()
                .into_response())
        }
        Ok(false) => Ok(StatusCode::NOT_FOUND.into_response()),
        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    }
}

/// Delete manifest
async fn delete_manifest(
    Path((name, reference)): Path<(String, String)>,
    State(state): State<RegistryState>,
) -> Result<impl IntoResponse> {
    if !validate_repository_access(&name, "delete").await? {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    match state.storage.delete_manifest(&name, &reference).await {
        Ok(()) => Ok(StatusCode::ACCEPTED.into_response()),
        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    }
}

/// List tags for a repository
async fn list_tags(
    Path(name): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    State(state): State<RegistryState>,
) -> Result<impl IntoResponse> {
    if !validate_repository_access(&name, "read").await? {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    let n = params.get("n").and_then(|s| s.parse::<usize>().ok()).unwrap_or(100);
    let last = params.get("last");

    match state.storage.list_tags(&name, Some(n), last.map(|s| s.as_str())).await {
        Ok(tags) => {
            let response = serde_json::json!({
                "name": name,
                "tags": tags
            });
            Ok(Json(response).into_response())
        }
        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    }
}

/// Get repository catalog
async fn get_catalog(
    Query(params): Query<std::collections::HashMap<String, String>>,
    State(state): State<RegistryState>,
) -> Result<impl IntoResponse> {
    let n = params.get("n").and_then(|s| s.parse::<usize>().ok()).unwrap_or(100);
    let last = params.get("last");

    match state.storage.list_repositories(Some(n), last.map(|s| s.as_str())).await {
        Ok(repositories) => {
            let response = serde_json::json!({
                "repositories": repositories
            });
            Ok(Json(response).into_response())
        }
        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    }
}

/// Helper functions

#[derive(Debug)]
struct RangeData {
    start: usize,
    end: usize,
    data: Vec<u8>,
}

fn handle_range_request(data: &[u8], range_header: &str) -> Option<RangeData> {
    // Parse range header like "bytes=0-1023"
    if let Some(range_part) = range_header.strip_prefix("bytes=") {
        if let Some((start_str, end_str)) = range_part.split_once('-') {
            let start = start_str.parse::<usize>().ok()?;
            let end = if end_str.is_empty() {
                data.len() - 1
            } else {
                end_str.parse::<usize>().ok()?.min(data.len() - 1)
            };
            
            if start <= end && start < data.len() {
                return Some(RangeData {
                    start,
                    end,
                    data: data[start..=end].to_vec(),
                });
            }
        }
    }
    None
}

async fn validate_repository_access(repository: &str, action: &str) -> Result<bool> {
    // TODO: Implement proper repository access validation
    // For now, allow all access
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_request_parsing() {
        let data = b"Hello, World!";
        
        let range = handle_range_request(data, "bytes=0-4").unwrap();
        assert_eq!(range.start, 0);
        assert_eq!(range.end, 4);
        assert_eq!(range.data, b"Hello");
        
        let range = handle_range_request(data, "bytes=7-").unwrap();
        assert_eq!(range.start, 7);
        assert_eq!(range.end, 12);
        assert_eq!(range.data, b"World!");
    }
}
