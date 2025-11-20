use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

// Re-export Currency from issun
pub use issun::plugin::Currency;

/// Identifier for player-controlled factions or contractors.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactionId(String);

impl FactionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for FactionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifier for contested territories.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TerritoryId(String);

impl TerritoryId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for TerritoryId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifier for high-risk Vault sites (new planets).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VaultId(String);

impl VaultId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for VaultId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifier for a prototype weapon platform.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WeaponPrototypeId(String);

impl WeaponPrototypeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl Display for WeaponPrototypeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifier for market demand segments.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MarketSegmentId(String);

impl MarketSegmentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl Display for MarketSegmentId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Budget channels where currency can be allocated.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BudgetChannel {
    Research,
    Operations,
    Reserve,
    Innovation,
    Security,
}

impl Display for BudgetChannel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let label = match self {
            BudgetChannel::Research => "R&D",
            BudgetChannel::Operations => "Ops",
            BudgetChannel::Reserve => "Reserve",
            BudgetChannel::Innovation => "Innovation",
            BudgetChannel::Security => "Security",
        };
        write!(f, "{}", label)
    }
}

// Currency type removed - now using issun::plugin::Currency

/// Reputation metrics feed bonuses into demand & payouts.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReputationStanding {
    pub stability: f32,
    pub innovation: f32,
    pub trust: f32,
}

impl ReputationStanding {
    pub fn adjust(&mut self, delta: f32) {
        self.stability = (self.stability + delta).clamp(0.0, 100.0);
        self.innovation = (self.innovation + delta * 0.8).clamp(0.0, 100.0);
        self.trust = (self.trust + delta * 0.6).clamp(0.0, 100.0);
    }
}

/// Demand profile for a territory/segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandProfile {
    pub stability_bias: f32,
    pub violence_index: f32,
    pub logistics_weight: f32,
}

impl DemandProfile {
    pub fn frontier() -> Self {
        Self {
            stability_bias: 0.4,
            violence_index: 0.8,
            logistics_weight: 0.5,
        }
    }

    pub fn metroplex() -> Self {
        Self {
            stability_bias: 0.8,
            violence_index: 0.3,
            logistics_weight: 0.7,
        }
    }
}

impl Default for DemandProfile {
    fn default() -> Self {
        Self::frontier()
    }
}
