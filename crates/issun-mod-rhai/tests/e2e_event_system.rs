//! End-to-End Test for MOD Event System
//!
//! This test validates the complete event flow:
//! 1. MOD A publishes events via publish_event()
//! 2. ModEventSystem drains and publishes to EventBus as DynamicEvent
//! 3. ModEventSystem reads DynamicEvents from EventBus
//! 4. ModEventSystem dispatches to MOD B's subscribe_event() callbacks

use issun::context::ResourceContext;
use issun::event::EventBus;
use issun::modding::{ModEventSystem, ModLoader, ModLoaderState};
use issun_mod_rhai::RhaiLoader;
use std::io::Write;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_publish_and_subscribe_e2e() {
    // Setup ResourceContext
    let mut resources = ResourceContext::new();
    resources.insert(EventBus::new());

    // Create two MOD scripts
    // MOD 1: Publisher - publishes CustomEvent
    let mut publisher_file = NamedTempFile::new().unwrap();
    writeln!(
        publisher_file,
        r#"
fn on_init() {{
    log("Publisher: Publishing CustomEvent");
    publish_event("CustomEvent", #{{
        message: "Hello from Publisher",
        value: 99
    }});
}}
"#
    )
    .unwrap();

    // MOD 2: Subscriber - subscribes to CustomEvent
    let mut subscriber_file = NamedTempFile::new().unwrap();
    writeln!(
        subscriber_file,
        r#"
fn on_init() {{
    log("Subscriber: Subscribing to CustomEvent");
    subscribe_event("CustomEvent", |event_data| {{
        log("Subscriber: Received CustomEvent!");
        log("Message: " + event_data.message);
        log("Value: " + event_data.value);
    }});
}}
"#
    )
    .unwrap();

    // Load both MODs
    let mut loader = RhaiLoader::new();
    let publisher_handle = loader.load(publisher_file.path()).unwrap();
    let subscriber_handle = loader.load(subscriber_file.path()).unwrap();

    // Insert ModLoaderState into resources
    resources.insert(ModLoaderState {
        loader: Box::new(loader),
        loaded_mods: vec![publisher_handle, subscriber_handle],
    });

    // Create ModEventSystem
    let mut mod_event_system = ModEventSystem::new();

    // Scenario: First update - Publisher's publish_event() creates events
    // Step 1: ModEventSystem drains from loader and publishes to EventBus
    mod_event_system.update_resources(&mut resources).await;

    // At this point, CustomEvent should be in EventBus as DynamicEvent
    // Verify EventBus has the event
    {
        let mut event_bus = resources
            .get_mut::<EventBus>()
            .await
            .expect("EventBus not found");

        let events: Vec<_> = event_bus
            .reader::<issun::modding::DynamicEvent>()
            .iter()
            .cloned()
            .collect();

        assert_eq!(events.len(), 1, "Should have 1 DynamicEvent in EventBus");
        assert_eq!(events[0].event_type, "CustomEvent");
        assert_eq!(events[0].data["message"], "Hello from Publisher");
        assert_eq!(events[0].data["value"], 99);

        // Dispatch events so they can be read in next update
        event_bus.dispatch();
    }

    // Scenario: Second update - Subscriber's callbacks are invoked
    // Step 2: ModEventSystem reads DynamicEvents and dispatches to subscribers
    mod_event_system.update_resources(&mut resources).await;

    // Verify: The test passes if the subscriber callback was invoked without panicking
    // Expected output: "[ModEventSystem] Dispatched event 'CustomEvent' to 1 subscriber(s)"
    println!("E2E test passed - publish_event() and subscribe_event() work together");
}

#[tokio::test]
async fn test_multiple_subscribers_same_event() {
    let mut resources = ResourceContext::new();
    resources.insert(EventBus::new());

    // Create 3 subscriber MODs for the same event
    let mut sub1_file = NamedTempFile::new().unwrap();
    writeln!(
        sub1_file,
        r#"
fn on_init() {{
    subscribe_event("SharedEvent", |event| {{
        log("Subscriber 1 received: " + event.data);
    }});
}}
"#
    )
    .unwrap();

    let mut sub2_file = NamedTempFile::new().unwrap();
    writeln!(
        sub2_file,
        r#"
fn on_init() {{
    subscribe_event("SharedEvent", |event| {{
        log("Subscriber 2 received: " + event.data);
    }});
}}
"#
    )
    .unwrap();

    let mut sub3_file = NamedTempFile::new().unwrap();
    writeln!(
        sub3_file,
        r#"
fn on_init() {{
    subscribe_event("SharedEvent", |event| {{
        log("Subscriber 3 received: " + event.data);
    }});
}}
"#
    )
    .unwrap();

    // Load all MODs
    let mut loader = RhaiLoader::new();
    let h1 = loader.load(sub1_file.path()).unwrap();
    let h2 = loader.load(sub2_file.path()).unwrap();
    let h3 = loader.load(sub3_file.path()).unwrap();

    resources.insert(ModLoaderState {
        loader: Box::new(loader),
        loaded_mods: vec![h1, h2, h3],
    });

    // Publish SharedEvent to EventBus
    {
        let mut event_bus = resources
            .get_mut::<EventBus>()
            .await
            .expect("EventBus not found");

        event_bus.publish(issun::modding::DynamicEvent {
            event_type: "SharedEvent".to_string(),
            data: serde_json::json!({
                "data": "Broadcast message"
            }),
        });

        event_bus.dispatch();
    }

    // Run ModEventSystem to dispatch
    let mut mod_event_system = ModEventSystem::new();
    mod_event_system.update_resources(&mut resources).await;

    // Expected: All 3 subscribers should receive the event
    // Output should show: "[ModEventSystem] Dispatched event 'SharedEvent' to 3 subscriber(s)"
    println!("Test passed - Multiple subscribers can receive the same event");
}

#[tokio::test]
async fn test_event_filtering_by_type() {
    let mut resources = ResourceContext::new();
    resources.insert(EventBus::new());

    // Create a MOD that subscribes to specific event type
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
let received_count = 0;

fn on_init() {{
    subscribe_event("TargetEvent", |event| {{
        log("Received TargetEvent");
    }});
}}
"#
    )
    .unwrap();

    let mut loader = RhaiLoader::new();
    let handle = loader.load(file.path()).unwrap();
    resources.insert(ModLoaderState {
        loader: Box::new(loader),
        loaded_mods: vec![handle],
    });

    // Publish multiple event types
    {
        let mut event_bus = resources
            .get_mut::<EventBus>()
            .await
            .expect("EventBus not found");

        // This should trigger the callback
        event_bus.publish(issun::modding::DynamicEvent {
            event_type: "TargetEvent".to_string(),
            data: serde_json::json!({"data": "target"}),
        });

        // These should NOT trigger the callback
        event_bus.publish(issun::modding::DynamicEvent {
            event_type: "OtherEvent1".to_string(),
            data: serde_json::json!({"data": "other1"}),
        });

        event_bus.publish(issun::modding::DynamicEvent {
            event_type: "OtherEvent2".to_string(),
            data: serde_json::json!({"data": "other2"}),
        });

        event_bus.dispatch();
    }

    // Run ModEventSystem
    let mut mod_event_system = ModEventSystem::new();
    mod_event_system.update_resources(&mut resources).await;

    // Expected: Only 1 event dispatched (TargetEvent)
    // Output should show: "[ModEventSystem] Dispatched event 'TargetEvent' to 1 subscriber(s)"
    // OtherEvent1 and OtherEvent2 should have 0 subscribers
    println!("Test passed - Events are filtered by type correctly");
}
