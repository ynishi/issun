use crate::models::{entities::RarityExt, scenes::CardSelectionSceneData};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render card selection screen
pub fn render_card_selection(frame: &mut Frame, data: &CardSelectionSceneData) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Title
        Constraint::Min(0),    // Card list
        Constraint::Length(3), // Instructions
    ])
    .split(frame.area());

    // Title
    let title = Paragraph::new("BUFF CARD SELECTION - Choose your power!")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(title, chunks[0]);

    // Card list
    render_card_list(frame, chunks[1], data);

    // Instructions
    let instructions =
        Paragraph::new("↑/↓: Navigate | Enter: Select | Selected card will be applied")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray));
    frame.render_widget(instructions, chunks[2]);
}

fn render_card_list(frame: &mut Frame, area: Rect, data: &CardSelectionSceneData) {
    let items: Vec<ListItem> = data
        .available_cards
        .iter()
        .enumerate()
        .map(|(i, card)| {
            let cursor = if i == data.cursor { "→ " } else { "  " };
            let selected = if data.selected_index == Some(i) {
                "[✓] "
            } else {
                "[ ] "
            };

            let rarity_symbol = card.rarity.ui_symbol();
            let rarity_color = card.rarity.ui_color();

            let line = Line::from(vec![
                Span::raw(cursor),
                Span::raw(selected),
                Span::styled(rarity_symbol, Style::default().fg(rarity_color)),
                Span::raw(" "),
                Span::styled(&card.name, Style::default().fg(rarity_color)),
                Span::raw(" - "),
                Span::styled(&card.description, Style::default().fg(Color::White)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Available Cards"),
    );

    frame.render_widget(list, area);
}
