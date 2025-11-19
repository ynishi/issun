# Multiplayer Pong

A simple 2-player networked pong game demonstrating ISSUN's network-transparent EventBus.

## Features

- **Network-Transparent Events**: Players' paddle movements are automatically synchronized via the relay server
- **Host/Client Model**: First player (even ID) acts as host and simulates ball physics
- **Real-time Gameplay**: 60 FPS game loop with immediate network event propagation
- **Simple Graphics**: ASCII art rendering in terminal

## How It Works

### Network Events

1. **PaddleMove**: Each player broadcasts their paddle position every frame
2. **BallUpdate**: Host broadcasts ball state; client receives and renders

### Architecture

```
Player 1 (Host)                Relay Server                Player 2 (Client)
     │                              │                              │
     │──── PaddleMove(P1, y=10) ───►│────────────────────────────►│
     │                              │                              │
     │◄─── PaddleMove(P2, y=15) ────│◄────────────────────────────│
     │                              │                              │
     │──── BallUpdate(x, y, vx, vy) ►│────────────────────────────►│
     │      (Host simulates physics) │   (Client receives state)   │
```

## Running the Game

### 1. Start the Relay Server

In one terminal:

```bash
make server
```

Or manually:

```bash
RUST_LOG=issun_server=info cargo run -p issun-server --release
```

### 2. Start Player 1 (Host)

In another terminal:

```bash
cargo run -p multiplayer-pong -- --server 127.0.0.1:5000
```

You'll see:
```
🎮 Connecting to relay server at 127.0.0.1:5000...
✅ Connected! Your Player ID: 123456789
🎲 You are the HOST player
⏳ Waiting for another player to join...
```

### 3. Start Player 2 (Client)

In a third terminal:

```bash
cargo run -p multiplayer-pong -- --server 127.0.0.1:5000
```

Both players should now see the game start!

## Controls

- **W**: Move paddle up
- **S**: Move paddle down
- **Q**: Quit game

## Game Rules

1. Hit the ball (●) with your paddle (█)
2. If the ball goes past your paddle, the other player scores
3. Ball bounces off top and bottom walls
4. First to... well, it's just for fun! 😄

## Technical Details

### Networked Events

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct PaddleMove {
    player_id: u64,
    y_position: i32,
}

impl Event for PaddleMove {
    fn is_networked() -> bool { true }
    fn network_scope() -> NetworkScope { NetworkScope::Broadcast }
}
```

### EventBus Integration

```rust
// Create EventBus with network backend
let backend = QuicClientBackend::connect_to_server(&args.server).await?;
let mut bus = EventBus::new().with_network(backend);

// Register networked events
bus.register_networked_event::<PaddleMove>();
bus.register_networked_event::<BallUpdate>();

// Game loop
loop {
    bus.poll_network();           // Receive remote events
    bus.publish(my_paddle_move);  // Send local events
    bus.dispatch();               // Swap buffers
    // ... update and render
}
```

## Troubleshooting

**"Connection failed"**
- Make sure the relay server is running on the specified address
- Check firewall settings

**"Waiting for another player"**
- Start a second client in another terminal
- Both clients must connect to the same relay server

**Input not working**
- Some terminals require raw mode for single-key input
- Try pressing keys followed by Enter if immediate response doesn't work

## Next Steps

Try modifying the game:
- Add score tracking
- Implement power-ups
- Add sound effects (terminal beeps)
- Support more than 2 players
- Add a lobby system before starting

## Code Structure

- `main.rs`: Entry point, game loop, and rendering
- `PaddleMove` / `BallUpdate`: Networked event types
- `GameState`: Game logic and physics simulation
- Input handling: Async thread reading stdin
