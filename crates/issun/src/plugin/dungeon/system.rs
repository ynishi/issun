//! Dungeon progression system (orchestration)

use super::service::DungeonService;
use super::types::{Connection, DungeonState, RoomId};

/// Dungeon progression system
///
/// Manages dungeon state and orchestrates navigation.
/// This is stateful - keeps track of dungeon progression.
#[derive(crate::System, Debug)]
#[system(name = "dungeon_system")]
pub struct DungeonSystem {
    service: DungeonService,
}

impl DungeonSystem {
    pub fn new() -> Self {
        Self {
            service: DungeonService::new(),
        }
    }

    /// Move to a specific room on current floor
    pub fn advance_room(&self, state: &mut DungeonState, room: u32) {
        let room_id = RoomId::new(state.current_floor, room);

        if !state.visited_rooms.contains(&room_id) {
            state.visited_rooms.push(room_id.clone());
        }

        state.current_room = room;
    }

    /// Advance to next floor
    pub fn advance_floor(&self, state: &mut DungeonState) {
        state.current_floor += 1;
        state.current_room = 1;

        let room_id = RoomId::new(state.current_floor, 1);
        if !state.visited_rooms.contains(&room_id) {
            state.visited_rooms.push(room_id);
        }
    }

    /// Unlock a connection between rooms
    pub fn unlock_connection(&self, state: &mut DungeonState, from: RoomId, to: RoomId) {
        let connection = Connection::new(from, to);
        if !state.unlocked_connections.contains(&connection) {
            state.unlocked_connections.push(connection);
        }
    }

    /// Get the service (for external use)
    pub fn service(&self) -> &DungeonService {
        &self.service
    }
}

impl Default for DungeonSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advance_room() {
        let system = DungeonSystem::new();
        let mut state = DungeonState::default();

        system.advance_room(&mut state, 2);
        assert_eq!(state.current_room, 2);
        assert_eq!(state.visited_rooms.len(), 2); // Initial room + new room
    }

    #[test]
    fn test_advance_floor() {
        let system = DungeonSystem::new();
        let mut state = DungeonState::default();

        system.advance_floor(&mut state);
        assert_eq!(state.current_floor, 2);
        assert_eq!(state.current_room, 1);
    }

    #[test]
    fn test_unlock_connection() {
        let system = DungeonSystem::new();
        let mut state = DungeonState::default();

        let from = RoomId::new(1, 1);
        let to = RoomId::new(1, 3);

        system.unlock_connection(&mut state, from.clone(), to.clone());
        assert_eq!(state.unlocked_connections.len(), 1);
        assert_eq!(state.unlocked_connections[0].from, from);
        assert_eq!(state.unlocked_connections[0].to, to);
    }
}
