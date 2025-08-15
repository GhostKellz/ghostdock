use anyhow::Result;
use clap::Parser;
use ghostdock::{cli::Cli, server::Server};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Parse CLI arguments
    let cli = Cli::parse();
    
    info!("Starting GhostDock Registry v{}", env!("CARGO_PKG_VERSION"));

    // Create and start server
    let server = Server::new(cli.config).await?;
    server.run().await?;

    Ok(())
}
