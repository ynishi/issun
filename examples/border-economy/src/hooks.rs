//! Custom hooks for border-economy

use async_trait::async_trait;
use issun::context::ResourceContext;
use issun::plugin::action::{ActionConsumed, ActionHook};

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
