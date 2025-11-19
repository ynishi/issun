use crate::events::{MissionRequested, MissionResolved};
use crate::models::TerritoryId;
use issun::event::EventBus;
use issun::plugin::PluginBuilderExt;
use issun::prelude::*;
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(Default)]
pub struct FactionPlugin;

#[async_trait::async_trait]
impl Plugin for FactionPlugin {
    fn name(&self) -> &'static str {
        "faction_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        builder.register_service(Box::new(MissionPlannerService::default()));
        builder.register_system(Box::new(FactionSystem::default()));
        builder.register_runtime_state(FactionOpsState::default());
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactionOpsState {
    pub sorties_launched: u32,
    pub active_operations: Vec<TerritoryId>,
    pub recent_reports: Vec<String>,
    pub total_casualties: u64,
}

#[derive(Clone, Default)]
pub struct MissionPlannerService;

#[async_trait::async_trait]
impl Service for MissionPlannerService {
    fn name(&self) -> &'static str {
        "mission_planner"
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

pub struct FactionSystem {
    pub ticks: u32,
}

impl Default for FactionSystem {
    fn default() -> Self {
        Self { ticks: 0 }
    }
}

impl FactionSystem {
    pub async fn process_events(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        let (requests, resolved) = match resources.get_mut::<EventBus>().await {
            Some(mut bus) => (
                super::collect_events::<MissionRequested>(&mut bus),
                super::collect_events::<MissionResolved>(&mut bus),
            ),
            None => return,
        };

        if requests.is_empty() && resolved.is_empty() {
            return;
        }

        if let Some(mut ops) = resources.get_mut::<FactionOpsState>().await {
            for request in requests {
                ops.sorties_launched = ops.sorties_launched.saturating_add(1);
                ops.active_operations.push(request.target.clone());
                ops.recent_reports.insert(
                    0,
                    format!("{} deploys to {}", request.faction, request.target),
                );
            }

            for result in resolved {
                ops.recent_reports.insert(
                    0,
                    format!(
                        "{} secured {:.0}% share in {}",
                        result.faction,
                        result.secured_share * 100.0,
                        result.target
                    ),
                );
                ops.active_operations.retain(|t| t != &result.target);
                ops.total_casualties += result.casualties as u64;
            }

            ops.recent_reports.truncate(5);
        }
    }
}

#[async_trait::async_trait]
impl System for FactionSystem {
    fn name(&self) -> &'static str {
        "faction_system"
    }

    async fn update(&mut self, _ctx: &mut Context) {
        self.ticks += 1;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
