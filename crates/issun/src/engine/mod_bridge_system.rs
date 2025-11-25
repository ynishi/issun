//! MOD Bridge System
//!
//! This system bridges MOD events to Plugin configurations, enabling runtime control
//! of plugins through MOD scripts.

use crate::context::{Context, ResourceContext};
use crate::event::EventBus;
use crate::modding::events::*;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;

/// System that bridges MOD events to Plugin configurations
///
/// This system listens to MOD-issued events (PluginEnabledEvent, PluginDisabledEvent,
/// PluginParameterChangedEvent) and updates plugin configurations accordingly.
///
/// # Supported Plugins
///
/// Currently supports:
/// - `combat` / `issun:combat` - Combat system
/// - `inventory` / `issun:inventory` - Inventory system
///
/// # Example
///
/// ```ignore
/// use issun::engine::ModBridgeSystem;
///
/// // Register in GameBuilder
/// builder.with_system(ModBridgeSystem::new());
/// ```
///
/// # MOD Usage
///
/// ```rhai
/// // In a MOD script
/// enable_plugin("combat");
/// set_plugin_param("combat", "max_hp", 150);
/// ```
pub struct ModBridgeSystem;

impl ModBridgeSystem {
    /// Create a new ModBridgeSystem
    pub fn new() -> Self {
        Self
    }

    /// Update method using ResourceContext (Modern pattern)
    ///
    /// This method is the recommended way to update the system.
    pub async fn update_resources(&mut self, resources: &mut ResourceContext) {
        // Step 1: Collect all MOD events
        let enabled_events: Vec<PluginEnabledEvent> = {
            if let Some(mut event_bus) = resources.get_mut::<EventBus>().await {
                event_bus
                    .reader::<PluginEnabledEvent>()
                    .iter()
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            }
        };

        let disabled_events: Vec<PluginDisabledEvent> = {
            if let Some(mut event_bus) = resources.get_mut::<EventBus>().await {
                event_bus
                    .reader::<PluginDisabledEvent>()
                    .iter()
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            }
        };

        let param_events: Vec<PluginParameterChangedEvent> = {
            if let Some(mut event_bus) = resources.get_mut::<EventBus>().await {
                event_bus
                    .reader::<PluginParameterChangedEvent>()
                    .iter()
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            }
        };

        // Step 2: Process enable events
        for event in enabled_events {
            Self::handle_enable_resources(resources, &event).await;
        }

        // Step 3: Process disable events
        for event in disabled_events {
            Self::handle_disable_resources(resources, &event).await;
        }

        // Step 4: Process parameter changes
        for event in param_events {
            Self::handle_parameter_change_resources(resources, &event).await;
        }
    }

    /// Handle plugin enable event (ResourceContext version)
    async fn handle_enable_resources(resources: &mut ResourceContext, event: &PluginEnabledEvent) {
        match Self::normalize_plugin_name(&event.plugin_name) {
            "combat" => {
                if let Some(mut config) = resources.get_mut::<crate::plugin::CombatConfig>().await {
                    config.enabled = true;
                    println!("[MOD Bridge] Enabled plugin: combat");
                } else {
                    eprintln!("[MOD Bridge] Combat config not found");
                }
            }
            "inventory" => {
                if let Some(mut config) = resources.get_mut::<crate::plugin::InventoryConfig>().await {
                    config.enabled = true;
                    println!("[MOD Bridge] Enabled plugin: inventory");
                } else {
                    eprintln!("[MOD Bridge] Inventory config not found");
                }
            }
            name => {
                eprintln!("[MOD Bridge] Plugin '{}' is not MOD-controllable yet", name);
            }
        }
    }

    /// Handle plugin disable event (ResourceContext version)
    async fn handle_disable_resources(resources: &mut ResourceContext, event: &PluginDisabledEvent) {
        match Self::normalize_plugin_name(&event.plugin_name) {
            "combat" => {
                if let Some(mut config) = resources.get_mut::<crate::plugin::CombatConfig>().await {
                    config.enabled = false;
                    println!("[MOD Bridge] Disabled plugin: combat");
                } else {
                    eprintln!("[MOD Bridge] Combat config not found");
                }
            }
            "inventory" => {
                if let Some(mut config) = resources.get_mut::<crate::plugin::InventoryConfig>().await {
                    config.enabled = false;
                    println!("[MOD Bridge] Disabled plugin: inventory");
                } else {
                    eprintln!("[MOD Bridge] Inventory config not found");
                }
            }
            name => {
                eprintln!("[MOD Bridge] Plugin '{}' is not MOD-controllable yet", name);
            }
        }
    }

    /// Handle parameter change event (ResourceContext version)
    async fn handle_parameter_change_resources(
        resources: &mut ResourceContext,
        event: &PluginParameterChangedEvent,
    ) {
        match Self::normalize_plugin_name(&event.plugin_name) {
            "combat" => Self::apply_combat_param_resources(resources, &event.key, &event.value).await,
            "inventory" => Self::apply_inventory_param_resources(resources, &event.key, &event.value).await,
            name => {
                eprintln!("[MOD Bridge] Plugin '{}' is not MOD-controllable yet", name);
            }
        }
    }

    /// Apply parameter to combat config (ResourceContext version)
    async fn apply_combat_param_resources(
        resources: &mut ResourceContext,
        key: &str,
        value: &serde_json::Value,
    ) {
        if let Some(mut config) = resources.get_mut::<crate::plugin::CombatConfig>().await {
            match key {
                "enabled" => {
                    if let Some(enabled) = value.as_bool() {
                        config.enabled = enabled;
                        println!("[MOD Bridge] Combat.enabled = {}", enabled);
                    }
                }
                "max_hp" => {
                    if let Some(hp) = value.as_i64() {
                        config.default_max_hp = hp as u32;
                        println!("[MOD Bridge] Combat.max_hp = {}", hp);
                    }
                }
                "difficulty" => {
                    if let Some(diff) = value.as_f64() {
                        config.difficulty_multiplier = diff as f32;
                        println!("[MOD Bridge] Combat.difficulty = {}", diff);
                    }
                }
                _ => {
                    eprintln!("[MOD Bridge] Unknown combat parameter: {}", key);
                }
            }
        } else {
            eprintln!("[MOD Bridge] Combat config not found");
        }
    }

    /// Apply parameter to inventory config (ResourceContext version)
    async fn apply_inventory_param_resources(
        resources: &mut ResourceContext,
        key: &str,
        value: &serde_json::Value,
    ) {
        if let Some(mut config) = resources.get_mut::<crate::plugin::InventoryConfig>().await {
            match key {
                "enabled" => {
                    if let Some(enabled) = value.as_bool() {
                        config.enabled = enabled;
                        println!("[MOD Bridge] Inventory.enabled = {}", enabled);
                    }
                }
                "max_slots" => {
                    if let Some(slots) = value.as_i64() {
                        config.default_capacity = slots as usize;
                        println!("[MOD Bridge] Inventory.max_slots = {}", slots);
                    }
                }
                "allow_stacking" => {
                    if let Some(allow) = value.as_bool() {
                        config.allow_stacking = allow;
                        println!("[MOD Bridge] Inventory.allow_stacking = {}", allow);
                    }
                }
                _ => {
                    eprintln!("[MOD Bridge] Unknown inventory parameter: {}", key);
                }
            }
        } else {
            eprintln!("[MOD Bridge] Inventory config not found");
        }
    }

    /// Normalize plugin name (handle both "combat" and "issun:combat")
    fn normalize_plugin_name(name: &str) -> &str {
        name.strip_prefix("issun:").unwrap_or(name)
    }
}

impl Default for ModBridgeSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl System for ModBridgeSystem {
    fn name(&self) -> &'static str {
        "mod_bridge_system"
    }

    async fn update(&mut self, _ctx: &mut Context) {
        // Legacy Context support (deprecated path)
        // Modern usage should call update_resources() directly
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
    use crate::context::ResourceContext;

    #[test]
    fn test_normalize_plugin_name() {
        assert_eq!(ModBridgeSystem::normalize_plugin_name("combat"), "combat");
        assert_eq!(
            ModBridgeSystem::normalize_plugin_name("issun:combat"),
            "combat"
        );
        assert_eq!(
            ModBridgeSystem::normalize_plugin_name("inventory"),
            "inventory"
        );
        assert_eq!(
            ModBridgeSystem::normalize_plugin_name("issun:inventory"),
            "inventory"
        );
    }

    #[tokio::test]
    async fn test_mod_bridge_system_creation() {
        let system = ModBridgeSystem::new();
        assert_eq!(system.name(), "mod_bridge_system");
    }

    #[tokio::test]
    async fn test_combat_plugin_enable() {
        let mut resources = ResourceContext::new();
        resources.insert(EventBus::new());
        resources.insert(crate::plugin::CombatConfig::default());

        // Publish enable event
        {
            let mut event_bus = resources.get_mut::<EventBus>().await.unwrap();
            event_bus.publish(PluginEnabledEvent {
                plugin_name: "combat".to_string(),
            });
            event_bus.dispatch();
        }

        // Run system
        let mut system = ModBridgeSystem::new();
        system.update_resources(&mut resources).await;

        // Check config was enabled
        let config = resources.get::<crate::plugin::CombatConfig>().await.unwrap();
        assert!(config.enabled);
    }

    #[tokio::test]
    async fn test_combat_plugin_disable() {
        let mut resources = ResourceContext::new();
        resources.insert(EventBus::new());
        resources.insert(crate::plugin::CombatConfig::default());

        // Publish disable event
        {
            let mut event_bus = resources.get_mut::<EventBus>().await.unwrap();
            event_bus.publish(PluginDisabledEvent {
                plugin_name: "combat".to_string(),
            });
            event_bus.dispatch();
        }

        // Run system
        let mut system = ModBridgeSystem::new();
        system.update_resources(&mut resources).await;

        // Check config was disabled
        let config = resources.get::<crate::plugin::CombatConfig>().await.unwrap();
        assert!(!config.enabled);
    }

    #[tokio::test]
    async fn test_combat_parameter_change() {
        let mut resources = ResourceContext::new();
        resources.insert(EventBus::new());
        resources.insert(crate::plugin::CombatConfig::default());

        // Publish parameter change event
        {
            let mut event_bus = resources.get_mut::<EventBus>().await.unwrap();
            event_bus.publish(PluginParameterChangedEvent {
                plugin_name: "combat".to_string(),
                key: "max_hp".to_string(),
                value: serde_json::json!(150),
            });
            event_bus.dispatch();
        }

        // Run system
        let mut system = ModBridgeSystem::new();
        system.update_resources(&mut resources).await;

        // Check config was updated
        let config = resources.get::<crate::plugin::CombatConfig>().await.unwrap();
        assert_eq!(config.default_max_hp, 150);
    }

    #[tokio::test]
    async fn test_combat_difficulty_change() {
        let mut resources = ResourceContext::new();
        resources.insert(EventBus::new());
        resources.insert(crate::plugin::CombatConfig::default());

        // Publish difficulty change event
        {
            let mut event_bus = resources.get_mut::<EventBus>().await.unwrap();
            event_bus.publish(PluginParameterChangedEvent {
                plugin_name: "combat".to_string(),
                key: "difficulty".to_string(),
                value: serde_json::json!(2.5),
            });
            event_bus.dispatch();
        }

        // Run system
        let mut system = ModBridgeSystem::new();
        system.update_resources(&mut resources).await;

        // Check config was updated
        let config = resources.get::<crate::plugin::CombatConfig>().await.unwrap();
        assert_eq!(config.difficulty_multiplier, 2.5);
    }

    #[tokio::test]
    async fn test_inventory_plugin_enable() {
        let mut resources = ResourceContext::new();
        resources.insert(EventBus::new());
        resources.insert(crate::plugin::InventoryConfig::default());

        // Publish enable event
        {
            let mut event_bus = resources.get_mut::<EventBus>().await.unwrap();
            event_bus.publish(PluginEnabledEvent {
                plugin_name: "inventory".to_string(),
            });
            event_bus.dispatch();
        }

        // Run system
        let mut system = ModBridgeSystem::new();
        system.update_resources(&mut resources).await;

        // Check config was enabled
        let config = resources
            .get::<crate::plugin::InventoryConfig>()
            .await
            .unwrap();
        assert!(config.enabled);
    }

    #[tokio::test]
    async fn test_inventory_parameter_change() {
        let mut resources = ResourceContext::new();
        resources.insert(EventBus::new());
        resources.insert(crate::plugin::InventoryConfig::default());

        // Publish parameter change event
        {
            let mut event_bus = resources.get_mut::<EventBus>().await.unwrap();
            event_bus.publish(PluginParameterChangedEvent {
                plugin_name: "inventory".to_string(),
                key: "max_slots".to_string(),
                value: serde_json::json!(50),
            });
            event_bus.dispatch();
        }

        // Run system
        let mut system = ModBridgeSystem::new();
        system.update_resources(&mut resources).await;

        // Check config was updated
        let config = resources
            .get::<crate::plugin::InventoryConfig>()
            .await
            .unwrap();
        assert_eq!(config.default_capacity, 50);
    }

    #[tokio::test]
    async fn test_namespaced_plugin_name() {
        let mut resources = ResourceContext::new();
        resources.insert(EventBus::new());
        resources.insert(crate::plugin::CombatConfig::default());

        // Use namespaced name "issun:combat"
        {
            let mut event_bus = resources.get_mut::<EventBus>().await.unwrap();
            event_bus.publish(PluginEnabledEvent {
                plugin_name: "issun:combat".to_string(),
            });
            event_bus.dispatch();
        }

        // Run system
        let mut system = ModBridgeSystem::new();
        system.update_resources(&mut resources).await;

        // Check config was enabled
        let config = resources.get::<crate::plugin::CombatConfig>().await.unwrap();
        assert!(config.enabled);
    }

    #[tokio::test]
    async fn test_multiple_events_in_one_update() {
        let mut resources = ResourceContext::new();
        resources.insert(EventBus::new());
        resources.insert(crate::plugin::CombatConfig::default());
        resources.insert(crate::plugin::InventoryConfig::default());

        // Publish multiple events
        {
            let mut event_bus = resources.get_mut::<EventBus>().await.unwrap();
            event_bus.publish(PluginEnabledEvent {
                plugin_name: "combat".to_string(),
            });
            event_bus.publish(PluginParameterChangedEvent {
                plugin_name: "combat".to_string(),
                key: "max_hp".to_string(),
                value: serde_json::json!(200),
            });
            event_bus.publish(PluginDisabledEvent {
                plugin_name: "inventory".to_string(),
            });
            event_bus.dispatch();
        }

        // Run system once
        let mut system = ModBridgeSystem::new();
        system.update_resources(&mut resources).await;

        // Check all changes were applied
        let combat_config = resources.get::<crate::plugin::CombatConfig>().await.unwrap();
        assert!(combat_config.enabled);
        assert_eq!(combat_config.default_max_hp, 200);

        let inventory_config = resources
            .get::<crate::plugin::InventoryConfig>()
            .await
            .unwrap();
        assert!(!inventory_config.enabled);
    }
}
