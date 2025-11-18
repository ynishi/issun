//! Minimal System demonstrating orchestration + Service usage.

use crate::models::{GameContext, PingPongMessageDeck, PingPongStage};
use crate::services::PingPongLogService;
use issun::prelude::{ResourceContext, ServiceContext};

/// Tracks ping/pong bounces and writes log entries to GameContext.
#[derive(Default, issun::System)]
#[system(name = "ping_pong_system")]
pub struct PingPongSystem {
    total_bounces: u32,
}

impl PingPongSystem {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a new bounce and push a log entry via PingPongLogService.
    pub fn process_bounce(
        &mut self,
        ctx: &mut GameContext,
        services: &ServiceContext,
        resources: &ResourceContext,
        stage: PingPongStage,
    ) -> PingPongBounceResult {
        self.total_bounces += 1;
        let special = self.total_bounces % 3 == 0;
        let heal_amount = if special { Some(10) } else { None };

        let flavor = resources
            .try_get_mut::<PingPongMessageDeck>()
            .map(|mut deck| deck.draw(special).to_string())
            .unwrap_or_else(|| "Keep bouncing!".to_string());

        let log_service = services
            .get_as::<PingPongLogService>(PingPongLogService::NAME)
            .expect("PingPongLogService must be registered via GameBuilder::with_service");

        if let Some(amount) = heal_amount {
            ctx.player.heal(amount);
        }

        let message =
            log_service.describe_bounce(stage, self.total_bounces, &flavor, special, heal_amount);
        ctx.ping_pong_log.push(message.clone());

        PingPongBounceResult {
            total_bounces: self.total_bounces,
            message,
            player_hp: ctx.player.hp,
        }
    }
}

pub struct PingPongBounceResult {
    pub total_bounces: u32,
    pub message: String,
    pub player_hp: i32,
}
