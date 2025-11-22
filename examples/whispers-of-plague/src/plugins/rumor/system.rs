use super::hook::RumorHook;
use super::models::{Rumor, RumorEffect, RumorId, RumorRegistry, RumorState};
use super::service::RumorService;
use crate::models::{CityMap, PlagueGameContext};
use issun::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Rumor system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RumorConfig {
    pub decay_rate: f32,
    pub effectiveness_threshold: f32,
    pub max_active_rumors: usize,
}

impl Default for RumorConfig {
    fn default() -> Self {
        Self {
            decay_rate: 0.9,
            effectiveness_threshold: 0.1,
            max_active_rumors: 3,
        }
    }
}

/// Rumor orchestration system
#[derive(DeriveSystem)]
#[system(name = "rumor_system")]
pub struct RumorSystem {
    service: RumorService,
    hook: Arc<dyn RumorHook>,
    config: RumorConfig,
}

impl RumorSystem {
    pub fn new(hook: Arc<dyn RumorHook>) -> Self {
        Self {
            service: RumorService,
            hook,
            config: RumorConfig::default(),
        }
    }

    /// Apply a rumor by ID
    pub async fn apply_rumor(
        &mut self,
        rumor_id: &RumorId,
        resources: &mut ResourceContext,
    ) -> std::result::Result<String, String> {
        // 1. Get rumor definition
        let rumor = {
            let registry = resources
                .get::<RumorRegistry>()
                .await
                .ok_or("RumorRegistry not found")?;
            registry
                .get(rumor_id)
                .ok_or(format!("Rumor {} not found", rumor_id))?
                .clone()
        };

        // 2. Validate via hook
        self.hook.on_before_apply(&rumor, resources).await?;

        // 3. Check max active rumors
        {
            let state = resources
                .get::<RumorState>()
                .await
                .ok_or("RumorState not found")?;
            if state.active_rumors.len() >= self.config.max_active_rumors {
                return Err(format!(
                    "Max active rumors reached ({})",
                    self.config.max_active_rumors
                ));
            }
        }

        // 4. Apply effect
        let log_message = self.apply_effect(&rumor, resources).await?;

        // 5. Call hook
        self.hook
            .on_rumor_applied(&rumor, &rumor.effect, resources)
            .await;

        // 6. Activate rumor in state
        {
            let mut state = resources
                .get_mut::<RumorState>()
                .await
                .ok_or("RumorState not found")?;
            state.activate(rumor.id.clone(), rumor.initial_credibility);
            state.record_history(log_message.clone());
        }

        Ok(log_message)
    }

    /// Decay all active rumors (called per turn)
    pub async fn decay_rumors(&mut self, resources: &mut ResourceContext) -> Vec<String> {
        let mut logs = vec![];

        let expired: Vec<RumorId> = {
            let mut state = match resources.get_mut::<RumorState>().await {
                Some(s) => s,
                None => return logs,
            };

            state.decay_all(self.config.decay_rate);

            // Collect expired rumors
            state
                .active_rumors
                .values()
                .filter(|r| r.credibility < self.config.effectiveness_threshold)
                .map(|r| r.rumor_id.clone())
                .collect()
        };

        // Notify hook about expired rumors
        for rumor_id in expired {
            self.hook.on_rumor_expired(&rumor_id, resources).await;
            logs.push(format!("Rumor '{}' has lost credibility", rumor_id));
        }

        logs
    }

    /// Get available rumors for current mode (excluding already active rumors)
    pub async fn get_available_rumors(&self, resources: &ResourceContext) -> Vec<Rumor> {
        let ctx = match resources.get::<PlagueGameContext>().await {
            Some(c) => c,
            None => return vec![],
        };

        let registry = match resources.get::<RumorRegistry>().await {
            Some(r) => r,
            None => return vec![],
        };

        let state = match resources.get::<RumorState>().await {
            Some(s) => s,
            None => return vec![],
        };

        registry
            .get_by_mode(ctx.mode)
            .into_iter()
            .filter(|r| !state.active_rumors.contains_key(&r.id))
            .cloned()
            .collect()
    }

    /// Internal: Apply rumor effect to game state
    async fn apply_effect(
        &self,
        rumor: &Rumor,
        resources: &mut ResourceContext,
    ) -> std::result::Result<String, String> {
        match &rumor.effect {
            RumorEffect::IncreasePanic(_delta) => {
                let mut city = resources
                    .get_mut::<CityMap>()
                    .await
                    .ok_or("CityMap not found")?;

                for district in &mut city.districts {
                    district.panic_level = self
                        .service
                        .calculate_panic_delta(&rumor.effect, district.panic_level);
                }

                Ok(format!("'{}' spread: Panic increased", rumor.name))
            }

            RumorEffect::DecreasePanic(_delta) => {
                let mut city = resources
                    .get_mut::<CityMap>()
                    .await
                    .ok_or("CityMap not found")?;

                for district in &mut city.districts {
                    district.panic_level = self
                        .service
                        .calculate_panic_delta(&rumor.effect, district.panic_level);
                }

                Ok(format!("'{}' spread: Panic decreased", rumor.name))
            }

            RumorEffect::PromoteMigration { rate } => {
                // Determine migration direction via hook
                let (from_idx, to_idx) = {
                    let city = resources
                        .get::<CityMap>()
                        .await
                        .ok_or("CityMap not found")?;
                    let from_idx = city
                        .districts
                        .iter()
                        .position(|d| d.infected > 0)
                        .unwrap_or(0);
                    let to_idx = self
                        .hook
                        .calculate_migration_target(from_idx, resources)
                        .await
                        .unwrap_or(city.districts.len() - 1);
                    (from_idx, to_idx)
                };

                let migrants = {
                    let city = resources
                        .get::<CityMap>()
                        .await
                        .ok_or("CityMap not found")?;
                    let from = &city.districts[from_idx];
                    self.service.calculate_migration(from.population, *rate)
                };

                {
                    let mut city = resources
                        .get_mut::<CityMap>()
                        .await
                        .ok_or("CityMap not found")?;
                    if from_idx != to_idx && migrants > 0 {
                        city.districts[from_idx].population =
                            city.districts[from_idx].population.saturating_sub(migrants);
                        city.districts[to_idx].population =
                            city.districts[to_idx].population.saturating_add(migrants);
                    }
                }

                Ok(format!(
                    "'{}' spread: {} people migrated",
                    rumor.name, migrants
                ))
            }

            RumorEffect::PromoteIsolation { panic_reduction } => {
                let mut city = resources
                    .get_mut::<CityMap>()
                    .await
                    .ok_or("CityMap not found")?;

                for district in &mut city.districts {
                    district.panic_level = (district.panic_level - panic_reduction).max(0.0);
                }

                Ok(format!("'{}' spread: People isolating", rumor.name))
            }
        }
    }
}

impl Default for RumorSystem {
    fn default() -> Self {
        Self::new(Arc::new(super::hook::DefaultRumorHook))
    }
}
