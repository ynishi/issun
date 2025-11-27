//! Spread strategy implementations.
//!
//! This module provides concrete implementations of the `SpreadPolicy` trait.

mod exponential;
mod linear;

pub use exponential::ExponentialSpread;
pub use linear::LinearSpread;
