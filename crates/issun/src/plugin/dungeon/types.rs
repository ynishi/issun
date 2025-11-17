//! Dungeon plugin types

use serde::{Deserialize, Serialize};

/// Dungeon structure configuration
///
/// This resource defines the overall dungeon structure.
/// Register it during game initialization.
#[derive(crate::Resource, Clone, Debug, Serialize, Deserialize)]
pub struct DungeonConfig {
    pub total_floors: u32,
    pub rooms_per_floor: u32,
    pub connection_pattern: ConnectionPattern,
}

impl Default for DungeonConfig {
    fn default() -> Self {
        Self {
            total_floors: 5,
            rooms_per_floor: 3,
            connection_pattern: ConnectionPattern::Linear,
        }
    }
}

/// Room connection pattern
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConnectionPattern {
    /// Linear progression: Room1 → Room2 → Room3 → ...
    Linear,
    /// Branching progression with choices
    Branching,
    /// Free-form graph exploration
    Graph,
}

/// Current dungeon state (stored in `ResourceContext`)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DungeonState {
    pub current_floor: u32,
    pub current_room: u32,
    pub visited_rooms: Vec<RoomId>,
    pub unlocked_connections: Vec<Connection>,
}

impl Default for DungeonState {
    fn default() -> Self {
        Self {
            current_floor: 1,
            current_room: 1,
            visited_rooms: vec![RoomId { floor: 1, room: 1 }],
            unlocked_connections: vec![],
        }
    }
}

/// Room identifier
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId {
    pub floor: u32,
    pub room: u32,
}

impl RoomId {
    pub fn new(floor: u32, room: u32) -> Self {
        Self { floor, room }
    }
}

/// Connection between two rooms
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    pub from: RoomId,
    pub to: RoomId,
}

impl Connection {
    pub fn new(from: RoomId, to: RoomId) -> Self {
        Self { from, to }
    }
}
