# QUIC Relay Server Design

## Overview

Lightweight, stateless relay server for transparent EventBus networking in ISSUN games.
Uses QUIC protocol (via quinn) for reliable, encrypted, low-latency event routing.

## Architecture

### High-Level Design

```
┌─────────────┐         QUIC          ┌──────────────┐         QUIC          ┌─────────────┐
│  Client A   │◄─────────────────────►│ Relay Server │◄─────────────────────►│  Client B   │
│  (Player 1) │                        │   (Stateless)│                        │  (Player 2) │
└─────────────┘                        └──────────────┘                        └─────────────┘
      │                                      │                                        │
      │ Publish(Event)                      │                                        │
      ├────────────────────────────────────►│                                        │
      │                                      │ Broadcast to all clients               │
      │                                      ├───────────────────────────────────────►│
      │                                      │                                        │
      │                                      │◄───────────────────────────────────────┤
      │                                      │ Publish(Event)                         │
      │◄────────────────────────────────────┤                                        │
      │ Receive(Event)                      │                                        │
```

### Deployment Options

#### Option 1: Standalone Binary (Recommended for MVP)
- **Pros**: Simple, portable, minimal dependencies
- **Cons**: Manual deployment, no auto-scaling
- **Use Case**: Development, small-scale games (< 100 concurrent players)

#### Option 2: Docker Container
- **Pros**: Reproducible, easy to deploy on any platform
- **Cons**: Slightly more overhead
- **Use Case**: Production deployments, Kubernetes/Docker Swarm

#### Option 3: Cloud-Native (Shuttle, Fly.io, Railway)
- **Pros**: Auto-scaling, managed infrastructure, global distribution
- **Cons**: Vendor lock-in, cost
- **Use Case**: Commercial games, global multiplayer

#### Option 4: Serverless (CloudRun, Lambda with Function URLs)
- **Pros**: Pay-per-use, infinite scale
- **Cons**: QUIC support limited, cold starts
- **Use Case**: Future consideration (requires WebTransport fallback)

**Recommendation for Phase 1**: **Option 1 + Option 2**
- Build standalone binary for development
- Add Dockerfile for production deployment
- Keep cloud-native adapters as future work (Phase 2)

### Crate Structure

```
issun/
├── crates/
│   ├── issun/           # Core library (existing)
│   ├── issun-macros/    # Proc macros (existing)
│   └── issun-server/    # NEW: Relay server
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs           # Entry point
│       │   ├── relay.rs          # Core relay logic
│       │   ├── connection.rs     # QUIC connection management
│       │   ├── room.rs           # Optional: Room/lobby system
│       │   └── config.rs         # Configuration
│       ├── Dockerfile
│       ├── docker-compose.yml
│       └── deploy/
│           ├── shuttle.toml      # Shuttle.rs config
│           └── fly.toml          # Fly.io config
└── examples/
    └── multiplayer-pong/         # NEW: Networked example
```

### Relay Server Components

#### 1. Core Relay (`relay.rs`)

```rust
pub struct RelayServer {
    /// QUIC endpoint
    endpoint: quinn::Endpoint,

    /// Active client connections
    clients: Arc<RwLock<HashMap<NodeId, ClientConnection>>>,

    /// Room manager (optional, for Phase 2)
    rooms: Option<RoomManager>,

    /// Configuration
    config: ServerConfig,
}

impl RelayServer {
    pub async fn new(config: ServerConfig) -> Result<Self>;
    pub async fn run(&mut self) -> Result<()>;

    // Core relay functions
    async fn handle_connection(&self, conn: quinn::Connection);
    async fn relay_event(&self, from: NodeId, event: RawNetworkEvent);
}
```

**Relay Modes:**
- **Broadcast**: Send to all connected clients (default)
- **Targeted**: Send to specific NodeId
- **Room-based**: Send to clients in same room (Phase 2)

#### 2. Connection Management (`connection.rs`)

```rust
pub struct ClientConnection {
    node_id: NodeId,
    connection: quinn::Connection,
    send_stream: Arc<Mutex<quinn::SendStream>>,
    last_seen: Instant,
}

impl ClientConnection {
    pub async fn send_event(&self, event: &RawNetworkEvent) -> Result<()>;
    pub async fn receive_events(&mut self) -> Result<Vec<RawNetworkEvent>>;
    pub fn is_alive(&self) -> bool;
}
```

**Connection Lifecycle:**
1. Client connects → Handshake (exchange NodeId)
2. Server adds to `clients` map
3. Bidirectional event streaming
4. Heartbeat every 5 seconds
5. Client disconnects → Remove from map

#### 3. Configuration (`config.rs`)

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Server bind address
    pub bind_addr: SocketAddr,

    /// TLS certificate (for QUIC)
    pub cert_path: PathBuf,
    pub key_path: PathBuf,

    /// Max concurrent connections
    pub max_clients: usize,

    /// Heartbeat interval (seconds)
    pub heartbeat_interval: u64,

    /// Enable room system
    pub enable_rooms: bool,
}
```

**Default Config:**
- `bind_addr`: `0.0.0.0:5000`
- `max_clients`: 1000
- `heartbeat_interval`: 5
- `enable_rooms`: false (Phase 2)

#### 4. Room System (`room.rs`) - Phase 2

```rust
pub struct RoomManager {
    rooms: HashMap<RoomId, Room>,
}

pub struct Room {
    id: RoomId,
    clients: HashSet<NodeId>,
    max_clients: usize,
}
```

### Client-Side Integration

#### Update `NetworkBackend` for QUIC Client

```rust
// crates/issun/src/network/backend.rs

pub struct QuicClientBackend {
    node_id: NodeId,
    connection: quinn::Connection,
    send_tx: mpsc::Sender<RawNetworkEvent>,
    recv_rx: mpsc::Receiver<RawNetworkEvent>,
}

impl QuicClientBackend {
    pub async fn connect(server_addr: &str) -> Result<Self>;
}

#[async_trait]
impl NetworkBackend for QuicClientBackend {
    fn node_id(&self) -> NodeId { self.node_id }
    async fn send(&self, event: RawNetworkEvent) -> Result<()>;
    fn receive_stream(&self) -> mpsc::Receiver<RawNetworkEvent>;
    async fn connect(&mut self, addr: &str) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    fn is_connected(&self) -> bool;
}
```

### Protocol Specification

#### Wire Format (Bincode over QUIC)

```
┌──────────────────────────────────────────────────────────────┐
│ Frame Header (8 bytes)                                       │
├──────────────────────────────────────────────────────────────┤
│ Magic (2 bytes): 0x4953 ("IS" for ISSUN)                   │
│ Version (1 byte): 0x01                                       │
│ Frame Type (1 byte): 0x01=Event, 0x02=Heartbeat, 0x03=Ack  │
│ Payload Length (4 bytes): u32 little-endian                 │
├──────────────────────────────────────────────────────────────┤
│ Payload (N bytes): Bincode-serialized RawNetworkEvent       │
└──────────────────────────────────────────────────────────────┘
```

#### Handshake Flow

```
Client                          Server
  │                               │
  │─────── QUIC Connect ─────────►│
  │                               │
  │◄──── TLS Handshake (QUIC) ───┤
  │                               │
  │──── Handshake Frame ─────────►│
  │   (NodeId, Version)           │
  │                               │
  │◄──── Handshake Ack ───────────┤
  │   (Assigned NodeId if 0)      │
  │                               │
  │◄───── Event Stream ──────────►│
```

### Deployment Configuration

#### Dockerfile

```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p issun-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/issun-server /usr/local/bin/
EXPOSE 5000/udp
CMD ["issun-server"]
```

#### docker-compose.yml

```yaml
version: '3.8'
services:
  relay:
    build:
      context: .
      dockerfile: crates/issun-server/Dockerfile
    ports:
      - "5000:5000/udp"
    environment:
      - RUST_LOG=info
      - ISSUN_BIND_ADDR=0.0.0.0:5000
      - ISSUN_MAX_CLIENTS=1000
    volumes:
      - ./certs:/app/certs:ro
    restart: unless-stopped
```

#### Shuttle.rs Config

```toml
# crates/issun-server/deploy/shuttle.toml
name = "issun-relay"
version = "0.1.0"

[runtime]
type = "custom"
entry = "target/release/issun-server"

[resources.udp]
port = 5000
```

#### Environment Variables

```bash
# .env
ISSUN_BIND_ADDR=0.0.0.0:5000
ISSUN_CERT_PATH=/app/certs/cert.pem
ISSUN_KEY_PATH=/app/certs/key.pem
ISSUN_MAX_CLIENTS=1000
ISSUN_HEARTBEAT_INTERVAL=5
ISSUN_METRICS_PORT=9090
RUST_LOG=issun_server=info
```

### TLS Certificate Generation

**Development (Self-signed):**
```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes \
  -subj "/CN=localhost"
```

**Production (Let's Encrypt):**
```bash
certbot certonly --standalone -d relay.yourgame.com
```

### Monitoring & Observability

#### Metrics (via Prometheus)

```rust
// crates/issun-server/src/metrics.rs
pub struct Metrics {
    pub connected_clients: Gauge,
    pub active_rooms: Gauge,
    pub events_relayed: CounterVec,  // by scope (broadcast/targeted/to_server)
    pub connection_duration: HistogramVec,  // by status
    pub relay_latency: HistogramVec,  // by scope (microseconds)
    pub bytes_sent: CounterVec,  // by client_id
    pub bytes_received: CounterVec,  // by client_id
}
```

**Exposed Metrics Endpoints**:
- `http://localhost:9090/metrics` - Prometheus format
- `http://localhost:9090/health` - Health check

#### Logging

```rust
use tracing::{info, warn, error};

info!(
    client_count = clients.len(),
    "Client connected: {:?}",
    node_id
);
```

### Security Considerations

1. **TLS Encryption**: QUIC mandates TLS 1.3
2. **Rate Limiting**: Per-client event rate limit (100 events/sec)
3. **Authentication**: Optional token-based auth (Phase 2)
4. **DDoS Protection**: Connection limit, IP-based throttling

### Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Max Concurrent Clients | 1,000 | Per instance |
| Latency (P50) | < 50ms | Server-to-client relay |
| Latency (P99) | < 200ms | Under load |
| Throughput | 10,000 events/sec | Per instance |
| Memory | < 100MB | Base + 10KB per client |

### Implementation Plan

#### Phase 1, Week 2 (Completed)
- [x] Create `issun-server` crate
- [x] Implement basic QUIC relay server
- [x] Implement `QuicClientBackend`
- [x] Add handshake protocol
- [x] Basic connection management

#### Phase 1, Week 3 (Completed)
- [x] Add Dockerfile
- [x] Add docker-compose.yml
- [x] Create multiplayer-pong example
- [x] Documentation
- [x] Integration tests

#### Phase 2 (Completed)
- [x] Room/lobby system
- [x] Prometheus metrics
- [ ] Token-based authentication (Future)
- [ ] Cloud deployment adapters (Shuttle, Fly.io) (Future)

### Quick Start

#### 1. Generate TLS Certificates (First Time Only)

For development, create self-signed certificates:

```bash
mkdir -p certs
openssl req -x509 -newkey rsa:4096 -keyout certs/key.pem -out certs/cert.pem \
  -days 365 -nodes -subj "/CN=localhost"
```

For production, use Let's Encrypt:

```bash
certbot certonly --standalone -d relay.yourgame.com
```

#### 2. Start the Server

**Development (local):**
```bash
# Using cargo
RUST_LOG=issun_server=info cargo run -p issun-server --release

# Or use Make
make server
```

**Production (Docker):**
```bash
docker-compose up -d
```

**With custom configuration:**
```bash
ISSUN_BIND_ADDR=0.0.0.0:8080 \
ISSUN_CERT_PATH=/etc/letsencrypt/live/relay.yourgame.com/fullchain.pem \
ISSUN_KEY_PATH=/etc/letsencrypt/live/relay.yourgame.com/privkey.pem \
cargo run -p issun-server --release
```

#### 3. Verify Server is Running

Server should output:
```
INFO Starting ISSUN relay server...
INFO Configuration loaded: ServerConfig { bind_addr: 0.0.0.0:5000, ... }
INFO Relay server listening on 0.0.0.0:5000
INFO Relay server started, waiting for connections...
```

Test connectivity:
```bash
# Check if port is open
nc -zv localhost 5000
```

### Example Usage

#### Server Startup

```bash
# Development
cargo run -p issun-server

# Docker
docker-compose up -d

# With custom config
ISSUN_BIND_ADDR=0.0.0.0:8080 cargo run -p issun-server
```

#### Client Connection

```rust
use issun::prelude::*;
use issun::network::QuicClientBackend;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to relay server
    let backend = QuicClientBackend::connect("relay.yourgame.com:5000").await?;

    // Create EventBus with network
    let mut bus = EventBus::new().with_network(backend);
    bus.register_networked_event::<PlayerMove>();

    // Game loop
    loop {
        // Poll network events
        bus.poll_network();

        // Publish local events (auto-broadcast)
        bus.publish(PlayerMove { x: 10, y: 20 });

        // Dispatch
        bus.dispatch();

        // Read events (local + remote)
        for event in bus.reader::<PlayerMove>().iter() {
            println!("Player moved: {:?}", event);
        }

        tokio::time::sleep(Duration::from_millis(16)).await; // 60 FPS
    }
}
```

### Testing Strategy

#### Unit Tests
- Connection lifecycle
- Event serialization/deserialization
- Room management

#### Integration Tests
- 2-client ping-pong
- 10-client broadcast stress test
- Connection drop/reconnect

#### Load Tests
- 1000 concurrent clients
- 10,000 events/sec throughput

### Migration Path (P2P in Future)

Relay server design is intentionally simple to allow future P2P migration:
- Clients can discover peers via relay
- Establish direct QUIC connections
- Relay acts as fallback for NAT traversal

---

## Summary

**Recommended Approach**:
1. Implement standalone `issun-server` binary
2. Add Docker support for easy deployment
3. Keep cloud-native as optional future work
4. Focus on simplicity and statelessness

**Key Design Principles**:
- **Stateless**: No game state on server
- **Transparent**: Clients use same EventBus API
- **Scalable**: Horizontal scaling via load balancer
- **Portable**: Docker + bare metal support

**Next Steps**:
1. Create `crates/issun-server/` crate
2. Implement basic QUIC relay
3. Add `QuicClientBackend` to `issun` crate
4. Create multiplayer example
