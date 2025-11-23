use crate::models::{
    CityMap, GameMode, GameScene, GameSceneData, PlagueGameContext, ResultSceneData,
    TitleSceneData, VictoryResult,
};
use issun::plugin::contagion::ContagionState;
use issun::prelude::ResourceContext;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render_scene(frame: &mut Frame, scene: &GameScene, resources: &ResourceContext) {
    match scene {
        GameScene::Title(data) => render_title(frame, data),
        GameScene::Game(data) => {
            let ctx_guard = resources.try_get::<PlagueGameContext>();
            let city_guard = resources.try_get::<CityMap>();
            let contagion_guard = resources.try_get::<ContagionState>();

            let ctx = ctx_guard.as_deref();
            let city = city_guard.as_deref();
            let contagion_state = contagion_guard.as_deref();

            render_game(frame, ctx, city, contagion_state, data);
        }
        GameScene::Result(data) => render_result(frame, data),
    }
}

fn render_title(frame: &mut Frame, data: &TitleSceneData) {
    let area = frame.area();

    let title_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "WHISPERS OF PLAGUE",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Select Mode:"),
        Line::from(""),
        Line::from(
            if data.selected_mode == Some(GameMode::Plague) {
                Span::styled(
                    "[1] Plague Mode (Spread infection)",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::raw("[1] Plague Mode (Spread infection)")
            },
        ),
        Line::from(
            if data.selected_mode == Some(GameMode::Savior) {
                Span::styled(
                    "[2] Savior Mode (Save the city)",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::raw("[2] Savior Mode (Save the city)")
            },
        ),
        Line::from(""),
        Line::from("Press ENTER to start"),
        Line::from("Press Q to quit"),
    ];

    let paragraph =
        Paragraph::new(title_text).block(Block::default().borders(Borders::ALL).title("Title"));

    frame.render_widget(paragraph, area);
}

fn render_game(
    frame: &mut Frame,
    ctx: Option<&PlagueGameContext>,
    city: Option<&CityMap>,
    contagion_state: Option<&ContagionState>,
    data: &GameSceneData,
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
    let header_text = if let Some(ctx) = ctx {
        format!("Turn {}/{} | Mode: {:?}", ctx.turn, ctx.max_turns, ctx.mode)
    } else {
        "Loading...".into()
    };
    let header =
        Paragraph::new(header_text).block(Block::default().borders(Borders::ALL).title("Status"));
    frame.render_widget(header, chunks[0]);

    // Main area: Districts + Virus/Rumors info
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    // Districts list
    let districts_items: Vec<ListItem> = if let Some(city) = city {
        city.districts
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let style = if i == data.selected_district {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                // Map district ID to emoji
                let emoji = match d.id.as_str() {
                    "downtown" => "🏙️",
                    "industrial" => "🏭",
                    "residential" => "🏘️",
                    "suburbs" => "🏡",
                    "harbor" => "⚓",
                    _ => "📍",
                };

                // Generate panic bar
                let panic_pct = (d.panic_level * 100.0) as u32;
                let panic_bars = (d.panic_level * 10.0) as usize;
                let panic_bar = "█".repeat(panic_bars) + &"░".repeat(10 - panic_bars);

                let text = format!(
                    "{} {}: {} infected, {} dead | Panic: {} {}%",
                    emoji, d.name, d.infected, d.dead, panic_bar, panic_pct
                );
                ListItem::new(text).style(style)
            })
            .collect()
    } else {
        vec![ListItem::new("No data")]
    };

    let districts_list =
        List::new(districts_items).block(Block::default().borders(Borders::ALL).title("Districts"));
    frame.render_widget(districts_list, main_chunks[0]);

    // Right panel: Virus + Rumors
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(5)])
        .split(main_chunks[1]);

    // Contagion info
    let contagion_text = if let Some(state) = contagion_state {
        let contagion_count = state.contagion_count();
        vec![
            Line::from(format!("Active Contagions: {}", contagion_count)),
            Line::from(""),
            Line::from("(Disease + Rumors spreading)"),
            Line::from(""),
            Line::from(Span::styled(
                "[R] to spread rumor",
                Style::default().fg(Color::Cyan),
            )),
        ]
    } else {
        vec![Line::from("No contagion data")]
    };

    let contagion_block = Paragraph::new(contagion_text)
        .block(Block::default().borders(Borders::ALL).title("Contagions"));
    frame.render_widget(contagion_block, right_chunks[0]);

    // Log messages
    let log_items: Vec<ListItem> = data
        .log_messages
        .iter()
        .map(|msg| ListItem::new(msg.as_str()))
        .collect();

    // Build control help text based on game mode
    let controls = if let Some(ctx) = ctx {
        match ctx.mode {
            GameMode::Plague => {
                "Log | [N] Next Turn | [R] Rumor | [Q] Quit"
            }
            GameMode::Savior => {
                "Log | [N] Next Turn | [T] Treat | [C] Calm | [Q] Quit"
            }
        }
    } else {
        "Log | [N] Next Turn | [Q] Quit"
    };

    let log_list = List::new(log_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(controls),
    );
    frame.render_widget(log_list, chunks[2]);
}

fn render_result(frame: &mut Frame, data: &ResultSceneData) {
    let area = frame.area();

    let (title, message, color) = match &data.result {
        VictoryResult::Victory(msg) => ("VICTORY!", msg.as_str(), Color::Green),
        VictoryResult::Defeat(msg) => ("DEFEAT", msg.as_str(), Color::Red),
    };

    let result_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            title,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(message),
        Line::from(""),
        Line::from("Press any key to quit"),
    ];

    let paragraph = Paragraph::new(result_text)
        .block(Block::default().borders(Borders::ALL).title("Game Over"));

    frame.render_widget(paragraph, area);
}
