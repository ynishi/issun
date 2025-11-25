//! Headless simulation with HTTP API (Pattern 2)
//!
//! Demonstrates Pattern 2: HTTP requests -> Channel -> ChannelHeadlessRunner -> EventBus -> Scene
//!
//! This example shows the event-driven approach using ChannelHeadlessRunner with tokio::select!
//! for immediate command processing (<1ms latency) via EventBus integration.
//!
//! Key differences from Pattern 1:
//! - ✅ Runner owns the command channel (not Scene)
//! - ✅ tokio::select! for immediate command processing
//! - ✅ Commands published to EventBus (standard ISSUN pattern)
//! - ✅ Scene subscribes via EventBus reader (reusable)
//! - ✅ Lower latency: <1ms vs ~25ms polling
//!
//! Usage:
//!   cargo run -p headless-request-v2
//!
//! Then in another terminal:
//!   curl -X POST http://localhost:3000/increment
//!   curl -X POST http://localhost:3000/reset
//!   curl http://localhost:3000/status

use async_trait::async_trait;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use issun::context::{ResourceContext, ServiceContext, SystemContext};
use issun::engine::HeadlessRunner;
use issun::error::Result;
use issun::event::{Event, EventBus};
use issun::plugin::time::BuiltInTimePlugin;
use issun::prelude::*;
use issun::scene::{Scene, SceneTransition};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

// Commands from HTTP API (implements Event trait for EventBus integration)
#[derive(Clone, Debug, Serialize, Deserialize)]
enum ApiCommand {
    Increment,
    Reset,
}

// Event trait implementation is required for EventBus integration
impl Event for ApiCommand {}

// Simulation state
#[derive(Clone)]
struct SimulationCounter {
    value: u64,
}

impl SimulationCounter {
    fn new() -> Self {
        Self { value: 0 }
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    fn reset(&mut self) {
        self.value = 0;
    }

    fn get(&self) -> u64 {
        self.value
    }
}

// Scene that processes commands via EventBus (Pattern 2)
struct EventDrivenSimulation {
    tick: u64,
}

impl EventDrivenSimulation {
    fn new() -> Self {
        Self { tick: 0 }
    }
}

#[async_trait]
impl Scene for EventDrivenSimulation {
    async fn on_update(
        &mut self,
        _services: &ServiceContext,
        _systems: &mut SystemContext,
        resources: &mut ResourceContext,
    ) -> SceneTransition<Self> {
        self.tick += 1;

        // Process commands from EventBus (standard ISSUN pattern)
        if let Some(mut event_bus) = resources.get_mut::<EventBus>().await {
            let reader = event_bus.reader::<ApiCommand>();

            for cmd in reader.iter() {
                // Get mutable counter to process command
                if let Some(mut counter) = resources.get_mut::<SimulationCounter>().await {
                    match cmd {
                        ApiCommand::Increment => {
                            counter.increment();
                            println!(
                                "[Tick {}] 📨✅ EventBus Increment -> Counter: {}",
                                self.tick,
                                counter.get()
                            );
                        }
                        ApiCommand::Reset => {
                            counter.reset();
                            println!("[Tick {}] 📨🔄 EventBus Reset -> Counter: 0", self.tick);
                        }
                    }
                }
            }
        }

        // Log every 100 ticks
        if self.tick % 100 == 0 {
            if let Some(counter) = resources.get::<SimulationCounter>().await {
                println!("[Tick {:>4}] Counter: {}", self.tick, counter.get());
            }
        }

        SceneTransition::Stay
    }
}

// HTTP API response types
#[derive(Serialize)]
struct StatusResponse {
    tick: u64,
    counter: u64,
}

// Shared state for HTTP handlers
struct AppState {
    command_tx: mpsc::Sender<ApiCommand>,
}

// HTTP handlers (same as Pattern 1)
async fn handle_increment(State(state): State<Arc<AppState>>) -> StatusCode {
    match state.command_tx.send(ApiCommand::Increment).await {
        Ok(_) => {
            println!("🌐 HTTP: Increment request queued");
            StatusCode::OK
        }
        Err(_) => {
            eprintln!("❌ HTTP: Failed to send command (channel closed)");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn handle_reset(State(state): State<Arc<AppState>>) -> StatusCode {
    match state.command_tx.send(ApiCommand::Reset).await {
        Ok(_) => {
            println!("🌐 HTTP: Reset request queued");
            StatusCode::OK
        }
        Err(_) => {
            eprintln!("❌ HTTP: Failed to send command (channel closed)");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn handle_status(State(_state): State<Arc<AppState>>) -> Json<StatusResponse> {
    // TODO: In production, implement a query channel to get real-time state
    // For now, returns placeholder values
    Json(StatusResponse { tick: 0, counter: 0 })
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting headless simulation with HTTP API (Pattern 2)...");
    println!("   Listen on: http://localhost:3000");
    println!();
    println!("Pattern 2 Features:");
    println!("  ✅ ChannelHeadlessRunner with tokio::select!");
    println!("  ✅ Commands published to EventBus immediately");
    println!("  ✅ Scene subscribes via EventBus reader (reusable)");
    println!("  ✅ Lower latency: <1ms vs ~25ms polling");
    println!();
    println!("Available endpoints:");
    println!("  POST /increment - Increment the counter");
    println!("  POST /reset     - Reset the counter");
    println!("  GET  /status    - Get current status");
    println!();

    // Create command channel for HTTP -> Simulation communication
    let (command_tx, command_rx) = mpsc::channel::<ApiCommand>(100);

    // Build game
    let builder = GameBuilder::new()
        .with_plugin(BuiltInTimePlugin::default())
        .map_err(|e| issun::error::IssunError::Plugin(e.to_string()))?;

    let mut game = builder.build().await?;

    // Initialize resources
    game.resources.insert(SimulationCounter::new());
    game.resources.insert(EventBus::new()); // EventBus is required for Pattern 2

    // Create director
    let director = SceneDirector::new(
        EventDrivenSimulation::new(),
        game.services,
        game.systems,
        game.resources,
    )
    .await;

    // Start HTTP server
    let app_state = Arc::new(AppState { command_tx });

    let app = Router::new()
        .route("/increment", post(handle_increment))
        .route("/reset", post(handle_reset))
        .route("/status", get(handle_status))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("✅ HTTP API started");
    println!();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Run headless simulation with command channel (Pattern 2)
    // ChannelHeadlessRunner uses tokio::select! to process commands immediately
    let runner = HeadlessRunner::new(director)
        .with_tick_rate(Duration::from_millis(50))
        .with_command_channel(command_rx); // 🆕 Pattern 2: Runner owns the channel

    // Commands are published to EventBus immediately upon receipt
    // Scene processes them via EventBus reader in on_update()
    runner.run().await?;

    Ok(())
}
