//! ECS state management for generation system

use super::types::*;
use crate::state::State;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// ECS state for generation system
#[derive(Default)]
pub struct GenerationStateECS {
    /// ECS world containing all generation entities
    pub world: hecs::World,
    /// History of generation events
    pub generation_events: Vec<GenerationEventECS>,
    /// Queue of completed entities to be removed
    pub completed_queue: Vec<hecs::Entity>,
    /// Performance metrics
    pub metrics: GenerationMetrics,
}

/// Generation event in ECS
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationEventECS {
    /// Entity that generated
    #[serde(skip)] // hecs::Entity is not serializable
    pub entity: Option<hecs::Entity>,
    /// Old generation value
    pub old_generation: f32,
    /// New generation value
    pub new_generation: f32,
    /// Progress amount added
    pub progress_amount: f32,
    /// Timestamp of event
    pub timestamp: SystemTime,
    /// Whether status changed
    pub status_changed: bool,
}

impl Clone for GenerationStateECS {
    fn clone(&self) -> Self {
        // Note: hecs::World doesn't implement Clone, so we create a new empty world
        // For runtime state, cloning should create a fresh instance
        Self::default()
    }
}

impl State for GenerationStateECS {}

impl GenerationStateECS {
    /// Create new ECS state
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn a new generation entity
    ///
    /// # Arguments
    /// * `generation` - Generation component
    /// * `environment` - Environmental factors
    ///
    /// # Returns
    /// Entity handle
    pub fn spawn_entity(
        &mut self,
        generation: Generation,
        environment: GenerationEnvironment,
    ) -> hecs::Entity {
        self.world.spawn((
            generation,
            environment,
            GenerationConditions::default(),
            GenerationHistory::default(),
            EntityTimestamp::new(),
        ))
    }

    /// Spawn entity with custom conditions
    pub fn spawn_entity_with_conditions(
        &mut self,
        generation: Generation,
        environment: GenerationEnvironment,
        conditions: GenerationConditions,
    ) -> hecs::Entity {
        self.world.spawn((
            generation,
            environment,
            conditions,
            GenerationHistory::default(),
            EntityTimestamp::new(),
        ))
    }

    /// Get entity count
    pub fn entity_count(&self) -> usize {
        self.world.len() as usize
    }

    /// Cleanup completed entities
    pub fn cleanup_completed(&mut self) {
        for entity in self.completed_queue.drain(..) {
            let _ = self.world.despawn(entity);
        }
    }

    /// Trim generation events to max count
    pub fn trim_generation_events(&mut self, max_events: usize) {
        if self.generation_events.len() > max_events {
            let remove_count = self.generation_events.len() - max_events;
            self.generation_events.drain(0..remove_count);
        }
    }

    /// Get metrics
    pub fn metrics(&self) -> &GenerationMetrics {
        &self.metrics
    }

    /// Get recent generation events
    pub fn recent_events(&self, count: usize) -> &[GenerationEventECS] {
        let start = self.generation_events.len().saturating_sub(count);
        &self.generation_events[start..]
    }

    /// Query entities with specific generation status
    pub fn entities_with_status(&self, status: GenerationStatus) -> Vec<hecs::Entity> {
        self.world
            .query::<&Generation>()
            .iter()
            .filter_map(|(entity, generation)| {
                if generation.status == status {
                    Some(entity)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Query paused entities
    pub fn paused_entities(&self) -> Vec<hecs::Entity> {
        self.world
            .query::<&Generation>()
            .iter()
            .filter_map(|(entity, generation)| {
                if generation.paused {
                    Some(entity)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_entity() {
        let mut state = GenerationStateECS::new();

        let entity = state.spawn_entity(
            Generation::new(100.0, 1.0, GenerationType::Organic),
            GenerationEnvironment::default(),
        );

        assert_eq!(state.entity_count(), 1);

        // Verify components
        let generation = state.world.get::<&Generation>(entity).unwrap();
        assert_eq!(generation.max, 100.0);
    }

    #[test]
    fn test_spawn_with_conditions() {
        let mut state = GenerationStateECS::new();

        let conditions = GenerationConditions::new().with_resource("wood".to_string(), 10);

        let entity = state.spawn_entity_with_conditions(
            Generation::new(100.0, 1.0, GenerationType::Construction),
            GenerationEnvironment::default(),
            conditions,
        );

        let cond = state.world.get::<&GenerationConditions>(entity).unwrap();
        assert_eq!(cond.required_resources.len(), 1);
    }

    #[test]
    fn test_cleanup_completed() {
        let mut state = GenerationStateECS::new();

        let entity1 = state.spawn_entity(
            Generation::new(100.0, 1.0, GenerationType::Production),
            GenerationEnvironment::default(),
        );

        let entity2 = state.spawn_entity(
            Generation::new(100.0, 1.0, GenerationType::Recovery),
            GenerationEnvironment::default(),
        );

        assert_eq!(state.entity_count(), 2);

        // Mark entity1 for removal
        state.completed_queue.push(entity1);

        state.cleanup_completed();

        assert_eq!(state.entity_count(), 1);
        assert!(state.world.get::<&Generation>(entity1).is_err());
        assert!(state.world.get::<&Generation>(entity2).is_ok());
    }

    #[test]
    fn test_trim_events() {
        let mut state = GenerationStateECS::new();

        // Add 10 events
        for i in 0..10 {
            state.generation_events.push(GenerationEventECS {
                entity: None,
                old_generation: i as f32,
                new_generation: (i + 1) as f32,
                progress_amount: 1.0,
                timestamp: SystemTime::now(),
                status_changed: false,
            });
        }

        assert_eq!(state.generation_events.len(), 10);

        // Trim to 5
        state.trim_generation_events(5);

        assert_eq!(state.generation_events.len(), 5);
        assert_eq!(state.generation_events[0].old_generation, 5.0);
    }

    #[test]
    fn test_entities_with_status() {
        let mut state = GenerationStateECS::new();

        state.spawn_entity(
            Generation::with_current(10.0, 100.0, 1.0, GenerationType::Organic),
            GenerationEnvironment::default(),
        );

        state.spawn_entity(
            Generation::with_current(50.0, 100.0, 1.0, GenerationType::Organic),
            GenerationEnvironment::default(),
        );

        state.spawn_entity(
            Generation::with_current(100.0, 100.0, 1.0, GenerationType::Organic),
            GenerationEnvironment::default(),
        );

        let seed_entities = state.entities_with_status(GenerationStatus::Seed);
        assert_eq!(seed_entities.len(), 1);

        let generating_entities = state.entities_with_status(GenerationStatus::Generating);
        assert_eq!(generating_entities.len(), 1);

        let completed_entities = state.entities_with_status(GenerationStatus::Completed);
        assert_eq!(completed_entities.len(), 1);
    }

    #[test]
    fn test_paused_entities() {
        let mut state = GenerationStateECS::new();

        let entity1 = state.spawn_entity(
            Generation::new(100.0, 1.0, GenerationType::Construction),
            GenerationEnvironment::default(),
        );

        let mut gen = Generation::new(100.0, 1.0, GenerationType::Production);
        gen.pause();
        state.spawn_entity(gen, GenerationEnvironment::default());

        let paused = state.paused_entities();
        assert_eq!(paused.len(), 1);

        // Verify entity1 is not paused
        assert!(!paused.contains(&entity1));
    }
}
