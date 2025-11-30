//! Prelude module for convenient imports.
//!
//! Import everything needed to use the organization mechanic:
//!
//! ```
//! use issun_core::mechanics::organization::prelude::*;
//! ```

pub use super::mechanic::{OrganizationMechanic, SimpleOrganizationMechanic};
pub use super::policies::OrganizationPolicy;
pub use super::strategies::SimpleOrganizationPolicy;
pub use super::types::{
    EfficiencyChangeReason, FitQuality, MemberArchetype, MemberId, OrganizationConfig,
    OrganizationEvent, OrganizationInput, OrganizationState, OrganizationType,
};
