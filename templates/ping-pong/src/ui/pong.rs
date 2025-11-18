//! Rendering for the Pong scene (scaffolding demo)

use crate::models::scenes::PongSceneData;
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_pong(frame: &mut Frame, data: &PongSceneData) {
    let mut lines = vec![
        Line::from(Span::styled(
            "PONG SCENE",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Current HP: {}", data.player_hp)),
        Line::from(format!("Bounce count: {}", data.bounce_count)),
    ];

    if let Some(message) = &data.last_message {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            message,
            Style::default().fg(Color::Yellow),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from("Press Enter to Ping, Q to return to Title"));

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, frame.area());
}
