//! RPG Arena - MOD System E2E Test Game
//!
//! A simple turn-based combat game demonstrating the ISSUN MOD system.
//! Players can load MODs to change game difficulty, HP, and inventory settings.

mod arena;
mod combat_state;
mod fighter;
mod item;
mod ui;

use arena::Arena;
use combat_state::CombatState;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use issun::modding::{ModLoadRequested, ModSystemPlugin, ModUnloadRequested};
use issun::prelude::*;
use issun::system::System;
use issun_mod_rhai::RhaiLoader;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

const TICK_RATE: Duration = Duration::from_millis(100);

#[tokio::main]
async fn main() -> io::Result<()> {
    // Initialize terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize ISSUN framework with MOD system
    let game = GameBuilder::new()
        .with_plugin(ModSystemPlugin::new().with_loader(RhaiLoader::new()))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?
        .build()
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let Game {
        mut resources,
        mut systems,
        ..
    } = game;

    // Add CombatConfig and InventoryConfig for MOD system
    resources.insert(
        "combat_config",
        issun::plugin::CombatConfig {
            enabled: true,
            default_max_hp: 100,
            difficulty_multiplier: 1.0,
        },
    );
    resources.insert(
        "inventory_config",
        issun::plugin::InventoryConfig {
            enabled: true,
            max_slots: 10,
            allow_stacking: true,
        },
    );

    // Create arena
    let arena = Arena::new(100, 10, true);
    resources.insert("arena", arena);

    // Track loaded MODs
    let mut loaded_mods: Vec<String> = Vec::new();

    // Run game loop
    let result = run_game_loop(&mut terminal, &mut resources, &mut systems, &mut loaded_mods).await;

    // Cleanup terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_game_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    resources: &mut ResourceContext,
    systems: &mut SystemContext,
    loaded_mods: &mut Vec<String>,
) -> io::Result<()> {
    let mut last_tick = std::time::Instant::now();

    loop {
        // Update arena from config changes
        update_arena_from_config(resources);

        // Render UI
        terminal.draw(|f| {
            if let Some(arena) = resources.try_get::<Arena>("arena") {
                ui::render(f, &arena, loaded_mods);
            }
        })?;

        // Handle input
        let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('n') => handle_new_combat(resources),
                        KeyCode::Char(' ') => handle_player_attack(resources),
                        KeyCode::Char('m') => handle_load_mod(resources, loaded_mods),
                        KeyCode::Char('u') => handle_unload_mod(resources, loaded_mods),
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            let index = c.to_digit(10).unwrap() as usize;
                            handle_use_item(resources, index);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Tick
        if last_tick.elapsed() >= TICK_RATE {
            handle_enemy_turn(resources);

            // Run MOD systems
            if let Some(mod_bridge) = systems.try_get_mut::<issun::engine::ModBridgeSystem>("mod_bridge_system") {
                mod_bridge.update(resources).await;
            }

            last_tick = std::time::Instant::now();
        }
    }
}

fn update_arena_from_config(resources: &mut ResourceContext) {
    let combat_config = resources
        .try_get::<issun::plugin::CombatConfig>("combat_config")
        .cloned();
    let inventory_config = resources
        .try_get::<issun::plugin::InventoryConfig>("inventory_config")
        .cloned();

    if let (Some(combat), Some(inventory), Some(mut arena)) = (
        combat_config,
        inventory_config,
        resources.try_get_mut::<Arena>("arena"),
    ) {
        arena.update_difficulty(combat.difficulty_multiplier);

        // Update inventory settings if changed
        if arena.inventory.max_slots != inventory.max_slots
            || arena.inventory.allow_stacking != inventory.allow_stacking
        {
            arena.inventory.max_slots = inventory.max_slots;
            arena.inventory.allow_stacking = inventory.allow_stacking;
        }
    }
}

fn handle_new_combat(resources: &mut ResourceContext) {
    if let Some(mut arena) = resources.try_get_mut::<Arena>("arena") {
        let combat_config = resources
            .try_get::<issun::plugin::CombatConfig>("combat_config")
            .cloned()
            .unwrap_or_default();
        let inventory_config = resources
            .try_get::<issun::plugin::InventoryConfig>("inventory_config")
            .cloned()
            .unwrap_or_default();

        arena.reset(
            combat_config.default_max_hp,
            inventory_config.max_slots,
            inventory_config.allow_stacking,
        );
        arena.update_difficulty(combat_config.difficulty_multiplier);
        arena.start_combat();
    }
}

fn handle_player_attack(resources: &mut ResourceContext) {
    if let Some(mut arena) = resources.try_get_mut::<Arena>("arena") {
        if arena.combat.state == CombatState::PlayerTurn {
            arena.player_attack();
        }
    }
}

fn handle_use_item(resources: &mut ResourceContext, index: usize) {
    if let Some(mut arena) = resources.try_get_mut::<Arena>("arena") {
        arena.use_item(index).ok();
    }
}

fn handle_enemy_turn(resources: &mut ResourceContext) {
    if let Some(mut arena) = resources.try_get_mut::<Arena>("arena") {
        if arena.combat.state == CombatState::EnemyTurn {
            arena.enemy_attack();
        }
    }
}

fn handle_load_mod(resources: &mut ResourceContext, loaded_mods: &mut Vec<String>) {
    // For demo, try to load a hardcoded MOD
    let mod_path = PathBuf::from("examples/rpg-arena/mods/easy_mode.rhai");

    if let Some(mut event_bus) = resources.try_get_mut::<EventBus>("event_bus") {
        event_bus.publish(ModLoadRequested {
            path: mod_path.clone(),
        });
        event_bus.dispatch();

        // Track loaded MOD
        if let Some(file_name) = mod_path.file_stem() {
            let mod_name = file_name.to_string_lossy().to_string();
            if !loaded_mods.contains(&mod_name) {
                loaded_mods.push(mod_name);
            }
        }
    }
}

fn handle_unload_mod(resources: &mut ResourceContext, loaded_mods: &mut Vec<String>) {
    if loaded_mods.is_empty() {
        return;
    }

    // Unload the first MOD
    let mod_id = loaded_mods.remove(0);

    if let Some(mut event_bus) = resources.try_get_mut::<EventBus>("event_bus") {
        event_bus.publish(ModUnloadRequested { mod_id });
        event_bus.dispatch();
    }
}
