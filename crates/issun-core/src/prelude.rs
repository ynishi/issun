//! Prelude module for issun-core.
//!
//! This module re-exports the most commonly used items from the library.
//!
//! # Examples
//!
//! ```
//! use issun_core::prelude::*;
//!
//! // Now you have access to the core traits:
//! // - Mechanic
//! // - EventEmitter
//! ```

pub use crate::mechanics::{EventEmitter, Mechanic};
