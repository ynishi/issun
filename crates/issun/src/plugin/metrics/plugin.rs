//! Metrics plugin implementation

use super::hook::{MetricsHook, NoOpMetricsHook};
use super::registry::{MetricsConfig, MetricsRegistry};
use super::system::MetricsSystem;
use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;
use std::sync::Arc;

/// Built-in metrics collection, aggregation, and reporting plugin
///
/// This plugin provides comprehensive metrics management for games.
/// It registers MetricsRegistry resource and MetricsSystem that handles:
/// - Processing metric definition requests
/// - Processing metric recording requests
/// - Processing snapshot and report generation requests
/// - Custom hooks for game-specific behavior
///
/// # Hook Customization
///
/// You can provide a custom hook to add game-specific behavior:
/// - React to metric recordings (e.g., send to external monitoring)
/// - Generate alerts on threshold violations
/// - Export reports to external systems
/// - Archive snapshots to storage
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::metrics::{MetricsPlugin, MetricsHook};
/// use async_trait::async_trait;
///
/// // Custom hook for external monitoring
/// struct MonitoringHook;
///
/// #[async_trait]
/// impl MetricsHook for MonitoringHook {
///     async fn on_metric_recorded(
///         &self,
///         value: &MetricValue,
///         resources: &mut ResourceContext,
///     ) {
///         // Send to Prometheus, Datadog, etc.
///         println!("Metric recorded: {:?}", value);
///     }
/// }
///
/// let game = GameBuilder::new()
///     .with_plugin(
///         MetricsPlugin::new()
///             .with_hook(MonitoringHook)
///     )
///     .build()
///     .await?;
/// ```
pub struct MetricsPlugin {
    hook: Arc<dyn MetricsHook>,
    config: MetricsConfig,
}

impl MetricsPlugin {
    /// Create a new metrics plugin
    ///
    /// Uses the default hook (no-op) and default config by default.
    /// Use `with_hook()` to add custom behavior and `with_config()` to customize configuration.
    pub fn new() -> Self {
        Self {
            hook: Arc::new(NoOpMetricsHook),
            config: MetricsConfig::default(),
        }
    }

    /// Add a custom hook for metrics behavior
    ///
    /// The hook will be called when:
    /// - Metric is defined (`on_metric_defined`)
    /// - Metric value is recorded (`on_metric_recorded`)
    /// - Snapshot is created (`on_snapshot_created`)
    /// - Report is generated (`on_report_generated`)
    /// - Metric is removed (`on_metric_removed`)
    /// - Registry is cleared (`on_registry_cleared`)
    ///
    /// # Arguments
    ///
    /// * `hook` - Implementation of MetricsHook trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::metrics::{MetricsPlugin, MetricsHook};
    ///
    /// struct MyHook;
    ///
    /// #[async_trait]
    /// impl MetricsHook for MyHook {
    ///     async fn on_metric_recorded(
    ///         &self,
    ///         value: &MetricValue,
    ///         resources: &mut ResourceContext,
    ///     ) {
    ///         // Custom behavior...
    ///     }
    /// }
    ///
    /// let plugin = MetricsPlugin::new().with_hook(MyHook);
    /// ```
    pub fn with_hook<H: MetricsHook + 'static>(mut self, hook: H) -> Self {
        self.hook = Arc::new(hook);
        self
    }

    /// Set custom metrics configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Metrics configuration (window size, periodic snapshots, etc.)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::metrics::{MetricsPlugin, MetricsConfig};
    ///
    /// let config = MetricsConfig {
    ///     max_values_per_metric: 2000,
    ///     enable_periodic_snapshots: true,
    ///     snapshot_period: 1,
    ///     enable_auto_report: true,
    ///     report_period: 7,
    /// };
    ///
    /// let plugin = MetricsPlugin::new().with_config(config);
    /// ```
    pub fn with_config(mut self, config: MetricsConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for MetricsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for MetricsPlugin {
    fn name(&self) -> &'static str {
        "metrics_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register metrics registry resource
        builder.register_runtime_state(MetricsRegistry::with_config(self.config.clone()));

        // Register metrics system with hook
        builder.register_system(Box::new(MetricsSystem::new(self.hook.clone())));
    }

    async fn initialize(&mut self) {
        // No initialization needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = MetricsPlugin::new();
        assert_eq!(plugin.name(), "metrics_plugin");
    }

    #[test]
    fn test_plugin_with_custom_hook() {
        struct CustomHook;

        #[async_trait::async_trait]
        impl MetricsHook for CustomHook {}

        let plugin = MetricsPlugin::new().with_hook(CustomHook);
        assert_eq!(plugin.name(), "metrics_plugin");
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let config = MetricsConfig {
            max_values_per_metric: 5000,
            enable_periodic_snapshots: true,
            ..Default::default()
        };

        let plugin = MetricsPlugin::new().with_config(config);
        assert_eq!(plugin.name(), "metrics_plugin");
    }
}
