//! Orchestration system for organizational transitions
//!
//! Coordinates service logic, hooks, state management, and event emission.

use super::config::OrgSuiteConfig;
use super::events::{TransitionFailedEvent, TransitionOccurredEvent, TransitionRequested};
use super::hook::OrgSuiteHook;
use super::service::TransitionService;
use super::state::OrgSuiteState;
use super::transition::ConditionContext;
use super::types::OrgArchetype;

/// Orchestration system for organizational transitions
///
/// This system coordinates:
/// - Condition evaluation (via TransitionService)
/// - Custom hooks (game-specific logic)
/// - State mutations
/// - Event emission
///
/// # Type Parameters
///
/// * `H` - Hook implementation for customization
pub struct OrgSuiteSystem<H: OrgSuiteHook> {
    service: TransitionService,
    hook: H,
}

impl<H: OrgSuiteHook> OrgSuiteSystem<H> {
    /// Create a new system with service and hook
    pub fn new(service: TransitionService, hook: H) -> Self {
        Self { service, hook }
    }

    /// Update system - evaluate all factions for potential transitions
    ///
    /// This should be called every tick/turn (respecting check_interval config).
    ///
    /// # Arguments
    ///
    /// * `state` - Mutable organizational state
    /// * `config` - Configuration (read-only)
    /// * `context_fn` - Function to get ConditionContext for each faction
    ///
    /// # Returns
    ///
    /// Vector of transition events that occurred
    pub async fn update<F>(
        &mut self,
        state: &mut OrgSuiteState,
        config: &OrgSuiteConfig,
        context_fn: F,
    ) -> Vec<TransitionOccurredEvent>
    where
        F: FnMut(&str) -> ConditionContext,
    {
        let mut events = Vec::new();

        // Advance tick counter
        state.tick();

        // Check interval
        if !state
            .current_tick()
            .is_multiple_of(config.transition_check_interval as u64)
        {
            return events;
        }

        // Skip if auto-transition disabled
        if !config.enable_auto_transition {
            return events;
        }

        // Evaluate all factions
        let transitions = self.service.evaluate_all_transitions(state, context_fn);

        // Execute each transition
        for (faction_id, from, to, trigger) in transitions {
            // Check custom hook conditions first
            if let Some((custom_to, custom_trigger)) = self
                .hook
                .evaluate_custom_transition(&faction_id, from)
                .await
            {
                // Custom trigger overrides standard evaluation
                match self
                    .execute_transition(state, &faction_id, from, custom_to, custom_trigger)
                    .await
                {
                    Ok(event) => events.push(event),
                    Err(failed_event) => {
                        self.hook.on_transition_failed(&failed_event).await;
                    }
                }
            } else {
                // Standard transition
                match self
                    .execute_transition(state, &faction_id, from, to, trigger)
                    .await
                {
                    Ok(event) => events.push(event),
                    Err(failed_event) => {
                        self.hook.on_transition_failed(&failed_event).await;
                    }
                }
            }
        }

        events
    }

    /// Handle a manual transition request
    ///
    /// # Arguments
    ///
    /// * `request` - Transition request event
    /// * `state` - Mutable organizational state
    ///
    /// # Returns
    ///
    /// Ok(TransitionOccurredEvent) on success, Err(TransitionFailedEvent) on failure
    pub async fn handle_transition_request(
        &mut self,
        request: TransitionRequested,
        state: &mut OrgSuiteState,
    ) -> Result<TransitionOccurredEvent, TransitionFailedEvent> {
        // Pre-transition validation via hook
        if let Err(reason) = self.hook.on_before_transition(&request).await {
            let failed = TransitionFailedEvent {
                faction_id: request.faction_id.clone(),
                from: request.from,
                to: request.to,
                error: reason,
            };
            self.hook.on_transition_failed(&failed).await;
            return Err(failed);
        }

        // Execute transition
        let trigger = super::types::TransitionTrigger::Custom {
            from: request.from,
            to: request.to,
            reason: request.reason,
        };

        self.execute_transition(
            state,
            &request.faction_id,
            request.from,
            request.to,
            trigger,
        )
        .await
    }

    /// Execute a transition (internal)
    ///
    /// Performs state mutation and emits events.
    async fn execute_transition(
        &mut self,
        state: &mut OrgSuiteState,
        faction_id: &str,
        from: OrgArchetype,
        to: OrgArchetype,
        trigger: super::types::TransitionTrigger,
    ) -> Result<TransitionOccurredEvent, TransitionFailedEvent> {
        // Verify converter exists
        if !self.service.registry().is_transition_valid(from, to) {
            let failed = TransitionFailedEvent {
                faction_id: faction_id.to_string(),
                from,
                to,
                error: format!("No converter registered for {:?} -> {:?}", from, to),
            };
            return Err(failed);
        }

        // Record transition in state
        match state.record_transition(faction_id, from, to, trigger.clone()) {
            Ok(_) => {
                let event = TransitionOccurredEvent {
                    faction_id: faction_id.to_string(),
                    from,
                    to,
                    trigger,
                    timestamp: state.current_tick(),
                };

                // Notify hook
                self.hook.on_transition_occurred(&event).await;

                Ok(event)
            }
            Err(e) => {
                let failed = TransitionFailedEvent {
                    faction_id: faction_id.to_string(),
                    from,
                    to,
                    error: e.to_string(),
                };
                Err(failed)
            }
        }
    }

    /// Get reference to the service
    pub fn service(&self) -> &TransitionService {
        &self.service
    }

    /// Get reference to the hook
    pub fn hook(&self) -> &H {
        &self.hook
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::org_suite::{
        DefaultOrgSuiteHook, HolacracyToHierarchyConverter, ScalingCondition, TransitionRegistry,
    };

    fn create_test_system() -> OrgSuiteSystem<DefaultOrgSuiteHook> {
        let mut registry = TransitionRegistry::new();
        registry.register_converter(Box::new(HolacracyToHierarchyConverter));
        registry.register_condition(Box::new(ScalingCondition::new(
            50,
            OrgArchetype::Holacracy,
            OrgArchetype::Hierarchy,
        )));

        let service = TransitionService::new(registry);
        OrgSuiteSystem::new(service, DefaultOrgSuiteHook)
    }

    #[tokio::test]
    async fn test_system_update_no_transitions() {
        let mut system = create_test_system();
        let mut state = OrgSuiteState::new();
        state.register_faction("test", OrgArchetype::Holacracy);

        let config = OrgSuiteConfig::default();

        let context_fn = |_: &str| ConditionContext {
            member_count: 30, // Below threshold
            ..Default::default()
        };

        let events = system.update(&mut state, &config, context_fn).await;

        assert_eq!(events.len(), 0);
        assert_eq!(state.current_tick(), 1);
    }

    #[tokio::test]
    async fn test_system_update_with_transition() {
        let mut system = create_test_system();
        let mut state = OrgSuiteState::new();
        state.register_faction("startup", OrgArchetype::Holacracy);

        let config = OrgSuiteConfig::default();

        let context_fn = |_: &str| ConditionContext {
            member_count: 60, // Above threshold
            ..Default::default()
        };

        let events = system.update(&mut state, &config, context_fn).await;

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].faction_id, "startup");
        assert_eq!(events[0].from, OrgArchetype::Holacracy);
        assert_eq!(events[0].to, OrgArchetype::Hierarchy);

        // Verify state changed
        assert_eq!(
            state.get_archetype("startup"),
            Some(OrgArchetype::Hierarchy)
        );
    }

    #[tokio::test]
    async fn test_system_respects_check_interval() {
        let mut system = create_test_system();
        let mut state = OrgSuiteState::new();
        state.register_faction("test", OrgArchetype::Holacracy);

        let config = OrgSuiteConfig::new().with_check_interval(5);

        let context_fn = |_: &str| ConditionContext {
            member_count: 60,
            ..Default::default()
        };

        // Tick 1: Should skip (1 % 5 != 0)
        let events = system.update(&mut state, &config, context_fn).await;
        assert_eq!(events.len(), 0);

        // Ticks 2-4: Should skip
        for _ in 0..3 {
            let events = system
                .update(&mut state, &config, |_| ConditionContext::default())
                .await;
            assert_eq!(events.len(), 0);
        }

        // Tick 5: Should execute (5 % 5 == 0)
        let events = system
            .update(&mut state, &config, |_| ConditionContext {
                member_count: 60,
                ..Default::default()
            })
            .await;
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_system_auto_transition_disabled() {
        let mut system = create_test_system();
        let mut state = OrgSuiteState::new();
        state.register_faction("test", OrgArchetype::Holacracy);

        let config = OrgSuiteConfig::new().with_auto_transition(false);

        let context_fn = |_: &str| ConditionContext {
            member_count: 60,
            ..Default::default()
        };

        let events = system.update(&mut state, &config, context_fn).await;

        assert_eq!(events.len(), 0, "Should not auto-transition when disabled");
    }

    #[tokio::test]
    async fn test_handle_transition_request_success() {
        let mut system = create_test_system();
        let mut state = OrgSuiteState::new();
        state.register_faction("rebels", OrgArchetype::Holacracy);

        let request = TransitionRequested {
            faction_id: "rebels".to_string(),
            from: OrgArchetype::Holacracy,
            to: OrgArchetype::Hierarchy,
            reason: "Manual transition test".to_string(),
        };

        let result = system.handle_transition_request(request, &mut state).await;

        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.faction_id, "rebels");
        assert_eq!(event.from, OrgArchetype::Holacracy);
        assert_eq!(event.to, OrgArchetype::Hierarchy);
    }

    #[tokio::test]
    async fn test_handle_transition_request_no_converter() {
        let mut system = create_test_system();
        let mut state = OrgSuiteState::new();
        state.register_faction("test", OrgArchetype::Social);

        let request = TransitionRequested {
            faction_id: "test".to_string(),
            from: OrgArchetype::Social,
            to: OrgArchetype::Culture,
            reason: "No converter registered".to_string(),
        };

        let result = system.handle_transition_request(request, &mut state).await;

        assert!(result.is_err());
        let failed = result.unwrap_err();
        assert!(failed.error.contains("No converter"));
    }

    #[tokio::test]
    async fn test_handle_transition_request_faction_not_found() {
        let mut system = create_test_system();
        let mut state = OrgSuiteState::new();
        // Don't register faction

        let request = TransitionRequested {
            faction_id: "unknown".to_string(),
            from: OrgArchetype::Holacracy,
            to: OrgArchetype::Hierarchy,
            reason: "Test".to_string(),
        };

        let result = system.handle_transition_request(request, &mut state).await;

        assert!(result.is_err());
        let failed = result.unwrap_err();
        assert!(failed.error.contains("not found"));
    }
}
