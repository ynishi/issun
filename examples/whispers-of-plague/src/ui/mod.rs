use crate::models::{
    CityMap, GameScene, GameSceneData, PlagueGameContext, ResultSceneData, TitleSceneData,
    VictoryResult, Virus,
};
use crate::plugins::rumor::{RumorRegistry, RumorState};
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
            let virus_guard = resources.try_get::<Virus>();
            let registry_guard = resources.try_get::<RumorRegistry>();
            let state_guard = resources.try_get::<RumorState>();

            let ctx = ctx_guard.as_deref();
            let city = city_guard.as_deref();
            let virus = virus_guard.as_deref();
            let registry = registry_guard.as_deref();
            let state = state_guard.as_deref();

            render_game(frame, ctx, city, virus, registry, state, data);
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
            if data.selected_mode == Some(crate::models::GameMode::Plague) {
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
            if data.selected_mode == Some(crate::models::GameMode::Savior) {
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
    virus: Option<&Virus>,
    registry: Option<&RumorRegistry>,
    state: Option<&RumorState>,
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
                let text = format!(
                    "{}: {} infected, {} dead (Pop: {})",
                    d.name, d.infected, d.dead, d.population
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

    // Virus info
    let virus_text = if let Some(virus) = virus {
        vec![
            Line::from(format!("Strain: {}", virus.name)),
            Line::from(format!("Spread Rate: {:.2}", virus.spread_rate)),
            Line::from(format!("Lethality: {:.2}", virus.lethality)),
            Line::from(format!("Stage: {}", virus.mutation_stage)),
        ]
    } else {
        vec![Line::from("No virus data")]
    };

    let virus_block =
        Paragraph::new(virus_text).block(Block::default().borders(Borders::ALL).title("Virus"));
    frame.render_widget(virus_block, right_chunks[0]);

    // Rumors info
    let rumor_text = if let (Some(reg), Some(st)) = (registry, state) {
        let active_count = st.active_rumors.len();

        let mut lines = vec![
            Line::from(format!("Active Rumors: {}", active_count)),
            Line::from(""),
        ];

        if st.active_rumors.is_empty() {
            lines.push(Line::from("No active rumors"));
        } else {
            for (rumor_id, active_rumor) in st.active_rumors.iter() {
                if let Some(rumor) = reg.get(rumor_id) {
                    let credibility_pct = (active_rumor.credibility * 100.0) as u32;
                    lines.push(Line::from(format!(
                        "• {} ({}%, T:{})",
                        rumor.name, credibility_pct, active_rumor.turns_active
                    )));
                }
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "[R] to spread rumor",
            Style::default().fg(Color::Cyan),
        )));

        lines
    } else {
        vec![Line::from("No rumor data")]
    };

    let rumor_block =
        Paragraph::new(rumor_text).block(Block::default().borders(Borders::ALL).title("Rumors"));
    frame.render_widget(rumor_block, right_chunks[1]);

    // Log messages
    let log_items: Vec<ListItem> = data
        .log_messages
        .iter()
        .map(|msg| ListItem::new(msg.as_str()))
        .collect();

    let log_list = List::new(log_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Log | [N] Next Turn | [R] Rumor | [Q] Quit"),
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
