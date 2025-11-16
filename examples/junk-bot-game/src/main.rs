//! Junk Bot: Salvage Run
//! A roguelike TUI game built with ISSUN
//!
//! Demonstrates ISSUN's Scene system and UI widgets

mod models;
mod systems;
mod assets;
mod game;
mod ui;

use issun::prelude::*;
use issun::ui::{Tui, InputEvent};
use models::{GameState, GameScene, handle_scene_input};
use std::time::{Duration, Instant};

const TICK_RATE: Duration = Duration::from_millis(33); // 30 FPS

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize terminal
    let mut tui = Tui::new()?;

    // Initialize game state with ISSUN's Scene system
    let mut state = GameState::new();
    let mut last_tick = Instant::now();

    // Main game loop
    let result = game_loop(&mut tui, &mut state, &mut last_tick);

    // Cleanup
    tui.restore()?;
    result
}

fn game_loop(
    tui: &mut Tui,
    state: &mut GameState,
    last_tick: &mut Instant,
) -> std::io::Result<()> {
    loop {
        // Draw UI based on current scene
        tui.terminal().draw(|f| {
            render_scene(f, &state.scene, &state.ctx);
        })?;

        // Calculate timeout for next tick
        let timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // Poll for input
        let input = issun::ui::input::poll_input(timeout)?;

        // Handle input and scene transitions
        if input != InputEvent::Other {
            let (next_scene, transition) = handle_scene_input(
                state.scene.clone(),
                &mut state.ctx,
                input,
            );

            state.scene = next_scene;

            match transition {
                SceneTransition::Quit => {
                    state.should_quit = true;
                }
                SceneTransition::Transition => {
                    // Scene changed, handled by handle_scene_input
                }
                SceneTransition::Stay => {
                    // Continue in current scene
                }
            }
        }

        // Update game state every tick
        if last_tick.elapsed() >= TICK_RATE {
            *last_tick = Instant::now();
        }

        // Exit condition
        if state.should_quit {
            break;
        }
    }

    Ok(())
}

/// Render the current scene
fn render_scene(
    frame: &mut ratatui::Frame,
    scene: &GameScene,
    ctx: &models::GameContext,
) {
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
        Span::styled("VICTORY!", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("GAME OVER", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
    };

    let lines = vec![
        Line::from(title),
        Line::from(""),
        Line::from(format!("Final Score: {}", data.final_score)),
        Line::from(""),
        Line::from(Span::styled("Press Enter to return to title", Style::default().fg(Color::DarkGray))),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, chunks[1]);
}
