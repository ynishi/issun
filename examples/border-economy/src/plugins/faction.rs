use crate::events::{MissionRequested, MissionResolved};
use crate::models::TerritoryId;
use issun::plugin::faction::*;
use issun::plugin::PluginBuilderExt;
use issun::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Border-economy faction statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactionOpsState {
    pub sorties_launched: u32,
    pub active_operations: Vec<TerritoryId>,
    pub recent_reports: Vec<String>,
    pub total_casualties: u64,
}

/// Custom hook for border-economy faction behavior
struct BorderEconomyFactionHook;

#[async_trait::async_trait]
impl FactionHook for BorderEconomyFactionHook {
    async fn on_operation_launched(
        &self,
        _faction: &Faction,
        operation: &Operation,
        resources: &mut ResourceContext,
    ) {
        // Update FactionOpsState statistics
        if let Some(mut state) = resources.get_mut::<FactionOpsState>().await {
            state.sorties_launched = state.sorties_launched.saturating_add(1);

            // Extract target from metadata
            if let Some(target) = operation.metadata["target"].as_str() {
                state.active_operations.push(TerritoryId::new(target));
            }

            // Extract faction name from metadata
            let faction_name = operation.metadata["faction"]
                .as_str()
                .unwrap_or("Unknown");
            let target_name = operation.metadata["target"].as_str().unwrap_or("Unknown");

            state.recent_reports.insert(
                0,
                format!("{} deploys to {}", faction_name, target_name),
            );
            state.recent_reports.truncate(5);
        }
    }

    async fn on_operation_completed(
        &self,
        _faction: &Faction,
        operation: &Operation,
        outcome: &Outcome,
        resources: &mut ResourceContext,
    ) {
        // Update FactionOpsState statistics
        if let Some(mut state) = resources.get_mut::<FactionOpsState>().await {
            // Extract data from outcome
            let faction_name = operation.metadata["faction"]
                .as_str()
                .unwrap_or("Unknown");
            let target_name = operation.metadata["target"].as_str().unwrap_or("Unknown");
            let secured_share = outcome.metrics.get("secured_share").unwrap_or(&0.0);
            let casualties = outcome.metrics.get("casualties").unwrap_or(&0.0);

            state.recent_reports.insert(
                0,
                format!(
                    "{} secured {:.0}% share in {}",
                    faction_name,
                    secured_share * 100.0,
                    target_name
                ),
            );

            // Remove from active operations
            if let Some(target) = operation.metadata["target"].as_str() {
                state
                    .active_operations
                    .retain(|t| t.as_str() != target);
            }

            state.total_casualties += *casualties as u64;
            state.recent_reports.truncate(5);
        }
    }
}

/// Bridge system that converts BE events to issun faction events
#[derive(Default, DeriveSystem)]
#[system(name = "faction_bridge_system")]
pub struct FactionBridgeSystem;

impl FactionBridgeSystem {
    pub async fn process_events(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        self.bridge_mission_requests(resources).await;
        self.bridge_mission_resolutions(resources).await;
    }

    async fn bridge_mission_requests(&mut self, resources: &mut ResourceContext) {
        // Collect MissionRequested events
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<MissionRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Convert to OperationLaunchRequested
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(OperationLaunchRequested {
                    faction_id: FactionId::new(request.faction.as_str()),
                    operation_name: format!("Mission to {}", request.target),
                    metadata: json!({
                        "faction": request.faction.as_str(),
                        "target": request.target.as_str(),
                        "prototype": request.prototype.as_str(),
                        "expected_payout": request.expected_payout.amount(),
                    }),
                });
            }
        }
    }

    async fn bridge_mission_resolutions(&mut self, resources: &mut ResourceContext) {
        // Collect MissionResolved events
        let resolutions = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<MissionResolved>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        // Find matching operations and resolve them
        for resolution in resolutions {
            // Find operation ID by matching faction + target in metadata
            let operation_id = {
                if let Some(registry) = resources.get::<FactionRegistry>().await {
                    registry
                        .operations()
                        .find(|op| {
                            op.metadata["faction"].as_str() == Some(resolution.faction.as_str())
                                && op.metadata["target"].as_str()
                                    == Some(resolution.target.as_str())
                                && !op.is_completed()
                                && !op.is_failed()
                        })
                        .map(|op| op.id.clone())
                } else {
                    None
                }
            };

            if let Some(op_id) = operation_id {
                // Publish OperationResolveRequested
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(OperationResolveRequested {
                        operation_id: op_id.clone(),
                        outcome: Outcome::new(op_id.as_str(), true)
                            .with_metric("casualties", resolution.casualties as f32)
                            .with_metric("secured_share", resolution.secured_share)
                            .with_metric("revenue_delta", resolution.revenue_delta.amount() as f32)
                            .with_metadata(json!({
                                "faction": resolution.faction.as_str(),
                                "target": resolution.target.as_str(),
                            })),
                    });
                }
            }
        }
    }
}

/// Border-economy faction plugin (using issun FactionPlugin)
pub struct FactionPlugin {
    issun_plugin: issun::plugin::FactionPlugin,
}

impl Default for FactionPlugin {
    fn default() -> Self {
        Self {
            issun_plugin: issun::plugin::FactionPlugin::new()
                .with_hook(BorderEconomyFactionHook),
        }
    }
}

#[async_trait::async_trait]
impl Plugin for FactionPlugin {
    fn name(&self) -> &'static str {
        "border_economy_faction_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register issun FactionPlugin components
        self.issun_plugin.build(builder);

        // Register BE-specific components
        builder.register_runtime_state(FactionOpsState::default());
        builder.register_system(Box::new(FactionBridgeSystem::default()));
    }

    async fn initialize(&mut self) {
        self.issun_plugin.initialize().await;
    }
}
