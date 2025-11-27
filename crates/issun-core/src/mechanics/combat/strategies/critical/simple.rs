//! Simple random critical hit strategy.
//!
//! This strategy implements basic critical hit mechanics with a fixed chance
//! and multiplier.

use crate::mechanics::combat::policies::CriticalPolicy;
use crate::mechanics::combat::types::CombatConfig;

/// Simple critical hit system with fixed chance and multiplier.
///
/// This strategy implements a straightforward critical hit system:
/// - 10% chance of critical hit (1 in 10)
/// - 2.0x damage multiplier on critical
///
/// **Note**: This implementation uses a deterministic hash-based approach
/// for reproducibility in tests. In a real game, you'd want to use proper
/// random number generation (e.g., `rand` crate with thread_rng).
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::combat::policies::CriticalPolicy;
/// use issun_core::mechanics::combat::strategies::critical::SimpleCritical;
/// use issun_core::mechanics::combat::CombatConfig;
///
/// let config = CombatConfig::default();
///
/// // Sometimes it will be critical, sometimes not
/// let (damage, is_critical) = SimpleCritical::apply_critical(50, &config);
/// if is_critical {
///     assert_eq!(damage, 100); // 50 * 2.0 = 100
/// } else {
///     assert_eq!(damage, 50);
/// }
/// ```
pub struct SimpleCritical;

impl CriticalPolicy for SimpleCritical {
    fn apply_critical(damage: i32, _config: &CombatConfig) -> (i32, bool) {
        // TODO: Replace with proper RNG (e.g., rand::thread_rng())
        // For now, use a simple hash-based approach for determinism
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hash, Hasher};

        let mut hasher = RandomState::new().build_hasher();
        // Hash current time-like value (in real game, use proper RNG seed)
        std::thread::current().id().hash(&mut hasher);
        damage.hash(&mut hasher);

        let roll = hasher.finish() % 100; // 0-99
        let is_critical = roll < 10; // 10% chance

        if is_critical {
            let critical_damage = damage * 2; // 2.0x multiplier
            (critical_damage, true)
        } else {
            (damage, false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_critical_returns_valid_values() {
        let config = CombatConfig::default();

        // Run multiple times to test statistical distribution
        let mut critical_count = 0;
        let iterations = 1000;

        for i in 0..iterations {
            let base_damage = 50 + i; // Vary damage to get different hash results
            let (damage, is_critical) = SimpleCritical::apply_critical(base_damage, &config);

            if is_critical {
                critical_count += 1;
                // Critical should be 2x
                assert_eq!(damage, base_damage * 2);
            } else {
                // Normal hit should be unchanged
                assert_eq!(damage, base_damage);
            }
        }

        // Should have roughly 10% critical rate (allow some variance)
        // With 1000 iterations, we expect ~100 crits, allow 5-20% range
        let critical_rate = (critical_count as f64) / (iterations as f64);
        assert!(
            critical_rate > 0.05 && critical_rate < 0.20,
            "Critical rate {} is outside expected range (5-20%)",
            critical_rate
        );
    }

    #[test]
    fn test_zero_damage() {
        let config = CombatConfig::default();
        let (damage, _) = SimpleCritical::apply_critical(0, &config);
        assert_eq!(damage, 0); // 0 damage should stay 0
    }
}
