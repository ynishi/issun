//! EntropyPluginECS - ECS-based entropy/decay system
//!
//! High-performance implementation using hecs and parallel processing.

use super::config::EntropyConfig;
use super::hook_ecs::{DefaultEntropyHookECS, EntropyHookECS};
use super::service::EntropyService;
use super::state_ecs::EntropyStateECS;
use super::system_ecs::EntropySystemECS;
use issun_macros::Plugin;
use std::sync::Arc;

/// EntropyPluginECS - ECS-based entropy system for large-scale games
///
/// # Example
///
/// ```ignore
/// use issun::plugin::entropy::EntropyPluginECS;
/// use issun::plugin::entropy::EntropyConfig;
///
/// let entropy = EntropyPluginECS::new()
///     .with_config(EntropyConfig {
///         global_decay_multiplier: 1.5,
///         ..Default::default()
///     });
/// ```
#[derive(Plugin)]
#[plugin(name = "issun:entropy_ecs")]
pub struct EntropyPluginECS {
    #[plugin(skip)]
    hook: Arc<dyn EntropyHookECS>,

    #[plugin(resource)]
    config: EntropyConfig,

    #[plugin(runtime_state)]
    state: EntropyStateECS,

    #[plugin(service)]
    service: EntropyService,

    #[plugin(system)]
    system: EntropySystemECS,
}

impl EntropyPluginECS {
    /// Create new EntropyPluginECS with default configuration
    pub fn new() -> Self {
        let hook = Arc::new(DefaultEntropyHookECS);
        Self {
            hook: hook.clone(),
            config: EntropyConfig::default(),
            state: EntropyStateECS::default(),
            service: EntropyService,
            system: EntropySystemECS::new(hook),
        }
    }

    /// Set custom hook for game-specific behavior
    pub fn with_hook<H: EntropyHookECS + 'static>(mut self, hook: H) -> Self {
        let hook = Arc::new(hook);
        self.hook = hook.clone();
        self.system = EntropySystemECS::new(hook);
        self
    }

    /// Set custom configuration
    pub fn with_config(mut self, config: EntropyConfig) -> Self {
        self.config = config;
        self
    }

    /// Get reference to config
    pub fn config(&self) -> &EntropyConfig {
        &self.config
    }

    /// Get reference to state
    pub fn state(&self) -> &EntropyStateECS {
        &self.state
    }

    /// Get mutable reference to state
    pub fn state_mut(&mut self) -> &mut EntropyStateECS {
        &mut self.state
    }

    /// Get reference to system
    pub fn system(&self) -> &EntropySystemECS {
        &self.system
    }

    /// Get mutable reference to system
    pub fn system_mut(&mut self) -> &mut EntropySystemECS {
        &mut self.system
    }
}

impl Default for EntropyPluginECS {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = EntropyPluginECS::new();

        assert_eq!(plugin.config().global_decay_multiplier, 1.0);
        assert_eq!(plugin.state().entity_count(), 0);
    }

    #[test]
    fn test_plugin_with_config() {
        let config = EntropyConfig {
            global_decay_multiplier: 2.0,
            auto_destroy_on_zero: false,
            max_decay_events: 500,
            ..Default::default()
        };

        let plugin = EntropyPluginECS::new().with_config(config);

        assert_eq!(plugin.config().global_decay_multiplier, 2.0);
        assert!(!plugin.config().auto_destroy_on_zero);
        assert_eq!(plugin.config().max_decay_events, 500);
    }

    #[tokio::test]
    async fn test_plugin_basic_workflow() {
        // Test directly with system and state (not via plugin to avoid borrow issues)
        let hook = Arc::new(DefaultEntropyHookECS);
        let mut system = EntropySystemECS::new(hook);
        let mut state = EntropyStateECS::new();
        let config = EntropyConfig::default();

        // Spawn entities
        let entity1 = state.spawn_entity(
            Durability::new(100.0, 0.01, MaterialType::Metal),
            EnvironmentalExposure::default(),
        );

        let entity2 = state.spawn_entity(
            Durability::new(100.0, 0.02, MaterialType::Organic),
            EnvironmentalExposure::default(),
        );

        assert_eq!(state.entity_count(), 2);

        // Update decay
        system.update_decay(&mut state, &config, 1.0).await;

        // Check entities decayed
        let dur1 = state.world.get::<&Durability>(entity1).unwrap();
        assert!(dur1.current < 100.0);

        let dur2 = state.world.get::<&Durability>(entity2).unwrap();
        assert!(dur2.current < 100.0);
        assert!(dur2.current < dur1.current); // Organic decays faster

        // Check metrics
        let metrics = state.metrics();
        assert_eq!(metrics.entities_processed, 2);
    }

    #[tokio::test]
    async fn test_plugin_repair() {
        // Test directly with system and state (not via plugin)
        let hook = Arc::new(DefaultEntropyHookECS);
        let mut system = EntropySystemECS::new(hook);
        let mut state = EntropyStateECS::new();

        // Spawn damaged entity
        let entity = state.spawn_entity(
            Durability {
                current: 50.0,
                max: 100.0,
                decay_rate: 0.01,
                material: MaterialType::Metal,
                status: DurabilityStatus::Worn,
            },
            EnvironmentalExposure::default(),
        );

        // Repair
        let repaired = system
            .repair_entity(entity, 30.0, &mut state)
            .await
            .unwrap();

        assert_eq!(repaired, 30.0);

        let durability = state.world.get::<&Durability>(entity).unwrap();
        assert_eq!(durability.current, 80.0);
        assert_eq!(durability.status, DurabilityStatus::Intact);
    }
}
