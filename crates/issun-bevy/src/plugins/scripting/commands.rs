//! Lua Commands Queue
//!
//! Provides a command queue for deferred ECS operations from Lua scripts.
//! Commands are queued and executed at the end of the frame by Bevy systems.

use bevy::prelude::*;
use mlua::{prelude::*, UserData};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Queued command from Lua script
#[derive(Debug, Clone)]
pub enum LuaCommand {
    /// Spawn entity from scene path
    SpawnEntity {
        scene_path: String,
    },
    /// Insert component on entity
    InsertComponent {
        entity: Entity,
        type_name: String,
        data: LuaValue,
    },
    /// Remove component from entity
    RemoveComponent {
        entity: Entity,
        type_name: String,
    },
    /// Despawn entity
    DespawnEntity {
        entity: Entity,
    },
}

/// Lua value that can be cloned (simplified for now)
#[derive(Debug, Clone)]
pub enum LuaValue {
    Nil,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(String),
    // Table serialization deferred to Phase 4
}

impl From<mlua::Value<'_>> for LuaValue {
    fn from(value: mlua::Value) -> Self {
        match value {
            mlua::Value::Nil => LuaValue::Nil,
            mlua::Value::Boolean(b) => LuaValue::Boolean(b),
            mlua::Value::Integer(i) => LuaValue::Integer(i),
            mlua::Value::Number(n) => LuaValue::Number(n),
            mlua::Value::String(s) => LuaValue::String(s.to_str().unwrap_or("").to_string()),
            _ => LuaValue::Nil, // Tables, functions, etc. not yet supported
        }
    }
}

/// Command queue resource
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct LuaCommandQueue {
    #[reflect(ignore)]
    queue: VecDeque<LuaCommand>,
}

impl LuaCommandQueue {
    /// Create a new command queue
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// Add a command to the queue
    pub fn push(&mut self, command: LuaCommand) {
        self.queue.push_back(command);
    }

    /// Get all commands and clear the queue
    pub fn drain(&mut self) -> Vec<LuaCommand> {
        self.queue.drain(..).collect()
    }

    /// Get the number of queued commands
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

/// Lua wrapper for command queue
///
/// Provides `commands` object in Lua scripts for queuing deferred operations.
#[derive(Clone)]
pub struct LuaCommands {
    queue: Arc<Mutex<LuaCommandQueue>>,
}

impl LuaCommands {
    /// Create a new LuaCommands with shared queue
    pub fn new(queue: Arc<Mutex<LuaCommandQueue>>) -> Self {
        Self { queue }
    }

    /// Queue a command (internal use)
    fn push_command(&self, command: LuaCommand) -> LuaResult<()> {
        self.queue
            .lock()
            .map_err(|e| LuaError::RuntimeError(format!("Failed to lock command queue: {}", e)))?
            .push(command);
        Ok(())
    }
}

impl UserData for LuaCommands {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        // commands:spawn_entity(scene_path) -> ()
        methods.add_method("spawn_entity", |_lua, this, scene_path: String| {
            this.push_command(LuaCommand::SpawnEntity { scene_path })
        });

        // commands:despawn_entity(entity_id) -> ()
        methods.add_method("despawn_entity", |_lua, this, entity_bits: u64| {
            // SAFETY: Entity::from_bits() is safe here because:
            // - This is just queuing an entity ID, not accessing the entity
            // - Actual entity access with safety checks happens during command execution
            // - Command execution system will use SafeEntityRef for validation
            this.push_command(LuaCommand::DespawnEntity {
                entity: Entity::from_bits(entity_bits),
            })
        });

        // commands:insert_component(entity_id, type_name, data) -> ()
        methods.add_method(
            "insert_component",
            |_lua, this, (entity_bits, type_name, data): (u64, String, mlua::Value)| {
                // SAFETY: Entity::from_bits() is safe here (see despawn_entity comment)
                this.push_command(LuaCommand::InsertComponent {
                    entity: Entity::from_bits(entity_bits),
                    type_name,
                    data: data.into(),
                })
            },
        );

        // commands:remove_component(entity_id, type_name) -> ()
        methods.add_method(
            "remove_component",
            |_lua, this, (entity_bits, type_name): (u64, String)| {
                // SAFETY: Entity::from_bits() is safe here (see despawn_entity comment)
                this.push_command(LuaCommand::RemoveComponent {
                    entity: Entity::from_bits(entity_bits),
                    type_name,
                })
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_queue_creation() {
        let queue = LuaCommandQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_command_queue_push_drain() {
        let mut queue = LuaCommandQueue::new();

        // Push some commands
        queue.push(LuaCommand::SpawnEntity {
            scene_path: "test.ron".to_string(),
        });
        queue.push(LuaCommand::DespawnEntity {
            entity: Entity::from_bits(42),
        });

        assert_eq!(queue.len(), 2);

        // Drain commands
        let commands = queue.drain();
        assert_eq!(commands.len(), 2);
        assert!(queue.is_empty());

        // Verify command types
        match &commands[0] {
            LuaCommand::SpawnEntity { scene_path } => {
                assert_eq!(scene_path, "test.ron");
            }
            _ => panic!("Expected SpawnEntity"),
        }

        match &commands[1] {
            LuaCommand::DespawnEntity { entity } => {
                assert_eq!(entity.to_bits(), 42);
            }
            _ => panic!("Expected DespawnEntity"),
        }
    }

    #[test]
    fn test_lua_value_conversion() {
        use mlua::Lua;

        let lua = Lua::new();

        // Test nil
        let nil_value: mlua::Value = lua.load("return nil").eval().unwrap();
        let lua_value: LuaValue = nil_value.into();
        assert!(matches!(lua_value, LuaValue::Nil));

        // Test boolean
        let bool_value: mlua::Value = lua.load("return true").eval().unwrap();
        let lua_value: LuaValue = bool_value.into();
        assert!(matches!(lua_value, LuaValue::Boolean(true)));

        // Test integer
        let int_value: mlua::Value = lua.load("return 42").eval().unwrap();
        let lua_value: LuaValue = int_value.into();
        assert!(matches!(lua_value, LuaValue::Integer(42)));

        // Test number
        let num_value: mlua::Value = lua.load("return 3.14").eval().unwrap();
        let lua_value: LuaValue = num_value.into();
        if let LuaValue::Number(n) = lua_value {
            assert!((n - 3.14).abs() < 0.001);
        } else {
            panic!("Expected Number");
        }

        // Test string
        let str_value: mlua::Value = lua.load(r#"return "hello""#).eval().unwrap();
        let lua_value: LuaValue = str_value.into();
        assert!(matches!(lua_value, LuaValue::String(s) if s == "hello"));
    }

    #[test]
    fn test_lua_commands_userdata() {
        use mlua::Lua;

        let lua = Lua::new();

        // Create command queue and LuaCommands
        let queue = Arc::new(Mutex::new(LuaCommandQueue::new()));
        let lua_commands = LuaCommands::new(queue.clone());

        // Set as global
        lua.globals().set("commands", lua_commands).unwrap();

        // Test spawn_entity command
        lua.load(r#"commands:spawn_entity("test.ron")"#)
            .exec()
            .unwrap();

        // Test despawn_entity command
        lua.load(r#"commands:despawn_entity(42)"#).exec().unwrap();

        // Test insert_component command
        lua.load(r#"commands:insert_component(42, "Health", 100)"#)
            .exec()
            .unwrap();

        // Test remove_component command
        lua.load(r#"commands:remove_component(42, "Defense")"#)
            .exec()
            .unwrap();

        // Verify commands were queued
        let commands = queue.lock().unwrap().drain();
        assert_eq!(commands.len(), 4);

        // Check command types
        match &commands[0] {
            LuaCommand::SpawnEntity { scene_path } => {
                assert_eq!(scene_path, "test.ron");
            }
            _ => panic!("Expected SpawnEntity"),
        }

        match &commands[1] {
            LuaCommand::DespawnEntity { entity } => {
                assert_eq!(entity.to_bits(), 42);
            }
            _ => panic!("Expected DespawnEntity"),
        }

        match &commands[2] {
            LuaCommand::InsertComponent {
                entity,
                type_name,
                data,
            } => {
                assert_eq!(entity.to_bits(), 42);
                assert_eq!(type_name, "Health");
                assert!(matches!(data, LuaValue::Integer(100)));
            }
            _ => panic!("Expected InsertComponent"),
        }

        match &commands[3] {
            LuaCommand::RemoveComponent { entity, type_name } => {
                assert_eq!(entity.to_bits(), 42);
                assert_eq!(type_name, "Defense");
            }
            _ => panic!("Expected RemoveComponent"),
        }
    }
}
