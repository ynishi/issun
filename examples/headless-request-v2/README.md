# Headless Request Example (Pattern 2)

Demonstrates **Pattern 2**: HTTP API controlling a headless ISSUN simulation with **ChannelHeadlessRunner** and **EventBus integration**.

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
ChannelHeadlessRunner (tokio::select!)
    ↓
EventBus::publish(cmd) + dispatch()
    ↓
Scene::on_update()
    ↓
EventBus::reader<ApiCommand>()
    ↓
SimulationCounter (updates via event processing)
```

## Key Features

- ✅ **ChannelHeadlessRunner** - Runner owns the command channel
- ✅ **tokio::select!** - Immediate command processing (<1ms latency)
- ✅ **EventBus integration** - Commands published as events
- ✅ **Reusable Scene** - Any Scene can subscribe to command events
- ✅ **Standard ISSUN pattern** - Follows EventBus best practices

## Pattern 2 vs Pattern 1

| Aspect | **Pattern 1** | **Pattern 2** (This) |
|--------|--------------|---------------------|
| **Channel Owner** | Scene | Runner |
| **Command Processing** | Polling (`try_recv()`) | Event-driven (`tokio::select!`) |
| **Latency** | ~25ms average (50ms tick / 2) | <1ms (immediate) |
| **Integration** | Direct resource updates | EventBus events |
| **Reusability** | Scene-specific | Any Scene can subscribe |
| **Complexity** | ⭐⭐ Simple | ⭐⭐⭐ Medium |
| **Standard Pattern** | No | ✅ Yes (EventBus) |

## Usage

### 1. Start the simulation

```bash
cargo run --release
```

Output:
```
🚀 Starting headless simulation with HTTP API (Pattern 2)...
   Listen on: http://localhost:3000

Pattern 2 Features:
  ✅ ChannelHeadlessRunner with tokio::select!
  ✅ Commands published to EventBus immediately
  ✅ Scene subscribes via EventBus reader (reusable)
  ✅ Lower latency: <1ms vs ~25ms polling

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

# Check status (TODO: Currently returns placeholder)
curl http://localhost:3000/status
```

### 3. Observe the logs

```
[Tick  30] 📨✅ EventBus Increment -> Counter: 1
[Tick  51] 📨✅ EventBus Increment -> Counter: 2
[Tick  71] 📨🔄 EventBus Reset -> Counter: 0
[Tick 100] Counter: 0
```

## Implementation Details

### Command Flow (Pattern 2)

1. **HTTP Handler** receives request
2. Sends `ApiCommand` through channel
3. **ChannelHeadlessRunner** wakes up immediately via `tokio::select!`
4. Publishes command to **EventBus** and dispatches immediately
5. **Scene::on_update()** reads from EventBus reader
6. Updates `SimulationCounter` resource
7. Logs to stdout

### Key Code: ChannelHeadlessRunner Usage

```rust
// Create command channel
let (command_tx, command_rx) = mpsc::channel::<ApiCommand>(100);

// Initialize EventBus (required for Pattern 2)
game.resources.insert(EventBus::new());

// Create runner with command channel
let runner = HeadlessRunner::new(director)
    .with_tick_rate(Duration::from_millis(50))
    .with_command_channel(command_rx); // 🆕 Pattern 2

runner.run().await?;
```

### Key Code: Scene Event Processing

```rust
#[async_trait]
impl Scene for EventDrivenSimulation {
    async fn on_update(..., resources: &mut ResourceContext) -> SceneTransition<Self> {
        // Process commands from EventBus (standard ISSUN pattern)
        if let Some(mut event_bus) = resources.get_mut::<EventBus>().await {
            let reader = event_bus.reader::<ApiCommand>();

            for cmd in reader.iter() {
                match cmd {
                    ApiCommand::Increment => counter.increment(),
                    ApiCommand::Reset => counter.reset(),
                }
            }
        }

        SceneTransition::Stay
    }
}
```

## Why Pattern 2?

### ✅ Advantages

1. **Reusability**: Any Scene can subscribe to command events
   - Pattern 1: Scene owns the channel (tight coupling)
   - Pattern 2: Scene subscribes via EventBus (loose coupling)

2. **Lower Latency**: Immediate command processing
   - Pattern 1: ~25ms average (polling every tick)
   - Pattern 2: <1ms (tokio::select! wakes up immediately)

3. **Standard Pattern**: Follows ISSUN's EventBus architecture
   - Pattern 1: Direct resource updates (bypasses EventBus)
   - Pattern 2: Commands as events (consistent with other systems)

4. **Extensibility**: Easy to add multiple command types
   - Multiple command channels can publish to the same EventBus
   - Scenes can subscribe to specific command types

### ⚠️ Trade-offs

1. **Complexity**: Requires understanding of EventBus
2. **Boilerplate**: Need to implement `Event` trait for commands
3. **EventBus Dependency**: Must initialize EventBus in resources

## Production Improvements

### 1. Real-time Status Query

Add a query channel for immediate state reads:

```rust
enum SimCommand {
    Increment,
    Reset,
    Query(oneshot::Sender<StatusResponse>), // 🆕
}
```

### 2. Multiple Command Types

Different channels for different command categories:

```rust
let runner = HeadlessRunner::new(director)
    .with_command_channel(game_cmd_rx)
    .with_query_channel(query_rx)      // 🆕
    .with_admin_channel(admin_cmd_rx); // 🆕
```

### 3. Graceful Shutdown

Add shutdown signal handling:

```rust
tokio::select! {
    _ = runner.run() => {},
    _ = tokio::signal::ctrl_c() => {
        println!("Shutting down...");
    }
}
```

## Comparison with Other Patterns

| Pattern | Channel Owner | Latency | Integration | Use Case |
|---------|---------------|---------|-------------|----------|
| **Pattern 1** | Scene | ~25ms | Direct | Simple single-use |
| **Pattern 2** (This) | Runner | <1ms | EventBus | Reusable/Extensible |
| Pattern 3 (QUIC) | Network | Network | QUIC | Distributed systems |

## Related Examples

- `headless-sim` - Basic headless simulation
- `headless-request` - Pattern 1 (simple polling approach)
- `multiplayer-pong` - QUIC network pattern (Pattern 3)

## When to Use Pattern 2

Use Pattern 2 when:
- ✅ You need low-latency command processing (<1ms)
- ✅ Multiple scenes might need to process the same commands
- ✅ You want to follow ISSUN's standard EventBus pattern
- ✅ You plan to extend with more command types

Use Pattern 1 when:
- ✅ Simplicity is more important than latency
- ✅ Commands are specific to one Scene
- ✅ 25ms latency is acceptable for your use case

## License

MIT OR Apache-2.0
