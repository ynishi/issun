use super::context::PlagueGameContext;
use super::scenes::{GameSceneData, ResultSceneData, TitleSceneData};
use issun::Scene;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Scene)]
#[scene(
    context = "PlagueGameContext",
    initial = "Title(TitleSceneData::new())",
    handler_params = "input: ::issun::ui::InputEvent"
)]
pub enum GameScene {
    Title(TitleSceneData),
    Game(GameSceneData),
    Result(ResultSceneData),
}
