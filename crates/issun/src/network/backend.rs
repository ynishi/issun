//! NetworkBackend trait and implementations

use super::types::{NetworkMetadata, NetworkScope, NodeId};
use crate::error::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Type-erased network event (for receiving)
#[derive(Debug, Clone)]
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
