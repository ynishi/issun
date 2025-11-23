//! CulturePlugin - Organizational culture and memetic behavior
//!
//! Provides culture-based organizational dynamics where "atmosphere" and implicit rules
//! drive member behavior, rather than explicit commands.
//!
//! # Core Concepts
//!
//! - **Culture Tags**: Memetic DNA defining organizational atmosphere (RiskTaking, Fanatic, etc.)
//! - **Personality Traits**: Individual member temperament (Cautious, Bold, Zealous, etc.)
//! - **Alignment**: Culture-personality fit affecting stress and fervor
//! - **Stress/Fervor**: Dynamic values showing member wellbeing and devotion
//!
//! # Example
//!
//! ```ignore
//! use issun::plugin::culture::*;
//!
//! let game = GameBuilder::new()
//!     .add_plugin(
//!         CulturePlugin::new()
//!             .with_config(CultureConfig::default()
//!                 .with_stress_rate(0.05))
//!             .register_faction("cult")
//!     )
//!     .build()
//!     .await?;
//! ```

// Module declarations
pub mod config;
pub mod events;
pub mod hook;
pub mod service;
pub mod state;
pub mod types;

// Public re-exports
pub use config::CultureConfig;
pub use events::*;
pub use hook::{CultureHook, DefaultCultureHook};
pub use service::CultureService;
pub use state::{CultureState, OrganizationCulture};
pub use types::{
    Alignment, CultureEffect, CultureError, CultureTag, FactionId, Member, MemberId,
    PersonalityTrait,
};
