//! Title screen rendering

use crate::models::scenes::TitleSceneData;
use issun::ui::ratatui::MenuWidget;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_title(frame: &mut Frame, data: &TitleSceneData) {
    let area = frame.area();

    // Split into title area and menu area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    render_game_title(frame, chunks[0]);

    // Menu
    render_menu(frame, chunks[1], data);

    // Footer
    render_footer(frame, chunks[2]);
}

fn render_game_title(frame: &mut Frame, area: Rect) {
    let title_lines = vec![
        Line::from(vec![Span::styled(
            "╔═══════════════════════════════════╗",
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(vec![
            Span::styled("║              ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "Ping Pong",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("            ║", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![Span::styled(
            "╚═══════════════════════════════════╝",
            Style::default().fg(Color::Cyan),
        )]),
    ];

    let title = Paragraph::new(title_lines).alignment(Alignment::Center);

    frame.render_widget(title, area);
}

fn render_menu(frame: &mut Frame, area: Rect, data: &TitleSceneData) {
    let menu_items = vec!["Ping-Pong Demo".to_string(), "Quit".to_string()];

    let menu = MenuWidget::new(menu_items)
        .with_title("Main Menu")
        .with_selected(data.selected_index);

    // Center the menu
    let menu_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(area)[1];

    menu.render(frame, menu_area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new("↑/↓: Navigate | Enter: Select | Q: Quit")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(footer, area);
}
