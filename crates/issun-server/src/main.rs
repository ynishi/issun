//! ISSUN QUIC Relay Server
//!
//! Lightweight, stateless relay server for transparent EventBus networking.

mod config;
mod connection;
mod relay;
mod room;

use config::ServerConfig;
use relay::RelayServer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("issun_server=info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting ISSUN relay server...");

    // Load configuration
    let config = ServerConfig::from_env()?;

    info!("Configuration loaded: {:?}", config);

    // Create and run server
    let mut server = RelayServer::new(config).await?;
    server.run().await?;

    Ok(())
}
