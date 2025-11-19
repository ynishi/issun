//! ISSUN QUIC Relay Server
//!
//! Lightweight, stateless relay server for transparent EventBus networking.

mod config;
mod connection;
mod http_server;
mod metrics;
mod relay;
mod room;

use config::ServerConfig;
use metrics::Metrics;
use relay::RelayServer;
use std::sync::Arc;
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

    // Initialize metrics
    let metrics = Arc::new(Metrics::new()?);
    info!("Metrics initialized");

    // Start HTTP server for metrics endpoint
    let metrics_addr = format!("0.0.0.0:{}", config.metrics_port)
        .parse()
        .expect("Invalid metrics address");
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        if let Err(e) = http_server::start_http_server(metrics_addr, metrics_clone).await {
            tracing::error!("HTTP server error: {}", e);
        }
    });

    // Create and run server
    let mut server = RelayServer::new(config, metrics).await?;
    server.run().await?;

    Ok(())
}
