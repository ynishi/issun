//! End-to-End Integration Tests for RPG Arena MOD System
//!
//! These tests validate the complete MOD system flow in a real game scenario:
//! 1. Game starts with default settings
//! 2. MOD is loaded and changes game parameters
//! 3. Game behavior reflects the MOD changes
//! 4. MOD can be unloaded and settings reset

use issun::context::Context;
use issun::engine::ModBridgeSystem;
use issun::event::EventBus;
use issun::modding::{ModLoadRequested, ModLoadSystem, ModLoaderState};
use issun::plugin::{CombatConfig, InventoryConfig};
use issun::system::System;
use issun_mod_rhai::RhaiLoader;
use std::path::PathBuf;
use std::sync::Arc;

/// Helper to setup test environment with MOD system
async fn setup_mod_system() -> Context {
    let mut ctx = Context::new();

    // Add event bus
    ctx.insert("event_bus", EventBus::new());

    // Add plugin configs (default values)
    ctx.insert(
        "combat_config",
        CombatConfig {
            enabled: true,
            default_max_hp: 100,
            difficulty_multiplier: 1.0,
        },
    );

    ctx.insert(
        "inventory_config",
        InventoryConfig {
            enabled: true,
            max_slots: 10,
            allow_stacking: true,
        },
    );

    // Add MOD loader
    let loader = Arc::new(RhaiLoader::new());
    ctx.insert("mod_loader_state", ModLoaderState::new());
    ctx.insert("mod_loader", loader);

    ctx
}

/// Helper to load a MOD and process events
async fn load_mod(ctx: &mut Context, mod_path: &str) {
    // Publish load request
    if let Some(mut event_bus) = ctx.get_mut::<EventBus>("event_bus") {
        event_bus.publish(ModLoadRequested {
            path: PathBuf::from(mod_path),
        });
        event_bus.dispatch();
    }

    // Run ModLoadSystem
    let mut mod_load_system = ModLoadSystem::new();
    mod_load_system.update(ctx).await;

    // Run ModBridgeSystem to apply changes
    let mut mod_bridge_system = ModBridgeSystem::new();
    mod_bridge_system.update(ctx).await;
}

#[tokio::test]
async fn test_default_settings() {
    let ctx = setup_mod_system().await;

    // Verify default settings
    let combat_config = ctx
        .get::<CombatConfig>("combat_config")
        .expect("Combat config not found");
    assert_eq!(combat_config.default_max_hp, 100);
    assert_eq!(combat_config.difficulty_multiplier, 1.0);

    let inventory_config = ctx
        .get::<InventoryConfig>("inventory_config")
        .expect("Inventory config not found");
    assert_eq!(inventory_config.max_slots, 10);
    assert!(inventory_config.allow_stacking);
}

#[tokio::test]
async fn test_easy_mode_mod() {
    let mut ctx = setup_mod_system().await;

    // Load easy_mode.rhai
    load_mod(&mut ctx, "examples/rpg-arena/mods/easy_mode.rhai").await;

    // Verify Easy Mode settings
    let combat_config = ctx
        .get::<CombatConfig>("combat_config")
        .expect("Combat config not found");
    assert_eq!(
        combat_config.default_max_hp, 200,
        "Easy Mode should set max_hp to 200"
    );
    assert_eq!(
        combat_config.difficulty_multiplier, 0.5,
        "Easy Mode should set difficulty to 0.5"
    );

    let inventory_config = ctx
        .get::<InventoryConfig>("inventory_config")
        .expect("Inventory config not found");
    assert_eq!(
        inventory_config.max_slots, 30,
        "Easy Mode should set max_slots to 30"
    );
    assert!(
        inventory_config.allow_stacking,
        "Easy Mode should enable stacking"
    );
}

#[tokio::test]
async fn test_hard_mode_mod() {
    let mut ctx = setup_mod_system().await;

    // Load hard_mode.rhai
    load_mod(&mut ctx, "examples/rpg-arena/mods/hard_mode.rhai").await;

    // Verify Hard Mode settings
    let combat_config = ctx
        .get::<CombatConfig>("combat_config")
        .expect("Combat config not found");
    assert_eq!(
        combat_config.default_max_hp, 50,
        "Hard Mode should set max_hp to 50"
    );
    assert_eq!(
        combat_config.difficulty_multiplier, 2.0,
        "Hard Mode should set difficulty to 2.0"
    );

    let inventory_config = ctx
        .get::<InventoryConfig>("inventory_config")
        .expect("Inventory config not found");
    assert_eq!(
        inventory_config.max_slots, 5,
        "Hard Mode should set max_slots to 5"
    );
    assert!(
        !inventory_config.allow_stacking,
        "Hard Mode should disable stacking"
    );
}

#[tokio::test]
async fn test_debug_mode_mod() {
    let mut ctx = setup_mod_system().await;

    // Load debug_mode.rhai
    load_mod(&mut ctx, "examples/rpg-arena/mods/debug_mode.rhai").await;

    // Verify Debug Mode settings
    let combat_config = ctx
        .get::<CombatConfig>("combat_config")
        .expect("Combat config not found");
    assert_eq!(
        combat_config.default_max_hp, 9999,
        "Debug Mode should set max_hp to 9999"
    );
    assert_eq!(
        combat_config.difficulty_multiplier, 0.1,
        "Debug Mode should set difficulty to 0.1"
    );

    let inventory_config = ctx
        .get::<InventoryConfig>("inventory_config")
        .expect("Inventory config not found");
    assert_eq!(
        inventory_config.max_slots, 999,
        "Debug Mode should set max_slots to 999"
    );
    assert!(
        inventory_config.allow_stacking,
        "Debug Mode should enable stacking"
    );
}

#[tokio::test]
async fn test_mod_override_sequence() {
    let mut ctx = setup_mod_system().await;

    // Load Easy Mode first
    load_mod(&mut ctx, "examples/rpg-arena/mods/easy_mode.rhai").await;

    let combat_config = ctx.get::<CombatConfig>("combat_config").unwrap();
    assert_eq!(combat_config.default_max_hp, 200);

    // Load Hard Mode (should override)
    load_mod(&mut ctx, "examples/rpg-arena/mods/hard_mode.rhai").await;

    let combat_config = ctx.get::<CombatConfig>("combat_config").unwrap();
    assert_eq!(
        combat_config.default_max_hp, 50,
        "Hard Mode should override Easy Mode settings"
    );
    assert_eq!(combat_config.difficulty_multiplier, 2.0);
}

#[tokio::test]
async fn test_mod_loader_state() {
    let mut ctx = setup_mod_system().await;

    // Initially no MODs loaded
    let loader_state = ctx
        .get::<ModLoaderState>("mod_loader_state")
        .expect("ModLoaderState not found");
    assert_eq!(loader_state.loaded_mods.len(), 0);

    // Load a MOD
    load_mod(&mut ctx, "examples/rpg-arena/mods/easy_mode.rhai").await;

    // Check MOD is tracked
    let loader_state = ctx.get::<ModLoaderState>("mod_loader_state").unwrap();
    assert_eq!(
        loader_state.loaded_mods.len(),
        1,
        "One MOD should be loaded"
    );

    let mod_handle = &loader_state.loaded_mods[0];
    assert_eq!(mod_handle.metadata.name, "Easy Mode");
    assert_eq!(mod_handle.metadata.version, "1.0.0");
}

#[tokio::test]
async fn test_nonexistent_mod() {
    let mut ctx = setup_mod_system().await;

    // Try to load non-existent MOD
    load_mod(&mut ctx, "examples/rpg-arena/mods/nonexistent.rhai").await;

    // Settings should remain at defaults
    let combat_config = ctx.get::<CombatConfig>("combat_config").unwrap();
    assert_eq!(
        combat_config.default_max_hp, 100,
        "Settings should remain default when MOD fails to load"
    );

    // No MODs should be loaded
    let loader_state = ctx.get::<ModLoaderState>("mod_loader_state").unwrap();
    assert_eq!(
        loader_state.loaded_mods.len(),
        0,
        "No MODs should be loaded"
    );
}
