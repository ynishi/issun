//! Core types for EntropyPlugin
//!
//! These types are shared between Simple and ECS implementations.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Material type affects environmental decay rates
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaterialType {
    /// Organic matter (food, wood) - fastest decay
    Organic,
    /// Metal - rust and oxidation
    Metal,
    /// Plastic - slow degradation
    Plastic,
    /// Stone - very slow erosion
    Stone,
    /// Electronics - environmental sensitivity
    Electronic,
    /// Custom material type
    Custom(String),
}

/// Durability status based on current ratio
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DurabilityStatus {
    /// 80-100% - Normal condition
    Intact,
    /// 50-80% - Showing wear
    Worn,
    /// 20-50% - Significant damage
    Damaged,
    /// 0-20% - Critical condition
    Critical,
    /// 0% - Completely destroyed
    Destroyed,
}

/// Durability component - tracks entity degradation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Durability {
    /// Current durability value (0.0 = destroyed, max = perfect)
    pub current: f32,
    /// Maximum durability value
    pub max: f32,
    /// Base decay rate per tick
    pub decay_rate: f32,
    /// Material type (affects environmental modifiers)
    pub material: MaterialType,
    /// Current status (derived from current/max ratio)
    pub status: DurabilityStatus,
}

impl Durability {
    /// Create new durability with max value
    pub fn new(max: f32, decay_rate: f32, material: MaterialType) -> Self {
        Self {
            current: max,
            max,
            decay_rate,
            material,
            status: DurabilityStatus::Intact,
        }
    }

    /// Get current durability ratio (0.0-1.0)
    pub fn current_ratio(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            (self.current / self.max).clamp(0.0, 1.0)
        }
    }

    /// Update status based on current ratio
    pub fn update_status(&mut self) {
        let ratio = self.current_ratio();
        self.status = if ratio >= 0.8 {
            DurabilityStatus::Intact
        } else if ratio >= 0.5 {
            DurabilityStatus::Worn
        } else if ratio >= 0.2 {
            DurabilityStatus::Damaged
        } else if ratio > 0.0 {
            DurabilityStatus::Critical
        } else {
            DurabilityStatus::Destroyed
        };
    }

    /// Check if entity is destroyed
    pub fn is_destroyed(&self) -> bool {
        self.status == DurabilityStatus::Destroyed
    }
}

/// Environmental exposure component - affects decay rates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentalExposure {
    /// Humidity level (0.0-1.0)
    pub humidity: f32,
    /// Pollution level (0.0-1.0)
    pub pollution: f32,
    /// Temperature in Celsius
    pub temperature: f32,
    /// Sunlight exposure (0.0-1.0)
    pub sunlight_exposure: f32,
}

impl Default for EnvironmentalExposure {
    fn default() -> Self {
        Self {
            humidity: 0.5,
            pollution: 0.0,
            temperature: 20.0,
            sunlight_exposure: 0.5,
        }
    }
}

impl EnvironmentalExposure {
    /// Create new environmental exposure with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom values
    pub fn with_values(humidity: f32, pollution: f32, temperature: f32, sunlight: f32) -> Self {
        Self {
            humidity: humidity.clamp(0.0, 1.0),
            pollution: pollution.clamp(0.0, 1.0),
            temperature,
            sunlight_exposure: sunlight.clamp(0.0, 1.0),
        }
    }
}

/// Maintenance history component - tracks repairs
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MaintenanceHistory {
    /// Last maintenance timestamp
    pub last_maintained: Option<SystemTime>,
    /// Total number of maintenance operations
    pub maintenance_count: u32,
    /// Total repair cost accumulated
    pub total_repair_cost: f32,
}

/// Entity timestamp component - creation and update times
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityTimestamp {
    /// Creation time
    pub created_at: SystemTime,
    /// Last update time
    pub last_updated: SystemTime,
}

impl EntityTimestamp {
    /// Create new timestamp with current time
    pub fn new() -> Self {
        let now = SystemTime::now();
        Self {
            created_at: now,
            last_updated: now,
        }
    }

    /// Update last_updated to current time
    pub fn touch(&mut self) {
        self.last_updated = SystemTime::now();
    }
}

impl Default for EntityTimestamp {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of durability change operation
#[derive(Clone, Debug)]
pub struct DurabilityChange {
    /// Old durability value
    pub old_value: f32,
    /// New durability value
    pub new_value: f32,
    /// Amount of decay applied
    pub decay_amount: f32,
    /// Whether status changed
    pub status_changed: bool,
    /// Whether entity was destroyed
    pub destroyed: bool,
}

/// Performance metrics for entropy system
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct EntropyMetrics {
    /// Total entities processed
    pub entities_processed: usize,
    /// Total entities destroyed
    pub entities_destroyed: usize,
    /// Last update duration in microseconds
    pub last_update_duration_us: u64,
    /// Total decay applied
    pub total_decay_applied: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_durability_ratio() {
        let dur = Durability::new(100.0, 0.01, MaterialType::Metal);
        assert_eq!(dur.current_ratio(), 1.0);

        let mut dur = Durability {
            current: 50.0,
            max: 100.0,
            decay_rate: 0.01,
            material: MaterialType::Metal,
            status: DurabilityStatus::Intact,
        };
        assert_eq!(dur.current_ratio(), 0.5);
        dur.update_status();
        assert_eq!(dur.status, DurabilityStatus::Worn);
    }

    #[test]
    fn test_status_transitions() {
        let mut dur = Durability::new(100.0, 0.01, MaterialType::Organic);

        dur.current = 85.0;
        dur.update_status();
        assert_eq!(dur.status, DurabilityStatus::Intact);

        dur.current = 60.0;
        dur.update_status();
        assert_eq!(dur.status, DurabilityStatus::Worn);

        dur.current = 30.0;
        dur.update_status();
        assert_eq!(dur.status, DurabilityStatus::Damaged);

        dur.current = 10.0;
        dur.update_status();
        assert_eq!(dur.status, DurabilityStatus::Critical);

        dur.current = 0.0;
        dur.update_status();
        assert_eq!(dur.status, DurabilityStatus::Destroyed);
        assert!(dur.is_destroyed());
    }

    #[test]
    fn test_environmental_exposure_clamping() {
        let env = EnvironmentalExposure::with_values(1.5, -0.5, 25.0, 0.8);
        assert_eq!(env.humidity, 1.0);
        assert_eq!(env.pollution, 0.0);
        assert_eq!(env.sunlight_exposure, 0.8);
    }
}
