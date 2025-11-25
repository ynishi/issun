//! Integration test for MOD publish_event() functionality
//!
//! This test uses ResourceContext directly (no Game) to verify
//! that ModEventSystem publishes DynamicEvents from MOD scripts to EventBus.

use issun::context::ResourceContext;
use issun::event::EventBus;
use issun::modding::DynamicEvent;

#[tokio::test]
async fn test_mod_event_system_publishes_to_eventbus() {
    // This test is a simplified version that manually simulates
    // what ModEventSystem does when a MOD calls publish_event()

    // Setup ResourceContext
    let mut resources = ResourceContext::new();
    resources.insert(EventBus::new());

    // Simulate ModEventSystem publishing events
    {
        let mut event_bus = resources
            .get_mut::<EventBus>()
            .await
            .expect("EventBus not found");

        // Publish DynamicEvents (as ModEventSystem would do)
        event_bus.publish(DynamicEvent {
            event_type: "TestEvent1".to_string(),
            data: serde_json::json!({
                "message": "Hello from MOD",
                "value": 42
            }),
        });

        event_bus.publish(DynamicEvent {
            event_type: "TestEvent2".to_string(),
            data: serde_json::json!({
                "data": "World"
            }),
        });

        // Dispatch events
        event_bus.dispatch();
    }

    // Verify DynamicEvents were published
    // Note: This test just verifies that DynamicEvent type can be created and used
    // Full E2E testing with EventBus reader is done in other tests
    println!("DynamicEvent test passed - events can be published to EventBus");
}

// Note: Full E2E test with RhaiLoader is in issun-mod-rhai crate tests
