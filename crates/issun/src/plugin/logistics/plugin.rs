//! LogisticsPlugin implementation

use super::config::LogisticsConfig;
use super::hook::{DefaultLogisticsHook, LogisticsHook};
use super::service::LogisticsService;
use super::state::LogisticsState;
use super::system::LogisticsSystem;
use crate::Plugin;
use std::sync::Arc;

/// Logistics plugin for automated resource transportation
///
/// This plugin provides logistics management functionality with:
/// - Automated resource transfer between inventories
/// - Event-driven scheduling for efficient processing
/// - Customizable behavior via hooks
/// - Performance metrics and monitoring
///
/// # Hook Customization
///
/// You can provide a custom hook to add game-specific behavior:
/// - React to successful/failed transfers
/// - Calculate transport costs (economy integration)
/// - Handle route failures and auto-disable
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::logistics::{LogisticsPlugin, Route, Transporter};
///
/// let game = GameBuilder::new()
///     .with_plugin(LogisticsPlugin::new())
///     .build()
///     .await?;
///
/// // Register route
/// let mut logistics = game.get_plugin_mut::<LogisticsPlugin>().unwrap();
/// let route = Route::new(
///     "mine_to_smelter",
///     "mine",
///     "smelter",
///     Transporter::new(10, 1.0), // 10 items/sec
/// );
/// logistics.state_mut().register_route(route);
/// ```
#[derive(Plugin)]
#[plugin(name = "issun:logistics")]
pub struct LogisticsPlugin {
    #[plugin(skip)]
    hook: Arc<dyn LogisticsHook>,

    #[plugin(resource)]
    config: LogisticsConfig,

    #[plugin(runtime_state)]
    state: LogisticsState,

    #[plugin(service)]
    service: LogisticsService,

    #[plugin(system)]
    system: LogisticsSystem,
}

impl LogisticsPlugin {
    /// Create a new logistics plugin
    ///
    /// Uses the default hook (no-op) and default config by default.
    /// Use `with_hook()` to add custom behavior and `with_config()` to customize configuration.
    pub fn new() -> Self {
        let hook = Arc::new(DefaultLogisticsHook);
        Self {
            hook: hook.clone(),
            config: LogisticsConfig::default(),
            state: LogisticsState::new(),
            service: LogisticsService,
            system: LogisticsSystem::new(hook),
        }
    }

    /// Add a custom hook for logistics behavior
    ///
    /// The hook will be called when:
    /// - Transfers complete (`on_transfer_complete`)
    /// - Transfers fail (`on_transfer_failed`)
    /// - Routes are auto-disabled (`on_route_disabled`)
    /// - Transport costs are calculated (`calculate_transport_cost`)
    ///
    /// # Arguments
    ///
    /// * `hook` - Implementation of LogisticsHook trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::logistics::{LogisticsPlugin, LogisticsHook};
    ///
    /// struct MyHook;
    ///
    /// #[async_trait]
    /// impl LogisticsHook for MyHook {
    ///     async fn on_transfer_complete(&self, route_id: &str, item_id: &str, amount: u32) {
    ///         println!("Transferred {} {}", amount, item_id);
    ///     }
    /// }
    ///
    /// let plugin = LogisticsPlugin::new().with_hook(MyHook);
    /// ```
    pub fn with_hook<H: LogisticsHook + 'static>(mut self, hook: H) -> Self {
        let hook = Arc::new(hook);
        self.hook = hook.clone();
        self.system = LogisticsSystem::new(hook);
        self
    }

    /// Set custom logistics configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Logistics configuration (throughput, limits, etc.)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::logistics::{LogisticsPlugin, LogisticsConfig};
    ///
    /// let config = LogisticsConfig::default()
    ///     .with_throughput_multiplier(2.0)
    ///     .with_max_routes_per_update(500);
    ///
    /// let plugin = LogisticsPlugin::new().with_config(config);
    /// ```
    pub fn with_config(mut self, config: LogisticsConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for LogisticsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::Plugin;

    #[test]
    fn test_plugin_creation() {
        let plugin = LogisticsPlugin::new();
        assert_eq!(plugin.name(), "issun:logistics");
    }

    #[test]
    fn test_plugin_with_custom_hook() {
        struct CustomHook;

        #[async_trait::async_trait]
        impl LogisticsHook for CustomHook {}

        let plugin = LogisticsPlugin::new().with_hook(CustomHook);
        assert_eq!(plugin.name(), "issun:logistics");
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let config = LogisticsConfig::default()
            .with_throughput_multiplier(2.0)
            .with_max_routes_per_update(500);

        let plugin = LogisticsPlugin::new().with_config(config);
        assert_eq!(plugin.name(), "issun:logistics");
    }
}
