//! The core OrganizationMechanic implementation.

use std::marker::PhantomData;

use crate::mechanics::{EventEmitter, Mechanic, Transactional};

use super::policies::OrganizationPolicy;
use super::strategies::SimpleOrganizationPolicy;
use super::types::{
    EfficiencyChangeReason, FitQuality, MemberId, OrganizationConfig, OrganizationEvent,
    OrganizationInput, OrganizationState,
};

/// The core organization mechanic that models organizational dynamics.
///
/// # Type Parameters
///
/// - `P`: Organization policy (determines how dynamics are calculated)
///
/// # Overview
///
/// The organization mechanic calculates:
/// - **Decision Speed**: How fast the organization can make decisions
/// - **Consensus Requirement**: What percentage of agreement is needed
/// - **Authority Distribution**: How power is distributed among members
/// - **Efficiency**: Overall organizational effectiveness
/// - **Loyalty Modifiers**: How well members fit the organization type
/// - **Cohesion**: How unified the organization is
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::organization::prelude::*;
/// use issun_core::mechanics::{Mechanic, EventEmitter};
///
/// // Use default policy (SimpleOrganizationPolicy)
/// type SimpleOrg = OrganizationMechanic;
///
/// // Create configuration
/// let config = OrganizationConfig::default();
/// let mut state = OrganizationState::default();
///
/// // Prepare input for a Cult organization
/// let input = OrganizationInput {
///     org_type: OrganizationType::Cult,
///     member_count: 50,
///     decision_importance: 0.8,
///     urgency: 0.9,
///     leader_charisma: 0.95,
///     member_archetypes: vec![
///         (MemberId("leader".into()), MemberArchetype::Devotee),
///         (MemberId("follower1".into()), MemberArchetype::Devotee),
///     ],
///     current_tick: 100,
/// };
///
/// // Event collector
/// # struct VecEmitter(Vec<OrganizationEvent>);
/// # impl EventEmitter<OrganizationEvent> for VecEmitter {
/// #     fn emit(&mut self, event: OrganizationEvent) { self.0.push(event); }
/// # }
/// let mut emitter = VecEmitter(vec![]);
///
/// // Execute one step
/// SimpleOrg::step(&config, &mut state, input, &mut emitter);
///
/// // Cult = very fast decisions, no consensus needed
/// assert!(state.decision_speed > 1.5);
/// assert_eq!(state.consensus_requirement, 0.0);
/// ```
pub struct OrganizationMechanic<P: OrganizationPolicy = SimpleOrganizationPolicy> {
    _marker: PhantomData<P>,
}

impl<P> Mechanic for OrganizationMechanic<P>
where
    P: OrganizationPolicy,
{
    type Config = OrganizationConfig;
    type State = OrganizationState;
    type Input = OrganizationInput;
    type Event = OrganizationEvent;

    // Organization mechanics affect multiple members - use transactional
    type Execution = Transactional;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        // 1. Calculate decision speed
        let new_decision_speed = P::calculate_decision_speed(config, &input);

        // 2. Calculate consensus requirement
        let new_consensus = P::calculate_consensus_requirement(config, &input);

        // 3. Emit decision dynamics event
        emitter.emit(OrganizationEvent::DecisionDynamicsCalculated {
            decision_speed: new_decision_speed,
            consensus_requirement: new_consensus,
        });

        // 4. Calculate authority distribution
        let new_authority = P::determine_authority_distribution(config, &input);

        // Find top authority holder and concentration
        let (top_authority, concentration_index) =
            calculate_authority_concentration(&new_authority);

        emitter.emit(OrganizationEvent::AuthorityRebalanced {
            top_authority,
            concentration_index,
        });

        // 5. Calculate loyalty modifiers for each member
        let mut new_loyalty_modifiers = std::collections::HashMap::new();
        for (member_id, archetype) in &input.member_archetypes {
            let modifier = P::calculate_loyalty_modifier(&input.org_type, archetype);
            let fit_quality = FitQuality::from_modifier(modifier);

            emitter.emit(OrganizationEvent::LoyaltyModified {
                member_id: member_id.clone(),
                modifier,
                fit_quality,
            });

            new_loyalty_modifiers.insert(member_id.clone(), modifier);
        }

        // 6. Calculate cohesion
        let old_cohesion = state.cohesion;
        let new_cohesion = P::calculate_cohesion(&new_loyalty_modifiers);

        if (new_cohesion - old_cohesion).abs() > 0.05 {
            emitter.emit(OrganizationEvent::CohesionChanged {
                old_cohesion,
                new_cohesion,
            });
        }

        // 7. Calculate efficiency
        // First update state with new loyalty modifiers for efficiency calculation
        state.loyalty_modifiers = new_loyalty_modifiers;
        state.cohesion = new_cohesion;

        let old_efficiency = state.efficiency;
        let new_efficiency = P::calculate_efficiency(config, state, &input);

        if (new_efficiency - old_efficiency).abs() > 0.05 {
            let reason = determine_efficiency_change_reason(old_efficiency, new_efficiency, &input);
            emitter.emit(OrganizationEvent::EfficiencyChanged {
                old_efficiency,
                new_efficiency,
                reason,
            });
        }

        // 8. Update state
        state.decision_speed = new_decision_speed;
        state.consensus_requirement = new_consensus;
        state.authority_distribution = new_authority;
        state.efficiency = new_efficiency;
        state.last_update = input.current_tick;
    }
}

/// Calculate authority concentration from distribution.
///
/// Returns (top authority holder, concentration index).
/// Concentration index is 0.0 for perfectly flat distribution, 1.0 for single holder.
fn calculate_authority_concentration(
    distribution: &std::collections::HashMap<MemberId, f32>,
) -> (Option<MemberId>, f32) {
    if distribution.is_empty() {
        return (None, 0.0);
    }

    // Find member with highest authority
    let top = distribution
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(id, _)| id.clone());

    // Calculate Gini coefficient as concentration index
    let n = distribution.len() as f32;
    if n <= 1.0 {
        return (top, 1.0);
    }

    let mut weights: Vec<f32> = distribution.values().copied().collect();
    weights.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let sum: f32 = weights.iter().sum();
    if sum == 0.0 {
        return (top, 0.0);
    }

    // Gini calculation
    let mut gini_sum = 0.0;
    for (i, &w) in weights.iter().enumerate() {
        gini_sum += (2.0 * (i as f32 + 1.0) - n - 1.0) * w;
    }

    let gini = gini_sum / (n * sum);
    let concentration = gini.clamp(0.0, 1.0);

    (top, concentration)
}

/// Determine the reason for efficiency change.
fn determine_efficiency_change_reason(
    _old: f32,
    _new: f32,
    input: &OrganizationInput,
) -> EfficiencyChangeReason {
    // Simple heuristic based on input
    if input.urgency > 0.8 {
        EfficiencyChangeReason::UrgencyStress
    } else if input.member_count < 3 || input.member_count > 200 {
        EfficiencyChangeReason::MemberCountChange
    } else {
        EfficiencyChangeReason::ArchetypeDistributionChange
    }
}

/// Type alias for simple organization mechanic using default policy.
pub type SimpleOrganizationMechanic = OrganizationMechanic<SimpleOrganizationPolicy>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::organization::types::{MemberArchetype, OrganizationType};

    struct VecEmitter {
        events: Vec<OrganizationEvent>,
    }

    impl EventEmitter<OrganizationEvent> for VecEmitter {
        fn emit(&mut self, event: OrganizationEvent) {
            self.events.push(event);
        }
    }

    fn create_test_input(org_type: OrganizationType, member_count: usize) -> OrganizationInput {
        let members: Vec<(MemberId, MemberArchetype)> = (0..member_count)
            .map(|i| (MemberId(format!("m{}", i)), MemberArchetype::Pragmatic))
            .collect();

        OrganizationInput {
            org_type,
            member_count,
            decision_importance: 0.5,
            urgency: 0.5,
            leader_charisma: 0.7,
            member_archetypes: members,
            current_tick: 100,
        }
    }

    #[test]
    fn test_mechanic_step_updates_state() {
        let config = OrganizationConfig::default();
        let mut state = OrganizationState::default();
        let input = create_test_input(OrganizationType::Cult, 10);

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleOrganizationMechanic::step(&config, &mut state, input, &mut emitter);

        // Cult should have fast decisions
        assert!(state.decision_speed > 1.5);
        // Cult should have no consensus requirement
        assert_eq!(state.consensus_requirement, 0.0);
        // State should be updated
        assert_eq!(state.last_update, 100);
    }

    #[test]
    fn test_mechanic_emits_decision_dynamics_event() {
        let config = OrganizationConfig::default();
        let mut state = OrganizationState::default();
        let input = create_test_input(OrganizationType::Democracy, 20);

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleOrganizationMechanic::step(&config, &mut state, input, &mut emitter);

        // Should emit DecisionDynamicsCalculated
        let has_dynamics_event = emitter
            .events
            .iter()
            .any(|e| matches!(e, OrganizationEvent::DecisionDynamicsCalculated { .. }));
        assert!(has_dynamics_event);
    }

    #[test]
    fn test_mechanic_emits_authority_rebalanced_event() {
        let config = OrganizationConfig::default();
        let mut state = OrganizationState::default();
        let input = create_test_input(OrganizationType::Hierarchy, 5);

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleOrganizationMechanic::step(&config, &mut state, input, &mut emitter);

        // Should emit AuthorityRebalanced
        let has_authority_event = emitter
            .events
            .iter()
            .any(|e| matches!(e, OrganizationEvent::AuthorityRebalanced { .. }));
        assert!(has_authority_event);
    }

    #[test]
    fn test_mechanic_emits_loyalty_events() {
        let config = OrganizationConfig::default();
        let mut state = OrganizationState::default();

        let input = OrganizationInput {
            org_type: OrganizationType::Cult,
            member_count: 3,
            decision_importance: 0.5,
            urgency: 0.5,
            leader_charisma: 0.9,
            member_archetypes: vec![
                (MemberId("leader".into()), MemberArchetype::Devotee),
                (MemberId("m1".into()), MemberArchetype::Autonomous), // Poor fit
                (MemberId("m2".into()), MemberArchetype::Pragmatic),
            ],
            current_tick: 100,
        };

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleOrganizationMechanic::step(&config, &mut state, input, &mut emitter);

        // Should emit LoyaltyModified for each member
        let loyalty_events: Vec<_> = emitter
            .events
            .iter()
            .filter(|e| matches!(e, OrganizationEvent::LoyaltyModified { .. }))
            .collect();

        assert_eq!(loyalty_events.len(), 3);

        // Check Devotee in Cult has excellent fit
        let devotee_event = loyalty_events.iter().find(|e| {
            if let OrganizationEvent::LoyaltyModified { member_id, .. } = e {
                member_id.0 == "leader"
            } else {
                false
            }
        });

        if let Some(OrganizationEvent::LoyaltyModified {
            modifier,
            fit_quality,
            ..
        }) = devotee_event
        {
            assert_eq!(*modifier, 1.5);
            assert_eq!(*fit_quality, FitQuality::Excellent);
        }
    }

    #[test]
    fn test_authority_concentration_cult() {
        let config = OrganizationConfig::default();
        let mut state = OrganizationState::default();

        let input = OrganizationInput {
            org_type: OrganizationType::Cult,
            member_count: 5,
            decision_importance: 0.5,
            urgency: 0.5,
            leader_charisma: 0.9,
            member_archetypes: vec![
                (MemberId("leader".into()), MemberArchetype::Devotee),
                (MemberId("m1".into()), MemberArchetype::Devotee),
                (MemberId("m2".into()), MemberArchetype::Devotee),
                (MemberId("m3".into()), MemberArchetype::Devotee),
                (MemberId("m4".into()), MemberArchetype::Devotee),
            ],
            current_tick: 100,
        };

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleOrganizationMechanic::step(&config, &mut state, input, &mut emitter);

        // In a Cult, leader should have 100% authority
        assert_eq!(
            state.authority_distribution.get(&MemberId("leader".into())),
            Some(&1.0)
        );

        // Concentration should be maximum
        let authority_event = emitter.events.iter().find_map(|e| {
            if let OrganizationEvent::AuthorityRebalanced {
                concentration_index,
                ..
            } = e
            {
                Some(*concentration_index)
            } else {
                None
            }
        });

        // Cult should have high concentration (close to 1.0)
        assert!(authority_event.unwrap() > 0.7);
    }

    #[test]
    fn test_authority_concentration_democracy() {
        let config = OrganizationConfig::default();
        let mut state = OrganizationState::default();

        let input = OrganizationInput {
            org_type: OrganizationType::Democracy,
            member_count: 4,
            decision_importance: 0.5,
            urgency: 0.5,
            leader_charisma: 0.5,
            member_archetypes: vec![
                (MemberId("m0".into()), MemberArchetype::Egalitarian),
                (MemberId("m1".into()), MemberArchetype::Egalitarian),
                (MemberId("m2".into()), MemberArchetype::Egalitarian),
                (MemberId("m3".into()), MemberArchetype::Egalitarian),
            ],
            current_tick: 100,
        };

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleOrganizationMechanic::step(&config, &mut state, input, &mut emitter);

        // In Democracy, all should have equal authority
        for weight in state.authority_distribution.values() {
            assert!((weight - 0.25).abs() < 0.01);
        }

        // Concentration should be low (flat distribution)
        let authority_event = emitter.events.iter().find_map(|e| {
            if let OrganizationEvent::AuthorityRebalanced {
                concentration_index,
                ..
            } = e
            {
                Some(*concentration_index)
            } else {
                None
            }
        });

        // Democracy should have low concentration (close to 0.0)
        assert!(authority_event.unwrap() < 0.1);
    }
}
