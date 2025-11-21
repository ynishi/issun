//! Accounting service for domain logic

use super::Currency;

/// Accounting service providing pure financial calculations
///
/// This service handles stateless calculations for accounting operations.
/// It follows Domain-Driven Design principles - accounting logic as a service.
///
/// # Example
///
/// ```ignore
/// use issun::plugin::accounting::{AccountingService, Currency};
///
/// let service = AccountingService::new();
/// let net = service.calculate_settlement_net(
///     Currency::new(1000),
///     Currency::new(400)
/// );
/// println!("Net income: {}", net);
/// ```
#[derive(Debug, Clone, Default, issun_macros::Service)]
#[service(name = "accounting_service")]
pub struct AccountingService;

impl AccountingService {
    /// Create a new accounting service
    pub fn new() -> Self {
        Self
    }

    /// Calculate settlement net income
    ///
    /// Simple formula: income - expenses
    ///
    /// # Arguments
    ///
    /// * `income` - Total income for the period
    /// * `expenses` - Total expenses for the period
    ///
    /// # Returns
    ///
    /// Net income (can be negative)
    pub fn calculate_settlement_net(&self, income: Currency, expenses: Currency) -> Currency {
        income - expenses
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
    fn test_calculate_settlement_net_positive() {
        let service = AccountingService::new();
        let income = Currency::new(1000);
        let expenses = Currency::new(400);

        let net = service.calculate_settlement_net(income, expenses);
        assert_eq!(net.amount(), 600);
    }

    #[test]
    fn test_calculate_settlement_net_negative() {
        let service = AccountingService::new();
        let income = Currency::new(300);
        let expenses = Currency::new(500);

        let net = service.calculate_settlement_net(income, expenses);
        assert_eq!(net.amount(), -200);
    }

    #[test]
    fn test_calculate_reserve_bonus_positive() {
        let service = AccountingService::new();
        let net_income = Currency::new(1000);

        let bonus = service.calculate_reserve_bonus(net_income, 0.1);
        assert_eq!(bonus.amount(), 100);
    }

    #[test]
    fn test_calculate_reserve_bonus_negative_income() {
        let service = AccountingService::new();
        let net_income = Currency::new(-500);

        let bonus = service.calculate_reserve_bonus(net_income, 0.1);
        assert_eq!(bonus.amount(), 0);
    }

    #[test]
    fn test_service_default() {
        let service = AccountingService;
        assert_eq!(service.name(), "accounting_service");
    }
}
