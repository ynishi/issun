//! Random number generation for ISSUN

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Seeded random number generator for reproducible gameplay
pub struct GameRng {
    rng: StdRng,
    seed: u64,
}

impl GameRng {
    /// Create a new RNG with a specific seed
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            seed,
        }
    }

    /// Create a new RNG with a random seed
    pub fn from_entropy() -> Self {
        let rng = StdRng::from_entropy();
        Self {
            seed: rand::random(),
            rng,
        }
    }

    /// Get the current seed
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Roll a dice with N sides (1..=sides)
    pub fn roll(&mut self, sides: u32) -> u32 {
        self.rng.gen_range(1..=sides)
    }

    /// Check if a random event occurs with given probability (0.0..=1.0)
    pub fn chance(&mut self, probability: f32) -> bool {
        self.rng.gen::<f32>() < probability
    }

    /// Generate a random number in range
    pub fn range(&mut self, min: i32, max: i32) -> i32 {
        self.rng.gen_range(min..=max)
    }

    /// Shuffle a slice in place
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        use rand::seq::SliceRandom;
        slice.shuffle(&mut self.rng);
    }

    /// Pick a random element from a slice
    pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        use rand::seq::SliceRandom;
        slice.choose(&mut self.rng)
    }

    /// Pick N random elements from a slice (without replacement)
    pub fn choose_multiple<'a, T>(&mut self, slice: &'a [T], amount: usize) -> Vec<&'a T> {
        use rand::seq::SliceRandom;
        slice.choose_multiple(&mut self.rng, amount).collect()
    }
}

impl Default for GameRng {
    fn default() -> Self {
        Self::from_entropy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seeded_rng() {
        let mut rng1 = GameRng::new(42);
        let mut rng2 = GameRng::new(42);

        // Same seed = same sequence
        assert_eq!(rng1.roll(6), rng2.roll(6));
        assert_eq!(rng1.roll(6), rng2.roll(6));
    }

    #[test]
    fn test_roll() {
        let mut rng = GameRng::new(42);

        for _ in 0..100 {
            let result = rng.roll(6);
            assert!(result >= 1 && result <= 6);
        }
    }

    #[test]
    fn test_chance() {
        let mut rng = GameRng::new(42);

        // With seed 42, check reproducibility
        let result1 = rng.chance(0.5);
        let mut rng2 = GameRng::new(42);
        let result2 = rng2.chance(0.5);

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_choose() {
        let mut rng = GameRng::new(42);
        let items = vec![1, 2, 3, 4, 5];

        let chosen = rng.choose(&items);
        assert!(chosen.is_some());
        assert!(items.contains(chosen.unwrap()));
    }
}
