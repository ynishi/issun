//! SubjectiveRealityPlugin implementation
//!
//! This plugin provides the "Fog of War" / subjective reality system for games.
//! It separates "God's View (Ground Truth)" from "Faction's View (Perception)".

use super::config::PerceptionConfig;
use super::hook::{DefaultPerceptionHook, PerceptionHook};
use super::state::KnowledgeBoardRegistry;
use super::system::PerceptionSystem;
use crate::Plugin;
use std::sync::Arc;

/// Built-in subjective reality / fog-of-war plugin
///
/// This plugin provides per-faction perception filtering with:
/// - Accuracy-based noise generation (±0-30% noise range)
/// - Confidence decay over time (exponential decay)
/// - Per-faction knowledge boards (Blackboard pattern)
/// - Customizable hooks for game-specific logic
///
/// # Core Concept
///
/// ```text
/// ┌────────────┐
/// │ Ground     │ ← God's View (absolute truth)
/// │ Truth      │
/// └─────┬──────┘
///       │
///       ├─────→ Accuracy Filter (Hook determines accuracy)
///       │
///       ├─────→ Perception Service (adds noise)
///       │
///       ├─────→ Per-Faction Knowledge Board
///       │       (Faction A sees 950 troops)
///       │       (Faction B sees 800 troops)
///       │       (actual: 1000 troops)
///       │
///       └─────→ Confidence Decay (info becomes stale)
/// ```
///
/// # Hook Customization
///
/// You can provide a custom hook to add game-specific perception logic:
/// - Calculate faction-specific accuracy (spy networks, tech levels, distance)
/// - Generate misinformation (propaganda, deception)
/// - Calculate fact priority (limited memory capacity)
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::subjective_reality::{SubjectiveRealityPlugin, PerceptionConfig};
///
/// let game = GameBuilder::new()
///     .with_plugin(
///         SubjectiveRealityPlugin::new()
///             .with_config(
///                 PerceptionConfig::default()
///                     .with_default_accuracy(0.7)
///                     .with_decay_rate(0.05) // 5% decay per turn
///             )
///             .register_faction("player")
///             .register_faction("enemy_a")
///     )
///     .build()
///     .await?;
/// ```
///
/// # With Custom Hook
///
/// ```ignore
/// use issun::plugin::subjective_reality::{PerceptionHook, GroundTruthFact};
/// use async_trait::async_trait;
///
/// struct SpyNetworkHook {
///     spy_locations: HashMap<FactionId, Vec<String>>,
/// }
///
/// #[async_trait]
/// impl PerceptionHook for SpyNetworkHook {
///     async fn get_faction_accuracies(
///         &self,
///         truth: &GroundTruthFact,
///         boards: &KnowledgeBoardRegistry,
///     ) -> HashMap<FactionId, f32> {
///         let mut accuracies = HashMap::new();
///         for (faction_id, _) in boards.all_boards() {
///             let mut accuracy = 0.5; // base
///             if let Some(location) = &truth.location {
///                 if self.has_spy(faction_id, location) {
///                     accuracy = 0.95; // high accuracy with spy
///                 }
///             }
///             accuracies.insert(faction_id.clone(), accuracy);
///         }
///         accuracies
///     }
/// }
///
/// let game = GameBuilder::new()
///     .with_plugin(
///         SubjectiveRealityPlugin::new()
///             .with_hook(SpyNetworkHook { spy_locations })
///     )
///     .build()
///     .await?;
/// ```
#[derive(Plugin)]
#[plugin(name = "issun:subjective_reality")]
pub struct SubjectiveRealityPlugin {
    /// Custom hook for game-specific perception logic
    #[plugin(skip)]
    hook: Arc<dyn PerceptionHook>,

    /// Read-only configuration (decay rate, min confidence, etc.)
    #[plugin(resource)]
    config: PerceptionConfig,

    /// Runtime state: per-faction knowledge boards
    #[plugin(runtime_state)]
    registry: KnowledgeBoardRegistry,

    /// System: orchestrates perception updates and confidence decay
    #[plugin(system)]
    system: PerceptionSystem,
}

impl SubjectiveRealityPlugin {
    /// Create a new subjective reality plugin with default settings
    ///
    /// Uses the default hook (all factions get 0.7 accuracy) by default.
    /// Use `with_hook()` to add custom perception logic.
    pub fn new() -> Self {
        let hook = Arc::new(DefaultPerceptionHook);
        Self {
            hook: hook.clone(),
            config: PerceptionConfig::default(),
            registry: KnowledgeBoardRegistry::new(),
            system: PerceptionSystem::new(hook),
        }
    }

    /// Set perception configuration
    ///
    /// # Example
    ///
    /// ```ignore
    /// SubjectiveRealityPlugin::new()
    ///     .with_config(
    ///         PerceptionConfig::default()
    ///             .with_default_accuracy(0.8)
    ///             .with_decay_rate(0.1) // 10% decay per turn
    ///             .with_min_confidence(0.2) // remove facts below 20%
    ///     )
    /// ```
    pub fn with_config(mut self, config: PerceptionConfig) -> Self {
        self.config = config;
        self
    }

    /// Add a custom hook for perception behavior
    ///
    /// The hook will be called when:
    /// - Determining faction-specific accuracy (`get_faction_accuracies`)
    /// - Generating misinformation (`generate_misinformation`)
    /// - Calculating fact priority (`calculate_fact_priority`)
    ///
    /// # Arguments
    ///
    /// * `hook` - Implementation of PerceptionHook trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// struct CustomHook;
    ///
    /// #[async_trait]
    /// impl PerceptionHook for CustomHook {
    ///     async fn get_faction_accuracies(
    ///         &self,
    ///         truth: &GroundTruthFact,
    ///         boards: &KnowledgeBoardRegistry,
    ///     ) -> HashMap<FactionId, f32> {
    ///         // Custom logic based on spy networks, tech levels, etc.
    ///     }
    /// }
    ///
    /// SubjectiveRealityPlugin::new()
    ///     .with_hook(CustomHook)
    /// ```
    pub fn with_hook<H: PerceptionHook + 'static>(mut self, hook: H) -> Self {
        let hook = Arc::new(hook);
        self.hook = hook.clone();
        self.system = PerceptionSystem::new(hook);
        self
    }

    /// Register a faction to receive perception updates
    ///
    /// Creates an empty knowledge board for the faction.
    ///
    /// # Example
    ///
    /// ```ignore
    /// SubjectiveRealityPlugin::new()
    ///     .register_faction("player")
    ///     .register_faction("enemy_a")
    ///     .register_faction("enemy_b")
    /// ```
    pub fn register_faction(mut self, faction_id: impl Into<String>) -> Self {
        self.registry.register_faction(faction_id.into());
        self
    }

    /// Register multiple factions at once
    ///
    /// # Example
    ///
    /// ```ignore
    /// SubjectiveRealityPlugin::new()
    ///     .register_factions(vec!["player", "enemy_a", "enemy_b"])
    /// ```
    pub fn register_factions<I, S>(mut self, faction_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for faction_id in faction_ids {
            self.registry.register_faction(faction_id.into());
        }
        self
    }
}

impl Default for SubjectiveRealityPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = SubjectiveRealityPlugin::new();
        assert_eq!(plugin.registry.faction_count(), 0);
    }

    #[test]
    fn test_register_faction() {
        let plugin = SubjectiveRealityPlugin::new().register_faction("faction_a");

        assert_eq!(plugin.registry.faction_count(), 1);
        assert!(plugin.registry.has_faction(&"faction_a".into()));
    }

    #[test]
    fn test_register_multiple_factions() {
        let plugin = SubjectiveRealityPlugin::new()
            .register_faction("faction_a")
            .register_faction("faction_b")
            .register_faction("faction_c");

        assert_eq!(plugin.registry.faction_count(), 3);
        assert!(plugin.registry.has_faction(&"faction_a".into()));
        assert!(plugin.registry.has_faction(&"faction_b".into()));
        assert!(plugin.registry.has_faction(&"faction_c".into()));
    }

    #[test]
    fn test_register_factions_vec() {
        let plugin = SubjectiveRealityPlugin::new()
            .register_factions(vec!["faction_a", "faction_b", "faction_c"]);

        assert_eq!(plugin.registry.faction_count(), 3);
    }

    #[test]
    fn test_with_config() {
        let config = PerceptionConfig::default()
            .with_default_accuracy(0.9)
            .with_decay_rate(0.1);

        let plugin = SubjectiveRealityPlugin::new().with_config(config.clone());

        assert_eq!(plugin.config.default_accuracy, config.default_accuracy);
        assert_eq!(plugin.config.decay_rate, config.decay_rate);
    }

    #[test]
    fn test_with_custom_hook() {
        use super::super::hook::PerceptionHook;
        use async_trait::async_trait;

        #[derive(Clone)]
        struct TestHook;

        #[async_trait]
        impl PerceptionHook for TestHook {}

        let plugin = SubjectiveRealityPlugin::new().with_hook(TestHook);

        // Hook is properly set (can't test directly, but construction succeeds)
        assert_eq!(plugin.registry.faction_count(), 0);
    }

    #[test]
    fn test_builder_pattern() {
        let plugin = SubjectiveRealityPlugin::new()
            .with_config(
                PerceptionConfig::default()
                    .with_default_accuracy(0.8)
                    .with_decay_rate(0.05),
            )
            .register_factions(vec!["player", "enemy_a", "enemy_b"]);

        assert_eq!(plugin.config.default_accuracy, 0.8);
        assert_eq!(plugin.config.decay_rate, 0.05);
        assert_eq!(plugin.registry.faction_count(), 3);
    }
}
