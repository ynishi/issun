mod components;

use crate::models::{
    CityMap, GameMode, GameScene, GameSceneData, PlagueGameContext, ResultSceneData,
    TitleSceneData, VictoryResult,
};
use components::{contagion_info_lines, statistics_lines};
use issun::plugin::contagion::ContagionState;
use issun::prelude::ResourceContext;
use issun::ui::core::Component;
use issun::ui::layer::{UILayer, UILayoutPresets};
use issun::ui::ratatui::{DistrictsComponent, HeaderComponent, RatatuiLayer};
use issun::drive_to;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render_scene(frame: &mut Frame, scene: &GameScene, resources: &ResourceContext) {
    match scene {
        GameScene::Title(data) => render_title(frame, data),
        GameScene::Game(data) => render_game(frame, resources, data),
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

fn render_game(frame: &mut Frame, resources: &ResourceContext, data: &GameSceneData) {
    let area = frame.area();

    // Get optional resource guards for fallback rendering
    let ctx = resources.try_get::<PlagueGameContext>();
    let contagion_state = resources.try_get::<ContagionState>();

    // Create layout
    let main_layout = RatatuiLayer::three_panel().apply(area);

    // Create components
    let header = HeaderComponent::<PlagueGameContext>::new();
    let districts = DistrictsComponent::<CityMap>::new();

    // Render header using drive_to! with fallback
    let header_fallback =
        Paragraph::new("Loading...").block(Block::default().borders(Borders::ALL).title("Status"));

    drive_to! {
        frame: frame,
        [
            (main_layout[0], header.render(resources), header_fallback),
        ]
    }

    // Main area: Districts + Right panel
    let main_chunks = RatatuiLayer::two_column(60).apply(main_layout[1]);

    // Render districts list
    if let Some(widget) = districts.render_with_selection(resources, data.selected_district) {
        frame.render_widget(widget, main_chunks[0]);
    } else {
        let fallback = List::new(vec![ListItem::new("No data")])
            .block(Block::default().borders(Borders::ALL).title("Districts"));
        frame.render_widget(fallback, main_chunks[0]);
    }

    // Right panel: Statistics + Contagions
    let right_layout = RatatuiLayer::new(
        "right_panel",
        issun::ui::layer::LayoutDirection::Vertical,
        vec![
            issun::ui::layer::LayoutConstraint::Length(10),
            issun::ui::layer::LayoutConstraint::Min(5),
        ],
    )
    .apply(main_chunks[1]);

    // Render statistics (using helper function)
    let stats_text = if let Some(ctx) = ctx.as_deref() {
        if let Some(city) = resources.try_get::<CityMap>().as_deref() {
            statistics_lines(ctx, city)
        } else {
            vec![Line::from("Loading...")]
        }
    } else {
        vec![Line::from("Loading...")]
    };

    let stats_block = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title("Statistics"));
    frame.render_widget(stats_block, right_layout[0]);

    // Render contagions info
    let contagion_text = if let Some(state) = contagion_state.as_deref() {
        contagion_info_lines(state)
    } else {
        vec![Line::from("No contagion data")]
    };

    let contagion_block = Paragraph::new(contagion_text)
        .block(Block::default().borders(Borders::ALL).title("Contagions"));
    frame.render_widget(contagion_block, right_layout[1]);

    // Build control help text based on game mode with action usage
    let controls = if let Some(ctx) = ctx.as_deref() {
        match ctx.mode {
            GameMode::Plague => {
                format!(
                    "Log | [N] Next Turn | [R] Rumor ({}/1) | [Q] Quit",
                    data.rumor_count
                )
            }
            GameMode::Savior => {
                format!(
                    "Log | [N] Next Turn | [T] Treat ({}/1) [C] Calm ({}/1) | [Q] Quit",
                    data.treat_count, data.calm_count
                )
            }
        }
    } else {
        "Log | [N] Next Turn | [Q] Quit".to_string()
    };

    // Render log with controls help text
    let log_items: Vec<ListItem> = data
        .log_messages
        .iter()
        .map(|msg| ListItem::new(msg.as_str()))
        .collect();

    let log_list = List::new(log_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(controls),
    );
    frame.render_widget(log_list, main_layout[2]);
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
