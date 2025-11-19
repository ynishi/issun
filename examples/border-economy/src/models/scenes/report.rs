use crate::models::{GameContext, GameScene};
use crate::plugins;
use issun::prelude::{ResourceContext, SceneTransition, ServiceContext, SystemContext};
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelReportSceneData {
    pub focus: usize,
    pub faction_lines: Vec<String>,
    pub territory_lines: Vec<String>,
    pub prototype_lines: Vec<String>,
}

impl IntelReportSceneData {
    pub fn from_context(ctx: &GameContext) -> Self {
        let faction_lines = ctx
            .factions
            .iter()
            .map(|f| format!("{}: 士気{}", f.codename, f.readiness))
            .collect();
        let territory_lines = ctx
            .territories
            .iter()
            .map(|t| {
                format!(
                    "{}: 支配率 {:>3.0}% / 不安 {:>3.0}%",
                    t.id.as_str(),
                    t.control * 100.0,
                    t.unrest * 100.0
                )
            })
            .collect();
        let prototype_lines = ctx
            .prototypes
            .iter()
            .map(|p| {
                format!(
                    "{}: 完成度 {:>3.0}% / 品質 {:>3.0}%",
                    p.codename,
                    p.progress * 100.0,
                    p.quality * 100.0
                )
            })
            .collect();

        Self {
            focus: 0,
            faction_lines,
            territory_lines,
            prototype_lines,
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
            InputEvent::Left => {
                if self.focus == 0 {
                    self.focus = 2;
                } else {
                    self.focus -= 1;
                }
                SceneTransition::Stay
            }
            InputEvent::Right => {
                self.focus = (self.focus + 1) % 3;
                SceneTransition::Stay
            }
            InputEvent::Cancel | InputEvent::Select => SceneTransition::Switch(
                GameScene::Strategy(super::strategy::StrategySceneData::new()),
            ),
            _ => SceneTransition::Stay,
        };
        plugins::pump_event_systems(services, systems, resources).await;
        transition
    }
}
