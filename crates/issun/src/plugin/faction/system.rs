//! Faction management system

use crate::context::{Context, ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

use super::events::*;
use super::factions::Factions;
use super::hook::FactionHook;
use super::state::FactionState;
use super::types::*;

/// System that processes faction events with hooks
///
/// This system:
/// 1. Processes operation launch requests
/// 2. Processes operation resolution requests
/// 3. Calls hooks for custom behavior
/// 4. Publishes state change events for network replication
///
/// # Feedback Loop
///
/// ```text
/// Command Event → State Update → Hook Call → State Event
/// ```
#[derive(Clone)]
pub struct FactionSystem {
    hook: Arc<dyn FactionHook>,
    /// Unique operation ID counter
    next_operation_id: u64,
}

impl FactionSystem {
    /// Create a new FactionSystem with a custom hook
    pub fn new(hook: Arc<dyn FactionHook>) -> Self {
        Self {
            hook,
            next_operation_id: 1,
        }
    }

    /// Generate a unique operation ID
    fn generate_operation_id(&mut self) -> OperationId {
        let id = OperationId::new(format!("op-{:06}", self.next_operation_id));
        self.next_operation_id += 1;
        id
    }

    /// Process operation launch requests
    ///
    /// Listens for `OperationLaunchRequested` events and:
    /// 1. Validates operation cost (via hook)
    /// 2. Creates operation and updates state
    /// 3. Calls hook
    /// 4. Publishes `OperationLaunchedEvent`
    pub async fn process_operation_launches(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect launch requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<OperationLaunchRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Get faction for hook
            let faction = {
                if let Some(factions) = resources.get::<Factions>().await {
                    match factions.get(&request.faction_id) {
                        Some(f) => f.clone(),
                        None => continue, // Faction not found, skip
                    }
                } else {
                    continue;
                }
            };

            // Generate operation ID
            let operation_id = self.generate_operation_id();

            // Create operation
            let operation = Operation::new(
                operation_id.as_str(),
                request.faction_id.clone(),
                request.operation_name.clone(),
            )
            .with_metadata(request.metadata.clone());

            // Validate cost via hook (read-only resources access)
            let cost = {
                let resources_ref = resources as &ResourceContext;
                match self
                    .hook
                    .calculate_operation_cost(&faction, &operation, resources_ref)
                    .await
                {
                    Ok(cost) => cost,
                    Err(_) => continue, // Hook rejected operation
                }
            };

            // NOTE: Cost deduction is game-specific and should be handled
            // by the hook or a separate system that listens to OperationLaunchedEvent
            let _ = cost; // Suppress unused warning

            // Launch operation (add to state)
            {
                if let Some(mut state) = resources.get_mut::<FactionState>().await {
                    if state.launch_operation(operation.clone()).is_err() {
                        continue; // Failed to launch
                    }
                } else {
                    continue;
                }
            }

            // Call hook (synchronous, immediate, local only)
            self.hook
                .on_operation_launched(&faction, &operation, resources)
                .await;

            // Publish event (asynchronous, for other systems and network)
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(OperationLaunchedEvent {
                    operation_id: operation.id.clone(),
                    faction_id: operation.faction_id.clone(),
                    operation_name: operation.name.clone(),
                });
            }
        }
    }

    /// Process operation resolution requests
    ///
    /// Listens for `OperationResolveRequested` events and:
    /// 1. Updates operation status based on outcome
    /// 2. Calls hook for feedback loop
    /// 3. Publishes `OperationCompletedEvent` or `OperationFailedEvent`
    pub async fn process_operation_resolutions(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect resolution requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<OperationResolveRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Get operation
            let operation = {
                if let Some(state) = resources.get::<FactionState>().await {
                    match state.get_operation(&request.operation_id) {
                        Some(op) => op.clone(),
                        None => continue, // Operation not found
                    }
                } else {
                    continue;
                }
            };

            // Get faction
            let faction = {
                if let Some(factions) = resources.get::<Factions>().await {
                    match factions.get(&operation.faction_id) {
                        Some(f) => f.clone(),
                        None => continue, // Faction not found
                    }
                } else {
                    continue;
                }
            };

            // Update operation status based on success
            let status = if request.outcome.success {
                OperationStatus::Completed
            } else {
                OperationStatus::Failed
            };

            {
                if let Some(mut state) = resources.get_mut::<FactionState>().await {
                    if state
                        .update_operation_status(&request.operation_id, status)
                        .is_err()
                    {
                        continue; // Failed to update status
                    }
                } else {
                    continue;
                }
            }

            // **Key feedback loop**: Call hook to interpret outcome and update resources
            if request.outcome.success {
                self.hook
                    .on_operation_completed(&faction, &operation, &request.outcome, resources)
                    .await;

                // Publish completion event
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(OperationCompletedEvent {
                        operation_id: request.operation_id.clone(),
                        faction_id: operation.faction_id.clone(),
                        success: true,
                        metrics: request.outcome.metrics.clone(),
                    });
                }
            } else {
                self.hook
                    .on_operation_failed(&faction, &operation, resources)
                    .await;

                // Publish failure event
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(OperationFailedEvent {
                        operation_id: request.operation_id.clone(),
                        faction_id: operation.faction_id.clone(),
                        reason: request
                            .outcome
                            .metadata
                            .get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown failure")
                            .to_string(),
                    });
                }
            }
        }
    }

    /// Process all faction events
    pub async fn process_events(
        &mut self,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        self.process_operation_launches(services, resources).await;
        self.process_operation_resolutions(services, resources)
            .await;
    }
}

#[async_trait]
impl System for FactionSystem {
    fn name(&self) -> &'static str {
        "faction_system"
    }

    async fn update(&mut self, _ctx: &mut Context) {
        // Legacy Context support (deprecated path)
        // Modern systems should use the async ResourceContext/ServiceContext pattern
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventBus;
    use crate::plugin::faction::{DefaultFactionHook, Faction};
    use serde_json::json;

    #[tokio::test]
    async fn test_faction_system_launch_operation() {
        let mut resources = ResourceContext::new();
        let mut factions = Factions::new();
        factions.add(Faction::new("crimson", "Crimson Syndicate"));
        resources.insert(factions);
        resources.insert(FactionState::new());
        resources.insert(EventBus::new());

        let services = ServiceContext::new();
        let hook = Arc::new(DefaultFactionHook);
        let mut system = FactionSystem::new(hook);

        // Publish operation launch request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(OperationLaunchRequested {
                faction_id: FactionId::new("crimson"),
                operation_name: "Test Operation".to_string(),
                metadata: json!({ "test": "data" }),
            });
            bus.dispatch();
        }

        // Process system
        system.process_events(&services, &mut resources).await;

        // Check state updated
        let state = resources.get::<FactionState>().await.unwrap();
        assert_eq!(state.operation_count(), 1);

        // Dispatch to make events visible
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.dispatch();
        }

        // Check event published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<OperationLaunchedEvent>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].operation_name, "Test Operation");
    }

    #[tokio::test]
    async fn test_faction_system_complete_operation() {
        let mut resources = ResourceContext::new();
        let mut factions = Factions::new();
        factions.add(Faction::new("crimson", "Crimson Syndicate"));
        resources.insert(factions);

        let mut state = FactionState::new();
        let op = Operation::new("op-001", FactionId::new("crimson"), "Test");
        state.launch_operation(op).unwrap();
        resources.insert(state);
        resources.insert(EventBus::new());

        let services = ServiceContext::new();
        let hook = Arc::new(DefaultFactionHook);
        let mut system = FactionSystem::new(hook);

        // Publish resolution request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(OperationResolveRequested {
                operation_id: OperationId::new("op-001"),
                outcome: Outcome::new("op-001", true).with_metric("test", 1.0),
            });
            bus.dispatch();
        }

        // Process system
        system.process_events(&services, &mut resources).await;

        // Check operation completed
        let state = resources.get::<FactionState>().await.unwrap();
        let op = state.get_operation(&OperationId::new("op-001")).unwrap();
        assert!(op.is_completed());

        // Dispatch to make events visible
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.dispatch();
        }

        // Check event published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<OperationCompletedEvent>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 1);
        assert!(events[0].success);
    }

    #[tokio::test]
    async fn test_faction_system_fail_operation() {
        let mut resources = ResourceContext::new();
        let mut factions = Factions::new();
        factions.add(Faction::new("crimson", "Crimson Syndicate"));
        resources.insert(factions);

        let mut state = FactionState::new();
        let op = Operation::new("op-001", FactionId::new("crimson"), "Test");
        state.launch_operation(op).unwrap();
        resources.insert(state);
        resources.insert(EventBus::new());

        let services = ServiceContext::new();
        let hook = Arc::new(DefaultFactionHook);
        let mut system = FactionSystem::new(hook);

        // Publish resolution request (failed)
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(OperationResolveRequested {
                operation_id: OperationId::new("op-001"),
                outcome: Outcome::new("op-001", false)
                    .with_metadata(json!({ "reason": "Test failure" })),
            });
            bus.dispatch();
        }

        // Process system
        system.process_events(&services, &mut resources).await;

        // Check operation failed
        let state = resources.get::<FactionState>().await.unwrap();
        let op = state.get_operation(&OperationId::new("op-001")).unwrap();
        assert!(op.is_failed());

        // Dispatch to make events visible
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.dispatch();
        }

        // Check event published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<OperationFailedEvent>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].reason, "Test failure");
    }

    #[tokio::test]
    async fn test_faction_system_operation_not_found() {
        let mut resources = ResourceContext::new();
        resources.insert(Factions::new());
        resources.insert(FactionState::new());
        resources.insert(EventBus::new());

        let services = ServiceContext::new();
        let hook = Arc::new(DefaultFactionHook);
        let mut system = FactionSystem::new(hook);

        // Request for non-existent operation
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(OperationResolveRequested {
                operation_id: OperationId::new("nonexistent"),
                outcome: Outcome::new("nonexistent", true),
            });
            bus.dispatch();
        }

        // Process system
        system.process_events(&services, &mut resources).await;

        // Dispatch events
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.dispatch();
        }

        // No event should be published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<OperationCompletedEvent>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 0);
    }
}
