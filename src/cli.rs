use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ghostdock")]
#[command(about = "A next-generation Docker registry with advanced management capabilities")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    pub config: PathBuf,
    
    /// Verbosity level (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    
    /// Registry bind address
    #[arg(long, default_value = "127.0.0.1")]
    pub bind: String,
    
    /// Registry port
    #[arg(long, default_value_t = crate::DEFAULT_REGISTRY_PORT)]
    pub port: u16,
    
    /// Web UI port
    #[arg(long, default_value_t = crate::DEFAULT_WEB_PORT)]
    pub web_port: u16,
    
    /// Storage directory
    #[arg(long, default_value = "./storage")]
    pub storage_dir: PathBuf,
    
    /// Database path
    #[arg(long, default_value = "./ghostdock.db")]
    pub database_path: PathBuf,
    
    /// Enable development mode (with additional logging and debug features)
    #[arg(long)]
    pub dev: bool,
}
