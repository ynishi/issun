//! Loot system implementation

use crate::context::{ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

use super::config::LootConfig;
use super::events::*;
use super::hook::LootHook;
use super::service::LootService;

/// System that processes loot events with hooks
///
/// This system:
/// 1. Processes loot generation requests
/// 2. Processes rarity roll requests
/// 3. Calls hooks for custom behavior
/// 4. Publishes state change events for network replication
///
/// # Feedback Loop
///
/// ```text
/// Command Event → Drop Roll (Service) → Hook (Generate Items) → Loot Event
/// ```
#[derive(Clone)]
pub struct LootSystem {
    hook: Arc<dyn LootHook>,
}

impl LootSystem {
    /// Create a new LootSystem with a custom hook
    pub fn new(hook: Arc<dyn LootHook>) -> Self {
        Self { hook }
    }

    /// Process all loot events
    pub async fn process_events(
        &mut self,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        self.process_loot_generate_requests(services, resources)
            .await;
        self.process_rarity_roll_requests(services, resources).await;
    }

    /// Process loot generation requests
    async fn process_loot_generate_requests(
        &mut self,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect loot generate requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<LootGenerateRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Get global multiplier
            let global_multiplier = {
                if let Some(config) = resources.get::<LootConfig>().await {
                    config.global_drop_multiplier
                } else {
                    1.0
                }
            };

            // Modify drop chance via hook
            let final_drop_rate = {
                let resources_ref = resources as &ResourceContext;
                self.hook
                    .modify_drop_chance(&request.source_id, request.drop_rate, resources_ref)
                    .await
            };

            // Apply global multiplier
            let effective_rate = (final_drop_rate * global_multiplier).min(1.0);

            // Roll for drop using service
            let should_drop = {
                if let Some(_service) = services.get_as::<LootService>("loot_service") {
                    let drop_config = super::types::DropConfig::new(effective_rate, 1.0);
                    let mut rng = rand::thread_rng();
                    LootService::should_drop(&drop_config, &mut rng)
                } else {
                    rand::random::<f32>() < effective_rate
                }
            };

            if !should_drop {
                // Publish no-loot event
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(LootNotGeneratedEvent {
                        source_id: request.source_id.clone(),
                    });
                }
                continue;
            }

            // Select rarity using service
            let rarity = {
                if let Some(_service) = services.get_as::<LootService>("loot_service") {
                    let mut rng = rand::thread_rng();
                    LootService::select_rarity(&mut rng)
                } else {
                    super::types::Rarity::Common
                }
            };

            // Generate loot items via hook
            let items = {
                let resources_ref = resources as &ResourceContext;
                self.hook
                    .generate_loot(&request.source_id, rarity, resources_ref)
                    .await
            };

            // Call hook
            self.hook
                .on_loot_generated(&request.source_id, &items, rarity, resources)
                .await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(LootGeneratedEvent {
                    source_id: request.source_id.clone(),
                    items,
                    rarity,
                });
            }
        }
    }

    /// Process rarity roll requests
    async fn process_rarity_roll_requests(
        &mut self,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect rarity roll requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<RarityRollRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Select rarity using service
            let rarity = {
                if let Some(_service) = services.get_as::<LootService>("loot_service") {
                    let mut rng = rand::thread_rng();
                    LootService::select_rarity(&mut rng)
                } else {
                    super::types::Rarity::Common
                }
            };

            // Generate loot items via hook
            let items = {
                let resources_ref = resources as &ResourceContext;
                self.hook
                    .generate_loot(&request.source_id, rarity, resources_ref)
                    .await
            };

            // Call hook
            self.hook
                .on_loot_generated(&request.source_id, &items, rarity, resources)
                .await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(LootGeneratedEvent {
                    source_id: request.source_id.clone(),
                    items,
                    rarity,
                });
            }
        }
    }
}

#[async_trait]
impl System for LootSystem {
    fn name(&self) -> &'static str {
        "loot_system"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
