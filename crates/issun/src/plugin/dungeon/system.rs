//! Dungeon system implementation

use crate::context::{ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

use super::events::*;
use super::hook::DungeonHook;
use super::types::DungeonState;

/// System that processes dungeon events with hooks
///
/// This system:
/// 1. Processes room move requests
/// 2. Processes floor advance requests
/// 3. Processes connection unlock requests
/// 4. Calls hooks for custom behavior
/// 5. Publishes state change events for network replication
///
/// # Feedback Loop
///
/// ```text
/// Command Event → Validation (Hook) → State Update → Hook Call → State Event
/// ```
pub struct DungeonSystem {
    hook: Arc<dyn DungeonHook>,
}

impl DungeonSystem {
    /// Create a new DungeonSystem with a custom hook
    pub fn new(hook: Arc<dyn DungeonHook>) -> Self {
        Self { hook }
    }

    /// Process all dungeon events
    pub async fn process_events(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        self.process_room_move_requests(resources).await;
        self.process_floor_advance_requests(resources).await;
        self.process_connection_unlock_requests(resources).await;
    }

    /// Process room move requests
    async fn process_room_move_requests(&mut self, resources: &mut ResourceContext) {
        // Collect room move requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<RoomMoveRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Get current room for validation
            let current_room = {
                if let Some(state) = resources.get::<DungeonState>().await {
                    super::types::RoomId::new(state.current_floor, state.current_room)
                } else {
                    continue;
                }
            };

            // Validate via hook
            {
                let resources_ref = resources as &ResourceContext;
                if let Err(_) = self
                    .hook
                    .validate_room_move(&current_room, &request.target_room, resources_ref)
                    .await
                {
                    continue;
                }
            }

            // Check if first visit
            let is_first_visit = {
                if let Some(state) = resources.get::<DungeonState>().await {
                    !state.visited_rooms.contains(&request.target_room)
                } else {
                    false
                }
            };

            // Move to room (update state)
            {
                if let Some(mut state) = resources.get_mut::<DungeonState>().await {
                    state.current_floor = request.target_room.floor;
                    state.current_room = request.target_room.room;

                    if is_first_visit {
                        state.visited_rooms.push(request.target_room.clone());
                    }
                } else {
                    continue;
                }
            }

            // Call hook
            self.hook
                .on_room_entered(&request.target_room, is_first_visit, resources)
                .await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(RoomEnteredEvent {
                    room_id: request.target_room.clone(),
                    is_first_visit,
                });
            }
        }
    }

    /// Process floor advance requests
    async fn process_floor_advance_requests(&mut self, resources: &mut ResourceContext) {
        // Collect floor advance requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<FloorAdvanceRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for _request in requests {
            // Advance floor (update state)
            let new_floor = {
                if let Some(mut state) = resources.get_mut::<DungeonState>().await {
                    state.current_floor += 1;
                    state.current_room = 1;

                    let room_id = super::types::RoomId::new(state.current_floor, 1);
                    if !state.visited_rooms.contains(&room_id) {
                        state.visited_rooms.push(room_id);
                    }

                    state.current_floor
                } else {
                    continue;
                }
            };

            // Call hook
            self.hook.on_floor_advanced(new_floor, resources).await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(FloorAdvancedEvent { new_floor });
            }
        }
    }

    /// Process connection unlock requests
    async fn process_connection_unlock_requests(&mut self, resources: &mut ResourceContext) {
        // Collect connection unlock requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ConnectionUnlockRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Check if already unlocked
            let already_unlocked = {
                if let Some(state) = resources.get::<DungeonState>().await {
                    state.unlocked_connections.contains(&request.connection)
                } else {
                    true
                }
            };

            if already_unlocked {
                continue;
            }

            // Unlock connection (update state)
            {
                if let Some(mut state) = resources.get_mut::<DungeonState>().await {
                    state.unlocked_connections.push(request.connection.clone());
                } else {
                    continue;
                }
            }

            // Call hook
            self.hook
                .on_connection_unlocked(&request.connection, resources)
                .await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(ConnectionUnlockedEvent {
                    connection: request.connection.clone(),
                });
            }
        }
    }
}

#[async_trait]
impl System for DungeonSystem {
    fn name(&self) -> &'static str {
        "dungeon_system"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
