//! System for macroeconomy plugin

use super::config::MacroeconomyConfig;
use super::resources::EconomicMetrics;
use super::service::MacroeconomyService;
use super::state::MacroeconomyState;
use issun_core::mechanics::macroeconomy::prelude::*;
use issun_core::mechanics::{EventEmitter, Mechanic};

/// Type alias for the default macroeconomy mechanic
pub type DefaultMacroeconomy = MacroeconomyMechanic<SimpleEconomicPolicy>;

/// System for running macroeconomy mechanic
#[derive(Debug, Clone, Copy)]
pub struct MacroeconomySystem;

impl MacroeconomySystem {
    /// Check if macroeconomy should update this tick
    pub fn should_update(current_tick: u64, config: &MacroeconomyConfig) -> bool {
        if config.update_interval == 0 {
            return true; // Update every tick
        }
        current_tick.is_multiple_of(config.update_interval)
    }

    /// Update economic indicators
    pub fn update_indicators<
        E: EventEmitter<issun_core::mechanics::macroeconomy::EconomicEvent>,
    >(
        config: &MacroeconomyConfig,
        state: &mut MacroeconomyState,
        metrics: &EconomicMetrics,
        emitter: &mut E,
    ) {
        let service = MacroeconomyService;
        let snapshot = service.create_snapshot(metrics);

        // Run the mechanic
        DefaultMacroeconomy::step(&config.parameters, &mut state.indicators, snapshot, emitter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_update_every_tick() {
        let mut config = MacroeconomyConfig::default();
        config.update_interval = 0;

        assert!(MacroeconomySystem::should_update(0, &config));
        assert!(MacroeconomySystem::should_update(1, &config));
        assert!(MacroeconomySystem::should_update(100, &config));
    }

    #[test]
    fn test_should_update_interval() {
        let mut config = MacroeconomyConfig::default();
        config.update_interval = 10;

        assert!(MacroeconomySystem::should_update(0, &config));
        assert!(!MacroeconomySystem::should_update(1, &config));
        assert!(!MacroeconomySystem::should_update(9, &config));
        assert!(MacroeconomySystem::should_update(10, &config));
        assert!(MacroeconomySystem::should_update(20, &config));
    }

    #[test]
    fn test_update_indicators() {
        let config = MacroeconomyConfig::default();
        let mut state = MacroeconomyState::default();
        let mut metrics = EconomicMetrics::default();
        metrics.current_tick = 100;

        struct TestEmitter {
            events: Vec<issun_core::mechanics::macroeconomy::EconomicEvent>,
        }
        impl EventEmitter<issun_core::mechanics::macroeconomy::EconomicEvent> for TestEmitter {
            fn emit(&mut self, event: issun_core::mechanics::macroeconomy::EconomicEvent) {
                self.events.push(event);
            }
        }

        let mut emitter = TestEmitter { events: vec![] };

        MacroeconomySystem::update_indicators(&config, &mut state, &metrics, &mut emitter);

        // State should be updated
        assert_eq!(state.indicators.last_update, 100);
    }
}
