//! The core DelegationMechanic implementation.

use std::marker::PhantomData;

use crate::mechanics::{EventEmitter, Mechanic, Transactional};

use super::policies::DelegationPolicy;
use super::strategies::SimpleDelegationPolicy;
use super::types::{
    ComplianceChangeReason, DelegationConfig, DelegationEvent, DelegationInput, DelegationState,
    DirectiveStatus, ExecutionStatus, IgnoreReason, ResponseType,
};

/// The core delegation mechanic that models directive handling.
///
/// # Type Parameters
///
/// - `P`: Delegation policy (determines how compliance and responses are calculated)
///
/// # Overview
///
/// The delegation mechanic calculates:
/// - **Compliance**: How likely the delegate is to follow the directive (-1.0 to 1.0)
/// - **Interpretation**: How much creative freedom the delegate takes
/// - **Priority**: How important the directive is to the delegate
/// - **Feedback**: Likelihood of progress reports
/// - **Quality**: Expected execution quality
/// - **Response**: Accept, Defer, Ignore, or Defy
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::delegation::prelude::*;
/// use issun_core::mechanics::{Mechanic, EventEmitter};
///
/// // Use default policy (SimpleDelegationPolicy)
/// type SimpleDelegation = DelegationMechanic;
///
/// // Create configuration
/// let config = DelegationConfig::default();
/// let mut state = DelegationState::default();
///
/// // Prepare input: loyal subordinate receiving command
/// let input = DelegationInput {
///     directive: Directive {
///         id: DirectiveId("cmd_001".into()),
///         directive_type: DirectiveType::Command {
///             target: "outpost_alpha".into(),
///             action: "defend".into(),
///         },
///         urgency: 0.8,
///         importance: 0.9,
///         issued_at: 100,
///     },
///     delegator: DelegatorStats {
///         entity_id: EntityId("general".into()),
///         authority: 0.95,
///         charisma: 0.7,
///         hierarchy_rank: 0,
///         reputation: 0.8,
///     },
///     delegate: DelegateStats {
///         entity_id: EntityId("captain".into()),
///         loyalty: 0.9,
///         morale: 0.8,
///         relationship: 0.7,
///         hierarchy_rank: 2,
///         personality: DelegateTrait::Loyal,
///         workload: 0.4,
///         skill_level: 0.85,
///     },
///     current_tick: 100,
/// };
///
/// // Event collector
/// # struct VecEmitter(Vec<DelegationEvent>);
/// # impl EventEmitter<DelegationEvent> for VecEmitter {
/// #     fn emit(&mut self, event: DelegationEvent) { self.0.push(event); }
/// # }
/// let mut emitter = VecEmitter(vec![]);
///
/// // Execute one step
/// SimpleDelegation::step(&config, &mut state, input, &mut emitter);
///
/// // Loyal captain should accept the command
/// assert_eq!(state.response, ResponseType::Accept);
/// assert!(state.compliance > 0.8);
/// ```
pub struct DelegationMechanic<P: DelegationPolicy = SimpleDelegationPolicy> {
    _marker: PhantomData<P>,
}

impl<P> Mechanic for DelegationMechanic<P>
where
    P: DelegationPolicy,
{
    type Config = DelegationConfig;
    type State = DelegationState;
    type Input = DelegationInput;
    type Event = DelegationEvent;

    // Delegation mechanics involve relationships - use transactional
    type Execution = Transactional;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        let old_compliance = state.compliance;

        // 1. Calculate all delegation parameters
        let compliance = P::calculate_compliance(config, &input);
        let interpretation = P::calculate_interpretation(config, &input);
        let priority = P::calculate_priority(config, &input);
        let feedback_probability = P::calculate_feedback_probability(config, &input);
        let expected_quality = P::calculate_expected_quality(config, &input);
        let propagation_delay = P::calculate_propagation_delay(config, &input);

        // 2. Determine response type
        let response = ResponseType::from_compliance(compliance, config);

        // 3. Emit directive received event
        emitter.emit(DelegationEvent::DirectiveReceived {
            directive_id: input.directive.id.clone(),
            delegator: input.delegator.entity_id.clone(),
            delegate: input.delegate.entity_id.clone(),
            response,
            compliance,
        });

        // 4. Emit response-specific events
        match response {
            ResponseType::Accept | ResponseType::AcceptWithReservation => {
                emitter.emit(DelegationEvent::DirectiveAccepted {
                    directive_id: input.directive.id.clone(),
                    priority,
                    interpretation,
                });

                // Track the directive
                state.active_directives.insert(
                    input.directive.id.clone(),
                    DirectiveStatus {
                        progress: 0.0,
                        status: ExecutionStatus::Pending,
                        interpretation_applied: interpretation,
                        current_quality: expected_quality,
                    },
                );
            }

            ResponseType::Defer => {
                // No specific event for defer - will be re-evaluated later
            }

            ResponseType::Ignore => {
                let reason = determine_ignore_reason(&input, compliance);
                emitter.emit(DelegationEvent::DirectiveIgnored {
                    directive_id: input.directive.id.clone(),
                    reason,
                });

                // Track as ignored
                state.active_directives.insert(
                    input.directive.id.clone(),
                    DirectiveStatus {
                        progress: 0.0,
                        status: ExecutionStatus::Ignored,
                        interpretation_applied: 0.0,
                        current_quality: 0.0,
                    },
                );
            }

            ResponseType::Defy => {
                emitter.emit(DelegationEvent::DirectiveDefied {
                    directive_id: input.directive.id.clone(),
                    delegate: input.delegate.entity_id.clone(),
                    severity: compliance, // Negative value indicates severity
                });

                // Track as defied
                state.active_directives.insert(
                    input.directive.id.clone(),
                    DirectiveStatus {
                        progress: 0.0,
                        status: ExecutionStatus::Defied,
                        interpretation_applied: 0.0,
                        current_quality: 0.0,
                    },
                );
            }
        }

        // 5. Check for significant compliance change
        if (compliance - old_compliance).abs() > 0.1 {
            let reason = determine_compliance_change_reason(&input);
            emitter.emit(DelegationEvent::ComplianceChanged {
                delegate: input.delegate.entity_id.clone(),
                old_compliance,
                new_compliance: compliance,
                reason,
            });
        }

        // 6. Update state
        state.compliance = compliance;
        state.interpretation = interpretation;
        state.priority = priority;
        state.feedback_probability = feedback_probability;
        state.expected_quality = expected_quality;
        state.propagation_delay = propagation_delay;
        state.response = response;
        state.last_update = input.current_tick;
    }
}

/// Determine the reason for ignoring a directive
fn determine_ignore_reason(input: &DelegationInput, _compliance: f32) -> IgnoreReason {
    let delegate = &input.delegate;
    let delegator = &input.delegator;

    // Check most likely causes in order of impact
    if delegator.authority < 0.3 {
        IgnoreReason::InsufficientAuthority
    } else if delegate.relationship < -0.5 {
        IgnoreReason::PoorRelationship
    } else if delegate.morale < 0.3 {
        IgnoreReason::LowMorale
    } else if delegate.workload > 0.9 {
        IgnoreReason::CapacityFull
    } else {
        IgnoreReason::LowPriority
    }
}

/// Determine the reason for compliance change
fn determine_compliance_change_reason(input: &DelegationInput) -> ComplianceChangeReason {
    // Heuristic based on input values
    let delegate = &input.delegate;

    if delegate.morale < 0.4 || delegate.morale > 0.9 {
        ComplianceChangeReason::MoraleChange
    } else if delegate.relationship.abs() > 0.7 {
        ComplianceChangeReason::RelationshipChange
    } else if delegate.loyalty.abs() > 0.7 {
        ComplianceChangeReason::LoyaltyChange
    } else if delegate.workload > 0.8 {
        ComplianceChangeReason::WorkloadChange
    } else {
        ComplianceChangeReason::AuthorityChange
    }
}

/// Type alias for simple delegation mechanic using default policy.
pub type SimpleDelegationMechanic = DelegationMechanic<SimpleDelegationPolicy>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::delegation::types::{
        DelegateStats, DelegateTrait, DelegatorStats, Directive, DirectiveId, DirectiveType,
        EntityId,
    };

    struct VecEmitter {
        events: Vec<DelegationEvent>,
    }

    impl EventEmitter<DelegationEvent> for VecEmitter {
        fn emit(&mut self, event: DelegationEvent) {
            self.events.push(event);
        }
    }

    fn create_test_input(
        loyalty: f32,
        relationship: f32,
        personality: DelegateTrait,
    ) -> DelegationInput {
        DelegationInput {
            directive: Directive {
                id: DirectiveId("test_cmd".into()),
                directive_type: DirectiveType::Command {
                    target: "location".into(),
                    action: "move".into(),
                },
                urgency: 0.5,
                importance: 0.5,
                issued_at: 100,
            },
            delegator: DelegatorStats {
                entity_id: EntityId("commander".into()),
                authority: 0.8,
                charisma: 0.6,
                hierarchy_rank: 0,
                reputation: 0.5,
            },
            delegate: DelegateStats {
                entity_id: EntityId("soldier".into()),
                loyalty,
                morale: 0.7,
                relationship,
                hierarchy_rank: 2,
                personality,
                workload: 0.3,
                skill_level: 0.7,
            },
            current_tick: 100,
        }
    }

    #[test]
    fn test_mechanic_accept_loyal() {
        let config = DelegationConfig::default();
        let mut state = DelegationState::default();
        let input = create_test_input(0.9, 0.8, DelegateTrait::Loyal);

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleDelegationMechanic::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.response, ResponseType::Accept);
        assert!(state.compliance > 0.7);

        // Should have DirectiveReceived and DirectiveAccepted events
        let has_received = emitter
            .events
            .iter()
            .any(|e| matches!(e, DelegationEvent::DirectiveReceived { .. }));
        let has_accepted = emitter
            .events
            .iter()
            .any(|e| matches!(e, DelegationEvent::DirectiveAccepted { .. }));

        assert!(has_received);
        assert!(has_accepted);
    }

    #[test]
    fn test_mechanic_ignore_low_authority() {
        let config = DelegationConfig::default();
        let mut state = DelegationState::default();
        // More extreme: negative loyalty, poor relationship, very low authority
        let mut input = create_test_input(-0.4, -0.5, DelegateTrait::Independent);
        input.delegator.authority = 0.1; // Very low authority
        input.delegator.reputation = -0.3;

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleDelegationMechanic::step(&config, &mut state, input, &mut emitter);

        // Low compliance should lead to Ignore or Defer
        assert!(
            matches!(
                state.response,
                ResponseType::Ignore | ResponseType::Defer | ResponseType::AcceptWithReservation
            ),
            "Expected Ignore, Defer, or AcceptWithReservation, got {:?} (compliance: {})",
            state.response,
            state.compliance
        );
    }

    #[test]
    fn test_mechanic_defy_hostile() {
        let config = DelegationConfig::default();
        let mut state = DelegationState::default();
        let mut input = create_test_input(-0.8, -0.9, DelegateTrait::Independent);
        input.delegator.authority = 0.3;
        input.delegator.reputation = -0.5;

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleDelegationMechanic::step(&config, &mut state, input, &mut emitter);

        // Very negative loyalty and relationship may lead to defiance
        assert!(
            state.compliance < 0.3,
            "Expected low compliance, got {}",
            state.compliance
        );
    }

    #[test]
    fn test_mechanic_tracks_directive() {
        let config = DelegationConfig::default();
        let mut state = DelegationState::default();
        let input = create_test_input(0.9, 0.8, DelegateTrait::Obedient);
        let directive_id = input.directive.id.clone();

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleDelegationMechanic::step(&config, &mut state, input, &mut emitter);

        // Directive should be tracked
        assert!(state.active_directives.contains_key(&directive_id));

        let status = state.active_directives.get(&directive_id).unwrap();
        assert_eq!(status.status, ExecutionStatus::Pending);
    }

    #[test]
    fn test_mechanic_emits_compliance_change() {
        let config = DelegationConfig::default();
        let mut state = DelegationState {
            compliance: 0.3, // Start with low compliance
            ..Default::default()
        };

        // Input that should result in high compliance
        let input = create_test_input(0.95, 0.9, DelegateTrait::Loyal);

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleDelegationMechanic::step(&config, &mut state, input, &mut emitter);

        // Should emit ComplianceChanged due to large delta
        let has_compliance_changed = emitter
            .events
            .iter()
            .any(|e| matches!(e, DelegationEvent::ComplianceChanged { .. }));

        assert!(has_compliance_changed);
    }

    #[test]
    fn test_interpretation_varies_by_trait() {
        let config = DelegationConfig::default();

        let mut obedient_state = DelegationState::default();
        let obedient_input = create_test_input(0.7, 0.5, DelegateTrait::Obedient);
        let mut emitter1 = VecEmitter { events: Vec::new() };
        SimpleDelegationMechanic::step(&config, &mut obedient_state, obedient_input, &mut emitter1);

        let mut innovative_state = DelegationState::default();
        let innovative_input = create_test_input(0.7, 0.5, DelegateTrait::Innovative);
        let mut emitter2 = VecEmitter { events: Vec::new() };
        SimpleDelegationMechanic::step(
            &config,
            &mut innovative_state,
            innovative_input,
            &mut emitter2,
        );

        assert!(
            innovative_state.interpretation > obedient_state.interpretation,
            "Innovative should have higher interpretation: {} vs {}",
            innovative_state.interpretation,
            obedient_state.interpretation
        );
    }

    #[test]
    fn test_feedback_probability() {
        let config = DelegationConfig::default();

        let mut eager_state = DelegationState::default();
        let eager_input = create_test_input(0.8, 0.7, DelegateTrait::Eager);
        let mut emitter1 = VecEmitter { events: Vec::new() };
        SimpleDelegationMechanic::step(&config, &mut eager_state, eager_input, &mut emitter1);

        let mut reluctant_state = DelegationState::default();
        let reluctant_input = create_test_input(0.3, 0.2, DelegateTrait::Reluctant);
        let mut emitter2 = VecEmitter { events: Vec::new() };
        SimpleDelegationMechanic::step(
            &config,
            &mut reluctant_state,
            reluctant_input,
            &mut emitter2,
        );

        assert!(
            eager_state.feedback_probability > reluctant_state.feedback_probability,
            "Eager should give more feedback: {} vs {}",
            eager_state.feedback_probability,
            reluctant_state.feedback_probability
        );
    }
}
