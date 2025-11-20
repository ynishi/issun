//! Custom hooks for border-economy

use async_trait::async_trait;
use issun::context::ResourceContext;
use issun::plugin::action::{ActionConsumed, ActionHook};
use issun::plugin::territory::{ControlChanged, Developed, Territory, TerritoryHook};

use crate::models::GameContext;

/// Hook that logs actions to GameContext
pub struct GameLogHook;

#[async_trait]
impl ActionHook for GameLogHook {
    async fn on_action_consumed(
        &self,
        consumed: &ActionConsumed,
        resources: &mut ResourceContext,
    ) {
        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            if !consumed.context.is_empty() {
                ctx.record(format!(
                    "{}を実施 (残り{}回)",
                    consumed.context,
                    consumed.remaining
                ));
            }

            if consumed.depleted {
                ctx.record("日次行動をすべて消費しました。");
            }
        }
    }

    async fn on_actions_depleted(&self, _resources: &mut ResourceContext) -> bool {
        // Always allow auto-advance when depleted
        true
    }

    async fn on_actions_reset(&self, new_count: u32, resources: &mut ResourceContext) {
        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            ctx.record(format!("新しい日が始まりました。行動ポイント: {}", new_count));
        }
    }
}

/// Hook that logs territory changes to GameContext
pub struct BorderEconomyTerritoryHook;

#[async_trait]
impl TerritoryHook for BorderEconomyTerritoryHook {
    async fn on_control_changed(
        &self,
        territory: &Territory,
        change: &ControlChanged,
        resources: &mut ResourceContext,
    ) {
        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            // Update corresponding TerritoryIntel (game-specific fields)
            if let Some(intel) = ctx.territories.iter_mut().find(|t| t.id.as_str() == territory.id.as_str()) {
                intel.control = change.new_control;
                intel.unrest = (intel.unrest - change.delta).clamp(0.0, 1.0);
                intel.enemy_share = (1.0 - change.new_control).clamp(0.0, 1.0);
                intel.conflict_intensity = (intel.conflict_intensity * 0.85).clamp(0.1, 1.0);

                if intel.enemy_share < 0.2 {
                    intel.battlefront = false;
                }
            }

            // Log the change
            if change.delta.abs() > 0.01 {
                ctx.record(format!(
                    "{} 支配率: {:.0}% → {:.0}%",
                    territory.name,
                    change.old_control * 100.0,
                    change.new_control * 100.0
                ));
            }
        }
    }

    async fn calculate_development_cost(
        &self,
        territory: &Territory,
        resources: &ResourceContext,
    ) -> Result<i64, String> {
        // Get policy bonus
        let bonus = if let Some(ctx) = resources.get::<GameContext>().await {
            ctx.active_policy().effects.investment_bonus
        } else {
            1.0
        };

        let base_cost = 100 * (territory.development_level + 1);
        let final_cost = (base_cost as f32 / bonus) as i64;
        Ok(final_cost)
    }

    async fn on_developed(
        &self,
        territory: &Territory,
        developed: &Developed,
        resources: &mut ResourceContext,
    ) {
        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            // Update corresponding TerritoryIntel.development_level
            if let Some(intel) = ctx.territories.iter_mut().find(|t| t.id.as_str() == territory.id.as_str()) {
                intel.development_level = developed.new_level as f32;
            }

            // Log development
            ctx.record(format!(
                "{} 開発レベル {} → {}",
                territory.name,
                developed.old_level,
                developed.new_level
            ));
        }
    }
}
