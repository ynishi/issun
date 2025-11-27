use crate::mechanics::diplomacy::policies::ResistancePolicy;
use crate::mechanics::diplomacy::types::DiplomacyConfig;

/// Resistance based on skepticism (relationship score).
///
/// Lower relationship score increases effective resistance.
/// - Friendly (1.0): Resistance is halved.
/// - Neutral (0.0): Resistance is normal.
/// - Hostile (-1.0): Resistance is doubled.
pub struct SkepticalResistance;

impl ResistancePolicy for SkepticalResistance {
    fn apply_resistance(
        influence: f32,
        base_resistance: f32,
        relationship: f32,
        config: &DiplomacyConfig,
    ) -> f32 {
        // Map relationship (-1.0 to 1.0) to multiplier (2.0 to 0.5)
        // -1.0 -> 2.0
        //  0.0 -> 1.0
        //  1.0 -> 0.5
        let multiplier = if relationship < 0.0 {
            1.0 + (relationship.abs()) // 1.0 to 2.0
        } else {
            1.0 - (relationship * 0.5) // 1.0 to 0.5
        };

        let effective_resistance = base_resistance * multiplier * config.difficulty;
        (influence - effective_resistance).max(0.0)
    }
}
