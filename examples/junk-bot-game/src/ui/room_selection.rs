use crate::models::scenes::RoomSelectionSceneData;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render room selection screen
pub fn render_room_selection(frame: &mut Frame, data: &RoomSelectionSceneData) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Title
        Constraint::Min(0),    // Room list
        Constraint::Length(3), // Instructions
    ])
    .split(frame.area());

    // Title
    let title = Paragraph::new("ROOM SELECTION - Choose your path!")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(title, chunks[0]);

    // Room list
    render_room_list(frame, chunks[1], data);

    // Instructions
    let instructions = Paragraph::new("↑/↓: Navigate | Enter: Select Room")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(instructions, chunks[2]);
}

fn render_room_list(frame: &mut Frame, area: Rect, data: &RoomSelectionSceneData) {
    let items: Vec<ListItem> = data
        .available_rooms
        .iter()
        .enumerate()
        .map(|(i, room)| {
            let cursor = if i == data.cursor { "→ " } else { "  " };

            let buff_icon = room.buff.icon();
            let buff_color = room.buff.color();
            let buff_name = room.buff.name();

            // Room info line
            let line = Line::from(vec![
                Span::raw(cursor),
                Span::styled(buff_icon, Style::default().fg(buff_color)),
                Span::raw(" "),
                Span::styled(buff_name, Style::default().fg(buff_color)),
                Span::raw(" - "),
                Span::styled(
                    format!("{} enemies", room.enemies.len()),
                    Style::default().fg(Color::White),
                ),
            ]);

            // Description line
            let desc_line = Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    room.buff.description(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(vec![line, desc_line])
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Available Rooms"),
    );

    frame.render_widget(list, area);
}
