//! Border Economy Prototype
//!
//! Scene-driven management sim that stitches together faction command,
//! tactical ops, and revenue management using ISSUN plugins.

mod events;
mod models;
mod plugins;
pub mod ui;

use issun::engine::GameRunner;
use issun::event::EventBus;
use issun::prelude::*;
use models::{handle_scene_input, GameContext, GameScene};
use plugins::{
    EconomyState, FactionOpsState, MarketPulse, PrototypeBacklog, ReputationLedger,
    TerritoryStateCache, VaultState,
};
use std::time::Duration;

const TICK_RATE: Duration = Duration::from_millis(120);

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut tui = issun::ui::Tui::new()?;

    let builder = GameBuilder::new()
        .with_plugin(plugins::FactionPlugin::default())
        .map_err(as_io)?
        .with_plugin(plugins::EconomyPlugin::default())
        .map_err(as_io)?
        .with_plugin(plugins::TerritoryPlugin::default())
        .map_err(as_io)?
        .with_plugin(plugins::WeaponPrototypePlugin::default())
        .map_err(as_io)?
        .with_plugin(plugins::MarketSharePlugin::default())
        .map_err(as_io)?
        .with_plugin(plugins::ReputationPlugin::default())
        .map_err(as_io)?
        .with_plugin(plugins::VaultPlugin::default())
        .map_err(as_io)?
        .with_resource(GameContext::new());

    let Game {
        mut resources,
        services,
        systems,
        ..
    } = builder
        .build()
        .await
        .map_err(|err| std::io::Error::other(err.to_string()))?;

    if !resources.contains::<GameContext>() {
        resources.insert(GameContext::new());
    }

    if !resources.contains::<EventBus>() {
        resources.insert(EventBus::new());
    }

    let initial_scene = GameScene::Title(models::scenes::TitleSceneData::new());
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
        .map_err(|err| std::io::Error::other(err.to_string()));

    tui.restore()?;
    result
}

fn render_scene(frame: &mut ratatui::Frame, scene: &GameScene, resources: &ResourceContext) {
    let ctx_guard = resources.try_get::<GameContext>();
    let ops_guard = resources.try_get::<FactionOpsState>();
    let econ_guard = resources.try_get::<EconomyState>();
    let territory_guard = resources.try_get::<TerritoryStateCache>();
    let proto_guard = resources.try_get::<PrototypeBacklog>();
    let market_guard = resources.try_get::<MarketPulse>();
    let reputation_guard = resources.try_get::<ReputationLedger>();
    let vault_guard = resources.try_get::<VaultState>();

    let ctx = ctx_guard.as_deref();
    let ops = ops_guard.as_deref();
    let econ = econ_guard.as_deref();
    let territory = territory_guard.as_deref();
    let proto = proto_guard.as_deref();
    let market = market_guard.as_deref();
    let reputation = reputation_guard.as_deref();
    let vault = vault_guard.as_deref();

    match scene {
        GameScene::Title(data) => ui::render_title(frame, data),
        GameScene::Strategy(data) => {
            ui::render_strategy(frame, ctx, ops, territory, reputation, data)
        }
        GameScene::Tactical(data) => ui::render_tactical(frame, ctx, data),
        GameScene::Economic(data) => ui::render_economic(frame, ctx, econ, proto, market, data),
        GameScene::IntelReport(data) => {
            ui::render_report(frame, ctx, territory, proto, reputation, data)
        }
        GameScene::Vault(data) => ui::render_vault(frame, ctx, vault, data),
    }
}

fn as_io(err: issun::error::IssunError) -> std::io::Error {
    std::io::Error::other(err.to_string())
}
