//! Scene Director - Manages scene lifecycle and transitions
//!
//! The SceneDirector handles:
//! - Scene lifecycle (on_enter, on_update, on_exit)
//! - Scene transitions (switching between scenes)
//! - Quit state management
//!
//! # Example
//!
//! ```ignore
//! use issun::prelude::*;
//!
//! #[derive(Scene)]
//! enum GameScene {
//!     Title(TitleData),
//!     Combat(CombatData),
//! }
//!
//! let mut director = SceneDirector::new(GameScene::Title(TitleData::new())).await;
//!
//! loop {
//!     let transition = director.update().await;
//!
//!     match transition {
//!         SceneTransition::Quit => break,
//!         _ => {}
//!     }
//!
//!     if director.should_quit() {
//!         break;
//!     }
//! }
//! ```

use super::{Scene, SceneTransition};
use crate::error::Result;

/// Scene Director manages scene lifecycle and transitions
///
/// Phase 1: Basic single-scene management with lifecycle hooks
pub struct SceneDirector<S> {
    /// Current active scene
    current: S,
    /// Whether the application should quit
    should_quit: bool,
}

impl<S: Scene> SceneDirector<S> {
    /// Create a new SceneDirector with an initial scene
    ///
    /// The initial scene's `on_enter()` will be called immediately.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let director = SceneDirector::new(GameScene::Title(TitleData::new())).await;
    /// ```
    pub async fn new(mut initial_scene: S) -> Self {
        // Call on_enter for the initial scene
        initial_scene.on_enter().await;

        Self {
            current: initial_scene,
            should_quit: false,
        }
    }

    /// Update the current scene
    ///
    /// Calls `on_update()` on the current scene and returns the transition result.
    ///
    /// # Returns
    ///
    /// The SceneTransition indicating what should happen next.
    pub async fn update(&mut self) -> SceneTransition {
        self.current.on_update().await
    }

    /// Transition to a new scene
    ///
    /// This will:
    /// 1. Call `on_exit()` on the current scene
    /// 2. Replace the current scene with the new scene
    /// 3. Call `on_enter()` on the new scene
    ///
    /// # Example
    ///
    /// ```ignore
    /// director.transition_to(GameScene::Combat(CombatData::new())).await;
    /// ```
    pub async fn transition_to(&mut self, mut next: S) {
        // Exit current scene
        self.current.on_exit().await;

        // Enter new scene
        next.on_enter().await;

        // Replace current scene
        self.current = next;
    }

    /// Request application quit
    ///
    /// This will:
    /// 1. Call `on_exit()` on the current scene
    /// 2. Set the quit flag to true
    pub async fn quit(&mut self) {
        self.current.on_exit().await;
        self.should_quit = true;
    }

    /// Check if the application should quit
    ///
    /// # Returns
    ///
    /// `true` if `quit()` has been called, `false` otherwise.
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Get a reference to the current scene
    ///
    /// Useful for rendering or inspecting scene state.
    pub fn current(&self) -> &S {
        &self.current
    }

    /// Get a mutable reference to the current scene
    ///
    /// Useful for direct scene manipulation.
    pub fn current_mut(&mut self) -> &mut S {
        &mut self.current
    }

    /// Handle a scene transition returned from update()
    ///
    /// This is a convenience method that interprets SceneTransition
    /// and calls the appropriate methods.
    ///
    /// Note: This only handles Stay, Transition, and Quit.
    /// The actual scene switching must be done by the caller
    /// since we don't have the next scene data here.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let transition = director.update().await;
    /// match transition {
    ///     SceneTransition::Transition => {
    ///         // Caller decides what scene to transition to
    ///         let next = determine_next_scene();
    ///         director.transition_to(next).await;
    ///     }
    ///     SceneTransition::Quit => {
    ///         director.quit().await;
    ///     }
    ///     SceneTransition::Stay => {
    ///         // Do nothing
    ///     }
    /// }
    /// ```
    pub async fn handle_transition_simple(&mut self, transition: SceneTransition) -> Result<()> {
        match transition {
            SceneTransition::Stay => {
                // Do nothing
            }
            SceneTransition::Transition => {
                // Transition requested, but we don't know the target scene
                // This should be handled by the caller
            }
            SceneTransition::Quit => {
                self.quit().await;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    // Test scene that tracks lifecycle calls
    #[derive(Debug, Clone)]
    struct TestScene {
        name: String,
        enter_count: usize,
        update_count: usize,
        exit_count: usize,
        should_transition: bool,
        should_quit: bool,
    }

    impl TestScene {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                enter_count: 0,
                update_count: 0,
                exit_count: 0,
                should_transition: false,
                should_quit: false,
            }
        }

        fn with_transition(mut self) -> Self {
            self.should_transition = true;
            self
        }

        fn with_quit(mut self) -> Self {
            self.should_quit = true;
            self
        }
    }

    #[async_trait]
    impl Scene for TestScene {
        async fn on_enter(&mut self) {
            self.enter_count += 1;
        }

        async fn on_update(&mut self) -> SceneTransition {
            self.update_count += 1;

            if self.should_quit {
                SceneTransition::Quit
            } else if self.should_transition {
                SceneTransition::Transition
            } else {
                SceneTransition::Stay
            }
        }

        async fn on_exit(&mut self) {
            self.exit_count += 1;
        }
    }

    #[tokio::test]
    async fn test_new_calls_on_enter() {
        let scene = TestScene::new("test");
        let director = SceneDirector::new(scene).await;

        assert_eq!(director.current().enter_count, 1);
        assert_eq!(director.current().update_count, 0);
        assert_eq!(director.current().exit_count, 0);
    }

    #[tokio::test]
    async fn test_update_calls_on_update() {
        let scene = TestScene::new("test");
        let mut director = SceneDirector::new(scene).await;

        let transition = director.update().await;
        assert_eq!(transition, SceneTransition::Stay);
        assert_eq!(director.current().update_count, 1);
    }

    #[tokio::test]
    async fn test_transition_to_calls_lifecycle() {
        let scene1 = TestScene::new("scene1");
        let mut director = SceneDirector::new(scene1).await;

        // Transition to scene2
        let scene2 = TestScene::new("scene2");
        director.transition_to(scene2).await;

        // scene1 should have exited (but we can't check it anymore)
        // scene2 should have entered
        assert_eq!(director.current().name, "scene2");
        assert_eq!(director.current().enter_count, 1);
        assert_eq!(director.current().exit_count, 0);
    }

    #[tokio::test]
    async fn test_quit_calls_on_exit() {
        let scene = TestScene::new("test");
        let mut director = SceneDirector::new(scene).await;

        assert!(!director.should_quit());

        director.quit().await;

        assert!(director.should_quit());
        assert_eq!(director.current().exit_count, 1);
    }

    #[tokio::test]
    async fn test_should_quit_returns_transition() {
        let scene = TestScene::new("test").with_quit();
        let mut director = SceneDirector::new(scene).await;

        let transition = director.update().await;
        assert_eq!(transition, SceneTransition::Quit);
    }

    #[tokio::test]
    async fn test_current_and_current_mut() {
        let scene = TestScene::new("test");
        let mut director = SceneDirector::new(scene).await;

        // Test immutable access
        assert_eq!(director.current().name, "test");

        // Test mutable access
        director.current_mut().name = "modified".to_string();
        assert_eq!(director.current().name, "modified");
    }

    #[tokio::test]
    async fn test_multiple_updates() {
        let scene = TestScene::new("test");
        let mut director = SceneDirector::new(scene).await;

        for i in 1..=5 {
            director.update().await;
            assert_eq!(director.current().update_count, i);
        }
    }
}
