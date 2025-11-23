use crate::hooks::PlagueContagionHook;
use crate::models::{CityMap, GameScene};
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
}

impl GameSceneData {
    pub fn new() -> Self {
        Self {
            selected_district: 0,
            log_messages: vec![],
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

                    // Collect all districts with active disease contagions
                    let mut infected_districts: HashSet<String> = HashSet::new();
                    for (_id, contagion) in contagion_state.all_contagions() {
                        // Only count disease contagions
                        if matches!(contagion.content, ContagionContent::Disease { .. }) {
                            for node_id in &contagion.spread {
                                infected_districts.insert(node_id.clone());
                            }
                        }
                    }

                    // Update CityMap districts with exponential growth
                    if let Some(mut city_map) = resources.get_mut::<CityMap>().await {
                        let mut newly_infected = Vec::new();

                        for district in &mut city_map.districts {
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
                        }

                        // Log new infections
                        if !newly_infected.is_empty() {
                            self.log_messages.insert(
                                0,
                                format!("⚠️  New outbreak in: {}", newly_infected.join(", ")),
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
                // === Spread Rumor (spawn new contagion) ===
                {
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

                    self.log_messages.insert(0, "📢 Rumor spreading...".into());
                    self.log_messages.truncate(10);
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
