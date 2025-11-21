use crate::events::{MissionRequested, ResearchQueued};
use crate::models::scenes::{
    EconomicSceneData, IntelReportSceneData, TacticalSceneData, VaultSceneData,
};
use crate::models::{Currency, TerritoryId};
use crate::models::{DemandProfile, GameContext, GameScene, WeaponPrototypeState};
use crate::plugins::EconomyState;
use issun::auto_pump;
use issun::event::EventBus;
use issun::plugin::action::{ActionConsumedEvent, ActionError, ActionPoints};
use issun::prelude::{ResourceContext, SceneTransition, ServiceContext, SystemContext};
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyAction {
    DeployOperation,
    FundResearch,
    InspectIntel,
    ManageBudget,
    InvestDevelopment,
    DiplomaticAction,
    SetPolicy,
    FortifyFront,
    ManageVaults,
    EndDay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySceneData {
    pub cursor: usize,
    pub status_line: String,
    pub actions: Vec<StrategyAction>,
}

impl StrategySceneData {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            status_line: "作戦を選択".into(),
            actions: vec![
                StrategyAction::DeployOperation,
                StrategyAction::FundResearch,
                StrategyAction::InspectIntel,
                StrategyAction::ManageBudget,
                StrategyAction::InvestDevelopment,
                StrategyAction::DiplomaticAction,
                StrategyAction::SetPolicy,
                StrategyAction::FortifyFront,
                StrategyAction::ManageVaults,
                StrategyAction::EndDay,
            ],
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
            InputEvent::Up => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                SceneTransition::Stay
            }
            InputEvent::Down => {
                if self.cursor + 1 < self.actions.len() {
                    self.cursor += 1;
                }
                SceneTransition::Stay
            }
            InputEvent::Cancel => {
                SceneTransition::Switch(GameScene::Title(super::title::TitleSceneData::new()))
            }
            InputEvent::Select => {
                let action = self.actions[self.cursor].clone();
                match action {
                    StrategyAction::DeployOperation => self.launch_operation(resources).await,
                    StrategyAction::FundResearch => self.allocate_research(resources).await,
                    StrategyAction::InspectIntel => self.open_report(resources).await,
                    StrategyAction::ManageBudget => {
                        SceneTransition::Switch(GameScene::Economic(EconomicSceneData::new()))
                    }
                    StrategyAction::InvestDevelopment => {
                        self.invest_in_development(resources).await;
                        SceneTransition::Stay
                    }
                    StrategyAction::DiplomaticAction => {
                        self.execute_diplomacy(resources).await;
                        SceneTransition::Stay
                    }
                    StrategyAction::SetPolicy => {
                        self.set_policy(resources).await;
                        SceneTransition::Stay
                    }
                    StrategyAction::FortifyFront => {
                        self.fortify_battlefront(resources).await;
                        SceneTransition::Stay
                    }
                    StrategyAction::ManageVaults => {
                        SceneTransition::Switch(GameScene::Vault(VaultSceneData::new()))
                    }
                    StrategyAction::EndDay => {
                        self.end_day_now(resources).await;
                        SceneTransition::Stay
                    }
                }
            }
            _ => SceneTransition::Stay,
        };
        transition
    }

    async fn launch_operation(
        &mut self,
        resources: &mut ResourceContext,
    ) -> SceneTransition<GameScene> {
        let ops_multiplier = issun::plugin::policy::PolicyService::get_active_effect(
            "ops_cost_multiplier",
            resources,
        )
        .await;

        let deployment_cost = Currency::new(((150.0 * ops_multiplier).round() as i64).max(80));

        // Spend from issun BudgetLedger
        let spend_ok = if let Some(mut ledger) = resources.get_mut::<issun::plugin::BudgetLedger>().await {
            ledger.try_spend(issun::plugin::BudgetChannel::Ops, deployment_cost)
        } else {
            return SceneTransition::Stay;
        };

        if !spend_ok {
            self.status_line = "作戦資金が不足しています".into();
            push_econ_warning(resources, "Ops資金不足: 作戦が延期されました").await;
            return SceneTransition::Stay;
        }

        let mut ctx = match resources.get_mut::<GameContext>().await {
            Some(ctx) => ctx,
            None => return SceneTransition::Stay,
        };

        let faction = match ctx.pick_ready_faction() {
            Some(faction) => faction,
            None => {
                self.status_line = "待機中の部隊なし".into();
                return SceneTransition::Stay;
            }
        };
        ctx.record_ops_spend(deployment_cost);
        let territory = ctx
            .territories
            .iter()
            .max_by(|a, b| a.unrest.partial_cmp(&b.unrest).unwrap())
            .cloned()
            .unwrap_or_else(|| ctx.territories[0].clone());
        let prototype = ctx.prototypes[0].clone();

        let expected_payout =
            Currency::new(((territory.unrest + faction.readiness as f32 / 100.0) * 500.0) as i64);
        let brief = super::tactical::MissionBrief {
            faction: faction.id.clone(),
            target: territory.id.clone(),
            prototype: prototype.id.clone(),
            expected_payout,
            threat_level: territory.unrest,
        };

        ctx.mark_faction_deployed(&brief.faction);
        ctx.record(format!(
            "{} が {} に展開",
            faction.codename,
            territory.id.as_str()
        ));

        drop(ctx);

        // Consume action using ActionPlugin
        if let Some(mut points) = resources.get_mut::<ActionPoints>().await {
            match points.consume_with("作戦展開") {
                Ok(consumed) => {
                    // Publish ActionConsumedEvent for systems to react
                    if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                        bus.publish(ActionConsumedEvent {
                            context: consumed.context,
                            remaining: consumed.remaining,
                            depleted: consumed.depleted,
                        });
                    }
                }
                Err(ActionError::Depleted) => {
                    // No actions remaining, should not happen here
                }
            }
        }

        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(MissionRequested {
                faction: brief.faction.clone(),
                target: brief.target.clone(),
                prototype: brief.prototype.clone(),
                expected_payout,
            });
        }

        SceneTransition::Switch(GameScene::Tactical(TacticalSceneData::from_brief(brief)))
    }

    async fn allocate_research(
        &mut self,
        resources: &mut ResourceContext,
    ) -> SceneTransition<GameScene> {
        let (proto_id, proto_codename) = {
            let ctx = match resources.get::<GameContext>().await {
                Some(ctx) => ctx,
                None => return SceneTransition::Stay,
            };
            let WeaponPrototypeState { id, codename, .. } = ctx.prototypes[0].clone();
            (id, codename)
        };

        let budget = Currency::new(120);

        // Spend from issun BudgetLedger
        let spend_ok = if let Some(mut ledger) = resources.get_mut::<issun::plugin::BudgetLedger>().await {
            ledger.try_spend(issun::plugin::BudgetChannel::Research, budget)
        } else {
            return SceneTransition::Stay;
        };

        if !spend_ok {
            self.status_line = "R&D資金が不足しています".into();
            push_econ_warning(resources, "R&D資金不足: 投資要求を却下").await;
            return SceneTransition::Stay;
        }

        // Calculate innovation multiplier from issun BudgetLedger
        let innovation_multiplier = if let Some(ledger) = resources.get::<issun::plugin::BudgetLedger>().await {
            (ledger.innovation_fund.amount() as f32 / 2000.0).clamp(0.0, 0.35)
        } else {
            0.0
        };

        let mut ctx = match resources.get_mut::<GameContext>().await {
            Some(ctx) => ctx,
            None => return SceneTransition::Stay,
        };
        ctx.record_rnd_spend(budget);
        ctx.queue_research(&proto_id, 0.08, innovation_multiplier);
        ctx.record(format!("{} へR&D投資", proto_codename));
        let demand = ctx
            .territories
            .iter()
            .find(|t| t.id == TerritoryId::new("nova-harbor"))
            .map(|t| t.demand.clone())
            .unwrap_or_else(DemandProfile::frontier);

        drop(ctx);

        // Consume action using ActionPlugin
        if let Some(mut points) = resources.get_mut::<ActionPoints>().await {
            if let Ok(consumed) = points.consume_with("R&D投資") {
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(ActionConsumedEvent {
                        context: consumed.context,
                        remaining: consumed.remaining,
                        depleted: consumed.depleted,
                    });
                }
            }
        }

        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(ResearchQueued {
                prototype: proto_id.clone(),
                budget: Currency::new(120),
                targeted_segment: demand,
            });
        }
        self.status_line = "研究ラインに追加".into();
        SceneTransition::Stay
    }

    async fn open_report(&mut self, resources: &mut ResourceContext) -> SceneTransition<GameScene> {
        if let Some(ctx) = resources.get::<GameContext>().await {
            return SceneTransition::Switch(GameScene::IntelReport(
                IntelReportSceneData::from_context(&ctx),
            ));
        }
        SceneTransition::Stay
    }

    async fn end_day_now(&mut self, resources: &mut ResourceContext) {
        // Update GameContext day
        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            ctx.force_end_of_day("司令部が日次を終了しました");
        }

        // Update issun GameTimer day
        if let Some(mut timer) = resources.get_mut::<issun::plugin::GameTimer>().await {
            timer.increment_day();
        }
    }

    async fn fortify_battlefront(&mut self, resources: &mut ResourceContext) {
        let (front_index, ops_multiplier) = {
            let ctx = match resources.get::<GameContext>().await {
                Some(ctx) => ctx,
                None => return,
            };

            let Some(front_index) = ctx.territories.iter().position(|t| t.battlefront) else {
                drop(ctx);
                self.status_line = "現在の前線はありません".into();
                return;
            };

            drop(ctx);

            let ops_multiplier = issun::plugin::policy::PolicyService::get_active_effect(
                "ops_cost_multiplier",
                resources,
            )
            .await;

            (front_index, ops_multiplier)
        };

        let cost = Currency::new(((120.0_f32 * ops_multiplier).round() as i64).max(60));

        // Spend from issun BudgetLedger
        let spend_ok = if let Some(mut ledger) = resources.get_mut::<issun::plugin::BudgetLedger>().await {
            ledger.try_spend(issun::plugin::BudgetChannel::Ops, cost)
        } else {
            return;
        };

        if !spend_ok {
            self.status_line = "Ops資金が不足しています".into();
            push_econ_warning(resources, "防衛強化失敗: Ops不足").await;
            return;
        }

        let mut ctx = match resources.get_mut::<GameContext>().await {
            Some(ctx) => ctx,
            None => return,
        };
        ctx.record_ops_spend(cost);

        let front_name;
        {
            let front = &mut ctx.territories[front_index];
            front.conflict_intensity = (front.conflict_intensity * 0.6).clamp(0.05, 1.0);
            front.unrest = (front.unrest - 0.05).clamp(0.0, 1.0);
            front.enemy_share = (front.enemy_share - 0.05).clamp(0.0, 1.0);
            front_name = front.id.as_str().to_string();
        }

        ctx.record(format!("{} に防衛部隊を派遣 (Ops {})", front_name, cost));
        self.status_line = format!("{} を要塞化", front_name);

        drop(ctx);

        // Consume action using ActionPlugin
        if let Some(mut points) = resources.get_mut::<ActionPoints>().await {
            if let Ok(consumed) = points.consume_with("防衛強化") {
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(ActionConsumedEvent {
                        context: consumed.context,
                        remaining: consumed.remaining,
                        depleted: consumed.depleted,
                    });
                }
            }
        }
    }

    async fn invest_in_development(&mut self, resources: &mut ResourceContext) {
        let ctx = match resources.get::<GameContext>().await {
            Some(ctx) => ctx,
            None => return,
        };

        let Some(target_index) = ctx
            .territories
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.battlefront)
            .min_by(|(_, a), (_, b)| {
                let a_score = a.development_level;
                let b_score = b.development_level;
                a_score.partial_cmp(&b_score).unwrap_or(Ordering::Equal)
            })
            .map(|(idx, _)| idx)
        else {
            self.status_line = "投資可能なエリアが見つかりません".into();
            return;
        };

        let amount = Currency::new(200);
        let territory_id = ctx.territories[target_index].id.clone();

        // Try spending from Innovation fund
        let spend_ok = if let Some(mut ledger) = resources.get_mut::<issun::plugin::BudgetLedger>().await {
            ledger.try_spend(issun::plugin::BudgetChannel::Innovation, amount)
        } else {
            false
        };

        if !spend_ok {
            self.status_line = "Innovation資金が不足しています".into();
            return;
        }

        let investment_bonus = issun::plugin::policy::PolicyService::get_active_effect(
            "investment_bonus",
            resources,
        )
        .await;

        let mut ctx = match resources.get_mut::<GameContext>().await {
            Some(ctx) => ctx,
            None => return,
        };

        if !ctx.apply_territory_investment(&territory_id, amount, investment_bonus) {
            self.status_line = "投資対象が見つかりません".into();
            return;
        }

        self.status_line = format!("{} に基盤投資 ({} )", territory_id.as_str(), amount);

        drop(ctx);

        // Consume action using ActionPlugin
        if let Some(mut points) = resources.get_mut::<ActionPoints>().await {
            if let Ok(consumed) = points.consume_with("開拓投資") {
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(ActionConsumedEvent {
                        context: consumed.context,
                        remaining: consumed.remaining,
                        depleted: consumed.depleted,
                    });
                }
            }
        }
    }

    async fn set_policy(&mut self, resources: &mut ResourceContext) {
        // Publish PolicyCycleRequested event
        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(issun::plugin::PolicyCycleRequested);
        }

        // Get the newly activated policy name for display
        if let Some(state) = resources.get::<issun::plugin::PolicyState>().await {
            if let Some(active_id) = state.active_policy_id() {
                if let Some(policies) = resources.get::<issun::plugin::Policies>().await {
                    if let Some(policy) = policies.get(active_id) {
                        self.status_line = format!("政策を「{}」に切替", policy.name);
                    }
                }
            }
        }
    }

    async fn execute_diplomacy(&mut self, resources: &mut ResourceContext) {
        let diplomacy_cost = Currency::new(100);

        // Spend from issun BudgetLedger
        let spend_ok = if let Some(mut ledger) = resources.get_mut::<issun::plugin::BudgetLedger>().await {
            ledger.try_spend(issun::plugin::BudgetChannel::Reserve, diplomacy_cost)
        } else {
            return;
        };

        if !spend_ok {
            self.status_line = "予備資金が不足しています".into();
            return;
        }

        let diplomacy_bonus = issun::plugin::policy::PolicyService::get_active_effect(
            "diplomacy_bonus",
            resources,
        )
        .await;

        let mut ctx = match resources.get_mut::<GameContext>().await {
            Some(ctx) => ctx,
            None => return,
        };

        if let Some(front) = ctx
            .territories
            .iter()
            .find(|t| t.battlefront)
            .map(|t| (t.enemy_faction.clone(), t.id.as_str().to_string()))
        {
            let (faction_id, front_name) = front;
            ctx.apply_campaign_response(&faction_id, 0.1, diplomacy_bonus);
            self.status_line = format!("{} へ警告外交を実施", front_name);
        } else if let Some(faction_id) = ctx.territories.get(0).map(|t| t.enemy_faction.clone()) {
            ctx.apply_campaign_response(&faction_id, 0.1, diplomacy_bonus);
            self.status_line = format!("{} 勢力と限定協力を実施", faction_id.as_str());
        }
    }
}

async fn push_econ_warning(resources: &mut ResourceContext, message: impl Into<String>) {
    if let Some(mut econ) = resources.get_mut::<EconomyState>().await {
        econ.warnings.insert(0, message.into());
        econ.warnings.truncate(5);
    }
}
