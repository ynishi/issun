use crate::models::{GameMode, Rumor, RumorEffect};
use issun::prelude::*;

/// Pure rumor logic service (stateless)
#[derive(Clone, Default, DeriveService)]
#[service(name = "rumor_service")]
pub struct RumorService;

impl RumorService {
    /// Get available rumors based on game mode
    pub fn get_available_rumors(&self, mode: GameMode) -> Vec<Rumor> {
        match mode {
            GameMode::Plague => vec![
                Rumor {
                    id: "fear_mongering".into(),
                    name: "Government Conspiracy".into(),
                    description: "Spread fear about government cover-ups".into(),
                    effect: RumorEffect::IncreasePanic(0.2),
                    credibility: 1.0,
                },
                Rumor {
                    id: "false_cure".into(),
                    name: "Miracle Cure Discovered".into(),
                    description: "False hope leads to dangerous behavior".into(),
                    effect: RumorEffect::DecreasePanic(0.3),
                    credibility: 1.0,
                },
                Rumor {
                    id: "safe_zone_lie".into(),
                    name: "Safe Zone Identified".into(),
                    description: "Direct people to infected area".into(),
                    effect: RumorEffect::PromoteMigration {
                        from_district: 0,
                        to_district: 2,
                    },
                    credibility: 1.0,
                },
            ],
            GameMode::Savior => vec![
                Rumor {
                    id: "quarantine_success".into(),
                    name: "Quarantine Works".into(),
                    description: "Encourage isolation measures".into(),
                    effect: RumorEffect::PromoteIsolation,
                    credibility: 1.0,
                },
                Rumor {
                    id: "hope_message".into(),
                    name: "Recovery Stories".into(),
                    description: "Share survivor stories to reduce panic".into(),
                    effect: RumorEffect::DecreasePanic(0.2),
                    credibility: 1.0,
                },
                Rumor {
                    id: "evacuation".into(),
                    name: "Evacuation Order".into(),
                    description: "Move people away from infected zones".into(),
                    effect: RumorEffect::PromoteMigration {
                        from_district: 2,
                        to_district: 4,
                    },
                    credibility: 1.0,
                },
            ],
        }
    }

    /// Calculate panic change from rumor
    pub fn calculate_panic_change(&self, base_panic: f32, delta: f32) -> f32 {
        (base_panic + delta).clamp(0.0, 1.0)
    }

    /// Decay rumor credibility over time
    pub fn decay_credibility(&self, rumor: &mut Rumor) {
        rumor.credibility = (rumor.credibility * 0.9).max(0.1);
    }
}
