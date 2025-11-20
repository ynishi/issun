//! Weapon prototype telemetry and backlog tracking
//!
//! NOTE: This module has been migrated to use issun::plugin::ResearchPlugin.
//! The core research mechanics (queuing, progress, completion) are now handled by
//! ResearchPlugin with a custom PrototypeResearchHook (see hooks.rs).
//!
//! This file now only contains:
//! - PrototypeBacklog: UI display state for queued research and field reports
//! - FieldTelemetryService: Quality modifier calculations for field test data

use issun::prelude::*;
use serde::{Deserialize, Serialize};

/// Service for calculating quality modifiers from field test telemetry
#[derive(Clone, Default, DeriveService)]
#[service(name = "field_telemetry")]
pub struct FieldTelemetryService;

impl FieldTelemetryService {
    /// Calculate quality modifier based on reliability metrics
    ///
    /// Clamps reliability to a reasonable range (0.2 to 1.2) to prevent
    /// extreme values from breaking game balance.
    pub fn quality_modifier(&self, reliability: f32) -> f32 {
        reliability.clamp(0.2, 1.2)
    }
}

/// UI display state for prototype research and field testing
///
/// This resource tracks:
/// - Recently queued research projects (for UI display)
/// - Recent field test feedback reports (effectiveness/reliability metrics)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrototypeBacklog {
    /// Recently queued research projects (max 6, newest first)
    ///
    /// Format: "{prototype_name} +{budget}c"
    pub queued: Vec<String>,

    /// Recent field test reports (max 6, newest first)
    ///
    /// Format: "{prototype_name} eff {effectiveness}% / rel {reliability}%"
    pub field_reports: Vec<String>,
}
