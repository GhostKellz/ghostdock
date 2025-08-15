use crate::{
    error::{Error, Result},
    server::AppState,
    types::*,
    utils::{validate_repository_name, validate_tag_name, validate_digest, sha256_digest},
    database::queries::*,
};
use axum::{
    extract::{Path, State, Request},
    response::{IntoResponse, Response},
    body::Body,
    http::{StatusCode, HeaderMap, header},
    Json,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

/// Get manifest by tag or digest
pub async fn get_manifest(
    State(state): State<AppState>,
    Path((name, reference)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    validate_repository_name(&name)?;
    
    let repo = get_repository_by_name(&state, &name).await?;
    
    let manifest = if reference.starts_with("sha256:") {
        // It's a digest
        validate_digest(&reference)?;
        get_manifest_by_digest(&state, &repo.id, &reference).await?
    } else {
        // It's a tag
        validate_tag_name(&reference)?;
        get_manifest_by_tag(&state, &repo.id, &reference).await?
    };
    
    // Parse the manifest content
    let manifest_json: Value = serde_json::from_str(&manifest.content)
        .map_err(|_| Error::internal("Invalid manifest JSON"))?;
    
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        manifest.media_type.parse().unwrap_or("application/vnd.docker.distribution.manifest.v2+json".parse().unwrap())
    );
    headers.insert(
        "Docker-Content-Digest",
        manifest.digest.parse().unwrap()
    );
    headers.insert(
        header::CONTENT_LENGTH,
        manifest.content.len().to_string().parse().unwrap()
    );

    Ok((StatusCode::OK, headers, manifest.content))
}

/// Head manifest by tag or digest
pub async fn head_manifest(
    State(state): State<AppState>,
    Path((name, reference)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    validate_repository_name(&name)?;
    
    let repo = get_repository_by_name(&state, &name).await?;
    
    let manifest = if reference.starts_with("sha256:") {
        validate_digest(&reference)?;
        get_manifest_by_digest(&state, &repo.id, &reference).await?
    } else {
        validate_tag_name(&reference)?;
        get_manifest_by_tag(&state, &repo.id, &reference).await?
    };
    
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        manifest.media_type.parse().unwrap_or("application/vnd.docker.distribution.manifest.v2+json".parse().unwrap())
    );
    headers.insert(
        "Docker-Content-Digest",
        manifest.digest.parse().unwrap()
    );
    headers.insert(
        header::CONTENT_LENGTH,
        manifest.content.len().to_string().parse().unwrap()
    );

    Ok((StatusCode::OK, headers))
}

/// Put manifest (upload)
pub async fn put_manifest(
    State(state): State<AppState>,
    Path((name, reference)): Path<(String, String)>,
    request: Request<Body>,
) -> Result<impl IntoResponse> {
    validate_repository_name(&name)?;
    
    // Get or create repository
    let repo = get_or_create_repository(&state, &name).await?;
    
    // Read manifest content
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX).await
        .map_err(|_| Error::bad_request("Failed to read manifest body"))?;
    
    let manifest_content = String::from_utf8(body_bytes.to_vec())
        .map_err(|_| Error::bad_request("Invalid UTF-8 in manifest"))?;
    
    // Calculate digest  
    let calculated_digest = sha256_digest(manifest_content.as_bytes());
    
    // Parse manifest to determine media type
    let manifest_json: Value = serde_json::from_str(&manifest_content)
        .map_err(|_| Error::bad_request("Invalid JSON manifest"))?;
    
    let media_type = manifest_json.get("mediaType")
        .and_then(|v| v.as_str())
        .unwrap_or("application/vnd.docker.distribution.manifest.v2+json")
        .to_string();
    
    // Validate manifest structure
    validate_manifest_structure(&manifest_json)?;
    
    // Store manifest
    let manifest_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO manifests (id, repository_id, digest, media_type, content, size, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (repository_id, digest) DO UPDATE SET
            media_type = EXCLUDED.media_type,
            content = EXCLUDED.content,
            size = EXCLUDED.size
        "#
    )
    .bind(manifest_id)
    .bind(&repo.id)
    .bind(&calculated_digest)
    .bind(&media_type)
    .bind(&manifest_content)
    .bind(manifest_content.len() as i64)
    .bind(chrono::Utc::now())
    .execute(&state.database.pool)
    .await?;
    
    // If reference is a tag (not a digest), create/update the tag
    if !reference.starts_with("sha256:") {
        validate_tag_name(&reference)?;
        
        sqlx::query(
            r#"
            INSERT INTO tags (id, repository_id, name, manifest_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (repository_id, name) DO UPDATE SET
                manifest_id = EXCLUDED.manifest_id,
                updated_at = EXCLUDED.updated_at
            "#
        )
        .bind(Uuid::new_v4())
        .bind(&repo.id)
        .bind(&reference)
        .bind(manifest_id)
        .bind(chrono::Utc::now())
        .bind(chrono::Utc::now())
        .execute(&state.database.pool)
        .await?;
    }
    
    // Create blob relationships if this is an image manifest
    if let Some(config) = manifest_json.get("config") {
        if let Some(digest) = config.get("digest").and_then(|d| d.as_str()) {
            link_manifest_to_blob(&state, manifest_id, digest).await?;
        }
    }
    
    if let Some(layers) = manifest_json.get("layers").and_then(|l| l.as_array()) {
        for layer in layers {
            if let Some(digest) = layer.get("digest").and_then(|d| d.as_str()) {
                link_manifest_to_blob(&state, manifest_id, digest).await?;
            }
        }
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        "Docker-Content-Digest",
        calculated_digest.parse().unwrap()
    );
    headers.insert(
        header::LOCATION,
        format!("/v2/{}/manifests/{}", name, calculated_digest).parse().unwrap()
    );

    Ok((StatusCode::CREATED, headers))
}

/// Delete manifest
pub async fn delete_manifest(
    State(state): State<AppState>,
    Path((name, reference)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    validate_repository_name(&name)?;
    
    let repo = get_repository_by_name(&state, &name).await?;
    
    if reference.starts_with("sha256:") {
        // Delete by digest
        validate_digest(&reference)?;
        delete_manifest_by_digest(&state, &repo.id, &reference).await?;
    } else {
        // Delete by tag
        validate_tag_name(&reference)?;
        delete_tag(&state, &repo.id, &reference).await?;
    }

    Ok(StatusCode::ACCEPTED)
}

/// Get repository tags
pub async fn get_tags(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse> {
    validate_repository_name(&name)?;
    
    let repo = get_repository_by_name(&state, &name).await?;
    
    let tags: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM tags WHERE repository_id = $1 ORDER BY created_at DESC"
    )
    .bind(&repo.id)
    .fetch_all(&state.database.pool)
    .await?;

    Ok(Json(json!({
        "name": name,
        "tags": tags
    })))
}

/// Validate manifest structure
fn validate_manifest_structure(manifest: &Value) -> Result<()> {
    // Check for required fields based on manifest type
    let media_type = manifest.get("mediaType")
        .and_then(|v| v.as_str())
        .unwrap_or("application/vnd.docker.distribution.manifest.v2+json");
    
    match media_type {
        "application/vnd.docker.distribution.manifest.v2+json" => {
            // Docker Image Manifest v2
            if !manifest.get("config").is_some() {
                return Err(Error::bad_request("Missing config in image manifest"));
            }
            if !manifest.get("layers").and_then(|l| l.as_array()).is_some() {
                return Err(Error::bad_request("Missing or invalid layers in image manifest"));
            }
        }
        "application/vnd.docker.distribution.manifest.list.v2+json" => {
            // Manifest List (multi-arch)
            if !manifest.get("manifests").and_then(|m| m.as_array()).is_some() {
                return Err(Error::bad_request("Missing or invalid manifests in manifest list"));
            }
        }
        _ => {
            // Allow other types but log a warning
            tracing::warn!("Unknown manifest media type: {}", media_type);
        }
    }
    
    Ok(())
}

/// Link manifest to blob
async fn link_manifest_to_blob(
    state: &AppState,
    manifest_id: Uuid,
    blob_digest: &str,
) -> Result<()> {
    // Find the blob by digest
    let blob_result: std::result::Result<Uuid, sqlx::Error> = sqlx::query_scalar(
        "SELECT id FROM blobs WHERE digest = $1"
    )
    .bind(blob_digest)
    .fetch_one(&state.database.pool)
    .await;
    
    if let Ok(blob_id) = blob_result {
        // Create the link
        sqlx::query(
            "INSERT INTO manifest_blobs (id, manifest_id, blob_id, created_at) VALUES ($1, $2, $3, $4)"
        )
        .bind(Uuid::new_v4())
        .bind(manifest_id)
        .bind(blob_id)
        .bind(chrono::Utc::now())
        .execute(&state.database.pool)
        .await?;
    } else {
        tracing::warn!("Referenced blob {} not found when linking to manifest", blob_digest);
    }
    
    Ok(())
}
