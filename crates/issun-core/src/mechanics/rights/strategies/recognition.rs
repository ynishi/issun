//! Recognition policy strategies.
//!
//! Provides concrete implementations of the RecognitionPolicy trait.

use crate::mechanics::rights::policies::RecognitionPolicy;
use crate::mechanics::rights::types::*;

/// Self-recognition strategy.
///
/// No external recognition required. Claims are valid by assertion alone.
/// Legitimacy remains constant at 100%.
///
/// # Use Cases
///
/// - Anarchist/stateless systems
/// - Self-sovereign identity
/// - Unregulated markets
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::strategies::SelfRecognition;
/// use issun_core::mechanics::rights::policies::RecognitionPolicy;
/// use issun_core::mechanics::rights::{RightsConfig, RightsState};
///
/// let config = RightsConfig::default();
/// let mut state = RightsState::new();
///
/// // No recognition required
/// assert!(!SelfRecognition::requires_recognition(&config));
///
/// // Legitimacy stays at 1.0
/// SelfRecognition::update_legitimacy(&mut state, 0, &config);
/// assert_eq!(state.legitimacy, 1.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelfRecognition;

impl RecognitionPolicy for SelfRecognition {
    fn requires_recognition(_config: &RightsConfig) -> bool {
        false
    }

    fn update_legitimacy(
        state: &mut RightsState,
        _recognition_count: usize,
        _config: &RightsConfig,
    ) {
        // Legitimacy always 1.0 (self-recognition)
        state.legitimacy = 1.0;
    }
}

/// Authority recognition strategy.
///
/// Claims require recognition from designated authorities.
/// Legitimacy based on whether recognized or not (binary).
///
/// # Use Cases
///
/// - State-recognized property rights
/// - Corporate governance (board approval)
/// - Centralized systems
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::strategies::AuthorityRecognition;
/// use issun_core::mechanics::rights::policies::RecognitionPolicy;
/// use issun_core::mechanics::rights::{RightsConfig, RightsState};
///
/// let config = RightsConfig {
///     require_recognition: true,
///     ..Default::default()
/// };
/// let mut state = RightsState::new();
///
/// // Recognition required
/// assert!(AuthorityRecognition::requires_recognition(&config));
///
/// // No recognition = low legitimacy
/// AuthorityRecognition::update_legitimacy(&mut state, 0, &config);
/// assert_eq!(state.legitimacy, 0.0);
///
/// // With recognition = full legitimacy
/// state.recognized_by.insert(1);
/// AuthorityRecognition::update_legitimacy(&mut state, 1, &config);
/// assert_eq!(state.legitimacy, 1.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthorityRecognition;

impl RecognitionPolicy for AuthorityRecognition {
    fn requires_recognition(config: &RightsConfig) -> bool {
        config.require_recognition
    }

    fn update_legitimacy(
        state: &mut RightsState,
        recognition_count: usize,
        config: &RightsConfig,
    ) {
        if config.require_recognition {
            // Binary: recognized or not
            state.legitimacy = if recognition_count > 0 { 1.0 } else { 0.0 };
        } else {
            state.legitimacy = 1.0;
        }
    }
}

/// Consensus recognition strategy.
///
/// Legitimacy scales with the number of entities recognizing claims.
/// More recognition = higher legitimacy.
///
/// # Use Cases
///
/// - DAO governance
/// - Peer-to-peer networks
/// - Democratic systems
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::strategies::ConsensusRecognition;
/// use issun_core::mechanics::rights::policies::RecognitionPolicy;
/// use issun_core::mechanics::rights::{RightsConfig, RightsState};
///
/// let config = RightsConfig::default();
/// let mut state = RightsState::new();
///
/// // Legitimacy scales with recognition count
/// ConsensusRecognition::update_legitimacy(&mut state, 0, &config);
/// assert_eq!(state.legitimacy, 0.0);
///
/// ConsensusRecognition::update_legitimacy(&mut state, 5, &config);
/// assert_eq!(state.legitimacy, 0.5); // 5 / (5 + 5) = 0.5
///
/// ConsensusRecognition::update_legitimacy(&mut state, 20, &config);
/// assert_eq!(state.legitimacy, 0.8); // 20 / (20 + 5) = 0.8
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsensusRecognition;

impl RecognitionPolicy for ConsensusRecognition {
    fn requires_recognition(_config: &RightsConfig) -> bool {
        false // Soft requirement (affects legitimacy but doesn't block)
    }

    fn update_legitimacy(
        state: &mut RightsState,
        recognition_count: usize,
        _config: &RightsConfig,
    ) {
        // Logarithmic scaling: legitimacy = count / (count + threshold)
        // This approaches 1.0 asymptotically as recognition increases
        const THRESHOLD: f32 = 5.0;

        if recognition_count == 0 {
            state.legitimacy = 0.0;
        } else {
            let count = recognition_count as f32;
            state.legitimacy = count / (count + THRESHOLD);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> RightsConfig {
        RightsConfig {
            allow_partial_claims: true,
            require_recognition: false,
            transfer_tax_rate: 0.0,
            legitimacy_decay_rate: 0.1,
        }
    }

    // SelfRecognition tests
    #[test]
    fn test_self_recognition_no_requirement() {
        let config = default_config();
        assert!(!SelfRecognition::requires_recognition(&config));
    }

    #[test]
    fn test_self_recognition_maintains_legitimacy() {
        let config = default_config();
        let mut state = RightsState::new();

        SelfRecognition::update_legitimacy(&mut state, 0, &config);
        assert_eq!(state.legitimacy, 1.0);

        SelfRecognition::update_legitimacy(&mut state, 100, &config);
        assert_eq!(state.legitimacy, 1.0);
    }

    #[test]
    fn test_self_recognition_no_decay() {
        let config = default_config();
        let mut state = RightsState::new();
        state.legitimacy = 1.0;

        // Default trait implementation should apply decay
        SelfRecognition::apply_decay(&mut state, 10, &config);
        // After 10 time units with 0.1 decay rate: 1.0 - (0.1 * 10) = 0.0
        assert_eq!(state.legitimacy, 0.0);
    }

    // AuthorityRecognition tests
    #[test]
    fn test_authority_recognition_requirement() {
        let config_required = RightsConfig {
            require_recognition: true,
            ..default_config()
        };
        assert!(AuthorityRecognition::requires_recognition(&config_required));

        let config_not_required = default_config();
        assert!(!AuthorityRecognition::requires_recognition(&config_not_required));
    }

    #[test]
    fn test_authority_recognition_binary_legitimacy() {
        let config = RightsConfig {
            require_recognition: true,
            ..default_config()
        };
        let mut state = RightsState::new();

        // Not recognized = 0 legitimacy
        AuthorityRecognition::update_legitimacy(&mut state, 0, &config);
        assert_eq!(state.legitimacy, 0.0);

        // Recognized = full legitimacy
        AuthorityRecognition::update_legitimacy(&mut state, 1, &config);
        assert_eq!(state.legitimacy, 1.0);

        // Multiple recognitions still = 1.0
        AuthorityRecognition::update_legitimacy(&mut state, 10, &config);
        assert_eq!(state.legitimacy, 1.0);
    }

    #[test]
    fn test_authority_recognition_without_requirement() {
        let config = default_config();
        let mut state = RightsState::new();

        // If recognition not required, legitimacy always 1.0
        AuthorityRecognition::update_legitimacy(&mut state, 0, &config);
        assert_eq!(state.legitimacy, 1.0);
    }

    // ConsensusRecognition tests
    #[test]
    fn test_consensus_recognition_no_hard_requirement() {
        let config = default_config();
        assert!(!ConsensusRecognition::requires_recognition(&config));
    }

    #[test]
    fn test_consensus_recognition_scaling() {
        let config = default_config();
        let mut state = RightsState::new();

        // Zero recognition
        ConsensusRecognition::update_legitimacy(&mut state, 0, &config);
        assert_eq!(state.legitimacy, 0.0);

        // Some recognition (5 / (5 + 5) = 0.5)
        ConsensusRecognition::update_legitimacy(&mut state, 5, &config);
        assert_eq!(state.legitimacy, 0.5);

        // More recognition (20 / (20 + 5) = 0.8)
        ConsensusRecognition::update_legitimacy(&mut state, 20, &config);
        assert_eq!(state.legitimacy, 0.8);

        // Lots of recognition (100 / (100 + 5) ≈ 0.95)
        ConsensusRecognition::update_legitimacy(&mut state, 100, &config);
        assert!((state.legitimacy - 0.952).abs() < 0.01);
    }

    #[test]
    fn test_consensus_recognition_asymptotic() {
        let config = default_config();
        let mut state = RightsState::new();

        // Approaches 1.0 but never quite reaches it
        ConsensusRecognition::update_legitimacy(&mut state, 10000, &config);
        assert!(state.legitimacy > 0.99);
        assert!(state.legitimacy < 1.0);
    }
}
