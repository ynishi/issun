//! System trait for ISSUN
//!
//! Systems orchestrate game logic using Services and managing state.
//! Examples: CombatEngine, TurnManager, QuestOrchestrator

use async_trait::async_trait;
use crate::context::Context;
use std::any::Any;

/// System trait for Application Logic
///
/// Systems handle:
/// - State management (turn count, score, logs)
/// - Orchestration (coordinating multiple services)
/// - Game loop logic
///
/// # Difference from Service
///
/// - **Service**: Pure functions, stateless (damage calculation, pathfinding)
/// - **System**: Stateful, orchestration (turn management, combat flow)
///
/// # Examples
///
/// ```ignore
/// use issun::prelude::*;
///
/// #[derive(System)]
/// #[system(name = "combat_engine")]
/// pub struct CombatEngine {
///     turn_count: u32,
///     log: Vec<String>,
/// }
///
/// impl CombatEngine {
///     pub fn new() -> Self {
///         Self {
///             turn_count: 0,
///             log: Vec::new(),
///         }
///     }
///
///     pub fn process_turn(&mut self) {
///         self.turn_count += 1;
///         self.log.push(format!("Turn {}", self.turn_count));
///     }
/// }
/// ```
#[async_trait]
pub trait System: Send + Sync + 'static {
    /// System name (must be unique)
    fn name(&self) -> &'static str;

    /// Initialize system (called once at startup)
    async fn initialize(&mut self, _ctx: &mut Context) {}

    /// Optional: Called each frame if system needs to update
    async fn update(&mut self, _ctx: &mut Context) {}

    /// Optional: Shutdown cleanup
    async fn shutdown(&mut self, _ctx: &mut Context) {}

    /// Downcast to Any for type-safe access
    fn as_any(&self) -> &dyn Any;

    /// Downcast to Any for type-safe mutable access
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Example: Turn management system
#[cfg(test)]
mod tests {
    use super::*;

    struct TurnManagerSystem {
        turn_count: u32,
        log: Vec<String>,
    }

    impl TurnManagerSystem {
        fn new() -> Self {
            Self {
                turn_count: 0,
                log: Vec::new(),
            }
        }

        fn next_turn(&mut self) {
            self.turn_count += 1;
            self.log.push(format!("Turn {}", self.turn_count));
        }

        fn get_turn_count(&self) -> u32 {
            self.turn_count
        }
    }

    #[async_trait]
    impl System for TurnManagerSystem {
        fn name(&self) -> &'static str {
            "turn_manager"
        }

        async fn initialize(&mut self, _ctx: &mut Context) {
            self.log.push("Turn manager initialized".to_string());
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[tokio::test]
    async fn test_system_creation() {
        let system = TurnManagerSystem::new();
        assert_eq!(system.name(), "turn_manager");
        assert_eq!(system.get_turn_count(), 0);
    }

    #[tokio::test]
    async fn test_system_turn_management() {
        let mut system = TurnManagerSystem::new();
        system.next_turn();
        system.next_turn();
        assert_eq!(system.get_turn_count(), 2);
        assert_eq!(system.log.len(), 2);
    }

    #[tokio::test]
    async fn test_system_initialize() {
        let mut system = TurnManagerSystem::new();
        let mut ctx = Context::new();
        system.initialize(&mut ctx).await;
        assert!(system.log.contains(&"Turn manager initialized".to_string()));
    }
}
