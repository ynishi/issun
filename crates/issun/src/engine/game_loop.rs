//! Game loop for ISSUN
//!
//! Async-enabled turn-based game loop with non-blocking operations support.

use crate::context::{ResourceContext, ServiceContext, SystemContext};
use crate::error::Result;
use crate::scene::{Scene, SceneTransition};
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

/// Game loop configuration
pub struct GameLoopConfig {
    /// Timeout for input polling (milliseconds)
    pub input_timeout_ms: u64,
    /// Enable input echo
    pub enable_echo: bool,
}

impl Default for GameLoopConfig {
    fn default() -> Self {
        Self {
            input_timeout_ms: 100,
            enable_echo: false,
        }
    }
}

/// Simple synchronous game loop for turn-based games
pub struct GameLoop {
    config: GameLoopConfig,
}

impl GameLoop {
    /// Create a new game loop with default configuration
    pub fn new() -> Self {
        Self {
            config: GameLoopConfig::default(),
        }
    }

    /// Create a game loop with custom configuration
    pub fn with_config(config: GameLoopConfig) -> Self {
        Self { config }
    }

    /// Run the game loop with a scene and contexts
    ///
    /// Returns when SceneTransition::Quit is received
    pub async fn run<S: Scene>(
        &self,
        mut scene: S,
        services: &ServiceContext,
        systems: &mut SystemContext,
        resources: &mut ResourceContext,
    ) -> Result<()> {
        scene.on_enter(services, systems, resources).await;

        loop {
            // Poll for input
            if event::poll(Duration::from_millis(self.config.input_timeout_ms))? {
                if let Event::Key(key_event) = event::read()? {
                    // Handle quit
                    if key_event.code == KeyCode::Char('q') || key_event.code == KeyCode::Esc {
                        break;
                    }
                }
            }

            // Update scene
            match scene.on_update(services, systems, resources).await {
                SceneTransition::Stay => {
                    // Continue current scene
                }
                SceneTransition::Switch(_) | SceneTransition::Push(_) | SceneTransition::Pop => {
                    // Scene requests transition
                    // TODO: Handle scene transitions with SceneDirector
                    break;
                }
                SceneTransition::Quit => {
                    // Quit requested
                    break;
                }
            }
        }

        scene.on_exit(services, systems, resources).await;
        Ok(())
    }
}

impl Default for GameLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    #[allow(dead_code)]
    struct TestScene {
        update_count: usize,
    }

    #[async_trait]
    impl Scene for TestScene {
        async fn on_enter(
            &mut self,
            _services: &ServiceContext,
            _systems: &mut SystemContext,
            _resources: &mut ResourceContext,
        ) {
            self.update_count = 0;
        }

        async fn on_update(
            &mut self,
            _services: &ServiceContext,
            _systems: &mut SystemContext,
            _resources: &mut ResourceContext,
        ) -> SceneTransition<Self> {
            self.update_count += 1;
            if self.update_count >= 3 {
                SceneTransition::Quit
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
        }
    }

    #[test]
    fn test_game_loop_config() {
        let config = GameLoopConfig::default();
        assert_eq!(config.input_timeout_ms, 100);
        assert!(!config.enable_echo);
    }
}
