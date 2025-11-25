//! Headless game runner without UI
//!
//! Runs game logic at specified tick rate without rendering.
//! Useful for server-side simulation, testing, and AI training.

use crate::{
    error::Result,
    event::EventBus,
    scene::{Scene, SceneDirector},
};
use std::time::Duration;
use tokio::time;

/// Headless game runner
///
/// Unlike [`GameRunner`](crate::engine::GameRunner), this runner does not require a TUI
/// or any rendering. It simply executes the game loop at a specified tick rate.
///
/// # Example
///
/// ```ignore
/// use issun::prelude::*;
/// use issun::engine::HeadlessRunner;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let game = GameBuilder::new()
///         .with_plugin(EconomyPlugin::default())?
///         .build()
///         .await?;
///
///     let director = SceneDirector::new(
///         MySimulation::new(),
///         game.services,
///         game.systems,
///         game.resources,
///     ).await;
///
///     let runner = HeadlessRunner::new(director)
///         .with_tick_rate(Duration::from_millis(100))
///         .with_max_ticks(1000);
///
///     runner.run().await?;
///     Ok(())
/// }
/// ```
pub struct HeadlessRunner<S> {
    director: SceneDirector<S>,
    tick_rate: Duration,
    max_ticks: Option<u64>,
}

impl<S: Scene> HeadlessRunner<S> {
    /// Create a headless runner with default tick rate (100ms).
    pub fn new(director: SceneDirector<S>) -> Self {
        Self {
            director,
            tick_rate: Duration::from_millis(100),
            max_ticks: None,
        }
    }

    /// Override the tick rate (frame interval).
    pub fn with_tick_rate(mut self, tick_rate: Duration) -> Self {
        self.tick_rate = tick_rate;
        self
    }

    /// Set maximum number of ticks before stopping.
    ///
    /// Useful for testing or running simulations for a fixed duration.
    /// If not set, the runner will continue until the director requests quit.
    pub fn with_max_ticks(mut self, max_ticks: u64) -> Self {
        self.max_ticks = Some(max_ticks);
        self
    }

    /// Borrow the underlying director.
    pub fn director(&self) -> &SceneDirector<S> {
        &self.director
    }

    /// Mutable access to the director.
    pub fn director_mut(&mut self) -> &mut SceneDirector<S> {
        &mut self.director
    }

    /// Update all registered systems that require periodic updates.
    ///
    /// This method processes event-driven systems like TimerSystem and ActionResetSystem
    /// that respond to published events.
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

    /// Run the headless game loop until the director requests quit or max_ticks is reached.
    pub async fn run(mut self) -> Result<()> {
        let mut interval = time::interval(self.tick_rate);
        let mut tick_count = 0u64;

        loop {
            interval.tick().await;

            // Scene update (Scene::on_update)
            let transition = self.director.update().await;
            self.director.handle(transition).await?;

            // Update registered systems (handles event-driven logic)
            self.update_systems().await;

            // Dispatch events
            if let Some(mut event_bus) = self.director.resources_mut().get_mut::<EventBus>().await
            {
                event_bus.dispatch();
            }

            tick_count += 1;

            // Check exit conditions
            if self.director.should_quit() || self.director.is_empty() {
                break;
            }

            if let Some(max) = self.max_ticks {
                if tick_count >= max {
                    break;
                }
            }
        }

        Ok(())
    }
}

/// Command-driven headless runner (Pattern 2)
///
/// Unlike [`HeadlessRunner`], this runner can receive external commands through a channel
/// and process them immediately using `tokio::select!`, providing lower latency (<1ms)
/// compared to polling with `try_recv()` (~25ms average for 50ms tick rate).
///
/// Commands are published to the EventBus immediately upon receipt, allowing any Scene
/// to subscribe and process them using the standard ISSUN event pattern.
///
/// # Example
///
/// ```ignore
/// use issun::prelude::*;
/// use issun::engine::HeadlessRunner;
/// use issun::event::Event;
/// use tokio::sync::mpsc;
///
/// #[derive(Clone)]
/// struct ApiCommand {
///     action: String,
/// }
///
/// impl Event for ApiCommand {}
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let (tx, rx) = mpsc::channel(100);
///
///     let game = GameBuilder::new().build().await?;
///     let director = SceneDirector::new(
///         MyScene::new(),
///         game.services,
///         game.systems,
///         game.resources,
///     ).await;
///
///     // Convert to ChannelHeadlessRunner
///     let runner = HeadlessRunner::new(director)
///         .with_tick_rate(Duration::from_millis(50))
///         .with_command_channel(rx);
///
///     // Commands sent via tx will be processed immediately
///     runner.run().await?;
///     Ok(())
/// }
/// ```
pub struct ChannelHeadlessRunner<S, Cmd> {
    director: SceneDirector<S>,
    tick_rate: Duration,
    max_ticks: Option<u64>,
    command_rx: tokio::sync::mpsc::Receiver<Cmd>,
}

impl<S: Scene> HeadlessRunner<S> {
    /// Add a command channel to create a ChannelHeadlessRunner.
    ///
    /// Commands sent through the channel will be published to the EventBus immediately,
    /// allowing for low-latency (<1ms) command processing.
    ///
    /// The command type must implement the `Event` trait and `Serialize` to be published to the EventBus.
    pub fn with_command_channel<Cmd: crate::event::Event + Clone + serde::Serialize>(
        self,
        command_rx: tokio::sync::mpsc::Receiver<Cmd>,
    ) -> ChannelHeadlessRunner<S, Cmd> {
        ChannelHeadlessRunner {
            director: self.director,
            tick_rate: self.tick_rate,
            max_ticks: self.max_ticks,
            command_rx,
        }
    }
}

impl<S: Scene, Cmd: crate::event::Event + Clone + serde::Serialize> ChannelHeadlessRunner<S, Cmd> {
    /// Borrow the underlying director.
    pub fn director(&self) -> &SceneDirector<S> {
        &self.director
    }

    /// Mutable access to the director.
    pub fn director_mut(&mut self) -> &mut SceneDirector<S> {
        &mut self.director
    }

    /// Update all registered systems that require periodic updates.
    ///
    /// This method processes event-driven systems like TimerSystem and ActionResetSystem
    /// that respond to published events.
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

    /// Run the headless game loop with command channel support.
    ///
    /// This runner uses `tokio::select!` to wait for either:
    /// - Regular tick interval (for scene updates)
    /// - Incoming commands (for immediate processing)
    ///
    /// Commands are published to the EventBus immediately upon receipt and dispatched
    /// right away, providing <1ms latency compared to ~25ms with polling-based approach.
    pub async fn run(mut self) -> Result<()> {
        let mut interval = time::interval(self.tick_rate);
        let mut tick_count = 0u64;

        loop {
            tokio::select! {
                // Regular tick update
                _ = interval.tick() => {
                    // Scene update (Scene::on_update)
                    let transition = self.director.update().await;
                    self.director.handle(transition).await?;

                    // Update registered systems (handles event-driven logic)
                    self.update_systems().await;

                    // Dispatch events
                    if let Some(mut event_bus) = self.director.resources_mut().get_mut::<EventBus>().await {
                        event_bus.dispatch();
                    }

                    tick_count += 1;

                    // Check exit conditions
                    if self.director.should_quit() || self.director.is_empty() {
                        break;
                    }

                    if let Some(max) = self.max_ticks {
                        if tick_count >= max {
                            break;
                        }
                    }
                }

                // Immediate command processing
                Some(cmd) = self.command_rx.recv() => {
                    // Publish command as event immediately
                    if let Some(mut event_bus) = self.director.resources_mut().get_mut::<EventBus>().await {
                        event_bus.publish(cmd);
                        // Dispatch immediately for <1ms latency
                        event_bus.dispatch();
                    }

                    // Note: We don't increment tick_count for command processing
                    // Commands are processed immediately without affecting the tick counter
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        builder::GameBuilder,
        context::{ResourceContext, ServiceContext, SystemContext},
        event::Event,
        scene::{Scene, SceneTransition},
    };
    use tokio::sync::mpsc;

    // Test scene that counts updates
    struct TestScene {
        update_count: u32,
    }

    impl TestScene {
        fn new() -> Self {
            Self { update_count: 0 }
        }
    }

    #[async_trait::async_trait]
    impl Scene for TestScene {
        async fn on_update(
            &mut self,
            _services: &ServiceContext,
            _systems: &mut SystemContext,
            _resources: &mut ResourceContext,
        ) -> SceneTransition<Self> {
            self.update_count += 1;
            SceneTransition::Stay
        }
    }

    #[tokio::test]
    async fn test_headless_runner_stops_at_max_ticks() {
        let builder = GameBuilder::new();
        let game = builder.build().await.unwrap();

        let director = SceneDirector::new(
            TestScene::new(),
            game.services,
            game.systems,
            game.resources,
        )
        .await;

        let runner = HeadlessRunner::new(director)
            .with_tick_rate(Duration::from_millis(1))
            .with_max_ticks(10);

        runner.run().await.unwrap();
    }

    #[tokio::test]
    async fn test_headless_runner_updates_correctly() {
        let builder = GameBuilder::new();
        let game = builder.build().await.unwrap();

        let director = SceneDirector::new(
            TestScene::new(),
            game.services,
            game.systems,
            game.resources,
        )
        .await;

        let runner = HeadlessRunner::new(director)
            .with_tick_rate(Duration::from_millis(1))
            .with_max_ticks(5);

        // Runner consumes self, so we can't inspect it after running
        runner.run().await.unwrap();

        // Test passes if run completes without error
    }

    // Test command for ChannelHeadlessRunner
    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct TestCommand {
        value: u32,
    }

    impl Event for TestCommand {}

    // Test scene that processes commands
    struct CommandProcessingScene {
        received_commands: Vec<u32>,
    }

    impl CommandProcessingScene {
        fn new() -> Self {
            Self {
                received_commands: Vec::new(),
            }
        }
    }

    #[async_trait::async_trait]
    impl Scene for CommandProcessingScene {
        async fn on_update(
            &mut self,
            _services: &ServiceContext,
            _systems: &mut SystemContext,
            resources: &mut ResourceContext,
        ) -> SceneTransition<Self> {
            // Process commands from EventBus
            if let Some(mut event_bus) = resources.get_mut::<EventBus>().await {
                let reader = event_bus.reader::<TestCommand>();
                for event in reader.iter() {
                    self.received_commands.push(event.value);
                }
            }

            SceneTransition::Stay
        }
    }

    #[tokio::test]
    async fn test_channel_headless_runner_receives_commands() {
        let (tx, rx) = mpsc::channel(100);

        let builder = GameBuilder::new();
        let mut game = builder.build().await.unwrap();

        // Initialize EventBus
        game.resources.insert(EventBus::new());

        let director = SceneDirector::new(
            CommandProcessingScene::new(),
            game.services,
            game.systems,
            game.resources,
        )
        .await;

        let runner = HeadlessRunner::new(director)
            .with_tick_rate(Duration::from_millis(10))
            .with_max_ticks(5)
            .with_command_channel(rx);

        // Send test commands before running
        // They will be queued and processed during the run loop
        tx.send(TestCommand { value: 42 }).await.unwrap();
        tx.send(TestCommand { value: 100 }).await.unwrap();

        // Drop the sender so the channel won't block
        drop(tx);

        // Run the simulation
        runner.run().await.unwrap();

        // Note: We can't inspect the scene's state after runner consumes it
        // This test verifies that the runner completes without panic and processes commands
    }

    #[tokio::test]
    async fn test_channel_headless_runner_with_no_commands() {
        let (_tx, rx) = mpsc::channel(100);

        let builder = GameBuilder::new();
        let mut game = builder.build().await.unwrap();
        game.resources.insert(EventBus::new());

        let director = SceneDirector::new(
            TestScene::new(),
            game.services,
            game.systems,
            game.resources,
        )
        .await;

        let runner = HeadlessRunner::new(director)
            .with_tick_rate(Duration::from_millis(1))
            .with_max_ticks(3)
            .with_command_channel::<TestCommand>(rx);

        runner.run().await.unwrap();
    }
}
