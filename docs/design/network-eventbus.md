# Network-Transparent EventBus Design

**Status:** Phase 1 - In Progress
**Author:** Claude + yutakanishimura
**Date:** 2025-01-19

## Overview

Network-transparent EventBus allows game events to be seamlessly transmitted across network nodes using the same API as local events. Players can dispatch events with `bus.dispatch(Event)` and have them automatically routed to other connected clients/servers.

## Goals

- **Transparency**: Same API for local and networked events
- **Performance**: QUIC transport (quinn) + binary serialization (bincode)
- **Simplicity**: Central relay server with lightweight routing only
- **Extensibility**: Clear path to P2P, state sync, and event sourcing

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Application Layer                       │
│  bus.dispatch(Event)  /  #[subscribe(Event)]                │
└──────────────────────────┬──────────────────────────────────┘
                           │ Transparent API
┌──────────────────────────▼──────────────────────────────────┐
│                       EventBus Core                          │
│  ┌────────────────┐  ┌─────────────────┐                   │
│  │ Local Dispatch │  │ Network Dispatch│                   │
│  │  (immediate)   │  │  (async queue)  │                   │
│  └────────────────┘  └────────┬────────┘                   │
└─────────────────────────────────┼──────────────────────────┘
                                  │
┌─────────────────────────────────▼──────────────────────────┐
│                    NetworkBackend Trait                      │
│  ┌──────────────┐  ┌─────────────┐  ┌──────────────┐      │
│  │   LocalOnly  │  │ CentralTCP  │  │  P2P (Future)│      │
│  └──────────────┘  └──────┬──────┘  └──────────────┘      │
└─────────────────────────────┼──────────────────────────────┘
                              │ QUIC (quinn)
                              │ serde + bincode
┌─────────────────────────────▼──────────────────────────────┐
│              Central Routing Server (Relay)                 │
│  - Lightweight event routing                                │
│  - NodeId registry                                          │
│  - No game logic                                            │
└─────────────────────────────────────────────────────────────┘
```

## Core Types

### NodeId

```rust
/// Unique identifier for network nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

impl NodeId {
    pub fn random() -> Self {
        Self(rand::random())
    }
}
```

### NetworkMetadata

```rust
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
```

### NetworkScope

```rust
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
```

### NetworkedEvent

```rust
/// Wrapper for networked events with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkedEvent<T> {
    pub metadata: NetworkMetadata,
    pub scope: NetworkScope,
    pub payload: T,
}
```

## Macro Extension

### #[event] Attribute

```rust
// Local-only event (default)
#[event]
pub LocalGameTick {
    pub frame: u64,
}

// Networked event - broadcast to all
#[event(networked)]
pub MissionRequested {
    pub faction: FactionId,
    pub target: TerritoryId,
}

// Networked - explicit broadcast
#[event(networked = "broadcast")]
pub ChatMessage {
    pub sender: String,
    pub text: String,
}

// Networked - client to server only
#[event(networked = "to_server")]
pub PlayerInput {
    pub action: Action,
}

// Networked - targeted send (scope specified at dispatch)
#[event(networked = "targeted")]
pub PrivateMessage {
    pub recipient: String,
    pub text: String,
}
```

### Generated Code

```rust
#[event(networked = "broadcast")]
pub MissionRequested {
    pub faction: FactionId,
    pub target: TerritoryId,
}

// Expands to:

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionRequested {
    pub faction: FactionId,
    pub target: TerritoryId,
}

impl Event for MissionRequested {
    fn type_id() -> TypeId {
        TypeId::of::<Self>()
    }

    fn is_networked() -> bool {
        true
    }

    fn network_scope() -> NetworkScope {
        NetworkScope::Broadcast
    }
}
```

## NetworkBackend Trait

```rust
#[async_trait::async_trait]
pub trait NetworkBackend: Send + Sync + 'static {
    /// Get this node's ID
    fn node_id(&self) -> NodeId;

    /// Send event to network
    async fn send<T: Event + Serialize>(
        &self,
        event: NetworkedEvent<T>,
    ) -> Result<()>;

    /// Receive stream of raw events
    fn receive_stream(&self) -> mpsc::Receiver<RawNetworkEvent>;

    /// Connect to network
    async fn connect(&mut self, addr: &str) -> Result<()>;

    /// Disconnect from network
    async fn disconnect(&mut self) -> Result<()>;
}

/// Type-erased network event (for receiving)
pub struct RawNetworkEvent {
    pub metadata: NetworkMetadata,
    pub type_name: String,
    pub payload: Vec<u8>,  // bincode serialized
}
```

## EventBus Integration

### Extended EventBus

```rust
pub struct EventBus {
    // Existing fields
    events: HashMap<TypeId, Vec<u8>>,

    // Network backend (optional)
    network: Option<Arc<dyn NetworkBackend>>,

    // Async send queue
    network_tx: mpsc::Sender<NetworkTask>,

    // Current event metadata (for subscribers)
    current_metadata: Option<NetworkMetadata>,
}

impl EventBus {
    /// Enable network support
    pub fn with_network(mut self, backend: impl NetworkBackend) -> Self {
        let (tx, rx) = mpsc::channel(1000);
        self.network = Some(Arc::new(backend));
        self.network_tx = tx;

        // Spawn background workers
        tokio::spawn(network_dispatch_worker(rx, self.network.clone()));
        tokio::spawn(network_receive_worker(self.network.clone(), /* event_bus_ref */));

        self
    }

    /// Dispatch event (transparent API)
    pub fn dispatch<T: Event>(&mut self, event: T) {
        // 1. Local dispatch (immediate)
        self.local_dispatch(event.clone());

        // 2. Network dispatch (async, if networked)
        if T::is_networked() && self.network.is_some() {
            let metadata = NetworkMetadata {
                sender: self.network.as_ref().unwrap().node_id(),
                timestamp: now_millis(),
                sequence: self.next_sequence(),
            };

            let networked = NetworkedEvent {
                metadata,
                scope: T::network_scope(),
                payload: event,
            };

            let _ = self.network_tx.try_send(NetworkTask::Send(Box::new(networked)));
        }
    }

    /// Inject remote event into local bus
    fn inject_remote<T: Event>(&mut self, networked: NetworkedEvent<T>) {
        self.current_metadata = Some(networked.metadata);
        self.local_dispatch(networked.payload);
        self.current_metadata = None;
    }

    /// Get metadata of current event (for subscribers)
    pub fn last_metadata(&self) -> Option<&NetworkMetadata> {
        self.current_metadata.as_ref()
    }
}
```

## Central Relay Server

### Server Implementation

```rust
use quinn::{Endpoint, ServerConfig};

pub struct RelayServer {
    endpoint: Endpoint,
    clients: Arc<RwLock<HashMap<NodeId, Connection>>>,
}

impl RelayServer {
    pub async fn new(bind_addr: &str) -> Result<Self> {
        let server_config = configure_server()?;
        let endpoint = Endpoint::server(server_config, bind_addr.parse()?)?;

        Ok(Self {
            endpoint,
            clients: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn run(self) -> Result<()> {
        while let Some(conn) = self.endpoint.accept().await {
            let clients = self.clients.clone();
            tokio::spawn(handle_client(conn, clients));
        }
        Ok(())
    }
}

async fn handle_client(
    connecting: quinn::Connecting,
    clients: Arc<RwLock<HashMap<NodeId, Connection>>>,
) -> Result<()> {
    let conn = connecting.await?;

    // Handshake: receive NodeID
    let mut recv = conn.accept_uni().await?;
    let node_id: NodeId = bincode::deserialize_from(&mut recv)?;

    clients.write().await.insert(node_id, conn.clone());

    // Event routing loop
    loop {
        let mut recv = conn.accept_uni().await?;
        let event: RawNetworkEvent = bincode::deserialize_from(&mut recv)?;

        // Route based on scope
        match event.scope {
            NetworkScope::Broadcast => {
                // Send to all clients except sender
                for (id, client_conn) in clients.read().await.iter() {
                    if *id != event.metadata.sender {
                        send_event(client_conn, &event).await?;
                    }
                }
            }
            NetworkScope::Targeted(target) => {
                if let Some(conn) = clients.read().await.get(&target) {
                    send_event(conn, &event).await?;
                }
            }
            NetworkScope::ToServer => {
                // Server-side processing (future)
            }
        }
    }
}
```

### Client Backend (CentralTcp)

```rust
pub struct CentralTcpBackend {
    node_id: NodeId,
    connection: Arc<RwLock<Option<Connection>>>,
    rx: mpsc::Receiver<RawNetworkEvent>,
}

#[async_trait::async_trait]
impl NetworkBackend for CentralTcpBackend {
    fn node_id(&self) -> NodeId {
        self.node_id
    }

    async fn send<T: Event + Serialize>(
        &self,
        event: NetworkedEvent<T>,
    ) -> Result<()> {
        let conn = self.connection.read().await;
        let conn = conn.as_ref().ok_or(Error::NotConnected)?;

        let raw = RawNetworkEvent {
            metadata: event.metadata,
            type_name: std::any::type_name::<T>().to_string(),
            payload: bincode::serialize(&event.payload)?,
        };

        let mut send = conn.open_uni().await?;
        bincode::serialize_into(&mut send, &raw)?;
        send.finish().await?;

        Ok(())
    }

    async fn connect(&mut self, server_addr: &str) -> Result<()> {
        let endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
        let conn = endpoint.connect(server_addr.parse()?, "issun-relay")?.await?;

        // Handshake: send NodeID
        let mut send = conn.open_uni().await?;
        bincode::serialize_into(&mut send, &self.node_id)?;
        send.finish().await?;

        *self.connection.write().await = Some(conn.clone());

        // Start receive loop
        let (tx, rx) = mpsc::channel(1000);
        self.rx = rx;
        tokio::spawn(receive_loop(conn, tx));

        Ok(())
    }
}
```

## Usage Example

```rust
// main.rs
#[tokio::main]
async fn main() -> Result<()> {
    // Setup network backend
    let network = CentralTcpBackend::new(NodeId::random())
        .connect("127.0.0.1:5000").await?;

    let game = GameBuilder::new()
        .with_title("Border Economy - Multiplayer")
        .with_network(network)  // Enable transparent networking
        .with_plugin(EconomyPlugin)
        .build()?;

    game.run().await
}

// events.rs
#[event(networked = "broadcast")]
pub MissionRequested {
    pub faction: FactionId,
    pub target: TerritoryId,
}

// plugins/faction.rs
#[subscribe(MissionRequested)]
async fn on_mission_requested(
    &mut self,
    event: &MissionRequested,
    #[state] ops: &mut FactionOpsState,
    bus: &EventBus,
) {
    // Get sender metadata
    if let Some(metadata) = bus.last_metadata() {
        println!("Received from node: {:?}", metadata.sender);
    }

    // Process normally
    ops.sorties_launched += 1;
}
```

## Phase 1 Implementation Plan

### Week 1: Core Infrastructure
- [ ] Define `NodeId`, `NetworkMetadata`, `NetworkScope` types
- [ ] Implement `NetworkBackend` trait
- [ ] Integrate `EventBus::with_network()`
- [ ] Add network task queue and workers

### Week 2: Macro Extension
- [ ] Extend `#[event]` macro with `networked` attribute
- [ ] Generate `NetworkedEventMarker` trait
- [ ] Add `is_networked()` and `network_scope()` to Event trait
- [ ] Write macro tests

### Week 3: Central Relay Server
- [ ] Implement `RelayServer` with quinn
- [ ] Implement `CentralTcpBackend`
- [ ] Add connection management
- [ ] Implement event routing logic
- [ ] Add bincode serialization

### Week 4: Integration & Testing
- [ ] Convert border-economy to multiplayer
- [ ] Test with multiple clients
- [ ] Add error handling and reconnection
- [ ] Write documentation
- [ ] Create examples

## Future Extensions (Phase 2+)

### Event Sourcing
- Command/Query separation
- Optimistic locking with version numbers
- Event replay and state reconstruction

### P2P Backend
- libp2p integration
- Peer discovery (mDNS, DHT)
- NAT traversal (STUN/TURN)

### State Synchronization
- Snapshot + delta sync
- Conflict resolution strategies
- Bandwidth optimization

### Advanced Features
- Compression (zstd) for large events
- Event batching for performance
- QoS and priority queues
- Metrics and monitoring

## Design Decisions

### Why QUIC (quinn)?
- Built-in encryption (TLS 1.3)
- 0-RTT connection establishment
- Multiplexing without head-of-line blocking
- Better than TCP for game networking

### Why bincode?
- Fast binary serialization
- Small payload size
- Already using serde
- Type-safe with Rust types

### Why Central Relay First?
- Simpler than P2P for Phase 1
- Easier to debug and test
- Clear upgrade path to P2P
- Can add server-side logic later

### Event vs State Sync?
- Events are natural for game logic
- Already using EventBus
- Clear semantics for networked actions
- State sync can be added later as optimization

## Open Questions

1. **Event Ordering**: Do we need guaranteed order for all events or just per-type?
2. **Reliability**: Should we support unreliable events (UDP-like) for position updates?
3. **Bandwidth**: Do we need event batching from the start?
4. **Security**: How to prevent malicious event injection?
5. **Server Authority**: Which events require server validation?

## References

- [quinn documentation](https://docs.rs/quinn/)
- [bincode documentation](https://docs.rs/bincode/)
- [Bevy networking RFC](https://github.com/bevyengine/bevy/discussions/10572)
- [Godot High-level Multiplayer](https://docs.godotengine.org/en/stable/tutorials/networking/high_level_multiplayer.html)
