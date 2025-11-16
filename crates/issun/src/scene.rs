//! Scene system for ISSUN
//!
//! Scenes represent distinct game states with their own data and lifecycle.

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
pub trait Scene {
    /// Called when entering this scene
    fn on_enter(&mut self);

    /// Called every frame, returns transition decision
    fn on_update(&mut self) -> SceneTransition;

    /// Called when leaving this scene
    fn on_exit(&mut self);
}
