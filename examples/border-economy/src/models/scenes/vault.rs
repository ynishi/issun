use crate::events::VaultInvested;
use crate::models::{
    BudgetChannel, Currency, GameContext, GameScene, SlotType, VaultInvestmentError,
};
use issun::auto_pump;
use issun::event::EventBus;
use issun::prelude::{ResourceContext, SceneTransition, ServiceContext, SystemContext};
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSceneData {
    pub vault_cursor: usize,
    pub slot_cursor: usize,
    pub amount_options: Vec<Currency>,
    pub amount_cursor: usize,
    pub status_line: String,
}

impl VaultSceneData {
    pub fn new() -> Self {
        Self {
            vault_cursor: 0,
            slot_cursor: 0,
            amount_options: vec![
                Currency::new(50),
                Currency::new(100),
                Currency::new(250),
                Currency::new(500),
            ],
            amount_cursor: 1,
            status_line: "Vaultを選択して投資を実行 (Enter)".into(),
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
                self.shift_vault(resources, -1).await;
                SceneTransition::Stay
            }
            InputEvent::Down => {
                self.shift_vault(resources, 1).await;
                SceneTransition::Stay
            }
            InputEvent::Left => {
                self.shift_slot(resources, -1).await;
                SceneTransition::Stay
            }
            InputEvent::Right => {
                self.shift_slot(resources, 1).await;
                SceneTransition::Stay
            }
            InputEvent::Char('a') => {
                let len = self.amount_options.len();
                if len > 0 {
                    self.amount_cursor = (self.amount_cursor + len - 1) % len;
                }
                SceneTransition::Stay
            }
            InputEvent::Char('d') => {
                let len = self.amount_options.len();
                if len > 0 {
                    self.amount_cursor = (self.amount_cursor + 1) % len;
                }
                SceneTransition::Stay
            }
            InputEvent::Select => {
                self.execute_investment(resources).await;
                SceneTransition::Stay
            }
            InputEvent::Cancel => SceneTransition::Switch(GameScene::Strategy(
                super::strategy::StrategySceneData::new(),
            )),
            _ => SceneTransition::Stay,
        };
        transition
    }

    async fn shift_vault(&mut self, resources: &ResourceContext, delta: i32) {
        if let Some(ctx) = resources.get::<GameContext>().await {
            let vaults = ctx.vaults();
            if vaults.is_empty() {
                self.status_line = "探索可能なVaultがまだありません".into();
                return;
            }
            let count = vaults.len();
            if delta > 0 {
                self.vault_cursor = (self.vault_cursor + 1) % count;
            } else {
                if self.vault_cursor == 0 {
                    self.vault_cursor = count - 1;
                } else {
                    self.vault_cursor -= 1;
                }
            }
            self.slot_cursor = 0;
        } else {
            self.status_line = "Vaultデータを取得できません".into();
        }
    }

    async fn shift_slot(&mut self, resources: &ResourceContext, delta: i32) {
        if let Some(ctx) = resources.get::<GameContext>().await {
            let vaults = ctx.vaults();
            if vaults.is_empty() {
                self.slot_cursor = 0;
                self.status_line = "Slot情報がありません".into();
                return;
            }
            let vault_index = self.vault_cursor.min(vaults.len() - 1);
            let slots = &vaults[vault_index].slots;
            if slots.is_empty() {
                self.slot_cursor = 0;
                self.status_line = "このVaultに投資スロットがありません".into();
                return;
            }
            let slot_count = slots.len();
            if delta > 0 {
                self.slot_cursor = (self.slot_cursor + 1) % slot_count;
            } else {
                if self.slot_cursor == 0 {
                    self.slot_cursor = slot_count - 1;
                } else {
                    self.slot_cursor -= 1;
                }
            }
        } else {
            self.status_line = "Slot情報を取得できません".into();
        }
    }

    async fn execute_investment(&mut self, resources: &mut ResourceContext) {
        let amount = self.amount_options[self.amount_cursor];
        let mut ctx = match resources.get_mut::<GameContext>().await {
            Some(ctx) => ctx,
            None => {
                self.status_line = "ゲームコンテキストが未初期化です".into();
                return;
            }
        };

        if ctx.vaults.is_empty() {
            self.status_line = "Vault候補がありません".into();
            return;
        }
        if self.vault_cursor >= ctx.vaults.len() {
            self.vault_cursor = ctx.vaults.len() - 1;
        }
        let vault_id = ctx.vaults[self.vault_cursor].id.clone();
        let slot_id = {
            let slots = &ctx.vaults[self.vault_cursor].slots;
            if slots.is_empty() {
                self.status_line = "このVaultには有効なスロットがありません".into();
                return;
            }
            let slot_index = self.slot_cursor.min(slots.len() - 1);
            slots[slot_index].slot_id.clone()
        };

        // Determine slot type after we have slot_id by borrowing immutably again.
        let slot_type = {
            let vault = &ctx.vaults[self.vault_cursor];
            vault
                .slots
                .iter()
                .find(|slot| slot.slot_id == slot_id)
                .map(|slot| slot.slot_type.clone())
        };
        let Some(slot_type) = slot_type else {
            self.status_line = "スロット情報の読み込みに失敗しました".into();
            return;
        };
        let channel = match slot_type {
            SlotType::Security => BudgetChannel::Security,
            SlotType::Research | SlotType::Special => BudgetChannel::Innovation,
        };

        match ctx.invest_in_vault_slot(&vault_id, &slot_id, amount, channel) {
            Ok(result) => {
                let day_advanced = ctx.consume_action("Vault投資");
                self.status_line = format!(
                    "{} へ {} 投資 (₡{}→₡{})",
                    result.slot_id, amount.amount(), result.before.amount(), result.after.amount()
                );
                drop(ctx);

                // Sync GameClock if day advanced
                if day_advanced {
                    if let Some(mut clock) = resources.get_mut::<issun::plugin::GameClock>().await {
                        clock.advance_day(crate::models::context::DAILY_ACTION_POINTS);
                    }
                }
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(VaultInvested {
                        vault_id: result.vault_id,
                        slot_id: result.slot_id,
                        amount: result.amount,
                        channel,
                    });
                }
            }
            Err(err) => {
                self.status_line = match err {
                    VaultInvestmentError::InvalidAmount => "投資額が無効です".into(),
                    VaultInvestmentError::VaultNotFound => "Vaultが見つかりません".into(),
                    VaultInvestmentError::SlotNotFound => "選択スロットが見つかりません".into(),
                    VaultInvestmentError::ChannelMismatch { expected, .. } => {
                        format!("資金ソースが不一致です (必要: {})", expected)
                    }
                    VaultInvestmentError::InsufficientFunds { required, .. } => {
                        format!("資金不足: 必要 {}", required)
                    }
                };
            }
        }
    }
}
