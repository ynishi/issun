//! Core relay server logic

use crate::config::ServerConfig;
use crate::connection::ClientConnection;
use crate::metrics::SharedMetrics;
use crate::room::RoomManager;
use anyhow::Result;
use issun::network::{backend::RawNetworkEvent, NodeId};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Relay server that routes events between clients
pub struct RelayServer {
    /// QUIC endpoint
    endpoint: quinn::Endpoint,

    /// Active client connections
    clients: Arc<RwLock<HashMap<NodeId, ClientConnection>>>,

    /// Room manager for lobby system
    room_manager: Arc<RoomManager>,

    /// Metrics collector
    metrics: SharedMetrics,

    /// Configuration
    config: ServerConfig,
}

impl RelayServer {
    /// Create a new relay server
    pub async fn new(config: ServerConfig, metrics: SharedMetrics) -> Result<Self> {
        info!("Initializing relay server on {}", config.bind_addr);

        // Load TLS certificates
        let (cert, key) = Self::load_certificates(&config)?;

        // Configure QUIC server
        let mut server_config = quinn::ServerConfig::with_single_cert(cert, key)?;

        // Set transport parameters
        let mut transport_config = quinn::TransportConfig::default();
        transport_config.max_concurrent_uni_streams(100_u32.into());
        transport_config
            .max_idle_timeout(Some(std::time::Duration::from_secs(60).try_into().unwrap()));

        server_config.transport_config(Arc::new(transport_config));

        // Bind endpoint
        let endpoint = quinn::Endpoint::server(server_config, config.bind_addr)?;

        info!("Relay server listening on {}", config.bind_addr);

        Ok(Self {
            endpoint,
            clients: Arc::new(RwLock::new(HashMap::new())),
            room_manager: Arc::new(RoomManager::new()),
            metrics,
            config,
        })
    }

    /// Run the relay server
    pub async fn run(&mut self) -> Result<()> {
        info!("Relay server started, waiting for connections...");

        loop {
            // Accept incoming connections
            if let Some(connecting) = self.endpoint.accept().await {
                let clients = self.clients.clone();
                let room_manager = self.room_manager.clone();
                let metrics = self.metrics.clone();
                let config = self.config.clone();

                tokio::spawn(async move {
                    match connecting.await {
                        Ok(connection) => {
                            let remote_addr = connection.remote_address();
                            info!("New connection established from {}", remote_addr);

                            if let Err(e) = Self::handle_connection(
                                connection,
                                clients,
                                room_manager,
                                metrics,
                                config,
                            )
                            .await
                            {
                                error!("Connection handler error: {}", e);
                            }
                        }
                        Err(e) => {
                            warn!("Connection failed: {}", e);
                        }
                    }
                });
            }
        }
    }

    /// Handle a single client connection
    async fn handle_connection(
        connection: quinn::Connection,
        clients: Arc<RwLock<HashMap<NodeId, ClientConnection>>>,
        room_manager: Arc<RoomManager>,
        metrics: SharedMetrics,
        _config: ServerConfig,
    ) -> Result<()> {
        let connection_start = Instant::now();
        // Perform handshake to get NodeId
        let node_id = Self::handshake(&connection).await?;

        info!("Client handshake completed: {:?}", node_id);

        // Create client connection
        let mut client = ClientConnection::new(node_id, connection.clone());

        // Add to clients map
        {
            let mut clients_guard = clients.write().await;
            if clients_guard.len() >= 1000 {
                anyhow::bail!("Max clients reached");
            }
            clients_guard.insert(node_id, ClientConnection::new(node_id, connection));
        }

        // Update metrics
        metrics.increment_connected_clients();
        let total_clients = clients.read().await.len();
        info!(
            "Client connected: {:?}, total clients: {}",
            node_id, total_clients
        );

        // Event loop: receive events and relay to other clients
        loop {
            match client.receive_events().await {
                Ok(events) if !events.is_empty() => {
                    for event in events {
                        Self::relay_event(node_id, event, &clients, &room_manager, &metrics).await;
                    }
                }
                Ok(_) => {
                    // No events, continue
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
                Err(e) => {
                    warn!("Client read error from {:?}: {}", node_id, e);
                    break;
                }
            }

            // Check if client is still alive
            if !client.is_alive() {
                warn!("Client timeout: {:?}", node_id);
                break;
            }
        }

        // Remove client and cleanup room
        room_manager.handle_disconnect(node_id).await;
        clients.write().await.remove(&node_id);

        // Update metrics
        metrics.decrement_connected_clients();
        let connection_duration = connection_start.elapsed().as_secs_f64();
        metrics.record_connection_duration("completed", connection_duration);

        let total_clients = clients.read().await.len();
        info!(
            "Client disconnected: {:?}, total clients: {}, duration: {:.2}s",
            node_id, total_clients, connection_duration
        );

        Ok(())
    }

    /// Perform handshake with client
    async fn handshake(connection: &quinn::Connection) -> Result<NodeId> {
        // Accept handshake stream
        let mut recv_stream = connection.accept_uni().await?;

        // Read NodeId (8 bytes)
        let mut buf = [0u8; 8];
        recv_stream.read_exact(&mut buf).await?;
        let node_id = NodeId::from_u64(u64::from_le_bytes(buf));

        // Send ack
        let mut send_stream = connection.open_uni().await?;
        send_stream
            .write_all(&node_id.as_u64().to_le_bytes())
            .await?;
        send_stream.finish()?;

        Ok(node_id)
    }

    /// Relay an event to all other clients (room-aware)
    async fn relay_event(
        from: NodeId,
        event: RawNetworkEvent,
        clients: &Arc<RwLock<HashMap<NodeId, ClientConnection>>>,
        room_manager: &Arc<RoomManager>,
        metrics: &SharedMetrics,
    ) {
        let relay_start = Instant::now();
        let clients_guard = clients.read().await;

        let target_clients: Vec<_> = match event.scope {
            issun::network::NetworkScope::Broadcast => {
                // Check if sender is in a room
                if let Some(_room_id) = room_manager.get_client_room(from).await {
                    // Room-scoped broadcast: send to all clients in the same room except sender
                    debug!("Room-scoped broadcast from {:?}", from);
                    room_manager
                        .get_room_clients(from)
                        .await
                        .into_iter()
                        .filter(|id| *id != from)
                        .collect()
                } else {
                    // Global broadcast: send to all clients except sender (not in any room)
                    debug!("Global broadcast from {:?}", from);
                    clients_guard
                        .iter()
                        .filter(|(id, _)| {
                            **id != from
                                && tokio::task::block_in_place(|| {
                                    tokio::runtime::Handle::current().block_on(async {
                                        room_manager.get_client_room(**id).await.is_none()
                                    })
                                })
                        })
                        .map(|(id, _)| *id)
                        .collect()
                }
            }
            issun::network::NetworkScope::Targeted(target) => {
                // Send to specific client
                if clients_guard.contains_key(&target) {
                    vec![target]
                } else {
                    vec![]
                }
            }
            issun::network::NetworkScope::ToServer => {
                // Server-only events not relayed
                vec![]
            }
        };

        drop(clients_guard);

        // Record metrics
        let scope_str = match event.scope {
            issun::network::NetworkScope::Broadcast => "broadcast",
            issun::network::NetworkScope::Targeted(_) => "targeted",
            issun::network::NetworkScope::ToServer => "to_server",
        };
        metrics.record_event_relayed(scope_str);

        // Send to target clients
        for target_id in target_clients {
            let clients_guard = clients.read().await;
            if let Some(client) = clients_guard.get(&target_id) {
                if let Err(e) = client.send_event(&event).await {
                    warn!(
                        "Failed to relay event from {:?} to {:?}: {}",
                        from, target_id, e
                    );
                }
            }
        }

        // Record relay latency
        let relay_duration_micros = relay_start.elapsed().as_micros() as f64;
        metrics.record_relay_latency(scope_str, relay_duration_micros);
    }

    /// Load TLS certificates from files
    fn load_certificates(
        config: &ServerConfig,
    ) -> Result<(
        Vec<rustls::pki_types::CertificateDer<'static>>,
        rustls::pki_types::PrivateKeyDer<'static>,
    )> {
        let cert_data = std::fs::read(&config.cert_path)?;
        let key_data = std::fs::read(&config.key_path)?;

        let cert_chain =
            rustls_pemfile::certs(&mut cert_data.as_slice()).collect::<Result<Vec<_>, _>>()?;

        let key = rustls_pemfile::pkcs8_private_keys(&mut key_data.as_slice())
            .next()
            .ok_or_else(|| anyhow::anyhow!("No private key in file"))??;

        Ok((cert_chain, key.into()))
    }
}
