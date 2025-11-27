//! Time management systems

use crate::context::{ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;

use super::events::{AdvanceTimeRequested, DayChanged};
use super::resources::GameTimer;

/// System that handles time advancement requests
///
/// Listens for `AdvanceTimeRequested` events and increments the day counter,
/// then publishes `DayChanged` events for other systems to react to.
///
/// This keeps business logic in systems rather than scene layers.
#[derive(Default)]
pub struct TimerSystem;

impl TimerSystem {
    /// Update method - processes time advancement requests
    ///
    /// This method is called with ResourceContext access for managing timer state.
    pub async fn update(&mut self, _services: &ServiceContext, resources: &mut ResourceContext) {
        // Check for time advancement requests
        let advance_requested = if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            let reader = bus.reader::<AdvanceTimeRequested>();
            let has_events = reader.iter().count() > 0;
            has_events
        } else {
            false
        };

        if !advance_requested {
            return;
        }

        // Increment day
        let new_day = if let Some(mut timer) = resources.get_mut::<GameTimer>().await {
            timer.increment_day()
        } else {
            return;
        };

        // Publish DayChanged event
        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(DayChanged { day: new_day });
        }
    }
}

#[async_trait]
impl System for TimerSystem {
    fn name(&self) -> &'static str {
        "timer"
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
    use crate::context::{ResourceContext, ServiceContext};
    use crate::event::EventBus;

    #[tokio::test]
    async fn test_timer_system_advances_day() {
        let mut resources = ResourceContext::new();
        resources.insert(GameTimer::new());
        resources.insert(EventBus::new());

        let services = ServiceContext::new();
        let mut system = TimerSystem;

        // Publish advancement request
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(AdvanceTimeRequested);
            bus.dispatch(); // Make events visible to readers
        }

        // Process system
        system.update(&services, &mut resources).await;

        // Dispatch DayChanged events to make them visible
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.dispatch();
        }

        // Check day incremented
        let timer = resources.get::<GameTimer>().await.unwrap();
        assert_eq!(timer.day, 2);

        // Check DayChanged event published
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<DayChanged>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].day, 2);
    }

    #[tokio::test]
    async fn test_timer_system_no_advancement_without_request() {
        let mut resources = ResourceContext::new();
        resources.insert(GameTimer::new());
        resources.insert(EventBus::new());

        let services = ServiceContext::new();
        let mut system = TimerSystem;

        // Process without request
        system.update(&services, &mut resources).await;

        // Day should not change
        let timer = resources.get::<GameTimer>().await.unwrap();
        assert_eq!(timer.day, 1);

        // No DayChanged events
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        let reader = bus.reader::<DayChanged>();
        let events: Vec<_> = reader.iter().collect();
        assert_eq!(events.len(), 0);
    }

    #[tokio::test]
    async fn test_timer_system_multiple_requests() {
        let mut resources = ResourceContext::new();
        resources.insert(GameTimer::new());
        resources.insert(EventBus::new());

        let services = ServiceContext::new();
        let mut system = TimerSystem;

        // Publish multiple requests
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(AdvanceTimeRequested);
            bus.publish(AdvanceTimeRequested);
            bus.dispatch(); // Make events visible to readers
        }

        // Process system (should only increment once per update call)
        system.update(&services, &mut resources).await;

        // Dispatch to make DayChanged events visible
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.dispatch();
        }

        let timer = resources.get::<GameTimer>().await.unwrap();
        assert_eq!(timer.day, 2); // Only incremented once
        drop(timer);

        // Publish another and process again
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(AdvanceTimeRequested);
            bus.dispatch(); // Make events visible to readers
        }

        system.update(&services, &mut resources).await;

        // Dispatch again
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.dispatch();
        }

        let timer = resources.get::<GameTimer>().await.unwrap();
        assert_eq!(timer.day, 3);
    }
}
