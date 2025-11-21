//! Custom hooks for border-economy

use async_trait::async_trait;
use issun::context::ResourceContext;
use issun::event::EventBus;
use issun::plugin::accounting::{AccountingHook, BudgetChannel, BudgetLedger, Currency};
use issun::plugin::action::{ActionConsumed, ActionHook};
use issun::plugin::research::{
    ResearchHook, ResearchProject, ResearchQueueRequested, ResearchResult,
};
use issun::plugin::territory::{ControlChanged, Developed, Territory, TerritoryHook};

use crate::events::{FieldTestFeedback, ResearchQueued};
use crate::models::GameContext;
use crate::plugins::{EconomyState, PrototypeBacklog};

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
        _territory: &Territory,
        current_level: u32,
        resources: &ResourceContext,
    ) -> Result<i64, String> {
        // Get policy bonus using PolicyService helper
        let bonus = issun::plugin::policy::PolicyService::get_active_effect(
            "investment_bonus",
            resources,
        )
        .await;

        let base_cost = 100 * (current_level + 1);
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

/// Hook that bridges ResearchPlugin with border-economy's prototype system
pub struct PrototypeResearchHook;

#[async_trait]
impl ResearchHook for PrototypeResearchHook {
    async fn on_research_queued(
        &self,
        project: &ResearchProject,
        resources: &mut ResourceContext,
    ) {
        // Update PrototypeBacklog for UI display
        if let Some(mut backlog) = resources.get_mut::<PrototypeBacklog>().await {
            backlog.queued.insert(
                0,
                format!("{} +{}c", project.name, project.cost),
            );
            backlog.queued.truncate(6);
        }

        // Log to GameContext
        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            ctx.record(format!("研究開始: {}", project.name));
        }
    }

    async fn on_research_completed(
        &self,
        project: &ResearchProject,
        result: &ResearchResult,
        resources: &mut ResourceContext,
    ) {
        // Log completion with metrics
        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            let effectiveness = result.final_metrics.get("effectiveness").unwrap_or(&1.0);
            let reliability = result.final_metrics.get("reliability").unwrap_or(&1.0);

            ctx.record(format!(
                "研究完了: {} (効果: {:.0}% / 信頼性: {:.0}%)",
                project.name,
                effectiveness * 100.0,
                reliability * 100.0
            ));
        }
    }
}

/// Hook that implements border-economy's settlement logic
pub struct BorderEconomyAccountingHook;

#[async_trait]
impl AccountingHook for BorderEconomyAccountingHook {
    async fn calculate_income(&self, _period: u32, resources: &ResourceContext) -> Currency {
        // Get base income from GameContext
        let base_income = if let Some(ctx) = resources.get::<GameContext>().await {
            ctx.base_income()
        } else {
            0
        };

        // Get innovation fund bonus (5% of innovation fund)
        let innovation_bonus = if let Some(ledger) = resources.get::<BudgetLedger>().await {
            (ledger.innovation_fund.amount() as f32 * 0.05) as i64
        } else {
            0
        };

        Currency::new(base_income + innovation_bonus)
    }

    async fn calculate_expenses(&self, _period: u32, resources: &ResourceContext) -> Currency {
        // Get base upkeep from GameContext
        let base_upkeep = if let Some(ctx) = resources.get::<GameContext>().await {
            ctx.base_upkeep()
        } else {
            0
        };

        // Get security fund offset (8% of security fund reduces upkeep)
        let security_offset = if let Some(ledger) = resources.get::<BudgetLedger>().await {
            (ledger.security_fund.amount() as f32 * 0.08) as i64
        } else {
            0
        };

        Currency::new((base_upkeep - security_offset).max(0))
    }

    async fn after_settlement(
        &self,
        _period: u32,
        income: Currency,
        expenses: Currency,
        net: Currency,
        resources: &mut ResourceContext,
    ) {
        let net_amount = net.amount();

        // Reset weekly spending from GameContext
        let (ops_spent, rnd_spent, dev_spent) = if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            ctx.reset_weekly_spending()
        } else {
            (Currency::ZERO, Currency::ZERO, Currency::ZERO)
        };

        // Apply bonuses and allocations if profitable
        if net_amount > 0 {
            if let Some(mut ledger) = resources.get_mut::<BudgetLedger>().await {
                // Reserve bonus (25% of net)
                let reserve_bonus = Currency::new((net_amount as f32 * 0.25) as i64);
                if reserve_bonus.amount() > 0 {
                    *ledger.get_channel_mut(BudgetChannel::Reserve) =
                        ledger.get_channel(BudgetChannel::Reserve).saturating_add(reserve_bonus);
                }

                // Investment allocation (30% of net, split 60/40 innovation/security)
                let invest_total = Currency::new((net_amount as f32 * 0.3) as i64);
                if invest_total.amount() > 0 {
                    let innovation_allocation = Currency::new((invest_total.amount() as f32 * 0.6) as i64);
                    let security_allocation = Currency::new(invest_total.amount() - innovation_allocation.amount());

                    if innovation_allocation.amount() > 0 {
                        *ledger.get_channel_mut(BudgetChannel::Innovation) =
                            ledger.get_channel(BudgetChannel::Innovation).saturating_add(innovation_allocation);
                    }
                    if security_allocation.amount() > 0 {
                        *ledger.get_channel_mut(BudgetChannel::Security) =
                            ledger.get_channel(BudgetChannel::Security).saturating_add(security_allocation);
                    }
                }
            }
        }

        // Apply investment decay
        if let Some(mut ledger) = resources.get_mut::<BudgetLedger>().await {
            // Innovation decay (8% per settlement)
            let innovation_loss = (ledger.innovation_fund.amount() as f32 * 0.08) as i64;
            if innovation_loss > 0 {
                *ledger.get_channel_mut(BudgetChannel::Innovation) =
                    ledger.get_channel(BudgetChannel::Innovation).saturating_sub(Currency::new(innovation_loss));
            }

            // Security decay (5% per settlement)
            let security_loss = (ledger.security_fund.amount() as f32 * 0.05) as i64;
            if security_loss > 0 {
                *ledger.get_channel_mut(BudgetChannel::Security) =
                    ledger.get_channel(BudgetChannel::Security).saturating_sub(Currency::new(security_loss));
            }
        }

        // Update EconomyState for UI
        if let Some(mut state) = resources.get_mut::<EconomyState>().await {
            state.last_cashflow = net.amount();
            state.rolling_income.push(net.amount());
            state.rolling_income.truncate(10);

            // Create KPI snapshot
            state.last_kpi = Some(crate::plugins::economy::SettlementKpi {
                income,
                upkeep: expenses,
                net,
                ops_spent,
                rnd_spent,
                dev_spent,
                net_margin: if income.amount() > 0 {
                    net.amount() as f32 / income.amount() as f32
                } else {
                    0.0
                },
            });

            // Log settlement
            state.settlement_log.insert(
                0,
                format!(
                    "決算: 収入{}c 支出{}c 純利益{}c",
                    income.amount(),
                    expenses.amount(),
                    net.amount()
                ),
            );
            state.settlement_log.truncate(10);
        }

        // Log to GameContext
        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            ctx.record(format!(
                "週次決算: 収入 {}c, 支出 {}c, 純利益 {}c",
                income.amount(),
                expenses.amount(),
                net.amount()
            ));
        }

        // Process dividend
        self.process_dividend(resources).await;
    }
}

impl BorderEconomyAccountingHook {
    /// Process dividend payment
    async fn process_dividend(&self, resources: &mut ResourceContext) {
        use crate::models::context::{DIVIDEND_BASE, DIVIDEND_RATE};
        use crate::models::context::DividendEventResult;

        // Get dividend multiplier using PolicyService helper
        let dividend_multiplier = issun::plugin::policy::PolicyService::get_active_effect(
            "dividend_multiplier",
            resources,
        )
        .await;

        let dividend_result = if let Some(mut ledger) = resources.get_mut::<BudgetLedger>().await {
            let demand_value = ((ledger.cash.amount().max(0) as f32 * DIVIDEND_RATE) * dividend_multiplier) as i64 + DIVIDEND_BASE;

            if demand_value <= 0 {
                None
            } else {
                let mut remaining = demand_value;
                let mut reserve_paid = 0;

                // Pay from reserve first
                if ledger.reserve.amount() > 0 {
                    let pay = remaining.min(ledger.reserve.amount());
                    *ledger.get_channel_mut(BudgetChannel::Reserve) =
                        Currency::new(ledger.reserve.amount() - pay);
                    remaining -= pay;
                    reserve_paid = pay;
                }

                // Pay from cash if needed
                let mut cash_paid = 0;
                if remaining > 0 && ledger.cash.amount() > 0 {
                    let pay = remaining.min(ledger.cash.amount());
                    *ledger.get_channel_mut(BudgetChannel::Cash) =
                        Currency::new(ledger.cash.amount() - pay);
                    remaining -= pay;
                    cash_paid = pay;
                }

                // If shortfall, reduce reputation
                let shortfall = remaining.max(0);
                if shortfall > 0 {
                    if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
                        ctx.reputation.adjust(-7.5);
                    }
                }

                Some(DividendEventResult {
                    demanded: Currency::new(demand_value),
                    paid_from_reserve: Currency::new(reserve_paid),
                    paid_from_cash: Currency::new(cash_paid),
                    shortfall: Currency::new(shortfall),
                })
            }
        } else {
            None
        };

        // Log dividend to EconomyState
        if let Some(dividend) = dividend_result {
            if let Some(mut state) = resources.get_mut::<EconomyState>().await {
                let entry = format!(
                    "配当: 要求{} / Reserve {} / Cash {} / 未払い {}",
                    dividend.demanded,
                    dividend.paid_from_reserve,
                    dividend.paid_from_cash,
                    dividend.shortfall
                );
                state.settlement_log.insert(0, entry);
                state.settlement_log.truncate(5);

                if dividend.shortfall.amount() > 0 {
                    state.warnings.insert(
                        0,
                        format!("配当未払い {} → 評判低下", dividend.shortfall),
                    );
                    state.warnings.truncate(5);
                }
            }
        }
    }
}
