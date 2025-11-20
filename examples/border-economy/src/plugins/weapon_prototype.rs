use crate::events::{FieldTestFeedback, ResearchQueued};
use issun::plugin::PluginBuilderExt;
use issun::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct WeaponPrototypePlugin;

#[async_trait::async_trait]
impl Plugin for WeaponPrototypePlugin {
    fn name(&self) -> &'static str {
        "weapon_prototype_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        builder.register_service(Box::new(FieldTelemetryService::default()));
        builder.register_system(Box::new(PrototypeSystem::default()));
        builder.register_runtime_state(PrototypeBacklog::default());
    }
}

#[derive(Clone, Default, DeriveService)]
#[service(name = "field_telemetry")]
pub struct FieldTelemetryService;

impl FieldTelemetryService {
    pub fn quality_modifier(&self, reliability: f32) -> f32 {
        reliability.clamp(0.2, 1.2)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrototypeBacklog {
    pub queued: Vec<String>,
    pub field_reports: Vec<String>,
}

#[derive(Default, DeriveSystem)]
#[system(name = "prototype_system")]
pub struct PrototypeSystem;

#[issun::event_handler(default_state = PrototypeBacklog)]
impl PrototypeSystem {
    #[subscribe(ResearchQueued)]
    async fn on_research_queued(&mut self, event: &ResearchQueued, backlog: &mut PrototypeBacklog) {
        backlog
            .queued
            .insert(0, format!("{} +{}c", event.prototype, event.budget.amount()));
        backlog.queued.truncate(6);
    }

    #[subscribe(FieldTestFeedback)]
    async fn on_field_test_feedback(
        &mut self,
        event: &FieldTestFeedback,
        backlog: &mut PrototypeBacklog,
    ) {
        backlog.field_reports.insert(
            0,
            format!(
                "{} eff {:>3.0}% / rel {:>3.0}%",
                event.prototype,
                event.effectiveness * 100.0,
                event.reliability * 100.0
            ),
        );
        backlog.field_reports.truncate(6);
    }
}
