//! Drop collection screen rendering

use crate::models::{entities::RarityExt, scenes::DropCollectionSceneData};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render_drop_collection(frame: &mut Frame, data: &DropCollectionSceneData) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Item list
            Constraint::Length(5), // Item details
            Constraint::Length(2), // Controls
        ])
        .split(area);

    render_title(frame, chunks[0]);
    render_item_list(frame, chunks[1], data);
    render_item_details(frame, chunks[2], data);
    render_controls(frame, chunks[3]);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled("🎁 ", Style::default().fg(Color::Yellow)),
        Span::styled(
            "Items Dropped!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(title, area);
}

fn render_item_list(frame: &mut Frame, area: Rect, data: &DropCollectionSceneData) {
    let block = Block::default()
        .title("Available Items")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    if data.drops.is_empty() {
        let msg = Paragraph::new("No items dropped.")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = data
        .drops
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let prefix = if i == data.selected_index { "> " } else { "  " };
            let rarity_symbol = item.rarity.ui_symbol();
            let line = Line::from(vec![
                Span::raw(prefix),
                Span::styled(rarity_symbol, Style::default().fg(item.rarity.ui_color())),
                Span::raw(" "),
                Span::styled(&item.name, Style::default().fg(item.rarity.ui_color())),
            ]);

            if i == data.selected_index {
                ListItem::new(line).style(Style::default().add_modifier(Modifier::BOLD))
            } else {
                ListItem::new(line)
            }
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_item_details(frame: &mut Frame, area: Rect, data: &DropCollectionSceneData) {
    let block = Block::default()
        .title("Item Details")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    if let Some(item) = data.selected_item() {
        let lines = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &item.name,
                    Style::default()
                        .fg(item.rarity.ui_color())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Rarity: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    item.rarity.display_name(),
                    Style::default().fg(item.rarity.ui_color()),
                ),
            ]),
            Line::from(vec![
                Span::styled("Effect: ", Style::default().fg(Color::Gray)),
                Span::raw(&item.description),
            ]),
        ];

        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    } else {
        let msg = Paragraph::new("No item selected")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(msg, area);
    }
}

fn render_controls(frame: &mut Frame, area: Rect) {
    let controls =
        Paragraph::new("↑/↓: Navigate | Enter: Take Item | Space: Take All | Q: Skip All")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(controls, area);
}
