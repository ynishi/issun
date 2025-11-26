//! Scripting Plugin
//!
//! Provides Lua/Rhai scripting support for entities.

use bevy::prelude::*;

use crate::IssunSet;

use super::{
    api_bindings,
    backend::ScriptingBackend,
    commands::{LuaCommand, LuaCommandQueue, LuaValue},
    components::LuaScript,
    mlua_backend::MluaBackend,
};
use crate::plugins::combat::components::Health;

/// Plugin for scripting system support
pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        // Create MluaBackend and register APIs
        let backend = MluaBackend::new().expect("Failed to create MluaBackend");

        // Register all Lua APIs
        api_bindings::register_all_apis(backend.lua()).expect("Failed to register Lua APIs");

        app
            // Register types
            .register_type::<LuaScript>()
            .register_type::<LuaCommandQueue>()
            // Add resources
            .insert_non_send_resource(backend)
            .init_resource::<LuaCommandQueue>()
            // Add systems
            .add_systems(
                Update,
                (load_scripts, execute_lua_commands)
                    .chain()
                    .in_set(IssunSet::Logic),
            );
    }
}

/// System to load scripts attached to entities
fn load_scripts(
    mut backend: NonSendMut<MluaBackend>,
    mut query: Query<(Entity, &mut LuaScript), Changed<LuaScript>>,
) {
    for (entity, mut script) in query.iter_mut() {
        if !script.is_loaded() {
            info!("Loading script '{}' for entity {:?}", script.path, entity);

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

/// System to execute queued Lua commands
///
/// Drains the command queue and executes each command with entity safety validation.
/// Uses Query<Entity> for entity existence checks (avoids &World parameter conflict).
fn execute_lua_commands(
    mut commands: Commands,
    mut queue: ResMut<LuaCommandQueue>,
    entities: Query<Entity>,
) {
    let queued_commands = queue.drain();

    if !queued_commands.is_empty() {
        info!("Executing {} Lua command(s)", queued_commands.len());
    }

    for command in queued_commands {
        match command {
            LuaCommand::SpawnEntity { scene_path } => {
                // TODO: Load DynamicScene and spawn
                warn!(
                    "SpawnEntity('{}') not yet implemented - requires DynamicScene loading",
                    scene_path
                );
            }

            LuaCommand::DespawnEntity { entity } => {
                // Check if entity exists using Query
                if entities.get(entity).is_ok() {
                    commands.entity(entity).despawn();
                    info!("Despawned entity {:?}", entity);
                } else {
                    warn!("Cannot despawn entity {:?} - already despawned", entity);
                }
            }

            LuaCommand::InsertComponent {
                entity,
                type_name,
                data,
            } => {
                // Check if entity exists using Query
                if entities.get(entity).is_err() {
                    warn!(
                        "Cannot insert component '{}' on entity {:?} - entity despawned",
                        type_name, entity
                    );
                    continue;
                }

                // Hardcoded component insertion for proof-of-concept
                // TODO: Generalize to use Reflection and TypeRegistry
                match type_name.as_str() {
                    "Health" => {
                        // Parse LuaValue to Health component
                        let health = match &data {
                            LuaValue::Integer(max) => Health::new(*max as i32),
                            _ => {
                                warn!("Invalid data type for Health component: {:?}", data);
                                continue;
                            }
                        };

                        commands.entity(entity).insert(health);
                        info!("Inserted Health component on entity {:?}", entity);
                    }
                    _ => {
                        warn!(
                            "Unknown component type '{}' - only 'Health' is currently supported",
                            type_name
                        );
                    }
                }
            }

            LuaCommand::RemoveComponent { entity, type_name } => {
                // Check if entity exists using Query
                if entities.get(entity).is_err() {
                    warn!(
                        "Cannot remove component '{}' from entity {:?} - entity despawned",
                        type_name, entity
                    );
                    continue;
                }

                // TODO: Reflect-based component removal
                warn!(
                    "RemoveComponent('{}') from entity {:?} not yet implemented - requires Reflection",
                    type_name, entity
                );
            }
        }
    }
}
