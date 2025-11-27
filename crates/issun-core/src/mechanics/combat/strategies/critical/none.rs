//! No critical hits strategy.
//!
//! This strategy disables critical hits entirely.

use crate::mechanics::combat::policies::CriticalPolicy;
use crate::mechanics::combat::types::CombatConfig;

/// No critical hits: always returns normal damage.
///
/// This strategy is used when you don't want critical hits in your game.
/// It simply passes through the damage value without modification and always
/// returns `is_critical = false`.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::combat::policies::CriticalPolicy;
/// use issun_core::mechanics::combat::strategies::critical::NoCritical;
/// use issun_core::mechanics::combat::CombatConfig;
///
/// let config = CombatConfig::default();
/// let (damage, is_critical) = NoCritical::apply_critical(50, &config);
/// assert_eq!(damage, 50);
/// assert!(!is_critical);
/// ```
pub struct NoCritical;

impl CriticalPolicy for NoCritical {
    fn apply_critical(damage: i32, _config: &CombatConfig) -> (i32, bool) {
        (damage, false) // Never critical
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_critical_passthrough() {
        let config = CombatConfig::default();

        // Damage should always be unchanged
        assert_eq!(NoCritical::apply_critical(50, &config), (50, false));
        assert_eq!(NoCritical::apply_critical(100, &config), (100, false));
        assert_eq!(NoCritical::apply_critical(1, &config), (1, false));
    }

    #[test]
    fn test_never_critical() {
        let config = CombatConfig::default();

        // Should never be critical, no matter how many times we call it
        for _ in 0..100 {
            let (_, is_critical) = NoCritical::apply_critical(50, &config);
            assert!(!is_critical);
        }
    }
}
