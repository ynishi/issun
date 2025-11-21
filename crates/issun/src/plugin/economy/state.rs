//! Runtime state for economy plugin

use super::types::{Currency, CurrencyId, ResourceId};
use crate::store::Store;

/// Runtime state of currency holdings (Mutable)
pub type Wallet = Store<CurrencyId, Currency>;

/// Helper methods for Wallet
#[allow(clippy::result_unit_err)]
pub trait WalletExt {
    fn balance(&self, currency: &CurrencyId) -> Currency;
    fn deposit(&mut self, currency: &CurrencyId, amount: Currency);
    fn withdraw(&mut self, currency: &CurrencyId, amount: Currency) -> Result<(), ()>;
}

// Note: Since Store is a generic type alias, we can't easily implement traits on it directly
// without a newtype wrapper or using the underlying HashMap methods directly in the Service.
// For now, we'll rely on the Service to handle the logic using standard Store methods.

// ============================================================================
// Root Resource System State
// ============================================================================

/// Runtime state of resource holdings (Mutable)
///
/// This tracks the actual quantities of resources available.
/// For infinite resources, the value represents the generation capacity (Flow type)
/// or an abstract power level (Abstract type).
pub type ResourceInventory = Store<ResourceId, i64>;
