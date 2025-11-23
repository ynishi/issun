//! Transition system for organizational metamorphosis
//!
//! This module provides the core framework for converting organizational data
//! between different archetypes and evaluating transition conditions.

pub mod converter;
pub mod condition;
pub mod registry;

pub use converter::OrgConverter;
pub use condition::{ConditionContext, TransitionCondition};
pub use registry::TransitionRegistry;
