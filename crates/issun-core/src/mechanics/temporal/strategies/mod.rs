//! Strategy implementations for temporal mechanic.
//!
//! This module contains concrete implementations of `TemporalPolicy`.

pub mod standard;

pub use standard::{
    PersonaStylePolicy, RealTimePolicy, StandardTemporalPolicy, StrategyGamePolicy, TurnBasedPolicy,
};
