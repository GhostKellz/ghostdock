use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub auth: AuthConfig,
    pub registry: RegistryConfig,
    pub web: WebConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub keep_alive: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub max_connections: u32,
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub backend: StorageBackend,
    pub path: PathBuf,
    pub max_upload_size: u64,
    pub enable_deduplication: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Filesystem,
    S3,
    GCS,
    Azure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    pub oauth: OAuthConfig,
    pub enable_anonymous_read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub google: Option<OAuthProvider>,
    pub github: Option<OAuthProvider>,
    pub microsoft: Option<OAuthProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProvider {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub name: String,
    pub title: String,
    pub description: String,
    pub enable_manifest_list: bool,
    pub enable_content_trust: bool,
    pub max_manifest_size: u64,
    pub max_layer_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub port: u16,
    pub enable_ui: bool,
    pub ui_path: PathBuf,
    pub cors_enabled: bool,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub file: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

impl Config {
    /// Load configuration from file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Create default configuration
    pub fn default() -> Self {
        Config {
            server: ServerConfig {
                bind: "127.0.0.1".to_string(),
                port: crate::DEFAULT_REGISTRY_PORT,
                workers: None,
                keep_alive: Some(60),
            },
            database: DatabaseConfig {
                path: PathBuf::from("./ghostdock.db"),
                max_connections: 10,
                connection_timeout: 30,
            },
            storage: StorageConfig {
                backend: StorageBackend::Filesystem,
                path: PathBuf::from("./storage"),
                max_upload_size: 5 * 1024 * 1024 * 1024, // 5GB
                enable_deduplication: true,
            },
            auth: AuthConfig {
                jwt_secret: "your-secret-key-change-this".to_string(),
                jwt_expiration: 86400, // 24 hours
                oauth: OAuthConfig {
                    google: None,
                    github: None,
                    microsoft: None,
                },
                enable_anonymous_read: true,
            },
            registry: RegistryConfig {
                name: "ghostdock".to_string(),
                title: "GhostDock Registry".to_string(),
                description: "A next-generation Docker registry".to_string(),
                enable_manifest_list: true,
                enable_content_trust: false,
                max_manifest_size: 1024 * 1024, // 1MB
                max_layer_size: 10 * 1024 * 1024 * 1024, // 10GB
            },
            web: WebConfig {
                port: crate::DEFAULT_WEB_PORT,
                enable_ui: true,
                ui_path: PathBuf::from("./web/dist"),
                cors_enabled: true,
                cors_origins: vec!["*".to_string()],
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: LogFormat::Pretty,
                file: None,
            },
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default()
    }
}
