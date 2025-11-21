//! Combat system implementation

use crate::context::{ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

use super::config::CombatConfig;
use super::events::*;
use super::hook::CombatHook;
use super::state::CombatState;
use super::types::CombatResult;

/// System that processes combat events with hooks
///
/// This system:
/// 1. Processes combat start requests
/// 2. Processes combat turn advance requests
/// 3. Processes combat end requests
/// 4. Calls hooks for custom behavior
/// 5. Publishes state change events for network replication
///
/// # Feedback Loop
///
/// ```text
/// Command Event → Validation (Hook) → State Update → Hook Call → State Event
/// ```
#[derive(Clone)]
pub struct CombatSystem {
    hook: Arc<dyn CombatHook>,
}

impl CombatSystem {
    /// Create a new CombatSystem with a custom hook
    pub fn new(hook: Arc<dyn CombatHook>) -> Self {
        Self { hook }
    }

    /// Process all combat events
    pub async fn process_events(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        self.process_start_requests(resources).await;
        self.process_turn_advance_requests(resources).await;
        self.process_end_requests(resources).await;
    }

    /// Process combat start requests
    async fn process_start_requests(&mut self, resources: &mut ResourceContext) {
        // Collect start requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<CombatStartRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Start battle (update state)
            {
                if let Some(mut state) = resources.get_mut::<CombatState>().await {
                    if state.start_battle(request.battle_id.clone()).is_err() {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(CombatStartedEvent {
                    battle_id: request.battle_id.clone(),
                });
            }
        }
    }

    /// Process combat turn advance requests
    async fn process_turn_advance_requests(&mut self, resources: &mut ResourceContext) {
        // Collect turn advance requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<CombatTurnAdvanceRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Verify battle is active
            let is_active = {
                if let Some(state) = resources.get::<CombatState>().await {
                    state.current_battle() == Some(&request.battle_id)
                } else {
                    false
                }
            };

            if !is_active {
                continue;
            }

            // Advance turn
            let turn = {
                if let Some(mut state) = resources.get_mut::<CombatState>().await {
                    match state.advance_turn() {
                        Ok(t) => t,
                        Err(_) => continue,
                    }
                } else {
                    continue;
                }
            };

            // Call hook: before_turn
            {
                let resources_ref = resources as &ResourceContext;
                if self
                    .hook
                    .before_turn(&request.battle_id, turn, resources_ref)
                    .await
                    .is_err()
                {
                    continue;
                }
            }

            // Call hook: process_turn (main combat logic)
            let log_entries = self
                .hook
                .process_turn(&request.battle_id, turn, resources)
                .await;

            // Add log entries to state
            {
                let config = resources.get::<CombatConfig>().await;
                if let Some(mut state) = resources.get_mut::<CombatState>().await {
                    if let Some(cfg) = config {
                        if cfg.enable_log {
                            for entry in &log_entries {
                                state.add_log(entry.clone(), cfg.max_log_entries);
                            }
                        }
                    }
                }
            }

            // Call hook: after_turn
            self.hook
                .after_turn(&request.battle_id, turn, &log_entries, resources)
                .await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(CombatTurnCompletedEvent {
                    battle_id: request.battle_id.clone(),
                    turn,
                    log_entries,
                });
            }
        }
    }

    /// Process combat end requests
    async fn process_end_requests(&mut self, resources: &mut ResourceContext) {
        // Collect end requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<CombatEndRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Get final state before ending
            let (total_turns, score) = {
                if let Some(state) = resources.get::<CombatState>().await {
                    if state.current_battle() != Some(&request.battle_id) {
                        continue;
                    }
                    (state.turn_count(), state.score())
                } else {
                    continue;
                }
            };

            // Default result is Ongoing (user requested end)
            let result = CombatResult::Ongoing;

            // End battle (update state)
            {
                if let Some(mut state) = resources.get_mut::<CombatState>().await {
                    if state.end_battle().is_err() {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Call hook
            self.hook
                .on_combat_ended(&request.battle_id, &result, total_turns, score, resources)
                .await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(CombatEndedEvent {
                    battle_id: request.battle_id.clone(),
                    result,
                    total_turns,
                    score,
                });
            }
        }
    }
}

#[async_trait]
impl System for CombatSystem {
    fn name(&self) -> &'static str {
        "combat_system"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
