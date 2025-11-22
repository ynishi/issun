pub mod plague;
pub mod rumor;

pub use plague::PlagueGamePlugin;
pub use rumor::RumorPlugin;

use issun::prelude::{ResourceContext, ServiceContext, SystemContext};

/// Pump event systems (required by auto_pump macro)
/// This game doesn't use events yet, so this is a no-op
pub async fn pump_event_systems(
    _services: &ServiceContext,
    _systems: &mut SystemContext,
    _resources: &mut ResourceContext,
) {
    // No event-based systems yet
}
