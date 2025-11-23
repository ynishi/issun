//! Hook system for customizing organizational transitions
//!
//! Provides extension points for games to add custom logic before, during,
//! and after organizational transitions.

use super::events::{TransitionFailedEvent, TransitionOccurredEvent, TransitionRequested};
use super::types::{OrgArchetype, TransitionTrigger};
use async_trait::async_trait;

/// Hook trait for customizing organizational transition behavior
///
/// Games implement this trait to:
/// - Add custom transition conditions
/// - Validate transitions before they occur
/// - Respond to transition events (logging, UI updates, narrative)
/// - Handle transition failures
#[async_trait]
pub trait OrgSuiteHook: Clone + Send + Sync + 'static {
    /// Evaluate custom transition conditions
    ///
    /// Called during transition evaluation to allow game-specific triggers.
    /// Returns Some((target_archetype, trigger)) if a custom condition is met.
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn evaluate_custom_transition(
    ///     &self,
    ///     faction_id: &str,
    ///     current: OrgArchetype,
    /// ) -> Option<(OrgArchetype, TransitionTrigger)> {
    ///     if leader_died(faction_id) && current == OrgArchetype::Hierarchy {
    ///         Some((OrgArchetype::Social, TransitionTrigger::Custom {
    ///             from: OrgArchetype::Hierarchy,
    ///             to: OrgArchetype::Social,
    ///             reason: "Power vacuum after leader death".to_string(),
    ///         }))
    ///     } else {
    ///         None
    ///     }
    /// }
    /// ```
    async fn evaluate_custom_transition(
        &self,
        _faction_id: &str,
        _current: OrgArchetype,
    ) -> Option<(OrgArchetype, TransitionTrigger)> {
        None
    }

    /// Pre-transition validation
    ///
    /// Called before a transition occurs. Return Err to cancel the transition.
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn on_before_transition(&self, event: &TransitionRequested) -> Result<(), String> {
    ///     if is_at_war(&event.faction_id) {
    ///         Err("Cannot change organization during war".to_string())
    ///     } else {
    ///         Ok(())
    ///     }
    /// }
    /// ```
    async fn on_before_transition(&self, _event: &TransitionRequested) -> Result<(), String> {
        Ok(())
    }

    /// Post-transition handling
    ///
    /// Called after a transition successfully completes.
    /// Use this for logging, UI updates, narrative events, etc.
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn on_transition_occurred(&self, event: &TransitionOccurredEvent) {
    ///     log_to_database(event).await;
    ///     notify_player(&format!(
    ///         "Organization {} transformed from {:?} to {:?}",
    ///         event.faction_id, event.from, event.to
    ///     ));
    /// }
    /// ```
    async fn on_transition_occurred(&self, _event: &TransitionOccurredEvent) {
        // Default: No-op
        // Games can override this to add logging, UI updates, etc.
    }

    /// Transition failure handling
    ///
    /// Called when a transition fails (validation error, missing converter, etc.)
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn on_transition_failed(&self, event: &TransitionFailedEvent) {
    ///     log_error(&format!(
    ///         "Transition failed for {}: {} -> {:?} ({})",
    ///         event.faction_id, event.from, event.to, event.error
    ///     ));
    /// }
    /// ```
    async fn on_transition_failed(&self, _event: &TransitionFailedEvent) {
        // Default: No-op
        // Games can override this to add error logging
    }
}

/// Default hook implementation with logging only
///
/// Provides basic logging for transitions without additional game logic.
/// Games can use this as-is or implement their own hook.
#[derive(Debug, Clone, Copy)]
pub struct DefaultOrgSuiteHook;

#[async_trait]
impl OrgSuiteHook for DefaultOrgSuiteHook {
    // Uses all default implementations with tracing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_hook_evaluate_custom_transition() {
        let hook = DefaultOrgSuiteHook;
        let result = hook
            .evaluate_custom_transition("test", OrgArchetype::Holacracy)
            .await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_default_hook_on_before_transition() {
        let hook = DefaultOrgSuiteHook;
        let event = TransitionRequested {
            faction_id: "test".to_string(),
            from: OrgArchetype::Holacracy,
            to: OrgArchetype::Hierarchy,
            reason: "test".to_string(),
        };

        let result = hook.on_before_transition(&event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_default_hook_on_transition_occurred() {
        let hook = DefaultOrgSuiteHook;
        let event = TransitionOccurredEvent {
            faction_id: "test".to_string(),
            from: OrgArchetype::Holacracy,
            to: OrgArchetype::Hierarchy,
            trigger: TransitionTrigger::Scaling {
                from: OrgArchetype::Holacracy,
                to: OrgArchetype::Hierarchy,
                member_count: 50,
            },
            timestamp: 100,
        };

        // Should not panic
        hook.on_transition_occurred(&event).await;
    }

    #[tokio::test]
    async fn test_default_hook_on_transition_failed() {
        let hook = DefaultOrgSuiteHook;
        let event = TransitionFailedEvent {
            faction_id: "test".to_string(),
            from: OrgArchetype::Holacracy,
            to: OrgArchetype::Hierarchy,
            error: "test error".to_string(),
        };

        // Should not panic
        hook.on_transition_failed(&event).await;
    }

    // Custom hook for testing
    #[derive(Clone)]
    struct TestHook {
        cancel_transitions: bool,
    }

    #[async_trait]
    impl OrgSuiteHook for TestHook {
        async fn on_before_transition(&self, _event: &TransitionRequested) -> Result<(), String> {
            if self.cancel_transitions {
                Err("Transition cancelled by test hook".to_string())
            } else {
                Ok(())
            }
        }

        async fn evaluate_custom_transition(
            &self,
            faction_id: &str,
            current: OrgArchetype,
        ) -> Option<(OrgArchetype, TransitionTrigger)> {
            if faction_id == "special" && current == OrgArchetype::Holacracy {
                Some((
                    OrgArchetype::Culture,
                    TransitionTrigger::Custom {
                        from: OrgArchetype::Holacracy,
                        to: OrgArchetype::Culture,
                        reason: "Special faction rule".to_string(),
                    },
                ))
            } else {
                None
            }
        }
    }

    #[tokio::test]
    async fn test_custom_hook_cancel_transition() {
        let hook = TestHook {
            cancel_transitions: true,
        };
        let event = TransitionRequested {
            faction_id: "test".to_string(),
            from: OrgArchetype::Holacracy,
            to: OrgArchetype::Hierarchy,
            reason: "test".to_string(),
        };

        let result = hook.on_before_transition(&event).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cancelled"));
    }

    #[tokio::test]
    async fn test_custom_hook_allow_transition() {
        let hook = TestHook {
            cancel_transitions: false,
        };
        let event = TransitionRequested {
            faction_id: "test".to_string(),
            from: OrgArchetype::Holacracy,
            to: OrgArchetype::Hierarchy,
            reason: "test".to_string(),
        };

        let result = hook.on_before_transition(&event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_custom_hook_custom_transition() {
        let hook = TestHook {
            cancel_transitions: false,
        };

        // Special faction should trigger custom transition
        let result = hook
            .evaluate_custom_transition("special", OrgArchetype::Holacracy)
            .await;
        assert!(result.is_some());

        let (target, trigger) = result.unwrap();
        assert_eq!(target, OrgArchetype::Culture);
        match trigger {
            TransitionTrigger::Custom { reason, .. } => {
                assert!(reason.contains("Special faction"));
            }
            _ => panic!("Expected Custom trigger"),
        }

        // Normal faction should not trigger
        let result = hook
            .evaluate_custom_transition("normal", OrgArchetype::Holacracy)
            .await;
        assert!(result.is_none());
    }
}
