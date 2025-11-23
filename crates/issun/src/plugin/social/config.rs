//! Configuration for SocialPlugin

use serde::{Deserialize, Serialize};

/// Centrality calculation weights
///
/// These weights determine how each centrality metric contributes
/// to the overall influence score.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CentralityWeights {
    /// Weight for degree centrality (0.0-1.0)
    /// "How well-connected" matters
    pub degree: f32,

    /// Weight for betweenness centrality (0.0-1.0)
    /// "Information broker" role matters
    pub betweenness: f32,

    /// Weight for closeness centrality (0.0-1.0)
    /// "Information spread speed" matters
    pub closeness: f32,

    /// Weight for eigenvector centrality (0.0-1.0)
    /// "Connection to power" matters
    pub eigenvector: f32,
}

impl Default for CentralityWeights {
    fn default() -> Self {
        Self {
            degree: 0.3,
            betweenness: 0.3,
            closeness: 0.2,
            eigenvector: 0.2,
        }
    }
}

impl CentralityWeights {
    /// Validate that weights sum to approximately 1.0
    pub fn is_valid(&self) -> bool {
        let sum = self.degree + self.betweenness + self.closeness + self.eigenvector;
        (sum - 1.0).abs() < 0.01
            && self.degree >= 0.0
            && self.betweenness >= 0.0
            && self.closeness >= 0.0
            && self.eigenvector >= 0.0
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize(&mut self) {
        let sum = self.degree + self.betweenness + self.closeness + self.eigenvector;
        if sum > 0.0 {
            self.degree /= sum;
            self.betweenness /= sum;
            self.closeness /= sum;
            self.eigenvector /= sum;
        }
    }
}

/// Social plugin configuration (Resource, ReadOnly)
///
/// This configuration controls network analysis, influence propagation,
/// and faction dynamics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SocialConfig {
    /// Centrality recalculation interval (in turns)
    ///
    /// Centrality calculation is expensive, so it's done periodically.
    /// Lower values = more responsive but slower
    /// Higher values = faster but less accurate
    pub centrality_recalc_interval: u32,

    /// Shadow leader detection threshold (0.0-1.0)
    ///
    /// Members with overall_influence above this are considered "shadow leaders" (KingMakers).
    /// - 0.75 = Moderately influential required
    /// - 0.85 = Highly influential required
    pub shadow_leader_threshold: f32,

    /// Trust relationship natural decay rate (0.0-1.0 per turn)
    ///
    /// Without interaction, trust gradually decreases.
    /// - 0.01 = 1% decay per turn (slow)
    /// - 0.05 = 5% decay per turn (fast)
    pub trust_decay_rate: f32,

    /// Favor expiration time (in turns)
    ///
    /// How long favors remain valid before they "expire".
    /// After expiration, the debt is forgiven.
    pub favor_expiration_turns: u32,

    /// Faction cohesion natural decay rate (0.0-1.0 per turn)
    ///
    /// Without shared victories or events, faction cohesion decreases.
    pub faction_cohesion_decay_rate: f32,

    /// Centrality calculation weights
    pub centrality_weights: CentralityWeights,

    /// Gossip spread rate (0.0-1.0)
    ///
    /// Probability that gossip spreads to connected members each turn.
    /// - 0.3 = Slow spread
    /// - 0.7 = Fast spread
    pub gossip_spread_rate: f32,

    /// Maximum number of factions allowed
    ///
    /// Limits faction proliferation for performance.
    pub max_factions: usize,

    /// Minimum faction size (members)
    ///
    /// Factions with fewer members than this are automatically dissolved.
    pub min_faction_size: usize,

    /// Faction cohesion threshold for split (0.0-1.0)
    ///
    /// If cohesion falls below this, faction may split.
    pub faction_split_threshold: f32,

    /// Enable automatic faction merging
    ///
    /// If true, factions with similar agendas may merge automatically.
    pub enable_faction_merging: bool,
}

impl crate::resources::Resource for SocialConfig {}

impl Default for SocialConfig {
    fn default() -> Self {
        Self {
            centrality_recalc_interval: 5,      // Every 5 turns
            shadow_leader_threshold: 0.75,      // 75% influence
            trust_decay_rate: 0.01,             // 1% per turn
            favor_expiration_turns: 50,         // 50 turns lifetime
            faction_cohesion_decay_rate: 0.02,  // 2% per turn
            centrality_weights: CentralityWeights::default(),
            gossip_spread_rate: 0.3,            // 30% spread chance
            max_factions: 10,                   // Maximum 10 factions
            min_faction_size: 3,                // At least 3 members
            faction_split_threshold: 0.3,       // 30% cohesion triggers split
            enable_faction_merging: true,
        }
    }
}

impl SocialConfig {
    /// Create a new configuration with custom values
    pub fn new(
        recalc_interval: u32,
        shadow_threshold: f32,
        trust_decay: f32,
        favor_expiration: u32,
    ) -> Self {
        Self {
            centrality_recalc_interval: recalc_interval.max(1),
            shadow_leader_threshold: shadow_threshold.clamp(0.0, 1.0),
            trust_decay_rate: trust_decay.clamp(0.0, 1.0),
            favor_expiration_turns: favor_expiration,
            ..Default::default()
        }
    }

    /// Builder: Set centrality recalculation interval
    pub fn with_recalc_interval(mut self, interval: u32) -> Self {
        self.centrality_recalc_interval = interval.max(1);
        self
    }

    /// Builder: Set shadow leader threshold
    pub fn with_shadow_threshold(mut self, threshold: f32) -> Self {
        self.shadow_leader_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set trust decay rate
    pub fn with_trust_decay(mut self, rate: f32) -> Self {
        self.trust_decay_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set favor expiration time
    pub fn with_favor_expiration(mut self, turns: u32) -> Self {
        self.favor_expiration_turns = turns;
        self
    }

    /// Builder: Set faction cohesion decay rate
    pub fn with_faction_decay(mut self, rate: f32) -> Self {
        self.faction_cohesion_decay_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set centrality weights
    pub fn with_centrality_weights(mut self, weights: CentralityWeights) -> Self {
        self.centrality_weights = weights;
        self
    }

    /// Builder: Set gossip spread rate
    pub fn with_gossip_spread_rate(mut self, rate: f32) -> Self {
        self.gossip_spread_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set maximum factions
    pub fn with_max_factions(mut self, max: usize) -> Self {
        self.max_factions = max;
        self
    }

    /// Builder: Set minimum faction size
    pub fn with_min_faction_size(mut self, min: usize) -> Self {
        self.min_faction_size = min.max(2);
        self
    }

    /// Builder: Set faction split threshold
    pub fn with_faction_split_threshold(mut self, threshold: f32) -> Self {
        self.faction_split_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Builder: Enable/disable faction merging
    pub fn with_faction_merging(mut self, enable: bool) -> Self {
        self.enable_faction_merging = enable;
        self
    }

    /// Validate configuration
    ///
    /// Returns true if all values are within valid ranges
    pub fn is_valid(&self) -> bool {
        self.centrality_recalc_interval > 0
            && self.shadow_leader_threshold >= 0.0
            && self.shadow_leader_threshold <= 1.0
            && self.trust_decay_rate >= 0.0
            && self.trust_decay_rate <= 1.0
            && self.faction_cohesion_decay_rate >= 0.0
            && self.faction_cohesion_decay_rate <= 1.0
            && self.gossip_spread_rate >= 0.0
            && self.gossip_spread_rate <= 1.0
            && self.min_faction_size >= 2
            && self.faction_split_threshold >= 0.0
            && self.faction_split_threshold <= 1.0
            && self.centrality_weights.is_valid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centrality_weights_default() {
        let weights = CentralityWeights::default();
        assert_eq!(weights.degree, 0.3);
        assert_eq!(weights.betweenness, 0.3);
        assert_eq!(weights.closeness, 0.2);
        assert_eq!(weights.eigenvector, 0.2);
        assert!(weights.is_valid());
    }

    #[test]
    fn test_centrality_weights_normalize() {
        let mut weights = CentralityWeights {
            degree: 1.0,
            betweenness: 1.0,
            closeness: 1.0,
            eigenvector: 1.0,
        };

        weights.normalize();
        assert!((weights.degree - 0.25).abs() < 0.01);
        assert!((weights.betweenness - 0.25).abs() < 0.01);
        assert!(weights.is_valid());
    }

    #[test]
    fn test_social_config_default() {
        let config = SocialConfig::default();
        assert_eq!(config.centrality_recalc_interval, 5);
        assert_eq!(config.shadow_leader_threshold, 0.75);
        assert_eq!(config.max_factions, 10);
        assert!(config.is_valid());
    }

    #[test]
    fn test_social_config_builder() {
        let config = SocialConfig::default()
            .with_recalc_interval(10)
            .with_shadow_threshold(0.85)
            .with_trust_decay(0.05)
            .with_max_factions(20);

        assert_eq!(config.centrality_recalc_interval, 10);
        assert_eq!(config.shadow_leader_threshold, 0.85);
        assert_eq!(config.trust_decay_rate, 0.05);
        assert_eq!(config.max_factions, 20);
        assert!(config.is_valid());
    }

    #[test]
    fn test_social_config_clamping() {
        let config = SocialConfig::default()
            .with_shadow_threshold(2.0) // Should clamp to 1.0
            .with_trust_decay(-0.5) // Should clamp to 0.0
            .with_recalc_interval(0); // Should clamp to 1

        assert_eq!(config.shadow_leader_threshold, 1.0);
        assert_eq!(config.trust_decay_rate, 0.0);
        assert_eq!(config.centrality_recalc_interval, 1);
        assert!(config.is_valid());
    }

    #[test]
    fn test_social_config_validation() {
        let valid = SocialConfig::default();
        assert!(valid.is_valid());

        let mut invalid = SocialConfig::default();
        invalid.min_faction_size = 1; // Too small
        assert!(!invalid.is_valid());
    }
}
