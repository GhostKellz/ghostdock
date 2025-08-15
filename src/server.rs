use crate::{
    config::Config,
    database::Database,
    error::Result,
    handlers::{auth, health, registry, manifest},
    storage::Storage,
    web,
};
use axum::{
    routing::{get, post, put, delete, head, patch},
    Router,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::{info, warn};

pub struct Server {
    config: Config,
    database: Arc<Database>,
    storage: Arc<Storage>,
}

impl Server {
    pub async fn new(config_path: PathBuf) -> Result<Self> {
        // Load configuration
        let config = if config_path.exists() {
            Config::load(&config_path)?
        } else {
            warn!("Config file not found, using default configuration");
            Config::default()
        };

        // Initialize database
        let database = Arc::new(Database::new(&config.database).await?);
        database.migrate().await?;

        // Initialize storage
        let storage = Arc::new(Storage::new(&config.storage).await?);

        Ok(Self {
            config,
            database,
            storage,
        })
    }

    pub async fn run(self) -> Result<()> {
        let registry_app = self.registry_router().await?;
        let web_app = self.web_router().await?;

        // Registry server
        let registry_addr: SocketAddr = format!("{}:{}", self.config.server.bind, self.config.server.port)
            .parse()
            .expect("Invalid server address");

        // Web server
        let web_addr: SocketAddr = format!("{}:{}", self.config.server.bind, self.config.web.port)
            .parse()
            .expect("Invalid web server address");

        info!("Starting GhostDock Registry on {}", registry_addr);
        info!("Starting GhostDock Web UI on {}", web_addr);

        // Start both servers concurrently
        let registry_listener = tokio::net::TcpListener::bind(&registry_addr).await?;
        let web_listener = tokio::net::TcpListener::bind(&web_addr).await?;

        let registry_server = axum::serve(registry_listener, registry_app);
        let web_server = axum::serve(web_listener, web_app);

        tokio::select! {
            result = registry_server => {
                if let Err(err) = result {
                    tracing::error!("Registry server error: {}", err);
                }
            }
            result = web_server => {
                if let Err(err) = result {
                    tracing::error!("Web server error: {}", err);
                }
            }
            _ = signal::ctrl_c() => {
                info!("Shutdown signal received");
            }
        }

        info!("GhostDock shutting down");
        Ok(())
    }

    async fn registry_router(&self) -> Result<Router> {
        let state = AppState {
            config: self.config.clone(),
            database: Arc::clone(&self.database),
            storage: Arc::clone(&self.storage),
        };

        let app = Router::new()
            // Docker Registry v2 API
            .route("/v2/", get(registry::root))
            .route("/v2/:name/blobs/:digest", get(registry::get_blob))
            .route("/v2/:name/blobs/:digest", head(registry::head_blob))
            .route("/v2/:name/blobs/:digest", delete(registry::delete_blob))
            .route("/v2/:name/blobs/uploads/", post(registry::initiate_blob_upload))
            .route("/v2/:name/blobs/uploads/:uuid", put(registry::complete_blob_upload))
            .route("/v2/:name/blobs/uploads/:uuid", patch(registry::upload_blob_chunk))
            .route("/v2/:name/blobs/uploads/:uuid", get(registry::get_upload_status))
            .route("/v2/:name/blobs/uploads/:uuid", delete(registry::cancel_upload))
            .route("/v2/:name/manifests/:reference", get(manifest::get_manifest))
            .route("/v2/:name/manifests/:reference", put(manifest::put_manifest))
            .route("/v2/:name/manifests/:reference", head(manifest::head_manifest))
            .route("/v2/:name/manifests/:reference", delete(manifest::delete_manifest))
            .route("/v2/:name/tags/list", get(manifest::get_tags))
            
            // Health check
            .route("/health", get(health::health_check))
            .route("/metrics", get(health::metrics))
            
            // Authentication
            .route("/auth/login", post(auth::login))
            .route("/auth/logout", post(auth::logout))
            .route("/auth/oauth/:provider", get(auth::oauth_redirect))
            .route("/auth/oauth/:provider/callback", get(auth::oauth_callback))
            
            // Middleware
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::permissive())
            .with_state(state);

        Ok(app)
    }

    async fn web_router(&self) -> Result<Router> {
        if !self.config.web.enable_ui {
            return Ok(Router::new()
                .route("/", get(|| async { "Web UI disabled" }))
            );
        }

        let app = Router::new()
            .merge(web::routes())
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::permissive());

        Ok(app)
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub database: Arc<Database>,
    pub storage: Arc<Storage>,
}
