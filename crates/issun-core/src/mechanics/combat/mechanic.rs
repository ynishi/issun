//! The core CombatMechanic implementation.
//!
//! This module defines the `CombatMechanic<D, F, E>` struct which composes
//! three policy types into a complete combat system.

use std::marker::PhantomData;

use crate::mechanics::{EventEmitter, Mechanic};

use super::policies::{CriticalPolicy, DamageCalculationPolicy, DefensePolicy, ElementalPolicy};
use super::strategies::{LinearDamageCalculation, NoCritical, NoElemental, SubtractiveDefense};
use super::types::{CombatConfig, CombatEvent, CombatInput, CombatState};

/// The core combat mechanic that composes three policy types.
///
/// This struct is the "shell" that assembles concrete strategies into a
/// complete combat system. All logic is delegated to the policy implementations,
/// ensuring zero-cost abstraction through static dispatch.
///
/// # Type Parameters
///
/// - `D`: Damage calculation policy (`DamageCalculationPolicy`)
/// - `F`: Defense application policy (`DefensePolicy`)
/// - `E`: Elemental affinity policy (`ElementalPolicy`)
/// - `C`: Critical hit policy (`CriticalPolicy`)
///
/// # Default Policies
///
/// If you don't specify type parameters, sensible defaults are used:
/// - Damage: `LinearDamageCalculation` (damage = attack power)
/// - Defense: `SubtractiveDefense` (damage = attack - defense)
/// - Elemental: `NoElemental` (no elemental system)
/// - Critical: `NoCritical` (no critical hits)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::combat::prelude::*;
/// use issun_core::mechanics::{Mechanic, EventEmitter};
///
/// // Use all defaults
/// type SimpleCombat = CombatMechanic;
///
/// // Customize only elemental system
/// type ElementalCombat = CombatMechanic<_, _, ElementalAffinity>;
///
/// // Fully customize
/// type AdvancedCombat = CombatMechanic<
///     ScalingDamageCalculation,
///     PercentageReduction,
///     ElementalAffinity,
/// >;
///
/// // Setup
/// let config = CombatConfig::default();
/// let mut state = CombatState::new(100);
/// let input = CombatInput {
///     attacker_power: 30,
///     defender_defense: 10,
///     attacker_element: None,
///     defender_element: None,
/// };
///
/// // Collect events
/// struct VecEmitter(Vec<CombatEvent>);
/// impl EventEmitter<CombatEvent> for VecEmitter {
///     fn emit(&mut self, event: CombatEvent) {
///         self.0.push(event);
///     }
/// }
/// let mut emitter = VecEmitter(vec![]);
///
/// // Execute combat logic
/// SimpleCombat::step(&config, &mut state, input, &mut emitter);
///
/// // Check results
/// assert_eq!(state.current_hp, 80); // 100 - (30 - 10) = 80
/// assert_eq!(emitter.0.len(), 1);
/// ```
pub struct CombatMechanic<
    D: DamageCalculationPolicy = LinearDamageCalculation,
    F: DefensePolicy = SubtractiveDefense,
    E: ElementalPolicy = NoElemental,
    C: CriticalPolicy = NoCritical,
> {
    _marker: PhantomData<(D, F, E, C)>,
}

impl<D, F, E, C> Mechanic for CombatMechanic<D, F, E, C>
where
    D: DamageCalculationPolicy,
    F: DefensePolicy,
    E: ElementalPolicy,
    C: CriticalPolicy,
{
    type Config = CombatConfig;
    type State = CombatState;
    type Input = CombatInput;
    type Event = CombatEvent;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        // 1. Calculate base damage (delegated to policy D)
        let base_damage = D::calculate_base_damage(input.attacker_power, config);

        // 2. Apply defense (delegated to policy F)
        let after_defense = F::apply_defense(base_damage, input.defender_defense, config);

        // 3. Apply elemental modifier (delegated to policy E)
        let after_elemental = E::apply_elemental_modifier(
            after_defense,
            input.attacker_element,
            input.defender_element,
        );

        // 4. Apply critical hit (delegated to policy C)
        let (final_damage, is_critical) = C::apply_critical(after_elemental, config);

        // 5. Check if damage is completely negated
        if final_damage <= 0 {
            emitter.emit(CombatEvent::Blocked {
                attempted_damage: after_elemental,
            });
            return;
        }

        // 6. Apply damage to state
        state.current_hp = (state.current_hp - final_damage).max(0);
        let is_fatal = state.current_hp == 0;

        // 7. Emit damage event
        emitter.emit(CombatEvent::DamageDealt {
            amount: final_damage,
            is_critical,
            is_fatal,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::combat::strategies::{
        ElementalAffinity, PercentageReduction, ScalingDamageCalculation,
    };
    use crate::mechanics::combat::types::Element;

    // Helper: Event collector
    struct VecEmitter(Vec<CombatEvent>);
    impl EventEmitter<CombatEvent> for VecEmitter {
        fn emit(&mut self, event: CombatEvent) {
            self.0.push(event);
        }
    }

    type SimpleCombat = CombatMechanic; // Uses all defaults

    #[test]
    fn test_simple_combat() {
        let config = CombatConfig::default();
        let mut state = CombatState::new(100);
        let input = CombatInput {
            attacker_power: 30,
            defender_defense: 10,
            attacker_element: None,
            defender_element: None,
        };

        let mut emitter = VecEmitter(vec![]);
        SimpleCombat::step(&config, &mut state, input, &mut emitter);

        // (30 - 10) = 20 damage
        assert_eq!(state.current_hp, 80);
        assert_eq!(emitter.0.len(), 1);

        match &emitter.0[0] {
            CombatEvent::DamageDealt {
                amount,
                is_critical,
                is_fatal,
            } => {
                assert_eq!(*amount, 20);
                assert!(!is_critical);
                assert!(!is_fatal);
            }
            _ => panic!("Expected DamageDealt event"),
        }
    }

    #[test]
    fn test_fatal_damage() {
        let config = CombatConfig::default();
        let mut state = CombatState::new(10);
        let input = CombatInput {
            attacker_power: 50,
            defender_defense: 0,
            attacker_element: None,
            defender_element: None,
        };

        let mut emitter = VecEmitter(vec![]);
        SimpleCombat::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.current_hp, 0);
        assert!(state.is_dead());

        match &emitter.0[0] {
            CombatEvent::DamageDealt { is_fatal, .. } => {
                assert!(is_fatal);
            }
            _ => panic!("Expected DamageDealt event"),
        }
    }

    #[test]
    fn test_min_damage() {
        let config = CombatConfig {
            min_damage: 1,
            ..Default::default()
        };
        let mut state = CombatState::new(100);
        let input = CombatInput {
            attacker_power: 5,
            defender_defense: 100, // Very high defense
            attacker_element: None,
            defender_element: None,
        };

        let mut emitter = VecEmitter(vec![]);
        SimpleCombat::step(&config, &mut state, input, &mut emitter);

        // Min damage should be enforced
        assert_eq!(state.current_hp, 99);

        match &emitter.0[0] {
            CombatEvent::DamageDealt { amount, .. } => {
                assert_eq!(*amount, 1);
            }
            _ => panic!("Expected DamageDealt event"),
        }
    }

    #[test]
    fn test_elemental_combat() {
        type ElementalCombat =
            CombatMechanic<LinearDamageCalculation, SubtractiveDefense, ElementalAffinity>;

        let config = CombatConfig::default();
        let mut state = CombatState::new(100);

        // Fire vs Ice (super effective, 2x)
        let input = CombatInput {
            attacker_power: 30,
            defender_defense: 10,
            attacker_element: Some(Element::Fire),
            defender_element: Some(Element::Ice),
        };

        let mut emitter = VecEmitter(vec![]);
        ElementalCombat::step(&config, &mut state, input, &mut emitter);

        // (30 - 10) * 2.0 = 40 damage
        assert_eq!(state.current_hp, 60);

        match &emitter.0[0] {
            CombatEvent::DamageDealt { amount, .. } => {
                assert_eq!(*amount, 40);
            }
            _ => panic!("Expected DamageDealt event"),
        }
    }

    #[test]
    fn test_percentage_defense() {
        type PercentageCombat =
            CombatMechanic<LinearDamageCalculation, PercentageReduction, NoElemental>;

        let config = CombatConfig::default();
        let mut state = CombatState::new(100);

        // 50% damage reduction
        let input = CombatInput {
            attacker_power: 100,
            defender_defense: 50, // 50% reduction
            attacker_element: None,
            defender_element: None,
        };

        let mut emitter = VecEmitter(vec![]);
        PercentageCombat::step(&config, &mut state, input, &mut emitter);

        // 100 * (100 - 50) / 100 = 50 damage
        assert_eq!(state.current_hp, 50);

        match &emitter.0[0] {
            CombatEvent::DamageDealt { amount, .. } => {
                assert_eq!(*amount, 50);
            }
            _ => panic!("Expected DamageDealt event"),
        }
    }

    #[test]
    fn test_scaling_damage() {
        type ScalingCombat =
            CombatMechanic<ScalingDamageCalculation, SubtractiveDefense, NoElemental>;

        let config = CombatConfig::default();
        let mut state = CombatState::new(200);

        let input = CombatInput {
            attacker_power: 50,
            defender_defense: 0,
            attacker_element: None,
            defender_element: None,
        };

        let mut emitter = VecEmitter(vec![]);
        ScalingCombat::step(&config, &mut state, input, &mut emitter);

        // 50^1.2 ≈ 109
        assert_eq!(state.current_hp, 91); // 200 - 109 = 91

        match &emitter.0[0] {
            CombatEvent::DamageDealt { amount, .. } => {
                assert_eq!(*amount, 109);
            }
            _ => panic!("Expected DamageDealt event"),
        }
    }

    #[test]
    fn test_blocked_damage() {
        let config = CombatConfig {
            min_damage: 0, // Allow zero damage
            ..Default::default()
        };
        let mut state = CombatState::new(100);
        let input = CombatInput {
            attacker_power: 0,
            defender_defense: 0,
            attacker_element: None,
            defender_element: None,
        };

        let mut emitter = VecEmitter(vec![]);
        SimpleCombat::step(&config, &mut state, input, &mut emitter);

        // No damage, HP unchanged
        assert_eq!(state.current_hp, 100);

        // Blocked event emitted
        match &emitter.0[0] {
            CombatEvent::Blocked { attempted_damage } => {
                assert_eq!(*attempted_damage, 0); // 0 damage before blocking
            }
            _ => panic!("Expected Blocked event"),
        }
    }
}
