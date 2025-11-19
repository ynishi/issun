pub mod economy;
pub mod faction;
pub mod market_share;
pub mod reputation;
pub mod territory;
pub mod vault;
pub mod weapon_prototype;

use issun::prelude::{ResourceContext, ServiceContext, SystemContext};

pub use economy::EconomyPlugin;
pub use faction::FactionPlugin;
pub use market_share::MarketSharePlugin;
pub use reputation::ReputationPlugin;
pub use territory::TerritoryPlugin;
pub use vault::VaultPlugin;
pub use weapon_prototype::WeaponPrototypePlugin;

use self::economy::EconomySystem;
use self::faction::FactionSystem;
use self::market_share::MarketShareSystem;
use self::reputation::ReputationSystem;
use self::territory::TerritorySystem;
use self::vault::VaultSystem;
use self::weapon_prototype::PrototypeSystem;
pub use economy::EconomyState;
pub use faction::FactionOpsState;
pub use market_share::MarketPulse;
pub use reputation::ReputationLedger;
pub use territory::TerritoryStateCache;
pub use vault::VaultState;
pub use weapon_prototype::PrototypeBacklog;

/// Pump every plugin system so they can react to newly dispatched events.
pub async fn pump_event_systems(
    services: &ServiceContext,
    systems: &mut SystemContext,
    resources: &mut ResourceContext,
) {
    if let Some(system) = systems.get_mut::<FactionSystem>() {
        system.process_events(services, resources).await;
    }
    if let Some(system) = systems.get_mut::<EconomySystem>() {
        system.process_events(services, resources).await;
    }
    if let Some(system) = systems.get_mut::<TerritorySystem>() {
        system.process_events(services, resources).await;
    }
    if let Some(system) = systems.get_mut::<PrototypeSystem>() {
        system.process_events(services, resources).await;
    }
    if let Some(system) = systems.get_mut::<MarketShareSystem>() {
        system.process_events(services, resources).await;
    }
    if let Some(system) = systems.get_mut::<ReputationSystem>() {
        system.process_events(services, resources).await;
    }
    if let Some(system) = systems.get_mut::<VaultSystem>() {
        system.process_events(services, resources).await;
    }
}
