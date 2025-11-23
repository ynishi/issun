//! Business logic for organizational transitions

use super::state::OrgSuiteState;
use super::transition::{ConditionContext, TransitionRegistry};
use super::types::{FactionId, OrgArchetype};

/// Service for evaluating transition conditions and orchestrating conversions
///
/// This is a pure logic service - it evaluates conditions and returns recommended
/// transitions without mutating state or emitting events. The System layer handles
/// those side effects.
pub struct TransitionService {
    registry: TransitionRegistry,
}

impl TransitionService {
    /// Create a new service with a transition registry
    pub fn new(registry: TransitionRegistry) -> Self {
        Self { registry }
    }

    /// Evaluate transition conditions for a single faction
    ///
    /// Returns the target archetype and trigger if any condition is met.
    ///
    /// # Arguments
    ///
    /// * `faction_id` - ID of the faction to evaluate
    /// * `current_archetype` - Current organizational archetype
    /// * `context` - Metrics for condition evaluation
    ///
    /// # Returns
    ///
    /// Some((target_archetype, trigger)) if transition should occur, None otherwise
    pub fn evaluate_transition(
        &self,
        faction_id: &str,
        current_archetype: OrgArchetype,
        context: &ConditionContext,
    ) -> Option<(OrgArchetype, super::types::TransitionTrigger)> {
        // Evaluate all registered conditions
        for condition in self.registry.conditions() {
            if let Some(trigger) = condition.evaluate(faction_id, current_archetype, context) {
                // Verify converter exists for this transition
                let target = match &trigger {
                    super::types::TransitionTrigger::Scaling { to, .. } => *to,
                    super::types::TransitionTrigger::Decay { to, .. } => *to,
                    super::types::TransitionTrigger::Radicalization { to, .. } => *to,
                    super::types::TransitionTrigger::Custom { to, .. } => *to,
                };

                if self.registry.is_transition_valid(current_archetype, target) {
                    return Some((target, trigger));
                }
            }
        }

        None
    }

    /// Evaluate transitions for all registered factions
    ///
    /// Returns a list of (faction_id, from, to, trigger) tuples for factions
    /// that should transition.
    ///
    /// # Arguments
    ///
    /// * `state` - Current organizational state
    /// * `context_fn` - Function to get ConditionContext for each faction
    ///
    /// # Returns
    ///
    /// Vector of transitions that should occur
    pub fn evaluate_all_transitions<F>(
        &self,
        state: &OrgSuiteState,
        mut context_fn: F,
    ) -> Vec<(
        FactionId,
        OrgArchetype,
        OrgArchetype,
        super::types::TransitionTrigger,
    )>
    where
        F: FnMut(&str) -> ConditionContext,
    {
        let mut transitions = Vec::new();

        for (faction_id, current_archetype) in state.factions() {
            let context = context_fn(faction_id);

            if let Some((target, trigger)) =
                self.evaluate_transition(faction_id, *current_archetype, &context)
            {
                transitions.push((faction_id.clone(), *current_archetype, target, trigger));
            }
        }

        transitions
    }

    /// Get reference to the transition registry
    pub fn registry(&self) -> &TransitionRegistry {
        &self.registry
    }

    /// Get mutable reference to the transition registry
    pub fn registry_mut(&mut self) -> &mut TransitionRegistry {
        &mut self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::org_suite::transition::TransitionCondition;
    use crate::plugin::org_suite::types::TransitionTrigger;

    // Mock condition for testing
    struct TestScalingCondition {
        threshold: usize,
    }

    impl TransitionCondition for TestScalingCondition {
        fn evaluate(
            &self,
            _faction_id: &str,
            current: OrgArchetype,
            context: &ConditionContext,
        ) -> Option<TransitionTrigger> {
            if current == OrgArchetype::Holacracy && context.member_count >= self.threshold {
                Some(TransitionTrigger::Scaling {
                    from: OrgArchetype::Holacracy,
                    to: OrgArchetype::Hierarchy,
                    member_count: context.member_count,
                })
            } else {
                None
            }
        }
    }

    // Mock converter for testing
    struct TestConverter;

    impl crate::plugin::org_suite::transition::OrgConverter for TestConverter {
        fn source_archetype(&self) -> OrgArchetype {
            OrgArchetype::Holacracy
        }

        fn target_archetype(&self) -> OrgArchetype {
            OrgArchetype::Hierarchy
        }

        fn convert(
            &self,
            _source: &serde_json::Value,
        ) -> Result<serde_json::Value, super::super::types::OrgSuiteError> {
            Ok(serde_json::json!({"converted": true}))
        }
    }

    #[test]
    fn test_evaluate_transition_condition_met() {
        let mut registry = TransitionRegistry::new();
        registry.register_condition(Box::new(TestScalingCondition { threshold: 50 }));
        registry.register_converter(Box::new(TestConverter));

        let service = TransitionService::new(registry);

        let context = ConditionContext {
            member_count: 60,
            ..Default::default()
        };

        let result = service.evaluate_transition("test", OrgArchetype::Holacracy, &context);

        assert!(result.is_some());
        let (target, trigger) = result.unwrap();
        assert_eq!(target, OrgArchetype::Hierarchy);

        match trigger {
            TransitionTrigger::Scaling { member_count, .. } => {
                assert_eq!(member_count, 60);
            }
            _ => panic!("Expected Scaling trigger"),
        }
    }

    #[test]
    fn test_evaluate_transition_condition_not_met() {
        let mut registry = TransitionRegistry::new();
        registry.register_condition(Box::new(TestScalingCondition { threshold: 50 }));
        registry.register_converter(Box::new(TestConverter));

        let service = TransitionService::new(registry);

        let context = ConditionContext {
            member_count: 30, // Below threshold
            ..Default::default()
        };

        let result = service.evaluate_transition("test", OrgArchetype::Holacracy, &context);

        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_transition_no_converter() {
        let mut registry = TransitionRegistry::new();
        registry.register_condition(Box::new(TestScalingCondition { threshold: 50 }));
        // No converter registered

        let service = TransitionService::new(registry);

        let context = ConditionContext {
            member_count: 60,
            ..Default::default()
        };

        let result = service.evaluate_transition("test", OrgArchetype::Holacracy, &context);

        // Should return None because no converter is available
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_all_transitions() {
        let mut registry = TransitionRegistry::new();
        registry.register_condition(Box::new(TestScalingCondition { threshold: 50 }));
        registry.register_converter(Box::new(TestConverter));

        let service = TransitionService::new(registry);

        let mut state = OrgSuiteState::new();
        state.register_faction("faction_a", OrgArchetype::Holacracy);
        state.register_faction("faction_b", OrgArchetype::Holacracy);
        state.register_faction("faction_c", OrgArchetype::Hierarchy);

        // Context function: faction_a triggers, faction_b doesn't, faction_c wrong type
        let context_fn = |faction_id: &str| match faction_id {
            "faction_a" => ConditionContext {
                member_count: 60,
                ..Default::default()
            },
            "faction_b" => ConditionContext {
                member_count: 30,
                ..Default::default()
            },
            _ => ConditionContext::default(),
        };

        let transitions = service.evaluate_all_transitions(&state, context_fn);

        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].0, "faction_a");
        assert_eq!(transitions[0].1, OrgArchetype::Holacracy);
        assert_eq!(transitions[0].2, OrgArchetype::Hierarchy);
    }

    #[test]
    fn test_registry_access() {
        let registry = TransitionRegistry::new();
        let service = TransitionService::new(registry);

        assert_eq!(service.registry().condition_count(), 0);
    }

    #[test]
    fn test_registry_mut_access() {
        let registry = TransitionRegistry::new();
        let mut service = TransitionService::new(registry);

        service
            .registry_mut()
            .register_condition(Box::new(TestScalingCondition { threshold: 50 }));

        assert_eq!(service.registry().condition_count(), 1);
    }
}
