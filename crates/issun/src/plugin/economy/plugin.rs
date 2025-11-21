//! Economy plugin implementation

use super::resources::{
    ConversionRules, CurrencyDefinitions, EconomyConfig, ExchangeRates, ResourceDefinitions,
};
use super::service::EconomyService;
use super::state::{ResourceInventory, Wallet};
use super::system::EconomySystem;
use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;

/// Plugin for economy system
pub struct EconomyPlugin;

impl Default for EconomyPlugin {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Plugin for EconomyPlugin {
    fn name(&self) -> &'static str {
        "issun:economy"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register currency resources
        builder.register_resource(CurrencyDefinitions::new());
        builder.register_resource(EconomyConfig::default());

        // Register resource system resources
        builder.register_resource(ResourceDefinitions::new());
        builder.register_resource(ConversionRules::new());

        // Register exchange system resources
        builder.register_resource(ExchangeRates::new());

        // Register runtime states
        builder.register_runtime_state(Wallet::new());
        builder.register_runtime_state(ResourceInventory::new());

        // Register service
        builder.register_service(Box::new(EconomyService));

        // Register system
        builder.register_system(Box::new(EconomySystem));
    }
}
