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

    // Old settlement method removed - now handled by BorderEconomyAccountingHook
}

// Old settlement code removed (200+ lines)
// Settlement logic now in hooks::BorderEconomyAccountingHook
