//! GenerationPluginECS - ECS-based generation/growth system
//!
//! High-performance implementation using hecs and parallel processing.

use super::config::GenerationConfig;
use super::hook_ecs::{DefaultGenerationHookECS, GenerationHookECS};
use super::service::GenerationService;
use super::state_ecs::GenerationStateECS;
use super::system_ecs::GenerationSystemECS;
use issun_macros::Plugin;
use std::sync::Arc;

/// GenerationPluginECS - ECS-based generation system for large-scale games
///
/// This plugin is the inverse of EntropyPlugin (negentropy/un-entropy).
/// Handles growth, construction, production, and recovery systems.
///
/// # Example
///
/// ```ignore
/// use issun::plugin::generation::GenerationPluginECS;
/// use issun::plugin::generation::GenerationConfig;
///
/// let generation = GenerationPluginECS::new()
///     .with_config(GenerationConfig {
///         global_generation_multiplier: 1.5,
///         ..Default::default()
///     });
/// ```
#[derive(Plugin)]
#[plugin(name = "issun:generation_ecs")]
pub struct GenerationPluginECS {
    #[plugin(skip)]
    hook: Arc<dyn GenerationHookECS>,

    #[plugin(resource)]
    config: GenerationConfig,

    #[plugin(runtime_state)]
    state: GenerationStateECS,

    #[plugin(service)]
    service: GenerationService,

    #[plugin(system)]
    system: GenerationSystemECS,
}

impl GenerationPluginECS {
    /// Create new GenerationPluginECS with default configuration
    pub fn new() -> Self {
        let hook = Arc::new(DefaultGenerationHookECS);
        Self {
            hook: hook.clone(),
            config: GenerationConfig::default(),
            state: GenerationStateECS::default(),
            service: GenerationService,
            system: GenerationSystemECS::new(hook),
        }
    }

    /// Set custom hook for game-specific behavior
    pub fn with_hook<H: GenerationHookECS + 'static>(mut self, hook: H) -> Self {
        let hook = Arc::new(hook);
        self.hook = hook.clone();
        self.system = GenerationSystemECS::new(hook);
        self
    }

    /// Set custom configuration
    pub fn with_config(mut self, config: GenerationConfig) -> Self {
        self.config = config;
        self
    }

    /// Get reference to config
    pub fn config(&self) -> &GenerationConfig {
        &self.config
    }

    /// Get reference to state
    pub fn state(&self) -> &GenerationStateECS {
        &self.state
    }

    /// Get mutable reference to state
    pub fn state_mut(&mut self) -> &mut GenerationStateECS {
        &mut self.state
    }

    /// Get reference to system
    pub fn system(&self) -> &GenerationSystemECS {
        &self.system
    }

    /// Get mutable reference to system
    pub fn system_mut(&mut self) -> &mut GenerationSystemECS {
        &mut self.system
    }
}

impl Default for GenerationPluginECS {
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
        let plugin = GenerationPluginECS::new();

        assert_eq!(plugin.config().global_generation_multiplier, 1.0);
        assert_eq!(plugin.state().entity_count(), 0);
    }

    #[test]
    fn test_plugin_with_config() {
        let config = GenerationConfig {
            global_generation_multiplier: 2.0,
            auto_remove_on_complete: true,
            max_generation_events: 500,
            ..Default::default()
        };

        let plugin = GenerationPluginECS::new().with_config(config);

        assert_eq!(plugin.config().global_generation_multiplier, 2.0);
        assert!(plugin.config().auto_remove_on_complete);
        assert_eq!(plugin.config().max_generation_events, 500);
    }

    #[tokio::test]
    async fn test_plugin_basic_workflow() {
        // Test directly with system and state (not via plugin to avoid borrow issues)
        let hook = Arc::new(DefaultGenerationHookECS);
        let mut system = GenerationSystemECS::new(hook);
        let mut state = GenerationStateECS::new();
        let config = GenerationConfig::default();

        // Spawn entities
        let entity1 = state.spawn_entity(
            Generation::new(100.0, 1.0, GenerationType::Organic),
            GenerationEnvironment::default(),
        );

        let entity2 = state.spawn_entity(
            Generation::new(100.0, 2.0, GenerationType::Production),
            GenerationEnvironment::default(),
        );

        assert_eq!(state.entity_count(), 2);

        // Update generation
        system.update_generation(&mut state, &config, 1.0).await;

        // Check entities generated
        let gen1 = state.world.get::<&Generation>(entity1).unwrap();
        assert!(gen1.current > 0.0);

        let gen2 = state.world.get::<&Generation>(entity2).unwrap();
        assert!(gen2.current > 0.0);
        assert!(gen2.current > gen1.current); // Production generates faster

        // Check metrics
        let metrics = state.metrics();
        assert_eq!(metrics.entities_processed, 2);
    }

    #[tokio::test]
    async fn test_plugin_reduce() {
        // Test directly with system and state (not via plugin)
        let hook = Arc::new(DefaultGenerationHookECS);
        let mut system = GenerationSystemECS::new(hook);
        let mut state = GenerationStateECS::new();

        // Spawn entity with 50% progress
        let entity = state.spawn_entity(
            Generation {
                current: 50.0,
                max: 100.0,
                generation_rate: 1.0,
                generation_type: GenerationType::Construction,
                status: GenerationStatus::Generating,
                paused: false,
            },
            GenerationEnvironment::default(),
        );

        // Reduce (damage/setback)
        let reduced = system
            .reduce_entity(entity, 20.0, &mut state)
            .await
            .unwrap();

        assert_eq!(reduced, 20.0);

        let generation = state.world.get::<&Generation>(entity).unwrap();
        assert_eq!(generation.current, 30.0);
        assert_eq!(generation.status, GenerationStatus::Generating);
    }

    #[tokio::test]
    async fn test_plugin_pause_resume() {
        // Test directly with system and state
        let hook = Arc::new(DefaultGenerationHookECS);
        let mut system = GenerationSystemECS::new(hook);
        let mut state = GenerationStateECS::new();

        let entity = state.spawn_entity(
            Generation::new(100.0, 1.0, GenerationType::Recovery),
            GenerationEnvironment::default(),
        );

        // Pause
        system.pause_entity(entity, &mut state).await.unwrap();

        {
            let generation = state.world.get::<&Generation>(entity).unwrap();
            assert!(generation.paused);
        }

        // Resume
        system.resume_entity(entity, &mut state).await.unwrap();

        {
            let generation = state.world.get::<&Generation>(entity).unwrap();
            assert!(!generation.paused);
        }
    }
}
