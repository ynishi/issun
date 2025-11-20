use crate::events::{VaultCaptured, VaultDiscovered, VaultInvested, VaultReportGenerated};
use crate::models::vault::VaultReport;
use crate::models::GameContext;
use issun::plugin::PluginBuilderExt;
use issun::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct VaultPlugin;

#[async_trait::async_trait]
impl Plugin for VaultPlugin {
    fn name(&self) -> &'static str {
        "vault_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        builder.register_system(Box::new(VaultSystem::default()));
        builder.register_runtime_state(VaultState::default());
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VaultState {
    pub alerts: Vec<String>,
    pub latest_reports: Vec<VaultReport>,
    pub last_settlement_day: u32,
}

#[derive(Default, DeriveSystem)]
#[system(name = "vault_system")]
pub struct VaultSystem;

impl VaultSystem {
    pub async fn process_events(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        let (discovered, invested, captured) = match resources.get_mut::<EventBus>().await {
            Some(mut bus) => (
                collect_events!(bus, VaultDiscovered),
                collect_events!(bus, VaultInvested),
                collect_events!(bus, VaultCaptured),
            ),
            None => return,
        };

        if let Some(mut state) = resources.get_mut::<VaultState>().await {
            for event in discovered {
                state
                    .alerts
                    .insert(0, format!("新Vault発見: {}", event.vault.codename));
            }

            for event in invested {
                state.alerts.insert(
                    0,
                    format!(
                        "{} へ {} 投資 ({})",
                        event.slot_id, event.amount, event.channel
                    ),
                );
            }

            for event in captured {
                state.alerts.insert(0, format!("{} 失陥", event.vault_id));
            }

            state.alerts.truncate(6);
        }

        self.dispatch_weekly_reports(resources).await;
    }

    async fn dispatch_weekly_reports(&mut self, resources: &mut ResourceContext) {
        use crate::models::context::SETTLEMENT_PERIOD_DAYS;

        // Try issun GameClock first, fallback to GameContext
        let (day, due) = if let Some(clock) = resources.get::<issun::plugin::GameClock>().await {
            (clock.day, clock.day % SETTLEMENT_PERIOD_DAYS == 0)
        } else if let Some(ctx) = resources.get::<GameContext>().await {
            (ctx.day, ctx.settlement_due())
        } else {
            return;
        };

        if !due {
            return;
        }

        let reports = {
            if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
                ctx.tick_vaults()
            } else {
                return;
            }
        };

        if reports.is_empty() {
            return;
        }

        if let Some(mut state) = resources.get_mut::<VaultState>().await {
            if state.last_settlement_day == day {
                return;
            }
            state.last_settlement_day = day;
            state.latest_reports = reports.clone();
            for report in &reports {
                for warning in &report.warnings {
                    state
                        .alerts
                        .insert(0, format!("{}: {}", report.codename, warning));
                }
            }
            state.alerts.truncate(6);
        }

        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(VaultReportGenerated { reports });
        }
    }
}
