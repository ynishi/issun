use super::context::GameContext;
use super::scenes::{
    EconomicSceneData, IntelReportSceneData, StrategySceneData, TacticalSceneData, TitleSceneData,
    VaultSceneData,
};
use issun::Scene;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Scene)]
#[scene(
    context = "GameContext",
    initial = "Title(TitleSceneData::new())",
    handler_params = "input: ::issun::ui::InputEvent"
)]
pub enum GameScene {
    Title(TitleSceneData),
    Strategy(StrategySceneData),
    Tactical(TacticalSceneData),
    Economic(EconomicSceneData),
    IntelReport(IntelReportSceneData),
    Vault(VaultSceneData),
}
