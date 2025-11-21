use crate::events::{MissionRequested, MissionResolved, ResearchQueued, VaultReportGenerated};
use crate::models::context::{DividendEventResult, SettlementResult};
use crate::models::{Currency, VaultOutcome};
use issun::plugin::PluginBuilderExt;
use issun::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct EconomyPlugin;

#[async_trait::async_trait]
impl Plugin for EconomyPlugin {
    fn name(&self) -> &'static str {
        "economy_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        builder.register_service(Box::new(LedgerForecastService::default()));
        builder.register_system(Box::new(EconomySystem::default()));
        builder.register_runtime_state(EconomyState::default());
    }
}

fn describe_outcome(outcome: &VaultOutcome) -> String {
    match outcome {
        VaultOutcome::Jackpot { credits, .. } => format!("Jackpot +{}", credits),
        VaultOutcome::Success { credits } => format!("Success +{}", credits),
        VaultOutcome::Mediocre { credits, .. } => format!("Mediocre +{}", credits),
        VaultOutcome::Disaster { debt, .. } => format!("Disaster {}", debt),
        VaultOutcome::Catastrophe { .. } => "Catastrophe!".into(),
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EconomyState {
    pub rolling_income: Vec<i64>,
    pub pending_operations: u32,
    pub research_backlog: Vec<String>,
    pub last_cashflow: i64,
    pub settlement_log: Vec<String>,
    pub warnings: Vec<String>,
    pub last_settlement_day: u32,
    pub last_kpi: Option<SettlementKpi>,
    pub vault_reports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementKpi {
    pub income: Currency,
    pub upkeep: Currency,
    pub net: Currency,
    pub ops_spent: Currency,
    pub rnd_spent: Currency,
    pub dev_spent: Currency,
    pub net_margin: f32,
}

#[derive(Clone, Default, DeriveService)]
#[service(name = "ledger_forecast")]
pub struct LedgerForecastService;

impl LedgerForecastService {
    pub fn predict(&self, ledger: &issun::plugin::BudgetLedger) -> i64 {
        (ledger.cash.amount() + ledger.research_pool.amount() + ledger.ops_pool.amount()) / 3
    }
}

#[derive(Default, DeriveSystem)]
#[system(name = "economy_system")]
pub struct EconomySystem;

impl EconomySystem {
    /// Process game-specific economy events
    ///
    /// Note: Settlement logic is now handled by issun::plugin::AccountingPlugin
    /// with BorderEconomyAccountingHook. This system only handles game-specific
    /// event tracking (missions, research, vault reports) for UI display.
    pub async fn process_events(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        let (requests, resolved, research, vault_reports) =
            match resources.get_mut::<EventBus>().await {
                Some(mut bus) => (
                    collect_events!(bus, MissionRequested),
                    collect_events!(bus, MissionResolved),
                    collect_events!(bus, ResearchQueued),
                    collect_events!(bus, VaultReportGenerated),
                ),
                None => return,
            };

        if let Some(mut state) = resources.get_mut::<EconomyState>().await {
            if !requests.is_empty() || !resolved.is_empty() || !research.is_empty() {
                state.pending_operations = state
                    .pending_operations
                    .saturating_add(requests.len() as u32);
                state.pending_operations = state
                    .pending_operations
                    .saturating_sub(resolved.len() as u32);

                for result in resolved {
                    state.last_cashflow = result.revenue_delta.amount();
                    state.rolling_income.push(result.revenue_delta.amount());
                }
                state.rolling_income.truncate(10);

                for job in research {
                    state
                        .research_backlog
                        .insert(0, format!("{} → {}c", job.prototype, job.budget.amount()));
                }
                state.research_backlog.truncate(6);
            }

            if !vault_reports.is_empty() {
                let mut digest = Vec::new();
                for event in vault_reports {
                    for report in event.reports {
                        let status_label = match report.status {
                            crate::models::VaultStatus::Active => "ACTIVE",
                            crate::models::VaultStatus::Peril { .. } => "PERIL",
                            crate::models::VaultStatus::Captured { .. } => "CAPTURED",
                        };
                        let outcome_text = report
                            .outcome
                            .as_ref()
                            .map(|o| describe_outcome(o))
                            .unwrap_or_else(|| "-".into());
                        digest.push(format!(
                            "{} [{}] Outcome {} | 投資 {} | 減耗 {}",
                            report.codename,
                            status_label,
                            outcome_text,
                            report.total_investment,
                            report.decay_applied
                        ));
                        if let Some(assault) = &report.assault_log {
                            digest.push(format!("   Assault: {}", assault));
                        }
                        for warning in report.warnings {
                            digest.push(format!("⚠ {}", warning));
                        }
                        if matches!(
                            report.outcome,
                            Some(VaultOutcome::Disaster { .. } | VaultOutcome::Catastrophe { .. })
                        ) {
                            state.warnings.insert(
                                0,
                                format!("Vault {} outcome {}", report.codename, outcome_text),
                            );
                            state.warnings.truncate(5);
                        }
                    }
                }
                state.vault_reports = digest;
                state.vault_reports.truncate(6);
            }
        }

        // Settlement logic is now handled by AccountingPlugin + BorderEconomyAccountingHook
    }

    // Settlement method removed - now handled by BorderEconomyAccountingHook
    #[allow(dead_code)]
    async fn _old_run_settlement(&mut self, resources: &mut ResourceContext) {
        use crate::models::context::SETTLEMENT_PERIOD_DAYS;
        use crate::models::GameContext;

        // Try issun GameTimer first, fallback to GameContext
        let (day, due) = if let Some(timer) = resources.get::<issun::plugin::GameTimer>().await {
            (timer.day, timer.day % SETTLEMENT_PERIOD_DAYS == 0)
        } else if let Some(ctx) = resources.get::<GameContext>().await {
            (ctx.day, ctx.day % SETTLEMENT_PERIOD_DAYS == 0)
        } else {
            return;
        };

        if !due {
            return;
        }

        let already_closed = if let Some(state) = resources.get::<EconomyState>().await {
            state.last_settlement_day == day
        } else {
            false
        };

        if already_closed {
            return;
        }

        // Get dividend multiplier from PolicyRegistry
        let dividend_multiplier = if let Some(registry) = resources.get::<issun::plugin::PolicyRegistry>().await {
            registry.get_effect("dividend_multiplier")
        } else {
            1.0
        };

        // Calculate income/upkeep from GameContext
        let (base_income, base_upkeep, ops_spent, rnd_spent, dev_spent) =
            if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
                let base_income = ctx.base_income();
                let base_upkeep = ctx.base_upkeep();
                let (ops, rnd, dev) = ctx.reset_weekly_spending();
                (base_income, base_upkeep, ops, rnd, dev)
            } else {
                return;
            };

        // Apply settlement to issun BudgetLedger
        let (income, upkeep, settlement) = if let Some(mut ledger) = resources.get_mut::<issun::plugin::BudgetLedger>().await {
            // Calculate bonuses from ledger funds
            let investment_income_bonus = (ledger.innovation_fund.amount() as f32 * 0.05) as i64;
            let security_upkeep_offset = (ledger.security_fund.amount() as f32 * 0.08) as i64;

            let income = Currency::new(base_income + investment_income_bonus);
            let upkeep = Currency::new((base_upkeep - security_upkeep_offset).max(0));

            // Apply to cash
            *ledger.get_channel_mut(issun::plugin::BudgetChannel::Cash) =
                ledger.get_channel_mut(issun::plugin::BudgetChannel::Cash)
                    .saturating_add(income)
                    .saturating_sub(upkeep);

            let net_amount = income.amount() - upkeep.amount();
            let mut reserve_bonus = Currency::ZERO;
            let mut innovation_allocation = Currency::ZERO;
            let mut security_allocation = Currency::ZERO;

            if net_amount > 0 {
                reserve_bonus = Currency::new((net_amount as f32 * 0.25) as i64);
                if reserve_bonus.amount() > 0 {
                    *ledger.get_channel_mut(issun::plugin::BudgetChannel::Reserve) =
                        ledger.get_channel_mut(issun::plugin::BudgetChannel::Reserve).saturating_add(reserve_bonus);
                }

                let invest_total = Currency::new((net_amount as f32 * 0.3) as i64);
                if invest_total.amount() > 0 {
                    innovation_allocation = Currency::new((invest_total.amount() as f32 * 0.6) as i64);
                    security_allocation = Currency::new(invest_total.amount() - innovation_allocation.amount());
                    if innovation_allocation.amount() > 0 {
                        *ledger.get_channel_mut(issun::plugin::BudgetChannel::Innovation) =
                            ledger.get_channel_mut(issun::plugin::BudgetChannel::Innovation).saturating_add(innovation_allocation);
                    }
                    if security_allocation.amount() > 0 {
                        *ledger.get_channel_mut(issun::plugin::BudgetChannel::Security) =
                            ledger.get_channel_mut(issun::plugin::BudgetChannel::Security).saturating_add(security_allocation);
                    }
                }
            }

            // Apply investment decay
            let innovation_loss = (ledger.innovation_fund.amount() as f32 * 0.08) as i64;
            if innovation_loss > 0 {
                let deduction = Currency::new(innovation_loss);
                *ledger.get_channel_mut(issun::plugin::BudgetChannel::Innovation) =
                    ledger.get_channel_mut(issun::plugin::BudgetChannel::Innovation).saturating_sub(deduction);
            }
            let security_loss = (ledger.security_fund.amount() as f32 * 0.05) as i64;
            if security_loss > 0 {
                let deduction = Currency::new(security_loss);
                *ledger.get_channel_mut(issun::plugin::BudgetChannel::Security) =
                    ledger.get_channel_mut(issun::plugin::BudgetChannel::Security).saturating_sub(deduction);
            }

            let settlement = SettlementResult {
                net: Currency::new(net_amount),
                reserve_bonus,
                innovation_allocation,
                security_allocation,
                ops_spent,
                rnd_spent,
                dev_spent,
            };
            (income, upkeep, settlement)
        } else {
            return;
        };

        // Process dividend event
        let dividend_result = if let Some(mut ledger) = resources.get_mut::<issun::plugin::BudgetLedger>().await {
            use crate::models::context::{DIVIDEND_BASE, DIVIDEND_RATE};
            let demand_value = ((ledger.cash.amount().max(0) as f32 * DIVIDEND_RATE) * dividend_multiplier) as i64 + DIVIDEND_BASE;

            if demand_value <= 0 {
                None
            } else {
                let mut remaining = demand_value;
                let mut reserve_paid = 0;
                if ledger.reserve.amount() > 0 {
                    let pay = remaining.min(ledger.reserve.amount());
                    *ledger.get_channel_mut(issun::plugin::BudgetChannel::Reserve) =
                        Currency::new(ledger.reserve.amount() - pay);
                    remaining -= pay;
                    reserve_paid = pay;
                }

                let mut cash_paid = 0;
                if remaining > 0 && ledger.cash.amount() > 0 {
                    let pay = remaining.min(ledger.cash.amount());
                    *ledger.get_channel_mut(issun::plugin::BudgetChannel::Cash) =
                        Currency::new(ledger.cash.amount() - pay);
                    remaining -= pay;
                    cash_paid = pay;
                }

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

        if let Some(mut econ) = resources.get_mut::<EconomyState>().await {
            econ.last_settlement_day = day;
            econ.pending_operations = 0;
            econ.settlement_log.insert(
                0,
                format!(
                    "Day {} Settlement: +{} - {} = {} | Reserve +{} | Invest I:{} S:{} | Ops {} R&D {} Dev {}",
                    day,
                    income,
                    upkeep,
                    settlement.net,
                    settlement.reserve_bonus,
                    settlement.innovation_allocation,
                    settlement.security_allocation,
                    settlement.ops_spent,
                    settlement.rnd_spent,
                    settlement.dev_spent
                ),
            );
            econ.settlement_log.truncate(4);
            econ.last_kpi = Some(SettlementKpi {
                income,
                upkeep,
                net: settlement.net,
                ops_spent: settlement.ops_spent,
                rnd_spent: settlement.rnd_spent,
                dev_spent: settlement.dev_spent,
                net_margin: if income.amount() != 0 {
                    settlement.net.amount() as f32 / income.amount() as f32
                } else {
                    0.0
                },
            });
            if let Some(dividend) = dividend_result {
                let entry = format!(
                    "Dividend: 要求{} / Reserve {} / Cash {} / 未払い {}",
                    dividend.demanded,
                    dividend.paid_from_reserve,
                    dividend.paid_from_cash,
                    dividend.shortfall
                );
                econ.settlement_log.insert(0, entry);
                econ.settlement_log.truncate(5);
                if dividend.shortfall.amount() > 0 {
                    econ.warnings
                        .insert(0, format!("配当未払い {} → 評判低下", dividend.shortfall));
                    econ.warnings.truncate(5);
                }
            }
        }
    }
}
