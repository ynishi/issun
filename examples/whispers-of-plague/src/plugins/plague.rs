use crate::services::{RumorService, VirusService, WinConditionService};
use crate::systems::TurnSystem;
use issun::prelude::*;

/// Main game plugin bundling all plague game components
#[derive(Default)]
pub struct PlagueGamePlugin;

#[async_trait::async_trait]
impl Plugin for PlagueGamePlugin {
    fn name(&self) -> &'static str {
        "plague_game"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register services (pure logic)
        builder.register_service(Box::new(VirusService));
        builder.register_service(Box::new(RumorService));
        builder.register_service(Box::new(WinConditionService));

        // Register system (orchestration)
        builder.register_system(Box::new(TurnSystem::new()));
    }
}
