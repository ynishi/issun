//! UI rendering for Garden Simulator

use crate::garden::Garden;
use crate::models::{GrowthStage, PlantHealth};
use crate::scene::SimulationSceneData;
use issun::plugin::entropy::Durability;
use issun::plugin::generation::Generation;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_simulation(frame: &mut Frame, data: &SimulationSceneData, garden: &Garden) {
    let area = frame.area();

    // Split into header, plant list, metrics, and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Plant list
            Constraint::Length(5), // Metrics
            Constraint::Length(3), // Footer
        ])
        .split(area);

    render_header(frame, chunks[0], data);
    render_plants(frame, chunks[1], garden);
    render_metrics(frame, chunks[2], garden);
    render_footer(frame, chunks[3], data);
}

fn render_header(frame: &mut Frame, area: ratatui::layout::Rect, data: &SimulationSceneData) {
    let status = if data.paused { "PAUSED" } else { "RUNNING" };
    let status_color = if data.paused { Color::Yellow } else { Color::Green };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "🌻 Garden Simulator",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Tick #{}", data.tick_count),
            Style::default().fg(Color::White),
        ),
        Span::raw(" | "),
        Span::styled(status, Style::default().fg(status_color)),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

fn render_plants(frame: &mut Frame, area: ratatui::layout::Rect, garden: &Garden) {
    let mut lines = vec![Line::from(Span::styled(
        "Plants:",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))];

    for (idx, (entity, species)) in garden.plants.iter().enumerate() {
        // Get generation status
        if let Ok(generation) = garden.generation_state.world.get::<&Generation>(*entity) {
            let progress = generation.progress_ratio();
            let stage = GrowthStage::from_progress(progress);

            // Get durability status
            let durability_ratio =
                if let Ok(durability) = garden.entropy_state.world.get::<&Durability>(*entity) {
                    durability.current / durability.max
                } else {
                    1.0
                };

            let health = PlantHealth::from_durability_ratio(durability_ratio);

            let health_color = match health {
                PlantHealth::Healthy => Color::Green,
                PlantHealth::Good => Color::Yellow,
                PlantHealth::Stressed => Color::LightYellow,
                PlantHealth::Dying => Color::Red,
                PlantHealth::Dead => Color::DarkGray,
            };

            lines.push(Line::from(vec![
                Span::raw(format!("{}. ", idx + 1)),
                Span::raw(format!("{} {} ", species.icon(), species.name())),
                Span::raw(format!("- Growth: {} ", stage.icon())),
                Span::styled(
                    format!("{:.1}%", progress * 100.0),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(" | Health: "),
                Span::styled(
                    format!("{:?}", health),
                    Style::default().fg(health_color),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{:.1}%", durability_ratio * 100.0),
                    Style::default().fg(health_color),
                ),
            ]));
        }
    }

    let plant_list = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Garden"));

    frame.render_widget(plant_list, area);
}

fn render_metrics(frame: &mut Frame, area: ratatui::layout::Rect, garden: &Garden) {
    let gen_metrics = garden.generation_state.metrics();
    let ent_metrics = garden.entropy_state.metrics();

    let lines = vec![
        Line::from(Span::styled(
            "📊 Metrics:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::raw("  Generation: "),
            Span::styled(
                format!("{}", gen_metrics.entities_processed),
                Style::default().fg(Color::Green),
            ),
            Span::raw(" entities, "),
            Span::styled(
                format!("{:.2}", gen_metrics.total_progress_applied),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" total progress"),
        ]),
        Line::from(vec![
            Span::raw("  Entropy: "),
            Span::styled(
                format!("{}", ent_metrics.entities_processed),
                Style::default().fg(Color::Red),
            ),
            Span::raw(" entities, "),
            Span::styled(
                format!("{:.2}", ent_metrics.total_decay_applied),
                Style::default().fg(Color::LightRed),
            ),
            Span::raw(" total decay"),
        ]),
    ];

    let metrics = Paragraph::new(lines).block(Block::default().borders(Borders::ALL));

    frame.render_widget(metrics, area);
}

fn render_footer(frame: &mut Frame, area: ratatui::layout::Rect, data: &SimulationSceneData) {
    let instruction = if data.paused {
        "Space: Resume | Q: Quit"
    } else {
        "Space: Pause | Q: Quit"
    };

    let footer = Paragraph::new(instruction)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(footer, area);
}
