//! Critical hit calculation strategies.
//!
//! This module provides concrete implementations of `CriticalPolicy`.

mod none;
mod simple;

pub use none::NoCritical;
pub use simple::SimpleCritical;
