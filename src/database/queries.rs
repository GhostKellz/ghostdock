use crate::{
    error::{Error, Result},
    server::AppState,
    types::*,
};
use uuid::Uuid;
use sqlx::Row;

/// Get repository by name
pub async fn get_repository_by_name(state: &AppState, name: &str) -> Result<Repository> {
    let row = sqlx::query(
        "SELECT id, name, description, is_public, created_at, updated_at FROM repositories WHERE name = $1"
    )
    .bind(name)
    .fetch_one(&state.database.pool)
    .await
    .map_err(|_| Error::not_found(format!("Repository '{}' not found", name)))?;
    
    Ok(Repository {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        is_public: row.get("is_public"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

/// Get or create repository
pub async fn get_or_create_repository(state: &AppState, name: &str) -> Result<Repository> {
    // Try to get existing repository first
    match get_repository_by_name(state, name).await {
        Ok(repo) => Ok(repo),
        Err(_) => {
            // Create new repository
            let repo_id = Uuid::new_v4();
            let now = chrono::Utc::now();
            
            sqlx::query(
                "INSERT INTO repositories (id, name, description, is_public, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)"
            )
            .bind(repo_id)
            .bind(name)
            .bind("")
            .bind(false)
            .bind(now)
            .bind(now)
            .execute(&state.database.pool)
            .await?;
            
            Ok(Repository {
                id: repo_id,
                name: name.to_string(),
                description: String::new(),
                is_public: false,
                created_at: now,
                updated_at: now,
            })
        }
    }
}

/// Get blob by digest
pub async fn get_blob_by_digest(state: &AppState, repository_id: &Uuid, digest: &str) -> Result<Blob> {
    let row = sqlx::query(
        r#"
        SELECT b.id, b.digest, b.media_type, b.size, b.storage_path, b.created_at, b.last_accessed
        FROM blobs b
        JOIN repository_blobs rb ON b.id = rb.blob_id
        WHERE rb.repository_id = $1 AND b.digest = $2
        "#
    )
    .bind(repository_id)
    .bind(digest)
    .fetch_one(&state.database.pool)
    .await
    .map_err(|_| Error::not_found(format!("Blob '{}' not found", digest)))?;
    
    Ok(Blob {
        id: row.get("id"),
        digest: row.get("digest"),
        media_type: row.get("media_type"),
        size: row.get("size"),
        storage_path: row.get("storage_path"),
        created_at: row.get("created_at"),
        last_accessed: row.get("last_accessed"),
    })
}

/// Update blob access time
pub async fn update_blob_access_time(state: &AppState, blob_id: &Uuid) -> Result<()> {
    sqlx::query("UPDATE blobs SET last_accessed = $1 WHERE id = $2")
        .bind(chrono::Utc::now())
        .bind(blob_id)
        .execute(&state.database.pool)
        .await?;
    
    Ok(())
}

/// Get manifest by digest
pub async fn get_manifest_by_digest(state: &AppState, repository_id: &Uuid, digest: &str) -> Result<Manifest> {
    let row = sqlx::query(
        "SELECT id, repository_id, digest, media_type, content, size, created_at FROM manifests WHERE repository_id = $1 AND digest = $2"
    )
    .bind(repository_id)
    .bind(digest)
    .fetch_one(&state.database.pool)
    .await
    .map_err(|_| Error::not_found(format!("Manifest '{}' not found", digest)))?;
    
    Ok(Manifest {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        digest: row.get("digest"),
        media_type: row.get("media_type"),
        content: row.get("content"),
        size: row.get("size"),
        created_at: row.get("created_at"),
    })
}

/// Get manifest by tag
pub async fn get_manifest_by_tag(state: &AppState, repository_id: &Uuid, tag: &str) -> Result<Manifest> {
    let row = sqlx::query(
        r#"
        SELECT m.id, m.repository_id, m.digest, m.media_type, m.content, m.size, m.created_at
        FROM manifests m
        JOIN tags t ON m.id = t.manifest_id
        WHERE t.repository_id = $1 AND t.name = $2
        "#
    )
    .bind(repository_id)
    .bind(tag)
    .fetch_one(&state.database.pool)
    .await
    .map_err(|_| Error::not_found(format!("Tag '{}' not found", tag)))?;
    
    Ok(Manifest {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        digest: row.get("digest"),
        media_type: row.get("media_type"),
        content: row.get("content"),
        size: row.get("size"),
        created_at: row.get("created_at"),
    })
}

/// Delete manifest by digest
pub async fn delete_manifest_by_digest(state: &AppState, repository_id: &Uuid, digest: &str) -> Result<()> {
    // First delete associated tags
    sqlx::query(
        r#"
        DELETE FROM tags 
        WHERE repository_id = $1 AND manifest_id IN (
            SELECT id FROM manifests WHERE repository_id = $1 AND digest = $2
        )
        "#
    )
    .bind(repository_id)
    .bind(digest)
    .execute(&state.database.pool)
    .await?;
    
    // Then delete manifest
    let result = sqlx::query(
        "DELETE FROM manifests WHERE repository_id = $1 AND digest = $2"
    )
    .bind(repository_id)
    .bind(digest)
    .execute(&state.database.pool)
    .await?;
    
    if result.rows_affected() == 0 {
        return Err(Error::not_found(format!("Manifest '{}' not found", digest)));
    }
    
    Ok(())
}

/// Delete tag
pub async fn delete_tag(state: &AppState, repository_id: &Uuid, tag_name: &str) -> Result<()> {
    let result = sqlx::query(
        "DELETE FROM tags WHERE repository_id = $1 AND name = $2"
    )
    .bind(repository_id)
    .bind(tag_name)
    .execute(&state.database.pool)
    .await?;
    
    if result.rows_affected() == 0 {
        return Err(Error::not_found(format!("Tag '{}' not found", tag_name)));
    }
    
    Ok(())
}

/// Get upload session
pub async fn get_upload_session(state: &AppState, uuid: Uuid) -> Result<UploadSession> {
    let row = sqlx::query(
        r#"
        SELECT id, uuid, repository_id, uploaded_size, storage_path, created_at, updated_at, expires_at
        FROM upload_sessions 
        WHERE uuid = $1 AND expires_at > $2
        "#
    )
    .bind(uuid)
    .bind(chrono::Utc::now())
    .fetch_one(&state.database.pool)
    .await
    .map_err(|_| Error::not_found("Upload session not found or expired"))?;
    
    Ok(UploadSession {
        id: row.get("id"),
        uuid: row.get("uuid"),
        repository_id: row.get("repository_id"),
        uploaded_size: row.get("uploaded_size"),
        storage_path: row.get("storage_path"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        expires_at: row.get("expires_at"),
    })
}

/// Cleanup upload session
pub async fn cleanup_upload_session(state: &AppState, uuid: Uuid) -> Result<()> {
    // TODO: Also cleanup any temporary files in storage
    sqlx::query("DELETE FROM upload_sessions WHERE uuid = $1")
        .bind(uuid)
        .execute(&state.database.pool)
        .await?;
    
    Ok(())
}
