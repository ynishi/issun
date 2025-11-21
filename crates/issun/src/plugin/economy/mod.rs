//! Economy plugin module

pub mod events;
pub mod plugin;
pub mod resources;
pub mod service;
pub mod state;
pub mod system;
pub mod types;

pub use events::*;
pub use plugin::EconomyPlugin;
pub use resources::{
    ConversionRules, CurrencyDefinitions, EconomyConfig, ExchangeRates, ResourceDefinitions,
};
pub use service::{EconomyError, EconomyResult, EconomyService};
pub use state::{ResourceInventory, Wallet, WalletExt};
pub use system::EconomySystem;
pub use types::{
    ConversionRule, Currency, CurrencyDefinition, CurrencyId, ExchangeRate, RateType,
    ResourceDefinition, ResourceId, ResourceType,
};
