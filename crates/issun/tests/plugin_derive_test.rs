//! Test to verify Plugin derive macro correctly registers resources

use issun::builder::GameBuilder;
use issun::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;

#[derive(Clone, Debug, PartialEq, issun_macros::Resource)]
struct TestConfig {
    value: i32,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self { value: 42 }
    }
}

/// Manual implementation (expected behavior)
struct ManualPlugin {
    config: TestConfig,
}

impl ManualPlugin {
    fn new() -> Self {
        Self {
            config: TestConfig::default(),
        }
    }

    fn with_config(mut self, config: TestConfig) -> Self {
        self.config = config;
        self
    }
}

#[async_trait]
impl Plugin for ManualPlugin {
    fn name(&self) -> &'static str {
        "manual_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        println!("[ManualPlugin] Registering config: {:?}", self.config);
        builder.register_resource(self.config.clone());
    }
}

/// Derived implementation (actual behavior)
#[derive(issun_macros::Plugin)]
#[plugin(name = "derived_plugin")]
struct DerivedPlugin {
    #[plugin(resource)]
    config: TestConfig,
}

impl DerivedPlugin {
    fn new() -> Self {
        Self {
            config: TestConfig::default(),
        }
    }

    fn with_config(mut self, config: TestConfig) -> Self {
        self.config = config;
        self
    }
}

#[tokio::test]
async fn test_manual_plugin_registers_config() {
    let config = TestConfig { value: 100 };

    let game = GameBuilder::new()
        .with_plugin(ManualPlugin::new().with_config(config.clone()))
        .expect("Failed to add manual plugin")
        .build()
        .await
        .expect("Failed to build game");

    assert!(
        game.resources.contains::<TestConfig>(),
        "Manual plugin should register TestConfig"
    );

    let registered_config = game.resources.get::<TestConfig>().await.unwrap();
    assert_eq!(registered_config.value, 100);
}

#[tokio::test]
async fn test_derived_plugin_registers_config() {
    let config = TestConfig { value: 200 };

    let game = GameBuilder::new()
        .with_plugin(DerivedPlugin::new().with_config(config.clone()))
        .expect("Failed to add derived plugin")
        .build()
        .await
        .expect("Failed to build game");

    assert!(
        game.resources.contains::<TestConfig>(),
        "Derived plugin should register TestConfig"
    );

    let registered_config = game.resources.get::<TestConfig>().await.unwrap();
    assert_eq!(registered_config.value, 200);
}
