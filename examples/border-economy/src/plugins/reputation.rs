use crate::events::MissionResolved;
use crate::models::GameContext;
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

#[issun::event_handler(default_state = ReputationLedger)]
impl ReputationSystem {
    #[subscribe(MissionResolved)]
    async fn on_mission_resolved(
        &mut self,
        event: &MissionResolved,
        ledger: &mut ReputationLedger,
        #[service(name = "reputation_service")] service: &ReputationService,
        #[state] ctx: &mut GameContext,
    ) {
        let delta = (event.secured_share * 50.0) - (event.casualties as f32 * 0.05);
        service.apply_delta(ctx, delta);
        ledger
            .events
            .insert(0, format!("{}: Δrep {:+.1}", event.target.as_str(), delta));
        ledger.events.truncate(8);
    }
}
