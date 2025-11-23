//! Transition condition evaluation

use crate::plugin::org_suite::types::{OrgArchetype, TransitionTrigger};

/// Context for condition evaluation
///
/// Contains metrics and state information used to evaluate transition conditions.
/// Games can extend this via custom conditions that access additional state.
#[derive(Debug, Clone)]
pub struct ConditionContext {
    /// Number of members in the organization
    pub member_count: usize,

    /// Average loyalty of members (0.0-1.0)
    pub average_loyalty: f32,

    /// Average morale of members (0.0-1.0)
    pub average_morale: f32,

    /// Corruption level (0.0-1.0, higher is more corrupt)
    pub corruption_level: f32,

    /// Fervor/zealotry level (0.0-1.0, higher is more fervent)
    pub fervor_level: f32,
}

impl Default for ConditionContext {
    fn default() -> Self {
        Self {
            member_count: 0,
            average_loyalty: 0.5,
            average_morale: 0.5,
            corruption_level: 0.0,
            fervor_level: 0.0,
        }
    }
}

/// Trait for transition condition evaluation
///
/// Conditions are evaluated every tick to determine if a transition should occur.
/// Games implement this trait to define custom transition triggers.
pub trait TransitionCondition: Send + Sync {
    /// Evaluate whether a transition should occur
    ///
    /// # Arguments
    ///
    /// * `faction_id` - The faction being evaluated
    /// * `current` - Current organizational archetype
    /// * `context` - Metrics and state for evaluation
    ///
    /// # Returns
    ///
    /// Some(TransitionTrigger) if condition is met, None otherwise
    fn evaluate(
        &self,
        faction_id: &str,
        current: OrgArchetype,
        context: &ConditionContext,
    ) -> Option<TransitionTrigger>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_context_default() {
        let ctx = ConditionContext::default();
        assert_eq!(ctx.member_count, 0);
        assert_eq!(ctx.average_loyalty, 0.5);
        assert_eq!(ctx.corruption_level, 0.0);
    }

    #[test]
    fn test_condition_context_custom() {
        let ctx = ConditionContext {
            member_count: 100,
            average_loyalty: 0.8,
            average_morale: 0.9,
            corruption_level: 0.1,
            fervor_level: 0.2,
        };

        assert_eq!(ctx.member_count, 100);
        assert_eq!(ctx.fervor_level, 0.2);
    }

    // Mock condition for testing
    struct AlwaysTrueCondition;

    impl TransitionCondition for AlwaysTrueCondition {
        fn evaluate(
            &self,
            _faction_id: &str,
            current: OrgArchetype,
            _context: &ConditionContext,
        ) -> Option<TransitionTrigger> {
            if current == OrgArchetype::Holacracy {
                Some(TransitionTrigger::Custom {
                    from: OrgArchetype::Holacracy,
                    to: OrgArchetype::Hierarchy,
                    reason: "Always true test".to_string(),
                })
            } else {
                None
            }
        }
    }

    #[test]
    fn test_mock_condition() {
        let condition = AlwaysTrueCondition;
        let ctx = ConditionContext::default();

        let result = condition.evaluate("test", OrgArchetype::Holacracy, &ctx);
        assert!(result.is_some());

        let result = condition.evaluate("test", OrgArchetype::Culture, &ctx);
        assert!(result.is_none());
    }
}
