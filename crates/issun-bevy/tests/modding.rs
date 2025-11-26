//! Modding Plugin Tests

use bevy::prelude::*;
use issun_bevy::plugins::modding::{DiscoveredMods, ModdingConfig, ModdingPlugin};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_modding_plugin_builds() {
    // Test that ModdingPlugin can be added to an app without panic
    let mut app = App::new();
    app.add_plugins(ModdingPlugin);

    // Run one frame to ensure startup systems execute
    app.update();
}

#[test]
fn test_discover_mods_from_empty_directory() {
    // Test that empty mods directory doesn't cause errors

    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();

    let mut app = App::new();
    app.insert_resource(ModdingConfig::with_directory(temp_dir.path()))
        .add_plugins(MinimalPlugins)
        .add_plugins(ModdingPlugin);

    app.update();

    // Verify DiscoveredMods resource exists
    let discovered = app.world().resource::<DiscoveredMods>();

    // Should be empty
    assert_eq!(discovered.ron_files.len(), 0);

    // temp_dir is automatically cleaned up when dropped
}

#[test]
fn test_discovered_mods_resource_initialized() {
    // Test that DiscoveredMods resource is properly initialized

    let temp_dir = TempDir::new().unwrap();

    let mut app = App::new();
    app.insert_resource(ModdingConfig::with_directory(temp_dir.path()))
        .add_plugins(MinimalPlugins)
        .add_plugins(ModdingPlugin);

    app.update();

    // Verify resource exists
    assert!(app.world().contains_resource::<DiscoveredMods>());

    // Resource should exist
    let _discovered = app.world().resource::<DiscoveredMods>();
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

#[test]
fn test_discover_combat_mod() {
    // Test that combat_test.ron is discovered from temp mods directory

    use issun_bevy::plugins::combat::CombatPlugin;

    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let mods_path = temp_dir.path();

    // Create combat_test.ron in temp directory
    let combat_mod_content = r#"(
  resources: [],
  entities: [
    (
      entity: 0,
      components: [
        {
          "issun_bevy::plugins::combat::components::Combatant": (
            name: "Test Goblin",
          ),
        },
        {
          "issun_bevy::plugins::combat::components::Health": (
            current: 50,
            max: 50,
          ),
        },
      ],
    ),
  ],
)"#;

    fs::write(mods_path.join("combat_test.ron"), combat_mod_content).unwrap();

    let mut app = App::new();
    app.insert_resource(ModdingConfig::with_directory(mods_path))
        .add_plugins(MinimalPlugins)
        .add_plugins(CombatPlugin::default())
        .add_plugins(ModdingPlugin);

    // Run startup systems (discover_mods runs in Startup)
    app.update();

    // Check if mod was discovered
    let discovered = app.world().resource::<DiscoveredMods>();

    assert_eq!(
        discovered.ron_files.len(),
        1,
        "Expected 1 mod file, found {}",
        discovered.ron_files.len()
    );

    // Check if combat_test.ron is in the list
    let has_combat_mod = discovered
        .ron_files
        .iter()
        .any(|p| p.file_name().unwrap() == "combat_test.ron");

    assert!(
        has_combat_mod,
        "combat_test.ron not found in discovered mods: {:?}",
        discovered.ron_files
    );

    // temp_dir is automatically cleaned up when dropped
}

#[test]
fn test_discover_accounting_mod() {
    // Test that accounting_test.ron is discovered from temp mods directory

    use issun_bevy::plugins::accounting::AccountingPlugin;

    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let mods_path = temp_dir.path();

    // Create accounting_test.ron in temp directory
    let accounting_mod_content = r#"(
  resources: [],
  entities: [
    (
      entity: 0,
      components: [
        {
          "issun_bevy::plugins::accounting::components::Organization": (
            name: "Test Corp",
          ),
        },
        {
          "issun_bevy::plugins::accounting::components::BudgetLedger": (
            cash: (5000),
            research_pool: (1000),
            ops_pool: (800),
            reserve: (500),
            innovation_fund: (0),
            security_fund: (0),
          ),
        },
      ],
    ),
  ],
)"#;

    fs::write(
        mods_path.join("accounting_test.ron"),
        accounting_mod_content,
    )
    .unwrap();

    let mut app = App::new();
    app.insert_resource(ModdingConfig::with_directory(mods_path))
        .add_plugins(MinimalPlugins)
        .add_plugins(AccountingPlugin::default())
        .add_plugins(ModdingPlugin);

    // Run startup systems
    app.update();

    // Check if mod was discovered
    let discovered = app.world().resource::<DiscoveredMods>();

    assert_eq!(
        discovered.ron_files.len(),
        1,
        "Expected 1 mod file, found {}",
        discovered.ron_files.len()
    );

    // Check if accounting_test.ron is in the list
    let has_accounting_mod = discovered
        .ron_files
        .iter()
        .any(|p| p.file_name().unwrap() == "accounting_test.ron");

    assert!(
        has_accounting_mod,
        "accounting_test.ron not found in discovered mods: {:?}",
        discovered.ron_files
    );

    // temp_dir is automatically cleaned up when dropped
}

#[test]
fn test_discover_multiple_mods() {
    // Test that multiple .ron files are discovered

    let temp_dir = TempDir::new().unwrap();
    let mods_path = temp_dir.path();

    // Create multiple mod files
    fs::write(mods_path.join("mod1.ron"), "(resources: [], entities: [])").unwrap();
    fs::write(mods_path.join("mod2.ron"), "(resources: [], entities: [])").unwrap();
    fs::write(mods_path.join("mod3.ron"), "(resources: [], entities: [])").unwrap();

    // Create a non-.ron file (should be ignored)
    fs::write(mods_path.join("readme.txt"), "This is not a mod").unwrap();

    let mut app = App::new();
    app.insert_resource(ModdingConfig::with_directory(mods_path))
        .add_plugins(MinimalPlugins)
        .add_plugins(ModdingPlugin);

    app.update();

    let discovered = app.world().resource::<DiscoveredMods>();

    // Should find exactly 3 .ron files
    assert_eq!(
        discovered.ron_files.len(),
        3,
        "Expected 3 mod files, found {}",
        discovered.ron_files.len()
    );

    // temp_dir is automatically cleaned up when dropped
}
