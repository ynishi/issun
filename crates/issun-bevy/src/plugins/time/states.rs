//! Turn phase state definitions (ADR 005)

use bevy::prelude::*;

/// Global turn phase state (ADR 005: Event-Driven Hybrid Turn Architecture)
///
/// Controls the high-level flow of turn-based gameplay.
/// Transitions are controlled by systems checking conditions
/// (e.g., "all actions consumed", "animations finished").
///
/// # Architecture
///
/// ```text
/// [PlayerInput] → [Processing] → [Visuals] → [EnemyTurn]
///        ↑                           |
///        └──────── (completed) ──────┘
/// ```
///
/// # Usage
///
/// ```ignore
/// use bevy::prelude::*;
/// use issun_bevy::plugins::time::TurnPhase;
///
/// // System runs only in PlayerInput phase
/// fn handle_input() { ... }
///
/// app.add_systems(Update, handle_input.run_if(in_state(TurnPhase::PlayerInput)));
/// ```
#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default, Reflect)]
#[reflect(opaque)]
pub enum TurnPhase {
    /// Waiting for player input
    ///
    /// Systems in this phase:
    /// - Handle keyboard/mouse input
    /// - Process UI interactions
    /// - Validate player commands
    #[default]
    PlayerInput,

    /// Processing logic (instant, atomic)
    ///
    /// Systems in this phase:
    /// - Calculate damage
    /// - Update inventory
    /// - Resolve effects
    /// - Emit result events
    ///
    /// **Critical**: Logic must complete instantly (microseconds).
    /// Never perform animations or wait for user input here.
    Processing,

    /// Playing visual effects (animations, UI updates)
    ///
    /// Systems in this phase:
    /// - Play animations
    /// - Update UI widgets
    /// - Display particle effects
    /// - Manage AnimationLock entities
    ///
    /// **Critical**: This phase waits for animations to complete
    /// before transitioning (via AnimationLock count).
    Visuals,

    /// Enemy AI turn
    ///
    /// Systems in this phase:
    /// - AI decision making
    /// - Enemy action execution
    /// - Pathfinding
    EnemyTurn,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_phase_default() {
        let phase = TurnPhase::default();
        assert_eq!(phase, TurnPhase::PlayerInput);
    }

    #[test]
    fn test_turn_phase_equality() {
        assert_eq!(TurnPhase::PlayerInput, TurnPhase::PlayerInput);
        assert_ne!(TurnPhase::PlayerInput, TurnPhase::Processing);
    }

    #[test]
    fn test_turn_phase_clone() {
        let phase1 = TurnPhase::Processing;
        let phase2 = phase1.clone();
        assert_eq!(phase1, phase2);
    }

    #[test]
    fn test_turn_phase_debug() {
        let phase = TurnPhase::Visuals;
        let debug_str = format!("{:?}", phase);
        assert!(debug_str.contains("Visuals"));
    }
}
