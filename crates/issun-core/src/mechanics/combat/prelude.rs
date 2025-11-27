//! Convenient re-exports for the combat mechanic.
//!
//! This module provides a prelude that includes the most commonly used types
//! and preset combat styles for working with the combat system.
//!
//! # Usage
//!
//! ```
//! use issun_core::mechanics::combat::prelude::*;
//!
//! // Use preset combat styles:
//! type MyRPG = ClassicJRPG;
//! type MyStrategy = FireEmblemStyle;
//!
//! // Or compose your own:
//! type CustomCombat = CombatMechanic<Linear, Percentage, Affinity>;
//! ```

// Core mechanic
pub use super::mechanic::CombatMechanic;

// Types
pub use super::types::{CombatConfig, CombatEvent, CombatInput, CombatState, Element};

// Policies (for custom implementations)
pub use super::policies::{DamageCalculationPolicy, DefensePolicy, ElementalPolicy};

// Common strategies
pub use super::strategies::{
    ElementalAffinity, LinearDamageCalculation, NoElemental, PercentageReduction,
    ScalingDamageCalculation, SubtractiveDefense,
};

// ==================== Strategy Aliases ====================
// Short, convenient names for composing custom combat types

/// Linear damage calculation (attack power as-is)
pub use LinearDamageCalculation as Linear;

/// Scaling damage calculation (power^1.2)
pub use ScalingDamageCalculation as Scaling;

/// Subtractive defense (damage - defense)
pub use SubtractiveDefense as Subtractive;

/// Percentage-based defense reduction
pub use PercentageReduction as Percentage;

/// Elemental affinity system (Pokémon-style)
pub use ElementalAffinity as Affinity;

// ==================== RPG Presets ====================
// Common combat styles for RPG games

/// Classic JRPG combat (Dragon Quest, early Final Fantasy)
///
/// - Linear damage: Simple attack - defense
/// - Subtractive defense: Traditional damage reduction
/// - No elemental system
///
/// # Example
/// ```ignore
/// app.add_plugins(CombatPluginV2::<ClassicJRPG>::default());
/// ```
pub type ClassicJRPG = CombatMechanic<Linear, Subtractive, NoElemental>;

/// Elemental RPG combat (Pokémon, modern Final Fantasy)
///
/// - Linear damage: Straightforward calculations
/// - Subtractive defense: Traditional armor system
/// - Elemental affinity: Type matchups with 2x/0.5x multipliers
///
/// # Example
/// ```ignore
/// app.add_plugins(CombatPluginV2::<ElementalRPG>::default());
/// ```
pub type ElementalRPG = CombatMechanic<Linear, Subtractive, Affinity>;

/// Modern action RPG combat (Dark Souls, Monster Hunter)
///
/// - Scaling damage: Non-linear power growth (power^1.2)
/// - Percentage defense: Damage reduction by percentage
/// - No elemental system
///
/// # Example
/// ```ignore
/// app.add_plugins(CombatPluginV2::<ModernARPG>::default());
/// ```
pub type ModernARPG = CombatMechanic<Scaling, Percentage, NoElemental>;

/// Souls-like combat (Dark Souls, Elden Ring)
///
/// Alias for ModernARPG - same combat style
pub type SoulsLike = ModernARPG;

// ==================== Strategy SIM Presets ====================
// Common combat styles for tactical/strategy games

/// Fire Emblem style combat
///
/// - Linear damage: Clear, predictable calculations
/// - Subtractive defense: Armor reduces damage directly
/// - Elemental affinity: Weapon triangle system
///
/// # Example
/// ```ignore
/// app.add_plugins(CombatPluginV2::<FireEmblemStyle>::default());
/// ```
pub type FireEmblemStyle = CombatMechanic<Linear, Subtractive, Affinity>;

/// Advance Wars style combat
///
/// - Linear damage: Unit stats are straightforward
/// - Subtractive defense: Simple armor calculation
/// - No elemental system: Focus on unit types and terrain
///
/// # Example
/// ```ignore
/// app.add_plugins(CombatPluginV2::<AdvanceWarsStyle>::default());
/// ```
pub type AdvanceWarsStyle = CombatMechanic<Linear, Subtractive, NoElemental>;

/// XCOM style combat
///
/// - Scaling damage: Weapons scale non-linearly with tech
/// - Percentage defense: Armor absorbs percentage of damage
/// - No elemental system: Focus on cover and flanking
///
/// # Example
/// ```ignore
/// app.add_plugins(CombatPluginV2::<XCOMStyle>::default());
/// ```
pub type XCOMStyle = CombatMechanic<Scaling, Percentage, NoElemental>;

/// Super Robot Wars (SRW) style combat
///
/// - Scaling damage: Robot power scales dramatically
/// - Subtractive defense: Armor value reduces damage
/// - Elemental affinity: Weapon attributes (Beam, Physical, etc.)
///
/// # Example
/// ```ignore
/// app.add_plugins(CombatPluginV2::<SRWStyle>::default());
/// ```
pub type SRWStyle = CombatMechanic<Scaling, Subtractive, Affinity>;

/// Tactics Ogre / Final Fantasy Tactics style combat
///
/// - Linear damage: Job stats are straightforward
/// - Subtractive defense: Equipment-based defense
/// - Elemental affinity: Element-based spells and abilities
///
/// # Example
/// ```ignore
/// app.add_plugins(CombatPluginV2::<TacticsStyle>::default());
/// ```
pub type TacticsStyle = CombatMechanic<Linear, Subtractive, Affinity>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::{EventEmitter, Mechanic};

    // Mock event emitter for tests
    struct TestEmitter {
        events: Vec<CombatEvent>,
    }

    impl EventEmitter<CombatEvent> for TestEmitter {
        fn emit(&mut self, event: CombatEvent) {
            self.events.push(event);
        }
    }

    #[test]
    fn test_rpg_presets_compile() {
        // Just verify that all RPG presets can be instantiated
        let config = CombatConfig::default();
        let mut state = CombatState::new(100);
        let input = CombatInput {
            attacker_power: 50,
            defender_defense: 10,
            attacker_element: None,
            defender_element: None,
        };
        let mut emitter = TestEmitter { events: vec![] };

        // ClassicJRPG
        ClassicJRPG::step(&config, &mut state, input.clone(), &mut emitter);
        assert_eq!(state.current_hp, 60); // 100 - (50 - 10) = 60

        // ElementalRPG
        state = CombatState::new(100);
        ElementalRPG::step(&config, &mut state, input.clone(), &mut emitter);
        assert_eq!(state.current_hp, 60);

        // ModernARPG
        state = CombatState::new(100);
        ModernARPG::step(&config, &mut state, input.clone(), &mut emitter);
        // Scaling: 50^1.2 ≈ 109, Percentage: 109 - (109 * 10 / 100) = 99
        assert_eq!(state.current_hp, 1); // 100 - 99 = 1

        // SoulsLike (alias)
        state = CombatState::new(100);
        SoulsLike::step(&config, &mut state, input.clone(), &mut emitter);
        assert_eq!(state.current_hp, 1);
    }

    #[test]
    fn test_strategy_sim_presets_compile() {
        let config = CombatConfig::default();
        let mut state = CombatState::new(100);
        let input = CombatInput {
            attacker_power: 50,
            defender_defense: 10,
            attacker_element: Some(Element::Fire),
            defender_element: Some(Element::Ice),
        };
        let mut emitter = TestEmitter { events: vec![] };

        // FireEmblemStyle - with elemental affinity
        FireEmblemStyle::step(&config, &mut state, input.clone(), &mut emitter);
        // (50 - 10) * 2 (Fire vs Ice) = 80
        assert_eq!(state.current_hp, 20); // 100 - 80 = 20

        // AdvanceWarsStyle - no elemental
        state = CombatState::new(100);
        let input_no_element = CombatInput {
            attacker_element: None,
            defender_element: None,
            ..input
        };
        AdvanceWarsStyle::step(&config, &mut state, input_no_element.clone(), &mut emitter);
        assert_eq!(state.current_hp, 60); // 100 - (50 - 10) = 60

        // XCOMStyle
        state = CombatState::new(100);
        XCOMStyle::step(&config, &mut state, input_no_element.clone(), &mut emitter);
        assert_eq!(state.current_hp, 1); // Scaling + Percentage = 99 damage

        // SRWStyle
        state = CombatState::new(200); // Higher HP for robot combat
        SRWStyle::step(&config, &mut state, input.clone(), &mut emitter);
        // Scaling: 50^1.2 ≈ 109, Subtractive: 109 - 10 = 99, Affinity: 99 * 2 = 198
        assert_eq!(state.current_hp, 2); // 200 - 198 = 2

        // TacticsStyle
        state = CombatState::new(100);
        TacticsStyle::step(&config, &mut state, input.clone(), &mut emitter);
        assert_eq!(state.current_hp, 20); // Same as FireEmblem
    }

    #[test]
    fn test_strategy_aliases() {
        // Verify that aliases work correctly
        let config = CombatConfig::default();
        let mut state = CombatState::new(100);
        let input = CombatInput {
            attacker_power: 50,
            defender_defense: 10,
            attacker_element: None,
            defender_element: None,
        };
        let mut emitter = TestEmitter { events: vec![] };

        // Test custom composition with aliases
        type CustomStyle = CombatMechanic<Linear, Percentage, NoElemental>;
        CustomStyle::step(&config, &mut state, input, &mut emitter);
        // Linear: 50, Percentage: 50 - (50 * 10 / 100) = 45
        assert_eq!(state.current_hp, 55); // 100 - 45 = 55
    }

    #[test]
    fn test_preset_documentation_accuracy() {
        // Verify that documented behavior matches actual behavior
        let config = CombatConfig::default();

        // ClassicJRPG: "Simple attack - defense"
        let mut state = CombatState::new(100);
        let input = CombatInput {
            attacker_power: 30,
            defender_defense: 10,
            attacker_element: None,
            defender_element: None,
        };
        let mut emitter = TestEmitter { events: vec![] };
        ClassicJRPG::step(&config, &mut state, input, &mut emitter);
        assert_eq!(state.current_hp, 80); // 100 - (30 - 10) = 80 ✓

        // ElementalRPG: "Type matchups with 2x/0.5x multipliers"
        state = CombatState::new(100);
        let input_super_effective = CombatInput {
            attacker_power: 30,
            defender_defense: 10,
            attacker_element: Some(Element::Fire),
            defender_element: Some(Element::Ice),
        };
        ElementalRPG::step(&config, &mut state, input_super_effective, &mut emitter);
        // (30 - 10) * 2 = 40
        assert_eq!(state.current_hp, 60); // 100 - 40 = 60 ✓
    }
}
