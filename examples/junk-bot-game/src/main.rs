//! Junk Bot: Salvage Run
//! A roguelike TUI game built with ISSUN
//!
//! Demonstrates ISSUN's Scene system and UI widgets

mod assets;
mod models;
mod systems;
// mod game; // Removed - not needed with SceneDirector
mod ui;

use issun::engine::GameRunner;
use issun::prelude::*;
use issun::ui::Tui;
use models::{handle_scene_input, GameContext, GameScene};
use std::time::Duration;

const TICK_RATE: Duration = Duration::from_millis(33); // 30 FPS

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize terminal
    let mut tui = Tui::new()?;

    // Initialize ISSUN framework with plugins
    let game = GameBuilder::new()
        .with_plugin(TurnBasedCombatPlugin::default())
        .map_err(|e| std::io::Error::other(e.to_string()))?
        .with_plugin(InventoryPlugin::new())
        .map_err(|e| std::io::Error::other(e.to_string()))?
        .with_plugin(LootPlugin::new())
        .map_err(|e| std::io::Error::other(e.to_string()))?
        .build()
        .await
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    // Destructure game to obtain contexts
    let Game {
        mut resources,
        services,
        systems,
        ..
    } = game;

    // Insert runtime game state resource
    resources.insert(GameContext::new());

    // Initialize SceneDirector with initial scene
    let initial_scene = GameScene::Title(models::scenes::TitleSceneData::new());
    let runner =
        GameRunner::new(SceneDirector::new(initial_scene, services, systems, resources).await)
            .with_tick_rate(TICK_RATE);

    let result = runner
        .run(
            &mut tui,
            |frame, scene, resources| {
                if let Some(state) = resources.try_get::<GameContext>() {
                    render_scene(frame, scene, &state);
                }
            },
            |scene, services, systems, resources, input| {
                Box::pin(handle_scene_input(
                    scene, services, systems, resources, input,
                ))
            },
        )
        .await
        .map_err(|e| std::io::Error::other(e.to_string()));

    // Cleanup
    tui.restore()?;
    result
}

/// Render the current scene
fn render_scene(frame: &mut ratatui::Frame, scene: &GameScene, ctx: &models::GameContext) {
    match scene {
        GameScene::Title(data) => ui::render_title(frame, data),
        GameScene::RoomSelection(data) => ui::render_room_selection(frame, data),
        GameScene::Combat(data) => ui::render_combat(frame, ctx, data),
        GameScene::DropCollection(data) => ui::render_drop_collection(frame, data),
        GameScene::CardSelection(data) => ui::render_card_selection(frame, data),
        GameScene::Floor4Choice(data) => ui::render_floor4_choice(frame, data),
        GameScene::Result(data) => render_result(frame, data),
    }
}

/// Render result scene
fn render_result(frame: &mut ratatui::Frame, data: &models::scenes::ResultSceneData) {
    use ratatui::{
        layout::{Alignment, Constraint, Direction, Layout},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::Paragraph,
    };

    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(5),
            Constraint::Percentage(40),
        ])
        .split(area);

    let title = if data.victory {
        Span::styled(
            "VICTORY!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            "GAME OVER",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    };

    let lines = vec![
        Line::from(title),
        Line::from(""),
        Line::from(format!("Final Score: {}", data.final_score)),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter to return to title",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, chunks[1]);
}
