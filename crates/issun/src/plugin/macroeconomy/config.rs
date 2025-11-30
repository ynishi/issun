//! Configuration for macroeconomy plugin

use issun_core::mechanics::macroeconomy::EconomicParameters;

/// Configuration for macroeconomy plugin
#[derive(Debug, Clone)]
pub struct MacroeconomyConfig {
    /// Economic parameters (from core mechanic)
    pub parameters: EconomicParameters,

    /// Update interval in ticks (0 = every tick)
    pub update_interval: u64,

    /// Enable economic shock detection
    pub enable_shock_detection: bool,
}

impl Default for MacroeconomyConfig {
    fn default() -> Self {
        Self {
            parameters: EconomicParameters::default(),
            update_interval: 10, // Update every 10 ticks
            enable_shock_detection: true,
        }
    }
}
