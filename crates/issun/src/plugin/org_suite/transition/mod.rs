//! Transition system for organizational metamorphosis
//!
//! This module provides the core framework for converting organizational data
//! between different archetypes and evaluating transition conditions.

pub mod converter;
pub mod condition;
pub mod registry;
pub mod defaults;

pub use converter::OrgConverter;
pub use condition::{ConditionContext, TransitionCondition};
pub use registry::TransitionRegistry;
pub use defaults::{
    // Default converters (12 total covering all 16 transitions)
    HolacracyToHierarchyConverter,
    HolacracyToSocialConverter,
    HolacracyToCultureConverter,
    HierarchyToHolacracyConverter,
    HierarchyToSocialConverter,
    HierarchyToCultureConverter,
    SocialToHolacracyConverter,
    SocialToHierarchyConverter,
    SocialToCultureConverter,
    CultureToHolacracyConverter,
    CultureToHierarchyConverter,
    CultureToSocialConverter,
    // Default conditions
    ScalingCondition,
    DecayCondition,
    RadicalizationCondition,
};
