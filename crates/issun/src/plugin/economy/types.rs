//! Core types for economy plugin

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Unique identifier for a currency
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CurrencyId(pub String);

impl CurrencyId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for CurrencyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for CurrencyId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Metadata definition for a currency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyDefinition {
    pub id: CurrencyId,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

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

// ============================================================================
// Root Resource System Types
// ============================================================================

/// Unique identifier for a resource
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(pub String);

impl ResourceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ResourceId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Type of resource behavior
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    /// Stock resource: accumulated and stored (Gold, Food, Iron)
    Stock,
    /// Flow resource: represents per-turn generation capacity (GDP, Production)
    Flow { per_turn: bool },
    /// Abstract resource: represents abstract values (National Power, Influence)
    Abstract,
    /// Hybrid: combines multiple aspects
    Hybrid,
}

/// Metadata definition for a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDefinition {
    pub id: ResourceId,
    pub name: String,
    pub description: String,
    pub resource_type: ResourceType,
    pub is_infinite: bool,
    pub metadata: HashMap<String, String>,
}

impl ResourceDefinition {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        resource_type: ResourceType,
    ) -> Self {
        Self {
            id: ResourceId::new(id),
            name: name.into(),
            description: description.into(),
            resource_type,
            is_infinite: true,
            metadata: HashMap::new(),
        }
    }

    pub fn with_finite(mut self) -> Self {
        self.is_infinite = false;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

// ============================================================================
// Currency Exchange System Types
// ============================================================================

/// Type of exchange rate
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RateType {
    /// Static rate that doesn't change
    Static,
    /// Dynamic rate that can be updated
    Dynamic,
}

/// Exchange rate between two currencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRate {
    pub from: CurrencyId,
    pub to: CurrencyId,
    /// How many units of 'to' currency you get for 1 unit of 'from' currency
    pub rate: f64,
    pub rate_type: RateType,
}

impl ExchangeRate {
    pub fn new(from: CurrencyId, to: CurrencyId, rate: f64) -> Self {
        Self {
            from,
            to,
            rate,
            rate_type: RateType::Static,
        }
    }

    pub fn dynamic(from: CurrencyId, to: CurrencyId, rate: f64) -> Self {
        Self {
            from,
            to,
            rate,
            rate_type: RateType::Dynamic,
        }
    }

    /// Calculate how much 'to' currency you get from 'from_amount'
    pub fn convert(&self, from_amount: Currency) -> Currency {
        let result = (from_amount.amount() as f64 * self.rate).round() as i64;
        Currency::new(result)
    }
}

// ============================================================================
// Resource to Currency Conversion Types
// ============================================================================

/// Conversion rule from resource to currency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionRule {
    pub resource: ResourceId,
    pub currency: CurrencyId,
    /// How many units of currency you get for 1 unit of resource
    pub conversion_rate: f64,
}

impl ConversionRule {
    pub fn new(resource: ResourceId, currency: CurrencyId, conversion_rate: f64) -> Self {
        Self {
            resource,
            currency,
            conversion_rate,
        }
    }

    /// Calculate how much currency you get from resource_amount
    pub fn convert(&self, resource_amount: i64) -> Currency {
        let result = (resource_amount as f64 * self.conversion_rate).round() as i64;
        Currency::new(result)
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

    #[test]
    fn test_resource_definition_builder() {
        let resource =
            ResourceDefinition::new("gold", "Gold", "Precious metal", ResourceType::Stock)
                .with_finite()
                .with_metadata("rarity", "legendary");

        assert_eq!(resource.id.0, "gold");
        assert_eq!(resource.name, "Gold");
        assert!(!resource.is_infinite);
        assert_eq!(resource.metadata.get("rarity").unwrap(), "legendary");
    }

    #[test]
    fn test_exchange_rate_convert() {
        let rate = ExchangeRate::new(CurrencyId::new("usd"), CurrencyId::new("jpy"), 150.0);

        let usd = Currency::new(100);
        let jpy = rate.convert(usd);
        assert_eq!(jpy.amount(), 15000);
    }

    #[test]
    fn test_conversion_rule_convert() {
        let rule = ConversionRule::new(
            ResourceId::new("gold_ore"),
            CurrencyId::new("gold_coin"),
            10.0,
        );

        let currency = rule.convert(5);
        assert_eq!(currency.amount(), 50);
    }

    #[test]
    fn test_exchange_rate_fractional() {
        let rate = ExchangeRate::new(CurrencyId::new("eur"), CurrencyId::new("usd"), 1.1);

        let eur = Currency::new(100);
        let usd = rate.convert(eur);
        assert_eq!(usd.amount(), 110);
    }
}
