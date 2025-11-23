use crate::hooks::PlagueContagionHook;
use crate::models::{CityMap, GameMode, GameScene, PlagueGameContext};
use crate::plugins::WinConditionPlugin;
use issun::auto_pump;
use issun::event::EventBus;
use issun::plugin::contagion::{Contagion, ContagionContent, ContagionState, ContagionSystem};
use issun::plugin::time::{AdvanceTimeRequested, DayChanged};
use issun::prelude::*;
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSceneData {
    pub selected_district: usize,
    pub log_messages: Vec<String>,
    pub rumor_count: u32,  // Actions per turn (Plague mode)
    pub treat_count: u32,  // Actions per turn (Savior mode)
    pub calm_count: u32,   // Actions per turn (Savior mode)
}

impl GameSceneData {
    pub fn new() -> Self {
        Self {
            selected_district: 0,
            log_messages: vec![],
            rumor_count: 0,
            treat_count: 0,
            calm_count: 0,
        }
    }

    #[auto_pump]
    pub async fn handle_input(
        &mut self,
        services: &ServiceContext,
        systems: &mut SystemContext,
        resources: &mut ResourceContext,
        input: InputEvent,
    ) -> SceneTransition<GameScene> {
        match input {
            InputEvent::Char('n') | InputEvent::Char('N') => {
                // === Next Turn (Event-driven + Scene orchestration) ===

                // 1. Request time advancement (TurnBasedTimePlugin listens)
                {
                    let mut event_bus = resources
                        .get_mut::<EventBus>()
                        .await
                        .expect("EventBus not found");
                    event_bus.publish(AdvanceTimeRequested);
                }

                // 2. Propagate contagions (Scene orchestration)
                // ContagionSystem doesn't implement System trait, so we call it directly
                {
                    let contagion_system = ContagionSystem::new(Arc::new(PlagueContagionHook));

                    match contagion_system.propagate_contagions(resources).await {
                        Ok(report) => {
                            if report.spread_count > 0 {
                                self.log_messages.insert(
                                    0,
                                    format!("🦠 {} spreads, {} mutations", report.spread_count, report.mutation_count),
                                );
                            }
                        }
                        Err(e) => {
                            self.log_messages.insert(0, format!("⚠️  Propagation error: {}", e));
                        }
                    }
                }

                // 2.5. Update district statistics from ContagionState
                {
                    use std::collections::HashSet;

                    let contagion_state = resources
                        .get::<ContagionState>()
                        .await
                        .expect("ContagionState not found");

                    // Collect all districts with active contagions
                    let mut infected_districts: HashSet<String> = HashSet::new();
                    let mut rumor_districts: HashSet<String> = HashSet::new();

                    for (_id, contagion) in contagion_state.all_contagions() {
                        for node_id in &contagion.spread {
                            match &contagion.content {
                                ContagionContent::Disease { .. } => {
                                    infected_districts.insert(node_id.clone());
                                }
                                ContagionContent::Political { .. } => {
                                    rumor_districts.insert(node_id.clone());
                                }
                                _ => {}
                            }
                        }
                    }

                    // Update CityMap districts
                    if let Some(mut city_map) = resources.get_mut::<CityMap>().await {
                        let mut newly_infected = Vec::new();
                        let mut panic_rising = Vec::new();

                        for district in &mut city_map.districts {
                            // Handle disease infections with exponential growth
                            if infected_districts.contains(&district.id) {
                                if district.infected == 0 {
                                    // New infection: start with initial outbreak
                                    district.infected = 100; // Initial outbreak size
                                    newly_infected.push(district.name.clone());
                                } else {
                                    // Exponential growth: 20% increase per turn + 50 new cases
                                    let growth = (district.infected as f32 * 0.20) as u32;
                                    district.infected += growth + 50;

                                    // Cap at population size
                                    if district.infected > district.population {
                                        district.infected = district.population;
                                    }
                                }
                            }

                            // Handle rumors → increase panic
                            if rumor_districts.contains(&district.id) {
                                let old_panic = district.panic_level;
                                district.panic_level += 0.1; // +10% panic per turn
                                district.panic_level = district.panic_level.min(1.0); // Cap at 100%

                                // Log significant panic increases
                                if old_panic < 0.5 && district.panic_level >= 0.5 {
                                    panic_rising.push(format!(
                                        "{} ({}%)",
                                        district.name,
                                        (district.panic_level * 100.0) as u32
                                    ));
                                }
                            }
                        }

                        // Log new infections
                        if !newly_infected.is_empty() {
                            self.log_messages.insert(
                                0,
                                format!("⚠️  New outbreak in: {}", newly_infected.join(", ")),
                            );
                        }

                        // Log panic increases
                        if !panic_rising.is_empty() {
                            self.log_messages.insert(
                                0,
                                format!("😱 Panic rising: {}", panic_rising.join(", ")),
                            );
                        }
                    }
                }

                // 3. Check for DayChanged events
                {
                    let mut event_bus = resources
                        .get_mut::<EventBus>()
                        .await
                        .expect("EventBus not found");
                    let reader = event_bus.reader::<DayChanged>();

                    for event in reader.iter() {
                        self.log_messages.push(format!("=== Turn {} ===", event.day));
                    }
                }

                self.log_messages.truncate(10);

                // Reset action counts for the new turn
                self.rumor_count = 0;
                self.treat_count = 0;
                self.calm_count = 0;

                // 4. Check victory condition
                let victory = WinConditionPlugin::check_victory(resources).await;

                if let Some(result) = victory {
                    return SceneTransition::Switch(GameScene::Result(
                        super::ResultSceneData::new(result),
                    ));
                }

                SceneTransition::Stay
            }

            InputEvent::Char('r') | InputEvent::Char('R') => {
                // === Spread Rumor (Plague mode only) ===
                let ctx = resources
                    .get::<PlagueGameContext>()
                    .await
                    .expect("PlagueGameContext not found");

                if ctx.mode == GameMode::Plague {
                    if self.rumor_count >= 1 {
                        self.log_messages.insert(0, "⚠️  Already spread rumor this turn".into());
                        self.log_messages.truncate(10);
                    } else {
                        let mut contagion_state = resources
                            .get_mut::<ContagionState>()
                            .await
                            .expect("ContagionState not found");

                        // Spawn a new rumor contagion
                        contagion_state.spawn_contagion(Contagion::new(
                            format!("rumor_{}", uuid::Uuid::new_v4()),
                            ContagionContent::Political {
                                faction: "player".to_string(),
                                claim: "Government conspiracy about plague".to_string(),
                            },
                            "downtown", // Start in downtown
                            0,          // Current turn
                        ));

                        self.rumor_count += 1;
                        self.log_messages.insert(0, "📢 Rumor spreading...".into());
                        self.log_messages.truncate(10);
                    }
                }

                SceneTransition::Stay
            }

            InputEvent::Char('t') | InputEvent::Char('T') => {
                // === Treat (Savior mode only) ===
                let ctx = resources
                    .get::<PlagueGameContext>()
                    .await
                    .expect("PlagueGameContext not found");

                if ctx.mode == GameMode::Savior {
                    if self.treat_count >= 1 {
                        self.log_messages.insert(0, "⚠️  Already treated this turn".into());
                        self.log_messages.truncate(10);
                    } else if let Some(mut city_map) = resources.get_mut::<CityMap>().await {
                        if self.selected_district < city_map.districts.len() {
                            let district = &mut city_map.districts[self.selected_district];
                            let treated = district.infected.min(200);
                            district.infected = district.infected.saturating_sub(200);

                            self.treat_count += 1;
                            self.log_messages.insert(
                                0,
                                format!("💊 Treated {} people in {}", treated, district.name),
                            );
                            self.log_messages.truncate(10);
                        }
                    }
                }

                SceneTransition::Stay
            }

            InputEvent::Char('c') | InputEvent::Char('C') => {
                // === Calm (Savior mode only) ===
                let ctx = resources
                    .get::<PlagueGameContext>()
                    .await
                    .expect("PlagueGameContext not found");

                if ctx.mode == GameMode::Savior {
                    if self.calm_count >= 1 {
                        self.log_messages.insert(0, "⚠️  Already calmed this turn".into());
                        self.log_messages.truncate(10);
                    } else if let Some(mut city_map) = resources.get_mut::<CityMap>().await {
                        if self.selected_district < city_map.districts.len() {
                            let district = &mut city_map.districts[self.selected_district];
                            let old_panic = district.panic_level;
                            district.panic_level = (district.panic_level - 0.3).max(0.0);

                            self.calm_count += 1;
                            self.log_messages.insert(
                                0,
                                format!(
                                    "🕊️  Calmed panic in {} ({}% → {}%)",
                                    district.name,
                                    (old_panic * 100.0) as u32,
                                    (district.panic_level * 100.0) as u32
                                ),
                            );
                            self.log_messages.truncate(10);
                        }
                    }
                }

                SceneTransition::Stay
            }

            InputEvent::Up => {
                if self.selected_district > 0 {
                    self.selected_district -= 1;
                }
                SceneTransition::Stay
            }

            InputEvent::Down => {
                self.selected_district += 1;
                SceneTransition::Stay
            }

            InputEvent::Cancel | InputEvent::Char('q') => SceneTransition::Quit,
            _ => SceneTransition::Stay,
        }
    }
}

impl Default for GameSceneData {
    fn default() -> Self {
        Self::new()
    }
}
