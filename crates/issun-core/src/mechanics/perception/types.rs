//! Type definitions for perception mechanic.
//!
//! Separates "Ground Truth" (absolute reality) from "Perceived Reality"
//! (what an observer believes to be true).

use std::collections::HashMap;

/// Unique identifier for observers (entities that perceive)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObserverId(pub String);

impl From<&str> for ObserverId {
    fn from(s: &str) -> Self {
        ObserverId(s.to_string())
    }
}

impl From<String> for ObserverId {
    fn from(s: String) -> Self {
        ObserverId(s)
    }
}

/// Unique identifier for targets (entities being observed)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetId(pub String);

impl From<&str> for TargetId {
    fn from(s: &str) -> Self {
        TargetId(s.to_string())
    }
}

impl From<String> for TargetId {
    fn from(s: String) -> Self {
        TargetId(s)
    }
}

/// Unique identifier for facts
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactId(pub String);

impl From<&str> for FactId {
    fn from(s: &str) -> Self {
        FactId(s.to_string())
    }
}

impl From<String> for FactId {
    fn from(s: String) -> Self {
        FactId(s)
    }
}

// ============================================================================
// Ground Truth Types
// ============================================================================

/// Ground truth value - the absolute reality (God's view)
///
/// This represents what actually exists in the game world,
/// independent of any observer's perception.
#[derive(Debug, Clone, PartialEq)]
pub enum GroundTruth {
    /// Numeric quantity (e.g., troop count, resource amount)
    Quantity { value: i64 },

    /// Floating-point value (e.g., health, price)
    Scalar { value: f32 },

    /// Position in 2D space
    Position { x: f32, y: f32 },

    /// Position in 3D space
    Position3D { x: f32, y: f32, z: f32 },

    /// Boolean presence (e.g., does this unit exist?)
    Presence { exists: bool },

    /// Categorical state (e.g., faction allegiance, unit type)
    Category { value: String },

    /// Composite fact with multiple fields
    Composite {
        fields: HashMap<String, GroundTruth>,
    },
}

impl GroundTruth {
    /// Create a quantity ground truth
    pub fn quantity(value: i64) -> Self {
        GroundTruth::Quantity { value }
    }

    /// Create a scalar ground truth
    pub fn scalar(value: f32) -> Self {
        GroundTruth::Scalar { value }
    }

    /// Create a 2D position ground truth
    pub fn position(x: f32, y: f32) -> Self {
        GroundTruth::Position { x, y }
    }

    /// Create a 3D position ground truth
    pub fn position_3d(x: f32, y: f32, z: f32) -> Self {
        GroundTruth::Position3D { x, y, z }
    }

    /// Create a presence ground truth
    pub fn presence(exists: bool) -> Self {
        GroundTruth::Presence { exists }
    }

    /// Create a category ground truth
    pub fn category(value: impl Into<String>) -> Self {
        GroundTruth::Category {
            value: value.into(),
        }
    }
}

// ============================================================================
// Perception Types
// ============================================================================

/// Perceived value - what an observer believes to be true
///
/// May differ from ground truth due to:
/// - Limited accuracy (noise)
/// - Information delay
/// - Confidence decay over time
#[derive(Debug, Clone, PartialEq)]
pub struct Perception {
    /// The perceived value (may have noise applied)
    pub value: GroundTruth,

    /// Accuracy of this perception (0.0-1.0)
    /// - 1.0 = perfect information
    /// - 0.0 = completely unreliable
    pub accuracy: f32,

    /// Confidence in this perception (0.0-1.0)
    /// Decays over time as information becomes stale
    pub confidence: f32,

    /// Tick when this observation was made
    pub observed_at: u64,

    /// Delay in ticks from when event occurred to when observed
    pub delay: u64,
}

impl Perception {
    /// Create a new perception
    pub fn new(value: GroundTruth, accuracy: f32, confidence: f32, observed_at: u64) -> Self {
        Self {
            value,
            accuracy: accuracy.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            observed_at,
            delay: 0,
        }
    }

    /// Create a perception with delay
    pub fn with_delay(mut self, delay: u64) -> Self {
        self.delay = delay;
        self
    }

    /// Check if perception is stale (confidence below threshold)
    pub fn is_stale(&self, min_confidence: f32) -> bool {
        self.confidence < min_confidence
    }
}

// ============================================================================
// Observer Capabilities
// ============================================================================

/// Statistics about an observer's perception capabilities
#[derive(Debug, Clone)]
pub struct ObserverStats {
    /// Entity ID of the observer
    pub entity_id: ObserverId,

    /// Base perception capability (0.0-1.0)
    /// Higher = better at gathering accurate information
    pub capability: f32,

    /// Perception range (distance units)
    pub range: f32,

    /// Technology/equipment bonus (multiplier, 1.0 = neutral)
    pub tech_bonus: f32,

    /// Special perception traits
    pub traits: Vec<ObserverTrait>,
}

impl Default for ObserverStats {
    fn default() -> Self {
        Self {
            entity_id: ObserverId("default".into()),
            capability: 0.5,
            range: 100.0,
            tech_bonus: 1.0,
            traits: Vec::new(),
        }
    }
}

/// Special traits that affect perception
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObserverTrait {
    /// Enhanced vision (better accuracy at distance)
    FarSight,

    /// Spy network (high accuracy for specific targets)
    SpyNetwork,

    /// Radar/sensor technology (detects presence easily)
    Sensors,

    /// Night vision (no accuracy penalty in darkness)
    NightVision,

    /// Psychic ability (can sense hidden units)
    Psychic,

    /// Paranoid (overestimates threats)
    Paranoid,

    /// Optimistic (underestimates threats)
    Optimistic,
}

// ============================================================================
// Target Concealment
// ============================================================================

/// Statistics about a target's concealment
#[derive(Debug, Clone)]
pub struct TargetStats {
    /// Entity ID of the target
    pub entity_id: TargetId,

    /// Base concealment level (0.0-1.0)
    /// Higher = harder to perceive accurately
    pub concealment: f32,

    /// Stealth technology bonus (multiplier, 1.0 = neutral)
    pub stealth_bonus: f32,

    /// Environmental concealment (terrain, weather, etc.)
    pub environmental_bonus: f32,

    /// Special concealment traits
    pub traits: Vec<TargetTrait>,
}

impl Default for TargetStats {
    fn default() -> Self {
        Self {
            entity_id: TargetId("default".into()),
            concealment: 0.0,
            stealth_bonus: 1.0,
            environmental_bonus: 1.0,
            traits: Vec::new(),
        }
    }
}

/// Special traits that affect concealment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetTrait {
    /// Camouflaged (harder to spot)
    Camouflage,

    /// Cloaked (invisible to normal sensors)
    Cloaked,

    /// Decoy (may appear as something else)
    Decoy,

    /// Jammer (interferes with sensors)
    Jammer,

    /// Loud (easier to detect)
    Loud,

    /// Glowing (easier to spot visually)
    Glowing,
}

// ============================================================================
// Config
// ============================================================================

/// Configuration parameters for perception mechanic
#[derive(Debug, Clone)]
pub struct PerceptionConfig {
    /// Base accuracy before modifiers (0.0-1.0)
    pub base_accuracy: f32,

    /// Distance penalty factor (accuracy decreases with distance)
    /// accuracy_penalty = distance * distance_penalty_factor
    pub distance_penalty_factor: f32,

    /// Confidence decay rate per tick (0.0-1.0)
    /// confidence = initial * (1 - decay_rate)^elapsed
    pub confidence_decay_rate: f32,

    /// Minimum confidence threshold
    /// Below this, perception is considered stale and unreliable
    pub min_confidence: f32,

    /// Maximum information delay (in ticks)
    pub max_delay: u64,

    /// Noise amplitude multiplier
    /// Higher = more noise at low accuracy
    pub noise_amplitude: f32,

    /// Minimum accuracy floor (even worst case has some chance)
    pub min_accuracy: f32,
}

impl Default for PerceptionConfig {
    fn default() -> Self {
        Self {
            base_accuracy: 0.7,
            distance_penalty_factor: 0.005,
            confidence_decay_rate: 0.05,
            min_confidence: 0.1,
            max_delay: 10,
            noise_amplitude: 0.3,
            min_accuracy: 0.1,
        }
    }
}

// ============================================================================
// Input
// ============================================================================

/// Input snapshot for perception mechanic
#[derive(Debug, Clone)]
pub struct PerceptionInput {
    /// The ground truth being observed
    pub ground_truth: GroundTruth,

    /// Fact identifier (for tracking in knowledge board)
    pub fact_id: FactId,

    /// Observer statistics
    pub observer: ObserverStats,

    /// Target statistics
    pub target: TargetStats,

    /// Distance between observer and target
    pub distance: f32,

    /// Random value for noise generation (0.0-1.0)
    pub rng: f32,

    /// Current tick
    pub current_tick: u64,
}

// ============================================================================
// State
// ============================================================================

/// State output from perception mechanic
#[derive(Debug, Clone, Default)]
pub struct PerceptionState {
    /// Calculated accuracy for this observation
    pub accuracy: f32,

    /// Current confidence level
    pub confidence: f32,

    /// The resulting perception
    pub perception: Option<Perception>,

    /// Knowledge board: accumulated perceptions indexed by fact_id
    pub knowledge: HashMap<FactId, Perception>,

    /// Last update tick
    pub last_update: u64,
}

impl PerceptionState {
    /// Get a perception from knowledge board
    pub fn get_perception(&self, fact_id: &FactId) -> Option<&Perception> {
        self.knowledge.get(fact_id)
    }

    /// Get all perceptions above confidence threshold
    pub fn confident_perceptions(&self, min_confidence: f32) -> Vec<(&FactId, &Perception)> {
        self.knowledge
            .iter()
            .filter(|(_, p)| p.confidence >= min_confidence)
            .collect()
    }

    /// Remove stale perceptions
    pub fn prune_stale(&mut self, min_confidence: f32) {
        self.knowledge.retain(|_, p| p.confidence >= min_confidence);
    }
}

// ============================================================================
// Events
// ============================================================================

/// Events emitted by perception mechanic
#[derive(Debug, Clone, PartialEq)]
pub enum PerceptionEvent {
    /// New observation made
    ObservationMade {
        /// Observer who made the observation
        observer: ObserverId,
        /// Target that was observed
        target: TargetId,
        /// Fact that was observed
        fact_id: FactId,
        /// Calculated accuracy
        accuracy: f32,
        /// Information delay
        delay: u64,
    },

    /// Perception updated with new information
    PerceptionUpdated {
        /// Fact that was updated
        fact_id: FactId,
        /// Old accuracy
        old_accuracy: f32,
        /// New accuracy
        new_accuracy: f32,
        /// New confidence
        confidence: f32,
    },

    /// Confidence decayed significantly
    ConfidenceDecayed {
        /// Fact with decaying confidence
        fact_id: FactId,
        /// Old confidence
        old_confidence: f32,
        /// New confidence
        new_confidence: f32,
    },

    /// Perception became stale (below min_confidence)
    PerceptionStale {
        /// Fact that became stale
        fact_id: FactId,
        /// Final confidence before pruning
        final_confidence: f32,
    },

    /// Perfect observation (ground truth revealed)
    TruthRevealed {
        /// Observer who gained perfect information
        observer: ObserverId,
        /// Fact that was revealed
        fact_id: FactId,
    },

    /// Detection failed (target not perceived)
    DetectionFailed {
        /// Observer who failed
        observer: ObserverId,
        /// Target that evaded detection
        target: TargetId,
        /// Reason for failure
        reason: DetectionFailureReason,
    },
}

/// Reason for detection failure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionFailureReason {
    /// Target out of range
    OutOfRange,
    /// Target too well concealed
    TooConcealed,
    /// Observer capability too low
    InsufficientCapability,
    /// Environmental interference
    EnvironmentalInterference,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ground_truth_constructors() {
        let qty = GroundTruth::quantity(100);
        assert!(matches!(qty, GroundTruth::Quantity { value: 100 }));

        let scalar = GroundTruth::scalar(0.5);
        assert!(matches!(scalar, GroundTruth::Scalar { value } if (value - 0.5).abs() < 0.001));

        let pos = GroundTruth::position(10.0, 20.0);
        assert!(matches!(pos, GroundTruth::Position { x: 10.0, y: 20.0 }));

        let presence = GroundTruth::presence(true);
        assert!(matches!(presence, GroundTruth::Presence { exists: true }));
    }

    #[test]
    fn test_perception_creation() {
        let perception = Perception::new(GroundTruth::quantity(100), 0.8, 0.9, 50);

        assert_eq!(perception.accuracy, 0.8);
        assert_eq!(perception.confidence, 0.9);
        assert_eq!(perception.observed_at, 50);
        assert_eq!(perception.delay, 0);
    }

    #[test]
    fn test_perception_with_delay() {
        let perception = Perception::new(GroundTruth::quantity(100), 0.8, 0.9, 50).with_delay(5);

        assert_eq!(perception.delay, 5);
    }

    #[test]
    fn test_perception_stale_check() {
        let perception = Perception::new(GroundTruth::quantity(100), 0.8, 0.15, 50);

        assert!(!perception.is_stale(0.1));
        assert!(perception.is_stale(0.2));
    }

    #[test]
    fn test_perception_clamping() {
        let perception = Perception::new(GroundTruth::quantity(100), 1.5, -0.5, 50);

        assert_eq!(perception.accuracy, 1.0);
        assert_eq!(perception.confidence, 0.0);
    }

    #[test]
    fn test_perception_config_default() {
        let config = PerceptionConfig::default();

        assert_eq!(config.base_accuracy, 0.7);
        assert_eq!(config.confidence_decay_rate, 0.05);
        assert_eq!(config.min_confidence, 0.1);
    }

    #[test]
    fn test_perception_state_knowledge_operations() {
        let mut state = PerceptionState::default();

        let perception = Perception::new(GroundTruth::quantity(100), 0.8, 0.9, 50);
        state
            .knowledge
            .insert(FactId("fact_001".into()), perception);

        assert!(state.get_perception(&FactId("fact_001".into())).is_some());
        assert!(state.get_perception(&FactId("fact_002".into())).is_none());

        let confident = state.confident_perceptions(0.5);
        assert_eq!(confident.len(), 1);
    }

    #[test]
    fn test_perception_state_prune_stale() {
        let mut state = PerceptionState::default();

        state.knowledge.insert(
            FactId("fresh".into()),
            Perception::new(GroundTruth::quantity(100), 0.8, 0.9, 50),
        );
        state.knowledge.insert(
            FactId("stale".into()),
            Perception::new(GroundTruth::quantity(50), 0.5, 0.05, 10),
        );

        assert_eq!(state.knowledge.len(), 2);

        state.prune_stale(0.1);

        assert_eq!(state.knowledge.len(), 1);
        assert!(state.get_perception(&FactId("fresh".into())).is_some());
        assert!(state.get_perception(&FactId("stale".into())).is_none());
    }

    #[test]
    fn test_id_from_string() {
        let observer: ObserverId = "player_1".into();
        assert_eq!(observer.0, "player_1");

        let target: TargetId = "enemy_unit".into();
        assert_eq!(target.0, "enemy_unit");

        let fact: FactId = "fact_001".into();
        assert_eq!(fact.0, "fact_001");
    }
}
