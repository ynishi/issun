//! MOD Event System
//!
//! Processes event subscriptions from MODs and dispatches events to them.

use crate::context::Context;
use crate::event::EventBus;
use crate::modding::{ModLoaderState, DynamicEvent};
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
}

#[async_trait]
impl System for ModEventSystem {
    fn name(&self) -> &'static str {
        "mod_event_system"
    }

    async fn update(&mut self, ctx: &mut Context) {
        // Step 1: Drain events from MOD loader and publish to EventBus
        let events_to_publish = {
            if let Some(loader_state) = ctx.get_mut::<ModLoaderState>("mod_loader_state") {
                loader_state.loader.drain_events()
            } else {
                Vec::new()
            }
        };

        // Publish drained events as DynamicEvent to EventBus
        if !events_to_publish.is_empty() {
            if let Some(event_bus) = ctx.get_mut::<EventBus>("event_bus") {
                for (event_type, data) in events_to_publish {
                    event_bus.publish(DynamicEvent {
                        event_type: event_type.clone(),
                        data,
                    });
                }
            }
        }

        // TODO: Step 2: Collect DynamicEvents from EventBus and dispatch to subscribers
        // This will require:
        // 1. Reading DynamicEvent from EventBus
        // 2. Matching event_type against MOD subscriptions
        // 3. Calling RhaiLoader::call_event_callback() for each matching subscription
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
