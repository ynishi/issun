//! Defense calculation strategies.
//!
//! This module provides concrete implementations of `DefensePolicy`.

mod percentage;
mod subtractive;

pub use percentage::PercentageReduction;
pub use subtractive::SubtractiveDefense;
