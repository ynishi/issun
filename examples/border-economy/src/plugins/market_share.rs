use crate::events::MissionResolved;
use issun::plugin::PluginBuilderExt;
use issun::prelude::*;
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(Default)]
pub struct MarketSharePlugin;

#[async_trait::async_trait]
impl Plugin for MarketSharePlugin {
    fn name(&self) -> &'static str {
        "market_share_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        builder.register_service(Box::new(MarketIntelService::default()));
        builder.register_system(Box::new(MarketShareSystem::default()));
        builder.register_runtime_state(MarketPulse::default());
    }
}

#[derive(Clone, Default)]
pub struct MarketIntelService;

impl MarketIntelService {
    pub fn estimate_share(&self, ctx: &crate::models::GameContext) -> f32 {
        ctx.territories.iter().map(|t| t.control).sum::<f32>() / ctx.territories.len() as f32
    }
}

#[async_trait::async_trait]
impl Service for MarketIntelService {
    fn name(&self) -> &'static str {
        "market_intel"
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
pub struct MarketPulse {
    pub snapshots: Vec<f32>,
}

#[derive(Default)]
pub struct MarketShareSystem;

#[async_trait::async_trait]
impl System for MarketShareSystem {
    fn name(&self) -> &'static str {
        "market_share_system"
    }

    async fn update(&mut self, _ctx: &mut Context) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[issun::event_handler(default_state = MarketPulse)]
impl MarketShareSystem {
    #[subscribe(MissionResolved)]
    async fn on_mission_resolved(
        &mut self,
        event: &MissionResolved,
        pulse: &mut MarketPulse,
        #[service(name = "market_intel")] intel: &MarketIntelService,
        #[state] ctx: &mut crate::models::GameContext,
    ) {
        let base_share = intel.estimate_share(ctx);
        pulse.snapshots.push(base_share);
        pulse.snapshots.push(event.secured_share);
        pulse.snapshots.truncate(10);
    }
}
