//! Resources for economy plugin

use super::types::{
    ConversionRule, CurrencyDefinition, CurrencyId, ExchangeRate, ResourceDefinition, ResourceId,
};
use crate::resources::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Registry of currency definitions (ReadOnly)
#[derive(Default, Debug, Clone)]
pub struct CurrencyDefinitions {
    definitions: HashMap<CurrencyId, CurrencyDefinition>,
}

impl Resource for CurrencyDefinitions {}

impl CurrencyDefinitions {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
        }
    }

    pub fn register(&mut self, definition: CurrencyDefinition) {
        self.definitions.insert(definition.id.clone(), definition);
    }

    pub fn get(&self, id: &CurrencyId) -> Option<&CurrencyDefinition> {
        self.definitions.get(id)
    }

    pub fn all(&self) -> impl Iterator<Item = &CurrencyDefinition> {
        self.definitions.values()
    }
}

/// Global configuration for economy (ReadOnly)
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct EconomyConfig {
    pub default_currency: Option<CurrencyId>,
}

impl Resource for EconomyConfig {}

// ============================================================================
// Root Resource System Resources
// ============================================================================

/// Registry of resource definitions (ReadOnly)
#[derive(Default, Debug, Clone)]
pub struct ResourceDefinitions {
    definitions: HashMap<ResourceId, ResourceDefinition>,
}

impl Resource for ResourceDefinitions {}

impl ResourceDefinitions {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
        }
    }

    pub fn register(&mut self, definition: ResourceDefinition) {
        self.definitions.insert(definition.id.clone(), definition);
    }

    pub fn get(&self, id: &ResourceId) -> Option<&ResourceDefinition> {
        self.definitions.get(id)
    }

    pub fn all(&self) -> impl Iterator<Item = &ResourceDefinition> {
        self.definitions.values()
    }

    pub fn is_infinite(&self, id: &ResourceId) -> bool {
        self.definitions
            .get(id)
            .map(|def| def.is_infinite)
            .unwrap_or(false)
    }
}

// ============================================================================
// Currency Exchange System Resources
// ============================================================================

/// Registry of exchange rates between currencies (ReadOnly, but Dynamic rates can be updated)
#[derive(Default, Debug, Clone)]
pub struct ExchangeRates {
    rates: HashMap<(CurrencyId, CurrencyId), ExchangeRate>,
}

impl Resource for ExchangeRates {}

impl ExchangeRates {
    pub fn new() -> Self {
        Self {
            rates: HashMap::new(),
        }
    }

    pub fn register(&mut self, rate: ExchangeRate) {
        let key = (rate.from.clone(), rate.to.clone());
        self.rates.insert(key, rate);
    }

    pub fn get(&self, from: &CurrencyId, to: &CurrencyId) -> Option<&ExchangeRate> {
        self.rates.get(&(from.clone(), to.clone()))
    }

    pub fn update_rate(&mut self, from: &CurrencyId, to: &CurrencyId, new_rate: f64) -> bool {
        if let Some(rate) = self.rates.get_mut(&(from.clone(), to.clone())) {
            rate.rate = new_rate;
            true
        } else {
            false
        }
    }

    pub fn all(&self) -> impl Iterator<Item = &ExchangeRate> {
        self.rates.values()
    }
}

// ============================================================================
// Resource to Currency Conversion Resources
// ============================================================================

/// Registry of conversion rules from resources to currencies (ReadOnly)
#[derive(Default, Debug, Clone)]
pub struct ConversionRules {
    // resource_id -> list of conversion rules (one resource can convert to multiple currencies)
    rules: HashMap<ResourceId, Vec<ConversionRule>>,
}

impl Resource for ConversionRules {}

impl ConversionRules {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    pub fn register(&mut self, rule: ConversionRule) {
        self.rules
            .entry(rule.resource.clone())
            .or_insert_with(Vec::new)
            .push(rule);
    }

    /// Get all conversion rules for a specific resource
    pub fn get(&self, resource_id: &ResourceId) -> Option<&Vec<ConversionRule>> {
        self.rules.get(resource_id)
    }

    /// Get conversion rule for a specific resource -> currency pair
    pub fn get_rule(
        &self,
        resource_id: &ResourceId,
        currency_id: &CurrencyId,
    ) -> Option<&ConversionRule> {
        self.rules.get(resource_id).and_then(|rules| {
            rules
                .iter()
                .find(|rule| &rule.currency == currency_id)
        })
    }

    pub fn all(&self) -> impl Iterator<Item = &ConversionRule> {
        self.rules.values().flat_map(|rules| rules.iter())
    }
}
