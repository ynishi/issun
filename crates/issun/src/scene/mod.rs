//! Scene system for ISSUN
//!
//! Scenes represent distinct game states with their own data and lifecycle.

use async_trait::async_trait;

// Sub-modules
pub mod director;

// Re-exports
pub use director::SceneDirector;

/// Scene transition result
///
/// Note: This is a simple enum without data.
/// Actual scene transitions are handled by the game-specific enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneTransition {
    /// Stay in current scene
    Stay,
    /// Request transition to a different scene
    /// (The actual target scene is determined by game logic)
    Transition,
    /// Quit the game
    Quit,
}

/// Scene trait with lifecycle methods
///
/// Scenes represent distinct game states (Title, Combat, etc.)
/// Each scene has its own data that is discarded on transition.
#[async_trait]
pub trait Scene: Send {
    /// Called when entering this scene
    async fn on_enter(&mut self);

    /// Called every frame, returns transition decision
    async fn on_update(&mut self) -> SceneTransition;

    /// Called when leaving this scene
    async fn on_exit(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Scene; // Import derive macro
    use async_trait::async_trait;

    // Test Scene derive macro
    #[derive(Scene)]
    struct TestScene {
        entered: bool,
    }

    #[tokio::test]
    async fn test_derived_scene() {
        let mut scene = TestScene { entered: false };

        // Default on_enter should work
        scene.on_enter().await;

        // Default on_update should return Stay
        let transition = scene.on_update().await;
        assert_eq!(transition, SceneTransition::Stay);

        // Default on_exit should work
        scene.on_exit().await;
    }

    #[derive(Scene)]
    enum GameScene {
        Title,
        Combat,
    }

    #[tokio::test]
    async fn test_derived_enum_scene() {
        let mut scene = GameScene::Title;

        scene.on_enter().await;
        let transition = scene.on_update().await;
        assert_eq!(transition, SceneTransition::Stay);
        scene.on_exit().await;
    }
}
