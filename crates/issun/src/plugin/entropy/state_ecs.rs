//! ECS-based state for EntropyPlugin
//!
//! Uses `hecs::World` for high-performance entity management.

use crate::state::State;
use super::types::*;
use serde::{Deserialize, Serialize};

/// Decay event for ECS version (uses hecs::Entity)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecayEventECS {
    /// Entity that decayed
    #[serde(skip)]  // hecs::Entity is not serializable
    pub entity: Option<hecs::Entity>,
    /// Old durability value
    pub old_durability: f32,
    /// New durability value
    pub new_durability: f32,
    /// Decay amount applied
    pub decay_amount: f32,
    /// Timestamp of event
    pub timestamp: std::time::SystemTime,
    /// Whether status changed
    pub status_changed: bool,
}

/// ECS-based entropy state
pub struct EntropyStateECS {
    /// hecs World containing all entities with durability
    pub world: hecs::World,

    /// Queue of destroyed entities (pending removal)
    pub destroyed_queue: Vec<hecs::Entity>,

    /// Decay event history (for debugging and stats)
    pub decay_events: Vec<DecayEventECS>,

    /// Performance metrics
    pub metrics: EntropyMetrics,
}

impl Default for EntropyStateECS {
    fn default() -> Self {
        Self {
            world: hecs::World::new(),
            destroyed_queue: Vec::new(),
            decay_events: Vec::new(),
            metrics: EntropyMetrics::default(),
        }
    }
}

impl Clone for EntropyStateECS {
    fn clone(&self) -> Self {
        // Note: hecs::World doesn't implement Clone, so we create a new empty world
        // For runtime state, cloning should create a fresh instance
        Self::default()
    }
}

impl State for EntropyStateECS {}

impl EntropyStateECS {
    /// Create new ECS state
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn a new entity with durability and environmental exposure
    pub fn spawn_entity(
        &mut self,
        durability: Durability,
        environment: EnvironmentalExposure,
    ) -> hecs::Entity {
        self.world.spawn((
            durability,
            environment,
            MaintenanceHistory::default(),
            EntityTimestamp::new(),
        ))
    }

    /// Spawn entity with full component set
    pub fn spawn_entity_full(
        &mut self,
        durability: Durability,
        environment: EnvironmentalExposure,
        maintenance: MaintenanceHistory,
        timestamp: EntityTimestamp,
    ) -> hecs::Entity {
        self.world.spawn((
            durability,
            environment,
            maintenance,
            timestamp,
        ))
    }

    /// Despawn an entity
    pub fn despawn(&mut self, entity: hecs::Entity) -> Result<(), hecs::ComponentError> {
        self.world.despawn(entity)?;
        Ok(())
    }

    /// Get total entity count
    pub fn entity_count(&self) -> usize {
        self.world.len() as usize
    }

    /// Clear destroyed queue
    pub fn clear_destroyed_queue(&mut self) {
        self.destroyed_queue.clear();
    }

    /// Cleanup destroyed entities
    pub fn cleanup_destroyed(&mut self) {
        let entities: Vec<_> = self.destroyed_queue.drain(..).collect();
        for entity in entities {
            let _ = self.despawn(entity);
        }
    }

    /// Get reference to metrics
    pub fn metrics(&self) -> &EntropyMetrics {
        &self.metrics
    }

    /// Get mutable reference to metrics
    pub fn metrics_mut(&mut self) -> &mut EntropyMetrics {
        &mut self.metrics
    }

    /// Trim decay events to max size
    pub fn trim_decay_events(&mut self, max_events: usize) {
        if self.decay_events.len() > max_events {
            let excess = self.decay_events.len() - max_events;
            self.decay_events.drain(0..excess);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_entity() {
        let mut state = EntropyStateECS::new();

        let durability = Durability::new(100.0, 0.01, MaterialType::Metal);
        let environment = EnvironmentalExposure::default();

        let entity = state.spawn_entity(durability.clone(), environment);

        assert_eq!(state.entity_count(), 1);

        // Verify components
        let dur = state.world.get::<&Durability>(entity).unwrap();
        assert_eq!(dur.max, 100.0);
        assert_eq!(dur.material, MaterialType::Metal);
    }

    #[test]
    fn test_despawn_entity() {
        let mut state = EntropyStateECS::new();

        let durability = Durability::new(100.0, 0.01, MaterialType::Organic);
        let environment = EnvironmentalExposure::default();

        let entity = state.spawn_entity(durability, environment);
        assert_eq!(state.entity_count(), 1);

        state.despawn(entity).unwrap();
        assert_eq!(state.entity_count(), 0);
    }

    #[test]
    fn test_destroyed_queue() {
        let mut state = EntropyStateECS::new();

        let entity1 = state.spawn_entity(
            Durability::new(100.0, 0.01, MaterialType::Metal),
            EnvironmentalExposure::default(),
        );

        let entity2 = state.spawn_entity(
            Durability::new(100.0, 0.01, MaterialType::Organic),
            EnvironmentalExposure::default(),
        );

        state.destroyed_queue.push(entity1);
        state.destroyed_queue.push(entity2);

        assert_eq!(state.destroyed_queue.len(), 2);
        assert_eq!(state.entity_count(), 2);

        state.cleanup_destroyed();

        assert_eq!(state.destroyed_queue.len(), 0);
        assert_eq!(state.entity_count(), 0);
    }

    #[test]
    fn test_trim_decay_events() {
        let mut state = EntropyStateECS::new();

        // Add 100 events
        for i in 0..100 {
            state.decay_events.push(DecayEventECS {
                entity: None,
                old_durability: 100.0,
                new_durability: 99.0 - i as f32,
                decay_amount: 1.0,
                timestamp: std::time::SystemTime::now(),
                status_changed: false,
            });
        }

        assert_eq!(state.decay_events.len(), 100);

        state.trim_decay_events(50);

        assert_eq!(state.decay_events.len(), 50);
        // Oldest events should be removed, newest kept
        assert_eq!(state.decay_events[0].new_durability, 49.0);
    }
}
