use crate::events::MissionResolved;
use crate::models::GameContext;
use issun::event::EventBus;
use issun::plugin::PluginBuilderExt;
use issun::prelude::*;
use serde::{Deserialize, Serialize};
use std::any::Any;

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

#[derive(Clone, Default)]
pub struct TerritoryDemandService;

impl TerritoryDemandService {
    pub fn score(&self, demand: &crate::models::DemandProfile) -> f32 {
        demand.stability_bias * 0.5 + demand.violence_index * 0.5
    }
}

#[async_trait::async_trait]
impl Service for TerritoryDemandService {
    fn name(&self) -> &'static str {
        "territory_demand"
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
pub struct TerritoryStateCache {
    pub updates: Vec<String>,
    pub active_front: Option<String>,
    pub active_front_faction: Option<String>,
    pub faction_reports: Vec<String>,
}

#[derive(Default)]
pub struct TerritorySystem;

#[async_trait::async_trait]
impl System for TerritorySystem {
    fn name(&self) -> &'static str {
        "territory_system"
    }

    async fn update(&mut self, _ctx: &mut Context) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl TerritorySystem {
    pub async fn process_events(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        let resolved = match resources.get_mut::<EventBus>().await {
            Some(mut bus) => super::collect_events::<MissionResolved>(&mut bus),
            None => return,
        };

        if resolved.is_empty() {
            return;
        }

        if let Some(mut cache) = resources.get_mut::<TerritoryStateCache>().await {
            for event in &resolved {
                cache.updates.insert(
                    0,
                    format!(
                        "{} stability shift: +{:.0}% share ({} KIA)",
                        event.target,
                        event.secured_share * 100.0,
                        event.casualties
                    ),
                );
            }
            cache.updates.truncate(6);

            if let Some(ctx) = resources.try_get::<GameContext>() {
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
            }
        }

        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            for event in &resolved {
                if let Some(territory) = ctx.territories.iter_mut().find(|t| t.id == event.target) {
                    territory.enemy_share =
                        (territory.enemy_share - event.secured_share).clamp(0.0, 1.0);
                    territory.conflict_intensity =
                        (territory.conflict_intensity * 0.75).clamp(0.1, 1.0);
                    if territory.enemy_share < 0.2 {
                        territory.battlefront = false;
                    }
                }
            }
        }
    }
}
