//! Configuration for CulturePlugin

use serde::{Deserialize, Serialize};

/// Culture plugin configuration (Resource, ReadOnly)
///
/// This configuration controls culture effects, stress accumulation, and alignment thresholds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CultureConfig {
    /// Base stress accumulation rate for misaligned members (0.0-1.0)
    ///
    /// This represents how much stress increases per turn when personality and culture mismatch.
    /// - 0.01 = 1% stress accumulation per turn
    /// - 0.05 = 5% stress accumulation per turn
    pub base_stress_rate: f32,

    /// Base fervor growth rate for aligned members (0.0-1.0)
    ///
    /// This represents how much fervor increases per turn when well-aligned with culture.
    /// - 0.02 = 2% fervor growth per turn
    /// - 0.05 = 5% fervor growth per turn
    pub base_fervor_growth_rate: f32,

    /// Stress threshold for member breakdown (0.0-1.0)
    ///
    /// When stress exceeds this threshold, members may:
    /// - Quit the organization
    /// - Act erratically
    /// - Suffer mental breakdown
    pub stress_breakdown_threshold: f32,

    /// Fervor threshold for fanaticism (0.0-1.0)
    ///
    /// When fervor exceeds this threshold, members become fanatical and may:
    /// - Ignore self-preservation
    /// - Execute orders without question
    /// - Spread the culture aggressively
    pub fervor_fanaticism_threshold: f32,

    /// Culture strength multiplier (0.0-2.0)
    ///
    /// Controls how strongly culture tags affect member behavior.
    /// - 0.5 = Weak culture (subtle effects)
    /// - 1.0 = Normal culture
    /// - 2.0 = Strong culture (powerful effects)
    pub culture_strength: f32,

    /// Enable natural stress decay over time
    ///
    /// If true, members gradually recover from stress when conditions improve.
    pub enable_stress_decay: bool,

    /// Stress decay rate per turn (0.0-1.0)
    ///
    /// Only applies if enable_stress_decay is true.
    pub stress_decay_rate: f32,
}

impl crate::resources::Resource for CultureConfig {}

impl Default for CultureConfig {
    fn default() -> Self {
        Self {
            base_stress_rate: 0.03,           // 3% per turn
            base_fervor_growth_rate: 0.02,    // 2% per turn
            stress_breakdown_threshold: 0.8,  // 80% stress threshold
            fervor_fanaticism_threshold: 0.9, // 90% fervor threshold
            culture_strength: 1.0,            // Normal strength
            enable_stress_decay: true,
            stress_decay_rate: 0.01, // 1% decay per turn
        }
    }
}

impl CultureConfig {
    /// Create a new configuration with custom values
    pub fn new(
        stress_rate: f32,
        fervor_rate: f32,
        stress_threshold: f32,
        fervor_threshold: f32,
        culture_strength: f32,
    ) -> Self {
        Self {
            base_stress_rate: stress_rate.clamp(0.0, 1.0),
            base_fervor_growth_rate: fervor_rate.clamp(0.0, 1.0),
            stress_breakdown_threshold: stress_threshold.clamp(0.0, 1.0),
            fervor_fanaticism_threshold: fervor_threshold.clamp(0.0, 1.0),
            culture_strength: culture_strength.clamp(0.0, 2.0),
            enable_stress_decay: true,
            stress_decay_rate: 0.01,
        }
    }

    /// Builder: Set base stress rate
    pub fn with_stress_rate(mut self, rate: f32) -> Self {
        self.base_stress_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set base fervor growth rate
    pub fn with_fervor_rate(mut self, rate: f32) -> Self {
        self.base_fervor_growth_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set stress breakdown threshold
    pub fn with_stress_threshold(mut self, threshold: f32) -> Self {
        self.stress_breakdown_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set fervor fanaticism threshold
    pub fn with_fervor_threshold(mut self, threshold: f32) -> Self {
        self.fervor_fanaticism_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set culture strength multiplier
    pub fn with_culture_strength(mut self, strength: f32) -> Self {
        self.culture_strength = strength.clamp(0.0, 2.0);
        self
    }

    /// Builder: Enable/disable stress decay
    pub fn with_stress_decay(mut self, enable: bool) -> Self {
        self.enable_stress_decay = enable;
        self
    }

    /// Builder: Set stress decay rate
    pub fn with_stress_decay_rate(mut self, rate: f32) -> Self {
        self.stress_decay_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Validate configuration
    ///
    /// Returns true if all values are within valid ranges
    pub fn is_valid(&self) -> bool {
        self.base_stress_rate >= 0.0
            && self.base_stress_rate <= 1.0
            && self.base_fervor_growth_rate >= 0.0
            && self.base_fervor_growth_rate <= 1.0
            && self.stress_breakdown_threshold >= 0.0
            && self.stress_breakdown_threshold <= 1.0
            && self.fervor_fanaticism_threshold >= 0.0
            && self.fervor_fanaticism_threshold <= 1.0
            && self.culture_strength >= 0.0
            && self.culture_strength <= 2.0
            && self.stress_decay_rate >= 0.0
            && self.stress_decay_rate <= 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CultureConfig::default();

        assert_eq!(config.base_stress_rate, 0.03);
        assert_eq!(config.base_fervor_growth_rate, 0.02);
        assert_eq!(config.stress_breakdown_threshold, 0.8);
        assert_eq!(config.fervor_fanaticism_threshold, 0.9);
        assert_eq!(config.culture_strength, 1.0);
        assert!(config.enable_stress_decay);
        assert_eq!(config.stress_decay_rate, 0.01);
        assert!(config.is_valid());
    }

    #[test]
    fn test_custom_config() {
        let config = CultureConfig::new(0.05, 0.03, 0.7, 0.85, 1.5);

        assert_eq!(config.base_stress_rate, 0.05);
        assert_eq!(config.base_fervor_growth_rate, 0.03);
        assert_eq!(config.stress_breakdown_threshold, 0.7);
        assert_eq!(config.fervor_fanaticism_threshold, 0.85);
        assert_eq!(config.culture_strength, 1.5);
        assert!(config.is_valid());
    }

    #[test]
    fn test_builder_pattern() {
        let config = CultureConfig::default()
            .with_stress_rate(0.04)
            .with_fervor_rate(0.025)
            .with_stress_threshold(0.75)
            .with_fervor_threshold(0.88)
            .with_culture_strength(1.2)
            .with_stress_decay(false)
            .with_stress_decay_rate(0.02);

        assert_eq!(config.base_stress_rate, 0.04);
        assert_eq!(config.base_fervor_growth_rate, 0.025);
        assert_eq!(config.stress_breakdown_threshold, 0.75);
        assert_eq!(config.fervor_fanaticism_threshold, 0.88);
        assert_eq!(config.culture_strength, 1.2);
        assert!(!config.enable_stress_decay);
        assert_eq!(config.stress_decay_rate, 0.02);
    }

    #[test]
    fn test_clamping() {
        let config = CultureConfig::new(1.5, -0.2, 2.0, -1.0, 3.0);

        assert_eq!(config.base_stress_rate, 1.0); // Clamped to max
        assert_eq!(config.base_fervor_growth_rate, 0.0); // Clamped to min
        assert_eq!(config.stress_breakdown_threshold, 1.0); // Clamped to max
        assert_eq!(config.fervor_fanaticism_threshold, 0.0); // Clamped to min
        assert_eq!(config.culture_strength, 2.0); // Clamped to max
        assert!(config.is_valid());
    }

    #[test]
    fn test_serialization() {
        let config = CultureConfig::default();

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: CultureConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_extreme_weak_culture() {
        let config = CultureConfig::default().with_culture_strength(0.1);

        assert_eq!(config.culture_strength, 0.1);
        assert!(config.is_valid());
    }

    #[test]
    fn test_extreme_strong_culture() {
        let config = CultureConfig::default().with_culture_strength(2.0);

        assert_eq!(config.culture_strength, 2.0);
        assert!(config.is_valid());
    }

    #[test]
    fn test_is_valid_with_invalid_values() {
        // Manually create invalid config (bypassing constructor clamping)
        let config = CultureConfig {
            base_stress_rate: 1.5, // Invalid
            ..Default::default()
        };

        assert!(!config.is_valid());
    }

    #[test]
    fn test_stress_decay_disabled() {
        let config = CultureConfig::default().with_stress_decay(false);

        assert!(!config.enable_stress_decay);
        // Decay rate should still be valid even when disabled
        assert!(config.is_valid());
    }
}
