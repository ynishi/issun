//! Junk Bot: Salvage Run
//! A roguelike TUI game built with ISSUN
//!
//! Demonstrates ISSUN's Scene system and UI widgets

mod assets;
mod models;
mod services;
mod systems;
// mod game; // Removed - not needed with SceneDirector
mod ui;

use issun::engine::GameRunner;
use issun::prelude::*;
use issun::ui::Tui;
use models::{handle_scene_input, GameContext, GameScene, PingPongMessageDeck};
use services::PingPongLogService;
use std::time::Duration;
use systems::ping_pong::PingPongSystem;

const TICK_RATE: Duration = Duration::from_millis(33); // 30 FPS

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize terminal
    let mut tui = Tui::new()?;

    // Initialize ISSUN framework with plugins
    let game = GameBuilder::new()
        .with_plugin(TurnBasedCombatPlugin::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
        .with_plugin(InventoryPlugin::new())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
        .with_plugin(LootPlugin::new())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
        .with_service(PingPongLogService::default())
        .with_system(PingPongSystem::default())
        .build()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    // Destructure game to obtain contexts
    let Game {
        mut resources,
        services,
        systems,
        ..
    } = game;

    // Insert runtime game state resource
    resources.insert(GameContext::new());
    resources.insert(PingPongMessageDeck::from_assets(
        assets::PING_PONG_NORMAL_LINES,
        assets::PING_PONG_CONGRATS_LINES,
    ));

    // Initialize SceneDirector with initial scene
    let initial_scene = GameScene::Title(models::scenes::TitleSceneData::new());
    let runner =
        GameRunner::new(SceneDirector::new(initial_scene, services, systems, resources).await)
            .with_tick_rate(TICK_RATE);

    let result = runner
        .run(
            &mut tui,
            |frame, scene, _resources| {
                render_scene(frame, scene);
            },
            |scene, services, systems, resources, input| {
                Box::pin(handle_scene_input(
                    scene, services, systems, resources, input,
                ))
            },
        )
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));

    // Cleanup
    tui.restore()?;
    result
}

/// Render the current scene
fn render_scene(frame: &mut ratatui::Frame, scene: &GameScene) {
    match scene {
        GameScene::Title(data) => ui::render_title(frame, data),
        GameScene::Ping(data) => ui::render_ping(frame, data),
        GameScene::Pong(data) => ui::render_pong(frame, data),
    }
}
