use crate::events::{MissionRequested, MissionResolved};
use crate::models::TerritoryId;
use issun::prelude::*;
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(Default, DerivePlugin)]
#[plugin(name = "faction_plugin")]
#[plugin(service = MissionPlannerService)]
#[plugin(system = FactionSystem)]
#[plugin(state = FactionOpsState)]
pub struct FactionPlugin;

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

#[issun::event_handler]
impl FactionSystem {
    #[subscribe(MissionRequested)]
    async fn on_mission_requested(
        &mut self,
        event: &MissionRequested,
        #[state] ops: &mut FactionOpsState,
    ) {
        ops.sorties_launched = ops.sorties_launched.saturating_add(1);
        ops.active_operations.push(event.target.clone());
        ops.recent_reports
            .insert(0, format!("{} deploys to {}", event.faction, event.target));
        ops.recent_reports.truncate(5);
    }

    #[subscribe(MissionResolved)]
    async fn on_mission_resolved(
        &mut self,
        event: &MissionResolved,
        #[state] ops: &mut FactionOpsState,
    ) {
        ops.recent_reports.insert(
            0,
            format!(
                "{} secured {:.0}% share in {}",
                event.faction,
                event.secured_share * 100.0,
                event.target
            ),
        );
        ops.active_operations.retain(|t| t != &event.target);
        ops.total_casualties += event.casualties as u64;
        ops.recent_reports.truncate(5);
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
