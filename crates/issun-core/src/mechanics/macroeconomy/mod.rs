//! Macroeconomy mechanic: Macro-economic indicators and context.
//!
//! This module provides a policy-based macroeconomic system that models
//! the "invisible hand" of market forces: inflation, market sentiment,
//! resource scarcity, and economic cycles.
//!
//! # Architecture
//!
//! The macroeconomy mechanic follows a **Policy-Based Design**:
//! - The core `MacroeconomyMechanic<P>` is generic over `EconomicPolicy`
//! - `P: EconomicPolicy` determines how to calculate indicators
//! - All logic is resolved at compile time via static dispatch
//!
//! # Quick Start
//!
//! ```
//! use issun_core::mechanics::macroeconomy::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//! use std::collections::HashMap;
//!
//! // Define your macroeconomy type
//! type SimpleMacroeconomy = MacroeconomyMechanic<SimpleEconomicPolicy>;
//!
//! // Create configuration
//! let config = EconomicParameters::default();
//! let mut state = EconomicIndicators::default();
//!
//! // Prepare input snapshot
//! let snapshot = EconomicSnapshot {
//!     transaction_volume: 5000.0,
//!     price_changes: HashMap::new(),
//!     production_output: 1200.0,
//!     currency_circulation: 50000.0,
//!     resource_availability: HashMap::new(),
//!     current_tick: 1000,
//! };
//!
//! // Simple event collector
//! # struct TestEmitter { events: Vec<EconomicEvent> }
//! # impl EventEmitter<EconomicEvent> for TestEmitter {
//! #     fn emit(&mut self, event: EconomicEvent) { self.events.push(event); }
//! # }
//! let mut emitter = TestEmitter { events: vec![] };
//!
//! // Execute one step
//! SimpleMacroeconomy::step(&config, &mut state, snapshot, &mut emitter);
//! ```
//!
//! # Module Organization
//!
//! - `types`: Basic data structures (Config, Input, Event, State)
//! - `policies`: Policy traits (EconomicPolicy)
//! - `strategies`: Concrete implementations of policies
//! - `mechanic`: The core `MacroeconomyMechanic<P>` implementation
//! - `prelude`: Convenient re-exports for common use

pub mod mechanic;
pub mod policies;
pub mod strategies;
pub mod types;

// Convenience
pub mod prelude;

// Re-export core types
pub use mechanic::MacroeconomyMechanic;
pub use policies::EconomicPolicy;
pub use types::{
    CyclePhase, EconomicEvent, EconomicIndicators, EconomicParameters, EconomicSnapshot,
    SentimentDirection, ShockType,
};
