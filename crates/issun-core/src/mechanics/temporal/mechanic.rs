//! The core TemporalMechanic implementation.

use std::marker::PhantomData;

use crate::mechanics::{EventEmitter, Mechanic, ParallelSafe};

use super::policies::TemporalPolicy;
use super::strategies::StandardTemporalPolicy;
use super::types::{
    ActionBudget, ActionCost, TemporalConfig, TemporalEvent, TemporalInput, TemporalState,
};

/// The core temporal mechanic for time and action budget management.
///
/// # Type Parameters
///
/// - `P`: Temporal policy (determines cost calculation, reset behavior, etc.)
///
/// # Overview
///
/// The temporal mechanic handles:
/// - **Time Tracking**: Current tick/datetime, time progression
/// - **Action Budgets**: Discrete points or continuous energy
/// - **Cost Calculation**: Policy-driven action costs with modifiers
/// - **Budget Reset**: Day-based, tick-based, or continuous regeneration
/// - **Time-Based Effects**: Time of day and season modifiers
///
/// # Supported Systems
///
/// - **Turn-Based RPG**: Discrete actions per day/turn
/// - **Real-Time RPG**: Stamina/energy with regeneration
/// - **Strategy Games**: Variable costs, season effects
/// - **Persona-Style**: Calendar-based with limited daily actions
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::temporal::prelude::*;
/// use issun_core::mechanics::{Mechanic, EventEmitter};
///
/// // Use default policy
/// type GameTemporal = TemporalMechanic;
///
/// // Create configuration
/// let config = TemporalConfig::turn_based(3); // 3 actions per day
/// let mut state = TemporalState::with_points(3);
///
/// // Request an action
/// let input = TemporalInput {
///     current_time: TemporalPoint::DateTime(GameDateTime::new(1, 1, 1, 10, 0)),
///     requested_action: Some(ActionRequest::new("move")),
///     ..Default::default()
/// };
///
/// // Event collector
/// # struct VecEmitter(Vec<TemporalEvent>);
/// # impl EventEmitter<TemporalEvent> for VecEmitter {
/// #     fn emit(&mut self, event: TemporalEvent) { self.0.push(event); }
/// # }
/// let mut emitter = VecEmitter(vec![]);
///
/// // Execute
/// GameTemporal::step(&config, &mut state, input, &mut emitter);
///
/// // Check result
/// if let ActionBudget::Points(ap) = &state.budget {
///     assert_eq!(ap.available, 2); // Used 1 action
/// }
/// ```
pub struct TemporalMechanic<P: TemporalPolicy = StandardTemporalPolicy> {
    _marker: PhantomData<P>,
}

impl<P> Mechanic for TemporalMechanic<P>
where
    P: TemporalPolicy,
{
    type Config = TemporalConfig;
    type State = TemporalState;
    type Input = TemporalInput;
    type Event = TemporalEvent;

    // Temporal is per-entity and doesn't need cross-entity access
    type Execution = ParallelSafe;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        // 1. Handle time-based regeneration (for real-time systems)
        if let Some(delta_seconds) = input.delta_seconds {
            let old_current = match &state.budget {
                ActionBudget::Energy(e) => e.current,
                _ => 0.0,
            };

            state.budget.regenerate(delta_seconds);

            // Emit regen event if energy changed
            if let ActionBudget::Energy(e) = &state.budget {
                if (e.current - old_current).abs() > 0.01 {
                    emitter.emit(TemporalEvent::EnergyRegenerated {
                        amount: e.current - old_current,
                        current: e.current,
                    });
                }
            }
        }

        // 2. Check for time-based reset
        let should_reset = P::should_reset(config, &state.last_reset_time, &input.current_time);

        if should_reset {
            state.budget.reset();
            state.last_reset_time = input.current_time.clone();
            state.actions_this_period = 0;

            let new_available = match &state.budget {
                ActionBudget::Points(ap) => ap.available,
                ActionBudget::Energy(ae) => ae.current as u32,
            };

            emitter.emit(TemporalEvent::BudgetReset { new_available });

            // Check for day change event
            let old_dt = state.last_action_time.to_datetime(&config.calendar);
            let new_dt = input.current_time.to_datetime(&config.calendar);
            if !old_dt.is_same_day(&new_dt) {
                emitter.emit(TemporalEvent::DayChanged {
                    new_day: new_dt.day,
                    new_month: new_dt.month,
                    new_year: new_dt.year,
                });
            }
        }

        // 3. Handle action request
        if let Some(action_request) = &input.requested_action {
            // Calculate cost
            let cost = if let Some(override_cost) = action_request.override_cost {
                ActionCost::new(override_cost)
            } else {
                P::calculate_cost(config, &action_request.action_type, &input.actor_context)
            };

            // Apply time-based modifiers
            let datetime = input.current_time.to_datetime(&config.calendar);
            let time_modifier = P::time_of_day_modifier(config, &datetime);
            let season = P::get_season(config, &datetime);
            let season_modifier = P::season_modifier(config, season);
            let final_cost = cost.apply_modifier(time_modifier * season_modifier);

            // Attempt to consume
            match state.budget.consume(&final_cost) {
                Ok(result) => {
                    state.last_action_time = input.current_time.clone();
                    state.actions_this_period += 1;

                    emitter.emit(TemporalEvent::ActionConsumed {
                        action_type: action_request.action_type.clone(),
                        cost: final_cost,
                        remaining: result.remaining,
                    });

                    if result.depleted {
                        emitter.emit(TemporalEvent::BudgetDepleted);
                    }
                }
                Err(reason) => {
                    emitter.emit(TemporalEvent::ActionRejected {
                        action_type: action_request.action_type.clone(),
                        reason,
                    });
                }
            }
        }

        // 4. Emit time advancement event if time changed
        if state.last_action_time != input.current_time {
            emitter.emit(TemporalEvent::TimeAdvanced {
                from: state.last_action_time.clone(),
                to: input.current_time.clone(),
            });
        }
    }
}

/// Type alias for simple temporal mechanic using default policy.
pub type SimpleTemporalMechanic = TemporalMechanic<StandardTemporalPolicy>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::temporal::types::{
        ActionBudget, ActionRequest, ActorContext, GameDateTime, TemporalPoint,
    };

    struct VecEmitter {
        events: Vec<TemporalEvent>,
    }

    impl EventEmitter<TemporalEvent> for VecEmitter {
        fn emit(&mut self, event: TemporalEvent) {
            self.events.push(event);
        }
    }

    #[test]
    fn test_action_consumption() {
        let config = TemporalConfig::turn_based(3);
        let mut state = TemporalState::with_points(3);

        let input = TemporalInput {
            current_time: TemporalPoint::DateTime(GameDateTime::new(1, 1, 1, 10, 0)),
            requested_action: Some(ActionRequest::new("move")),
            ..Default::default()
        };

        let mut emitter = VecEmitter { events: vec![] };
        SimpleTemporalMechanic::step(&config, &mut state, input, &mut emitter);

        // Should have consumed 1 action
        if let ActionBudget::Points(ap) = &state.budget {
            assert_eq!(ap.available, 2);
        } else {
            panic!("Expected ActionPoints");
        }

        // Should have emitted ActionConsumed
        let consumed = emitter
            .events
            .iter()
            .any(|e| matches!(e, TemporalEvent::ActionConsumed { .. }));
        assert!(consumed);
    }

    #[test]
    fn test_budget_depletion() {
        let config = TemporalConfig::turn_based(2);
        let mut state = TemporalState::with_points(2);

        let mut emitter = VecEmitter { events: vec![] };

        // Use first action
        let input1 = TemporalInput {
            current_time: TemporalPoint::DateTime(GameDateTime::new(1, 1, 1, 10, 0)),
            requested_action: Some(ActionRequest::new("move")),
            ..Default::default()
        };
        SimpleTemporalMechanic::step(&config, &mut state, input1, &mut emitter);

        // Use second action
        let input2 = TemporalInput {
            current_time: TemporalPoint::DateTime(GameDateTime::new(1, 1, 1, 10, 30)),
            requested_action: Some(ActionRequest::new("attack")),
            ..Default::default()
        };
        SimpleTemporalMechanic::step(&config, &mut state, input2, &mut emitter);

        // Should be depleted
        if let ActionBudget::Points(ap) = &state.budget {
            assert!(ap.is_depleted());
        }

        // Should have emitted BudgetDepleted
        let depleted = emitter
            .events
            .iter()
            .any(|e| matches!(e, TemporalEvent::BudgetDepleted));
        assert!(depleted);
    }

    #[test]
    fn test_action_rejection() {
        let config = TemporalConfig::turn_based(1);
        let mut state = TemporalState::with_points(1);

        // Use the only action
        let input1 = TemporalInput {
            current_time: TemporalPoint::DateTime(GameDateTime::new(1, 1, 1, 10, 0)),
            requested_action: Some(ActionRequest::new("move")),
            ..Default::default()
        };

        let mut emitter = VecEmitter { events: vec![] };
        SimpleTemporalMechanic::step(&config, &mut state, input1, &mut emitter);

        // Try to use another action
        let input2 = TemporalInput {
            current_time: TemporalPoint::DateTime(GameDateTime::new(1, 1, 1, 10, 30)),
            requested_action: Some(ActionRequest::new("attack")),
            ..Default::default()
        };
        SimpleTemporalMechanic::step(&config, &mut state, input2, &mut emitter);

        // Should have emitted ActionRejected
        let rejected = emitter
            .events
            .iter()
            .any(|e| matches!(e, TemporalEvent::ActionRejected { .. }));
        assert!(rejected);
    }

    #[test]
    fn test_day_change_reset() {
        let config = TemporalConfig::turn_based(3);
        let mut state = TemporalState::with_points(3);
        state.last_reset_time = TemporalPoint::DateTime(GameDateTime::new(1, 1, 1, 0, 0));
        state.last_action_time = TemporalPoint::DateTime(GameDateTime::new(1, 1, 1, 20, 0));

        // Use 2 actions on day 1
        let input1 = TemporalInput {
            current_time: TemporalPoint::DateTime(GameDateTime::new(1, 1, 1, 10, 0)),
            requested_action: Some(ActionRequest::new("move")),
            ..Default::default()
        };

        let mut emitter = VecEmitter { events: vec![] };
        SimpleTemporalMechanic::step(&config, &mut state, input1, &mut emitter);

        let input2 = TemporalInput {
            current_time: TemporalPoint::DateTime(GameDateTime::new(1, 1, 1, 12, 0)),
            requested_action: Some(ActionRequest::new("talk")),
            ..Default::default()
        };
        SimpleTemporalMechanic::step(&config, &mut state, input2, &mut emitter);

        if let ActionBudget::Points(ap) = &state.budget {
            assert_eq!(ap.available, 1);
        }

        // Advance to day 2
        let input3 = TemporalInput {
            current_time: TemporalPoint::DateTime(GameDateTime::new(1, 1, 2, 8, 0)),
            requested_action: None,
            ..Default::default()
        };
        SimpleTemporalMechanic::step(&config, &mut state, input3, &mut emitter);

        // Should have reset
        if let ActionBudget::Points(ap) = &state.budget {
            assert_eq!(ap.available, 3);
        }

        // Should have emitted BudgetReset and DayChanged
        let reset = emitter
            .events
            .iter()
            .any(|e| matches!(e, TemporalEvent::BudgetReset { .. }));
        let day_changed = emitter
            .events
            .iter()
            .any(|e| matches!(e, TemporalEvent::DayChanged { .. }));
        assert!(reset);
        assert!(day_changed);
    }

    #[test]
    fn test_energy_regeneration() {
        let config = TemporalConfig::real_time(100.0, 10.0); // 10 energy/sec
        let mut state = TemporalState::with_energy(100.0, 10.0);

        // Consume some energy
        if let ActionBudget::Energy(e) = &mut state.budget {
            e.consume(50.0).unwrap();
        }

        // Regenerate over 2 seconds
        let input = TemporalInput {
            current_time: TemporalPoint::Tick(100),
            delta_seconds: Some(2.0),
            requested_action: None,
            ..Default::default()
        };

        let mut emitter = VecEmitter { events: vec![] };
        SimpleTemporalMechanic::step(&config, &mut state, input, &mut emitter);

        // Should have regenerated 20 energy
        if let ActionBudget::Energy(e) = &state.budget {
            assert!((e.current - 70.0).abs() < 0.1);
        }

        // Should have emitted EnergyRegenerated
        let regen = emitter
            .events
            .iter()
            .any(|e| matches!(e, TemporalEvent::EnergyRegenerated { .. }));
        assert!(regen);
    }

    #[test]
    fn test_skill_reduces_cost() {
        let config = TemporalConfig::turn_based(3).with_action_cost("expensive", 2);
        let mut state = TemporalState::with_points(3);

        // Skilled actor
        let input = TemporalInput {
            current_time: TemporalPoint::DateTime(GameDateTime::new(1, 1, 1, 10, 0)),
            requested_action: Some(ActionRequest::new("expensive")),
            actor_context: ActorContext {
                skill_level: 1.0, // High skill
                efficiency_bonus: 0.0,
                temporary_modifiers: Default::default(),
            },
            ..Default::default()
        };

        let mut emitter = VecEmitter { events: vec![] };
        SimpleTemporalMechanic::step(&config, &mut state, input, &mut emitter);

        // With skill, cost should be reduced from 2
        if let ActionBudget::Points(ap) = &state.budget {
            // Base cost 2, with skill modifier should be less
            assert!(ap.available >= 1); // At least 1 remaining (cost <= 2)
        }
    }

    #[test]
    fn test_override_cost() {
        let config = TemporalConfig::turn_based(5);
        let mut state = TemporalState::with_points(5);

        // Override cost to 3
        let input = TemporalInput {
            current_time: TemporalPoint::DateTime(GameDateTime::new(1, 1, 1, 10, 0)),
            requested_action: Some(ActionRequest::with_cost("special", 3)),
            ..Default::default()
        };

        let mut emitter = VecEmitter { events: vec![] };
        SimpleTemporalMechanic::step(&config, &mut state, input, &mut emitter);

        // Should have used exactly 3
        if let ActionBudget::Points(ap) = &state.budget {
            assert_eq!(ap.available, 2);
        }
    }

    #[test]
    fn test_actions_this_period_tracking() {
        let config = TemporalConfig::turn_based(5);
        let mut state = TemporalState::with_points(5);

        let mut emitter = VecEmitter { events: vec![] };

        for i in 0..3 {
            let input = TemporalInput {
                current_time: TemporalPoint::DateTime(GameDateTime::new(1, 1, 1, 10 + i, 0)),
                requested_action: Some(ActionRequest::new("action")),
                ..Default::default()
            };
            SimpleTemporalMechanic::step(&config, &mut state, input, &mut emitter);
        }

        assert_eq!(state.actions_this_period, 3);
    }
}
