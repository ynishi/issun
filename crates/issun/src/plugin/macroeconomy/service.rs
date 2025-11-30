//! Service for macroeconomy plugin

use super::resources::EconomicMetrics;
use issun_core::mechanics::macroeconomy::EconomicSnapshot;

/// Service for macroeconomy aggregation logic
#[derive(Debug, Clone, Copy, Default)]
pub struct MacroeconomyService;

impl MacroeconomyService {
    /// Create economic snapshot from aggregated metrics
    pub fn create_snapshot(&self, metrics: &EconomicMetrics) -> EconomicSnapshot {
        EconomicSnapshot {
            transaction_volume: metrics.market.transaction_volume,
            price_changes: metrics.market.price_changes.clone(),
            production_output: metrics.production.total_output,
            currency_circulation: metrics.currency.total_circulation,
            resource_availability: metrics.resources.availability.clone(),
            current_tick: metrics.current_tick,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_create_snapshot() {
        let service = MacroeconomyService;
        let mut metrics = EconomicMetrics::default();
        metrics.market.transaction_volume = 5000.0;
        metrics.market.price_changes = vec![("bread".to_string(), 0.02)].into_iter().collect();
        metrics.production.total_output = 1000.0;
        metrics.currency.total_circulation = 100_000.0;
        metrics.resources.availability = vec![("wood".to_string(), 500)].into_iter().collect();
        metrics.current_tick = 100;

        let snapshot = service.create_snapshot(&metrics);

        assert_eq!(snapshot.transaction_volume, 5000.0);
        assert_eq!(snapshot.price_changes.len(), 1);
        assert_eq!(snapshot.production_output, 1000.0);
        assert_eq!(snapshot.currency_circulation, 100_000.0);
        assert_eq!(snapshot.resource_availability.len(), 1);
        assert_eq!(snapshot.current_tick, 100);
    }
}
