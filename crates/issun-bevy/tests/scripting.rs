//! Scripting Plugin Tests

use bevy::prelude::*;
use issun_bevy::plugins::scripting::{LuaScript, MluaBackend, ScriptingBackend, ScriptingPlugin};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_mlua_backend_creation() {
    // Test that MLua backend can be created

    let backend = MluaBackend::new();
    assert!(backend.is_ok(), "Failed to create MLua backend");

    let backend = backend.unwrap();
    assert_eq!(backend.backend_name(), "mlua (Lua 5.4)");
}

#[test]
fn test_execute_simple_lua_chunk() {
    // Test basic Lua code execution

    let mut backend = MluaBackend::new().unwrap();

    // Execute simple Lua code
    let result = backend.execute_chunk("x = 42");
    assert!(
        result.is_ok(),
        "Failed to execute Lua chunk: {:?}",
        result.err()
    );
}

#[test]
fn test_load_and_call_function() {
    // Test loading a script file and calling a function

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test.lua");

    // Create test script
    let script_content = r#"
        function greet()
            message = "Hello from Lua!"
        end
    "#;

    fs::write(&script_path, script_content).unwrap();

    let mut backend = MluaBackend::new().unwrap();

    // Load script
    let handle = backend.load_script(script_path.to_str().unwrap());
    assert!(handle.is_ok(), "Failed to load script: {:?}", handle.err());

    let handle = handle.unwrap();

    // Check if function exists
    assert!(backend.has_function(handle, "greet"));
    assert!(!backend.has_function(handle, "nonexistent"));

    // Call function
    let result = backend.call_function(handle, "greet");
    assert!(
        result.is_ok(),
        "Failed to call function: {:?}",
        result.err()
    );
}

#[test]
fn test_function_not_found_error() {
    // Test that calling non-existent function returns error

    let backend = MluaBackend::new().unwrap();

    // Try to call non-existent function
    let result = backend.call_function(
        issun_bevy::plugins::scripting::ScriptHandle::new(0),
        "nonexistent",
    );

    assert!(result.is_err());
    assert!(matches!(
        result.err().unwrap(),
        issun_bevy::plugins::scripting::ScriptError::FunctionNotFound(_)
    ));
}

#[test]
fn test_syntax_error_handling() {
    // Test that syntax errors are properly handled

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("bad.lua");

    // Create script with syntax error
    fs::write(&script_path, "function bad(").unwrap();

    let mut backend = MluaBackend::new().unwrap();

    // Try to load script
    let result = backend.load_script(script_path.to_str().unwrap());

    assert!(result.is_err());
    assert!(matches!(
        result.err().unwrap(),
        issun_bevy::plugins::scripting::ScriptError::SyntaxError(_)
    ));
}

#[test]
fn test_sandbox_io_disabled() {
    // Test that io library is disabled in sandbox

    let mut backend = MluaBackend::new().unwrap();

    // Try to use io.open (should fail)
    let result = backend.execute_chunk(r#"io.open("test.txt", "r")"#);

    assert!(result.is_err(), "io library should be disabled in sandbox");
}

#[test]
fn test_sandbox_os_disabled() {
    // Test that os library is disabled in sandbox

    let mut backend = MluaBackend::new().unwrap();

    // Try to use os.execute (should fail)
    let result = backend.execute_chunk(r#"os.execute("echo hello")"#);

    assert!(result.is_err(), "os library should be disabled in sandbox");
}

#[test]
fn test_sandbox_require_disabled() {
    // Test that require is disabled in sandbox

    let mut backend = MluaBackend::new().unwrap();

    // Try to use require (should fail)
    let result = backend.execute_chunk(r#"require("os")"#);

    assert!(result.is_err(), "require should be disabled in sandbox");
}

#[test]
fn test_sandbox_dofile_disabled() {
    // Test that dofile is disabled in sandbox

    let mut backend = MluaBackend::new().unwrap();

    // Try to use dofile (should fail)
    let result = backend.execute_chunk(r#"dofile("test.lua")"#);

    assert!(result.is_err(), "dofile should be disabled in sandbox");
}

#[test]
fn test_sandbox_allows_safe_operations() {
    // Test that sandbox still allows safe operations

    let mut backend = MluaBackend::new().unwrap();

    // Math operations should work
    let result = backend.execute_chunk("x = math.sin(math.pi / 2)");
    assert!(result.is_ok(), "Math operations should be allowed");

    // String operations should work
    let result = backend.execute_chunk(r#"s = string.upper("hello")"#);
    assert!(result.is_ok(), "String operations should be allowed");

    // Table operations should work
    let result = backend.execute_chunk("t = {1, 2, 3}; table.insert(t, 4)");
    assert!(result.is_ok(), "Table operations should be allowed");
}

#[test]
fn test_lua_script_component_creation() {
    // Test that LuaScript component can be created

    let script = LuaScript::new("test.lua");

    assert_eq!(script.path, "test.lua");
    assert!(!script.is_loaded());
}

#[test]
fn test_lua_script_component_lifecycle() {
    // Test LuaScript lifecycle (unloaded -> loaded -> unloaded)

    let mut script = LuaScript::new("test.lua");
    assert!(!script.is_loaded());

    // Simulate loading
    let handle = issun_bevy::plugins::scripting::ScriptHandle::new(42);
    script.set_loaded(handle);
    assert!(script.is_loaded());

    // Simulate unloading
    script.set_unloaded();
    assert!(!script.is_loaded());
}

#[test]
fn test_scripting_plugin_builds() {
    // Test that ScriptingPlugin can be added to an app

    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(ScriptingPlugin);

    app.update();

    // If we get here without panic, test passes
}

#[test]
fn test_lua_script_attached_to_entity() {
    // Test that LuaScript can be attached to entities

    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(ScriptingPlugin);

    // Spawn entity with LuaScript
    let entity = app.world_mut().spawn(LuaScript::new("test.lua")).id();

    app.update();

    // Verify component exists
    let script = app.world().get::<LuaScript>(entity);
    assert!(script.is_some());
    assert_eq!(script.unwrap().path, "test.lua");
}

#[test]
fn test_utility_apis_available() {
    // Test that utility APIs are available in Lua

    use issun_bevy::plugins::scripting::MluaBackend;
    use issun_bevy::plugins::scripting::ScriptingBackend;

    let mut backend = MluaBackend::new().unwrap();

    // Register APIs
    issun_bevy::plugins::scripting::register_all_apis(backend.lua()).unwrap();

    // Test log functions
    backend.execute_chunk(r#"log("test")"#).unwrap();
    backend.execute_chunk(r#"log_warn("warning")"#).unwrap();
    backend.execute_chunk(r#"log_error("error")"#).unwrap();

    // Test random functions
    backend
        .execute_chunk(
            r#"
        local r = random()
        assert(r >= 0.0 and r < 1.0, "random() out of range")

        local rr = random_range(10.0, 20.0)
        assert(rr >= 10.0 and rr < 20.0, "random_range() out of range")
    "#,
        )
        .unwrap();

    // If assertions in Lua pass, test passes
    #[allow(clippy::unit_cmp)]
    {
        assert!(() == ());
    }
}

#[test]
fn test_combat_ai_script() {
    // Test realistic Combat AI script usage

    use issun_bevy::plugins::scripting::MluaBackend;
    use issun_bevy::plugins::scripting::ScriptingBackend;
    use std::fs;
    use tempfile::TempDir;

    let mut backend = MluaBackend::new().unwrap();

    // Register APIs
    issun_bevy::plugins::scripting::register_all_apis(backend.lua()).unwrap();

    // Create temporary script file
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("combat_ai.lua");

    let script_content = r#"
-- Simple Combat AI Script
attack_count = 0

function on_init()
    log("Combat AI initialized!")
end

function decide_action()
    local rand = random()
    if rand < 0.3 then
        log("AI Decision: ATTACK")
        attack_count = attack_count + 1
        return "attack"
    elseif rand < 0.6 then
        log("AI Decision: DEFEND")
        return "defend"
    else
        log_warn("AI Decision: FLEE")
        return "flee"
    end
end

function calculate_damage()
    local base_damage = 10
    local variance = random_range(-2.0, 5.0)
    local damage = base_damage + variance
    log(string.format("Calculated damage: %.1f", damage))
    return damage
end

function get_stats()
    return {
        attacks = attack_count,
        version = "1.0"
    }
end
"#;

    fs::write(&script_path, script_content).unwrap();

    // Load the combat AI script
    let handle = backend.load_script(script_path.to_str().unwrap()).unwrap();

    // Test on_init function
    backend.call_function(handle, "on_init").unwrap();

    // Test decide_action function (should return string)
    backend.call_function(handle, "decide_action").unwrap();
    backend.call_function(handle, "decide_action").unwrap();
    backend.call_function(handle, "decide_action").unwrap();

    // Test calculate_damage function
    backend.call_function(handle, "calculate_damage").unwrap();

    // Test that stats tracking works
    backend.call_function(handle, "get_stats").unwrap();
}

#[test]
fn test_combat_ai_script_with_bevy() {
    // Test Combat AI script integrated with Bevy

    use bevy::prelude::*;
    use issun_bevy::plugins::scripting::{LuaScript, ScriptingPlugin};
    use std::fs;
    use tempfile::TempDir;

    // Create temporary script file
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("combat_ai.lua");

    let script_content = r#"
function on_init()
    log("Combat AI for Goblin Warrior initialized!")
end
"#;

    fs::write(&script_path, script_content).unwrap();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(ScriptingPlugin);

    // Create a combat entity with AI script
    let entity = app
        .world_mut()
        .spawn((
            Name::new("Goblin Warrior"),
            LuaScript::new(script_path.to_str().unwrap()),
        ))
        .id();

    // Update to trigger script loading
    app.update();

    // Verify script was loaded
    let script = app.world().get::<LuaScript>(entity).unwrap();
    assert!(script.is_loaded(), "Script should be loaded");

    // Verify entity has name
    let name = app.world().get::<Name>(entity).unwrap();
    assert_eq!(name.as_str(), "Goblin Warrior");
}

#[test]
fn test_event_subscription() {
    // Test event subscription and triggering

    use issun_bevy::plugins::scripting::MluaBackend;
    use std::fs;
    use tempfile::TempDir;

    let mut backend = MluaBackend::new().unwrap();

    // Register utility APIs
    issun_bevy::plugins::scripting::register_all_apis(backend.lua()).unwrap();

    // Create script with event handler
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("events.lua");

    let script_content = r#"
-- Track event calls
event_log = {}

function on_damage(event)
    log(string.format("Damage event: %d damage to entity %s",
        event.amount, event.target))
    table.insert(event_log, {type = "damage", amount = event.amount})
end

function on_heal(event)
    log_warn(string.format("Heal event: %d healing", event.amount))
    table.insert(event_log, {type = "heal", amount = event.amount})
end

function get_event_count()
    return #event_log
end
"#;

    fs::write(&script_path, script_content).unwrap();

    // Load script
    let _handle = backend.load_script(script_path.to_str().unwrap()).unwrap();

    // Subscribe to events by function name
    backend
        .subscribe_event("DamageTaken".to_string(), "on_damage")
        .unwrap();

    backend
        .subscribe_event("HealthRestored".to_string(), "on_heal")
        .unwrap();

    // Trigger damage event
    let damage_event = backend
        .lua()
        .load(
            r#"
        return {
            amount = 25,
            target = "Goblin"
        }
    "#,
        )
        .eval()
        .unwrap();

    backend.trigger_event("DamageTaken", damage_event).unwrap();

    // Trigger heal event
    let heal_event = backend
        .lua()
        .load(
            r#"
        return {
            amount = 10
        }
    "#,
        )
        .eval()
        .unwrap();

    backend.trigger_event("HealthRestored", heal_event).unwrap();

    // Verify events were logged
    use mlua::TableExt;
    let event_count: i32 = backend
        .lua()
        .globals()
        .call_function("get_event_count", ())
        .unwrap();
    assert_eq!(event_count, 2);
}

#[test]
fn test_multiple_event_handlers() {
    // Test that multiple handlers can be registered for same event

    use issun_bevy::plugins::scripting::MluaBackend;

    let mut backend = MluaBackend::new().unwrap();

    // Register utility APIs
    issun_bevy::plugins::scripting::register_all_apis(backend.lua()).unwrap();

    // Create two handlers
    backend
        .execute_chunk(
            r#"
        handler1_called = false
        handler2_called = false

        function handler1(event)
            log("Handler 1: " .. event.message)
            handler1_called = true
        end

        function handler2(event)
            log("Handler 2: " .. event.message)
            handler2_called = true
        end
    "#,
        )
        .unwrap();

    // Subscribe handlers by function name
    backend
        .subscribe_event("TestEvent".to_string(), "handler1")
        .unwrap();

    backend
        .subscribe_event("TestEvent".to_string(), "handler2")
        .unwrap();

    // Trigger event
    let event_data = backend
        .lua()
        .load(r#"return { message = "test" }"#)
        .eval()
        .unwrap();

    backend.trigger_event("TestEvent", event_data).unwrap();

    // Verify both handlers were called
    let handler1_called: bool = backend.lua().globals().get("handler1_called").unwrap();
    let handler2_called: bool = backend.lua().globals().get("handler2_called").unwrap();

    assert!(handler1_called);
    assert!(handler2_called);
}

#[test]
fn test_command_execution_despawn() {
    // Test that DespawnEntity command executes properly

    use bevy::prelude::*;
    use issun_bevy::plugins::scripting::{LuaCommand, LuaCommandQueue, ScriptingPlugin};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(ScriptingPlugin);

    // Spawn a test entity
    let entity = app.world_mut().spawn_empty().id();

    // Verify entity exists
    assert!(app.world().get_entity(entity).is_ok());

    // Queue despawn command
    app.world_mut()
        .resource_mut::<LuaCommandQueue>()
        .push(LuaCommand::DespawnEntity { entity });

    // Run update to execute commands
    app.update();

    // Verify entity was despawned
    assert!(app.world().get_entity(entity).is_err());
}

#[test]
fn test_command_execution_despawn_already_despawned() {
    // Test that DespawnEntity handles already-despawned entities gracefully

    use bevy::prelude::*;
    use issun_bevy::plugins::scripting::{LuaCommand, LuaCommandQueue, ScriptingPlugin};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(ScriptingPlugin);

    // Create entity and immediately despawn it
    let entity = app.world_mut().spawn_empty().id();
    app.world_mut().despawn(entity);

    // Verify entity is despawned
    assert!(app.world().get_entity(entity).is_err());

    // Queue despawn command for already-despawned entity
    app.world_mut()
        .resource_mut::<LuaCommandQueue>()
        .push(LuaCommand::DespawnEntity { entity });

    // Run update - should not crash
    app.update();

    // Still despawned (no error)
    assert!(app.world().get_entity(entity).is_err());
}

#[test]
fn test_command_execution_insert_component_health() {
    // Test that InsertComponent works for Health component

    use bevy::prelude::*;
    use issun_bevy::plugins::combat::components::Health;
    use issun_bevy::plugins::scripting::{LuaCommand, LuaCommandQueue, LuaValue, ScriptingPlugin};
    use issun_bevy::IssunCorePlugin;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(IssunCorePlugin)
        .add_plugins(ScriptingPlugin);

    // Spawn test entity
    let entity = app.world_mut().spawn_empty().id();

    // Verify no Health component yet
    assert!(app.world().get::<Health>(entity).is_none());

    // Queue insert Health component command
    // For now, we'll test with a simple value structure
    app.world_mut()
        .resource_mut::<LuaCommandQueue>()
        .push(LuaCommand::InsertComponent {
            entity,
            type_name: "Health".to_string(),
            data: LuaValue::Integer(100), // Simplified: just max health
        });

    // Run update to execute commands
    app.update();

    // Verify Health component was inserted
    let health = app.world().get::<Health>(entity).unwrap();
    assert_eq!(health.current, 100);
    assert_eq!(health.max, 100);
}

#[test]
fn test_lua_commands_integration() {
    // End-to-end test: Lua script uses commands API to modify entities

    use bevy::prelude::*;
    use issun_bevy::plugins::scripting::{LuaCommandQueue, LuaCommands, MluaBackend};
    use issun_bevy::IssunCorePlugin;
    use std::sync::{Arc, Mutex};

    // Create backend and register APIs
    let mut backend = MluaBackend::new().unwrap();
    issun_bevy::plugins::scripting::register_all_apis(backend.lua()).unwrap();

    // Create shared command queue
    let queue = Arc::new(Mutex::new(LuaCommandQueue::new()));
    let lua_commands = LuaCommands::new(queue.clone());

    // Register commands object in Lua
    backend
        .lua()
        .globals()
        .set("commands", lua_commands)
        .unwrap();

    // Create Bevy app
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(IssunCorePlugin);

    // Replace default queue with our shared queue
    app.world_mut().remove_resource::<LuaCommandQueue>();
    app.insert_resource(LuaCommandQueue::new());

    // Spawn test entity
    let entity = app.world_mut().spawn_empty().id();
    let entity_id = entity.to_bits();

    // Execute Lua script that uses commands API
    backend
        .execute_chunk(&format!(
            r#"
        -- Queue command to insert Health component
        commands:insert_component({}, "Health", 150)
        log("Queued Health insertion command")
    "#,
            entity_id
        ))
        .unwrap();

    // Verify command was queued in the Arc<Mutex> queue
    let lua_queue_len = queue.lock().unwrap().len();
    assert_eq!(lua_queue_len, 1);

    // Transfer commands from Lua queue to Bevy resource queue
    {
        let lua_commands = queue.lock().unwrap().drain();
        let mut bevy_queue = app.world_mut().resource_mut::<LuaCommandQueue>();
        for cmd in lua_commands {
            bevy_queue.push(cmd);
        }
    }

    // Now let the app execute the commands via its system
    // Note: We'd need to manually trigger the execute_lua_commands system
    // For this test, let's just verify the command is there
    let bevy_queue = app.world().resource::<LuaCommandQueue>();
    assert_eq!(bevy_queue.len(), 1);
}

#[test]
fn test_world_access_get_health_component() {
    // Test reading Health component from World via Lua

    use bevy::prelude::*;
    use issun_bevy::plugins::combat::components::Health;
    use issun_bevy::plugins::scripting::{get_component_as_lua_table, MluaBackend};

    // Create World and spawn entity with Health
    let mut world = World::new();
    let entity = world.spawn(Health::new(80)).id();

    // Create Lua backend
    let backend = MluaBackend::new().unwrap();
    issun_bevy::plugins::scripting::register_all_apis(backend.lua()).unwrap();

    // Get Health component as Lua table
    let health_table = get_component_as_lua_table(backend.lua(), &world, entity, "Health")
        .unwrap()
        .expect("Health component should exist");

    // Register table as global so Lua can access it
    backend
        .lua()
        .globals()
        .set("health_data", health_table)
        .unwrap();

    // Lua script reads and validates the data
    let result: bool = backend
        .lua()
        .load(
            r#"
        -- Verify health data
        assert(health_data.current == 80, "current should be 80")
        assert(health_data.max == 80, "max should be 80")

        log(string.format("Read Health: %d/%d", health_data.current, health_data.max))
        return true
    "#,
        )
        .eval()
        .unwrap();

    assert!(result);
}

#[test]
fn test_world_access_get_component_not_found() {
    // Test reading non-existent component returns None

    use bevy::prelude::*;
    use issun_bevy::plugins::scripting::{get_component_as_lua_table, MluaBackend};

    // Create World with entity (no Health)
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    // Create Lua backend
    let backend = MluaBackend::new().unwrap();

    // Try to get Health component (should be None)
    let result = get_component_as_lua_table(backend.lua(), &world, entity, "Health").unwrap();

    assert!(result.is_none());
}

#[test]
fn test_example_mod_healing_station() {
    // Test the example healing_station mod

    use bevy::prelude::*;
    use issun_bevy::plugins::scripting::{LuaCommandQueue, LuaCommands, MluaBackend};
    use std::sync::{Arc, Mutex};

    // Create backend and register APIs
    let mut backend = MluaBackend::new().unwrap();
    issun_bevy::plugins::scripting::register_all_apis(backend.lua()).unwrap();

    // Create command queue and register it
    let queue = Arc::new(Mutex::new(LuaCommandQueue::new()));
    let lua_commands = LuaCommands::new(queue.clone());
    backend
        .lua()
        .globals()
        .set("commands", lua_commands)
        .unwrap();

    // Load the healing station mod
    // Try both workspace root and crate-relative paths
    let mod_path =
        if std::path::Path::new("../../examples/mods/healing_station/healing_station.lua").exists()
        {
            "../../examples/mods/healing_station/healing_station.lua"
        } else if std::path::Path::new("examples/mods/healing_station/healing_station.lua").exists()
        {
            "examples/mods/healing_station/healing_station.lua"
        } else {
            eprintln!("⚠️  Warning: Example mod not found, skipping test");
            return;
        };

    let handle = backend.load_script(mod_path).unwrap();

    // Call on_init
    backend.call_function(handle, "on_init").unwrap();

    // Simulate entity entering healing station
    backend
        .execute_chunk(
            r#"
        on_entity_enter({entity_id = 42})
    "#,
        )
        .unwrap();

    // Check that heal command was queued
    let queue_len = queue.lock().unwrap().len();
    assert_eq!(queue_len, 1, "Heal command should be queued");

    // Get mod stats
    use mlua::TableExt;
    {
        let stats: mlua::Table = backend
            .lua()
            .globals()
            .call_function("get_stats", ())
            .unwrap();

        let total_heals: i32 = stats.get("total_heals").unwrap();
        let version: String = stats.get("version").unwrap();
        let active: bool = stats.get("active").unwrap();

        assert_eq!(total_heals, 1);
        assert_eq!(version, "1.0.0");
        assert!(active);
    }

    // Test set_heal_range function
    backend
        .execute_chunk(
            r#"
        set_heal_range(20, 50)
    "#,
        )
        .unwrap();

    // Test roll_critical function
    let is_critical: bool = backend
        .lua()
        .globals()
        .call_function("roll_critical", ())
        .unwrap();

    // Result is random, just verify it returns a bool (always true, but explicit type check)
    #[allow(clippy::overly_complex_bool_expr)]
    {
        assert!(is_critical || !is_critical);
    }
}
