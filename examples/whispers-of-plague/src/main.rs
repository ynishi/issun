mod hooks;
mod models;
mod plugins;
mod services;
mod systems;
mod ui;

use hooks::PlagueRumorHook;
use issun::engine::GameRunner;
use issun::event::EventBus;
use issun::prelude::*;
use models::{handle_scene_input, CityMap, GameScene, PlagueGameContext, Virus};
use plugins::rumor::{RumorRegistry, RumorState};
use plugins::{PlagueGamePlugin, RumorPlugin};
use std::sync::Arc;
use std::time::Duration;

const TICK_RATE: Duration = Duration::from_millis(120);

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut tui = issun::ui::Tui::new()?;

    // Build game with plugins
    let builder = GameBuilder::new()
        .with_plugin(PlagueGamePlugin::default())
        .map_err(as_io)?
        .with_plugin(RumorPlugin::new().with_hook(Arc::new(PlagueRumorHook)))
        .map_err(as_io)?
        .with_resource(PlagueGameContext::new())
        .with_resource(CityMap::new())
        .with_resource(Virus::initial());

    let Game {
        mut resources,
        services,
        systems,
        ..
    } = builder
        .build()
        .await
        .map_err(|err| std::io::Error::other(err.to_string()))?;

    // Ensure EventBus exists
    if !resources.contains::<EventBus>() {
        resources.insert(EventBus::new());
    }

    // Register Rumor resources manually
    if !resources.contains::<RumorRegistry>() {
        resources.insert(RumorRegistry::new());
    }
    if !resources.contains::<RumorState>() {
        resources.insert(RumorState::default());
    }

    // Create initial scene
    let initial_scene = GameScene::Title(models::TitleSceneData::new());

    // Create GameRunner
    let runner =
        GameRunner::new(SceneDirector::new(initial_scene, services, systems, resources).await)
            .with_tick_rate(TICK_RATE);

    // Run game loop
    let result = runner
        .run(
            &mut tui,
            |frame, scene, resources| {
                ui::render_scene(frame, scene, resources);
            },
            |scene, services, systems, resources, input| {
                Box::pin(handle_scene_input(
                    scene, services, systems, resources, input,
                ))
            },
        )
        .await
        .map_err(|err| std::io::Error::other(err.to_string()));

    tui.restore()?;
    result
}

fn as_io(err: issun::error::IssunError) -> std::io::Error {
    std::io::Error::other(err.to_string())
}
