//! Garden Simulator - Demonstration of GenerationPlugin + EntropyPlugin
//!
//! A simple garden management game where plants grow (GenerationPlugin)
//! and decay (EntropyPlugin) based on environmental conditions.
//!
//! Now with Scene-based TUI using Ratatui!
//!
//! Usage: garden-sim [plant_count]
//!   plant_count: Number of plants to simulate (default: 5, max: 100)

mod event_log;
mod garden;
mod hooks;
mod models;
mod scene;
mod ui;

use event_log::EventLog;
use garden::Garden;
use issun::engine::GameRunner;
use issun::prelude::*;
use issun::ui::Tui;
use models::PlantSpecies;
use scene::{GameScene, SimulationSceneData};
use std::env;
use std::time::Duration;

const TICK_RATE: Duration = Duration::from_millis(200); // 5 FPS for simulation
const DEFAULT_PLANT_COUNT: usize = 5;
const MAX_PLANT_COUNT: usize = 100;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    let plant_count = if args.len() > 1 {
        args[1]
            .parse::<usize>()
            .unwrap_or(DEFAULT_PLANT_COUNT)
            .min(MAX_PLANT_COUNT)
    } else {
        DEFAULT_PLANT_COUNT
    };

    // Initialize terminal
    let mut tui = Tui::new()?;

    // Initialize ISSUN framework
    let game = GameBuilder::new()
        .build()
        .await
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    // Destructure game to obtain contexts
    let Game {
        mut resources,
        services,
        systems,
        ..
    } = game;

    // Create and setup garden with specified plant count
    let mut garden = Garden::new();
    let plant_species = [
        PlantSpecies::Tomato,
        PlantSpecies::Lettuce,
        PlantSpecies::Carrot,
        PlantSpecies::Wheat,
        PlantSpecies::Sunflower,
    ];

    for i in 0..plant_count {
        let species = plant_species[i % plant_species.len()].clone();
        garden.plant_seed(species);
    }

    // Insert resources
    resources.insert(garden);
    resources.insert(EventLog::new());

    // Initialize SceneDirector with initial scene
    let initial_scene = GameScene::Simulation(SimulationSceneData::new());
    let runner =
        GameRunner::new(SceneDirector::new(initial_scene, services, systems, resources).await)
            .with_tick_rate(TICK_RATE);

    let result = runner
        .run(
            &mut tui,
            |frame, scene, resources| {
                render_scene(frame, scene, resources);
            },
            |scene, services, systems, resources, input| {
                Box::pin(handle_scene_input(
                    scene, services, systems, resources, input,
                ))
            },
        )
        .await
        .map_err(|e| std::io::Error::other(e.to_string()));

    // Cleanup
    tui.restore()?;
    result
}

/// Render the current scene
fn render_scene(frame: &mut ratatui::Frame, scene: &GameScene, resources: &ResourceContext) {
    match scene {
        GameScene::Simulation(data) => {
            if let (Some(garden), Some(event_log)) = (
                resources.try_get::<Garden>(),
                resources.try_get::<EventLog>(),
            ) {
                ui::render_simulation(frame, data, &garden, &event_log);
            }
        }
    }
}

/// Handle scene input
async fn handle_scene_input(
    scene: &mut GameScene,
    services: &ServiceContext,
    systems: &mut SystemContext,
    resources: &mut ResourceContext,
    input: issun::ui::InputEvent,
) -> SceneTransition<GameScene> {
    match scene {
        GameScene::Simulation(data) => {
            // Update tick count
            data.tick_count += 1;

            // Update garden if not paused
            if !data.paused {
                // Need to get both mutable references
                if let (Some(mut garden), Some(mut event_log)) = (
                    resources.try_get_mut::<Garden>(),
                    resources.try_get_mut::<EventLog>(),
                ) {
                    garden.update_tick(1.0, &mut event_log).await;
                }
            }

            // Handle input
            data.handle_input(services, systems, resources, input).await
        }
    }
}
