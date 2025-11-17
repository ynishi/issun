//! Service system for ISSUN
//!
//! Services provide reusable game systems and utilities

use crate::context::Context;
use async_trait::async_trait;
use std::any::Any;

/// Service trait for game systems
///
/// Services are reusable systems that provide functionality to the game.
/// Examples: DamageCalculator, PathFinder, DialogueSystem, QuestManager
#[async_trait]
pub trait Service: Send + Sync + 'static {
    /// Service name (must be unique)
    fn name(&self) -> &'static str;

    /// Clone this service as a boxed trait object
    ///
    /// Services are expected to be lightweight and typically `Clone`.
    /// This enables the builder to duplicate services across contexts.
    fn clone_box(&self) -> Box<dyn Service>;

    /// Initialize service (called once at startup)
    async fn initialize(&mut self, _ctx: &mut Context) {}

    /// Optional: Called each frame if service needs to update
    async fn update(&mut self, _ctx: &mut Context) {}

    /// Optional: Shutdown cleanup
    async fn shutdown(&mut self, _ctx: &mut Context) {}

    /// Downcast to Any for type-safe access
    fn as_any(&self) -> &dyn Any;

    /// Downcast to Any for type-safe mutable access
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Example: Damage calculation service
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct DamageCalculatorService {
        critical_multiplier: f32,
    }

    impl DamageCalculatorService {
        fn new() -> Self {
            Self {
                critical_multiplier: 2.0,
            }
        }

        fn calculate_damage(&self, base_damage: i32, is_critical: bool) -> i32 {
            if is_critical {
                (base_damage as f32 * self.critical_multiplier) as i32
            } else {
                base_damage
            }
        }
    }

    #[async_trait]
    impl Service for DamageCalculatorService {
        fn name(&self) -> &'static str {
            "damage_calculator"
        }

        fn clone_box(&self) -> Box<dyn Service> {
            Box::new(self.clone())
        }

        async fn initialize(&mut self, _ctx: &mut Context) {
            // Load configuration, etc.
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[tokio::test]
    async fn test_service_creation() {
        let service = DamageCalculatorService::new();
        assert_eq!(service.name(), "damage_calculator");
    }

    #[tokio::test]
    async fn test_service_damage_calculation() {
        let service = DamageCalculatorService::new();
        assert_eq!(service.calculate_damage(100, false), 100);
        assert_eq!(service.calculate_damage(100, true), 200);
    }

    #[tokio::test]
    async fn test_service_initialize() {
        let mut service = DamageCalculatorService::new();
        let mut ctx = Context::new();
        service.initialize(&mut ctx).await;
        // Initialize should complete without error
    }
}
