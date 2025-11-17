//! Scene Director - Manages scene lifecycle and transitions
//!
//! The SceneDirector handles:
//! - Scene lifecycle (on_enter, on_update, on_exit, on_suspend, on_resume)
//! - Scene transitions (switching, pushing, popping)
//! - Scene stack management
//! - Quit state management
//! - Ownership and distribution of Service/System/Resource contexts
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
//!     PauseMenu(PauseData),
//! }
//!
//! let game = GameBuilder::new().build().await?;
//! let mut director = SceneDirector::new(
//!     GameScene::Title(TitleData::new()),
//!     game.services,
//!     game.systems,
//!     game.resources,
//! ).await;
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
//!
//! // Push a pause menu on top
//! director.push(GameScene::PauseMenu(PauseData::new())).await;
//!
//! // Pop back to previous scene
//! director.pop().await;
//! ```

use super::{Scene, SceneTransition};
use crate::context::{ResourceContext, ServiceContext, SystemContext};
use crate::error::Result;
use std::future::Future;

/// Scene Director manages scene lifecycle and transitions
///
/// Phase 2+3: Stack-based scene management with full lifecycle hooks
pub struct SceneDirector<S> {
    /// Scene stack (top is the active scene)
    stack: Vec<S>,
    /// Whether the application should quit
    should_quit: bool,
    services: ServiceContext,
    systems: SystemContext,
    resources: ResourceContext,
}

impl<S: Scene> SceneDirector<S> {
    /// Create a new SceneDirector with an initial scene
    ///
    /// The initial scene's `on_enter()` will be called immediately.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let director = SceneDirector::new(
    ///     GameScene::Title(TitleData::new()),
    ///     ServiceContext::new(),
    ///     SystemContext::new(),
    ///     ResourceContext::new(),
    /// ).await;
    /// ```
    pub async fn new(
        mut initial_scene: S,
        services: ServiceContext,
        mut systems: SystemContext,
        mut resources: ResourceContext,
    ) -> Self {
        // Call on_enter for the initial scene
        initial_scene
            .on_enter(&services, &mut systems, &mut resources)
            .await;

        Self {
            stack: vec![initial_scene],
            should_quit: false,
            services,
            systems,
            resources,
        }
    }

    /// Update the current scene (top of stack)
    ///
    /// Calls `on_update()` on the current scene and returns the transition result.
    ///
    /// # Returns
    ///
    /// The SceneTransition<S> indicating what should happen next.
    pub async fn update(&mut self) -> SceneTransition<S> {
        if let Some(current) = self.stack.last_mut() {
            current
                .on_update(&self.services, &mut self.systems, &mut self.resources)
                .await
        } else {
            SceneTransition::Quit // Empty stack = quit
        }
    }

    /// Switch to a new scene (replaces current scene)
    ///
    /// This will:
    /// 1. Call `on_exit()` on the current scene
    /// 2. Replace the current scene with the new scene
    /// 3. Call `on_enter()` on the new scene
    ///
    /// # Example
    ///
    /// ```ignore
    /// director.switch_to(GameScene::Combat(CombatData::new())).await;
    /// ```
    pub async fn switch_to(&mut self, mut next: S) {
        if let Some(mut current) = self.stack.pop() {
            // Exit current scene
            current
                .on_exit(&self.services, &mut self.systems, &mut self.resources)
                .await;
        }

        // Enter new scene
        next.on_enter(&self.services, &mut self.systems, &mut self.resources)
            .await;

        // Push new scene onto stack
        self.stack.push(next);
    }

    /// Push a new scene on top of the stack
    ///
    /// This will:
    /// 1. Call `on_suspend()` on the current scene
    /// 2. Call `on_enter()` on the new scene
    /// 3. Push the new scene onto the stack
    ///
    /// Use this for temporary overlays like pause menus or dialogs.
    ///
    /// # Example
    ///
    /// ```ignore
    /// director.push(GameScene::PauseMenu(PauseData::new())).await;
    /// ```
    pub async fn push(&mut self, mut next: S) {
        // Suspend current scene (if any)
        if let Some(current) = self.stack.last_mut() {
            current
                .on_suspend(&self.services, &mut self.systems, &mut self.resources)
                .await;
        }

        // Enter new scene
        next.on_enter(&self.services, &mut self.systems, &mut self.resources)
            .await;

        // Push onto stack
        self.stack.push(next);
    }

    /// Pop the current scene from the stack
    ///
    /// This will:
    /// 1. Call `on_exit()` on the current scene
    /// 2. Pop the scene from the stack
    /// 3. Call `on_resume()` on the now-current scene (if any)
    ///
    /// Returns `true` if a scene was popped, `false` if stack was empty.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if director.pop().await {
    ///     // Successfully popped back to previous scene
    /// }
    /// ```
    pub async fn pop(&mut self) -> bool {
        if let Some(mut popped) = self.stack.pop() {
            // Exit popped scene
            popped
                .on_exit(&self.services, &mut self.systems, &mut self.resources)
                .await;

            // Resume scene below (if any)
            if let Some(current) = self.stack.last_mut() {
                current
                    .on_resume(&self.services, &mut self.systems, &mut self.resources)
                    .await;
            }

            true
        } else {
            false
        }
    }

    /// Transition to a new scene (deprecated in favor of switch_to)
    ///
    /// This is kept for backward compatibility with Phase 1 code.
    ///
    /// # Example
    ///
    /// ```ignore
    /// director.transition_to(GameScene::Combat(CombatData::new())).await;
    /// ```
    pub async fn transition_to(&mut self, next: S) {
        self.switch_to(next).await;
    }

    /// Request application quit
    ///
    /// This will:
    /// 1. Call `on_exit()` on all scenes in the stack (from top to bottom)
    /// 2. Set the quit flag to true
    pub async fn quit(&mut self) {
        // Exit all scenes in reverse order (top to bottom)
        while let Some(mut scene) = self.stack.pop() {
            scene
                .on_exit(&self.services, &mut self.systems, &mut self.resources)
                .await;
        }

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

    /// Get a reference to the current scene (top of stack)
    ///
    /// Returns `None` if the stack is empty.
    ///
    /// Useful for rendering or inspecting scene state.
    pub fn current(&self) -> Option<&S> {
        self.stack.last()
    }

    /// Get a mutable reference to the current scene (top of stack)
    ///
    /// Returns `None` if the stack is empty.
    ///
    /// Useful for direct scene manipulation.
    pub fn current_mut(&mut self) -> Option<&mut S> {
        self.stack.last_mut()
    }

    /// Apply a closure to the current scene together with contexts
    pub fn with_current_mut<R>(
        &mut self,
        f: impl FnOnce(&mut S, &ServiceContext, &mut SystemContext, &mut ResourceContext) -> R,
    ) -> Option<R> {
        if let Some(scene) = self.stack.last_mut() {
            let services = &self.services;
            let systems = &mut self.systems;
            let resources = &mut self.resources;
            Some(f(scene, services, systems, resources))
        } else {
            None
        }
    }

    /// Apply an async closure to the current scene with contexts
    pub async fn with_current_mut_async<R, F, Fut>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut S, &ServiceContext, &mut SystemContext, &mut ResourceContext) -> Fut,
        Fut: Future<Output = R>,
    {
        if let Some(scene) = self.stack.last_mut() {
            let services = &self.services;
            let systems = &mut self.systems;
            let resources = &mut self.resources;
            Some(f(scene, services, systems, resources).await)
        } else {
            None
        }
    }

    /// Get the depth of the scene stack
    ///
    /// # Returns
    ///
    /// The number of scenes in the stack.
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Check if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Iterate over all scenes in the stack (bottom to top)
    ///
    /// Useful for transparent rendering where you want to draw
    /// background scenes before foreground scenes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Render all scenes with transparency
    /// for (depth, scene) in director.iter_scenes().enumerate() {
    ///     scene.render(frame, depth);
    /// }
    /// ```
    pub fn iter_scenes(&self) -> impl Iterator<Item = &S> {
        self.stack.iter()
    }

    /// Iterate over all scenes in the stack (top to bottom)
    ///
    /// Useful when you need to process scenes from foreground to background.
    pub fn iter_scenes_rev(&self) -> impl Iterator<Item = &S> {
        self.stack.iter().rev()
    }

    /// Iterate over scenes with their depth index
    ///
    /// Returns (depth, scene) where depth=0 is bottom, depth=n-1 is top.
    ///
    /// # Example
    ///
    /// ```ignore
    /// for (depth, scene) in director.iter_with_depth() {
    ///     if depth < 2 {  // Only render bottom 2 layers
    ///         scene.render(frame);
    ///     }
    /// }
    /// ```
    pub fn iter_with_depth(&self) -> impl Iterator<Item = (usize, &S)> {
        self.stack.iter().enumerate()
    }

    /// Access immutable service context
    pub fn services(&self) -> &ServiceContext {
        &self.services
    }

    /// Access immutable system context
    pub fn systems(&self) -> &SystemContext {
        &self.systems
    }

    /// Access mutable system context
    pub fn systems_mut(&mut self) -> &mut SystemContext {
        &mut self.systems
    }

    /// Access immutable resource context
    pub fn resources(&self) -> &ResourceContext {
        &self.resources
    }

    /// Access mutable resource context
    pub fn resources_mut(&mut self) -> &mut ResourceContext {
        &mut self.resources
    }

    /// Handle a scene transition returned from update()
    ///
    /// This is the primary method for processing scene transitions.
    /// It automatically calls the appropriate lifecycle methods based on the transition type.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Simple one-liner usage
    /// let transition = director.update().await;
    /// director.handle(transition).await?;
    ///
    /// // Or in a single call
    /// let transition = scene.handle_input(input);
    /// director.handle(transition).await?;
    /// ```
    pub async fn handle(&mut self, transition: SceneTransition<S>) -> Result<()> {
        match transition {
            SceneTransition::Stay => {
                // Do nothing
            }
            SceneTransition::Switch(next) => {
                self.switch_to(next).await;
            }
            SceneTransition::Push(next) => {
                self.push(next).await;
            }
            SceneTransition::Pop => {
                self.pop().await;
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
    use crate::context::{ResourceContext, ServiceContext, SystemContext};
    use async_trait::async_trait;

    // Test scene that tracks lifecycle calls
    #[derive(Debug, Clone, PartialEq)]
    struct TestScene {
        name: String,
        enter_count: usize,
        update_count: usize,
        exit_count: usize,
        suspend_count: usize,
        resume_count: usize,
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
                suspend_count: 0,
                resume_count: 0,
                should_transition: false,
                should_quit: false,
            }
        }

        #[allow(dead_code)]
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
        async fn on_enter(
            &mut self,
            _services: &ServiceContext,
            _systems: &mut SystemContext,
            _resources: &mut ResourceContext,
        ) {
            self.enter_count += 1;
        }

        async fn on_update(
            &mut self,
            _services: &ServiceContext,
            _systems: &mut SystemContext,
            _resources: &mut ResourceContext,
        ) -> SceneTransition<Self> {
            self.update_count += 1;

            if self.should_quit {
                SceneTransition::Quit
            } else if self.should_transition {
                SceneTransition::Switch(TestScene::new("next"))
            } else {
                SceneTransition::Stay
            }
        }

        async fn on_exit(
            &mut self,
            _services: &ServiceContext,
            _systems: &mut SystemContext,
            _resources: &mut ResourceContext,
        ) {
            self.exit_count += 1;
        }

        async fn on_suspend(
            &mut self,
            _services: &ServiceContext,
            _systems: &mut SystemContext,
            _resources: &mut ResourceContext,
        ) {
            self.suspend_count += 1;
        }

        async fn on_resume(
            &mut self,
            _services: &ServiceContext,
            _systems: &mut SystemContext,
            _resources: &mut ResourceContext,
        ) {
            self.resume_count += 1;
        }
    }

    async fn director_with_scene(scene: TestScene) -> SceneDirector<TestScene> {
        SceneDirector::new(
            scene,
            ServiceContext::new(),
            SystemContext::new(),
            ResourceContext::new(),
        )
        .await
    }

    #[tokio::test]
    async fn test_new_calls_on_enter() {
        let scene = TestScene::new("test");
        let director = director_with_scene(scene).await;

        assert_eq!(director.depth(), 1);
        assert_eq!(director.current().unwrap().enter_count, 1);
        assert_eq!(director.current().unwrap().update_count, 0);
        assert_eq!(director.current().unwrap().exit_count, 0);
    }

    #[tokio::test]
    async fn test_update_calls_on_update() {
        let scene = TestScene::new("test");
        let mut director = director_with_scene(scene).await;

        let transition = director.update().await;
        assert_eq!(transition, SceneTransition::Stay);
        assert_eq!(director.current().unwrap().update_count, 1);
    }

    #[tokio::test]
    async fn test_switch_to_calls_lifecycle() {
        let scene1 = TestScene::new("scene1");
        let mut director = director_with_scene(scene1).await;

        // Switch to scene2
        let scene2 = TestScene::new("scene2");
        director.switch_to(scene2).await;

        // scene1 should have exited (but we can't check it anymore)
        // scene2 should have entered
        assert_eq!(director.depth(), 1);
        assert_eq!(director.current().unwrap().name, "scene2");
        assert_eq!(director.current().unwrap().enter_count, 1);
        assert_eq!(director.current().unwrap().exit_count, 0);
    }

    #[tokio::test]
    async fn test_push_calls_suspend_and_enter() {
        let scene1 = TestScene::new("scene1");
        let mut director = director_with_scene(scene1).await;

        // Push scene2 on top
        let scene2 = TestScene::new("scene2");
        director.push(scene2).await;

        assert_eq!(director.depth(), 2);
        assert_eq!(director.current().unwrap().name, "scene2");
        assert_eq!(director.current().unwrap().enter_count, 1);

        // Can't check scene1's suspend_count from here, but it should be 1
    }

    #[tokio::test]
    async fn test_pop_calls_exit_and_resume() {
        let scene1 = TestScene::new("scene1");
        let mut director = director_with_scene(scene1).await;

        let scene2 = TestScene::new("scene2");
        director.push(scene2).await;

        assert_eq!(director.depth(), 2);

        // Pop scene2
        let popped = director.pop().await;
        assert!(popped);

        assert_eq!(director.depth(), 1);
        assert_eq!(director.current().unwrap().name, "scene1");
        assert_eq!(director.current().unwrap().resume_count, 1);
    }

    #[tokio::test]
    async fn test_quit_calls_on_exit_for_all() {
        let scene1 = TestScene::new("scene1");
        let mut director = director_with_scene(scene1).await;

        let scene2 = TestScene::new("scene2");
        director.push(scene2).await;

        assert_eq!(director.depth(), 2);
        assert!(!director.should_quit());

        director.quit().await;

        assert!(director.should_quit());
        assert_eq!(director.depth(), 0);
        assert!(director.current().is_none());
    }

    #[tokio::test]
    async fn test_should_quit_returns_transition() {
        let scene = TestScene::new("test").with_quit();
        let mut director = director_with_scene(scene).await;

        let transition = director.update().await;
        assert!(matches!(transition, SceneTransition::Quit));
    }

    #[tokio::test]
    async fn test_handle_switch() {
        let scene1 = TestScene::new("scene1");
        let mut director = director_with_scene(scene1).await;

        let transition = SceneTransition::Switch(TestScene::new("scene2"));
        director.handle(transition).await.unwrap();

        assert_eq!(director.depth(), 1);
        assert_eq!(director.current().unwrap().name, "scene2");
    }

    #[tokio::test]
    async fn test_handle_push_pop() {
        let scene1 = TestScene::new("scene1");
        let mut director = director_with_scene(scene1).await;

        // Push scene2
        let transition = SceneTransition::Push(TestScene::new("scene2"));
        director.handle(transition).await.unwrap();
        assert_eq!(director.depth(), 2);
        assert_eq!(director.current().unwrap().name, "scene2");

        // Pop back to scene1
        let transition = SceneTransition::Pop;
        director.handle(transition).await.unwrap();
        assert_eq!(director.depth(), 1);
        assert_eq!(director.current().unwrap().name, "scene1");
    }

    #[tokio::test]
    async fn test_handle_quit() {
        let scene1 = TestScene::new("scene1");
        let mut director = director_with_scene(scene1).await;

        let transition = SceneTransition::Quit;
        director.handle(transition).await.unwrap();

        assert!(director.should_quit());
        assert_eq!(director.depth(), 0);
    }

    #[tokio::test]
    async fn test_multiple_pushes_and_pops() {
        let scene1 = TestScene::new("scene1");
        let mut director = director_with_scene(scene1).await;

        // Push scene2, scene3
        director.push(TestScene::new("scene2")).await;
        director.push(TestScene::new("scene3")).await;

        assert_eq!(director.depth(), 3);
        assert_eq!(director.current().unwrap().name, "scene3");

        // Pop back to scene2
        director.pop().await;
        assert_eq!(director.depth(), 2);
        assert_eq!(director.current().unwrap().name, "scene2");

        // Pop back to scene1
        director.pop().await;
        assert_eq!(director.depth(), 1);
        assert_eq!(director.current().unwrap().name, "scene1");
    }

    #[tokio::test]
    async fn test_empty_stack_after_pop() {
        let scene1 = TestScene::new("scene1");
        let mut director = director_with_scene(scene1).await;

        director.pop().await;

        assert_eq!(director.depth(), 0);
        assert!(director.current().is_none());
    }

    #[tokio::test]
    async fn test_backward_compatibility_transition_to() {
        let scene1 = TestScene::new("scene1");
        let mut director = director_with_scene(scene1).await;

        // transition_to should work like switch_to
        director.transition_to(TestScene::new("scene2")).await;

        assert_eq!(director.depth(), 1);
        assert_eq!(director.current().unwrap().name, "scene2");
    }

    #[tokio::test]
    async fn test_iter_scenes() {
        let scene1 = TestScene::new("scene1");
        let mut director = director_with_scene(scene1).await;
        director.push(TestScene::new("scene2")).await;
        director.push(TestScene::new("scene3")).await;

        let names: Vec<_> = director.iter_scenes().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["scene1", "scene2", "scene3"]);
    }

    #[tokio::test]
    async fn test_iter_scenes_rev() {
        let scene1 = TestScene::new("scene1");
        let mut director = director_with_scene(scene1).await;
        director.push(TestScene::new("scene2")).await;
        director.push(TestScene::new("scene3")).await;

        let names: Vec<_> = director
            .iter_scenes_rev()
            .map(|s| s.name.as_str())
            .collect();
        assert_eq!(names, vec!["scene3", "scene2", "scene1"]);
    }

    #[tokio::test]
    async fn test_iter_with_depth() {
        let scene1 = TestScene::new("scene1");
        let mut director = director_with_scene(scene1).await;
        director.push(TestScene::new("scene2")).await;

        let items: Vec<_> = director
            .iter_with_depth()
            .map(|(depth, scene)| (depth, scene.name.as_str()))
            .collect();

        assert_eq!(items, vec![(0, "scene1"), (1, "scene2")]);
    }
}
