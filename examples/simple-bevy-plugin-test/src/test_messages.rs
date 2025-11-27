//! Test for messages auto-registration
//!
//! Note: This test only verifies that the macro generates valid code.
//! Full message functionality testing is done in pandemic-crisis.

use bevy::prelude::*;
use issun_bevy::IssunCorePlugin;
use issun_macros::IssunBevyPlugin;

#[derive(Message, Clone, Debug)]
pub struct GameStartedMessage {
    pub player_name: String,
}

#[derive(Message, Clone, Debug)]
pub struct TurnAdvancedMessage {
    pub turn: u32,
}

#[derive(Message, Clone, Debug)]
pub struct VictoryMessage {
    pub reason: String,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct GameData {
    pub value: u32,
}

/// Test plugin with messages auto-registration
#[derive(Default, IssunBevyPlugin)]
#[plugin(
    name = "messages_test",
    messages = [GameStartedMessage, TurnAdvancedMessage, VictoryMessage]
)]
pub struct MessagesTestPlugin {
    #[resource]
    pub data: GameData,
}

pub fn run_messages_test() {
    println!("\nTest 5: messages auto-registration");

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(IssunCorePlugin)
        .add_plugins(MessagesTestPlugin::default());

    app.update();

    // Verify resource is registered
    assert!(
        app.world().get_resource::<GameData>().is_some(),
        "GameData should be registered"
    );

    // Note: Full message testing requires issun-bevy's MessageReader/MessageWriter
    // This test only verifies that the plugin builds and resources are registered

    println!("✅ messages auto-registration works correctly");
}
