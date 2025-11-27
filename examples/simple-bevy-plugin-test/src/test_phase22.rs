//! Test for Phase 2.2 features
//!
//! Tests: components, startup_systems, update_systems

use bevy::prelude::*;
use issun_bevy::IssunCorePlugin;
use issun_macros::IssunBevyPlugin;

// Test components for registration
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub struct Player {
    pub name: String,
}

#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub struct Enemy {
    pub hp: u32,
}

#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct GameData {
    pub value: u32,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct TestLog {
    pub messages: Vec<String>,
}

/// Test plugin with Phase 2.2 features
#[derive(Default, IssunBevyPlugin)]
#[plugin(
    name = "phase22_test",
    components = [Player, Enemy, Health],
    startup_systems = [setup_test],
    update_systems = [update_test],
)]
pub struct Phase22TestPlugin {
    #[resource]
    pub data: GameData,

    #[resource]
    pub log: TestLog,
}

fn setup_test(mut log: ResMut<TestLog>) {
    log.messages.push("setup_test called".to_string());
}

fn update_test(mut log: ResMut<TestLog>) {
    log.messages.push("update_test called".to_string());
}

pub fn run_phase22_tests() {
    test_components();
    test_startup_systems();
    test_update_systems();
}

fn test_components() {
    println!("\nTest 6: Component Registration (Phase 2.2)");

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(IssunCorePlugin)
        .add_plugins(Phase22TestPlugin::default());

    app.update();

    // Verify resource is registered
    assert!(
        app.world().get_resource::<GameData>().is_some(),
        "GameData should be registered"
    );

    // Verify components are registered for Reflection
    let registry = app.world().resource::<AppTypeRegistry>();
    let registry = registry.read();

    assert!(
        registry.get(std::any::TypeId::of::<Player>()).is_some(),
        "Player should be registered in type registry"
    );
    assert!(
        registry.get(std::any::TypeId::of::<Enemy>()).is_some(),
        "Enemy should be registered in type registry"
    );
    assert!(
        registry.get(std::any::TypeId::of::<Health>()).is_some(),
        "Health should be registered in type registry"
    );

    println!("✅ Component registration works correctly");
}

fn test_startup_systems() {
    println!("\nTest 7: Startup Systems (Phase 2.2)");

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(IssunCorePlugin)
        .add_plugins(Phase22TestPlugin::default());

    // update() runs Startup systems
    app.update();

    // Verify startup system ran
    let log = app.world().get_resource::<TestLog>().unwrap();
    assert!(
        log.messages.contains(&"setup_test called".to_string()),
        "setup_test should have been called"
    );

    println!("✅ Startup systems work correctly");
}

fn test_update_systems() {
    println!("\nTest 8: Update Systems (Phase 2.2)");

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(IssunCorePlugin)
        .add_plugins(Phase22TestPlugin::default());

    // First update runs Startup, second runs Update
    app.update();

    let log = app.world().get_resource::<TestLog>().unwrap();
    let initial_count = log.messages.iter()
        .filter(|msg| *msg == "update_test called")
        .count();

    // Run another update to trigger Update systems again
    app.update();

    let log = app.world().get_resource::<TestLog>().unwrap();
    let final_count = log.messages.iter()
        .filter(|msg| *msg == "update_test called")
        .count();

    assert!(
        final_count > initial_count,
        "update_test should be called on each update"
    );

    println!("✅ Update systems work correctly");
}
