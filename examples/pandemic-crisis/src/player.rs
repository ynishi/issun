//! Player actions and resources

use bevy::prelude::*;
use issun_bevy::plugins::action::*;
use issun_macros::IssunEntity;

/// Player resource
#[derive(Resource, IssunEntity)]
#[components(ActionPoints)]
pub struct Player {
    #[primary]
    pub entity: Entity,
}

/// Cure research progress
#[derive(Resource, Default, Clone)]
pub struct CureResearch {
    pub progress: f32, // 0.0 to 1.0 (100%)
    pub deployed: bool,
    pub deployment_turn: Option<u32>,
}

impl CureResearch {
    pub fn advance(&mut self, amount: f32) {
        self.progress = (self.progress + amount).min(1.0);
    }

    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }

    pub fn deploy(&mut self, current_turn: u32) {
        self.deployed = true;
        self.deployment_turn = Some(current_turn);
    }

    pub fn deployment_complete(&self, current_turn: u32) -> bool {
        if let Some(deploy_turn) = self.deployment_turn {
            current_turn >= deploy_turn + 3
        } else {
            false
        }
    }
}

/// Emergency healthcare budget
#[derive(Resource, Clone)]
pub struct EmergencyBudget {
    pub uses_remaining: u32,
    pub max_uses: u32,
}

impl Default for EmergencyBudget {
    fn default() -> Self {
        Self {
            uses_remaining: 2,
            max_uses: 2,
        }
    }
}

impl EmergencyBudget {
    pub fn can_use(&self) -> bool {
        self.uses_remaining > 0
    }

    pub fn use_budget(&mut self) -> bool {
        if self.can_use() {
            self.uses_remaining -= 1;
            true
        } else {
            false
        }
    }
}

/// Active quarantines
#[derive(Resource, Default, Clone)]
pub struct ActiveQuarantines {
    pub quarantines: Vec<Quarantine>,
}

#[derive(Clone)]
pub struct Quarantine {
    pub city_id: String,
    pub start_turn: u32,
    pub duration: u32,
}

impl Quarantine {
    pub fn is_active(&self, current_turn: u32) -> bool {
        current_turn < self.start_turn + self.duration
    }
}

impl ActiveQuarantines {
    pub fn add(&mut self, city_id: String, current_turn: u32, duration: u32) {
        self.quarantines.push(Quarantine {
            city_id,
            start_turn: current_turn,
            duration,
        });
    }

    pub fn is_quarantined(&self, city_id: &str, current_turn: u32) -> bool {
        self.quarantines
            .iter()
            .any(|q| q.city_id == city_id && q.is_active(current_turn))
    }

    pub fn cleanup(&mut self, current_turn: u32) {
        self.quarantines.retain(|q| q.is_active(current_turn));
    }

    pub fn active_count(&self, current_turn: u32) -> usize {
        self.quarantines
            .iter()
            .filter(|q| q.is_active(current_turn))
            .count()
    }
}

/// Active awareness campaigns
#[derive(Resource, Default, Clone)]
pub struct ActiveAwareness {
    pub campaigns: Vec<AwarenessCampaign>,
}

#[derive(Clone)]
pub struct AwarenessCampaign {
    pub city_id: String,
    pub start_turn: u32,
    pub duration: u32,
    pub resistance_boost: f32,
}

impl AwarenessCampaign {
    pub fn is_active(&self, current_turn: u32) -> bool {
        current_turn < self.start_turn + self.duration
    }
}

impl ActiveAwareness {
    pub fn add(&mut self, city_id: String, current_turn: u32) {
        self.campaigns.push(AwarenessCampaign {
            city_id,
            start_turn: current_turn,
            duration: 5,
            resistance_boost: 0.2,
        });
    }

    pub fn get_resistance_boost(&self, city_id: &str, current_turn: u32) -> f32 {
        self.campaigns
            .iter()
            .filter(|c| c.city_id == city_id && c.is_active(current_turn))
            .map(|c| c.resistance_boost)
            .sum()
    }

    pub fn cleanup(&mut self, current_turn: u32) {
        self.campaigns.retain(|c| c.is_active(current_turn));
    }
}

/// Active emergency healthcare
#[derive(Resource, Default, Clone)]
pub struct ActiveEmergencyHealthcare {
    pub campaigns: Vec<EmergencyHealthcareCampaign>,
}

#[derive(Clone)]
pub struct EmergencyHealthcareCampaign {
    pub city_id: String,
    pub start_turn: u32,
    pub duration: u32,
    pub resistance_boost: f32,
}

impl EmergencyHealthcareCampaign {
    pub fn is_active(&self, current_turn: u32) -> bool {
        current_turn < self.start_turn + self.duration
    }
}

impl ActiveEmergencyHealthcare {
    pub fn add(&mut self, city_id: String, current_turn: u32) {
        self.campaigns.push(EmergencyHealthcareCampaign {
            city_id,
            start_turn: current_turn,
            duration: 3,
            resistance_boost: 0.3,
        });
    }

    pub fn get_resistance_boost(&self, city_id: &str, current_turn: u32) -> f32 {
        self.campaigns
            .iter()
            .filter(|c| c.city_id == city_id && c.is_active(current_turn))
            .map(|c| c.resistance_boost)
            .sum()
    }

    pub fn cleanup(&mut self, current_turn: u32) {
        self.campaigns.retain(|c| c.is_active(current_turn));
    }
}

/// Travel ban status
#[derive(Resource, Default, Clone)]
pub struct TravelBanStatus {
    pub active: bool,
    pub start_turn: Option<u32>,
    pub duration: u32,
}

impl TravelBanStatus {
    pub fn activate(&mut self, current_turn: u32) {
        self.active = true;
        self.start_turn = Some(current_turn);
        self.duration = 2;
    }

    pub fn is_active(&self, current_turn: u32) -> bool {
        if let Some(start) = self.start_turn {
            current_turn < start + self.duration
        } else {
            false
        }
    }

    pub fn update(&mut self, current_turn: u32) {
        if self.is_active(current_turn) {
            // Still active
        } else {
            self.active = false;
            self.start_turn = None;
        }
    }
}

/// Setup player
pub fn setup_player(mut commands: Commands) {
    let player_entity = commands
        .spawn(ActionPoints::new(10))
        .id();

    commands.insert_resource(Player {
        entity: player_entity,
    });

    info!("Player created with 10 AP");
}

/// Regenerate AP at turn start
pub fn regenerate_ap(
    player: Res<Player>,
    mut action_points: Query<&mut ActionPoints>,
) {
    if let Ok(mut ap) = action_points.get_mut(player.entity) {
        ap.available = (ap.available + 8).min(15);
        info!("AP regenerated: {}/15", ap.available);
    }
}

/// Helper to consume N action points
pub fn try_consume_ap(ap: &mut ActionPoints, cost: u32) -> bool {
    if ap.can_consume(cost) {
        ap.available -= cost;
        true
    } else {
        false
    }
}
