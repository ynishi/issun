//! Economy plugin implementation

use super::resources::{
    ConversionRules, CurrencyDefinitions, EconomyConfig, ExchangeRates, ResourceDefinitions,
};
use super::service::EconomyService;
use super::state::{ResourceInventory, Wallet};
use super::system::EconomySystem;
use crate::Plugin;

/// Plugin for economy system
#[derive(Default, Plugin)]
#[plugin(name = "issun:economy")]
// Resources
#[plugin(resource = CurrencyDefinitions)]
#[plugin(resource = EconomyConfig)]
#[plugin(resource = ResourceDefinitions)]
#[plugin(resource = ConversionRules)]
#[plugin(resource = ExchangeRates)]
// States
#[plugin(state = Wallet)]
#[plugin(state = ResourceInventory)]
// Service
#[plugin(service = EconomyService)]
// System
#[plugin(system = EconomySystem)]
pub struct EconomyPlugin;
