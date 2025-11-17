//! Dungeon progression system (orchestration)

use super::service::DungeonService;
use super::types::{Connection, DungeonState, RoomId};
use crate::context::ResourceContext;

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
    pub async fn advance_room(&self, resources: &mut ResourceContext, room: u32) {
        let mut state = resources
            .get_mut::<DungeonState>()
            .await
            .expect("DungeonState not registered in ResourceContext");
        let room_id = RoomId::new(state.current_floor, room);

        if !state.visited_rooms.contains(&room_id) {
            state.visited_rooms.push(room_id.clone());
        }

        state.current_room = room;
    }

    /// Advance to next floor
    pub async fn advance_floor(&self, resources: &mut ResourceContext) {
        let mut state = resources
            .get_mut::<DungeonState>()
            .await
            .expect("DungeonState not registered in ResourceContext");
        state.current_floor += 1;
        state.current_room = 1;

        let room_id = RoomId::new(state.current_floor, 1);
        if !state.visited_rooms.contains(&room_id) {
            state.visited_rooms.push(room_id);
        }
    }

    /// Unlock a connection between rooms
    pub async fn unlock_connection(
        &self,
        resources: &mut ResourceContext,
        from: RoomId,
        to: RoomId,
    ) {
        let mut state = resources
            .get_mut::<DungeonState>()
            .await
            .expect("DungeonState not registered in ResourceContext");
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
    use crate::context::ResourceContext;

    fn context_with_state() -> ResourceContext {
        let mut resources = ResourceContext::new();
        resources.insert(DungeonState::default());
        resources
    }

    #[tokio::test]
    async fn test_advance_room() {
        let system = DungeonSystem::new();
        let mut resources = context_with_state();

        system.advance_room(&mut resources, 2).await;
        let state = resources.get::<DungeonState>().await.unwrap();
        assert_eq!(state.current_room, 2);
        assert_eq!(state.visited_rooms.len(), 2); // Initial room + new room
    }

    #[tokio::test]
    async fn test_advance_floor() {
        let system = DungeonSystem::new();
        let mut resources = context_with_state();

        system.advance_floor(&mut resources).await;
        let state = resources.get::<DungeonState>().await.unwrap();
        assert_eq!(state.current_floor, 2);
        assert_eq!(state.current_room, 1);
    }

    #[tokio::test]
    async fn test_unlock_connection() {
        let system = DungeonSystem::new();
        let mut resources = context_with_state();

        let from = RoomId::new(1, 1);
        let to = RoomId::new(1, 3);

        system
            .unlock_connection(&mut resources, from.clone(), to.clone())
            .await;

        let state = resources.get::<DungeonState>().await.unwrap();
        assert_eq!(state.unlocked_connections.len(), 1);
        assert_eq!(state.unlocked_connections[0].from, from);
        assert_eq!(state.unlocked_connections[0].to, to);
    }
}
