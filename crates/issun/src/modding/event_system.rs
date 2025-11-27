//! MOD Event System
//!
//! Processes event subscriptions from MODs and dispatches events to them.

use crate::event::EventBus;
use crate::modding::{DynamicEvent, ModLoaderState};
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;

/// System for processing MOD event subscriptions
///
/// This system:
/// 1. Publishes MOD-generated events to EventBus as DynamicEvent
/// 2. Collects DynamicEvents from EventBus
/// 3. Matches them against MOD subscriptions
/// 4. Calls MOD callbacks with event data
pub struct ModEventSystem;

impl ModEventSystem {
    pub fn new() -> Self {
        Self
    }

    /// Update method using ResourceContext (Modern API)
    ///
    /// This method is the recommended way to update the system.
    pub async fn update_resources(&mut self, resources: &mut crate::context::ResourceContext) {
        // Step 1: Drain events from MOD loader and publish to EventBus
        let events_to_publish = {
            if let Some(mut loader_state) = resources.get_mut::<ModLoaderState>().await {
                loader_state.loader.drain_events()
            } else {
                Vec::new()
            }
        };

        // Publish drained events as DynamicEvent to EventBus
        if !events_to_publish.is_empty() {
            if let Some(mut event_bus) = resources.get_mut::<EventBus>().await {
                for (event_type, data) in events_to_publish {
                    event_bus.publish(DynamicEvent {
                        event_type: event_type.clone(),
                        data,
                    });
                }
                // Dispatch so events are available for reading
                event_bus.dispatch();
            }
        }

        // Step 2: Collect DynamicEvents from EventBus and dispatch to subscribers
        let dynamic_events: Vec<DynamicEvent> = {
            if let Some(mut event_bus) = resources.get_mut::<EventBus>().await {
                event_bus.reader::<DynamicEvent>().iter().cloned().collect()
            } else {
                Vec::new()
            }
        };

        // Step 3: Dispatch events to MOD subscribers
        if !dynamic_events.is_empty() {
            if let Some(mut loader_state) = resources.get_mut::<ModLoaderState>().await {
                for event in &dynamic_events {
                    let count = loader_state
                        .loader
                        .dispatch_event(&event.event_type, &event.data);
                    if count > 0 {
                        println!(
                            "[ModEventSystem] Dispatched event '{}' to {} subscriber(s)",
                            event.event_type, count
                        );
                    }
                }
            }
        }
    }
}

#[async_trait]
impl System for ModEventSystem {
    fn name(&self) -> &'static str {
        "mod_event_system"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
