//! Loot service - Drop calculation and rarity selection
//!
//! Domain Service for loot system logic (pure functions).

use super::types::{DropConfig, Rarity};
use rand::Rng;

/// Loot service for drop calculations and rarity selection
///
/// Provides pure functions for:
/// - Drop rate calculations
/// - Weighted rarity selection
/// - Drop quantity determination
#[derive(Debug, Clone, issun_macros::Service)]
#[service(name = "loot_service")]
pub struct LootService;

impl LootService {
    pub fn new() -> Self {
        Self
    }

    /// Determine if a drop should occur based on drop config
    ///
    /// # Arguments
    /// * `config` - Drop configuration with base rate and multiplier
    /// * `rng` - Random number generator
    ///
    /// # Example
    /// ```ignore
    /// let config = DropConfig::new(0.3, 1.5); // 30% * 1.5 = 45% chance
    /// let mut rng = rand::thread_rng();
    /// if LootService::should_drop(&config, &mut rng) {
    ///     // Generate loot
    /// }
    /// ```
    pub fn should_drop(config: &DropConfig, rng: &mut impl Rng) -> bool {
        rng.gen_bool(config.final_rate() as f64)
    }

    /// Select a random rarity based on drop weights
    ///
    /// Uses weighted random selection where each rarity has a weight
    /// (Common = 50, Uncommon = 25, Rare = 15, Epic = 7, Legendary = 3)
    ///
    /// # Arguments
    /// * `rng` - Random number generator
    ///
    /// # Returns
    /// Randomly selected rarity tier
    pub fn select_rarity(rng: &mut impl Rng) -> Rarity {
        let rarities = Rarity::all();
        let total_weight: f32 = rarities.iter().map(|r| r.drop_weight()).sum();

        let mut roll = rng.gen_range(0.0..total_weight);

        for rarity in rarities {
            roll -= rarity.drop_weight();
            if roll <= 0.0 {
                return rarity;
            }
        }

        // Fallback (should never reach here)
        Rarity::Common
    }

    /// Calculate number of drops from multiple sources
    ///
    /// Each source has an independent chance to drop based on config.
    ///
    /// # Arguments
    /// * `count` - Number of potential drop sources (e.g., number of defeated enemies)
    /// * `config` - Drop configuration
    /// * `rng` - Random number generator
    ///
    /// # Returns
    /// Number of successful drops
    ///
    /// # Example
    /// ```ignore
    /// let config = DropConfig::new(0.3, 1.0);
    /// let mut rng = rand::thread_rng();
    /// let drop_count = LootService::calculate_drop_count(5, &config, &mut rng);
    /// // Returns 0-5 based on 30% chance per source
    /// ```
    pub fn calculate_drop_count(
        count: usize,
        config: &DropConfig,
        rng: &mut impl Rng,
    ) -> usize {
        (0..count)
            .filter(|_| Self::should_drop(config, rng))
            .count()
    }

    /// Generate multiple rarities for drops
    ///
    /// # Arguments
    /// * `count` - Number of drops to generate rarities for
    /// * `rng` - Random number generator
    ///
    /// # Returns
    /// Vector of randomly selected rarities
    pub fn generate_rarities(count: usize, rng: &mut impl Rng) -> Vec<Rarity> {
        (0..count).map(|_| Self::select_rarity(rng)).collect()
    }
}

impl Default for LootService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::Service;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_should_drop_deterministic() {
        let config = DropConfig::new(1.0, 1.0); // 100% drop rate
        let mut rng = StdRng::seed_from_u64(42);
        assert!(LootService::should_drop(&config, &mut rng));
    }

    #[test]
    fn test_select_rarity() {
        let mut rng = StdRng::seed_from_u64(42);
        let rarity = LootService::select_rarity(&mut rng);
        // Should return a valid rarity
        assert!(Rarity::all().contains(&rarity));
    }

    #[test]
    fn test_calculate_drop_count() {
        let config = DropConfig::new(1.0, 1.0); // 100% drop rate
        let mut rng = StdRng::seed_from_u64(42);
        let count = LootService::calculate_drop_count(5, &config, &mut rng);
        assert_eq!(count, 5); // All should drop at 100%
    }

    #[test]
    fn test_generate_rarities() {
        let mut rng = StdRng::seed_from_u64(42);
        let rarities = LootService::generate_rarities(10, &mut rng);
        assert_eq!(rarities.len(), 10);
        // All should be valid rarities
        for rarity in rarities {
            assert!(Rarity::all().contains(&rarity));
        }
    }

    #[test]
    fn test_service_trait() {
        let service = LootService::new();
        assert_eq!(service.name(), "loot_service");
    }
}
