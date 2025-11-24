//! Garden Simulator - Demonstration of GenerationPlugin + EntropyPlugin
//!
//! A simple garden management game where plants grow (GenerationPlugin)
//! and decay (EntropyPlugin) based on environmental conditions.
//!
//! Now with Scene-based TUI using Ratatui!

mod garden;
mod hooks;
mod models;
mod scene;
mod ui;

use garden::Garden;
use issun::engine::GameRunner;
use issun::prelude::*;
use issun::ui::Tui;
use models::PlantSpecies;
use scene::{GameScene, SimulationSceneData};
use std::time::Duration;

const TICK_RATE: Duration = Duration::from_millis(200); // 5 FPS for simulation

#[tokio::main]
async fn main() -> std::io::Result<()> {
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

    // Create and setup garden
    let mut garden = Garden::new();
    garden.plant_seed(PlantSpecies::Tomato);
    garden.plant_seed(PlantSpecies::Lettuce);
    garden.plant_seed(PlantSpecies::Carrot);
    garden.plant_seed(PlantSpecies::Wheat);
    garden.plant_seed(PlantSpecies::Sunflower);

    // Insert garden as resource
    resources.insert(garden);

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
            if let Some(garden) = resources.try_get::<Garden>() {
                ui::render_simulation(frame, data, &garden);
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
                if let Some(mut garden) = resources.try_get_mut::<Garden>() {
                    garden.update_tick(1.0).await;
                }
            }

            // Handle input
            data.handle_input(services, systems, resources, input)
                .await
        }
    }
}
