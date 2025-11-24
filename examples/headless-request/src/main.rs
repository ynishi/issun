//! Headless simulation with HTTP API
//!
//! Demonstrates Pattern 1: HTTP requests -> Channel -> EventBus -> HeadlessRunner
//!
//! This example shows how to control a headless simulation via HTTP API.
//! External clients send HTTP requests, which are converted to commands
//! and sent through a channel to the simulation loop.
//!
//! Usage:
//!   cargo run -p headless-request
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
use tokio::sync::{mpsc, RwLock};

// Commands from HTTP API
#[derive(Clone, Debug, Serialize, Deserialize)]
enum ApiCommand {
    Increment,
    Reset,
}

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

// Scene with command channel
struct ApiControlledSimulation {
    tick: u64,
    command_rx: mpsc::Receiver<ApiCommand>,
}

impl ApiControlledSimulation {
    fn new(command_rx: mpsc::Receiver<ApiCommand>) -> Self {
        Self { tick: 0, command_rx }
    }
}

#[async_trait]
impl Scene for ApiControlledSimulation {
    async fn on_update(
        &mut self,
        _services: &ServiceContext,
        _systems: &mut SystemContext,
        resources: &mut ResourceContext,
    ) -> SceneTransition<Self> {
        self.tick += 1;

        // Process incoming HTTP commands directly
        // Note: For simplicity, we bypass EventBus and update counter directly
        while let Ok(cmd) = self.command_rx.try_recv() {
            if let Some(mut counter) = resources.get_mut::<SimulationCounter>().await {
                match cmd {
                    ApiCommand::Increment => {
                        counter.increment();
                        println!("[Tick {}] 📨✅ HTTP Increment -> Counter: {}", self.tick, counter.get());
                    }
                    ApiCommand::Reset => {
                        counter.reset();
                        println!("[Tick {}] 📨🔄 HTTP Reset -> Counter: 0", self.tick);
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
    counter: Arc<RwLock<u64>>, // Shared counter for status endpoint
}

// HTTP handlers
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

async fn handle_status(State(state): State<Arc<AppState>>) -> Json<StatusResponse> {
    // TODO: In production, implement a query channel to get real-time state
    // For now, returns cached value which won't update
    let counter = *state.counter.read().await;

    Json(StatusResponse {
        tick: 0,
        counter,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting headless simulation with HTTP API...");
    println!("   Listen on: http://localhost:3000");
    println!();
    println!("Available endpoints:");
    println!("  POST /increment - Increment the counter");
    println!("  POST /reset     - Reset the counter");
    println!("  GET  /status    - Get current status");
    println!();

    // Create command channel for HTTP -> Simulation communication
    let (command_tx, command_rx) = mpsc::channel::<ApiCommand>(100);

    // Shared counter for status endpoint
    let shared_counter = Arc::new(RwLock::new(0u64));

    // Build game
    let builder = GameBuilder::new()
        .with_plugin(BuiltInTimePlugin::default())
        .map_err(|e| issun::error::IssunError::Plugin(e.to_string()))?;

    let mut game = builder.build().await?;

    // Initialize resources
    game.resources.insert(SimulationCounter::new());
    game.resources.insert(EventBus::new());

    // Create director with command channel
    let director = SceneDirector::new(
        ApiControlledSimulation::new(command_rx),
        game.services,
        game.systems,
        game.resources,
    )
    .await;

    // Start HTTP server
    let app_state = Arc::new(AppState {
        command_tx,
        counter: shared_counter,
    });

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

    // Note: In production, you'd implement a proper state synchronization
    // For now, the /status endpoint returns a cached value
    // TODO: Add query channel for real-time status

    // Run headless simulation
    let runner = HeadlessRunner::new(director)
        .with_tick_rate(Duration::from_millis(50));

    // Note: In production, you'd want to handle graceful shutdown
    runner.run().await?;

    Ok(())
}
