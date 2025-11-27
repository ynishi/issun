//! Elemental calculation strategies.
//!
//! This module provides concrete implementations of `ElementalPolicy`.

mod affinity;
mod none;

pub use affinity::ElementalAffinity;
pub use none::NoElemental;
