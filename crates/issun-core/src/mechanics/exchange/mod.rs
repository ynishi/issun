//! Exchange mechanic: Asset and value exchange system.
//!
//! This module provides a policy-based exchange system that can model
//! various types of trades, from simple barter to complex market exchanges.
//!
//! # Architecture
//!
//! The exchange mechanic follows a **Policy-Based Design**:
//! - The core `ExchangeMechanic<V, E>` is generic over two policies
//! - `V: ValuationPolicy` determines how to calculate fair value
//! - `E: ExecutionPolicy` determines when to execute trades
//! - All logic is resolved at compile time via static dispatch
//!
//! # Quick Start
//!
//! ```
//! use issun_core::mechanics::exchange::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // Define your exchange type
//! type SimpleExchange = ExchangeMechanic<SimpleValuation, FairTradeExecution>;
//!
//! // Create configuration
//! let config = ExchangeConfig::default();
//! let mut state = ExchangeState::default();
//!
//! // Prepare input for this frame
//! let input = ExchangeInput {
//!     offered_value: 100.0,
//!     requested_value: 95.0,
//!     market_liquidity: 0.7,
//!     urgency: 0.3,
//! };
//!
//! // Simple event collector
//! # struct TestEmitter { events: Vec<ExchangeEvent> }
//! # impl EventEmitter<ExchangeEvent> for TestEmitter {
//! #     fn emit(&mut self, event: ExchangeEvent) { self.events.push(event); }
//! # }
//! let mut emitter = TestEmitter { events: vec![] };
//!
//! // Execute one step
//! SimpleExchange::step(&config, &mut state, input, &mut emitter);
//! ```
//!
//! # Module Organization
//!
//! - `types`: Basic data structures (Config, Input, Event, State)
//! - `policies`: Policy traits (ValuationPolicy, ExecutionPolicy)
//! - `strategies`: Concrete implementations of policies
//! - `mechanic`: The core `ExchangeMechanic<V, E>` implementation
//! - `prelude`: Convenient re-exports for common use
//!
//! # Policy Combinations
//!
//! ## Valuation Policies
//! - `SimpleValuation`: Direct value comparison with fairness threshold
//! - `MarketAdjustedValuation`: Adjusts value based on liquidity and reputation
//!
//! ## Execution Policies
//! - `FairTradeExecution`: Strict fairness enforcement
//! - `UrgentExecution`: Relaxes fairness based on urgency
//!
//! # Examples
//!
//! ## Example 1: Simple Fair Trade
//!
//! ```
//! use issun_core::mechanics::exchange::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! type FairMarket = ExchangeMechanic<SimpleValuation, FairTradeExecution>;
//!
//! let config = ExchangeConfig {
//!     transaction_fee_rate: 0.02, // 2% fee
//!     minimum_value_threshold: 10.0,
//!     fairness_threshold: 0.8, // 0.8x to 1.25x is fair
//! };
//!
//! let mut state = ExchangeState::default();
//! let input = ExchangeInput {
//!     offered_value: 100.0,
//!     requested_value: 105.0, // Slightly unfavorable but within threshold
//!     market_liquidity: 0.5,
//!     urgency: 0.0,
//! };
//!
//! # struct TestEmitter { events: Vec<ExchangeEvent> }
//! # impl EventEmitter<ExchangeEvent> for TestEmitter {
//! #     fn emit(&mut self, event: ExchangeEvent) { self.events.push(event); }
//! # }
//! let mut emitter = TestEmitter { events: vec![] };
//!
//! FairMarket::step(&config, &mut state, input, &mut emitter);
//! // Trade succeeds, fair_value = 100.0, fee = 2.0, final = 98.0
//! ```
//!
//! ## Example 2: Urgent Market Trade
//!
//! ```
//! use issun_core::mechanics::exchange::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! type UrgentMarket = ExchangeMechanic<MarketAdjustedValuation, UrgentExecution>;
//!
//! let config = ExchangeConfig::default();
//! let mut state = ExchangeState::new(0.8); // High reputation
//!
//! let input = ExchangeInput {
//!     offered_value: 70.0,
//!     requested_value: 100.0, // Unfavorable ratio
//!     market_liquidity: 0.9, // High liquidity helps
//!     urgency: 0.8, // High urgency allows unfair trade
//! };
//!
//! # struct TestEmitter { events: Vec<ExchangeEvent> }
//! # impl EventEmitter<ExchangeEvent> for TestEmitter {
//! #     fn emit(&mut self, event: ExchangeEvent) { self.events.push(event); }
//! # }
//! let mut emitter = TestEmitter { events: vec![] };
//!
//! UrgentMarket::step(&config, &mut state, input, &mut emitter);
//! // Trade succeeds due to urgency, but reputation decreases
//! ```

pub mod mechanic;
pub mod policies;
pub mod strategies;
pub mod types;

// Convenience
pub mod prelude;

// Re-export core types
pub use mechanic::ExchangeMechanic;
pub use policies::{ExecutionPolicy, ValuationPolicy};
pub use types::{ExchangeConfig, ExchangeEvent, ExchangeInput, ExchangeState, RejectionReason};
