//! Pong scene data - pairs with Ping scene for scaffolding example

use crate::models::{ping_pong::PingPongStage, GameContext, GameScene};
use crate::systems::ping_pong::PingPongSystem;
use issun::prelude::{ResourceContext, SceneTransition, ServiceContext, SystemContext};
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};

use super::{PingSceneData, TitleSceneData};

/// Simple scene data used to demonstrate Scene derive flow
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PongSceneData {
    pub bounce_count: u32,
    pub last_message: Option<String>,
    pub player_hp: i32,
}

impl PongSceneData {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_bounce(bounce_count: u32, last_message: Option<String>, player_hp: i32) -> Self {
        Self {
            bounce_count,
            last_message,
            player_hp,
        }
    }

    pub async fn handle_input(
        &mut self,
        services: &ServiceContext,
        systems: &mut SystemContext,
        resources: &mut ResourceContext,
        input: InputEvent,
    ) -> SceneTransition<GameScene> {
        match input {
            InputEvent::Select => {
                if let Some(ping_pong_system) = systems.get_mut::<PingPongSystem>() {
                    let mut ctx = resources
                        .get_mut::<GameContext>()
                        .await
                        .expect("GameContext resource not registered");
                    let result = ping_pong_system.process_bounce(
                        &mut ctx,
                        services,
                        resources,
                        PingPongStage::Pong,
                    );
                    self.last_message = Some(result.message.clone());
                    self.player_hp = result.player_hp;
                    SceneTransition::Switch(GameScene::Ping(PingSceneData::with_bounce(
                        result.total_bounces,
                        Some(result.message),
                        result.player_hp,
                    )))
                } else {
                    SceneTransition::Stay
                }
            }
            InputEvent::Cancel => SceneTransition::Switch(GameScene::Title(TitleSceneData::new())),
            _ => SceneTransition::Stay,
        }
    }
}
