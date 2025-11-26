//! Scripting Plugin
//!
//! Provides Lua/Rhai scripting support for entities.

use bevy::prelude::*;

use crate::IssunSet;

use super::{
    api_bindings,
    backend::ScriptingBackend,
    components::LuaScript,
    mlua_backend::MluaBackend,
};

/// Plugin for scripting system support
pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        // Create MluaBackend and register APIs
        let backend = MluaBackend::new().expect("Failed to create MluaBackend");

        // Register all Lua APIs
        api_bindings::register_all_apis(backend.lua())
            .expect("Failed to register Lua APIs");

        app
            // Register types
            .register_type::<LuaScript>()
            // Add MluaBackend as NonSend resource (Lua is not Send+Sync)
            .insert_non_send_resource(backend)
            // Add systems
            .add_systems(Update, load_scripts.in_set(IssunSet::Logic));
    }
}

/// System to load scripts attached to entities
fn load_scripts(
    mut backend: NonSendMut<MluaBackend>,
    mut query: Query<(Entity, &mut LuaScript), Changed<LuaScript>>,
) {
    for (entity, mut script) in query.iter_mut() {
        if !script.is_loaded() {
            info!(
                "Loading script '{}' for entity {:?}",
                script.path, entity
            );

            // Load the script
            match backend.load_script(&script.path) {
                Ok(handle) => {
                    script.set_loaded(handle);
                    info!("Successfully loaded script '{}'", script.path);
                }
                Err(e) => {
                    error!("Failed to load script '{}': {:?}", script.path, e);
                }
            }
        }
    }
}
