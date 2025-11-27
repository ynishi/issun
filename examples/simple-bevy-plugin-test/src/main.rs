//! Simple test for IssunBevyPlugin derive macro
//!
//! This example demonstrates the basic usage of the macro with:
//! - #[config] field
//! - #[resource] fields
//! - Builder methods
//! - Custom plugin name
//! - #[skip] attribute
//! - messages auto-registration

mod test_skip;
mod test_messages;
mod test_phase22;

use bevy::prelude::*;
use issun_macros::IssunBevyPlugin;
use serde::{Deserialize, Serialize};

#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct GameConfig {
    pub difficulty: f32,
    pub max_level: u32,
}

#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct GameStats {
    pub score: u32,
    pub level: u32,
}

#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlayerProgress {
    pub xp: u32,
}

/// Test plugin with auto-generated boilerplate
#[derive(Default, IssunBevyPlugin)]
#[plugin(name = "simple_game")]
pub struct SimpleGamePlugin {
    #[config]
    pub config: GameConfig,

    #[resource]
    pub stats: GameStats,

    #[resource]
    pub progress: PlayerProgress,
}

fn main() {
    println!("=== IssunBevyPlugin Test ===\n");

    let mut app = App::new();

    // Test 1: Default plugin
    println!("Test 1: Default plugin");
    app.add_plugins(MinimalPlugins)
        .add_plugins(SimpleGamePlugin::default());

    app.update();

    // Verify resources are registered
    assert!(
        app.world().get_resource::<GameConfig>().is_some(),
        "GameConfig should be registered"
    );
    assert!(
        app.world().get_resource::<GameStats>().is_some(),
        "GameStats should be registered"
    );
    assert!(
        app.world().get_resource::<PlayerProgress>().is_some(),
        "PlayerProgress should be registered"
    );
    println!("✅ All resources registered\n");

    // Test 2: Builder methods
    println!("Test 2: Builder methods");
    let mut app2 = App::new();
    app2.add_plugins(MinimalPlugins).add_plugins(
        SimpleGamePlugin::default()
            .with_config(GameConfig {
                difficulty: 2.0,
                max_level: 100,
            })
            .with_stats(GameStats {
                score: 1000,
                level: 5,
            })
            .with_progress(PlayerProgress { xp: 500 }),
    );

    app2.update();

    let config = app2.world().get_resource::<GameConfig>().unwrap();
    assert_eq!(config.difficulty, 2.0);
    assert_eq!(config.max_level, 100);

    let stats = app2.world().get_resource::<GameStats>().unwrap();
    assert_eq!(stats.score, 1000);
    assert_eq!(stats.level, 5);

    let progress = app2.world().get_resource::<PlayerProgress>().unwrap();
    assert_eq!(progress.xp, 500);

    println!("✅ Builder methods work correctly\n");

    // Test 3: Skip field
    test_skip::run_skip_test();

    // Test 4: auto_register_types
    test_auto_register();

    // Test 5: messages auto-registration
    test_messages::run_messages_test();

    // Test 6-8: Phase 2.2 features
    test_phase22::run_phase22_tests();

    println!("\n=== All tests passed! ===");
}

// Test auto_register_types
#[derive(Resource, Clone, Debug, Default, Reflect)]
#[reflect(Resource)]
pub struct ReflectedResource {
    pub value: u32,
}

#[derive(Default, IssunBevyPlugin)]
#[plugin(name = "auto_register_test", auto_register_types = true)]
pub struct AutoRegisterPlugin {
    #[resource]
    pub reflected: ReflectedResource,
}

fn test_auto_register() {
    println!("\nTest 4: auto_register_types");

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AutoRegisterPlugin::default());

    app.update();

    // Verify resource is registered
    assert!(
        app.world().get_resource::<ReflectedResource>().is_some(),
        "ReflectedResource should be registered"
    );

    // Verify type is registered for Reflection
    let registry = app.world().resource::<AppTypeRegistry>();
    let registry = registry.read();
    assert!(
        registry.get(std::any::TypeId::of::<ReflectedResource>()).is_some(),
        "ReflectedResource should be registered in type registry"
    );

    println!("✅ auto_register_types works correctly");
}
