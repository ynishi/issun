//! Core network types for transparent event transmission

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unique identifier for network nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

impl NodeId {
    /// Generate a random NodeId
    pub fn random() -> Self {
        Self(rand::random())
    }

    /// Create NodeId from u64
    pub fn from_u64(id: u64) -> Self {
        Self(id)
    }

    /// Get the inner u64 value
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node({})", self.0)
    }
}

/// Metadata attached to networked events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetadata {
    /// Source node that originated this event
    pub sender: NodeId,
    /// Unix timestamp in milliseconds
    pub timestamp: u64,
    /// Sequence number for ordering guarantees
    pub sequence: u64,
}

impl NetworkMetadata {
    /// Create new metadata
    pub fn new(sender: NodeId, sequence: u64) -> Self {
        Self {
            sender,
            timestamp: now_millis(),
            sequence,
        }
    }
}

/// Event propagation scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkScope {
    /// Broadcast to all nodes (default)
    Broadcast,
    /// Send to server only (Client -> Server)
    ToServer,
    /// Send to specific node
    Targeted(NodeId),
}

impl Default for NetworkScope {
    fn default() -> Self {
        Self::Broadcast
    }
}

/// Wrapper for networked events with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkedEvent<T> {
    pub metadata: NetworkMetadata,
    pub scope: NetworkScope,
    pub payload: T,
}

impl<T> NetworkedEvent<T> {
    /// Create new networked event
    pub fn new(payload: T, sender: NodeId, sequence: u64, scope: NetworkScope) -> Self {
        Self {
            metadata: NetworkMetadata::new(sender, sequence),
            scope,
            payload,
        }
    }
}

/// Get current time in milliseconds since UNIX epoch
pub fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id() {
        let id1 = NodeId::random();
        let id2 = NodeId::random();
        assert_ne!(id1, id2);

        let id3 = NodeId::from_u64(42);
        assert_eq!(id3.as_u64(), 42);
    }

    #[test]
    fn test_network_metadata() {
        let sender = NodeId::from_u64(1);
        let metadata = NetworkMetadata::new(sender, 10);

        assert_eq!(metadata.sender, sender);
        assert_eq!(metadata.sequence, 10);
        assert!(metadata.timestamp > 0);
    }

    #[test]
    fn test_network_scope() {
        assert_eq!(NetworkScope::default(), NetworkScope::Broadcast);
    }

    #[test]
    fn test_networked_event() {
        let sender = NodeId::from_u64(1);
        let event = NetworkedEvent::new("test".to_string(), sender, 1, NetworkScope::Broadcast);

        assert_eq!(event.payload, "test");
        assert_eq!(event.metadata.sender, sender);
        assert_eq!(event.scope, NetworkScope::Broadcast);
    }
}
