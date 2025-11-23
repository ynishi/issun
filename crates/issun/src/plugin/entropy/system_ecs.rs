//! ECS-based system for entropy processing
//!
//! Uses parallel iteration for high-performance decay calculations.

use super::config::EntropyConfig;
use super::hook_ecs::EntropyHookECS;
use super::service::EntropyService;
use super::state_ecs::{DecayEventECS, EntropyStateECS};
use super::types::*;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;
use std::time::{Instant, SystemTime};

/// ECS-based entropy system
#[derive(Clone)]
pub struct EntropySystemECS {
    hook: Arc<dyn EntropyHookECS>,
    service: EntropyService,
}

#[async_trait]
impl System for EntropySystemECS {
    fn name(&self) -> &'static str {
        "issun:entropy_system_ecs"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl EntropySystemECS {
    /// Create new ECS system with hook
    pub fn new(hook: Arc<dyn EntropyHookECS>) -> Self {
        Self {
            hook,
            service: EntropyService,
        }
    }

    /// Update all entities with parallel decay processing
    pub async fn update_decay(
        &mut self,
        state: &mut EntropyStateECS,
        config: &EntropyConfig,
        delta_time: f32,
    ) {
        use rayon::prelude::*;

        let start = Instant::now();
        let mut processed = 0;
        let mut total_decay = 0.0;

        // Collect changes in parallel
        let changes: Vec<_> = state
            .world
            .query_mut::<(&mut Durability, &EnvironmentalExposure, &mut EntityTimestamp)>()
            .into_iter()
            .par_bridge() // ← Parallel iteration
            .map(|(entity, (durability, environment, timestamp))| {
                // Calculate decay
                let decay_amount = EntropyService::calculate_decay(
                    durability.decay_rate,
                    &durability.material,
                    environment,
                    &config.environment_modifiers,
                    config.global_decay_multiplier,
                    delta_time,
                );

                // Apply decay
                let old_value = durability.current;
                let old_status = durability.status.clone();

                durability.current = (durability.current - decay_amount).max(0.0);
                durability.update_status();

                // Update timestamp
                timestamp.last_updated = SystemTime::now();

                let status_changed = old_status != durability.status;
                let destroyed = durability.is_destroyed();

                (
                    entity,
                    old_value,
                    durability.current,
                    decay_amount,
                    status_changed,
                    destroyed,
                )
            })
            .collect();

        // Process results sequentially (event recording, hook calls)
        for (entity, old_value, new_value, decay_amount, status_changed, destroyed) in changes {
            processed += 1;
            total_decay += decay_amount;

            // Record event if status changed
            if status_changed {
                state.decay_events.push(DecayEventECS {
                    entity: Some(entity),
                    old_durability: old_value,
                    new_durability: new_value,
                    decay_amount,
                    timestamp: SystemTime::now(),
                    status_changed: true,
                });

                // Call hook
                self.hook
                    .on_durability_status_changed(entity, new_value)
                    .await;
            }

            // Handle destruction
            if destroyed {
                self.hook.on_entity_destroyed(entity, state).await;

                if config.auto_destroy_on_zero {
                    state.destroyed_queue.push(entity);
                }
            }
        }

        // Trim events if needed
        state.trim_decay_events(config.max_decay_events);

        // Update metrics
        state.metrics.entities_processed = processed;
        state.metrics.entities_destroyed += state.destroyed_queue.len();
        state.metrics.total_decay_applied = total_decay;
        state.metrics.last_update_duration_us = start.elapsed().as_micros() as u64;
    }

    /// Cleanup destroyed entities
    pub fn cleanup_destroyed(&self, state: &mut EntropyStateECS) {
        state.cleanup_destroyed();
    }

    /// Repair a specific entity
    pub async fn repair_entity(
        &mut self,
        entity: hecs::Entity,
        repair_amount: f32,
        state: &mut EntropyStateECS,
    ) -> Result<f32, String> {
        // Get components
        let mut durability = state
            .world
            .get::<&mut Durability>(entity)
            .map_err(|_| "Entity not found or missing Durability component")?;

        let mut maintenance = state
            .world
            .get::<&mut MaintenanceHistory>(entity)
            .map_err(|_| "Entity missing MaintenanceHistory component")?;

        // Repair
        let repaired = EntropyService::repair(&mut durability, repair_amount);

        // Update maintenance history
        maintenance.last_maintained = Some(SystemTime::now());
        maintenance.maintenance_count += 1;

        // Calculate cost via hook
        let cost = self.hook.calculate_repair_cost(entity, repaired).await;
        maintenance.total_repair_cost += cost;

        Ok(repaired)
    }

    /// Get current metrics
    pub fn metrics<'a>(&self, state: &'a EntropyStateECS) -> &'a EntropyMetrics {
        state.metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::hook_ecs::DefaultEntropyHookECS;

    #[tokio::test]
    async fn test_update_decay_basic() {
        let hook = Arc::new(DefaultEntropyHookECS);
        let mut system = EntropySystemECS::new(hook);
        let mut state = EntropyStateECS::new();
        let config = EntropyConfig::default();

        // Spawn entity
        let entity = state.spawn_entity(
            Durability::new(100.0, 0.01, MaterialType::Metal),
            EnvironmentalExposure::default(),
        );

        // Update decay
        system.update_decay(&mut state, &config, 1.0).await;

        // Check durability decreased
        let durability = state.world.get::<&Durability>(entity).unwrap();
        assert!(durability.current < 100.0);
        assert_eq!(state.metrics.entities_processed, 1);
    }

    #[tokio::test]
    async fn test_update_decay_multiple_entities() {
        let hook = Arc::new(DefaultEntropyHookECS);
        let mut system = EntropySystemECS::new(hook);
        let mut state = EntropyStateECS::new();
        let config = EntropyConfig::default();

        // Spawn 100 entities
        for _ in 0..100 {
            state.spawn_entity(
                Durability::new(100.0, 0.01, MaterialType::Organic),
                EnvironmentalExposure::default(),
            );
        }

        // Update decay
        system.update_decay(&mut state, &config, 1.0).await;

        assert_eq!(state.metrics.entities_processed, 100);
    }

    #[tokio::test]
    async fn test_auto_destroy() {
        let hook = Arc::new(DefaultEntropyHookECS);
        let mut system = EntropySystemECS::new(hook);
        let mut state = EntropyStateECS::new();
        let config = EntropyConfig {
            auto_destroy_on_zero: true,
            ..Default::default()
        };

        // Spawn entity with 0 durability
        let entity = state.spawn_entity(
            Durability {
                current: 0.0,
                max: 100.0,
                decay_rate: 0.0,
                material: MaterialType::Metal,
                status: DurabilityStatus::Destroyed,
            },
            EnvironmentalExposure::default(),
        );

        assert_eq!(state.entity_count(), 1);

        // Update decay
        system.update_decay(&mut state, &config, 1.0).await;

        assert_eq!(state.destroyed_queue.len(), 1);

        // Cleanup
        system.cleanup_destroyed(&mut state);

        assert_eq!(state.entity_count(), 0);
    }

    #[tokio::test]
    async fn test_repair_entity() {
        let hook = Arc::new(DefaultEntropyHookECS);
        let mut system = EntropySystemECS::new(hook);
        let mut state = EntropyStateECS::new();

        // Spawn damaged entity
        let entity = state.spawn_entity(
            Durability {
                current: 50.0,
                max: 100.0,
                decay_rate: 0.01,
                material: MaterialType::Metal,
                status: DurabilityStatus::Worn,
            },
            EnvironmentalExposure::default(),
        );

        // Repair
        let repaired = system.repair_entity(entity, 30.0, &mut state).await.unwrap();

        assert_eq!(repaired, 30.0);

        let durability = state.world.get::<&Durability>(entity).unwrap();
        assert_eq!(durability.current, 80.0);

        let maintenance = state
            .world
            .get::<&MaintenanceHistory>(entity)
            .unwrap();
        assert_eq!(maintenance.maintenance_count, 1);
    }

    #[tokio::test]
    async fn test_parallel_performance() {
        let hook = Arc::new(DefaultEntropyHookECS);
        let mut system = EntropySystemECS::new(hook);
        let mut state = EntropyStateECS::new();
        let config = EntropyConfig::default();

        // Spawn 10,000 entities
        for _ in 0..10_000 {
            state.spawn_entity(
                Durability::new(100.0, 0.01, MaterialType::Metal),
                EnvironmentalExposure::default(),
            );
        }

        let start = Instant::now();
        system.update_decay(&mut state, &config, 1.0).await;
        let elapsed = start.elapsed();

        println!("10,000 entities processed in {:?}", elapsed);
        assert_eq!(state.metrics.entities_processed, 10_000);
        // Should be fast due to parallel processing
        assert!(elapsed.as_millis() < 1000); // Less than 1 second
    }
}
