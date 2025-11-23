use crate::models::{CityMap, GameMode, PlagueGameContext, VictoryResult};
use crate::services::WinConditionService;
use issun::prelude::*;
use issun::Plugin;

/// Minimal win condition plugin (only custom logic)
#[derive(Plugin)]
#[plugin(name = "whispers:win_condition")]
pub struct WinConditionPlugin {
    #[plugin(service)]
    win_service: WinConditionService,
}

impl WinConditionPlugin {
    pub fn new() -> Self {
        Self {
            win_service: WinConditionService,
        }
    }

    /// Check victory conditions (called from Scene layer)
    pub async fn check_victory(resources: &ResourceContext) -> Option<VictoryResult> {
        let ctx = resources.get::<PlagueGameContext>().await?;
        let city_map = resources.get::<CityMap>().await?;

        // Service is stateless, so we can instantiate it directly
        let win_service = WinConditionService;

        match ctx.mode {
            GameMode::Plague => win_service.check_plague_victory(&city_map, ctx.turn, ctx.max_turns),
            GameMode::Savior => win_service.check_savior_victory(&city_map, ctx.turn, ctx.max_turns),
        }
    }
}

impl Default for WinConditionPlugin {
    fn default() -> Self {
        Self::new()
    }
}
