//! Systems for the Accounting Plugin

use bevy::prelude::*;

use super::components::*;
use super::events::*;

/// Resource for accounting configuration
#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct AccountingConfig {
    pub settlement_period_days: u32,
}

impl Default for AccountingConfig {
    fn default() -> Self {
        Self {
            settlement_period_days: 7, // Weekly settlements
        }
    }
}

/// Resource for tracking current game day
/// TODO: This should come from TimePlugin
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct GameDay {
    pub current: u32,
}

/// System: Handle day changed events and trigger settlements
pub fn handle_day_changed(
    mut commands: Commands,
    config: Res<AccountingConfig>,
    day: Res<GameDay>,
    mut orgs: Query<(Entity, &mut SettlementHistory), With<BudgetLedger>>,
) {
    let current_day = day.current;

    for (org_entity, history) in orgs.iter_mut() {
        if history.should_run_settlement(current_day, config.settlement_period_days) {
            let period = current_day / config.settlement_period_days;

            commands.write_message(SettlementRequested {
                organization: org_entity,
                period,
                day: current_day,
            });
        }
    }
}

/// System: Start settlement sessions
pub fn start_settlement_sessions(
    mut commands: Commands,
    mut events: MessageReader<SettlementRequested>,
) {
    for event in events.read() {
        // Spawn a settlement session entity
        commands.spawn((
            SettlementSession {
                organization: event.organization,
                period: event.period,
                day: event.day,
                status: SettlementStatus::Calculating,
            },
            SettlementCalculation::default(),
        ));
    }
}

/// System: Calculate income (observers can add sources)
pub fn calculate_income(
    mut sessions: Query<
        (Entity, &SettlementSession, &mut SettlementCalculation),
        With<SettlementSession>,
    >,
    mut commands: Commands,
) {
    for (session_entity, session, mut calc) in sessions.iter_mut() {
        if session.status != SettlementStatus::Calculating {
            continue;
        }

        // Trigger event for observers to add income sources
        commands.trigger(IncomeCalculationEvent {
            settlement_entity: session_entity,
            organization: session.organization,
            period: session.period,
            sources: calc.income_sources.clone(),
        });

        // Calculate total income
        calc.total_income = calc.income_sources.iter().map(|s| s.amount).sum();
    }
}

/// System: Calculate expenses (observers can add items)
pub fn calculate_expenses(
    mut sessions: Query<
        (Entity, &SettlementSession, &mut SettlementCalculation),
        With<SettlementSession>,
    >,
    mut commands: Commands,
) {
    for (session_entity, session, mut calc) in sessions.iter_mut() {
        if session.status != SettlementStatus::Calculating {
            continue;
        }

        // Trigger event for observers to add expense items
        commands.trigger(ExpenseCalculationEvent {
            settlement_entity: session_entity,
            organization: session.organization,
            period: session.period,
            items: calc.expense_items.clone(),
        });

        // Calculate total expenses
        calc.total_expenses = calc.expense_items.iter().map(|e| e.amount).sum();
    }
}

/// System: Finalize settlements
pub fn finalize_settlements(
    mut commands: Commands,
    mut sessions: Query<(Entity, &mut SettlementSession, &SettlementCalculation)>,
    mut ledgers: Query<&mut BudgetLedger>,
    mut histories: Query<&mut SettlementHistory>,
) {
    for (session_entity, mut session, calc) in sessions.iter_mut() {
        if session.status != SettlementStatus::Calculating {
            continue;
        }

        // Calculate net
        let net = calc.total_income - calc.total_expenses;

        // ⚠️ CRITICAL: Use if let Ok(...) for component access
        if let Ok(mut ledger) = ledgers.get_mut(session.organization) {
            ledger.cash = ledger.cash.saturating_add(net);

            // Update history
            if let Ok(mut history) = histories.get_mut(session.organization) {
                history.record_settlement(SettlementRecord {
                    period: session.period,
                    day: session.day,
                    income: calc.total_income,
                    expenses: calc.total_expenses,
                    net,
                    cash_after: ledger.cash,
                });
            }

            // Publish event
            commands.write_message(SettlementCompletedEvent {
                organization: session.organization,
                period: session.period,
                income: calc.total_income,
                expenses: calc.total_expenses,
                net,
            });

            // Mark completed
            session.status = SettlementStatus::Completed;
        } else {
            // Organization deleted or missing BudgetLedger - despawn session
            warn!(
                "Settlement session {:?} references deleted or invalid organization {:?}",
                session_entity, session.organization
            );
            commands.entity(session_entity).despawn();
        }
    }
}

/// System: Handle budget transfers
pub fn handle_budget_transfers(
    mut commands: Commands,
    mut events: MessageReader<BudgetTransferRequested>,
    mut ledgers: Query<&mut BudgetLedger>,
) {
    for event in events.read() {
        // ⚠️ CRITICAL: Use if let Ok(...) for component access
        if let Ok(mut ledger) = ledgers.get_mut(event.organization) {
            if ledger.transfer(event.from, event.to, event.amount) {
                commands.write_message(BudgetTransferredEvent {
                    organization: event.organization,
                    from: event.from,
                    to: event.to,
                    amount: event.amount,
                });
            } else {
                warn!(
                    "Budget transfer failed: insufficient funds in {:?} channel",
                    event.from
                );
            }
        } else {
            // Organization deleted or missing BudgetLedger
            warn!(
                "Budget transfer requested for deleted or invalid organization {:?}",
                event.organization
            );
        }
    }
}

/// System: Cleanup completed settlement sessions
pub fn cleanup_settlement_sessions(
    mut commands: Commands,
    sessions: Query<(Entity, &SettlementSession)>,
) {
    for (entity, session) in sessions.iter() {
        if session.status == SettlementStatus::Completed {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accounting_config_default() {
        let config = AccountingConfig::default();
        assert_eq!(config.settlement_period_days, 7);
    }

    #[test]
    fn test_settlement_flow() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add resources
        app.insert_resource(AccountingConfig::default());
        app.insert_resource(GameDay { current: 7 });

        // Add messages
        app.add_message::<SettlementRequested>();
        app.add_message::<SettlementCompletedEvent>();

        // Add systems
        app.add_systems(
            Update,
            (
                start_settlement_sessions,
                calculate_income,
                calculate_expenses,
                finalize_settlements,
            )
                .chain(),
        );

        // Spawn organization
        let org = app
            .world_mut()
            .spawn((
                Organization {
                    name: "Test Corp".to_string(),
                },
                BudgetLedger::default(),
                SettlementHistory::default(),
            ))
            .id();

        // Trigger settlement
        app.world_mut().write_message(SettlementRequested {
            organization: org,
            period: 1,
            day: 7,
        });

        // Run systems
        app.update();

        // Verify settlement session was created
        let sessions: Vec<_> = app
            .world_mut()
            .query::<&SettlementSession>()
            .iter(app.world())
            .collect();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].period, 1);
    }

    #[test]
    fn test_budget_transfer_system() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add messages
        app.add_message::<BudgetTransferRequested>();
        app.add_message::<BudgetTransferredEvent>();

        app.add_systems(Update, handle_budget_transfers);

        // Spawn organization
        let org = app
            .world_mut()
            .spawn((
                Organization {
                    name: "Test Corp".to_string(),
                },
                BudgetLedger::default(),
            ))
            .id();

        // Trigger transfer
        app.world_mut().write_message(BudgetTransferRequested {
            organization: org,
            from: BudgetChannel::Cash,
            to: BudgetChannel::Research,
            amount: Currency::new(100),
        });

        // Run system
        app.update();

        // Verify transfer occurred
        let ledger = app.world().get::<BudgetLedger>(org).unwrap();
        assert_eq!(ledger.cash, Currency::new(2300));
        assert_eq!(ledger.research_pool, Currency::new(700));
    }

    #[test]
    fn test_deleted_organization_handling() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.add_systems(Update, finalize_settlements);

        // Spawn organization and delete it
        let org = app
            .world_mut()
            .spawn((
                Organization {
                    name: "Test Corp".to_string(),
                },
                BudgetLedger::default(),
            ))
            .id();

        app.world_mut().entity_mut(org).despawn();

        // Spawn settlement session referencing deleted org
        app.world_mut().spawn((
            SettlementSession {
                organization: org,
                period: 1,
                day: 7,
                status: SettlementStatus::Calculating,
            },
            SettlementCalculation::default(),
        ));

        // Run system - should not panic
        app.update();

        // Session should still exist (will be cleaned up by cleanup system)
        let sessions: Vec<_> = app
            .world_mut()
            .query::<&SettlementSession>()
            .iter(app.world())
            .collect();

        // Session should be despawned due to invalid org
        assert_eq!(sessions.len(), 0);
    }
}
