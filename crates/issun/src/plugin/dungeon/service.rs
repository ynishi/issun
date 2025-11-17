//! Dungeon navigation service (pure logic)

use super::types::{ConnectionPattern, DungeonConfig, DungeonState, RoomId};

/// Dungeon navigation service
///
/// Provides pure functions for dungeon navigation logic.
/// No state management - only calculations.
#[derive(crate::Service, Debug, Clone)]
#[service(name = "dungeon_service")]
pub struct DungeonService;

impl DungeonService {
    pub fn new() -> Self {
        Self
    }

    /// Get available rooms from current position
    pub fn available_rooms(&self, config: &DungeonConfig, state: &DungeonState) -> Vec<u32> {
        match config.connection_pattern {
            ConnectionPattern::Linear => {
                // Next room only
                if state.current_room < config.rooms_per_floor {
                    vec![state.current_room + 1]
                } else {
                    vec![]
                }
            }
            ConnectionPattern::Branching => {
                // For now, simple branching: current + 1 and current + 2
                let mut rooms = Vec::new();
                if state.current_room + 1 <= config.rooms_per_floor {
                    rooms.push(state.current_room + 1);
                }
                if state.current_room + 2 <= config.rooms_per_floor {
                    rooms.push(state.current_room + 2);
                }
                rooms
            }
            ConnectionPattern::Graph => {
                // Use unlocked connections
                state
                    .unlocked_connections
                    .iter()
                    .filter(|conn| {
                        conn.from.floor == state.current_floor
                            && conn.from.room == state.current_room
                    })
                    .map(|conn| conn.to.room)
                    .collect()
            }
        }
    }

    /// Check if can advance to next floor
    pub fn can_advance_floor(&self, config: &DungeonConfig, state: &DungeonState) -> bool {
        // Boss room cleared (last room on floor)
        state.current_room >= config.rooms_per_floor && state.current_floor < config.total_floors
    }

    /// Check if dungeon is completed
    pub fn is_completed(&self, config: &DungeonConfig, state: &DungeonState) -> bool {
        state.current_floor >= config.total_floors && state.current_room >= config.rooms_per_floor
    }

    /// Check if a room has been visited
    pub fn is_room_visited(&self, state: &DungeonState, room_id: &RoomId) -> bool {
        state.visited_rooms.contains(room_id)
    }

    /// Get progress percentage
    pub fn progress_percentage(&self, config: &DungeonConfig, state: &DungeonState) -> f32 {
        let total_rooms = config.total_floors * config.rooms_per_floor;
        let completed_rooms =
            (state.current_floor - 1) * config.rooms_per_floor + state.current_room;
        (completed_rooms as f32 / total_rooms as f32) * 100.0
    }
}

impl Default for DungeonService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_available_rooms() {
        let service = DungeonService::new();
        let config = DungeonConfig {
            total_floors: 3,
            rooms_per_floor: 3,
            connection_pattern: ConnectionPattern::Linear,
        };
        let state = DungeonState {
            current_floor: 1,
            current_room: 1,
            ..Default::default()
        };

        let available = service.available_rooms(&config, &state);
        assert_eq!(available, vec![2]);
    }

    #[test]
    fn test_can_advance_floor() {
        let service = DungeonService::new();
        let config = DungeonConfig::default();
        let state = DungeonState {
            current_floor: 1,
            current_room: 3,
            ..Default::default()
        };

        assert!(service.can_advance_floor(&config, &state));
    }

    #[test]
    fn test_progress_percentage() {
        let service = DungeonService::new();
        let config = DungeonConfig {
            total_floors: 5,
            rooms_per_floor: 3,
            connection_pattern: ConnectionPattern::Linear,
        };
        let state = DungeonState {
            current_floor: 3,
            current_room: 2,
            ..Default::default()
        };

        let progress = service.progress_percentage(&config, &state);
        // (2 * 3 + 2) / 15 * 100 = 8 / 15 * 100 = 53.33...
        assert!((progress - 53.33).abs() < 0.1);
    }
}
