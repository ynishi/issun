//! Action points management systems

use crate::context::{Context, ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::plugin::time::{AdvanceTimeRequested, DayChanged};
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

use super::events::{ActionConsumedEvent, ActionsResetEvent};
use super::hook::ActionHook;
use super::resources::{ActionConsumed, ActionPoints};

/// System that processes ActionConsumedEvent with hooks
///
/// This system listens for `ActionConsumedEvent` and:
/// 1. Calls the hook's `on_action_consumed()` for custom behavior
/// 2. If actions are depleted, calls `on_actions_depleted()` and optionally publishes `AdvanceTimeRequested`
pub struct ActionSystem {
    hook: Arc<dyn ActionHook>,
}

impl ActionSystem {
    /// Create a new ActionSystem with a custom hook
    pub fn new(hook: Arc<dyn ActionHook>) -> Self {
        Self { hook }
    }

    /// Process action consumed events
    pub async fn process_events(&mut self, _services: &ServiceContext, resources: &mut ResourceContext) {
        // Collect consumed events
        let events = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ActionConsumedEvent>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for event in events {
            let consumed = ActionConsumed {
                context: event.context,
                remaining: event.remaining,
                depleted: event.depleted,
            };

            // Call hook for custom behavior
            self.hook.on_action_consumed(&consumed, resources).await;

            // If depleted, check if should auto-advance
            if consumed.depleted {
                let should_advance = self.hook.on_actions_depleted(resources).await;

                if should_advance {
                    if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                        bus.publish(AdvanceTimeRequested);
                    }
                }
            }
        }
    }
}

#[async_trait]
impl System for ActionSystem {
    fn name(&self) -> &'static str {
        "action_system"
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

/// System that resets action points when day changes
///
/// Listens for `DayChanged` events and resets ActionPoints to maximum.
/// Calls the hook's `on_actions_reset()` and publishes `ActionsResetEvent`.
pub struct ActionResetSystem {
    hook: Arc<dyn ActionHook>,
}

impl ActionResetSystem {
    /// Create a new ActionResetSystem with a custom hook
    pub fn new(hook: Arc<dyn ActionHook>) -> Self {
        Self { hook }
    }

    /// Update method - processes day changed events and resets action points
    ///
    /// This method is called with ResourceContext access for managing action points.
    pub async fn update(&mut self, _services: &ServiceContext, resources: &mut ResourceContext) {
        // Check for day changed events
        let day_changed = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<DayChanged>();
                !reader.iter().collect::<Vec<_>>().is_empty()
            } else {
                false
            }
        };

        if !day_changed {
            return;
        }

        // Reset action points
        let new_count = {
            if let Some(mut points) = resources.get_mut::<ActionPoints>().await {
                points.reset();
                points.available
            } else {
                return;
            }
        };

        // Call hook
        self.hook.on_actions_reset(new_count, resources).await;

        // Publish reset event
        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(ActionsResetEvent { new_count });
        }
    }
}

#[async_trait]
impl System for ActionResetSystem {
    fn name(&self) -> &'static str {
        "action_reset"
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

/// System that auto-advances time when actions are depleted (deprecated)
///
/// **Deprecated**: Use `ActionSystem` with hooks instead.
/// This system is kept for backwards compatibility.
///
/// Checks if ActionPoints are depleted and publishes `AdvanceTimeRequested` event.
/// This provides automatic turn progression for turn-based games.
#[derive(Default)]
#[deprecated(
    since = "0.2.0",
    note = "Use ActionSystem with hooks instead for better control"
)]
pub struct ActionAutoAdvanceSystem;

impl ActionAutoAdvanceSystem {
    /// Update method - checks if actions depleted and requests time advancement
    ///
    /// This method is called with ResourceContext access for managing time advancement.
    pub async fn update(&mut self, _services: &ServiceContext, resources: &mut ResourceContext) {
        // Check if actions are depleted
        let depleted = if let Some(points) = resources.get::<ActionPoints>().await {
            points.is_depleted()
        } else {
            false
        };

        if !depleted {
            return;
        }

        // Request time advancement
        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(AdvanceTimeRequested);
        }
    }
}

#[async_trait]
impl System for ActionAutoAdvanceSystem {
    fn name(&self) -> &'static str {
        "action_auto_advance"
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
    use crate::plugin::action::DefaultActionHook;

    #[tokio::test]
    async fn test_action_reset_system() {
        let mut resources = ResourceContext::new();
        resources.insert(ActionPoints::new(3));
        resources.insert(EventBus::new());

        let services = ServiceContext::new();
        let hook = Arc::new(DefaultActionHook);
        let mut system = ActionResetSystem::new(hook);

        // Consume some actions
        {
            let mut points = resources.get_mut::<ActionPoints>().await.unwrap();
            points.consume();
            points.consume();
            assert_eq!(points.available, 1);
        }

        // Publish DayChanged event
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(DayChanged { day: 2 });
            bus.dispatch(); // Make events visible to readers
        }

        // Process system
        system.update(&services, &mut resources).await;

        // Check points reset
        let points = resources.get::<ActionPoints>().await.unwrap();
        assert_eq!(points.available, 3);

        // Dispatch events to make them visible
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.dispatch();
        }

        // Check ActionsResetEvent published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<ActionsResetEvent>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].new_count, 3);
    }

    #[tokio::test]
    async fn test_action_reset_system_no_event() {
        let mut resources = ResourceContext::new();
        resources.insert(ActionPoints::new(3));
        resources.insert(EventBus::new());

        let services = ServiceContext::new();
        let hook = Arc::new(DefaultActionHook);
        let mut system = ActionResetSystem::new(hook);

        // Consume actions
        {
            let mut points = resources.get_mut::<ActionPoints>().await.unwrap();
            points.consume();
            assert_eq!(points.available, 2);
        }

        // Process without event
        system.update(&services, &mut resources).await;

        // Points should not reset
        let points = resources.get::<ActionPoints>().await.unwrap();
        assert_eq!(points.available, 2);
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn test_action_auto_advance_system_depleted() {
        let mut resources = ResourceContext::new();
        resources.insert(ActionPoints::new(2));
        resources.insert(EventBus::new());

        let services = ServiceContext::new();
        let mut system = ActionAutoAdvanceSystem::default();

        // Deplete all actions
        {
            let mut points = resources.get_mut::<ActionPoints>().await.unwrap();
            points.consume();
            points.consume();
            assert!(points.is_depleted());
        }

        // Process system
        system.update(&services, &mut resources).await;

        // Dispatch events to make them visible
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.dispatch();
        }

        // Check AdvanceTimeRequested published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<AdvanceTimeRequested>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn test_action_auto_advance_system_not_depleted() {
        let mut resources = ResourceContext::new();
        resources.insert(ActionPoints::new(3));
        resources.insert(EventBus::new());

        let services = ServiceContext::new();
        let mut system = ActionAutoAdvanceSystem::default();

        // Consume some but not all
        {
            let mut points = resources.get_mut::<ActionPoints>().await.unwrap();
            points.consume();
            assert!(!points.is_depleted());
        }

        // Process system
        system.update(&services, &mut resources).await;

        // No event should be published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<AdvanceTimeRequested>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 0);
    }

    #[tokio::test]
    async fn test_action_system_with_hook() {
        let mut resources = ResourceContext::new();
        resources.insert(EventBus::new());

        let services = ServiceContext::new();
        let hook = Arc::new(DefaultActionHook);
        let mut system = ActionSystem::new(hook);

        // Publish ActionConsumedEvent
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(ActionConsumedEvent {
                context: "test action".to_string(),
                remaining: 0,
                depleted: true,
            });
            bus.dispatch();
        }

        // Process system
        system.process_events(&services, &mut resources).await;

        // Dispatch events to make them visible
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.dispatch();
        }

        // Check AdvanceTimeRequested published (default hook allows auto-advance)
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<AdvanceTimeRequested>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 1);
    }
}
