//! Room buff system implementation

use crate::context::{ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

use super::events::*;
use super::hook::RoomBuffHook;
use super::types::{ActiveBuff, ActiveBuffs, RoomBuffDatabase};

/// System that processes room buff events with hooks
///
/// This system:
/// 1. Processes buff apply requests
/// 2. Processes buff remove requests
/// 3. Processes buff tick requests (turn advancement)
/// 4. Calls hooks for custom behavior
/// 5. Publishes state change events for network replication
///
/// # Feedback Loop
///
/// ```text
/// Command Event → Database Lookup → State Update → Hook Call → State Event
/// ```
pub struct BuffSystem {
    hook: Arc<dyn RoomBuffHook>,
}

impl BuffSystem {
    /// Create a new BuffSystem with a custom hook
    pub fn new(hook: Arc<dyn RoomBuffHook>) -> Self {
        Self { hook }
    }

    /// Process all buff events
    pub async fn process_events(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        self.process_buff_apply_requests(resources).await;
        self.process_buff_remove_requests(resources).await;
        self.process_buff_tick_requests(resources).await;
    }

    /// Process buff apply requests
    async fn process_buff_apply_requests(&mut self, resources: &mut ResourceContext) {
        // Collect buff apply requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<BuffApplyRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Get buff config from database
            let buff_config = {
                if let Some(database) = resources.get::<RoomBuffDatabase>().await {
                    database.get(&request.buff_id).cloned()
                } else {
                    None
                }
            };

            let buff_config = match buff_config {
                Some(config) => config,
                None => continue, // Buff not found in database
            };

            // Create active buff
            let active_buff = ActiveBuff::new(buff_config);

            // Apply buff (update state)
            {
                if let Some(mut buffs) = resources.get_mut::<ActiveBuffs>().await {
                    buffs.add(active_buff.clone());
                } else {
                    continue;
                }
            }

            // Call hook
            self.hook.on_buff_applied(&active_buff, resources).await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(BuffAppliedEvent {
                    buff_id: request.buff_id.clone(),
                });
            }
        }
    }

    /// Process buff remove requests
    async fn process_buff_remove_requests(&mut self, resources: &mut ResourceContext) {
        // Collect buff remove requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<BuffRemoveRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Find and remove buff
            let removed_buff = {
                if let Some(mut buffs) = resources.get_mut::<ActiveBuffs>().await {
                    let index = buffs
                        .buffs
                        .iter()
                        .position(|b| b.config.id == request.buff_id);

                    match index {
                        Some(idx) => Some(buffs.buffs.remove(idx)),
                        None => None,
                    }
                } else {
                    None
                }
            };

            if let Some(buff) = removed_buff {
                // Call hook
                self.hook.on_buff_removed(&buff, resources).await;

                // Publish event
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(BuffRemovedEvent {
                        buff_id: request.buff_id.clone(),
                    });
                }
            }
        }
    }

    /// Process buff tick requests
    async fn process_buff_tick_requests(&mut self, resources: &mut ResourceContext) {
        // Collect buff tick requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<BuffTickRequested>();
                reader.iter().count() // Just count, we don't need the actual events
            } else {
                0
            }
        };

        if requests == 0 {
            return;
        }

        // Get current buffs for ticking
        let current_buffs = {
            if let Some(buffs) = resources.get::<ActiveBuffs>().await {
                buffs.buffs.clone()
            } else {
                Vec::new()
            }
        };

        // Call hook for each buff
        for buff in &current_buffs {
            self.hook.on_buff_tick(buff, resources).await;
        }

        // Tick and remove expired buffs
        let mut expired_buffs = Vec::new();

        {
            if let Some(mut buffs) = resources.get_mut::<ActiveBuffs>().await {
                for buff in &mut buffs.buffs {
                    buff.tick();
                    if buff.is_expired() {
                        expired_buffs.push(buff.clone());
                    }
                }

                buffs.buffs.retain(|buff| !buff.is_expired());
            }
        }

        // Call hook and publish events for expired buffs
        for buff in expired_buffs {
            self.hook.on_buff_expired(&buff, resources).await;

            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(BuffExpiredEvent {
                    buff_id: buff.config.id.clone(),
                });
            }
        }
    }
}

impl Default for BuffSystem {
    fn default() -> Self {
        Self::new(Arc::new(super::hook::DefaultRoomBuffHook))
    }
}

#[async_trait]
impl System for BuffSystem {
    fn name(&self) -> &'static str {
        "room_buff_system"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
