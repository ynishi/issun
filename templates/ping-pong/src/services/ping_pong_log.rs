//! Simple service to format Ping/Pong log messages with configurable pacing.

use crate::models::ping_pong::PingPongStage;

/// Formats messages describing ping/pong interactions, injecting Service-side
/// framing for whatever flavor text was selected from the asset deck.
#[derive(Clone, issun::Service)]
#[service(name = "ping_pong_log_service")]
pub struct PingPongLogService {
    congrats_every: u32,
}

impl PingPongLogService {
    #[allow(dead_code)]
    pub fn new(congrats_every: u32) -> Self {
        Self { congrats_every }
    }

    /// Produce a friendly message for the bounce that just happened.
    pub fn describe_bounce(
        &self,
        stage: PingPongStage,
        total_bounces: u32,
        flavor: &str,
        special_override: bool,
        heal_amount: Option<i32>,
    ) -> String {
        let special = special_override
            || (self.congrats_every > 0 && total_bounces % self.congrats_every == 0);
        let heal_suffix = heal_amount
            .filter(|amount| *amount > 0)
            .map(|amount| format!(" (+{} HP restored)", amount))
            .unwrap_or_default();

        if special {
            format!(
                "{} bounce #{total_bounces}! {flavor}{heal_suffix}",
                stage.label()
            )
        } else {
            format!(
                "{} bounce #{total_bounces} - {flavor}{heal_suffix}",
                stage.label()
            )
        }
    }
}

impl Default for PingPongLogService {
    fn default() -> Self {
        Self { congrats_every: 3 }
    }
}
