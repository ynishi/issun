//! Service for economy plugin

use super::resources::{ConversionRules, ExchangeRates, ResourceDefinitions};
use super::state::{ResourceInventory, Wallet};
use super::types::{Currency, CurrencyId, ResourceId};
use crate::service::Service;
use std::any::Any;

/// Result type for economy operations
pub type EconomyResult<T> = Result<T, EconomyError>;

/// Errors that can occur in economy operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EconomyError {
    InsufficientFunds,
    InsufficientResource,
    ExchangeRateNotFound,
    ConversionRuleNotFound,
    ResourceNotFound,
    ResourceNotFinite,
}

/// Service for currency calculations and transactions
#[derive(Clone, Default)]
pub struct EconomyService;

impl EconomyService {
    // ========================================================================
    // Currency Operations
    // ========================================================================

    /// Check balance of a specific currency in the wallet
    pub fn balance(&self, wallet: &Wallet, currency_id: &CurrencyId) -> Currency {
        wallet.get(currency_id).cloned().unwrap_or(Currency::ZERO)
    }

    /// Add currency to the wallet
    pub fn deposit(&self, wallet: &mut Wallet, currency_id: &CurrencyId, amount: Currency) {
        let current = self.balance(wallet, currency_id);
        wallet.insert(currency_id.clone(), current + amount);
    }

    /// Subtract currency from the wallet
    ///
    /// Returns `Ok(())` if successful, `Err(())` if insufficient funds (and wallet doesn't allow debt - though Currency allows negative).
    /// For now, we allow negative balance as debt, so this always succeeds.
    /// Future improvements could add debt limits.
    pub fn withdraw(&self, wallet: &mut Wallet, currency_id: &CurrencyId, amount: Currency) {
        let current = self.balance(wallet, currency_id);
        wallet.insert(currency_id.clone(), current - amount);
    }

    /// Transfer currency between two wallets (if we had multiple wallets, but currently we have one global wallet store)
    /// This method is a placeholder for future multi-wallet support.
    pub fn transfer(
        &self,
        from_wallet: &mut Wallet,
        to_wallet: &mut Wallet,
        currency_id: &CurrencyId,
        amount: Currency,
    ) {
        self.withdraw(from_wallet, currency_id, amount);
        self.deposit(to_wallet, currency_id, amount);
    }

    // ========================================================================
    // Currency Exchange Operations
    // ========================================================================

    /// Exchange currency from one type to another
    ///
    /// This performs: wallet[from] -= from_amount, wallet[to] += converted_amount
    ///
    /// Returns the amount of target currency received.
    pub fn exchange(
        &self,
        wallet: &mut Wallet,
        exchange_rates: &ExchangeRates,
        from_currency: &CurrencyId,
        to_currency: &CurrencyId,
        from_amount: Currency,
    ) -> EconomyResult<Currency> {
        // Check if exchange rate exists
        let rate = exchange_rates
            .get(from_currency, to_currency)
            .ok_or(EconomyError::ExchangeRateNotFound)?;

        // Check if wallet has enough funds
        let current_balance = self.balance(wallet, from_currency);
        if current_balance < from_amount {
            return Err(EconomyError::InsufficientFunds);
        }

        // Calculate converted amount
        let to_amount = rate.convert(from_amount);

        // Execute exchange
        self.withdraw(wallet, from_currency, from_amount);
        self.deposit(wallet, to_currency, to_amount);

        Ok(to_amount)
    }

    // ========================================================================
    // Resource Operations
    // ========================================================================

    /// Get resource quantity
    pub fn resource_quantity(
        &self,
        inventory: &ResourceInventory,
        resource_id: &ResourceId,
    ) -> i64 {
        inventory.get(resource_id).cloned().unwrap_or(0)
    }

    /// Add resource to inventory
    pub fn add_resource(
        &self,
        inventory: &mut ResourceInventory,
        _resource_definitions: &ResourceDefinitions,
        resource_id: &ResourceId,
        amount: i64,
    ) -> EconomyResult<()> {
        // For infinite resources, this just updates the capacity/power level
        let current = self.resource_quantity(inventory, resource_id);
        inventory.insert(resource_id.clone(), current + amount);
        Ok(())
    }

    /// Consume resource from inventory
    ///
    /// Returns error if insufficient resource (for finite resources).
    /// For infinite resources, this operation doesn't actually consume but may
    /// represent a temporary reduction in capacity.
    pub fn consume_resource(
        &self,
        inventory: &mut ResourceInventory,
        resource_definitions: &ResourceDefinitions,
        resource_id: &ResourceId,
        amount: i64,
    ) -> EconomyResult<()> {
        let current = self.resource_quantity(inventory, resource_id);

        // Check if we have enough (only for finite resources)
        if !resource_definitions.is_infinite(resource_id) && current < amount {
            return Err(EconomyError::InsufficientResource);
        }

        inventory.insert(resource_id.clone(), current - amount);
        Ok(())
    }

    // ========================================================================
    // Resource to Currency Conversion
    // ========================================================================

    /// Convert resource to currency
    ///
    /// This consumes the resource and generates currency in the wallet.
    ///
    /// Returns the amount of currency generated.
    #[allow(clippy::too_many_arguments)]
    pub fn convert_resource_to_currency(
        &self,
        inventory: &mut ResourceInventory,
        wallet: &mut Wallet,
        resource_definitions: &ResourceDefinitions,
        conversion_rules: &ConversionRules,
        resource_id: &ResourceId,
        currency_id: &CurrencyId,
        resource_amount: i64,
    ) -> EconomyResult<Currency> {
        // Get conversion rule
        let rule = conversion_rules
            .get_rule(resource_id, currency_id)
            .ok_or(EconomyError::ConversionRuleNotFound)?;

        // Consume resource (if finite, this will check availability)
        self.consume_resource(
            inventory,
            resource_definitions,
            resource_id,
            resource_amount,
        )?;

        // Generate currency
        let currency_amount = rule.convert(resource_amount);
        self.deposit(wallet, currency_id, currency_amount);

        Ok(currency_amount)
    }
}

#[async_trait::async_trait]
impl Service for EconomyService {
    fn name(&self) -> &'static str {
        "economy_service"
    }

    fn clone_box(&self) -> Box<dyn Service> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
