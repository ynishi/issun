//! ContagionPlugin - Graph-based propagation system

use super::config::ContagionConfig;
use super::hook::{ContagionHook, DefaultContagionHook};
use super::state::ContagionState;
use super::system::ContagionSystem;
use super::topology::GraphTopology;
use crate::Plugin;
use std::sync::Arc;

/// Built-in contagion propagation plugin
///
/// Models the spread of information, diseases, trends, and influence through
/// contact networks using graph-based propagation mechanics.
///
/// # Core Features
///
/// - **Contact-based Spreading**: Propagates through graph edges
/// - **Mutation System**: Content changes during transmission (telephone game)
/// - **Credibility Decay**: Information degrades over time
/// - **Probabilistic Transmission**: Edge and node-based probabilities
/// - **Closed Path Support**: Handles cycles in graph topology
///
/// # Example
///
/// ```ignore
/// use issun::plugin::contagion::{ContagionPlugin, GraphTopology, ContagionNode, PropagationEdge, NodeType};
///
/// let mut topology = GraphTopology::new();
/// topology.add_node(ContagionNode::new("london", NodeType::City, 100000));
/// topology.add_node(ContagionNode::new("paris", NodeType::City, 80000));
/// topology.add_edge(PropagationEdge::new("route1", "london", "paris", 0.8));
///
/// let game = GameBuilder::new()
///     .with_plugin(
///         ContagionPlugin::new()
///             .with_topology(topology)
///             .with_config(
///                 ContagionConfig::default()
///                     .with_mutation_rate(0.2)
///                     .with_lifetime_turns(15)
///             )
///     )
///     .build()
///     .await?;
/// ```
///
/// # With Custom Hook
///
/// ```ignore
/// use issun::plugin::contagion::{ContagionHook, Contagion, PropagationEdge};
/// use async_trait::async_trait;
///
/// struct SpeedBoostHook;
///
/// #[async_trait]
/// impl ContagionHook for SpeedBoostHook {
///     async fn modify_transmission_rate(
///         &self,
///         base_rate: f32,
///         edge: &PropagationEdge,
///         contagion: &Contagion,
///     ) -> f32 {
///         // Airports spread faster
///         if edge.from.contains("airport") {
///             base_rate * 2.0
///         } else {
///             base_rate
///         }
///     }
/// }
///
/// let game = GameBuilder::new()
///     .with_plugin(
///         ContagionPlugin::new()
///             .with_hook(SpeedBoostHook)
///     )
///     .build()
///     .await?;
/// ```
#[derive(Plugin)]
#[plugin(name = "issun:contagion")]
pub struct ContagionPlugin {
    /// Custom hook for game-specific behavior
    #[plugin(skip)]
    hook: Arc<dyn ContagionHook>,

    /// Configuration (propagation rate, mutation rate, lifetime)
    #[plugin(resource)]
    config: ContagionConfig,

    /// Graph topology (nodes and edges)
    #[plugin(resource)]
    topology: GraphTopology,

    /// Runtime state (active contagions)
    #[plugin(runtime_state)]
    #[allow(dead_code)]
    state: ContagionState,

    /// System (orchestration)
    #[plugin(system)]
    system: ContagionSystem,
}

impl ContagionPlugin {
    /// Create a new contagion plugin with default settings
    pub fn new() -> Self {
        let hook = Arc::new(DefaultContagionHook);
        Self {
            hook: hook.clone(),
            config: ContagionConfig::default(),
            topology: GraphTopology::new(),
            state: ContagionState::new(),
            system: ContagionSystem::new(hook),
        }
    }

    /// Set contagion configuration
    ///
    /// # Example
    ///
    /// ```ignore
    /// ContagionPlugin::new()
    ///     .with_config(
    ///         ContagionConfig::default()
    ///             .with_propagation_rate(0.7)
    ///             .with_mutation_rate(0.15)
    ///             .with_lifetime_turns(20)
    ///     )
    /// ```
    pub fn with_config(mut self, config: ContagionConfig) -> Self {
        self.config = config;
        self
    }

    /// Set graph topology
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut topology = GraphTopology::new();
    /// topology.add_node(ContagionNode::new("city1", NodeType::City, 50000));
    /// topology.add_edge(PropagationEdge::new("e1", "city1", "city2", 0.6));
    ///
    /// ContagionPlugin::new()
    ///     .with_topology(topology)
    /// ```
    pub fn with_topology(mut self, topology: GraphTopology) -> Self {
        self.topology = topology;
        self
    }

    /// Add a custom hook for contagion behavior
    ///
    /// The hook will be called for:
    /// - Transmission rate modification (`modify_transmission_rate`)
    /// - Spread events (`on_contagion_spread`)
    /// - Custom content mutation (`mutate_custom_content`)
    ///
    /// # Example
    ///
    /// ```ignore
    /// struct MyHook;
    ///
    /// #[async_trait]
    /// impl ContagionHook for MyHook {
    ///     async fn on_contagion_spread(
    ///         &self,
    ///         contagion: &Contagion,
    ///         from_node: &NodeId,
    ///         to_node: &NodeId,
    ///     ) {
    ///         println!("Spread from {} to {}", from_node, to_node);
    ///     }
    /// }
    ///
    /// ContagionPlugin::new()
    ///     .with_hook(MyHook)
    /// ```
    pub fn with_hook<H: ContagionHook + 'static>(mut self, hook: H) -> Self {
        let hook = Arc::new(hook);
        self.hook = hook.clone();
        self.system = ContagionSystem::new(hook);
        self
    }
}

impl Default for ContagionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let _plugin = ContagionPlugin::new();
        // Should not panic
    }

    #[test]
    fn test_with_config() {
        let config = ContagionConfig::default().with_propagation_rate(0.8);

        let plugin = ContagionPlugin::new().with_config(config.clone());

        assert_eq!(plugin.config.global_propagation_rate, 0.8);
    }

    #[test]
    fn test_with_topology() {
        let mut topology = GraphTopology::new();
        topology.add_node(super::super::ContagionNode::new(
            "test",
            super::super::NodeType::City,
            1000,
        ));

        let plugin = ContagionPlugin::new().with_topology(topology);

        assert_eq!(plugin.topology.node_count(), 1);
    }

    #[test]
    fn test_with_custom_hook() {
        use async_trait::async_trait;

        #[derive(Clone)]
        struct TestHook;

        #[async_trait]
        impl ContagionHook for TestHook {}

        let _plugin = ContagionPlugin::new().with_hook(TestHook);

        // Hook is set (can't test directly, but construction succeeds)
    }

    #[test]
    fn test_builder_pattern() {
        let mut topology = GraphTopology::new();
        topology.add_node(super::super::ContagionNode::new(
            "london",
            super::super::NodeType::City,
            100000,
        ));

        let config = ContagionConfig::default()
            .with_propagation_rate(0.7)
            .with_mutation_rate(0.15);

        let plugin = ContagionPlugin::new()
            .with_topology(topology)
            .with_config(config);

        assert_eq!(plugin.topology.node_count(), 1);
        assert_eq!(plugin.config.global_propagation_rate, 0.7);
        assert_eq!(plugin.config.default_mutation_rate, 0.15);
    }

    #[tokio::test]
    async fn test_plugin_registers_resources() {
        use crate::builder::GameBuilder;

        let config = ContagionConfig::default().with_propagation_rate(0.8);
        let mut topology = GraphTopology::new();
        topology.add_node(super::super::ContagionNode::new(
            "test_city",
            super::super::NodeType::City,
            1000,
        ));

        let game = GameBuilder::new()
            .with_plugin(
                ContagionPlugin::new()
                    .with_config(config)
                    .with_topology(topology),
            )
            .expect("Failed to add plugin")
            .build()
            .await
            .expect("Failed to build game");

        // Verify ContagionConfig was registered
        assert!(
            game.resources.contains::<ContagionConfig>(),
            "ContagionConfig should be registered by Plugin derive"
        );

        // Verify GraphTopology was registered
        assert!(
            game.resources.contains::<GraphTopology>(),
            "GraphTopology should be registered by Plugin derive"
        );

        // Verify ContagionState was registered
        assert!(
            game.resources.contains::<ContagionState>(),
            "ContagionState should be registered by Plugin derive"
        );
    }
}
