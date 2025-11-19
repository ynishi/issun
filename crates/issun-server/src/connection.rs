//! Client connection management

use anyhow::Result;
use issun::network::{backend::RawNetworkEvent, NodeId};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// Client connection state
pub struct ClientConnection {
    /// Unique node ID
    pub node_id: NodeId,

    /// QUIC connection
    connection: quinn::Connection,

    /// Send stream (reused for all events)
    send_stream: Arc<Mutex<Option<quinn::SendStream>>>,

    /// Last seen timestamp
    last_seen: Instant,
}

impl ClientConnection {
    /// Create a new client connection
    pub fn new(node_id: NodeId, connection: quinn::Connection) -> Self {
        Self {
            node_id,
            connection,
            send_stream: Arc::new(Mutex::new(None)),
            last_seen: Instant::now(),
        }
    }

    /// Send an event to this client
    pub async fn send_event(&self, event: &RawNetworkEvent) -> Result<()> {
        // Serialize event with frame header
        let payload = bincode::serialize(event)?;
        let frame = Self::create_frame(FrameType::Event, &payload);

        // Get or create send stream
        let mut stream_guard = self.send_stream.lock().await;
        if stream_guard.is_none() {
            *stream_guard = Some(self.connection.open_uni().await?);
        }

        if let Some(stream) = stream_guard.as_mut() {
            stream.write_all(&frame).await?;
            debug!(
                node_id = ?self.node_id,
                size = frame.len(),
                "Sent event to client"
            );
        }

        Ok(())
    }

    /// Receive events from this client
    pub async fn receive_events(&mut self) -> Result<Vec<RawNetworkEvent>> {
        let mut events = Vec::new();

        // Accept incoming streams
        while let Ok(recv_result) = tokio::time::timeout(
            tokio::time::Duration::from_millis(10),
            self.connection.accept_uni(),
        )
        .await
        {
            let mut stream = recv_result?;

            // Read frame header
            let mut header = [0u8; 8];
            stream.read_exact(&mut header).await?;

            let (frame_type, payload_len) = Self::parse_header(&header)?;

            if frame_type != FrameType::Event {
                warn!("Unexpected frame type: {:?}", frame_type);
                continue;
            }

            // Read payload
            let mut payload = vec![0u8; payload_len];
            stream.read_exact(&mut payload).await?;

            // Deserialize event
            let event: RawNetworkEvent = bincode::deserialize(&payload)?;
            events.push(event);

            self.last_seen = Instant::now();
        }

        Ok(events)
    }

    /// Check if connection is alive
    pub fn is_alive(&self) -> bool {
        self.last_seen.elapsed().as_secs() < 30 // 30 second timeout
    }

    /// Create a wire frame with header
    fn create_frame(frame_type: FrameType, payload: &[u8]) -> Vec<u8> {
        let mut frame = Vec::with_capacity(8 + payload.len());

        // Magic: "IS" (0x4953)
        frame.extend_from_slice(&[0x49, 0x53]);

        // Version: 0x01
        frame.push(0x01);

        // Frame type
        frame.push(frame_type as u8);

        // Payload length (u32 little-endian)
        frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());

        // Payload
        frame.extend_from_slice(payload);

        frame
    }

    /// Parse frame header
    fn parse_header(header: &[u8; 8]) -> Result<(FrameType, usize)> {
        // Check magic
        if header[0] != 0x49 || header[1] != 0x53 {
            anyhow::bail!("Invalid magic bytes");
        }

        // Check version
        if header[2] != 0x01 {
            anyhow::bail!("Unsupported version: {}", header[2]);
        }

        // Parse frame type
        let frame_type = FrameType::from_u8(header[3])?;

        // Parse payload length
        let payload_len = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;

        Ok((frame_type, payload_len))
    }
}

/// Frame types for wire protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum FrameType {
    Event = 0x01,
    Heartbeat = 0x02,
    Ack = 0x03,
}

impl FrameType {
    fn from_u8(value: u8) -> Result<Self> {
        match value {
            0x01 => Ok(FrameType::Event),
            0x02 => Ok(FrameType::Heartbeat),
            0x03 => Ok(FrameType::Ack),
            _ => anyhow::bail!("Unknown frame type: {:#x}", value),
        }
    }
}
