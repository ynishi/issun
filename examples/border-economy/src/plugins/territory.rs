use crate::events::MissionResolved;
use crate::models::GameContext;
use issun::plugin::PluginBuilderExt;
use issun::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct TerritoryPlugin;

#[async_trait::async_trait]
impl Plugin for TerritoryPlugin {
    fn name(&self) -> &'static str {
        "territory_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        builder.register_service(Box::new(TerritoryDemandService::default()));
        builder.register_system(Box::new(TerritorySystem::default()));
        builder.register_runtime_state(TerritoryStateCache::default());
    }
}

#[derive(Clone, Default, DeriveService)]
#[service(name = "territory_demand")]
pub struct TerritoryDemandService;

impl TerritoryDemandService {
    pub fn score(&self, demand: &crate::models::DemandProfile) -> f32 {
        demand.stability_bias * 0.5 + demand.violence_index * 0.5
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TerritoryStateCache {
    pub updates: Vec<String>,
    pub active_front: Option<String>,
    pub active_front_faction: Option<String>,
    pub faction_reports: Vec<String>,
}

#[derive(Default, DeriveSystem)]
#[system(name = "territory_system")]
pub struct TerritorySystem;

#[issun::event_handler(default_state = TerritoryStateCache)]
impl TerritorySystem {
    #[subscribe(MissionResolved)]
    async fn on_mission_resolved(
        &mut self,
        event: &MissionResolved,
        cache: &mut TerritoryStateCache,
        #[state] ctx: &mut GameContext,
    ) {
        cache.updates.insert(
            0,
            format!(
                "{} stability shift: +{:.0}% share ({} KIA)",
                event.target,
                event.secured_share * 100.0,
                event.casualties
            ),
        );
        cache.updates.truncate(6);

        if let Some(front) = ctx
            .territories
            .iter()
            .find(|territory| territory.battlefront)
        {
            cache.active_front = Some(front.id.as_str().to_string());
            cache.active_front_faction = ctx
                .enemy_faction_by_id(&front.enemy_faction)
                .map(|f| f.codename.clone());
        } else {
            cache.active_front = None;
            cache.active_front_faction = None;
        }

        cache.faction_reports = ctx
            .enemy_factions
            .iter()
            .map(|f| {
                format!(
                    "{}: 関係 {:.0}% / Hostility {:.0}% {}",
                    f.codename,
                    f.relations * 100.0,
                    f.hostility * 100.0,
                    f.last_event.as_deref().unwrap_or("特記事項なし")
                )
            })
            .collect();

        if let Some(territory) = ctx.territories.iter_mut().find(|t| t.id == event.target) {
            territory.enemy_share = (territory.enemy_share - event.secured_share).clamp(0.0, 1.0);
            territory.conflict_intensity = (territory.conflict_intensity * 0.75).clamp(0.1, 1.0);
            if territory.enemy_share < 0.2 {
                territory.battlefront = false;
            }
        }
    }
}
