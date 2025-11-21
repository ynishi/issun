//! Core types for economy plugin

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Currency value with safe arithmetic operations
///
/// This type wraps an `i64` and provides saturating arithmetic to prevent overflow/underflow.
/// Negative values represent debt.
///
/// # Example
///
/// ```ignore
/// use issun::plugin::economy::Currency;
///
/// let balance = Currency::new(1000);
/// let cost = Currency::new(300);
/// let remaining = balance - cost;
/// assert_eq!(remaining.amount(), 700);
/// ```
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Currency(i64);

impl Currency {
    /// Zero currency constant
    pub const ZERO: Currency = Currency(0);

    /// Create a new currency value
    pub fn new(amount: i64) -> Self {
        Self(amount)
    }

    /// Get the raw amount
    pub fn amount(&self) -> i64 {
        self.0
    }

    /// Add with saturation (won't overflow)
    pub fn saturating_add(self, other: Currency) -> Currency {
        Currency(self.0.saturating_add(other.0))
    }

    /// Subtract with saturation (won't underflow)
    pub fn saturating_sub(self, other: Currency) -> Currency {
        Currency(self.0.saturating_sub(other.0))
    }

    /// Check if currency is positive
    pub fn is_positive(&self) -> bool {
        self.0 > 0
    }

    /// Check if currency is zero
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Check if currency is negative (debt)
    pub fn is_negative(&self) -> bool {
        self.0 < 0
    }

    /// Get absolute value
    pub fn abs(&self) -> Currency {
        Currency(self.0.abs())
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add for Currency {
    type Output = Currency;

    fn add(self, other: Currency) -> Currency {
        Currency(self.0 + other.0)
    }
}

impl AddAssign for Currency {
    fn add_assign(&mut self, other: Currency) {
        self.0 += other.0;
    }
}

impl Sub for Currency {
    type Output = Currency;

    fn sub(self, other: Currency) -> Currency {
        Currency(self.0 - other.0)
    }
}

impl SubAssign for Currency {
    fn sub_assign(&mut self, other: Currency) {
        self.0 -= other.0;
    }
}

impl From<i64> for Currency {
    fn from(amount: i64) -> Self {
        Currency::new(amount)
    }
}

impl From<Currency> for i64 {
    fn from(currency: Currency) -> i64 {
        currency.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_creation() {
        let c = Currency::new(100);
        assert_eq!(c.amount(), 100);
    }

    #[test]
    fn test_currency_add() {
        let a = Currency::new(100);
        let b = Currency::new(50);
        assert_eq!((a + b).amount(), 150);
    }

    #[test]
    fn test_currency_sub() {
        let a = Currency::new(100);
        let b = Currency::new(30);
        assert_eq!((a - b).amount(), 70);
    }

    #[test]
    fn test_currency_saturating_add() {
        let a = Currency::new(i64::MAX - 10);
        let b = Currency::new(100);
        assert_eq!(a.saturating_add(b), Currency::new(i64::MAX));
    }

    #[test]
    fn test_currency_saturating_sub() {
        let a = Currency::new(i64::MIN + 10);
        let b = Currency::new(100);
        assert_eq!(a.saturating_sub(b), Currency::new(i64::MIN));
    }

    #[test]
    fn test_currency_predicates() {
        assert!(Currency::new(100).is_positive());
        assert!(!Currency::new(0).is_positive());
        assert!(!Currency::new(-100).is_positive());

        assert!(Currency::new(0).is_zero());
        assert!(!Currency::new(100).is_zero());

        assert!(Currency::new(-100).is_negative());
        assert!(!Currency::new(0).is_negative());
    }

    #[test]
    fn test_currency_abs() {
        assert_eq!(Currency::new(-100).abs(), Currency::new(100));
        assert_eq!(Currency::new(100).abs(), Currency::new(100));
    }

    #[test]
    fn test_currency_display() {
        let c = Currency::new(12345);
        assert_eq!(format!("{}", c), "12345");
    }

    #[test]
    fn test_currency_from_i64() {
        let c: Currency = 500.into();
        assert_eq!(c.amount(), 500);
    }
}
