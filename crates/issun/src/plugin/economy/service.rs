//! Economy service for domain logic

use super::{BudgetLedger, Currency, EconomyConfig};

/// Economy service providing pure financial calculations
///
/// This service handles stateless calculations for economic operations.
/// It follows Domain-Driven Design principles - economy logic as a service.
///
/// # Example
///
/// ```ignore
/// use issun::plugin::economy::{EconomyService, Currency, BudgetLedger, EconomyConfig};
///
/// let service = EconomyService::new();
/// let config = EconomyConfig::default();
/// let ledger = BudgetLedger::default();
///
/// let dividend = service.calculate_dividend(&config, &ledger);
/// println!("Dividend amount: {}", dividend);
/// ```
#[derive(Debug, Clone, Default, issun_macros::Service)]
#[service(name = "economy_service")]
pub struct EconomyService;

impl EconomyService {
    /// Create a new economy service
    pub fn new() -> Self {
        Self
    }

    /// Calculate dividend payment amount
    ///
    /// Formula: base + (total_assets * rate)
    ///
    /// # Arguments
    ///
    /// * `config` - Economy configuration
    /// * `ledger` - Current budget state
    ///
    /// # Returns
    ///
    /// Calculated dividend amount
    pub fn calculate_dividend(&self, config: &EconomyConfig, ledger: &BudgetLedger) -> Currency {
        let base = Currency::new(config.dividend_base);
        let total_assets = ledger.total_assets();
        let variable = Currency::new((total_assets.amount() as f32 * config.dividend_rate) as i64);

        base.saturating_add(variable)
    }

    /// Calculate settlement net income
    ///
    /// Simple formula: income - upkeep
    ///
    /// # Arguments
    ///
    /// * `income` - Total income for the period
    /// * `upkeep` - Total upkeep costs
    ///
    /// # Returns
    ///
    /// Net income (can be negative)
    pub fn calculate_settlement_net(&self, income: Currency, upkeep: Currency) -> Currency {
        income - upkeep
    }

    /// Calculate reserve bonus allocation
    ///
    /// Allocates a percentage of net income to reserve.
    ///
    /// # Arguments
    ///
    /// * `net_income` - Net income from settlement
    /// * `reserve_rate` - Rate to allocate (e.g., 0.1 = 10%)
    ///
    /// # Returns
    ///
    /// Amount to add to reserve
    pub fn calculate_reserve_bonus(&self, net_income: Currency, reserve_rate: f32) -> Currency {
        if net_income.is_negative() {
            return Currency::ZERO;
        }

        Currency::new((net_income.amount() as f32 * reserve_rate) as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::Service;

    #[test]
    fn test_calculate_dividend_base_only() {
        let service = EconomyService::new();
        let config = EconomyConfig {
            settlement_period_days: 7,
            dividend_base: 200,
            dividend_rate: 0.0, // No rate
        };
        let ledger = BudgetLedger::new(Currency::new(1000));

        let dividend = service.calculate_dividend(&config, &ledger);
        assert_eq!(dividend.amount(), 200);
    }

    #[test]
    fn test_calculate_dividend_with_rate() {
        let service = EconomyService::new();
        let config = EconomyConfig {
            settlement_period_days: 7,
            dividend_base: 200,
            dividend_rate: 0.04,
        };
        let ledger = BudgetLedger::new(Currency::new(1000));

        let total_assets = ledger.total_assets().amount();
        let expected_variable = (total_assets as f32 * 0.04) as i64;
        let expected = 200 + expected_variable;

        let dividend = service.calculate_dividend(&config, &ledger);
        assert_eq!(dividend.amount(), expected);
    }

    #[test]
    fn test_calculate_settlement_net_positive() {
        let service = EconomyService::new();
        let income = Currency::new(1000);
        let upkeep = Currency::new(400);

        let net = service.calculate_settlement_net(income, upkeep);
        assert_eq!(net.amount(), 600);
    }

    #[test]
    fn test_calculate_settlement_net_negative() {
        let service = EconomyService::new();
        let income = Currency::new(300);
        let upkeep = Currency::new(500);

        let net = service.calculate_settlement_net(income, upkeep);
        assert_eq!(net.amount(), -200);
    }

    #[test]
    fn test_calculate_reserve_bonus_positive() {
        let service = EconomyService::new();
        let net_income = Currency::new(1000);

        let bonus = service.calculate_reserve_bonus(net_income, 0.1);
        assert_eq!(bonus.amount(), 100);
    }

    #[test]
    fn test_calculate_reserve_bonus_negative_income() {
        let service = EconomyService::new();
        let net_income = Currency::new(-500);

        let bonus = service.calculate_reserve_bonus(net_income, 0.1);
        assert_eq!(bonus.amount(), 0);
    }

    #[test]
    fn test_service_default() {
        let service = EconomyService::default();
        assert_eq!(service.name(), "economy_service");
    }
}
