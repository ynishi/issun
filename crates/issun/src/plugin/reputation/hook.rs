//! Hook trait for custom reputation behavior

use crate::context::ResourceContext;
use async_trait::async_trait;

use super::types::*;

/// Trait for custom reputation behavior
///
/// **Hook vs Event**:
/// - **Hook**: Synchronous, direct call, can modify resources, NO network replication
/// - **Event**: Asynchronous, Pub-Sub, network-friendly, for loose coupling
///
/// **Use Hook for**:
/// - Immediate calculations (e.g., delta modifiers)
/// - Direct resource modification (e.g., unlocking content)
/// - Performance critical paths
/// - Local machine only
///
/// **Use Event for**:
/// - Notifying other systems (e.g., UI updates)
/// - Network replication (multiplayer)
/// - Audit log / replay
///
/// # Example
///
/// ```ignore
/// use issun::plugin::reputation::{ReputationHook, SubjectId};
/// use issun::context::ResourceContext;
/// use async_trait::async_trait;
///
/// struct GameLogHook;
///
/// #[async_trait]
/// impl ReputationHook for GameLogHook {
///     async fn on_reputation_changed(
///         &self,
///         subject_id: &SubjectId,
///         old_score: f32,
///         new_score: f32,
///         delta: f32,
///         _category: Option<&str>,
///         resources: &mut ResourceContext,
///     ) {
///         if let Some(mut log) = resources.get_mut::<GameLog>().await {
///             log.record(format!(
///                 "Reputation with {} changed: {:.1} -> {:.1} ({:+.1})",
///                 subject_id, old_score, new_score, delta
///             ));
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait ReputationHook: Send + Sync {
    /// Called when reputation is changed
    ///
    /// This is called immediately after the score is updated,
    /// allowing you to modify other resources (e.g., trigger events, unlock content).
    ///
    /// # Arguments
    ///
    /// * `subject_id` - The subject whose reputation changed
    /// * `old_score` - Previous score
    /// * `new_score` - New score
    /// * `delta` - Change amount
    /// * `category` - Optional category
    /// * `resources` - Access to game resources for modification
    async fn on_reputation_changed(
        &self,
        _subject_id: &SubjectId,
        _old_score: f32,
        _new_score: f32,
        _delta: f32,
        _category: Option<&str>,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Called when reputation crosses a threshold
    ///
    /// This is useful for triggering events when reputation enters a new level
    /// (e.g., "You are now Allied with the Kingdom!")
    ///
    /// # Arguments
    ///
    /// * `subject_id` - The subject whose reputation changed
    /// * `old_threshold` - Previous threshold (if any)
    /// * `new_threshold` - New threshold
    /// * `resources` - Access to game resources for modification
    async fn on_threshold_crossed(
        &self,
        _subject_id: &SubjectId,
        _old_threshold: Option<&ReputationThreshold>,
        _new_threshold: &ReputationThreshold,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Validate whether a reputation change is allowed
    ///
    /// Return `Ok(())` to allow, `Err(reason)` to prevent.
    ///
    /// # Use Cases
    ///
    /// - Prevent reputation gain during "Scandal" event
    /// - Cap reputation based on player level
    /// - Require special items to improve reputation
    ///
    /// # Arguments
    ///
    /// * `subject_id` - The subject to modify
    /// * `delta` - Proposed change
    /// * `category` - Optional category
    /// * `resources` - Access to game resources (read-only for validation)
    ///
    /// # Returns
    ///
    /// `Ok(())` if change is allowed, `Err(reason)` if prevented
    async fn validate_change(
        &self,
        _subject_id: &SubjectId,
        _delta: f32,
        _category: Option<&str>,
        _resources: &ResourceContext,
    ) -> Result<(), String> {
        // Default: always allow
        Ok(())
    }

    /// Calculate effective delta with game-specific modifiers
    ///
    /// This allows games to apply multipliers or bonuses.
    ///
    /// # Examples
    ///
    /// - Double reputation gain during "Festival" event
    /// - Reduce reputation loss with "Diplomat" trait
    /// - Apply faction-specific modifiers
    ///
    /// # Arguments
    ///
    /// * `subject_id` - The subject to modify
    /// * `base_delta` - Base reputation change
    /// * `category` - Optional category
    /// * `resources` - Access to game resources (read-only for calculations)
    ///
    /// # Returns
    ///
    /// Effective delta (potentially modified)
    async fn calculate_delta(
        &self,
        _subject_id: &SubjectId,
        base_delta: f32,
        _category: Option<&str>,
        _resources: &ResourceContext,
    ) -> f32 {
        // Default: no modification
        base_delta
    }
}

/// Default hook that does nothing
#[derive(Default, Debug, Clone, Copy)]
pub struct DefaultReputationHook;

#[async_trait]
impl ReputationHook for DefaultReputationHook {
    // All methods use default implementations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_hook() {
        let hook = DefaultReputationHook;
        let mut resources = ResourceContext::new();
        let id = SubjectId::new("player", "kingdom");

        // Should not panic
        hook.on_reputation_changed(&id, 0.0, 10.0, 10.0, None, &mut resources)
            .await;

        let threshold = ReputationThreshold::new("Friendly", 0.0, 50.0);
        hook.on_threshold_crossed(&id, None, &threshold, &mut resources)
            .await;

        let result = hook.validate_change(&id, 10.0, None, &resources).await;
        assert!(result.is_ok());

        let delta = hook.calculate_delta(&id, 10.0, None, &resources).await;
        assert_eq!(delta, 10.0);
    }
}
