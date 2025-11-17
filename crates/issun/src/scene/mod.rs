//! Scene system for ISSUN
//!
//! Scenes represent distinct game states with their own data and lifecycle.

use crate::context::{ResourceContext, ServiceContext, SystemContext};
use async_trait::async_trait;

// Sub-modules
pub mod director;

// Re-exports
pub use director::SceneDirector;

/// Scene transition result
///
/// Generic over the scene type S to allow transitions to carry scene data.
/// This enables type-safe scene transitions without manual coupling.
///
/// # Example
///
/// ```ignore
/// enum GameScene {
///     Title(TitleData),
///     Combat(CombatData),
/// }
///
/// // Return a transition with scene data
/// fn handle_input(&mut self) -> SceneTransition<GameScene> {
///     SceneTransition::Switch(GameScene::Combat(CombatData::new()))
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneTransition<S> {
    /// Stay in current scene
    Stay,
    /// Switch to a new scene (replaces current)
    Switch(S),
    /// Push a new scene on top of the stack
    Push(S),
    /// Pop the current scene from the stack
    Pop,
    /// Quit the game
    Quit,
}

/// Scene trait with lifecycle methods
///
/// Scenes represent distinct game states (Title, Combat, etc.)
/// Each scene has its own data that is discarded on transition.
///
/// # Lifecycle Hooks
///
/// - `on_enter()`: Called when this scene becomes active
/// - `on_exit()`: Called when this scene is removed
/// - `on_suspend()`: Called when another scene is pushed on top (Phase 2+)
/// - `on_resume()`: Called when the scene on top is popped (Phase 2+)
/// - `on_update()`: Called every frame while active
///
/// # Sized Requirement
///
/// This trait requires `Sized` because `SceneTransition<Self>` needs to know
/// the size of the scene type at compile time.
#[async_trait]
pub trait Scene: Send + Sized {
    /// Called when entering this scene
    ///
    /// This is called when:
    /// - Scene is first created (via `SceneDirector::new(..)` with contexts)
    /// - Scene is pushed onto the stack (via `Push`)
    /// - Scene is switched to (via `Switch`)
    async fn on_enter(
        &mut self,
        _services: &ServiceContext,
        _systems: &mut SystemContext,
        _resources: &mut ResourceContext,
    ) {
    }

    /// Called when leaving this scene permanently
    ///
    /// This is called when:
    /// - Scene is popped from the stack (via `Pop`)
    /// - Scene is replaced (via `Switch`)
    /// - Application quits (via `Quit`)
    async fn on_exit(
        &mut self,
        _services: &ServiceContext,
        _systems: &mut SystemContext,
        _resources: &mut ResourceContext,
    ) {
    }

    /// Called when another scene is pushed on top of this scene
    ///
    /// This is called when a scene is pushed onto the stack,
    /// temporarily hiding this scene. Use this to pause animations,
    /// stop timers, etc.
    ///
    /// Default: do nothing (Phase 1 compatibility)
    async fn on_suspend(
        &mut self,
        _services: &ServiceContext,
        _systems: &mut SystemContext,
        _resources: &mut ResourceContext,
    ) {
    }

    /// Called when the scene on top is popped, revealing this scene again
    ///
    /// This is called when the scene above is popped, making this
    /// scene active again. Use this to resume animations, restart
    /// timers, etc.
    ///
    /// Default: do nothing (Phase 1 compatibility)
    async fn on_resume(
        &mut self,
        _services: &ServiceContext,
        _systems: &mut SystemContext,
        _resources: &mut ResourceContext,
    ) {
    }

    /// Called every frame, returns transition decision
    ///
    /// Return `SceneTransition::Stay` to continue in current scene.
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn on_update(
    ///     &mut self,
    ///     services: &ServiceContext,
    ///     systems: &mut SystemContext,
    ///     resources: &mut ResourceContext,
    /// ) -> SceneTransition<Self> {
    ///     if self.should_quit {
    ///         SceneTransition::Quit
    ///     } else if self.should_go_to_combat {
    ///         SceneTransition::Switch(GameScene::Combat(CombatData::new()))
    ///     } else {
    ///         SceneTransition::Stay
    ///     }
    /// }
    /// ```
    async fn on_update(
        &mut self,
        _services: &ServiceContext,
        _systems: &mut SystemContext,
        _resources: &mut ResourceContext,
    ) -> SceneTransition<Self> {
        SceneTransition::Stay
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Scene; // Import derive macro

    // Test Scene derive macro
    #[derive(Scene)]
    struct TestScene {
        entered: bool,
    }

    #[tokio::test]
    async fn test_derived_scene() {
        let mut scene = TestScene { entered: false };

        // Verify initial state
        assert!(!scene.entered);

        // Default on_enter should work
        scene
            .on_enter(
                &crate::context::ServiceContext::new(),
                &mut crate::context::SystemContext::new(),
                &mut crate::context::ResourceContext::new(),
            )
            .await;

        // Default on_update should return Stay
        let transition = scene
            .on_update(
                &crate::context::ServiceContext::new(),
                &mut crate::context::SystemContext::new(),
                &mut crate::context::ResourceContext::new(),
            )
            .await;
        assert!(matches!(transition, SceneTransition::Stay));

        // Default on_exit should work
        scene
            .on_exit(
                &crate::context::ServiceContext::new(),
                &mut crate::context::SystemContext::new(),
                &mut crate::context::ResourceContext::new(),
            )
            .await;
    }

    #[derive(Scene)]
    #[allow(dead_code)]
    enum GameScene {
        Title,
        Combat,
    }

    #[tokio::test]
    async fn test_derived_enum_scene() {
        let mut scene = GameScene::Title;

        scene
            .on_enter(
                &crate::context::ServiceContext::new(),
                &mut crate::context::SystemContext::new(),
                &mut crate::context::ResourceContext::new(),
            )
            .await;
        let transition = scene
            .on_update(
                &crate::context::ServiceContext::new(),
                &mut crate::context::SystemContext::new(),
                &mut crate::context::ResourceContext::new(),
            )
            .await;
        assert!(matches!(transition, SceneTransition::Stay));
        scene
            .on_exit(
                &crate::context::ServiceContext::new(),
                &mut crate::context::SystemContext::new(),
                &mut crate::context::ResourceContext::new(),
            )
            .await;
    }
}
