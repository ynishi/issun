use crate::models::{entities::Floor4Choice, scenes::Floor4ChoiceSceneData};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render Floor 4 choice screen
pub fn render_floor4_choice(frame: &mut Frame, data: &Floor4ChoiceSceneData) {
    let chunks = Layout::vertical([
        Constraint::Length(5), // Title
        Constraint::Min(0),    // Choice list
        Constraint::Length(3), // Instructions
    ])
    .split(frame.area());

    // Title
    let title_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "FLOOR 4: CRITICAL DECISION",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from("Choose your path wisely..."),
    ];
    let title = Paragraph::new(title_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(title, chunks[0]);

    // Choice list
    render_choice_list(frame, chunks[1], data);

    // Instructions
    let instructions = Paragraph::new("↑/↓: Navigate | Enter: Select Path")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(instructions, chunks[2]);
}

fn render_choice_list(frame: &mut Frame, area: Rect, data: &Floor4ChoiceSceneData) {
    let choices = [Floor4Choice::Easy, Floor4Choice::Normal, Floor4Choice::Hard];

    let items: Vec<ListItem> = choices
        .iter()
        .enumerate()
        .map(|(i, choice)| {
            let cursor = if i == data.cursor { "→ " } else { "  " };

            let (color, icon) = match choice {
                Floor4Choice::Easy => (Color::Green, "🟢"),
                Floor4Choice::Normal => (Color::Yellow, "🟡"),
                Floor4Choice::Hard => (Color::Red, "🔴"),
            };

            // Choice name line
            let name_line = Line::from(vec![
                Span::raw(cursor),
                Span::styled(icon, Style::default().fg(color)),
                Span::raw(" "),
                Span::styled(choice.name(), Style::default().fg(color)),
            ]);

            // Description line
            let desc_line = Line::from(vec![
                Span::raw("    "),
                Span::styled(choice.description(), Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(vec![name_line, desc_line, Line::from("")])
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Available Paths"),
    );

    frame.render_widget(list, area);
}
