//! Server configuration

use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server bind address
    pub bind_addr: SocketAddr,

    /// TLS certificate path
    pub cert_path: PathBuf,

    /// TLS private key path
    pub key_path: PathBuf,

    /// Maximum concurrent connections
    pub max_clients: usize,

    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:5000".parse().unwrap(),
            cert_path: PathBuf::from("certs/cert.pem"),
            key_path: PathBuf::from("certs/key.pem"),
            max_clients: 1000,
            heartbeat_interval: 5,
        }
    }
}

impl ServerConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let bind_addr = std::env::var("ISSUN_BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:5000".to_string())
            .parse()?;

        let cert_path = std::env::var("ISSUN_CERT_PATH")
            .unwrap_or_else(|_| "certs/cert.pem".to_string())
            .into();

        let key_path = std::env::var("ISSUN_KEY_PATH")
            .unwrap_or_else(|_| "certs/key.pem".to_string())
            .into();

        let max_clients = std::env::var("ISSUN_MAX_CLIENTS")
            .unwrap_or_else(|_| "1000".to_string())
            .parse()?;

        let heartbeat_interval = std::env::var("ISSUN_HEARTBEAT_INTERVAL")
            .unwrap_or_else(|_| "5".to_string())
            .parse()?;

        Ok(Self {
            bind_addr,
            cert_path,
            key_path,
            max_clients,
            heartbeat_interval,
        })
    }
}
