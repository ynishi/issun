//! World Access Helpers for Lua Scripts
//!
//! Provides safe read-only access to World state from Lua scripts.
//! Currently proof-of-concept with hardcoded Health component.

use bevy::prelude::*;
use mlua::{Lua, Result as LuaResult, Table};

use crate::plugins::combat::components::Health;

/// Helper to create Lua table from Health component
///
/// Proof-of-concept: Converts Health component to Lua table.
/// TODO: Generalize using Reflection and TypeRegistry.
pub fn health_to_lua_table<'lua>(lua: &'lua Lua, health: &Health) -> LuaResult<Table<'lua>> {
    let table = lua.create_table()?;
    table.set("current", health.current)?;
    table.set("max", health.max)?;
    Ok(table)
}

/// Helper to get component data from World as Lua table
///
/// Proof-of-concept: Only supports Health component.
/// TODO: Generalize using TypeRegistry.
pub fn get_component_as_lua_table<'lua>(
    lua: &'lua Lua,
    world: &World,
    entity: Entity,
    type_name: &str,
) -> LuaResult<Option<Table<'lua>>> {
    match type_name {
        "Health" => {
            if let Some(health) = world.get::<Health>(entity) {
                Ok(Some(health_to_lua_table(lua, health)?))
            } else {
                Ok(None)
            }
        }
        _ => {
            // Unknown component type
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_to_lua_table() {
        let lua = Lua::new();
        let health = Health::new(100);

        let table = health_to_lua_table(&lua, &health).unwrap();

        let current: i32 = table.get("current").unwrap();
        let max: i32 = table.get("max").unwrap();

        assert_eq!(current, 100);
        assert_eq!(max, 100);
    }

    #[test]
    fn test_get_component_health() {
        let lua = Lua::new();
        let mut world = World::new();

        // Spawn entity with Health
        let entity = world.spawn(Health::new(75)).id();

        // Get component as Lua table
        let table = get_component_as_lua_table(&lua, &world, entity, "Health")
            .unwrap()
            .unwrap();

        let current: i32 = table.get("current").unwrap();
        let max: i32 = table.get("max").unwrap();

        assert_eq!(current, 75);
        assert_eq!(max, 75);
    }

    #[test]
    fn test_get_component_not_found() {
        let lua = Lua::new();
        let mut world = World::new();

        // Spawn entity without Health
        let entity = world.spawn_empty().id();

        // Get component should return None
        let result = get_component_as_lua_table(&lua, &world, entity, "Health").unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_get_component_unknown_type() {
        let lua = Lua::new();
        let mut world = World::new();

        let entity = world.spawn_empty().id();

        // Unknown component type should return None
        let result = get_component_as_lua_table(&lua, &world, entity, "UnknownComponent").unwrap();

        assert!(result.is_none());
    }
}
