//! Dialog widget for ISSUN

use crate::ui::widgets::Widget as IssunWidget;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

/// Dialog widget for messages and confirmations
pub struct DialogWidget {
    /// Dialog title
    title: String,
    /// Dialog message
    message: String,
    /// Buttons (e.g., ["OK", "Cancel"])
    buttons: Vec<String>,
    /// Selected button index
    selected: usize,
}

impl DialogWidget {
    /// Create a new dialog
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            buttons: vec!["OK".into()],
            selected: 0,
        }
    }

    /// Set buttons
    pub fn with_buttons(mut self, buttons: Vec<String>) -> Self {
        self.buttons = buttons;
        self.selected = 0;
        self
    }

    /// Get selected button index
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Get selected button text
    pub fn selected_button(&self) -> Option<&str> {
        self.buttons.get(self.selected).map(|s| s.as_str())
    }

    /// Move selection left
    pub fn move_left(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection right
    pub fn move_right(&mut self) {
        if self.selected < self.buttons.len().saturating_sub(1) {
            self.selected += 1;
        }
    }
}

impl IssunWidget for DialogWidget {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        // Create a block with border
        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL)
            .style(Style::default());

        // Render block border (clone to keep block for inner calculation)
        block.clone().render(area, buf);

        // Inner area for content
        let inner = block.inner(area);

        // Render message
        let message_height = inner.height.saturating_sub(2); // Reserve 2 lines for buttons
        let message_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: message_height,
        };

        let paragraph = Paragraph::new(self.message.as_str())
            .wrap(Wrap { trim: true });
        paragraph.render(message_area, buf);

        // Render buttons
        let button_y = inner.y + message_height;
        let total_button_width: usize = self.buttons.iter().map(|b| b.len() + 4).sum();
        let button_spacing = (inner.width as usize).saturating_sub(total_button_width) / (self.buttons.len() + 1);

        let mut x = inner.x + button_spacing as u16;
        for (i, button) in self.buttons.iter().enumerate() {
            let style = if i == self.selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let button_text = if i == self.selected {
                format!("[ {} ]", button)
            } else {
                format!("  {}  ", button)
            };

            buf.set_string(x, button_y, button_text, style);
            x += (button.len() + 4 + button_spacing) as u16;
        }
    }

    fn min_size(&self) -> (u16, u16) {
        let width = self.message.len().max(40) as u16;
        let height = 6; // Title + message + buttons + borders
        (width, height)
    }

    fn handle_input(&mut self, key: crossterm::event::KeyCode) -> bool {
        match key {
            crossterm::event::KeyCode::Left | crossterm::event::KeyCode::Char('h') => {
                self.move_left();
                true
            }
            crossterm::event::KeyCode::Right | crossterm::event::KeyCode::Char('l') => {
                self.move_right();
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_creation() {
        let dialog = DialogWidget::new("Test", "Message");
        assert_eq!(dialog.selected(), 0);
        assert_eq!(dialog.selected_button(), Some("OK"));
    }

    #[test]
    fn test_dialog_buttons() {
        let dialog = DialogWidget::new("Confirm", "Are you sure?")
            .with_buttons(vec!["Yes".into(), "No".into()]);
        
        assert_eq!(dialog.buttons.len(), 2);
        assert_eq!(dialog.selected_button(), Some("Yes"));
    }

    #[test]
    fn test_button_navigation() {
        let mut dialog = DialogWidget::new("Test", "Message")
            .with_buttons(vec!["Yes".into(), "No".into(), "Cancel".into()]);
        
        assert_eq!(dialog.selected(), 0);
        
        dialog.move_right();
        assert_eq!(dialog.selected(), 1);
        
        dialog.move_right();
        assert_eq!(dialog.selected(), 2);
        
        dialog.move_right(); // Should not go beyond
        assert_eq!(dialog.selected(), 2);
        
        dialog.move_left();
        assert_eq!(dialog.selected(), 1);
    }
}
