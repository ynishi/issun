//! Hook trait for custom dungeon behavior

use crate::context::ResourceContext;
use async_trait::async_trait;

use super::types::{Connection, RoomId};

/// Trait for custom dungeon behavior
///
/// **Hook vs Event**:
/// - **Hook**: Synchronous, direct call, can modify resources, NO network replication
/// - **Event**: Asynchronous, Pub-Sub, network-friendly, for loose coupling
///
/// **Use Hook for**:
/// - Immediate calculations (e.g., spawn enemies based on room)
/// - Direct resource modification (e.g., triggering combat, applying room buffs)
/// - Performance critical paths
/// - Local machine only
///
/// **Use Event for**:
/// - Notifying other systems (e.g., UI updates, achievement tracking)
/// - Network replication (multiplayer)
/// - Audit log / replay
#[async_trait]
pub trait DungeonHook: Send + Sync {
    /// Validate whether a room move is allowed
    ///
    /// Return `Ok(())` to allow, `Err(reason)` to prevent.
    ///
    /// # Arguments
    ///
    /// * `current_room` - Current room ID
    /// * `target_room` - Target room ID
    /// * `resources` - Access to game resources (read-only for validation)
    ///
    /// # Returns
    ///
    /// `Ok(())` if move is allowed, `Err(reason)` if prevented
    ///
    /// # Default
    ///
    /// Always allows movement
    ///
    /// # Example Use Cases
    ///
    /// - Check if connection exists and is unlocked
    /// - Require key items to access certain rooms
    /// - Prevent backtracking in some game modes
    async fn validate_room_move(
        &self,
        _current_room: &RoomId,
        _target_room: &RoomId,
        _resources: &ResourceContext,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Called when player enters a room
    ///
    /// **This is the key feedback loop method.**
    ///
    /// Hook interprets room entry and updates other resources.
    /// For example:
    /// - Roguelike: Spawn enemies, place loot, apply room buffs
    /// - Puzzle game: Initialize puzzle state
    /// - Story game: Trigger cutscenes, dialogue
    ///
    /// # Arguments
    ///
    /// * `room_id` - Room being entered
    /// * `is_first_visit` - Whether this is the first time visiting this room
    /// * `resources` - Access to game resources for modification
    async fn on_room_entered(
        &self,
        _room_id: &RoomId,
        _is_first_visit: bool,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Called when player advances to a new floor
    ///
    /// Use this for:
    /// - Increasing difficulty
    /// - Resetting floor-specific state
    /// - Triggering floor transition events (boss spawns, etc.)
    ///
    /// # Arguments
    ///
    /// * `new_floor` - The floor number being entered
    /// * `resources` - Access to game resources for modification
    async fn on_floor_advanced(&self, _new_floor: u32, _resources: &mut ResourceContext) {
        // Default: do nothing
    }

    /// Called when a connection is unlocked
    ///
    /// Use this for:
    /// - Logging unlock events
    /// - Triggering achievements
    /// - Updating map UI
    ///
    /// # Arguments
    ///
    /// * `connection` - The connection that was unlocked
    /// * `resources` - Access to game resources for modification
    async fn on_connection_unlocked(
        &self,
        _connection: &Connection,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }
}

/// Default hook that does nothing
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultDungeonHook;

#[async_trait]
impl DungeonHook for DefaultDungeonHook {
    // All methods use default implementations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_hook_does_nothing() {
        let hook = DefaultDungeonHook;
        let room1 = RoomId::new(1, 1);
        let room2 = RoomId::new(1, 2);
        let connection = Connection::new(room1.clone(), room2.clone());
        let resources = ResourceContext::new();

        // Should not panic
        let result = hook.validate_room_move(&room1, &room2, &resources).await;
        assert!(result.is_ok());

        let mut resources = ResourceContext::new();
        hook.on_room_entered(&room2, true, &mut resources).await;
        hook.on_floor_advanced(2, &mut resources).await;
        hook.on_connection_unlocked(&connection, &mut resources)
            .await;
    }
}
