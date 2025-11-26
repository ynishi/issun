//! Lua Entity UserData
//!
//! Provides Lua bindings for Bevy Entity with lifetime safety.

use bevy::prelude::*;
use mlua::{prelude::*, UserData};

/// Lua wrapper for Bevy Entity with World access
///
/// This is passed to Lua scripts to represent an entity.
/// All operations check entity lifetime safety via SafeEntityRef.
#[derive(Clone)]
pub struct LuaEntity {
    entity: Entity,
    // TODO: Add World access (via Arc<RwLock<World>> or similar)
}

impl LuaEntity {
    /// Create a new LuaEntity
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }

    /// Get the underlying Entity
    pub fn entity(&self) -> Entity {
        self.entity
    }
}

impl UserData for LuaEntity {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        // entity:has_component(type_name) -> bool
        methods.add_method("has_component", |_lua, this, type_name: String| {
            // TODO: Implement with World access and SafeEntityRef
            tracing::warn!(
                "has_component('{}') called on entity {:?} - not yet implemented",
                type_name,
                this.entity
            );
            Ok(false)
        });

        // entity:get_component(type_name) -> table | error
        methods.add_method(
            "get_component",
            |_lua, this, type_name: String| -> LuaResult<LuaTable> {
                // TODO: Implement with Reflection
                tracing::warn!(
                    "get_component('{}') called on entity {:?} - not yet implemented",
                    type_name,
                    this.entity
                );
                Err(LuaError::RuntimeError(
                    "get_component not yet implemented".to_string(),
                ))
            },
        );

        // entity:set_component(type_name, data) -> () | error
        methods.add_method(
            "set_component",
            |_lua, this, (type_name, _data): (String, LuaTable)| -> LuaResult<()> {
                // TODO: Implement with Reflection
                tracing::warn!(
                    "set_component('{}') called on entity {:?} - not yet implemented",
                    type_name,
                    this.entity
                );
                Err(LuaError::RuntimeError(
                    "set_component not yet implemented".to_string(),
                ))
            },
        );

        // entity:despawn() -> () | error
        methods.add_method("despawn", |_lua, this, ()| -> LuaResult<()> {
            // TODO: Implement with Commands
            tracing::warn!(
                "despawn() called on entity {:?} - not yet implemented",
                this.entity
            );
            Err(LuaError::RuntimeError(
                "despawn not yet implemented".to_string(),
            ))
        });

        // entity:id() -> number (for debugging)
        methods.add_method("id", |_lua, this, ()| {
            Ok(this.entity.to_bits())
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_entity_creation() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let lua_entity = LuaEntity::new(entity);
        assert_eq!(lua_entity.entity(), entity);
    }

    #[test]
    fn test_lua_entity_methods_available() {
        let lua = Lua::new();
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let lua_entity = LuaEntity::new(entity);

        // Test that methods are available
        lua.globals().set("entity", lua_entity).unwrap();

        // Test id() method
        let entity_bits: u64 = lua.load(r#"return entity:id()"#).eval().unwrap();
        assert_eq!(entity_bits, entity.to_bits());

        // Test has_component() stub (returns false for now)
        let has_component: bool = lua
            .load(r#"return entity:has_component("Health")"#)
            .eval()
            .unwrap();
        assert_eq!(has_component, false);

        // Test get_component() stub (returns error)
        let result = lua
            .load(r#"return entity:get_component("Health")"#)
            .eval::<LuaValue>();
        assert!(result.is_err());

        // Test set_component() stub (returns error)
        let result = lua
            .load(r#"return entity:set_component("Health", {current = 100})"#)
            .eval::<LuaValue>();
        assert!(result.is_err());

        // Test despawn() stub (returns error)
        let result = lua.load(r#"return entity:despawn()"#).eval::<LuaValue>();
        assert!(result.is_err());
    }
}
