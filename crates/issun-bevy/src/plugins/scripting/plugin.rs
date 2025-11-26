//! Scripting Plugin
//!
//! Provides Lua/Rhai scripting support for entities.

use bevy::prelude::*;

use crate::IssunSet;

use super::components::LuaScript;

/// Plugin for scripting system support
pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register types
            .register_type::<LuaScript>()
            // Add systems
            .add_systems(Update, load_scripts.in_set(IssunSet::Logic));
    }
}

/// System to load scripts attached to entities
fn load_scripts(query: Query<(Entity, &LuaScript), Changed<LuaScript>>) {
    for (entity, script) in query.iter() {
        if !script.is_loaded() {
            info!(
                "Loading script '{}' for entity {:?}",
                script.path, entity
            );
            // TODO: Actually load the script using ScriptingBackend
        }
    }
}
