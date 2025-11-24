//! Headless simulation example
//!
//! Runs a simple simulation for a specified number of ticks without UI.
//! Demonstrates ISSUN's headless capabilities for server-side simulation.

use async_trait::async_trait;
use issun::context::{ResourceContext, ServiceContext, SystemContext};
use issun::engine::HeadlessRunner;
use issun::error::Result;
use issun::plugin::time::{BuiltInTimePlugin, GameTimer};
use issun::prelude::*;
use issun::scene::{Scene, SceneTransition};
use std::time::Duration;

/// Simple counter to track simulation state
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

    fn get(&self) -> u64 {
        self.value
    }
}

/// Simple simulation scene that tracks ticks and time
struct SimpleSimulation {
    tick: u64,
}

impl SimpleSimulation {
    fn new() -> Self {
        Self { tick: 0 }
    }
}

#[async_trait]
impl Scene for SimpleSimulation {
    async fn on_update(
        &mut self,
        _services: &ServiceContext,
        _systems: &mut SystemContext,
        resources: &mut ResourceContext,
    ) -> SceneTransition<Self> {
        self.tick += 1;

        // Update counter
        if let Some(mut counter) = resources.get_mut::<SimulationCounter>().await {
            counter.increment();
        }

        // Log progress every 100 ticks
        if self.tick % 100 == 0 {
            let day = if let Some(timer) = resources.get::<GameTimer>().await {
                timer.day
            } else {
                0
            };

            let count = if let Some(counter) = resources.get::<SimulationCounter>().await {
                counter.get()
            } else {
                0
            };

            println!(
                "[Tick {:>4}] Day: {:>3} | Counter: {}",
                self.tick, day, count
            );
        }

        SceneTransition::Stay
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting headless economy simulation...");
    println!("   This demo runs without UI, suitable for server deployment.");
    println!();

    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let max_ticks = if args.len() > 1 {
        args[1].parse::<u64>().unwrap_or(1000)
    } else {
        1000
    };

    let tick_rate_ms = if args.len() > 2 {
        args[2].parse::<u64>().unwrap_or(10)
    } else {
        10
    };

    println!("Configuration:");
    println!("  Max ticks: {}", max_ticks);
    println!("  Tick rate: {}ms", tick_rate_ms);
    println!();

    // Build game with time plugin
    let builder = GameBuilder::new()
        .with_plugin(BuiltInTimePlugin::default())
        .map_err(|e| issun::error::IssunError::Plugin(e.to_string()))?;

    let mut game = builder.build().await?;

    // Initialize simulation counter
    game.resources.insert(SimulationCounter::new());

    // Create scene director
    let director = SceneDirector::new(
        SimpleSimulation::new(),
        game.services,
        game.systems,
        game.resources,
    )
    .await;

    // Run headless
    let start = std::time::Instant::now();

    let runner = HeadlessRunner::new(director)
        .with_tick_rate(Duration::from_millis(tick_rate_ms))
        .with_max_ticks(max_ticks);

    runner.run().await?;

    let elapsed = start.elapsed();

    println!();
    println!("✅ Simulation completed!");
    println!("   Total time: {:.2}s", elapsed.as_secs_f64());
    println!(
        "   Tick rate: {:.2} ticks/sec",
        max_ticks as f64 / elapsed.as_secs_f64()
    );

    Ok(())
}
