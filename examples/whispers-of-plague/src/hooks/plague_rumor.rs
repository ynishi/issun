use crate::models::{CityMap, GameMode, PlagueGameContext};
use crate::plugins::rumor::{Rumor, RumorEffect, RumorHook, RumorId};
use async_trait::async_trait;
use issun::prelude::ResourceContext;

/// Game-specific rumor hook for Whispers of Plague
#[derive(Debug, Clone, Copy, Default)]
pub struct PlagueRumorHook;

#[async_trait]
impl RumorHook for PlagueRumorHook {
    async fn on_rumor_applied(
        &self,
        _rumor: &Rumor,
        effect: &RumorEffect,
        resources: &mut ResourceContext,
    ) {
        // Plague mode rumors have bonus panic increase
        if let Some(ctx) = resources.get::<PlagueGameContext>().await {
            if ctx.mode == GameMode::Plague {
                if matches!(effect, RumorEffect::IncreasePanic(_)) {
                    // Bonus panic increase in Plague mode
                    if let Some(mut city) = resources.get_mut::<CityMap>().await {
                        for district in &mut city.districts {
                            district.panic_level = (district.panic_level * 1.1).min(1.0);
                        }
                    }
                }
            }
        }
    }

    async fn calculate_migration_target(
        &self,
        from_district: usize,
        resources: &ResourceContext,
    ) -> Option<usize> {
        let ctx = resources.get::<PlagueGameContext>().await?;
        let city = resources.get::<CityMap>().await?;

        match ctx.mode {
            GameMode::Plague => {
                // Find most infected district (trap)
                city.districts
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != from_district)
                    .max_by_key(|(_, d)| d.infected)
                    .map(|(i, _)| i)
            }
            GameMode::Savior => {
                // Find least infected district (safe)
                city.districts
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != from_district)
                    .min_by_key(|(_, d)| d.infected)
                    .map(|(i, _)| i)
            }
        }
    }
}
