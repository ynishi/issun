//! NetworkBackend trait and implementations

use super::types::{NetworkMetadata, NetworkScope, NodeId};
use crate::error::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Type-erased network event (for receiving)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RawNetworkEvent {
    pub metadata: NetworkMetadata,
    pub scope: NetworkScope,
    pub type_name: String,
    pub payload: Vec<u8>, // bincode serialized
}

/// Network backend trait for event transmission
#[async_trait]
pub trait NetworkBackend: Send + Sync + 'static {
    /// Get this node's ID
    fn node_id(&self) -> NodeId;

    /// Send event to network
    async fn send(&self, event: RawNetworkEvent) -> Result<()>;

    /// Receive stream of raw events
    fn receive_stream(&self) -> mpsc::Receiver<RawNetworkEvent>;

    /// Connect to network
    async fn connect(&mut self, addr: &str) -> Result<()>;

    /// Disconnect from network
    async fn disconnect(&mut self) -> Result<()>;

    /// Check if connected
    fn is_connected(&self) -> bool;
}

/// Local-only backend (no network)
pub struct LocalOnlyBackend {
    node_id: NodeId,
}

impl LocalOnlyBackend {
    pub fn new() -> Self {
        Self {
            node_id: NodeId::from_u64(0),
        }
    }
}

impl Default for LocalOnlyBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NetworkBackend for LocalOnlyBackend {
    fn node_id(&self) -> NodeId {
        self.node_id
    }

    async fn send(&self, _event: RawNetworkEvent) -> Result<()> {
        // No-op for local-only
        Ok(())
    }

    fn receive_stream(&self) -> mpsc::Receiver<RawNetworkEvent> {
        // Return empty channel
        let (_tx, rx) = mpsc::channel(1);
        rx
    }

    async fn connect(&mut self, _addr: &str) -> Result<()> {
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        Ok(())
    }

    fn is_connected(&self) -> bool {
        false
    }
}

/// QUIC client backend for connecting to relay server
pub struct QuicClientBackend {
    node_id: NodeId,
    connection: Option<quinn::Connection>,
    send_tx: mpsc::Sender<RawNetworkEvent>,
    recv_rx: std::sync::Arc<std::sync::Mutex<Option<mpsc::Receiver<RawNetworkEvent>>>>,
    connected: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl QuicClientBackend {
    /// Create a new QUIC client backend
    pub fn new() -> Self {
        let (send_tx, _send_rx) = mpsc::channel(1000);

        Self {
            node_id: NodeId::random(),
            connection: None,
            send_tx,
            recv_rx: std::sync::Arc::new(std::sync::Mutex::new(None)),
            connected: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Connect to relay server
    pub async fn connect_to_server(addr: &str) -> Result<Self> {
        use std::sync::Arc;

        // Install default crypto provider if not already installed
        let _ = rustls::crypto::ring::default_provider().install_default();

        // Generate random node ID
        let node_id = NodeId::random();

        // Configure QUIC client
        // For development, accept invalid certificates
        let crypto = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
            .with_no_client_auth();

        let client_config = quinn::ClientConfig::new(Arc::new(
            quinn::crypto::rustls::QuicClientConfig::try_from(crypto).map_err(|e| {
                crate::error::IssunError::NetworkError(format!("TLS config error: {}", e))
            })?,
        ));

        let mut endpoint = quinn::Endpoint::client("[::]:0".parse().unwrap()).map_err(|e| {
            crate::error::IssunError::NetworkError(format!("Failed to create endpoint: {}", e))
        })?;

        endpoint.set_default_client_config(client_config);

        // Connect to server
        let connection = endpoint
            .connect(addr.parse().unwrap(), "localhost")
            .map_err(|e| {
                crate::error::IssunError::NetworkError(format!("Connection failed: {}", e))
            })?
            .await
            .map_err(|e| {
                crate::error::IssunError::NetworkError(format!("Connection failed: {}", e))
            })?;

        // Perform handshake
        let mut send_stream = connection.open_uni().await.map_err(|e| {
            crate::error::IssunError::NetworkError(format!("Failed to open stream: {}", e))
        })?;

        send_stream
            .write_all(&node_id.as_u64().to_le_bytes())
            .await
            .map_err(|e| {
                crate::error::IssunError::NetworkError(format!("Handshake failed: {}", e))
            })?;

        send_stream.finish().map_err(|e| {
            crate::error::IssunError::NetworkError(format!("Handshake failed: {}", e))
        })?;

        // Receive handshake ack
        let mut recv_stream = connection.accept_uni().await.map_err(|e| {
            crate::error::IssunError::NetworkError(format!("Handshake failed: {}", e))
        })?;

        let mut buf = [0u8; 8];
        recv_stream.read_exact(&mut buf).await.map_err(|e| {
            crate::error::IssunError::NetworkError(format!("Handshake failed: {}", e))
        })?;

        // Create channels
        let (send_tx, mut send_rx) = mpsc::channel::<RawNetworkEvent>(1000);
        let (recv_tx, recv_rx) = mpsc::channel::<RawNetworkEvent>(1000);

        let connected = Arc::new(std::sync::atomic::AtomicBool::new(true));

        // Spawn send worker
        let conn_send = connection.clone();
        let connected_send = connected.clone();
        tokio::spawn(async move {
            while let Some(event) = send_rx.recv().await {
                if !connected_send.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }

                if let Err(e) = Self::send_event(&conn_send, &event).await {
                    eprintln!("Failed to send event: {:?}", e);
                }
            }
        });

        // Spawn receive worker
        let conn_recv = connection.clone();
        let connected_recv = connected.clone();
        tokio::spawn(async move {
            loop {
                if !connected_recv.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }

                match Self::receive_event(&conn_recv).await {
                    Ok(Some(event)) => {
                        let _ = recv_tx.send(event).await;
                    }
                    Ok(None) => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    }
                    Err(e) => {
                        eprintln!("Failed to receive event: {:?}", e);
                        break;
                    }
                }
            }
        });

        Ok(Self {
            node_id,
            connection: Some(connection),
            send_tx,
            recv_rx: Arc::new(std::sync::Mutex::new(Some(recv_rx))),
            connected,
        })
    }

    async fn send_event(connection: &quinn::Connection, event: &RawNetworkEvent) -> Result<()> {
        let mut stream = connection.open_uni().await.map_err(|e| {
            crate::error::IssunError::NetworkError(format!("Failed to open stream: {}", e))
        })?;

        // Create frame
        let payload = bincode::serialize(event).map_err(|e| {
            crate::error::IssunError::NetworkError(format!("Serialization failed: {}", e))
        })?;

        let mut frame = Vec::with_capacity(8 + payload.len());
        frame.extend_from_slice(&[0x49, 0x53]); // Magic: "IS"
        frame.push(0x01); // Version
        frame.push(0x01); // Frame type: Event
        frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        frame.extend_from_slice(&payload);

        stream
            .write_all(&frame)
            .await
            .map_err(|e| crate::error::IssunError::NetworkError(format!("Write failed: {}", e)))?;

        stream
            .finish()
            .map_err(|e| crate::error::IssunError::NetworkError(format!("Finish failed: {}", e)))?;

        Ok(())
    }

    async fn receive_event(connection: &quinn::Connection) -> Result<Option<RawNetworkEvent>> {
        // Try to accept incoming stream with timeout
        let recv_result = tokio::time::timeout(
            tokio::time::Duration::from_millis(10),
            connection.accept_uni(),
        )
        .await;

        let mut stream = match recv_result {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                return Err(crate::error::IssunError::NetworkError(format!(
                    "Accept failed: {}",
                    e
                )));
            }
            Err(_) => return Ok(None), // Timeout
        };

        // Read frame header
        let mut header = [0u8; 8];
        stream.read_exact(&mut header).await.map_err(|e| {
            crate::error::IssunError::NetworkError(format!("Read header failed: {}", e))
        })?;

        // Parse header
        if header[0] != 0x49 || header[1] != 0x53 {
            return Err(crate::error::IssunError::NetworkError(
                "Invalid magic bytes".to_string(),
            ));
        }

        let payload_len = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;

        // Read payload
        let mut payload = vec![0u8; payload_len];
        stream.read_exact(&mut payload).await.map_err(|e| {
            crate::error::IssunError::NetworkError(format!("Read payload failed: {}", e))
        })?;

        // Deserialize event
        let event: RawNetworkEvent = bincode::deserialize(&payload).map_err(|e| {
            crate::error::IssunError::NetworkError(format!("Deserialization failed: {}", e))
        })?;

        Ok(Some(event))
    }
}

impl Default for QuicClientBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NetworkBackend for QuicClientBackend {
    fn node_id(&self) -> NodeId {
        self.node_id
    }

    async fn send(&self, event: RawNetworkEvent) -> Result<()> {
        self.send_tx
            .send(event)
            .await
            .map_err(|e| crate::error::IssunError::NetworkError(format!("Send failed: {}", e)))
    }

    fn receive_stream(&self) -> mpsc::Receiver<RawNetworkEvent> {
        // Take ownership of the receiver from the Arc<Mutex<Option>>
        // This can only be called once - EventBus::with_network() takes ownership
        if let Ok(mut guard) = self.recv_rx.lock() {
            if let Some(rx) = guard.take() {
                return rx;
            }
        }

        // If already taken or lock failed, return empty channel
        let (_tx, rx) = mpsc::channel(1);
        rx
    }

    async fn connect(&mut self, addr: &str) -> Result<()> {
        let backend = Self::connect_to_server(addr).await?;
        *self = backend;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected
            .store(false, std::sync::atomic::Ordering::SeqCst);
        if let Some(connection) = self.connection.take() {
            connection.close(0u32.into(), b"disconnect");
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Skip server certificate verification for development
#[derive(Debug)]
struct SkipServerVerification;

impl rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        // Return all signature schemes supported by ring crypto provider
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_only_backend() {
        let mut backend = LocalOnlyBackend::new();

        assert_eq!(backend.node_id(), NodeId::from_u64(0));
        assert!(!backend.is_connected());

        backend.connect("dummy").await.unwrap();
        assert!(!backend.is_connected());

        backend.disconnect().await.unwrap();
    }
}
