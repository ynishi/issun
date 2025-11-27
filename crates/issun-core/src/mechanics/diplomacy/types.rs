//! Core types for the diplomacy system.

/// Configuration for diplomacy mechanics.
#[derive(Debug, Clone)]
pub struct DiplomacyConfig {
    /// Base difficulty multiplier (default: 1.0)
    pub difficulty: f32,
    /// Maximum patience turns before target leaves
    pub max_patience: u32,
    /// Agreement threshold (0-100) required for success
    pub agreement_threshold: f32,
}

impl Default for DiplomacyConfig {
    fn default() -> Self {
        Self {
            difficulty: 1.0,
            max_patience: 5,
            agreement_threshold: 100.0,
        }
    }
}

/// The mutable state of a diplomatic negotiation.
#[derive(Debug, Clone)]
pub struct DiplomacyState {
    /// Current progress towards agreement (0.0 to 100.0)
    pub agreement_progress: f32,
    /// Turns remaining before the target loses interest
    pub patience: u32,
    /// Current relationship score (-1.0 to 1.0)
    /// -1.0 = Hostile, 0.0 = Neutral, 1.0 = Friendly
    pub relationship_score: f32,
    /// Whether the negotiation has ended
    pub is_finished: bool,
}

impl DiplomacyState {
    /// Creates a new diplomacy state.
    ///
    /// # Arguments
    ///
    /// * `relationship_score` - Initial relationship score, clamped to [-1.0, 1.0]
    /// * `max_patience` - Maximum patience before negotiation fails
    pub fn new(relationship_score: f32, max_patience: u32) -> Self {
        Self {
            agreement_progress: 0.0,
            patience: max_patience,
            relationship_score: relationship_score.clamp(-1.0, 1.0),
            is_finished: false,
        }
    }
}

/// Input for a single diplomatic turn.
#[derive(Debug, Clone)]
pub struct DiplomacyInput {
    /// The strength of the argument or offer
    pub argument_strength: f32,
    /// The type of argument (e.g., Logic, Emotion, Aggression)
    /// Represented as an enum or tag in a real game, here simplified.
    pub argument_type: ArgumentType,
    /// The target's inherent resistance to this specific topic
    pub target_resistance: f32,
}

/// Type of diplomatic argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArgumentType {
    /// Logical reasoning-based argument.
    Logic,
    /// Emotional appeal-based argument.
    Emotion,
    /// Bribery or material incentive-based argument.
    Bribe,
    /// Intimidation or threat-based argument.
    Intimidation,
}

/// Events emitted during diplomacy.
#[derive(Debug, Clone, PartialEq)]
pub enum DiplomacyEvent {
    /// Progress was made towards agreement
    ProgressMade {
        /// Amount of progress made this turn
        amount: f32,
        /// Current total progress
        current: f32,
    },
    /// The argument was rejected or had no effect
    ArgumentRejected,
    /// Patience decreased
    PatienceLost {
        /// Remaining patience
        remaining: u32,
    },
    /// Negotiation succeeded (Agreement reached)
    AgreementReached,
    /// Negotiation failed (Patience ran out)
    NegotiationFailed,
}
