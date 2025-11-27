//! Test for #[skip] attribute

use bevy::prelude::*;
use issun_bevy::IssunCorePlugin;
use issun_macros::IssunBevyPlugin;
use serde::{Deserialize, Serialize};

#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct PublicConfig {
    pub value: u32,
}

#[derive(Clone, Debug, Default)]
pub struct InternalState {
    pub private_data: String,
}

/// Test plugin with skipped field
#[derive(Default, IssunBevyPlugin)]
pub struct SkipTestPlugin {
    #[resource]
    pub config: PublicConfig,

    /// This field should NOT be registered as a resource
    #[skip]
    pub internal: InternalState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_field() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(IssunCorePlugin)
            .add_plugins(SkipTestPlugin::default());

        app.update();

        // Verify config is registered
        assert!(
            app.world().get_resource::<PublicConfig>().is_some(),
            "PublicConfig should be registered"
        );

        // Verify internal is NOT registered (would fail to compile if it were)
        // InternalState doesn't implement Resource, so this test ensures
        // that the #[skip] attribute prevented registration
    }
}

pub fn run_skip_test() {
    println!("\nTest 3: Skip field");

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(IssunCorePlugin)
        .add_plugins(SkipTestPlugin::default().with_config(PublicConfig { value: 42 }));

    app.update();

    let config = app.world().get_resource::<PublicConfig>().unwrap();
    assert_eq!(config.value, 42);

    println!("✅ Skip field works correctly");
}
