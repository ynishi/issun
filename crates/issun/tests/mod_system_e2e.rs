//! End-to-End Integration Test for MOD System
//!
//! This test validates the complete MOD system flow:
//! 1. Load a MOD (RhaiLoader)
//! 2. MOD calls plugin control API (enable_plugin, set_plugin_param)
//! 3. ModLoadSystem processes MOD commands
//! 4. PluginControlSystem converts commands to events
//! 5. ModBridgeSystem bridges events to plugin configs
//! 6. Plugin configs are updated correctly

use issun::context::ResourceContext;
use issun::engine::ModBridgeSystem;
use issun::event::EventBus;
use issun::plugin::{CombatConfig, InventoryConfig};
use issun::system::System;

// NOTE: The full end-to-end test with RhaiLoader is in issun-mod-rhai crate
// This file tests the ModBridgeSystem integration with the event system

#[tokio::test]
async fn test_mod_bridge_with_plugins() {
    // Setup ResourceContext with EventBus and plugin configs
    let mut resources = ResourceContext::new();
    resources.insert(EventBus::new());
    resources.insert(CombatConfig::default());
    resources.insert(InventoryConfig::default());

    // Simulate MOD events being published
    {
        let mut event_bus = resources
            .get_mut::<EventBus>()
            .await
            .expect("EventBus not found");

        // Enable combat
        event_bus.publish(issun::modding::events::PluginEnabledEvent {
            plugin_name: "combat".to_string(),
        });

        // Set combat parameters
        event_bus.publish(issun::modding::events::PluginParameterChangedEvent {
            plugin_name: "combat".to_string(),
            key: "max_hp".to_string(),
            value: serde_json::json!(200),
        });

        event_bus.publish(issun::modding::events::PluginParameterChangedEvent {
            plugin_name: "combat".to_string(),
            key: "difficulty".to_string(),
            value: serde_json::json!(3.0),
        });

        // Disable inventory
        event_bus.publish(issun::modding::events::PluginDisabledEvent {
            plugin_name: "inventory".to_string(),
        });

        // Dispatch events
        event_bus.dispatch();
    }

    // Run ModBridgeSystem
    let mut bridge_system = ModBridgeSystem::new();
    bridge_system.update_resources(&mut resources).await;

    // Verify combat config was updated
    {
        let combat_config = resources
            .get::<CombatConfig>()
            .await
            .expect("Combat config not found");
        assert!(combat_config.enabled, "Combat should be enabled");
        assert_eq!(
            combat_config.default_max_hp, 200,
            "Combat max_hp should be 200"
        );
        assert_eq!(
            combat_config.difficulty_multiplier, 3.0,
            "Combat difficulty should be 3.0"
        );
    }

    // Verify inventory config was updated
    {
        let inventory_config = resources
            .get::<InventoryConfig>()
            .await
            .expect("Inventory config not found");
        assert!(
            !inventory_config.enabled,
            "Inventory should be disabled"
        );
    }
}

#[tokio::test]
async fn test_namespaced_plugin_names() {
    let mut resources = ResourceContext::new();
    resources.insert(EventBus::new());
    resources.insert(CombatConfig::default());

    // Use namespaced plugin name
    {
        let mut event_bus = resources
            .get_mut::<EventBus>()
            .await
            .expect("EventBus not found");
        event_bus.publish(issun::modding::events::PluginEnabledEvent {
            plugin_name: "issun:combat".to_string(), // Namespaced
        });
        event_bus.dispatch();
    }

    let mut bridge_system = ModBridgeSystem::new();
    bridge_system.update_resources(&mut resources).await;

    {
        let combat_config = resources
            .get::<CombatConfig>()
            .await
            .expect("Combat config not found");
        assert!(
            combat_config.enabled,
            "Combat should be enabled via namespaced name"
        );
    }
}

#[tokio::test]
async fn test_unknown_plugin_handling() {
    let mut resources = ResourceContext::new();
    resources.insert(EventBus::new());

    // Try to enable unknown plugin
    {
        let mut event_bus = resources
            .get_mut::<EventBus>()
            .await
            .expect("EventBus not found");
        event_bus.publish(issun::modding::events::PluginEnabledEvent {
            plugin_name: "unknown_plugin".to_string(),
        });
        event_bus.dispatch();
    }

    let mut bridge_system = ModBridgeSystem::new();
    // Should not panic, just log error
    bridge_system.update_resources(&mut resources).await;
}

#[tokio::test]
async fn test_unknown_parameter_handling() {
    let mut resources = ResourceContext::new();
    resources.insert(EventBus::new());
    resources.insert(CombatConfig::default());

    // Try to set unknown parameter
    {
        let mut event_bus = resources
            .get_mut::<EventBus>()
            .await
            .expect("EventBus not found");
        event_bus.publish(issun::modding::events::PluginParameterChangedEvent {
            plugin_name: "combat".to_string(),
            key: "unknown_param".to_string(),
            value: serde_json::json!(999),
        });
        event_bus.dispatch();
    }

    let mut bridge_system = ModBridgeSystem::new();
    // Should not panic, just log error
    bridge_system.update_resources(&mut resources).await;

    // Config should remain unchanged
    {
        let combat_config = resources
            .get::<CombatConfig>()
            .await
            .expect("Combat config not found");
        assert_eq!(combat_config.default_max_hp, 100); // Default value
    }
}
