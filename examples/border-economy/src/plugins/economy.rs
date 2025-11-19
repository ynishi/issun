use crate::events::{MissionRequested, MissionResolved, ResearchQueued, VaultReportGenerated};
use crate::models::{Currency, VaultOutcome};
use issun::plugin::PluginBuilderExt;
use issun::prelude::*;
use serde::{Deserialize, Serialize};
use std::any::Any;

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
    pub fn predict(&self, ledger: &crate::models::BudgetLedger) -> i64 {
        (ledger.cash.0 + ledger.research_pool.0 + ledger.ops_pool.0) / 3
    }
}

#[derive(Default, DeriveSystem)]
#[system(name = "economy_system")]
pub struct EconomySystem;

impl EconomySystem {
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
                    state.last_cashflow = result.revenue_delta.0;
                    state.rolling_income.push(result.revenue_delta.0);
                }
                state.rolling_income.truncate(10);

                for job in research {
                    state
                        .research_backlog
                        .insert(0, format!("{} → {}c", job.prototype, job.budget.0));
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

        self.run_settlement(resources).await;
    }

    async fn run_settlement(&mut self, resources: &mut ResourceContext) {
        use crate::models::context::SETTLEMENT_PERIOD_DAYS;
        use crate::models::GameContext;

        let (day, due) = if let Some(ctx) = resources.get::<GameContext>().await {
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

        let (income, upkeep, settlement) =
            if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
                let income = ctx.forecast_income();
                let upkeep = ctx.forecast_upkeep();
                let settlement = ctx.apply_settlement(income, upkeep);
                (income, upkeep, settlement)
            } else {
                return;
            };

        let dividend_result = {
            if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
                ctx.process_dividend_event()
            } else {
                None
            }
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
                net_margin: if income.0 != 0 {
                    settlement.net.0 as f32 / income.0 as f32
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
                if dividend.shortfall.0 > 0 {
                    econ.warnings
                        .insert(0, format!("配当未払い {} → 評判低下", dividend.shortfall));
                    econ.warnings.truncate(5);
                }
            }
        }
    }
}
