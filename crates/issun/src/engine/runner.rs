//! Game runner that wires TUI, SceneDirector, and the new runtime contexts.
//!
//! This utility provides a structured game loop so examples no longer need to
//! hand-roll the same `poll_input` + `SceneDirector::handle` logic.

use crate::{
    context::{ResourceContext, ServiceContext, SystemContext},
    error::Result,
    event::EventBus,
    scene::{Scene, SceneDirector, SceneTransition},
    ui::{input::poll_input, InputEvent, Tui},
};
use ratatui::Frame;
use std::{
    future::Future,
    pin::Pin,
    time::{Duration, Instant},
};

/// High level runner that owns the game loop.
pub struct GameRunner<S> {
    director: SceneDirector<S>,
    tick_rate: Duration,
}

impl<S: Scene> GameRunner<S> {
    /// Create a runner with default tick rate (30 FPS).
    pub fn new(director: SceneDirector<S>) -> Self {
        Self {
            director,
            tick_rate: Duration::from_millis(33),
        }
    }

    /// Override the tick rate (frame interval).
    pub fn with_tick_rate(mut self, tick_rate: Duration) -> Self {
        self.tick_rate = tick_rate;
        self
    }

    /// Borrow the underlying director.
    pub fn director(&self) -> &SceneDirector<S> {
        &self.director
    }

    /// Mutable access to the director (e.g., for inspecting stack).
    pub fn director_mut(&mut self) -> &mut SceneDirector<S> {
        &mut self.director
    }

    /// Update all registered systems that require periodic updates.
    ///
    /// This method processes event-driven systems like TimerSystem and ActionResetSystem
    /// that respond to published events (AdvanceTimeRequested, DayChanged, etc.).
    ///
    /// Systems with custom update signatures will be called with appropriate contexts.
    async fn update_systems(&mut self) {
        use crate::plugin::action::ActionResetSystem;
        use crate::plugin::time::TimerSystem;

        // Update TimerSystem (processes AdvanceTimeRequested → DayChanged)
        self.director
            .with_current_async(|_, services, systems, resources| {
                Box::pin(async move {
                    if let Some(timer_system) = systems.get_mut::<TimerSystem>() {
                        timer_system.update(services, resources).await;
                    }
                })
            })
            .await;

        // Update ActionResetSystem (processes DayChanged → reset action points)
        self.director
            .with_current_async(|_, services, systems, resources| {
                Box::pin(async move {
                    if let Some(action_reset) = systems.get_mut::<ActionResetSystem>() {
                        action_reset.update(services, resources).await;
                    }
                })
            })
            .await;
    }

    /// Run the game loop until the director requests quit.
    ///
    /// # Parameters
    /// - `tui`: initialized [`Tui`] instance.
    /// - `render`: callback invoked every frame with the current scene and resources.
    /// - `on_input`: async handler invoked whenever an [`InputEvent`] is received.
    pub async fn run<R, H>(mut self, tui: &mut Tui, mut render: R, mut on_input: H) -> Result<()>
    where
        R: FnMut(&mut Frame, &S, &ResourceContext),
        H: for<'a> FnMut(
            &'a mut S,
            &'a ServiceContext,
            &'a mut SystemContext,
            &'a mut ResourceContext,
            InputEvent,
        ) -> Pin<Box<dyn Future<Output = SceneTransition<S>> + 'a>>,
    {
        let mut last_tick = Instant::now();

        loop {
            // Draw
            tui.terminal().draw(|frame| {
                if let Some(scene) = self.director.current() {
                    render(frame, scene, self.director.resources());
                }
            })?;

            // Calculate timeout for next tick
            let timeout = self
                .tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::ZERO);

            // Poll input with timeout
            let input = poll_input(timeout)?;

            if input != InputEvent::Other {
                if let Some(transition) = self
                    .director
                    .with_current_async(|scene, services, systems, resources| {
                        on_input(scene, services, systems, resources, input)
                    })
                    .await
                {
                    self.director.handle(transition).await?;
                }
            }

            // Periodic update (Scene::on_update)
            if last_tick.elapsed() >= self.tick_rate {
                let transition = self.director.update().await;
                self.director.handle(transition).await?;

                // Update registered systems (handles event-driven logic)
                self.update_systems().await;

                last_tick = Instant::now();
            }

            if self.director.should_quit() || self.director.is_empty() {
                break;
            }

            if let Some(mut event_bus) = self.director.resources_mut().get_mut::<EventBus>().await {
                event_bus.dispatch();
            }
        }

        Ok(())
    }
}
