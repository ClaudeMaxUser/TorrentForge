//! Torrent Engine Binary
//!
//! Standalone torrent engine with IPC server for desktop integration.
//! Can be used independently or integrated with Electron.

use std::path::PathBuf;
use torrent_engine::{config::Config, logging};

#[tokio::main]
async fn main() -> torrent_engine::Result<()> {
    // Initialize structured logging
    logging::init();

    tracing::info!(
        version = torrent_engine::VERSION,
        "Starting Torrent Engine"
    );

    // Load or create configuration
    let config_path = dirs::config_dir()
        .map(|p| p.join("torrent-forge").join("config.json"))
        .unwrap_or_else(|| PathBuf::from("config.json"));

    let config = Config::load(&config_path).unwrap_or_default();
    config.validate()?;

    tracing::info!(
        download_dir = ?config.download_dir,
        max_peers = config.max_peers,
        listen_port = config.listen_port,
        "Configuration loaded"
    );

    // Engine would be initialized here
    // For now, this is a placeholder for M0

    tracing::info!("Torrent Engine ready");

    // Keep running until interrupted
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down");

    Ok(())
}
