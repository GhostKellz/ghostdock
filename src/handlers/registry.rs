use crate::{
    error::{Error, Result},
    server::AppState,
    storage::Storage,
    types::*,
    utils::{validate_repository_name, validate_tag_name, validate_digest, sha256_digest},
    database::queries::*,
};
use axum::{
    extract::{Path, State, Query, Request},
    response::{IntoResponse, Response},
    body::Body,
    http::{StatusCode, HeaderMap, header},
    Json,
};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;
use tokio::io::AsyncReadExt;

/// Docker Registry v2 API root endpoint
/// Returns API version information
pub async fn root() -> Result<impl IntoResponse> {
    Ok((
        StatusCode::OK,
        Json(json!({
            "registry": {
                "version": "2.0",
                "name": "GhostDock Registry",
                "vendor": "ghostkellz",
                "build": env!("CARGO_PKG_VERSION")
            }
        }))
    ))
}

/// Get blob by digest
pub async fn get_blob(
    State(state): State<AppState>,
    Path((name, digest)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    // Validate inputs
    validate_repository_name(&name)?;
    validate_digest(&digest)?;

    // Get blob data from storage
    let blob_data = state.storage.get_blob(&digest).await
        .map_err(|e| Error::Storage { message: e.to_string() })?;
    
    // Create response headers
    let mut headers = HeaderMap::new();
    headers.insert("content-type", "application/octet-stream".parse().unwrap());
    headers.insert("docker-content-digest", digest.parse().unwrap());
    
    // Return the blob data if found
    match blob_data {
        Some(data) => {
            headers.insert("content-length", data.len().to_string().parse().unwrap());
            Ok((StatusCode::OK, headers, data))
        }
        None => Err(Error::NotFound {
            resource: format!("blob {}", digest),
        })
    }
}

/// Head blob by digest (same as GET but without body)
pub async fn head_blob(
    State(state): State<AppState>,
    Path((name, digest)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    // Validate inputs
    validate_repository_name(&name)?;
    validate_digest(&digest)?;

    // Check if repository exists
    let repo = get_repository_by_name(&state, &name).await?;
    
    // Check if blob exists for this repository
    let blob = get_blob_by_digest(&state, &repo.id, &digest).await?;
    
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        blob.media_type.parse().unwrap_or("application/octet-stream".parse().unwrap())
    );
    headers.insert(
        header::CONTENT_LENGTH,
        blob.size.to_string().parse().unwrap()
    );
    headers.insert(
        "Docker-Content-Digest",
        digest.parse().unwrap()
    );

    Ok((StatusCode::OK, headers))
}

/// Delete blob by digest
pub async fn delete_blob(
    State(state): State<AppState>,
    Path((name, digest)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    // Validate inputs
    validate_repository_name(&name)?;
    validate_digest(&digest)?;

    // Check if repository exists
    let repo = get_repository_by_name(&state, &name).await?;
    
    // Check if blob exists for this repository
    let blob = get_blob_by_digest(&state, &repo.id, &digest).await?;
    
    // Remove blob from storage
    state.storage.delete_blob(&digest).await?;
    
    // Remove blob from database
    sqlx::query("DELETE FROM blobs WHERE id = $1")
        .bind(&blob.id)
        .execute(&state.database.pool)
        .await?;
    
    // Remove repository-blob relationship
    sqlx::query("DELETE FROM repository_blobs WHERE repository_id = $1 AND blob_id = $2")
        .bind(&repo.id)
        .bind(&blob.id)
        .execute(&state.database.pool)
        .await?;

    Ok(StatusCode::ACCEPTED)
}

/// Initiate blob upload
pub async fn initiate_blob_upload(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse> {
    validate_repository_name(&name)?;

    // Get or create repository
    let repo = get_or_create_repository(&state, &name).await?;
    
    // Create upload session
    let upload_uuid = Uuid::new_v4();
    let storage_path = format!("uploads/{}", upload_uuid);
    
    sqlx::query(
        r#"
        INSERT INTO upload_sessions (id, uuid, repository_id, uploaded_size, storage_path, created_at, updated_at, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#
    )
    .bind(Uuid::new_v4())
    .bind(upload_uuid)
    .bind(&repo.id)
    .bind(0i64)
    .bind(&storage_path)
    .bind(chrono::Utc::now())
    .bind(chrono::Utc::now())
    .bind(chrono::Utc::now() + chrono::Duration::hours(24)) // 24 hour expiry
    .execute(&state.database.pool)
    .await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Docker-Upload-UUID",
        upload_uuid.to_string().parse().unwrap()
    );
    headers.insert(
        header::LOCATION,
        format!("/v2/{}/blobs/uploads/{}", name, upload_uuid).parse().unwrap()
    );
    headers.insert(
        "Range",
        "0-0".parse().unwrap()
    );

    Ok((StatusCode::ACCEPTED, headers))
}

/// Complete blob upload
pub async fn complete_blob_upload(
    State(state): State<AppState>,
    Path((name, uuid)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
    request: Request<Body>,
) -> Result<impl IntoResponse> {
    validate_repository_name(&name)?;
    
    let upload_uuid = Uuid::parse_str(&uuid)
        .map_err(|_| Error::bad_request("Invalid upload UUID"))?;
    
    let expected_digest = params.get("digest")
        .ok_or_else(|| Error::bad_request("Missing digest parameter"))?;
    validate_digest(expected_digest)?;

    // Get upload session
    let upload_session = get_upload_session(&state, upload_uuid).await?;
    
    // Read request body
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX).await
        .map_err(|_| Error::bad_request("Failed to read request body"))?;
    
    // Calculate digest
    let calculated_digest = sha256_digest(&body_bytes);
    
    if &calculated_digest != expected_digest {
        return Err(Error::bad_request(format!(
            "Digest mismatch: expected {}, got {}", 
            expected_digest, calculated_digest
        )));
    }
    
    // Store blob
    state.storage.put_blob(expected_digest, &body_bytes).await?;
    
    // Create blob record
    let blob_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO blobs (id, digest, media_type, size, storage_path, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#
    )
    .bind(blob_id)
    .bind(expected_digest)
    .bind("application/octet-stream") // Default media type
    .bind(body_bytes.len() as i64)
    .bind(format!("blobs/{}", expected_digest))
    .bind(chrono::Utc::now())
    .execute(&state.database.pool)
    .await?;
    
    // Link blob to repository
    sqlx::query(
        "INSERT INTO repository_blobs (id, repository_id, blob_id, created_at) VALUES ($1, $2, $3, $4)"
    )
    .bind(Uuid::new_v4())
    .bind(&upload_session.repository_id)
    .bind(blob_id)
    .bind(chrono::Utc::now())
    .execute(&state.database.pool)
    .await?;
    
    // Clean up upload session
    cleanup_upload_session(&state, upload_uuid).await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Docker-Content-Digest",
        expected_digest.parse().unwrap()
    );
    headers.insert(
        header::LOCATION,
        format!("/v2/{}/blobs/{}", name, expected_digest).parse().unwrap()
    );

    Ok((StatusCode::CREATED, headers))
}

/// Upload blob chunk (PATCH)
pub async fn upload_blob_chunk(
    State(_state): State<AppState>,
    Path((name, uuid)): Path<(String, String)>,
) -> Result<StatusCode> {
    // TODO: Implement chunked upload
    tracing::warn!("Chunked upload not yet implemented for {} upload {}", name, uuid);
    Err(Error::registry("Chunked upload not yet implemented".to_string()))
}

/// Get upload status
pub async fn get_upload_status(
    State(state): State<AppState>,
    Path((name, uuid)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    validate_repository_name(&name)?;
    
    let upload_uuid = Uuid::parse_str(&uuid)
        .map_err(|_| Error::bad_request("Invalid upload UUID"))?;
    
    let upload_session = get_upload_session(&state, upload_uuid).await?;
    
    let mut headers = HeaderMap::new();
    headers.insert(
        "Docker-Upload-UUID",
        upload_uuid.to_string().parse().unwrap()
    );
    headers.insert(
        "Range",
        format!("0-{}", upload_session.uploaded_size).parse().unwrap()
    );

    Ok((StatusCode::NO_CONTENT, headers))
}

/// Cancel upload
pub async fn cancel_upload(
    State(state): State<AppState>,
    Path((name, uuid)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    validate_repository_name(&name)?;
    
    let upload_uuid = Uuid::parse_str(&uuid)
        .map_err(|_| Error::bad_request("Invalid upload UUID"))?;
    
    cleanup_upload_session(&state, upload_uuid).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get manifest by reference
pub async fn get_manifest(
    State(_state): State<AppState>,
    Path((name, reference)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    // TODO: Implement manifest retrieval
    tracing::info!("Getting manifest {} for repository {}", reference, name);
    Ok("Manifest get endpoint - not yet implemented")
}

/// Put manifest by reference
pub async fn put_manifest(
    State(_state): State<AppState>,
    Path((name, reference)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    // TODO: Implement manifest storage
    tracing::info!("Putting manifest {} for repository {}", reference, name);
    Ok("Manifest put endpoint - not yet implemented")
}

/// Head manifest by reference
pub async fn head_manifest(
    State(_state): State<AppState>,
    Path((name, reference)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    // TODO: Implement manifest head
    tracing::info!("Head manifest {} for repository {}", reference, name);
    Ok("Manifest head endpoint - not yet implemented")
}

/// Delete manifest by reference
pub async fn delete_manifest(
    State(_state): State<AppState>,
    Path((name, reference)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    // TODO: Implement manifest deletion
    tracing::info!("Deleting manifest {} for repository {}", reference, name);
    Ok("Manifest delete endpoint - not yet implemented")
}

/// List tags for repository
pub async fn list_tags(
    State(_state): State<AppState>,
    Path(name): Path<String>,
    Query(_params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse> {
    // TODO: Implement tag listing
    tracing::info!("Listing tags for repository {}", name);
    Ok(Json(json!({
        "name": name,
        "tags": []
    })))
}
