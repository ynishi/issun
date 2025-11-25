use crate::components::District;
use crate::resources::{GameContext, GameMode, UIState, VictoryResult, VictoryState};
use crate::states::GameScene;
use bevy::prelude::*;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

pub type Tui = Terminal<CrosstermBackend<io::Stdout>>;

/// Initialize terminal
pub fn init_terminal() -> io::Result<Tui> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Restore terminal
pub fn restore_terminal(terminal: &mut Tui) -> io::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Render frame from Bevy World
pub fn render_frame(terminal: &mut Tui, world: &mut World) -> io::Result<()> {
    let current_state = world.resource::<State<GameScene>>().get().clone();
    let game_context = world.resource::<GameContext>().clone();
    let ui_state = world.resource::<UIState>().clone();
    let victory_result = world.resource::<VictoryResult>().clone();

    // Collect districts
    let mut districts = Vec::new();
    let mut query = world.query::<&District>();
    for district in query.iter(world) {
        districts.push(district.clone());
    }

    terminal.draw(|frame| match current_state {
        GameScene::Title => render_title_scene(frame),
        GameScene::Game => render_game_scene(frame, &game_context, &districts, &ui_state),
        GameScene::Result => render_result_scene(frame, &victory_result),
    })?;

    Ok(())
}

/// Render title scene
fn render_title_scene(frame: &mut Frame) {
    let area = frame.area();

    let block = Block::default()
        .title("🦠 Whispers of Plague 🦠")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let text = vec![
        Line::from(""),
        Line::from("Welcome to Whispers of Plague"),
        Line::from(""),
        Line::from("Select Game Mode:"),
        Line::from(""),
        Line::from(vec![
            Span::styled("1", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" - Plague Mode (Spread infection)"),
        ]),
        Line::from(vec![
            Span::styled("2", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" - Savior Mode (Save the city)"),
        ]),
        Line::from(""),
        Line::from("Press 1 or 2 to start"),
    ];

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

/// Render game scene
fn render_game_scene(
    frame: &mut Frame,
    game_context: &GameContext,
    districts: &[District],
    ui_state: &UIState,
) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(12),
        ])
        .split(area);

    // Header
    let mode_color = match game_context.mode {
        GameMode::Plague => Color::Red,
        GameMode::Savior => Color::Green,
    };

    let header = Paragraph::new(format!(
        "Mode: {:?} | Turn: {}/{} | [1-5]: Select District | N: Next Turn | R: Spread Rumor | I: Isolation | Q: Quit",
        game_context.mode, game_context.turn, game_context.max_turns
    ))
    .style(Style::default().fg(mode_color))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, chunks[0]);

    // Districts
    let district_list: Vec<ListItem> = districts
        .iter()
        .enumerate()
        .map(|(idx, d)| {
            let infection_rate = d.infection_rate() * 100.0;
            let color = if infection_rate > 50.0 {
                Color::Red
            } else if infection_rate > 20.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            let prefix = if idx == ui_state.selected_district {
                "►"
            } else {
                " "
            };

            let mut style = Style::default().fg(color);
            if idx == ui_state.selected_district {
                style = style.add_modifier(Modifier::BOLD);
            }

            ListItem::new(format!(
                "{} {:<19} | Pop: {:>6} | Infected: {:>6} ({:>5.1}%) | Dead: {:>5} | Panic: {:.1}%",
                prefix,
                d.name,
                d.population,
                d.infected,
                infection_rate,
                d.dead,
                d.panic_level * 100.0
            ))
            .style(style)
        })
        .collect();

    let districts_widget = List::new(district_list)
        .block(Block::default().title("Districts").borders(Borders::ALL));

    frame.render_widget(districts_widget, chunks[1]);

    // Messages
    let messages: Vec<ListItem> = ui_state
        .messages
        .iter()
        .map(|msg| ListItem::new(msg.as_str()))
        .collect();

    let messages_widget = List::new(messages)
        .block(Block::default().title("Messages").borders(Borders::ALL));

    frame.render_widget(messages_widget, chunks[2]);
}

/// Render result scene
fn render_result_scene(frame: &mut Frame, victory_result: &VictoryResult) {
    let area = frame.area();

    let (title, message, color) = match &victory_result.0 {
        Some(VictoryState::Victory(msg)) => ("🎉 Victory! 🎉", msg.as_str(), Color::Green),
        Some(VictoryState::Defeat(msg)) => ("💀 Defeat 💀", msg.as_str(), Color::Red),
        None => ("Result", "Unknown", Color::White),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(color));

    let text = vec![
        Line::from(""),
        Line::from(message),
        Line::from(""),
        Line::from(""),
        Line::from("Press ENTER to return to title"),
    ];

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}
