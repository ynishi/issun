//! Preset type aliases for common contagion mechanics.
//!
//! This module provides convenient type aliases that combine commonly-used
//! strategies into ready-to-use mechanics. These presets demonstrate best
//! practices and serve as starting points for custom implementations.

use super::mechanic::ContagionMechanic;
use super::strategies::{ExponentialSpread, LinearProgression, LinearSpread, ThresholdProgression};

// ============================================================================
// Basic Presets (using default generics)
// ============================================================================

/// Simple virus with default behavior.
///
/// - Spread: `LinearSpread` (proportional to density)
/// - Progression: `ThresholdProgression<10>` (resistance threshold: 10)
///
/// # Use Cases
///
/// - Tutorial or early-game mechanics
/// - Simple contact-based diseases
/// - Testing and baseline comparisons
///
/// # Example
///
/// ```
/// use issun_core::mechanics::contagion::presets::SimpleVirus;
/// use issun_core::mechanics::contagion::{ContagionConfig, SimpleSeverity, ContagionInput};
/// use issun_core::mechanics::{Mechanic, EventEmitter};
///
/// # struct TestEmitter;
/// # impl EventEmitter<issun_core::mechanics::contagion::ContagionEvent> for TestEmitter {
/// #     fn emit(&mut self, _event: issun_core::mechanics::contagion::ContagionEvent) {}
/// # }
/// let config = ContagionConfig { base_rate: 0.1 };
/// let mut state = SimpleSeverity::default();
/// let input = ContagionInput { density: 0.5, resistance: 5, rng: 0.03 };
/// let mut emitter = TestEmitter;
///
/// SimpleVirus::step(&config, &mut state, input, &mut emitter);
/// ```
pub type SimpleVirus = ContagionMechanic;

// ============================================================================
// Spread-focused Presets
// ============================================================================

/// Explosive pandemic-style virus.
///
/// - Spread: `ExponentialSpread` (density-squared scaling)
/// - Progression: `ThresholdProgression<10>` (default resistance threshold)
///
/// # Characteristics
///
/// - Slow spread in isolated areas
/// - Explosive spread in crowded areas (tipping point behavior)
/// - Good for modeling highly contagious airborne diseases
///
/// # Use Cases
///
/// - Pandemic simulations
/// - Zombie virus outbreaks
/// - Airborne diseases (COVID-19, measles)
///
/// # Example
///
/// ```
/// use issun_core::mechanics::contagion::presets::ExplosiveVirus;
/// use issun_core::mechanics::contagion::{ContagionConfig, SimpleSeverity, ContagionInput};
/// use issun_core::mechanics::{Mechanic, EventEmitter};
///
/// # struct TestEmitter;
/// # impl EventEmitter<issun_core::mechanics::contagion::ContagionEvent> for TestEmitter {
/// #     fn emit(&mut self, _event: issun_core::mechanics::contagion::ContagionEvent) {}
/// # }
/// let config = ContagionConfig { base_rate: 0.15 };
/// let mut state = SimpleSeverity::default();
/// // High density = exponentially higher spread rate
/// let input = ContagionInput { density: 0.8, resistance: 5, rng: 0.05 };
/// let mut emitter = TestEmitter;
///
/// ExplosiveVirus::step(&config, &mut state, input, &mut emitter);
/// ```
pub type ExplosiveVirus = ContagionMechanic<ExponentialSpread>;

/// Slow, steady virus with linear spread.
///
/// - Spread: `LinearSpread` (proportional to density)
/// - Progression: `LinearProgression<10>` (resistance threshold: 10)
///
/// # Characteristics
///
/// - Predictable spread rate
/// - Linear scaling with population density
/// - Uses `LinearProgression` (>= threshold blocks progression)
///
/// # Use Cases
///
/// - Contact-based diseases (flu, cold)
/// - When you want predictable, proportional behavior
/// - Early-game or tutorial mechanics
pub type SteadyVirus = ContagionMechanic<LinearSpread, LinearProgression>;

// ============================================================================
// Resistance-focused Presets
// ============================================================================

/// Virus that's hard to resist (low threshold).
///
/// - Spread: `LinearSpread` (proportional to density)
/// - Progression: `ThresholdProgression<5>` (low resistance threshold)
///
/// # Characteristics
///
/// - Only high resistance (>5) can block progression
/// - Most entities will be vulnerable
/// - Creates urgency to build resistance
///
/// # Use Cases
///
/// - Hard difficulty modes
/// - Late-game challenges
/// - Viruses designed to pressure players
pub type VirulentVirus = ContagionMechanic<LinearSpread, ThresholdProgression<5>>;

/// Virus that's easy to resist (high threshold).
///
/// - Spread: `LinearSpread` (proportional to density)
/// - Progression: `ThresholdProgression<20>` (high resistance threshold)
///
/// # Characteristics
///
/// - Even moderate resistance (>20) can block progression
/// - Most entities can resist with basic stats
/// - Good for early-game or tutorial content
///
/// # Use Cases
///
/// - Easy difficulty modes
/// - Tutorial mechanics
/// - Background flavor without serious threat
pub type WeakVirus = ContagionMechanic<LinearSpread, ThresholdProgression<20>>;

// ============================================================================
// Specialized Presets
// ============================================================================

/// Zombie virus: explosive spread + hard to resist.
///
/// - Spread: `ExponentialSpread` (pandemic-style)
/// - Progression: `ThresholdProgression<5>` (low threshold)
///
/// # Characteristics
///
/// - Spreads exponentially in crowded areas
/// - Very hard to resist (only resistance >5 works)
/// - Creates classic zombie outbreak scenarios
///
/// # Use Cases
///
/// - Zombie apocalypse games
/// - High-stakes infection scenarios
/// - When you want both fast spread AND hard-to-resist progression
///
/// # Example
///
/// ```
/// use issun_core::mechanics::contagion::presets::ZombieVirus;
/// use issun_core::mechanics::contagion::{ContagionConfig, SimpleSeverity, ContagionInput};
/// use issun_core::mechanics::{Mechanic, EventEmitter};
///
/// # struct TestEmitter;
/// # impl EventEmitter<issun_core::mechanics::contagion::ContagionEvent> for TestEmitter {
/// #     fn emit(&mut self, _event: issun_core::mechanics::contagion::ContagionEvent) {}
/// # }
/// let config = ContagionConfig { base_rate: 0.2 };
/// let mut state = SimpleSeverity::default();
/// // Dense crowd + low resistance = high infection chance
/// let input = ContagionInput { density: 0.9, resistance: 3, rng: 0.05 };
/// let mut emitter = TestEmitter;
///
/// ZombieVirus::step(&config, &mut state, input, &mut emitter);
/// // Likely to infect due to exponential spread and low resistance threshold
/// ```
pub type ZombieVirus = ContagionMechanic<ExponentialSpread, ThresholdProgression<5>>;

/// Plague: moderate spread but very persistent.
///
/// - Spread: `LinearSpread` (proportional to density)
/// - Progression: `LinearProgression<5>` (low threshold for progression)
///
/// # Characteristics
///
/// - Linear spread rate
/// - Very hard to stop progression once infected
/// - Encourages prevention over cure
///
/// # Use Cases
///
/// - Medieval plague simulations
/// - When you want infection to be serious and persistent
/// - Scenarios where prevention is key
pub type PlagueVirus = ContagionMechanic<LinearSpread, LinearProgression<5>>;
