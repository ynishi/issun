mod hooks;
mod models;
mod plugins;
mod services;
mod systems;
mod ui;

use hooks::PlagueContagionHook;
use issun::engine::GameRunner;
use issun::event::EventBus;
use issun::plugin::contagion::{
    Contagion, ContagionConfig, ContagionContent, ContagionPlugin, ContagionState, DiseaseLevel,
};
use issun::plugin::time::TurnBasedTimePlugin;
use issun::prelude::*;
use models::{build_city_topology, handle_scene_input, CityMap, GameScene, PlagueGameContext};
use plugins::WinConditionPlugin;
use std::time::Duration;

const TICK_RATE: Duration = Duration::from_millis(120);

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut tui = issun::ui::Tui::new()?;

    // Build city topology (districts + routes)
    let topology = build_city_topology();

    // Configure contagion plugin
    let contagion_config = ContagionConfig::default()
        .with_propagation_rate(0.7) // 70% base propagation chance
        .with_mutation_rate(0.15) // 15% chance of mutation per spread
        .with_lifetime_turns(20); // Contagions last 20 turns

    // Build game with plugins (80% ISSUN, 20% custom)
    let builder = GameBuilder::new()
        // ISSUN built-in plugins
        .with_plugin(TurnBasedTimePlugin::new(1, 10)) // 10 turns max
        .map_err(as_io)?
        .with_plugin(
            ContagionPlugin::new()
                .with_topology(topology)
                .with_config(contagion_config)
                .with_hook(PlagueContagionHook),
        )
        .map_err(as_io)?
        // Custom plugin (only win condition logic)
        .with_plugin(WinConditionPlugin::new())
        .map_err(as_io)?
        // Game resources
        .with_resource(PlagueGameContext::new())
        .with_resource(CityMap::new());

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

    // Note: ContagionPlugin now correctly registers all resources via #[plugin(...)] attributes
    // No manual workarounds needed anymore!

    // Register initial contagions
    {
        let mut contagion_state = resources
            .get_mut::<ContagionState>()
            .await
            .expect("ContagionState should exist");

        // Initial disease outbreak in downtown
        contagion_state.spawn_contagion(Contagion::new(
            "alpha_virus",
            ContagionContent::Disease {
                severity: DiseaseLevel::Mild,
                location: "downtown".to_string(),
            },
            "downtown", // Start in downtown
            0,          // Turn 0
        ));
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
