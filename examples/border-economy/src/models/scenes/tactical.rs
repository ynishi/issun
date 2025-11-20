use crate::events::{FieldTestFeedback, MissionResolved};
use crate::models::scenes::EconomicSceneData;
use crate::models::{Currency, GameContext, GameScene};
use issun::auto_pump;
use issun::event::EventBus;
use issun::plugin::territory::TerritoryControlChangeRequested;
use issun::prelude::{ResourceContext, SceneTransition, ServiceContext, SystemContext};
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionBrief {
    pub faction: crate::models::FactionId,
    pub target: crate::models::TerritoryId,
    pub prototype: crate::models::WeaponPrototypeId,
    pub expected_payout: Currency,
    pub threat_level: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticalSceneData {
    pub brief: MissionBrief,
    pub progress: f32,
    pub resolved: bool,
}

impl TacticalSceneData {
    pub fn from_brief(brief: MissionBrief) -> Self {
        Self {
            brief,
            progress: 0.0,
            resolved: false,
        }
    }

    #[auto_pump]
    pub async fn handle_input(
        &mut self,
        services: &ServiceContext,
        systems: &mut SystemContext,
        resources: &mut ResourceContext,
        input: InputEvent,
    ) -> SceneTransition<GameScene> {
        let transition = match input {
            InputEvent::Select => {
                if self.resolved {
                    SceneTransition::Switch(GameScene::Economic(
                        EconomicSceneData::after_operation(self.brief.clone()),
                    ))
                } else {
                    self.resolve_operation(resources).await
                }
            }
            InputEvent::Cancel => SceneTransition::Switch(GameScene::Strategy(
                super::strategy::StrategySceneData::new(),
            )),
            _ => SceneTransition::Stay,
        };
        transition
    }

    async fn resolve_operation(
        &mut self,
        resources: &mut ResourceContext,
    ) -> SceneTransition<GameScene> {
        let odds = (1.0 - self.brief.threat_level).clamp(0.25, 0.9);
        self.progress = (self.progress + odds * 0.6).min(1.0);
        self.resolved = self.progress >= 0.95;

        let payout = if self.resolved {
            self.brief.expected_payout
        } else {
            Currency::new((self.brief.expected_payout.amount() as f32 * self.progress) as i64)
        };
        let territory = self.brief.target.clone();
        let faction = self.brief.faction.clone();
        let prototype = self.brief.prototype.clone();

        // Apply revenue to issun BudgetLedger
        if let Some(mut ledger) = resources.get_mut::<issun::plugin::BudgetLedger>().await {
            let reserve_bonus = Currency::new((payout.amount() as f32 * 0.25) as i64);
            *ledger.get_channel_mut(issun::plugin::BudgetChannel::Cash) =
                ledger.get_channel_mut(issun::plugin::BudgetChannel::Cash).saturating_add(payout);
            *ledger.get_channel_mut(issun::plugin::BudgetChannel::Reserve) =
                ledger.get_channel_mut(issun::plugin::BudgetChannel::Reserve).saturating_add(reserve_bonus);
        }

        // Request territory control change via TerritoryPlugin
        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(TerritoryControlChangeRequested {
                id: territory.as_str().into(),
                delta: self.progress * 0.15,
            });
        }

        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            // Game-specific updates (unrest, conflict_intensity, etc.) handled by Hook
            ctx.push_telemetry(&prototype, self.progress * 0.4);
            ctx.record(format!("{} 作戦が {} で進展", faction, territory));
        }

        let casualties = (100.0 * (1.0 - odds)) as u32;
        let share_gain = self.progress * 0.1;
        let resolved_event = MissionResolved {
            faction: faction.clone(),
            target: territory.clone(),
            casualties,
            secured_share: share_gain,
            revenue_delta: payout,
        };
        let feedback = FieldTestFeedback {
            prototype: prototype.clone(),
            effectiveness: self.progress,
            reliability: odds,
        };

        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(resolved_event);
            bus.publish(feedback.clone());
        }

        // Update PrototypeBacklog for UI display
        if let Some(mut backlog) = resources.get_mut::<crate::plugins::PrototypeBacklog>().await {
            backlog.field_reports.insert(
                0,
                format!(
                    "{} eff {:>3.0}% / rel {:>3.0}%",
                    prototype,
                    feedback.effectiveness * 100.0,
                    feedback.reliability * 100.0
                ),
            );
            backlog.field_reports.truncate(6);
        }

        SceneTransition::Stay
    }
}
