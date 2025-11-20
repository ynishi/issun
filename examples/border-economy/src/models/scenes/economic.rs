use crate::models::scenes::{IntelReportSceneData, MissionBrief};
use crate::models::{BudgetChannel, Currency, GameContext, GameScene};
use crate::plugins::EconomyState;
use issun::auto_pump;
use issun::prelude::{ResourceContext, SceneTransition, ServiceContext, SystemContext};
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicSceneData {
    pub cursor: usize,
    pub channels: Vec<BudgetChannel>,
    pub last_transfer: String,
    pub origin_story: Option<String>,
    pub amount_options: Vec<Currency>,
    pub amount_cursor: usize,
    pub diplomacy_mode: bool,
}

impl EconomicSceneData {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            channels: vec![
                BudgetChannel::Research,
                BudgetChannel::Operations,
                BudgetChannel::Reserve,
                BudgetChannel::Innovation,
                BudgetChannel::Security,
            ],
            last_transfer: "リソース配分調整".into(),
            origin_story: None,
            amount_options: vec![
                Currency::new(50),
                Currency::new(100),
                Currency::new(250),
                Currency::new(500),
            ],
            amount_cursor: 0,
            diplomacy_mode: false,
        }
    }

    pub fn after_operation(brief: MissionBrief) -> Self {
        let mut data = Self::new();
        data.origin_story = Some(format!(
            "{} 戦功で {} クレジット見込",
            brief.faction, brief.expected_payout
        ));
        data
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
                if self.cursor == 0 {
                    self.cursor = self.channels.len() - 1;
                } else {
                    self.cursor -= 1;
                }
                SceneTransition::Stay
            }
            InputEvent::Down => {
                self.cursor = (self.cursor + 1) % self.channels.len();
                SceneTransition::Stay
            }
            InputEvent::Char('g') => {
                self.diplomacy_mode = !self.diplomacy_mode;
                if self.diplomacy_mode {
                    self.last_transfer = "外交投資モードに切替 (Gで解除)".into();
                } else {
                    self.last_transfer = "予算配分モードに戻りました".into();
                }
                SceneTransition::Stay
            }
            InputEvent::Char('a') => {
                self.amount_cursor = (self.amount_cursor + self.amount_options.len() - 1)
                    % self.amount_options.len();
                SceneTransition::Stay
            }
            InputEvent::Char('d') => {
                self.amount_cursor = (self.amount_cursor + 1) % self.amount_options.len();
                SceneTransition::Stay
            }
            InputEvent::Char('f') => {
                if self.diplomacy_mode {
                    self.perform_diplomatic_investment(resources).await;
                } else {
                    self.transfer_from_cash(resources).await;
                }
                SceneTransition::Stay
            }
            InputEvent::Left => {
                self.shift(resources, -1).await;
                SceneTransition::Stay
            }
            InputEvent::Right => {
                self.shift(resources, 1).await;
                SceneTransition::Stay
            }
            InputEvent::Select => {
                if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
                    ctx.force_end_of_day("経済レポートを作成");
                }
                if let Some(ctx) = resources.get::<GameContext>().await {
                    return SceneTransition::Switch(GameScene::IntelReport(
                        IntelReportSceneData::from_context(&ctx),
                    ));
                }
                SceneTransition::Stay
            }
            InputEvent::Cancel => SceneTransition::Switch(GameScene::Strategy(
                super::strategy::StrategySceneData::new(),
            )),
            _ => SceneTransition::Stay,
        };
        transition
    }

    async fn shift(&mut self, resources: &mut ResourceContext, direction: i32) {
        let channel = self.channels[self.cursor];
        let amount = Currency::new(50);
        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            let success = if direction > 0 {
                ctx.ledger.shift(BudgetChannel::Reserve, channel, amount)
            } else {
                ctx.ledger.shift(channel, BudgetChannel::Reserve, amount)
            };
            drop(ctx);
            if success {
                if direction > 0 {
                    self.last_transfer = format!("予備→{} に {}", channel, amount);
                } else {
                    self.last_transfer = format!("{}→予備 に {}", channel, amount);
                }
            } else {
                self.last_transfer = "資金不足で配分できませんでした".into();
                push_econ_warning(resources, "配分失敗: 資金不足").await;
            }
        }
    }

    async fn transfer_from_cash(&mut self, resources: &mut ResourceContext) {
        let channel = self.channels[self.cursor];
        let amount = self.amount_options[self.amount_cursor];
        if !matches!(
            channel,
            BudgetChannel::Reserve | BudgetChannel::Innovation | BudgetChannel::Security
        ) {
            self.last_transfer = "Cash投資はReserve/Innovation/Securityのみ可能".into();
            return;
        }

        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            if !ctx.ledger.transfer_from_cash(amount) {
                self.last_transfer = "Cash残高不足で投資できません".into();
                drop(ctx);
                push_econ_warning(resources, "Cash→投資 失敗: 残高不足").await;
                return;
            }

            let target_balance = ctx.ledger.channel_mut(channel);
            *target_balance += amount;
            self.last_transfer = format!("Cash→{} に {}", channel, amount);
            ctx.record(format!("Cashから{}へ{}を投資", channel, amount));
        }
    }

    async fn perform_diplomatic_investment(&mut self, resources: &mut ResourceContext) {
        let amount = self.amount_options[self.amount_cursor];
        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            if !ctx.ledger.transfer_from_cash(amount) {
                self.last_transfer = "Cash残高不足で外交投資できません".into();
                return;
            }

            if let Some(front) = ctx.territories.iter().find(|t| t.battlefront) {
                let faction_id = front.enemy_faction.clone();
                let faction_name = ctx
                    .enemy_faction_by_id(&faction_id)
                    .map(|f| f.codename.clone())
                    .unwrap_or_else(|| "敵勢力".into());
                ctx.apply_campaign_response(&faction_id, amount.amount() as f32 / 500.0);
                self.last_transfer = format!("{} に友好投資: {}", faction_name, amount);
            } else if let Some(territory) = ctx.territories.get(0) {
                let faction_id = territory.enemy_faction.clone();
                ctx.apply_campaign_response(&faction_id, amount.amount() as f32 / 600.0);
                self.last_transfer = format!("{} 勢力に限定協力: {}", faction_id.as_str(), amount);
            }
        }
    }
}

impl Default for EconomicSceneData {
    fn default() -> Self {
        Self::new()
    }
}

async fn push_econ_warning(resources: &mut ResourceContext, message: impl Into<String>) {
    if let Some(mut econ) = resources.get_mut::<EconomyState>().await {
        econ.warnings.insert(0, message.into());
        econ.warnings.truncate(5);
    }
}
