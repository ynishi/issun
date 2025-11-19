use crate::events::MissionResolved;
use crate::models::GameContext;
use issun::event::EventBus;
use issun::plugin::PluginBuilderExt;
use issun::prelude::*;
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(Default)]
pub struct ReputationPlugin;

#[async_trait::async_trait]
impl Plugin for ReputationPlugin {
    fn name(&self) -> &'static str {
        "reputation_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        builder.register_service(Box::new(ReputationService::default()));
        builder.register_system(Box::new(ReputationSystem::default()));
        builder.register_runtime_state(ReputationLedger::default());
    }
}

#[derive(Clone, Default)]
pub struct ReputationService;

impl ReputationService {
    pub fn apply_delta(&self, ctx: &mut crate::models::GameContext, delta: f32) {
        ctx.reputation.adjust(delta);
    }
}

#[async_trait::async_trait]
impl Service for ReputationService {
    fn name(&self) -> &'static str {
        "reputation_service"
    }

    fn clone_box(&self) -> Box<dyn Service> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReputationLedger {
    pub events: Vec<String>,
}

#[derive(Default)]
pub struct ReputationSystem;

#[async_trait::async_trait]
impl System for ReputationSystem {
    fn name(&self) -> &'static str {
        "reputation_system"
    }

    async fn update(&mut self, _ctx: &mut Context) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ReputationSystem {
    pub async fn process_events(
        &mut self,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        let resolved = match resources.get_mut::<EventBus>().await {
            Some(mut bus) => super::collect_events::<MissionResolved>(&mut bus),
            None => return,
        };

        if resolved.is_empty() {
            return;
        }

        let deltas: Vec<f32> = resolved
            .iter()
            .map(|event| (event.secured_share * 50.0) - (event.casualties as f32 * 0.05))
            .collect();

        if let Some(service) = services.get_as::<ReputationService>("reputation_service") {
            if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
                for delta in &deltas {
                    service.apply_delta(&mut ctx, *delta);
                }
            }
        }

        if let Some(mut ledger) = resources.get_mut::<ReputationLedger>().await {
            for (event, delta) in resolved.into_iter().zip(deltas.into_iter()) {
                ledger
                    .events
                    .insert(0, format!("{}: Δrep {:+.1}", event.target.as_str(), delta));
            }
            ledger.events.truncate(8);
        }
    }
}
