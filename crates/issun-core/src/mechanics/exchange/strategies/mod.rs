//! Concrete strategy implementations for exchange policies.

mod fair_trade_execution;
mod market_adjusted_valuation;
mod simple_valuation;
mod urgent_execution;

pub use fair_trade_execution::FairTradeExecution;
pub use market_adjusted_valuation::MarketAdjustedValuation;
pub use simple_valuation::SimpleValuation;
pub use urgent_execution::UrgentExecution;
