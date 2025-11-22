//! Hook pattern for game-specific customization

use super::state::Contagion;
use super::topology::PropagationEdge;
use super::types::{ContagionContent, NodeId};
use async_trait::async_trait;

/// Hook for game-specific contagion behavior
///
/// Customize how contagions spread through your game world.
#[async_trait]
pub trait ContagionHook: Send + Sync {
    /// Called when contagion spreads to a new node
    ///
    /// Use this to:
    /// - Log spread events
    /// - Update game state
    /// - Trigger notifications
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn on_contagion_spread(
    ///     &self,
    ///     contagion: &Contagion,
    ///     from_node: &NodeId,
    ///     to_node: &NodeId,
    /// ) {
    ///     println!("{} spread from {} to {}",
    ///         contagion.id, from_node, to_node);
    /// }
    /// ```
    async fn on_contagion_spread(
        &self,
        _contagion: &Contagion,
        _from_node: &NodeId,
        _to_node: &NodeId,
    ) {
        // Default: no-op
    }

    /// Custom mutation logic for specific content types
    ///
    /// Return Some(content) to replace the mutated content,
    /// or None to use default mutation.
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn mutate_custom_content(
    ///     &self,
    ///     content: &ContagionContent,
    ///     noise_level: f32,
    /// ) -> Option<ContagionContent> {
    ///     match content {
    ///         ContagionContent::Custom { key, data } if key == "my_type" => {
    ///             // Custom mutation logic
    ///             Some(ContagionContent::Custom {
    ///                 key: key.clone(),
    ///                 data: mutated_data,
    ///             })
    ///         }
    ///         _ => None
    ///     }
    /// }
    /// ```
    async fn mutate_custom_content(
        &self,
        _content: &ContagionContent,
        _noise_level: f32,
    ) -> Option<ContagionContent> {
        // Default: no custom mutation
        None
    }

    /// Modify transmission rate based on game state
    ///
    /// Return modified transmission rate.
    /// Can be used to implement:
    /// - Seasonal effects (winter reduces spread)
    /// - Event-based modifiers (quarantine zones)
    /// - Special node properties (airports increase spread)
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn modify_transmission_rate(
    ///     &self,
    ///     base_rate: f32,
    ///     edge: &PropagationEdge,
    ///     contagion: &Contagion,
    /// ) -> f32 {
    ///     // Airports spread faster
    ///     if edge.from.contains("airport") {
    ///         base_rate * 2.0
    ///     } else {
    ///         base_rate
    ///     }
    /// }
    /// ```
    async fn modify_transmission_rate(
        &self,
        base_rate: f32,
        _edge: &PropagationEdge,
        _contagion: &Contagion,
    ) -> f32 {
        // Default: no modification
        base_rate
    }
}

/// Default no-op hook implementation
pub struct DefaultContagionHook;

#[async_trait]
impl ContagionHook for DefaultContagionHook {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::contagion::types::DiseaseLevel;

    #[tokio::test]
    async fn test_default_hook_no_spread_event() {
        let hook = DefaultContagionHook;

        let contagion = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        );

        // Should not panic
        hook.on_contagion_spread(&contagion, &"london".to_string(), &"paris".to_string())
            .await;
    }

    #[tokio::test]
    async fn test_default_hook_no_custom_mutation() {
        let hook = DefaultContagionHook;

        let content = ContagionContent::Custom {
            key: "test".to_string(),
            data: serde_json::json!({"value": 42}),
        };

        let result = hook.mutate_custom_content(&content, 0.5).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_default_hook_no_rate_modification() {
        let hook = DefaultContagionHook;

        let edge = PropagationEdge::new("e1", "a", "b", 0.8);
        let contagion = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        );

        let modified = hook.modify_transmission_rate(0.5, &edge, &contagion).await;
        assert_eq!(modified, 0.5);
    }

    #[tokio::test]
    async fn test_custom_hook_implementation() {
        struct TestHook {
            spread_multiplier: f32,
        }

        #[async_trait]
        impl ContagionHook for TestHook {
            async fn modify_transmission_rate(
                &self,
                base_rate: f32,
                _edge: &PropagationEdge,
                _contagion: &Contagion,
            ) -> f32 {
                base_rate * self.spread_multiplier
            }
        }

        let hook = TestHook {
            spread_multiplier: 2.0,
        };

        let edge = PropagationEdge::new("e1", "a", "b", 0.8);
        let contagion = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        );

        let modified = hook.modify_transmission_rate(0.5, &edge, &contagion).await;
        assert_eq!(modified, 1.0);
    }
}
