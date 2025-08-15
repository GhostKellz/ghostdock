use crate::{config::DatabaseConfig, error::Result};
use sqlx::{SqlitePool, Pool, Sqlite};

pub mod migrations;
pub mod queries;

pub struct Database {
    pub pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = config.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        let pool = SqlitePool::connect(&format!("sqlite:{}", config.path.display())).await?;
        
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        // For now, use our basic table creation
        // Later we can switch to proper migrations
        migrations::create_tables(&self.pool).await?;
        Ok(())
    }

    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1").fetch_optional(&self.pool).await?;
        Ok(())
    }
}
