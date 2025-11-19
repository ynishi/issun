//! Domain events flowing through the ISSUN event bus.
//!
//! Scenes and systems communicate exclusively via these typed payloads to keep
//! plugin modules decoupled.

use crate::models::ids::VaultId;
use crate::models::vault::{Vault, VaultReport};
use crate::models::{
    BudgetChannel, Currency, DemandProfile, FactionId, TerritoryId, WeaponPrototypeId,
};
use issun::event;

event! {
    /// Fires when the strategy scene requests a new deployment.
    pub MissionRequested {
        pub faction: FactionId,
        pub target: TerritoryId,
        pub prototype: WeaponPrototypeId,
        pub expected_payout: Currency,
    }

    /// Published after a tactical scene resolves the battle simulation.
    pub MissionResolved {
        pub faction: FactionId,
        pub target: TerritoryId,
        pub casualties: u32,
        pub secured_share: f32,
        pub revenue_delta: Currency,
    }

    /// Queues a new research allocation.
    pub ResearchQueued {
        pub prototype: WeaponPrototypeId,
        pub budget: Currency,
        pub targeted_segment: DemandProfile,
    }

    /// Field telemetry from tactical ops that feed R&D quality scores.
    pub FieldTestFeedback {
        pub prototype: WeaponPrototypeId,
        pub effectiveness: f32,
        pub reliability: f32,
    }

    /// Economy system snapshot for UI scenes.
    pub MarketSnapshot {
        pub total_share: f32,
        pub highlighted_channel: BudgetChannel,
        pub reputation_delta: f32,
    }

    pub VaultDiscovered {
        pub vault: Vault,
    }

    pub VaultInvested {
        pub vault_id: VaultId,
        pub slot_id: String,
        pub amount: Currency,
        pub channel: BudgetChannel,
    }

    pub VaultCaptured {
        pub vault_id: VaultId,
    }

    pub VaultReportGenerated {
        pub reports: Vec<VaultReport>,
    }
}
