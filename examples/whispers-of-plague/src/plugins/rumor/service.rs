use super::models::RumorEffect;
use issun::prelude::*;

/// Pure rumor calculation service (stateless)
#[derive(Clone, Default, DeriveService)]
#[service(name = "rumor_service")]
pub struct RumorService;

impl RumorService {
    /// Calculate panic change from rumor effect
    pub fn calculate_panic_delta(&self, effect: &RumorEffect, base_panic: f32) -> f32 {
        match effect {
            RumorEffect::IncreasePanic(delta) => base_panic + delta,
            RumorEffect::DecreasePanic(delta) => base_panic - delta,
            _ => base_panic,
        }
        .clamp(0.0, 1.0)
    }

    /// Calculate migration population
    pub fn calculate_migration(&self, from_pop: u32, rate: f32) -> u32 {
        ((from_pop as f32) * rate).round() as u32
    }

    /// Calculate credibility decay
    pub fn decay_credibility(&self, current: f32, decay_rate: f32) -> f32 {
        (current * decay_rate).max(0.1)
    }

    /// Check if rumor is still effective
    pub fn is_effective(&self, credibility: f32, threshold: f32) -> bool {
        credibility >= threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panic_calculation() {
        let service = RumorService;
        let effect = RumorEffect::IncreasePanic(0.3);
        let result = service.calculate_panic_delta(&effect, 0.5);
        assert_eq!(result, 0.8);
    }

    #[test]
    fn test_migration_calculation() {
        let service = RumorService;
        let migrants = service.calculate_migration(10000, 0.1);
        assert_eq!(migrants, 1000);
    }

    #[test]
    fn test_credibility_decay() {
        let service = RumorService;
        let decayed = service.decay_credibility(1.0, 0.9);
        assert_eq!(decayed, 0.9);
    }
}
