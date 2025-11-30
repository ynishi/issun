//! State for macroeconomy plugin

use issun_core::mechanics::macroeconomy::EconomicIndicators;

/// State for macroeconomy plugin
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct MacroeconomyState {
    /// Current economic indicators
    pub indicators: EconomicIndicators,
}

