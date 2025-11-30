//! State for macroeconomy plugin

use issun_core::mechanics::macroeconomy::EconomicIndicators;

/// State for macroeconomy plugin
#[derive(Debug, Clone)]
pub struct MacroeconomyState {
    /// Current economic indicators
    pub indicators: EconomicIndicators,
}

impl Default for MacroeconomyState {
    fn default() -> Self {
        Self {
            indicators: EconomicIndicators::default(),
        }
    }
}
