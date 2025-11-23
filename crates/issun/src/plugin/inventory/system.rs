//! Inventory system implementation

use crate::context::{ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

use super::events::*;
use super::hook::InventoryHook;
use super::state::InventoryState;

/// System that processes inventory events with hooks
///
/// This system:
/// 1. Processes item add requests
/// 2. Processes item remove requests
/// 3. Processes item use requests
/// 4. Processes item transfer requests
/// 5. Calls hooks for custom behavior
/// 6. Publishes state change events for network replication
///
/// # Feedback Loop
///
/// ```text
/// Command Event → Validation (Hook) → State Update → Hook Call → State Event
/// ```
#[derive(Clone)]
pub struct InventorySystem {
    hook: Arc<dyn InventoryHook>,
}

impl InventorySystem {
    /// Create a new InventorySystem with a custom hook
    pub fn new(hook: Arc<dyn InventoryHook>) -> Self {
        Self { hook }
    }

    /// Process all inventory events
    pub async fn process_events(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        self.process_add_requests(resources).await;
        self.process_remove_requests(resources).await;
        self.process_use_requests(resources).await;
        self.process_transfer_requests(resources).await;
    }

    /// Process item add requests
    async fn process_add_requests(&mut self, resources: &mut ResourceContext) {
        // Collect add requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ItemAddRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Validate via hook
            {
                let resources_ref = resources as &ResourceContext;
                if self
                    .hook
                    .validate_add_item(
                        &request.entity_id,
                        &request.item_id,
                        request.quantity,
                        resources_ref,
                    )
                    .await
                    .is_err()
                {
                    continue;
                }
            }

            // Add item (update state)
            {
                if let Some(mut state) = resources.get_mut::<InventoryState>().await {
                    if state
                        .add_item(&request.entity_id, &request.item_id, request.quantity)
                        .is_err()
                    {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Call hook
            self.hook
                .on_item_added(
                    &request.entity_id,
                    &request.item_id,
                    request.quantity,
                    resources,
                )
                .await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(ItemAddedEvent {
                    entity_id: request.entity_id.clone(),
                    item_id: request.item_id.clone(),
                    quantity: request.quantity,
                });
            }
        }
    }

    /// Process item remove requests
    async fn process_remove_requests(&mut self, resources: &mut ResourceContext) {
        // Collect remove requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ItemRemoveRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Remove item (update state)
            {
                if let Some(mut state) = resources.get_mut::<InventoryState>().await {
                    if state
                        .remove_item(&request.entity_id, &request.item_id, request.quantity)
                        .is_err()
                    {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Call hook
            self.hook
                .on_item_removed(
                    &request.entity_id,
                    &request.item_id,
                    request.quantity,
                    resources,
                )
                .await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(ItemRemovedEvent {
                    entity_id: request.entity_id.clone(),
                    item_id: request.item_id.clone(),
                    quantity: request.quantity,
                });
            }
        }
    }

    /// Process item use requests
    async fn process_use_requests(&mut self, resources: &mut ResourceContext) {
        // Collect use requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ItemUseRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Verify item exists
            let has_item = {
                if let Some(state) = resources.get::<InventoryState>().await {
                    state.has_item(&request.entity_id, &request.item_id, 1)
                } else {
                    false
                }
            };

            if !has_item {
                continue;
            }

            // Call hook (item effect)
            if self
                .hook
                .on_item_used(&request.entity_id, &request.item_id, resources)
                .await
                .is_err()
            {
                continue;
            }

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(ItemUsedEvent {
                    entity_id: request.entity_id.clone(),
                    item_id: request.item_id.clone(),
                });
            }
        }
    }

    /// Process item transfer requests
    async fn process_transfer_requests(&mut self, resources: &mut ResourceContext) {
        // Collect transfer requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ItemTransferRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Validate via hook
            {
                let resources_ref = resources as &ResourceContext;
                if self
                    .hook
                    .validate_transfer(
                        &request.from_entity,
                        &request.to_entity,
                        &request.item_id,
                        request.quantity,
                        resources_ref,
                    )
                    .await
                    .is_err()
                {
                    continue;
                }
            }

            // Transfer item (update state)
            {
                if let Some(mut state) = resources.get_mut::<InventoryState>().await {
                    if state
                        .transfer_item(
                            &request.from_entity,
                            &request.to_entity,
                            &request.item_id,
                            request.quantity,
                        )
                        .is_err()
                    {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Call hook
            self.hook
                .on_item_transferred(
                    &request.from_entity,
                    &request.to_entity,
                    &request.item_id,
                    request.quantity,
                    resources,
                )
                .await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(ItemTransferredEvent {
                    from_entity: request.from_entity.clone(),
                    to_entity: request.to_entity.clone(),
                    item_id: request.item_id.clone(),
                    quantity: request.quantity,
                });
            }
        }
    }
}

#[async_trait]
impl System for InventorySystem {
    fn name(&self) -> &'static str {
        "inventory_system"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
