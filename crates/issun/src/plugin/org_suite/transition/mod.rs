//! Transition system for organizational metamorphosis
//!
//! This module provides the core framework for converting organizational data
//! between different archetypes and evaluating transition conditions.

pub mod condition;
pub mod converter;
pub mod defaults;
pub mod registry;

pub use condition::{ConditionContext, TransitionCondition};
pub use converter::OrgConverter;
pub use defaults::{
    CultureToHierarchyConverter,
    CultureToHolacracyConverter,
    CultureToSocialConverter,
    DecayCondition,
    HierarchyToCultureConverter,
    HierarchyToHolacracyConverter,
    HierarchyToSocialConverter,
    HolacracyToCultureConverter,
    // Default converters (12 total covering all 16 transitions)
    HolacracyToHierarchyConverter,
    HolacracyToSocialConverter,
    RadicalizationCondition,
    // Default conditions
    ScalingCondition,
    SocialToCultureConverter,
    SocialToHierarchyConverter,
    SocialToHolacracyConverter,
};
pub use registry::TransitionRegistry;
