use crate::models::{GameScene, VictoryResult};
use issun::auto_pump;
use issun::prelude::*;
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSceneData {
    pub result: VictoryResult,
}

impl ResultSceneData {
    pub fn new(result: VictoryResult) -> Self {
        Self { result }
    }

    #[auto_pump]
    pub async fn handle_input(
        &mut self,
        _services: &ServiceContext,
        _systems: &mut SystemContext,
        _resources: &mut ResourceContext,
        input: InputEvent,
    ) -> SceneTransition<GameScene> {
        match input {
            InputEvent::Cancel
            | InputEvent::Char('q')
            | InputEvent::Char('Q')
            | InputEvent::Select => SceneTransition::Quit,
            _ => SceneTransition::Stay,
        }
    }
}
