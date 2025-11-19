use crate::models::{GameContext, GameScene};
use crate::plugins;
use issun::prelude::{ResourceContext, SceneTransition, ServiceContext, SystemContext};
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleSceneData {
    pub selected_index: usize,
    pub options: Vec<String>,
}

impl TitleSceneData {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            options: vec!["キャンペーン開始".into(), "終了".into()],
        }
    }

    pub async fn handle_input(
        &mut self,
        services: &ServiceContext,
        systems: &mut SystemContext,
        resources: &mut ResourceContext,
        input: InputEvent,
    ) -> SceneTransition<GameScene> {
        plugins::pump_event_systems(services, systems, resources).await;
        let transition = match input {
            InputEvent::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                SceneTransition::Stay
            }
            InputEvent::Down => {
                if self.selected_index + 1 < self.options.len() {
                    self.selected_index += 1;
                }
                SceneTransition::Stay
            }
            InputEvent::Select => match self.selected_index {
                0 => {
                    if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
                        ctx.record("ボードが戦略会議を召集");
                    }
                    SceneTransition::Switch(GameScene::Strategy(
                        super::strategy::StrategySceneData::new(),
                    ))
                }
                _ => SceneTransition::Quit,
            },
            InputEvent::Cancel => SceneTransition::Quit,
            _ => SceneTransition::Stay,
        };
        plugins::pump_event_systems(services, systems, resources).await;
        transition
    }
}

impl Default for TitleSceneData {
    fn default() -> Self {
        Self::new()
    }
}
