//! Prelude for the contagion mechanic.
//!
//! This module re-exports the most commonly used items for the contagion mechanic.
//! Import this to get everything you need in one line.
//!
//! # Examples
//!
//! ```
//! use issun_core::mechanics::contagion::prelude::*;
//!
//! // Now you have access to:
//! // Basic types:
//! // - ContagionMechanic<S, P>
//! // - ContagionConfig, SimpleSeverity, ContagionInput, ContagionEvent
//! // - SpreadPolicy, ProgressionPolicy
//! // - LinearSpread, ExponentialSpread
//! // - LinearProgression, ThresholdProgression
//! // - Presets: SimpleVirus, ExplosiveVirus, ZombieVirus, etc.
//! //
//! // Advanced types:
//! // - Duration, InfectionState, InfectionStateType
//! // - ContagionContent, DiseaseLevel, TrendDirection
//!
//! // Use a preset
//! type MyVirus = ZombieVirus;
//!
//! // Or customize from scratch
//! type CustomVirus = ContagionMechanic<LinearSpread, LinearProgression>;
//! ```

// Basic types
pub use super::mechanic::ContagionMechanic;
pub use super::policies::{ProgressionPolicy, SpreadPolicy};
pub use super::presets::*;
pub use super::strategies::{
    ExponentialSpread, LinearProgression, LinearSpread, ThresholdProgression,
};
pub use super::types::{ContagionConfig, ContagionEvent, ContagionInput, SimpleSeverity};

// Advanced types
pub use super::content::{ContagionContent, DiseaseLevel, TrendDirection};
pub use super::duration::Duration;
pub use super::state::{InfectionState, InfectionStateType};
