//! Border Economy Prototype
//!
//! Scene-driven management sim that stitches together faction command,
//! tactical ops, and revenue management using ISSUN plugins.

mod events;
mod hooks;
mod models;
mod plugins;
pub mod ui;

use hooks::{BorderEconomyTerritoryHook, GameLogHook, PrototypeResearchHook};
use issun::engine::GameRunner;
use issun::event::EventBus;
use issun::plugin::action::{ActionConfig, ActionPlugin};
use issun::plugin::policy::{Policy, PolicyPlugin, PolicyRegistry};
use issun::plugin::research::ResearchPlugin;
use issun::plugin::territory::{Territory, TerritoryPlugin, TerritoryRegistry};
use issun::prelude::*;
use models::{handle_scene_input, GameContext, GameScene, DAILY_ACTION_POINTS};
use plugins::{
    EconomyState, FactionOpsState, FieldTelemetryService, MarketPulse, PrototypeBacklog,
    ReputationLedger, TerritoryStateCache, VaultState,
};
use std::time::Duration;

const TICK_RATE: Duration = Duration::from_millis(120);

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut tui = issun::ui::Tui::new()?;

    let builder = GameBuilder::new()
        // New issun built-in plugins (parallel demonstration)
        .with_plugin(issun::plugin::BuiltInTimePlugin::default())
        .map_err(as_io)?
        .with_plugin(
            ActionPlugin::new(ActionConfig {
                max_per_period: DAILY_ACTION_POINTS,
            })
            .with_hook(GameLogHook),
        )
        .map_err(as_io)?
        .with_plugin(
            TerritoryPlugin::new()
                .with_hook(BorderEconomyTerritoryHook),
        )
        .map_err(as_io)?
        .with_plugin(PolicyPlugin::new())
        .map_err(as_io)?
        .with_plugin(issun::plugin::BuiltInEconomyPlugin::default())
        .map_err(as_io)?
        .with_plugin(
            ResearchPlugin::new()
                .with_hook(PrototypeResearchHook),
        )
        .map_err(as_io)?
        // Existing border-economy plugins
        .with_plugin(plugins::FactionPlugin::default())
        .map_err(as_io)?
        .with_plugin(plugins::EconomyPlugin::default())
        .map_err(as_io)?
        .with_plugin(plugins::TerritoryPlugin::default())
        .map_err(as_io)?
        // WeaponPrototypePlugin migrated to ResearchPlugin (see above)
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

    // Initialize PrototypeBacklog for UI display
    if !resources.contains::<PrototypeBacklog>() {
        resources.insert(PrototypeBacklog::default());
    }

    // Initialize TerritoryRegistry from GameContext territories
    {
        let ctx = resources.get::<GameContext>().await.unwrap();
        let mut registry = resources.get_mut::<TerritoryRegistry>().await.unwrap();

        for intel in &ctx.territories {
            let territory = Territory::new(intel.id.as_str(), intel.id.as_str())
                .with_control(intel.control)
                .with_development(intel.development_level as u32);
            registry.add(territory);
        }
    }

    // Initialize PolicyRegistry from GameContext policies
    {
        let ctx = resources.get::<GameContext>().await.unwrap();
        let mut registry = resources.get_mut::<PolicyRegistry>().await.unwrap();

        // Convert PolicyCard to Policy
        for card in &ctx.policies {
            let policy = Policy::new(&card.id, &card.name, &card.description)
                .add_effect("dividend_multiplier", card.effects.dividend_multiplier)
                .add_effect("investment_bonus", card.effects.investment_bonus)
                .add_effect("ops_cost_multiplier", card.effects.ops_cost_multiplier)
                .add_effect("diplomacy_bonus", card.effects.diplomacy_bonus)
                .with_metadata(serde_json::json!({
                    "available_actions": card.available_actions,
                }));
            registry.add(policy);
        }

        // Activate the initial policy (first one)
        if !ctx.policies.is_empty() {
            let initial_policy_id = issun::plugin::policy::PolicyId::new(&ctx.policies[0].id);
            let _ = registry.activate(&initial_policy_id);
        }
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
    let clock_guard = resources.try_get::<issun::plugin::GameTimer>();
    let ledger_guard = resources.try_get::<issun::plugin::BudgetLedger>();
    let points_guard = resources.try_get::<issun::plugin::ActionPoints>();
    let ops_guard = resources.try_get::<FactionOpsState>();
    let econ_guard = resources.try_get::<EconomyState>();
    let territory_guard = resources.try_get::<TerritoryStateCache>();
    let proto_guard = resources.try_get::<PrototypeBacklog>();
    let market_guard = resources.try_get::<MarketPulse>();
    let reputation_guard = resources.try_get::<ReputationLedger>();
    let vault_guard = resources.try_get::<VaultState>();
    let policy_guard = resources.try_get::<PolicyRegistry>();

    let ctx = ctx_guard.as_deref();
    let clock = clock_guard.as_deref();
    let ledger = ledger_guard.as_deref();
    let points = points_guard.as_deref();
    let ops = ops_guard.as_deref();
    let econ = econ_guard.as_deref();
    let territory = territory_guard.as_deref();
    let proto = proto_guard.as_deref();
    let market = market_guard.as_deref();
    let reputation = reputation_guard.as_deref();
    let vault = vault_guard.as_deref();
    let policy_registry = policy_guard.as_deref();

    match scene {
        GameScene::Title(data) => ui::render_title(frame, data),
        GameScene::Strategy(data) => {
            ui::render_strategy(frame, ctx, clock, ledger, ops, territory, reputation, points, policy_registry, data)
        }
        GameScene::Tactical(data) => ui::render_tactical(frame, ctx, data),
        GameScene::Economic(data) => {
            ui::render_economic(frame, ctx, clock, ledger, econ, proto, market, policy_registry, data)
        }
        GameScene::IntelReport(data) => {
            ui::render_report(frame, ctx, clock, ledger, territory, proto, reputation, policy_registry, data)
        }
        GameScene::Vault(data) => ui::render_vault(frame, ctx, clock, ledger, vault, data),
    }
}

fn as_io(err: issun::error::IssunError) -> std::io::Error {
    std::io::Error::other(err.to_string())
}
