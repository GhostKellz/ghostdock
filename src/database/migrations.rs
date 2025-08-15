// Database migration utilities will go here
// For now, we'll implement basic table creation

use crate::error::Result;
use sqlx::SqlitePool;

pub async fn create_tables(pool: &SqlitePool) -> Result<()> {
    // Users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT,
            full_name TEXT,
            avatar_url TEXT,
            provider TEXT,
            provider_id TEXT,
            is_admin BOOLEAN NOT NULL DEFAULT FALSE,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_login DATETIME
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Repositories table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS repositories (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            namespace TEXT,
            description TEXT,
            is_public BOOLEAN NOT NULL DEFAULT FALSE,
            owner_id TEXT NOT NULL,
            star_count INTEGER NOT NULL DEFAULT 0,
            pull_count INTEGER NOT NULL DEFAULT 0,
            push_count INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (owner_id) REFERENCES users (id),
            UNIQUE(namespace, name)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Manifests table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS manifests (
            id TEXT PRIMARY KEY,
            repository_id TEXT NOT NULL,
            digest TEXT UNIQUE NOT NULL,
            media_type TEXT NOT NULL,
            schema_version INTEGER NOT NULL,
            content BLOB NOT NULL,
            size INTEGER NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (repository_id) REFERENCES repositories (id)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Tags table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            repository_id TEXT NOT NULL,
            name TEXT NOT NULL,
            manifest_id TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_pulled DATETIME,
            FOREIGN KEY (repository_id) REFERENCES repositories (id),
            FOREIGN KEY (manifest_id) REFERENCES manifests (id),
            UNIQUE(repository_id, name)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Blobs table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS blobs (
            id TEXT PRIMARY KEY,
            digest TEXT UNIQUE NOT NULL,
            media_type TEXT NOT NULL,
            size INTEGER NOT NULL,
            content_type TEXT,
            storage_path TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_accessed DATETIME
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Repository-blob relationship table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS repository_blobs (
            id TEXT PRIMARY KEY,
            repository_id TEXT NOT NULL,
            blob_id TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (repository_id) REFERENCES repositories (id),
            FOREIGN KEY (blob_id) REFERENCES blobs (id),
            UNIQUE(repository_id, blob_id)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Upload sessions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS upload_sessions (
            id TEXT PRIMARY KEY,
            uuid TEXT UNIQUE NOT NULL,
            repository_id TEXT NOT NULL,
            uploaded_size INTEGER NOT NULL DEFAULT 0,
            total_size INTEGER,
            digest TEXT,
            storage_path TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            expires_at DATETIME NOT NULL,
            FOREIGN KEY (repository_id) REFERENCES repositories (id)
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
