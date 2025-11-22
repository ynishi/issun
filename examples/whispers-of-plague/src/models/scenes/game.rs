use crate::models::GameScene;
use crate::plugins::rumor::RumorSystem;
use crate::systems::TurnSystem;
use issun::auto_pump;
use issun::prelude::*;
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};

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
        _services: &ServiceContext,
        systems: &mut SystemContext,
        resources: &mut ResourceContext,
        input: InputEvent,
    ) -> SceneTransition<GameScene> {
        match input {
            InputEvent::Char('n') | InputEvent::Char('N') => {
                // Execute next turn
                let mut logs = {
                    if let Some(system) = systems.get_mut::<TurnSystem>() {
                        system.next_turn(resources).await
                    } else {
                        vec![]
                    }
                };

                // Decay rumors (separate call to avoid borrow checker issues)
                if let Some(rumor_system) = systems.get_mut::<RumorSystem>() {
                    let decay_logs = rumor_system.decay_rumors(resources).await;
                    logs.extend(decay_logs);
                }

                self.log_messages.extend(logs);
                self.log_messages.truncate(10);

                // Check victory condition
                let victory = {
                    if let Some(system) = systems.get::<TurnSystem>() {
                        system.check_victory(resources).await
                    } else {
                        None
                    }
                };

                if let Some(result) = victory {
                    return SceneTransition::Switch(GameScene::Result(
                        super::ResultSceneData::new(result),
                    ));
                }

                SceneTransition::Stay
            }
            InputEvent::Char('r') | InputEvent::Char('R') => {
                // Apply first available rumor
                if let Some(system) = systems.get_mut::<RumorSystem>() {
                    let available = system.get_available_rumors(resources).await;

                    if let Some(rumor) = available.first() {
                        match system.apply_rumor(&rumor.id, resources).await {
                            Ok(log) => {
                                self.log_messages.insert(0, log);
                            }
                            Err(err) => {
                                self.log_messages
                                    .insert(0, format!("Rumor failed: {}", err));
                            }
                        }
                        self.log_messages.truncate(10);
                    } else {
                        self.log_messages.insert(0, "No rumors available".into());
                    }
                } else {
                    self.log_messages.insert(0, "RumorSystem not found".into());
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
