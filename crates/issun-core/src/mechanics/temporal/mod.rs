//! Temporal mechanic: Time management and action budget systems.
//!
//! This module provides a policy-based system for managing game time,
//! action points, and temporal mechanics across various game types.
//!
//! # Key Concepts
//!
//! ## DateTime vs Tick
//!
//! The temporal mechanic supports two time representations:
//!
//! - **GameDateTime**: Semantic calendar time (year/month/day/hour/minute)
//!   - Useful for games with day/night cycles, seasons, schedules
//!   - Supports Gregorian and 360-day calendars
//!
//! - **Tick**: Abstract counter
//!   - Useful for pure turn-based systems
//!   - Convertible to/from DateTime via `CalendarConfig`
//!
//! ## Action Budgets
//!
//! Two budget types for different game styles:
//!
//! - **ActionPoints**: Discrete budget for turn-based games
//!   - Resets at configurable boundaries (per day, per turn)
//!   - Example: 3 actions per day (Persona-style)
//!
//! - **ActionEnergy**: Continuous budget for real-time games
//!   - Regenerates over time based on `regen_rate`
//!   - Example: Stamina system in action RPGs
//!
//! # Architecture
//!
//! The temporal mechanic follows **Policy-Based Design**:
//! - The core `TemporalMechanic<P>` is generic over `TemporalPolicy`
//! - `P: TemporalPolicy` determines cost calculation and reset behavior
//! - All logic is resolved at compile time via static dispatch
//!
//! # Quick Start
//!
//! ```
//! use issun_core::mechanics::temporal::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // Use default turn-based configuration
//! type GameTemporal = TemporalMechanic;
//!
//! let config = TemporalConfig::turn_based(3); // 3 actions per day
//! let mut state = TemporalState::with_points(3);
//!
//! // Request an action
//! let input = TemporalInput {
//!     current_time: TemporalPoint::DateTime(GameDateTime::new(1, 1, 1, 10, 0)),
//!     requested_action: Some(ActionRequest::new("move")),
//!     ..Default::default()
//! };
//!
//! // Event collector
//! # struct TestEmitter { events: Vec<TemporalEvent> }
//! # impl EventEmitter<TemporalEvent> for TestEmitter {
//! #     fn emit(&mut self, event: TemporalEvent) { self.events.push(event); }
//! # }
//! let mut emitter = TestEmitter { events: vec![] };
//!
//! // Execute
//! GameTemporal::step(&config, &mut state, input, &mut emitter);
//!
//! // Check remaining actions
//! if let ActionBudget::Points(ap) = &state.budget {
//!     println!("Actions remaining: {}", ap.available);
//! }
//! ```
//!
//! # Calendar Systems
//!
//! Built-in calendar configurations:
//!
//! - **Gregorian**: Standard Earth calendar with leap years
//! - **Uniform 360**: 30 days × 12 months (common in simulations)
//! - **Custom**: Define your own month lengths and seasons
//!
//! ```
//! use issun_core::mechanics::temporal::types::CalendarConfig;
//!
//! let gregorian = CalendarConfig::gregorian();
//! let uniform = CalendarConfig::uniform_360();
//! let custom = CalendarConfig::custom(10, vec![36; 10]); // 10 months of 36 days
//! ```
//!
//! # Policy Strategies
//!
//! Available policy implementations:
//!
//! - `StandardTemporalPolicy`: Sensible defaults for most games
//! - `TurnBasedPolicy`: Optimized for turn-based RPGs
//! - `RealTimePolicy`: Optimized for action games with stamina
//! - `PersonaStylePolicy`: Calendar-focused with limited daily actions
//! - `StrategyGamePolicy`: Variable costs with season effects
//!
//! # Use Cases
//!
//! - **Turn-Based RPG**: Fixed actions per day, reset at midnight
//! - **Action RPG**: Stamina regeneration, continuous time
//! - **Strategy Game**: Variable costs, season modifiers
//! - **Life Sim**: Calendar events, time-slot activities
//! - **Roguelike**: Turn counting, hunger/time pressure
//!
//! # Global Time Interface
//!
//! For ECS integration, implement `GlobalTimeProvider` and `GlobalTimeController`:
//!
//! ```ignore
//! // In Bevy, back this with a Resource
//! impl GlobalTimeProvider for MyTimeResource {
//!     fn current_tick(&self) -> u64 { self.tick }
//!     fn time_flow_mode(&self) -> TimeFlowMode { self.mode }
//! }
//! ```

pub mod mechanic;
pub mod policies;
pub mod strategies;
pub mod types;

// Re-export core types
pub use mechanic::{SimpleTemporalMechanic, TemporalMechanic};
pub use policies::TemporalPolicy;
pub use types::{
    ActionBudget, ActionCost, ActionEnergy, ActionPoints, ActionRequest, ActionType, ActorContext,
    BudgetError, CalendarConfig, ConsumeResult, GameDateTime, GlobalTimeController,
    GlobalTimeProvider, ResetTrigger, Season, TemporalConfig, TemporalEvent, TemporalInput,
    TemporalPoint, TemporalState, TimeFlowMode, TimeOfDay,
};

/// Prelude module for convenient imports.
///
/// Import everything needed to use the temporal mechanic:
///
/// ```
/// use issun_core::mechanics::temporal::prelude::*;
/// ```
pub mod prelude {
    pub use super::mechanic::{SimpleTemporalMechanic, TemporalMechanic};
    pub use super::policies::TemporalPolicy;
    pub use super::strategies::{
        PersonaStylePolicy, RealTimePolicy, StandardTemporalPolicy, StrategyGamePolicy,
        TurnBasedPolicy,
    };
    pub use super::types::{
        ActionBudget, ActionCost, ActionEnergy, ActionPoints, ActionRequest, ActionType,
        ActorContext, BudgetError, CalendarConfig, ConsumeResult, GameDateTime,
        GlobalTimeController, GlobalTimeProvider, ResetTrigger, Season, TemporalConfig,
        TemporalEvent, TemporalInput, TemporalPoint, TemporalState, TimeFlowMode, TimeOfDay,
    };
}
