//! Territory management system

use crate::context::{ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

use super::events::*;
use super::hook::TerritoryHook;
use super::state::TerritoryState;
use super::territories::Territories;

/// System that processes territory events with hooks
///
/// This system:
/// 1. Processes control change requests
/// 2. Processes development requests
/// 3. Calls hooks for custom behavior
/// 4. Publishes state change events for network replication
#[derive(Clone)]
pub struct TerritorySystem {
    hook: Arc<dyn TerritoryHook>,
}

impl TerritorySystem {
    /// Create a new TerritorySystem with a custom hook
    pub fn new(hook: Arc<dyn TerritoryHook>) -> Self {
        Self { hook }
    }

    /// Process control change requests
    ///
    /// Listens for `TerritoryControlChangeRequested` events and:
    /// 1. Updates TerritoryState
    /// 2. Calls hook
    /// 3. Publishes `TerritoryControlChangedEvent`
    pub async fn process_control_changes(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect control change requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<TerritoryControlChangeRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Update state
            let change = {
                if let Some(mut state) = resources.get_mut::<TerritoryState>().await {
                    match state.adjust_control(&request.id, request.delta) {
                        Some(change) => change,
                        None => continue,
                    }
                } else {
                    continue;
                }
            };

            // Get territory definition for hook
            let territory = {
                if let Some(territories) = resources.get::<Territories>().await {
                    match territories.get(&request.id) {
                        Some(t) => t.clone(),
                        None => continue,
                    }
                } else {
                    continue;
                }
            };

            // Call hook (synchronous, immediate, local only)
            self.hook
                .on_control_changed(&territory, &change, resources)
                .await;

            // Publish event (asynchronous, for other systems and network)
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(TerritoryControlChangedEvent {
                    id: change.id.clone(),
                    old_control: change.old_control,
                    new_control: change.new_control,
                    delta: change.delta,
                });
            }
        }
    }

    /// Process development requests
    ///
    /// Listens for `TerritoryDevelopmentRequested` events and:
    /// 1. Calls hook to calculate cost
    /// 2. Updates TerritoryState if cost calculation succeeds
    /// 3. Calls hook for post-development
    /// 4. Publishes `TerritoryDevelopedEvent`
    pub async fn process_development_requests(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect development requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<TerritoryDevelopmentRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Get territory definition and current level
            let (territory, current_level) = {
                let territories = match resources.get::<Territories>().await {
                    Some(t) => t,
                    None => continue,
                };

                let territory = match territories.get(&request.id) {
                    Some(t) => t.clone(),
                    None => continue,
                };

                let state = match resources.get::<TerritoryState>().await {
                    Some(s) => s,
                    None => continue,
                };

                let current_level = match state.get_development(&request.id) {
                    Some(level) => level,
                    None => continue,
                };

                (territory, current_level)
            };

            // Calculate cost via hook
            let _cost = match self
                .hook
                .calculate_development_cost(&territory, current_level, resources)
                .await
            {
                Ok(cost) => cost,
                Err(_) => continue, // Hook rejected development
            };

            // NOTE: Cost deduction is game-specific and should be handled
            // by the hook or a separate system that listens to TerritoryDevelopedEvent

            // Develop territory
            let developed = {
                if let Some(mut state) = resources.get_mut::<TerritoryState>().await {
                    match state.develop(&request.id) {
                        Some(dev) => dev,
                        None => continue,
                    }
                } else {
                    continue;
                }
            };

            // Call hook (synchronous, immediate, local only)
            self.hook
                .on_developed(&territory, &developed, resources)
                .await;

            // Publish event (asynchronous, for other systems and network)
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(TerritoryDevelopedEvent {
                    id: developed.id.clone(),
                    old_level: developed.old_level,
                    new_level: developed.new_level,
                });
            }
        }
    }

    /// Process all territory events
    pub async fn process_events(
        &mut self,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        self.process_control_changes(services, resources).await;
        self.process_development_requests(services, resources).await;
    }
}

#[async_trait]
impl System for TerritorySystem {
    fn name(&self) -> &'static str {
        "territory_system"
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
    use crate::plugin::territory::{DefaultTerritoryHook, Territories, Territory, TerritoryId};

    #[tokio::test]
    async fn test_territory_system_control_change() {
        let mut resources = ResourceContext::new();

        // Set up Territories (definitions)
        let mut territories = Territories::new();
        territories.add(Territory::new("nova", "Nova Harbor"));
        resources.insert(territories);

        // Set up TerritoryState (runtime state)
        let mut state = TerritoryState::new();
        state.initialize(&TerritoryId::new("nova"));
        state.set_control(&TerritoryId::new("nova"), 0.5);
        resources.insert(state);

        resources.insert(EventBus::new());

        let services = ServiceContext::new();
        let hook = Arc::new(DefaultTerritoryHook);
        let mut system = TerritorySystem::new(hook);

        // Publish control change request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(TerritoryControlChangeRequested {
                id: "nova".into(),
                delta: 0.2,
            });
            bus.dispatch();
        }

        // Process system
        system.process_events(&services, &mut resources).await;

        // Check state updated
        let state = resources.get::<TerritoryState>().await.unwrap();
        let control = state.get_control(&"nova".into()).unwrap();
        assert!((control - 0.7).abs() < 0.001);

        // Dispatch to make events visible
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.dispatch();
        }

        // Check event published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<TerritoryControlChangedEvent>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].old_control, 0.5);
        assert!((events[0].new_control - 0.7).abs() < 0.001);
        assert!((events[0].delta - 0.2).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_territory_system_development() {
        let mut resources = ResourceContext::new();

        // Set up Territories (definitions)
        let mut territories = Territories::new();
        territories.add(Territory::new("nova", "Nova Harbor"));
        resources.insert(territories);

        // Set up TerritoryState (runtime state)
        let mut state = TerritoryState::new();
        state.initialize(&TerritoryId::new("nova"));
        state.set_development(&TerritoryId::new("nova"), 1);
        resources.insert(state);

        resources.insert(EventBus::new());

        let services = ServiceContext::new();
        let hook = Arc::new(DefaultTerritoryHook);
        let mut system = TerritorySystem::new(hook);

        // Publish development request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(TerritoryDevelopmentRequested { id: "nova".into() });
            bus.dispatch();
        }

        // Process system
        system.process_events(&services, &mut resources).await;

        // Check state updated
        let state = resources.get::<TerritoryState>().await.unwrap();
        let dev_level = state.get_development(&"nova".into()).unwrap();
        assert_eq!(dev_level, 2);

        // Dispatch to make events visible
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.dispatch();
        }

        // Check event published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<TerritoryDevelopedEvent>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].old_level, 1);
        assert_eq!(events[0].new_level, 2);
    }

    #[tokio::test]
    async fn test_territory_system_not_found() {
        let mut resources = ResourceContext::new();
        resources.insert(Territories::new());
        resources.insert(TerritoryState::new());
        resources.insert(EventBus::new());

        let services = ServiceContext::new();
        let hook = Arc::new(DefaultTerritoryHook);
        let mut system = TerritorySystem::new(hook);

        // Request for non-existent territory
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(TerritoryControlChangeRequested {
                id: "nonexistent".into(),
                delta: 0.1,
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
        let reader = bus.reader::<TerritoryControlChangedEvent>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 0);
    }
}
