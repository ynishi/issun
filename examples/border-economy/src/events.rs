//! Domain events flowing through the ISSUN event bus.
//!
//! Scenes and systems communicate exclusively via these typed payloads to keep
//! plugin modules decoupled.

use crate::models::ids::VaultId;
use crate::models::vault::{Vault, VaultReport};
use crate::models::{
    BudgetChannel, Currency, DemandProfile, FactionId, TerritoryId, WeaponPrototypeId,
};
use serde::{Deserialize, Serialize};

/// Fires when the strategy scene requests a new deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionRequested {
    pub faction: FactionId,
    pub target: TerritoryId,
    pub prototype: WeaponPrototypeId,
    pub expected_payout: Currency,
}

/// Published after a tactical scene resolves the battle simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionResolved {
    pub faction: FactionId,
    pub target: TerritoryId,
    pub casualties: u32,
    pub secured_share: f32,
    pub revenue_delta: Currency,
}

/// Queues a new research allocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchQueued {
    pub prototype: WeaponPrototypeId,
    pub budget: Currency,
    pub targeted_segment: DemandProfile,
}

/// Field telemetry from tactical ops that feed R&D quality scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldTestFeedback {
    pub prototype: WeaponPrototypeId,
    pub effectiveness: f32,
    pub reliability: f32,
}

/// Economy system snapshot for UI scenes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub total_share: f32,
    pub highlighted_channel: BudgetChannel,
    pub reputation_delta: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultDiscovered {
    pub vault: Vault,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInvested {
    pub vault_id: VaultId,
    pub slot_id: String,
    pub amount: Currency,
    pub channel: BudgetChannel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultCaptured {
    pub vault_id: VaultId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultReportGenerated {
    pub reports: Vec<VaultReport>,
}
