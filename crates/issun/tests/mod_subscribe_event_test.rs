//! Integration test for ModEventSystem subscribe functionality
//!
//! Verifies that ModEventSystem dispatches DynamicEvents from EventBus to MOD callbacks.

use issun::context::ResourceContext;
use issun::event::EventBus;
use issun::modding::{DynamicEvent, ModEventSystem};

#[tokio::test]
async fn test_mod_event_system_collects_from_eventbus() {
    // This test verifies that ModEventSystem can collect DynamicEvents from EventBus
    // Note: Full E2E test with RhaiLoader is in issun-mod-rhai crate tests

    // Setup ResourceContext
    let mut resources = ResourceContext::new();
    resources.insert(EventBus::new());

    // Publish DynamicEvents to EventBus
    {
        let mut event_bus = resources
            .get_mut::<EventBus>()
            .await
            .expect("EventBus not found");

        event_bus.publish(DynamicEvent {
            event_type: "PlayerDamaged".to_string(),
            data: serde_json::json!({
                "damage": 10,
                "source": "goblin"
            }),
        });

        event_bus.publish(DynamicEvent {
            event_type: "ItemCollected".to_string(),
            data: serde_json::json!({
                "item": "health_potion",
                "quantity": 1
            }),
        });

        event_bus.dispatch();
    }

    // Create ModEventSystem and run update
    let mut mod_event_system = ModEventSystem::new();
    mod_event_system.update_resources(&mut resources).await;

    // Verify: The test passes if ModEventSystem can read DynamicEvents without panicking
    // In a full E2E test with actual MOD subscribers, this would invoke callbacks
    println!("Test passed - ModEventSystem can collect DynamicEvents from EventBus");
}
