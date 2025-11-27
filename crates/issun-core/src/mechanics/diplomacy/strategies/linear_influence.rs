use crate::mechanics::diplomacy::policies::InfluencePolicy;
use crate::mechanics::diplomacy::types::{ArgumentType, DiplomacyConfig};

/// Simple linear influence calculation.
///
/// Influence = Base Strength * Multiplier
pub struct LinearInfluence;

impl InfluencePolicy for LinearInfluence {
    fn calculate_influence(
        base_strength: f32,
        _arg_type: ArgumentType,
        _config: &DiplomacyConfig,
    ) -> f32 {
        base_strength
    }
}
