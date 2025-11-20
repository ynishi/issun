//! Action points management systems

use crate::context::ResourceContext;
use crate::event::EventBus;
use crate::plugin::time::{AdvanceTimeRequested, DayChanged};
use crate::system::{DeriveSystem, System};
use async_trait::async_trait;

use super::resources::ActionPoints;

/// System that resets action points when day changes
///
/// Listens for `DayChanged` events and resets ActionPoints to maximum.
#[derive(Default, DeriveSystem)]
#[system(name = "action_reset")]
pub struct ActionResetSystem;

#[async_trait]
impl System for ActionResetSystem {
    async fn process(&mut self, resources: &mut ResourceContext) {
        // Check for day changed events
        let day_changed = if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            let events: Vec<_> = bus.reader::<DayChanged>().iter().collect();
            !events.is_empty()
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

/// System that auto-advances time when actions are depleted
///
/// Checks if ActionPoints are depleted and publishes `AdvanceTimeRequested` event.
/// This provides automatic turn progression for turn-based games.
#[derive(Default, DeriveSystem)]
#[system(name = "action_auto_advance")]
pub struct ActionAutoAdvanceSystem;

#[async_trait]
impl System for ActionAutoAdvanceSystem {
    async fn process(&mut self, resources: &mut ResourceContext) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventBus;

    #[tokio::test]
    async fn test_action_reset_system() {
        let mut resources = ResourceContext::new();
        resources.insert(ActionPoints::new(3));
        resources.insert(EventBus::new());

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
        }

        // Process system
        system.process(&mut resources).await;

        // Check points reset
        let points = resources.get::<ActionPoints>().await.unwrap();
        assert_eq!(points.available, 3);
    }

    #[tokio::test]
    async fn test_action_reset_system_no_event() {
        let mut resources = ResourceContext::new();
        resources.insert(ActionPoints::new(3));
        resources.insert(EventBus::new());

        let mut system = ActionResetSystem::default();

        // Consume actions
        {
            let mut points = resources.get_mut::<ActionPoints>().await.unwrap();
            points.consume();
            assert_eq!(points.available, 2);
        }

        // Process without event
        system.process(&mut resources).await;

        // Points should not reset
        let points = resources.get::<ActionPoints>().await.unwrap();
        assert_eq!(points.available, 2);
    }

    #[tokio::test]
    async fn test_action_auto_advance_system_depleted() {
        let mut resources = ResourceContext::new();
        resources.insert(ActionPoints::new(2));
        resources.insert(EventBus::new());

        let mut system = ActionAutoAdvanceSystem::default();

        // Deplete all actions
        {
            let mut points = resources.get_mut::<ActionPoints>().await.unwrap();
            points.consume();
            points.consume();
            assert!(points.is_depleted());
        }

        // Process system
        system.process(&mut resources).await;

        // Check AdvanceTimeRequested published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let events: Vec<_> = bus.reader::<AdvanceTimeRequested>().iter().collect();
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_action_auto_advance_system_not_depleted() {
        let mut resources = ResourceContext::new();
        resources.insert(ActionPoints::new(3));
        resources.insert(EventBus::new());

        let mut system = ActionAutoAdvanceSystem::default();

        // Consume some but not all
        {
            let mut points = resources.get_mut::<ActionPoints>().await.unwrap();
            points.consume();
            assert!(!points.is_depleted());
        }

        // Process system
        system.process(&mut resources).await;

        // No event should be published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let events: Vec<_> = bus.reader::<AdvanceTimeRequested>().iter().collect();
        assert_eq!(events.len(), 0);
    }
}
