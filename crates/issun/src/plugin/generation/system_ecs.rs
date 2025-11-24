//! ECS-based system for generation processing
//!
//! Uses parallel iteration for high-performance generation calculations.

use super::config::GenerationConfig;
use super::hook_ecs::GenerationHookECS;
use super::service::GenerationService;
use super::state_ecs::{GenerationEventECS, GenerationStateECS};
use super::types::*;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;
use std::time::{Instant, SystemTime};

/// ECS-based generation system
#[derive(Clone)]
#[allow(dead_code)]
pub struct GenerationSystemECS {
    hook: Arc<dyn GenerationHookECS>,
}

#[async_trait]
impl System for GenerationSystemECS {
    fn name(&self) -> &'static str {
        "issun:generation_system_ecs"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl GenerationSystemECS {
    /// Create new ECS system with hook
    pub fn new(hook: Arc<dyn GenerationHookECS>) -> Self {
        Self { hook }
    }

    /// Update all entities with parallel generation processing
    pub async fn update_generation(
        &mut self,
        state: &mut GenerationStateECS,
        config: &GenerationConfig,
        delta_time: f32,
    ) {
        use rayon::prelude::*;

        let start = Instant::now();
        let mut processed = 0;
        let mut total_progress = 0.0;

        // Collect changes in parallel
        let changes: Vec<_> = state
            .world
            .query_mut::<(
                &mut Generation,
                &GenerationEnvironment,
                &GenerationConditions,
                &mut EntityTimestamp,
            )>()
            .into_iter()
            .par_bridge() // ← Parallel iteration
            .filter_map(
                |(entity, (generation, environment, conditions, timestamp))| {
                    // Level 1: Skip paused entities
                    if generation.paused {
                        return None;
                    }

                    // Level 2: Check conditions
                    if !GenerationService::check_conditions(
                        conditions,
                        environment.temperature,
                        &[], // Resource check would be done via hook in real game
                    ) {
                        return None;
                    }

                    // Get environment modifiers for this type
                    let modifiers = config
                        .environment_modifiers
                        .get(&generation.generation_type)
                        .cloned()
                        .unwrap_or(super::config::EnvironmentModifiers {
                            temperature_factor: 0.0,
                            fertility_factor: 0.0,
                            resource_factor: 0.0,
                            light_factor: 0.0,
                        });

                    // Calculate generation
                    let progress_amount = GenerationService::calculate_generation(
                        generation.generation_rate,
                        &generation.generation_type,
                        environment,
                        &modifiers,
                        config.global_generation_multiplier,
                        delta_time,
                    );

                    // Apply generation
                    let old_value = generation.current;
                    let old_status = generation.status.clone();

                    generation.current = (generation.current + progress_amount).min(generation.max);
                    generation.update_status();

                    // Update timestamp
                    timestamp.last_updated = SystemTime::now();

                    let status_changed = old_status != generation.status;
                    let completed = generation.is_completed();

                    Some((
                        entity,
                        old_value,
                        generation.current,
                        progress_amount,
                        status_changed,
                        completed,
                    ))
                },
            )
            .collect();

        // Process results sequentially (event recording, hook calls)
        for (entity, old_value, new_value, progress_amount, status_changed, completed) in changes {
            processed += 1;
            total_progress += progress_amount;

            // Record event if status changed
            if status_changed {
                state.generation_events.push(GenerationEventECS {
                    entity: Some(entity),
                    old_generation: old_value,
                    new_generation: new_value,
                    progress_amount,
                    timestamp: SystemTime::now(),
                    status_changed: true,
                });

                // Call hook
                self.hook
                    .on_generation_status_changed(entity, new_value)
                    .await;
            }

            // Handle completion
            if completed {
                self.hook.on_generation_completed(entity, state).await;

                if config.auto_remove_on_complete {
                    state.completed_queue.push(entity);
                }
            }
        }

        // Trim events if needed
        state.trim_generation_events(config.max_generation_events);

        // Update metrics
        state.metrics.entities_processed = processed;
        state.metrics.entities_completed += state.completed_queue.len();
        state.metrics.total_progress_applied = total_progress;
        state.metrics.last_update_duration_us = start.elapsed().as_micros() as u64;
    }

    /// Cleanup completed entities
    pub fn cleanup_completed(&self, state: &mut GenerationStateECS) {
        state.cleanup_completed();
    }

    /// Reduce generation for a specific entity (damage/setback)
    pub async fn reduce_entity(
        &mut self,
        entity: hecs::Entity,
        reduction_amount: f32,
        state: &mut GenerationStateECS,
    ) -> Result<f32, String> {
        // Get components
        let mut generation = state
            .world
            .get::<&mut Generation>(entity)
            .map_err(|_| "Entity not found or missing Generation component")?;

        let mut history = state
            .world
            .get::<&mut GenerationHistory>(entity)
            .map_err(|_| "Entity missing GenerationHistory component")?;

        // Reduce generation
        let reduced = GenerationService::reduce_generation(&mut generation, reduction_amount);

        // Update history (track setbacks)
        history.total_resources_consumed += reduced; // Track as resource loss

        Ok(reduced)
    }

    /// Pause generation for a specific entity
    pub async fn pause_entity(
        &mut self,
        entity: hecs::Entity,
        state: &mut GenerationStateECS,
    ) -> Result<(), String> {
        let mut generation = state
            .world
            .get::<&mut Generation>(entity)
            .map_err(|_| "Entity not found or missing Generation component")?;

        if !generation.paused {
            generation.pause();
            self.hook.on_generation_paused(entity).await;
        }

        Ok(())
    }

    /// Resume generation for a specific entity
    pub async fn resume_entity(
        &mut self,
        entity: hecs::Entity,
        state: &mut GenerationStateECS,
    ) -> Result<(), String> {
        let mut generation = state
            .world
            .get::<&mut Generation>(entity)
            .map_err(|_| "Entity not found or missing Generation component")?;

        if generation.paused {
            generation.resume();
            self.hook.on_generation_resumed(entity).await;
        }

        Ok(())
    }

    /// Get current metrics
    pub fn metrics<'a>(&self, state: &'a GenerationStateECS) -> &'a GenerationMetrics {
        state.metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::super::hook_ecs::DefaultGenerationHookECS;
    use super::*;

    #[tokio::test]
    async fn test_update_generation_basic() {
        let hook = Arc::new(DefaultGenerationHookECS);
        let mut system = GenerationSystemECS::new(hook);
        let mut state = GenerationStateECS::new();
        let config = GenerationConfig::default();

        // Spawn entity
        let entity = state.spawn_entity(
            Generation::new(100.0, 1.0, GenerationType::Organic),
            GenerationEnvironment::default(),
        );

        // Update generation
        system.update_generation(&mut state, &config, 1.0).await;

        // Check generation increased
        let generation = state.world.get::<&Generation>(entity).unwrap();
        assert!(generation.current > 0.0);
        assert_eq!(state.metrics.entities_processed, 1);
    }

    #[tokio::test]
    async fn test_update_generation_multiple_entities() {
        let hook = Arc::new(DefaultGenerationHookECS);
        let mut system = GenerationSystemECS::new(hook);
        let mut state = GenerationStateECS::new();
        let config = GenerationConfig::default();

        // Spawn 100 entities
        for _ in 0..100 {
            state.spawn_entity(
                Generation::new(100.0, 1.0, GenerationType::Production),
                GenerationEnvironment::default(),
            );
        }

        // Update generation
        system.update_generation(&mut state, &config, 1.0).await;

        assert_eq!(state.metrics.entities_processed, 100);
    }

    #[tokio::test]
    async fn test_paused_entities_skipped() {
        let hook = Arc::new(DefaultGenerationHookECS);
        let mut system = GenerationSystemECS::new(hook);
        let mut state = GenerationStateECS::new();
        let config = GenerationConfig::default();

        // Spawn paused entity
        let mut gen = Generation::new(100.0, 1.0, GenerationType::Construction);
        gen.pause();
        let entity = state.spawn_entity(gen, GenerationEnvironment::default());

        // Update generation
        system.update_generation(&mut state, &config, 1.0).await;

        // Should not process paused entity
        assert_eq!(state.metrics.entities_processed, 0);

        let generation = state.world.get::<&Generation>(entity).unwrap();
        assert_eq!(generation.current, 0.0); // No progress
    }

    #[tokio::test]
    async fn test_auto_remove_on_complete() {
        let hook = Arc::new(DefaultGenerationHookECS);
        let mut system = GenerationSystemECS::new(hook);
        let mut state = GenerationStateECS::new();
        let config = GenerationConfig {
            auto_remove_on_complete: true,
            ..Default::default()
        };

        // Spawn entity already at 100%
        let _entity = state.spawn_entity(
            Generation {
                current: 100.0,
                max: 100.0,
                generation_rate: 0.0,
                generation_type: GenerationType::Recovery,
                status: GenerationStatus::Completed,
                paused: false,
            },
            GenerationEnvironment::default(),
        );

        assert_eq!(state.entity_count(), 1);

        // Update generation
        system.update_generation(&mut state, &config, 1.0).await;

        assert_eq!(state.completed_queue.len(), 1);

        // Cleanup
        system.cleanup_completed(&mut state);

        assert_eq!(state.entity_count(), 0);
    }

    #[tokio::test]
    async fn test_reduce_entity() {
        let hook = Arc::new(DefaultGenerationHookECS);
        let mut system = GenerationSystemECS::new(hook);
        let mut state = GenerationStateECS::new();

        // Spawn entity with 50% progress
        let entity = state.spawn_entity(
            Generation {
                current: 50.0,
                max: 100.0,
                generation_rate: 1.0,
                generation_type: GenerationType::Construction,
                status: GenerationStatus::Generating,
                paused: false,
            },
            GenerationEnvironment::default(),
        );

        // Reduce (damage/setback)
        let reduced = system
            .reduce_entity(entity, 20.0, &mut state)
            .await
            .unwrap();

        assert_eq!(reduced, 20.0);

        let generation = state.world.get::<&Generation>(entity).unwrap();
        assert_eq!(generation.current, 30.0);
    }

    #[tokio::test]
    async fn test_pause_resume() {
        let hook = Arc::new(DefaultGenerationHookECS);
        let mut system = GenerationSystemECS::new(hook);
        let mut state = GenerationStateECS::new();

        let entity = state.spawn_entity(
            Generation::new(100.0, 1.0, GenerationType::Organic),
            GenerationEnvironment::default(),
        );

        // Pause
        system.pause_entity(entity, &mut state).await.unwrap();

        {
            let generation = state.world.get::<&Generation>(entity).unwrap();
            assert!(generation.paused);
        }

        // Resume
        system.resume_entity(entity, &mut state).await.unwrap();

        {
            let generation = state.world.get::<&Generation>(entity).unwrap();
            assert!(!generation.paused);
        }
    }

    #[tokio::test]
    async fn test_parallel_performance() {
        let hook = Arc::new(DefaultGenerationHookECS);
        let mut system = GenerationSystemECS::new(hook);
        let mut state = GenerationStateECS::new();
        let config = GenerationConfig::default();

        // Spawn 10,000 entities
        for _ in 0..10_000 {
            state.spawn_entity(
                Generation::new(100.0, 1.0, GenerationType::Production),
                GenerationEnvironment::default(),
            );
        }

        let start = Instant::now();
        system.update_generation(&mut state, &config, 1.0).await;
        let elapsed = start.elapsed();

        println!("10,000 entities processed in {:?}", elapsed);
        assert_eq!(state.metrics.entities_processed, 10_000);
        // Should be fast due to parallel processing
        assert!(elapsed.as_millis() < 1000); // Less than 1 second
    }
}
