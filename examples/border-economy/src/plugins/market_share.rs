use crate::events::MissionResolved;
use crate::models::GameContext;
use issun::event::EventBus;
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

impl MarketShareSystem {
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

        let base_share = if let (Some(intel), Some(ctx)) = (
            services.get_as::<MarketIntelService>("market_intel"),
            resources.get::<GameContext>().await,
        ) {
            intel.estimate_share(&ctx)
        } else {
            0.0
        };

        if let Some(mut pulse) = resources.get_mut::<MarketPulse>().await {
            pulse.snapshots.push(base_share);
            for report in resolved {
                pulse.snapshots.push(report.secured_share);
            }
            pulse.snapshots.truncate(10);
        }
    }
}
