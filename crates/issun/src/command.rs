//! Command-based state modification system
//!
//! This module provides a transparent command pattern where users write OOP-style code
//! like `entity.add_hp(-10)`, but internally all state changes are deferred as Commands
//! and processed through a Hook Chain before being applied to the World.
//!
//! # Architecture
//!
//! ```text
//! User Code (OOP)          Engine (Transparent)
//! ─────────────────────────────────────────────
//! entity.add_hp(-10)   →   Command::ModifyHp
//! entity.die()         →   Command::Despawn
//!                      →   Hook Chain
//!                      →   Apply to World
//! ```

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::time::Instant;

// Re-export typetag for plugin use
pub use typetag;

// Re-export from entity.rs and system.rs
pub use crate::entity::{EntityId, InvalidEntityId, SerializableEntityId};
pub use crate::system::SystemId;

/// Command represents all possible state changes in the engine
/// Note: Does not derive Clone because CustomCommand doesn't require it
#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    /// Spawn a new entity
    Spawn {
        // Blueprint data (will be expanded in Phase 2)
        entity_id: Option<SerializableEntityId>, // None = auto-generate
    },

    /// Despawn an entity
    Despawn {
        entity: SerializableEntityId,
    },

    /// Add a component to an entity (generic, type-erased)
    AddComponent {
        entity: SerializableEntityId,
        component_type: ComponentTypeId,
        // Actual data handled by CustomCommand
    },

    /// Remove a component from an entity
    RemoveComponent {
        entity: SerializableEntityId,
        component_type: ComponentTypeId,
    },

    /// Custom command (plugin-defined, extensible)
    /// This is where add_hp, set_position, etc. are implemented
    Custom(Box<dyn CustomCommand>),
}

impl Command {
    /// Get the type of this command
    pub fn command_type(&self) -> CommandType {
        match self {
            Command::Spawn { .. } => CommandType::Spawn,
            Command::Despawn { .. } => CommandType::Despawn,
            Command::AddComponent { .. } => CommandType::AddComponent,
            Command::RemoveComponent { .. } => CommandType::RemoveComponent,
            Command::Custom(cmd) => CommandType::Custom(cmd.command_name()),
        }
    }

    /// Get the target entity (if applicable)
    ///
    /// Note: For Custom commands, if the EntityId cannot be converted to SerializableEntityId,
    /// this will return None (though this should never happen in practice).
    pub fn target_entity(&self) -> Option<SerializableEntityId> {
        match self {
            Command::Despawn { entity } => Some(*entity),
            Command::AddComponent { entity, .. } => Some(*entity),
            Command::RemoveComponent { entity, .. } => Some(*entity),
            Command::Custom(cmd) => cmd.target_entity().map(|e| e.into()),
            _ => None,
        }
    }
}

/// Type of command (for filtering in Hook system)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandType {
    Spawn,
    Despawn,
    AddComponent,
    RemoveComponent,
    Custom(&'static str),
}

/// CustomCommand trait - plugins implement this for domain-specific commands
///
/// # Example: Combat Plugin
///
/// ```ignore
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// pub struct AddHpCommand {
///     pub entity: EntityId,
///     pub delta: i32,
/// }
///
/// #[typetag::serde]
/// impl CustomCommand for AddHpCommand {
///     fn apply(&self, world: &mut hecs::World) -> Result<(), ApplyError> {
///         if let Ok(mut health) = world.get_mut::<Health>(self.entity) {
///             health.current = (health.current + self.delta).clamp(0, health.max);
///             Ok(())
///         } else {
///             Err(ApplyError::ComponentNotFound)
///         }
///     }
///
///     fn command_name(&self) -> &'static str {
///         "AddHp"
///     }
///
///     fn target_entity(&self) -> Option<EntityId> {
///         Some(self.entity)
///     }
/// }
/// ```
#[typetag::serde(tag = "type")]
pub trait CustomCommand: Debug + Send + Sync {
    /// Apply this command to the world
    fn apply(&self, world: &mut hecs::World) -> Result<(), ApplyError>;

    /// Get the command name (for debugging/filtering)
    fn command_name(&self) -> &'static str;

    /// Get the target entity (if applicable)
    fn target_entity(&self) -> Option<EntityId> {
        None
    }
}

/// Errors that can occur when applying commands
#[derive(Debug, Clone, thiserror::Error)]
pub enum ApplyError {
    #[error("Target entity not found")]
    EntityNotFound,

    #[error("Component not found on entity")]
    ComponentNotFound,

    #[error("Precondition failed: {0}")]
    PreconditionFailed(String),

    #[error("Custom error: {0}")]
    Custom(String),
}

/// Component type identifier (stored as string for serialization)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentTypeId(pub String);

impl ComponentTypeId {
    pub fn of<T: 'static>() -> Self {
        Self(std::any::type_name::<T>().to_string())
    }

    pub fn from_str(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

/// Command metadata (source, timing, etc.)
#[derive(Debug, Clone)]
pub struct CommandMeta {
    /// Which system generated this command
    pub source: SystemId,
    /// Frame number when generated
    pub frame: u64,
    /// Timestamp when generated
    pub timestamp: Instant,
}

impl CommandMeta {
    pub fn new(source: SystemId, frame: u64) -> Self {
        Self {
            source,
            frame,
            timestamp: Instant::now(),
        }
    }
}

/// CommandQueue - collects commands during a frame
pub struct CommandQueue {
    pending: Vec<(Command, CommandMeta)>,
    current_frame: u64,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            current_frame: 0,
        }
    }

    /// Push a command with metadata
    pub fn push(&mut self, command: Command, meta: CommandMeta) {
        self.pending.push((command, meta));
    }

    /// Push a command with auto-generated metadata
    pub fn push_with_source(&mut self, command: Command, source: SystemId) {
        let meta = CommandMeta::new(source, self.current_frame);
        self.push(command, meta);
    }

    /// Drain all pending commands
    pub fn drain(&mut self) -> Vec<(Command, CommandMeta)> {
        std::mem::take(&mut self.pending)
    }

    /// Get number of pending commands
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Set current frame number
    pub fn set_frame(&mut self, frame: u64) {
        self.current_frame = frame;
    }

    /// Get current frame number
    pub fn frame(&self) -> u64 {
        self.current_frame
    }
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entity_id() -> SerializableEntityId {
        SerializableEntityId(1)
    }

    #[test]
    fn test_command_queue_basic() {
        let mut queue = CommandQueue::new();
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());

        let cmd = Command::Despawn {
            entity: test_entity_id(),
        };

        queue.push_with_source(cmd, SystemId::of::<()>());
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());

        let commands = queue.drain();
        assert_eq!(commands.len(), 1);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_command_type() {
        let cmd = Command::Despawn {
            entity: test_entity_id(),
        };
        assert_eq!(cmd.command_type(), CommandType::Despawn);
        assert_eq!(cmd.target_entity(), Some(test_entity_id()));
    }

    #[test]
    fn test_command_queue_frame_tracking() {
        let mut queue = CommandQueue::new();
        queue.set_frame(42);
        assert_eq!(queue.frame(), 42);

        let cmd = Command::Despawn {
            entity: test_entity_id(),
        };
        queue.push_with_source(cmd, SystemId::of::<()>());

        let commands = queue.drain();
        assert_eq!(commands[0].1.frame, 42);
    }

    // Note: SerializableEntityId and SystemId tests are in entity.rs and system.rs
}
