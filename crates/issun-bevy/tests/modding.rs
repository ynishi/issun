//! Modding Plugin Tests

use bevy::prelude::*;
use issun_bevy::plugins::modding::{DiscoveredMods, ModdingPlugin};
use std::fs;

#[test]
fn test_modding_plugin_builds() {
    // Test that ModdingPlugin can be added to an app without panic
    let mut app = App::new();
    app.add_plugins(ModdingPlugin);

    // Run one frame to ensure startup systems execute
    app.update();
}

#[test]
fn test_discover_mods_dir_missing() {
    // Test that missing mods/ directory doesn't cause panic

    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(ModdingPlugin);

    app.update();

    // Verify DiscoveredMods resource exists
    let discovered = app.world().resource::<DiscoveredMods>();

    // Should be empty since mods/ directory doesn't exist
    assert_eq!(discovered.ron_files.len(), 0);
}

#[test]
fn test_discovered_mods_resource_initialized() {
    // Test that DiscoveredMods resource is properly initialized

    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(ModdingPlugin);

    app.update();

    // Verify resource exists
    assert!(app.world().contains_resource::<DiscoveredMods>());

    let discovered = app.world().resource::<DiscoveredMods>();
    assert!(discovered.ron_files.is_empty());
}

#[test]
fn test_ron_file_exists() {
    // Test that test fixture .ron file exists and is readable

    let result = fs::read_to_string("tests/fixtures/test_mod.ron");
    assert!(
        result.is_ok(),
        "Failed to read test fixture: {:?}",
        result.err()
    );

    let content = result.unwrap();
    assert!(!content.is_empty(), "Test fixture is empty");
    assert!(
        content.contains("Health"),
        "Test fixture should contain Health component"
    );
}
