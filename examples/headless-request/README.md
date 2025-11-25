# Headless Request Example

Demonstrates **Pattern 1**: HTTP API controlling a headless ISSUN simulation.

## Architecture

```
HTTP Client (curl)
    ↓
  POST /increment
  POST /reset
    ↓
Axum HTTP Server
    ↓
tokio::sync::mpsc::channel
    ↓
HeadlessRunner
    ↓
Scene::on_update()
    ↓
SimulationCounter (updates directly)
```

## Key Features

- ✅ **No UI dependencies** - Pure headless execution
- ✅ **HTTP API** - Control via REST endpoints
- ✅ **Channel-based** - Commands sent through tokio mpsc
- ✅ **Event-driven** - Processes commands in game loop

## Usage

### 1. Start the simulation

```bash
cargo run --release
```

Output:
```
🚀 Starting headless simulation with HTTP API...
   Listen on: http://localhost:3000

Available endpoints:
  POST /increment - Increment the counter
  POST /reset     - Reset the counter
  GET  /status    - Get current status

✅ HTTP API started
```

### 2. Send commands

In another terminal:

```bash
# Increment the counter
curl -X POST http://localhost:3000/increment

# Increment again
curl -X POST http://localhost:3000/increment

# Reset to zero
curl -X POST http://localhost:3000/reset

# Check status (TODO: Currently returns cached value)
curl http://localhost:3000/status
```

### 3. Observe the logs

```
[Tick  30] 📨✅ HTTP Increment -> Counter: 1
[Tick  51] 📨✅ HTTP Increment -> Counter: 2
[Tick  71] 📨🔄 HTTP Reset -> Counter: 0
[Tick 100] Counter: 0
```

## Implementation Details

### Command Flow

1. **HTTP Handler** receives request
2. Sends `ApiCommand` through channel
3. **Scene::on_update()** processes commands with `try_recv()`
4. Updates `SimulationCounter` resource directly
5. Logs to stdout

### Why Direct Updates?

For simplicity, this example updates the counter **directly** instead of using EventBus:

```rust
// Process incoming HTTP commands directly
while let Ok(cmd) = self.command_rx.try_recv() {
    if let Some(mut counter) = resources.get_mut::<SimulationCounter>().await {
        match cmd {
            ApiCommand::Increment => counter.increment(),
            ApiCommand::Reset => counter.reset(),
        }
    }
}
```

### Alternative: EventBus Pattern

For more complex games, you'd publish events:

```rust
// HTTP command -> Event
bus.publish(IncrementRequested);

// System processes event
for event in bus.reader::<IncrementRequested>().iter() {
    counter.increment();
}
```

## Limitations (TODOs)

1. **Status endpoint** - Currently returns cached value
   - Solution: Add query channel for real-time state
2. **No graceful shutdown** - Ctrl+C leaves zombie process
   - Solution: Add tokio signal handlers
3. **Single simulation** - Only one game instance
   - Solution: Use routing parameters for multiple games

## Production Improvements

### 1. Real-time Status Query

```rust
enum SimCommand {
    Increment,
    Reset,
    Query(oneshot::Sender<StatusResponse>), // 🆕
}
```

### 2. Graceful Shutdown

```rust
tokio::select! {
    _ = runner.run() => {},
    _ = tokio::signal::ctrl_c() => {
        println!("Shutting down...");
    }
}
```

### 3. Multiple Game Instances

```rust
// Router with game ID
.route("/game/:id/increment", post(handle_increment))

// State holds multiple channels
struct AppState {
    games: HashMap<String, mpsc::Sender<ApiCommand>>,
}
```

## Comparison with Other Patterns

| Pattern | Complexity | Performance | Use Case |
|---------|-----------|-------------|----------|
| **Pattern 1** (This) | ⭐⭐ Simple | ⭐⭐⭐ Fast | Single simulation |
| **Pattern 2** | ⭐⭐⭐ Medium | ⭐⭐⭐ Faster | Extensible/Reusable |
| Pattern 3 (QUIC) | ⭐⭐⭐⭐ Complex | ⭐⭐ Network | Distributed |

### Pattern 1 vs Pattern 2 Details

| Aspect | **Pattern 1** (This) | **Pattern 2** |
|--------|---------------------|---------------|
| **Channel Owner** | Scene | ChannelHeadlessRunner |
| **Command Processing** | Polling (`try_recv()`) | Event-driven (`tokio::select!`) |
| **Latency** | ~25ms average (50ms tick / 2) | <1ms (immediate) |
| **Integration** | Direct resource updates | EventBus events |
| **Reusability** | Scene-specific | Any Scene can subscribe |
| **Standard Pattern** | No | ✅ Yes (EventBus) |

**When to use Pattern 1:**
- ✅ Simplicity is more important than latency
- ✅ Commands are specific to one Scene
- ✅ 25ms latency is acceptable

**When to use Pattern 2:**
- ✅ Need low-latency command processing (<1ms)
- ✅ Multiple scenes might process the same commands
- ✅ Want to follow ISSUN's standard EventBus pattern
- ✅ Plan to extend with more command types

## Related Examples

- `headless-sim` - Basic headless simulation
- `headless-request-v2` - Pattern 2 (ChannelHeadlessRunner with EventBus)
- `multiplayer-pong` - QUIC network pattern (Pattern 3)

## License

MIT OR Apache-2.0
