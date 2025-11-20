//! Action points management systems

use crate::context::{Context, ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::plugin::time::{AdvanceTimeRequested, DayChanged};
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;

use super::resources::ActionPoints;

/// System that resets action points when day changes
///
/// Listens for `DayChanged` events and resets ActionPoints to maximum.
#[derive(Default)]
pub struct ActionResetSystem;

impl ActionResetSystem {
    /// Update method - processes day changed events and resets action points
    ///
    /// This method is called with ResourceContext access for managing action points.
    pub async fn update(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Check for day changed events
        let day_changed = if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            let reader = bus.reader::<DayChanged>();
            let has_events = reader.iter().count() > 0;
            has_events
        } else {
            false
        };

        if !day_changed {
            return;
        }

        // Reset action points
        if let Some(mut points) = resources.get_mut::<ActionPoints>().await {
            points.reset();
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

/// System that auto-advances time when actions are depleted
///
/// Checks if ActionPoints are depleted and publishes `AdvanceTimeRequested` event.
/// This provides automatic turn progression for turn-based games.
#[derive(Default)]
pub struct ActionAutoAdvanceSystem;

impl ActionAutoAdvanceSystem {
    /// Update method - checks if actions depleted and requests time advancement
    ///
    /// This method is called with ResourceContext access for managing time advancement.
    pub async fn update(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
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

    #[tokio::test]
    async fn test_action_reset_system() {
        let mut resources = ResourceContext::new();
        resources.insert(ActionPoints::new(3));
        resources.insert(EventBus::new());

        let services = ServiceContext::new();
        let mut system = ActionResetSystem::default();

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
    }

    #[tokio::test]
    async fn test_action_reset_system_no_event() {
        let mut resources = ResourceContext::new();
        resources.insert(ActionPoints::new(3));
        resources.insert(EventBus::new());

        let services = ServiceContext::new();
        let mut system = ActionResetSystem::default();

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

        // Check AdvanceTimeRequested published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<AdvanceTimeRequested>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
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
}
