//! Core types for GenerationPlugin
//!
//! Handles growth, construction, production, and recovery systems.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Generation type determines growth behavior and modifiers
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GenerationType {
    /// Biological growth (organisms, populations)
    Organic,
    /// Construction progress (buildings, structures)
    Construction,
    /// Resource production (factories, mines)
    Production,
    /// Recovery and repair (healing, maintenance)
    Recovery,
    /// Custom generation type
    Custom(String),
}

/// Generation status based on current progress ratio
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenerationStatus {
    /// 0-20% - Initial stage (seed, foundation)
    Seed,
    /// 20-60% - Active growth
    Generating,
    /// 60-90% - Maturing
    Maturing,
    /// 90-100% - Nearly complete
    Mature,
    /// 100% - Fully generated
    Completed,
}

/// Generation component - tracks entity growth/construction/production
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Generation {
    /// Current generation progress (0.0 = start, max = complete)
    pub current: f32,
    /// Maximum generation value (completion threshold)
    pub max: f32,
    /// Base generation rate per tick
    pub generation_rate: f32,
    /// Type of generation (affects modifiers)
    pub generation_type: GenerationType,
    /// Current status (derived from current/max ratio)
    pub status: GenerationStatus,
    /// Whether generation is paused
    pub paused: bool,
}

impl Generation {
    /// Create new generation starting at 0
    pub fn new(max: f32, generation_rate: f32, generation_type: GenerationType) -> Self {
        Self {
            current: 0.0,
            max,
            generation_rate,
            generation_type,
            status: GenerationStatus::Seed,
            paused: false,
        }
    }

    /// Create generation with custom starting value
    pub fn with_current(
        current: f32,
        max: f32,
        generation_rate: f32,
        generation_type: GenerationType,
    ) -> Self {
        let mut gen = Self::new(max, generation_rate, generation_type);
        gen.current = current.clamp(0.0, max);
        gen.update_status();
        gen
    }

    /// Get current progress ratio (0.0-1.0)
    pub fn progress_ratio(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            (self.current / self.max).clamp(0.0, 1.0)
        }
    }

    /// Update status based on current ratio
    pub fn update_status(&mut self) {
        let ratio = self.progress_ratio();
        self.status = if ratio >= 1.0 {
            GenerationStatus::Completed
        } else if ratio >= 0.9 {
            GenerationStatus::Mature
        } else if ratio >= 0.6 {
            GenerationStatus::Maturing
        } else if ratio >= 0.2 {
            GenerationStatus::Generating
        } else {
            GenerationStatus::Seed
        };
    }

    /// Check if generation is complete
    pub fn is_completed(&self) -> bool {
        self.status == GenerationStatus::Completed
    }

    /// Pause generation
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume generation
    pub fn resume(&mut self) {
        self.paused = false;
    }
}

/// Generation conditions - resources and requirements for generation
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GenerationConditions {
    /// Required resources per tick (resource_id -> amount)
    pub required_resources: Vec<(String, u32)>,
    /// Minimum temperature for generation
    pub min_temperature: Option<f32>,
    /// Maximum temperature for generation
    pub max_temperature: Option<f32>,
    /// Required building/facility
    pub required_building: Option<String>,
}

impl GenerationConditions {
    /// Create new empty conditions
    pub fn new() -> Self {
        Self::default()
    }

    /// Add required resource
    pub fn with_resource(mut self, resource_id: String, amount: u32) -> Self {
        self.required_resources.push((resource_id, amount));
        self
    }

    /// Set temperature range
    pub fn with_temperature_range(mut self, min: f32, max: f32) -> Self {
        self.min_temperature = Some(min);
        self.max_temperature = Some(max);
        self
    }

    /// Set required building
    pub fn with_building(mut self, building_id: String) -> Self {
        self.required_building = Some(building_id);
        self
    }
}

/// Environmental factors affecting generation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationEnvironment {
    /// Temperature in Celsius
    pub temperature: f32,
    /// Fertility level (0.0-1.0)
    pub fertility: f32,
    /// Resource availability (0.0-1.0)
    pub resource_availability: f32,
    /// Light exposure (0.0-1.0)
    pub light_exposure: f32,
}

impl Default for GenerationEnvironment {
    fn default() -> Self {
        Self {
            temperature: 20.0,
            fertility: 0.7,
            resource_availability: 1.0,
            light_exposure: 0.8,
        }
    }
}

impl GenerationEnvironment {
    /// Create new environment with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom values
    pub fn with_values(temperature: f32, fertility: f32, resources: f32, light: f32) -> Self {
        Self {
            temperature,
            fertility: fertility.clamp(0.0, 1.0),
            resource_availability: resources.clamp(0.0, 1.0),
            light_exposure: light.clamp(0.0, 1.0),
        }
    }
}

/// Generation history component - tracks progress
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GenerationHistory {
    /// Start timestamp
    pub started_at: Option<SystemTime>,
    /// Completion timestamp
    pub completed_at: Option<SystemTime>,
    /// Total generation cycles
    pub cycle_count: u32,
    /// Total resources consumed
    pub total_resources_consumed: f32,
}

/// Entity timestamp component (same as entropy)
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

/// Result of generation operation
#[derive(Clone, Debug)]
pub struct GenerationChange {
    /// Old generation value
    pub old_value: f32,
    /// New generation value
    pub new_value: f32,
    /// Amount of progress added
    pub progress_amount: f32,
    /// Whether status changed
    pub status_changed: bool,
    /// Whether generation completed
    pub completed: bool,
}

/// Performance metrics for generation system
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct GenerationMetrics {
    /// Total entities processed
    pub entities_processed: usize,
    /// Total entities completed
    pub entities_completed: usize,
    /// Last update duration in microseconds
    pub last_update_duration_us: u64,
    /// Total progress applied
    pub total_progress_applied: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_progress_ratio() {
        let gen = Generation::new(100.0, 1.0, GenerationType::Organic);
        assert_eq!(gen.progress_ratio(), 0.0);

        let mut gen = Generation::with_current(50.0, 100.0, 1.0, GenerationType::Construction);
        assert_eq!(gen.progress_ratio(), 0.5);
        gen.update_status();
        assert_eq!(gen.status, GenerationStatus::Generating);
    }

    #[test]
    fn test_status_transitions() {
        let mut gen = Generation::new(100.0, 1.0, GenerationType::Production);

        gen.current = 10.0;
        gen.update_status();
        assert_eq!(gen.status, GenerationStatus::Seed);

        gen.current = 30.0;
        gen.update_status();
        assert_eq!(gen.status, GenerationStatus::Generating);

        gen.current = 70.0;
        gen.update_status();
        assert_eq!(gen.status, GenerationStatus::Maturing);

        gen.current = 95.0;
        gen.update_status();
        assert_eq!(gen.status, GenerationStatus::Mature);

        gen.current = 100.0;
        gen.update_status();
        assert_eq!(gen.status, GenerationStatus::Completed);
        assert!(gen.is_completed());
    }

    #[test]
    fn test_pause_resume() {
        let mut gen = Generation::new(100.0, 1.0, GenerationType::Organic);
        assert!(!gen.paused);

        gen.pause();
        assert!(gen.paused);

        gen.resume();
        assert!(!gen.paused);
    }

    #[test]
    fn test_generation_conditions_builder() {
        let conditions = GenerationConditions::new()
            .with_resource("wood".to_string(), 10)
            .with_resource("stone".to_string(), 5)
            .with_temperature_range(15.0, 25.0)
            .with_building("farm".to_string());

        assert_eq!(conditions.required_resources.len(), 2);
        assert_eq!(conditions.min_temperature, Some(15.0));
        assert_eq!(conditions.max_temperature, Some(25.0));
        assert_eq!(conditions.required_building, Some("farm".to_string()));
    }

    #[test]
    fn test_environment_clamping() {
        let env = GenerationEnvironment::with_values(25.0, 1.5, -0.5, 0.5);
        assert_eq!(env.fertility, 1.0);
        assert_eq!(env.resource_availability, 0.0);
        assert_eq!(env.light_exposure, 0.5);
    }
}
