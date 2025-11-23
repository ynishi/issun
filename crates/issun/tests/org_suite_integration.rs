//! Integration tests for OrganizationSuitePlugin
//!
//! Tests the complete transition workflow including service, converters,
//! conditions, and state management.

use issun::plugin::org_suite::*;

#[test]
fn test_single_transition_holacracy_to_hierarchy() {
    // Setup: Create service with scaling condition and converter
    let mut registry = TransitionRegistry::new();
    registry.register_converter(Box::new(HolacracyToHierarchyConverter));
    registry.register_condition(Box::new(ScalingCondition::new(
        50,
        OrgArchetype::Holacracy,
        OrgArchetype::Hierarchy,
    )));

    let service = TransitionService::new(registry);

    // Setup: Create state with a holacracy faction
    let mut state = OrgSuiteState::new();
    state.register_faction("startup", OrgArchetype::Holacracy);

    // Test: Faction under threshold - no transition
    let context = ConditionContext {
        member_count: 30,
        ..Default::default()
    };

    let result = service.evaluate_transition("startup", OrgArchetype::Holacracy, &context);
    assert!(result.is_none(), "Should not transition below threshold");

    // Test: Faction exceeds threshold - should transition
    let context = ConditionContext {
        member_count: 60,
        ..Default::default()
    };

    let result = service.evaluate_transition("startup", OrgArchetype::Holacracy, &context);
    assert!(result.is_some(), "Should transition above threshold");

    let (target, trigger) = result.unwrap();
    assert_eq!(target, OrgArchetype::Hierarchy);

    match trigger {
        TransitionTrigger::Scaling {
            from,
            to,
            member_count,
        } => {
            assert_eq!(from, OrgArchetype::Holacracy);
            assert_eq!(to, OrgArchetype::Hierarchy);
            assert_eq!(member_count, 60);
        }
        _ => panic!("Expected Scaling trigger"),
    }

    // Apply transition to state
    state
        .record_transition("startup", OrgArchetype::Holacracy, target, trigger)
        .expect("Transition recording should succeed");

    // Verify state changed
    assert_eq!(
        state.get_archetype("startup"),
        Some(OrgArchetype::Hierarchy)
    );

    // Verify history
    let history = state.get_history();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].from, OrgArchetype::Holacracy);
    assert_eq!(history[0].to, OrgArchetype::Hierarchy);
}

#[test]
fn test_full_transition_cycle() {
    // Setup: Register all converters and conditions
    let mut registry = TransitionRegistry::new();

    // Converters
    registry.register_converter(Box::new(HolacracyToHierarchyConverter));
    registry.register_converter(Box::new(HierarchyToSocialConverter));
    registry.register_converter(Box::new(SocialToCultureConverter));

    // Conditions
    registry.register_condition(Box::new(ScalingCondition::new(
        50,
        OrgArchetype::Holacracy,
        OrgArchetype::Hierarchy,
    )));
    registry.register_condition(Box::new(DecayCondition::new(
        0.8,
        OrgArchetype::Hierarchy,
        OrgArchetype::Social,
    )));
    registry.register_condition(Box::new(RadicalizationCondition::new(
        0.9,
        OrgArchetype::Social,
        OrgArchetype::Culture,
    )));

    let service = TransitionService::new(registry);
    let mut state = OrgSuiteState::new();
    state.register_faction("rebels", OrgArchetype::Holacracy);

    // Step 1: Holacracy → Hierarchy (Scaling)
    let context = ConditionContext {
        member_count: 60,
        ..Default::default()
    };

    let (target, trigger) = service
        .evaluate_transition("rebels", OrgArchetype::Holacracy, &context)
        .expect("Should transition");

    assert_eq!(target, OrgArchetype::Hierarchy);
    state
        .record_transition("rebels", OrgArchetype::Holacracy, target, trigger)
        .unwrap();

    // Step 2: Hierarchy → Social (Decay)
    let context = ConditionContext {
        corruption_level: 0.9,
        ..Default::default()
    };

    let (target, trigger) = service
        .evaluate_transition("rebels", OrgArchetype::Hierarchy, &context)
        .expect("Should transition");

    assert_eq!(target, OrgArchetype::Social);
    state
        .record_transition("rebels", OrgArchetype::Hierarchy, target, trigger)
        .unwrap();

    // Step 3: Social → Culture (Radicalization)
    let context = ConditionContext {
        fervor_level: 0.95,
        ..Default::default()
    };

    let (target, trigger) = service
        .evaluate_transition("rebels", OrgArchetype::Social, &context)
        .expect("Should transition");

    assert_eq!(target, OrgArchetype::Culture);
    state
        .record_transition("rebels", OrgArchetype::Social, target, trigger)
        .unwrap();

    // Verify final state
    assert_eq!(state.get_archetype("rebels"), Some(OrgArchetype::Culture));

    // Verify history
    let history = state.get_history();
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].from, OrgArchetype::Holacracy);
    assert_eq!(history[0].to, OrgArchetype::Hierarchy);
    assert_eq!(history[1].from, OrgArchetype::Hierarchy);
    assert_eq!(history[1].to, OrgArchetype::Social);
    assert_eq!(history[2].from, OrgArchetype::Social);
    assert_eq!(history[2].to, OrgArchetype::Culture);
}

#[test]
fn test_evaluate_all_transitions_multiple_factions() {
    // Setup
    let mut registry = TransitionRegistry::new();
    registry.register_converter(Box::new(HolacracyToHierarchyConverter));
    registry.register_condition(Box::new(ScalingCondition::new(
        50,
        OrgArchetype::Holacracy,
        OrgArchetype::Hierarchy,
    )));

    let service = TransitionService::new(registry);

    let mut state = OrgSuiteState::new();
    state.register_faction("faction_a", OrgArchetype::Holacracy);
    state.register_faction("faction_b", OrgArchetype::Holacracy);
    state.register_faction("faction_c", OrgArchetype::Hierarchy);

    // Context: Only faction_a should trigger
    let context_fn = |faction_id: &str| match faction_id {
        "faction_a" => ConditionContext {
            member_count: 60, // Above threshold
            ..Default::default()
        },
        "faction_b" => ConditionContext {
            member_count: 30, // Below threshold
            ..Default::default()
        },
        _ => ConditionContext::default(),
    };

    let transitions = service.evaluate_all_transitions(&state, context_fn);

    // Only faction_a should transition
    assert_eq!(transitions.len(), 1);
    assert_eq!(transitions[0].0, "faction_a");
    assert_eq!(transitions[0].1, OrgArchetype::Holacracy);
    assert_eq!(transitions[0].2, OrgArchetype::Hierarchy);
}

#[test]
fn test_no_converter_registered() {
    // Setup: Condition without matching converter
    let mut registry = TransitionRegistry::new();
    registry.register_condition(Box::new(ScalingCondition::new(
        50,
        OrgArchetype::Holacracy,
        OrgArchetype::Hierarchy,
    )));
    // No converter registered

    let service = TransitionService::new(registry);

    let context = ConditionContext {
        member_count: 60,
        ..Default::default()
    };

    // Should return None because converter is missing
    let result = service.evaluate_transition("test", OrgArchetype::Holacracy, &context);
    assert!(result.is_none(), "Should not transition without converter");
}

#[test]
fn test_multiple_conditions_priority() {
    // Setup: Multiple conditions, first match wins
    let mut registry = TransitionRegistry::new();

    registry.register_converter(Box::new(HolacracyToHierarchyConverter));

    // Register two conditions for the same archetype
    registry.register_condition(Box::new(ScalingCondition::new(
        50,
        OrgArchetype::Holacracy,
        OrgArchetype::Hierarchy,
    )));
    registry.register_condition(Box::new(ScalingCondition::new(
        100, // Higher threshold
        OrgArchetype::Holacracy,
        OrgArchetype::Hierarchy,
    )));

    let service = TransitionService::new(registry);

    // Context: Meets first condition (50) but not second (100)
    let context = ConditionContext {
        member_count: 60,
        ..Default::default()
    };

    let result = service.evaluate_transition("test", OrgArchetype::Holacracy, &context);
    assert!(result.is_some(), "First condition should trigger");
}

#[test]
fn test_config_and_state_integration() {
    // Test that config and state work together
    let config = OrgSuiteConfig::new()
        .with_auto_transition(true)
        .with_check_interval(5)
        .with_logging(true);

    let mut state = OrgSuiteState::new();
    state.register_faction("test", OrgArchetype::Holacracy);

    // Simulate ticks
    for _ in 0..10 {
        state.tick();
    }

    assert_eq!(state.current_tick(), 10);

    // Config should be accessible
    assert_eq!(config.enable_auto_transition, true);
    assert_eq!(config.transition_check_interval, 5);
}
