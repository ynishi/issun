//! Menu widget for ratatui backend

use crate::ui::core::menu::Menu;
use crate::ui::core::widget::{InputEvent, Widget};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    Frame,
};

/// Menu widget with cursor navigation (ratatui implementation)
///
/// # Example
///
/// ```ignore
/// use issun::ui::ratatui::MenuWidget;
///
/// let mut menu = MenuWidget::new(vec!["Start Game".into(), "Quit".into()]);
/// menu.render(frame, area);
/// ```
pub struct MenuWidget {
    /// Menu items
    items: Vec<String>,
    /// Currently selected index
    selected: usize,
    /// Title (optional)
    title: Option<String>,
    /// Style for selected item
    selected_style: Style,
    /// Style for normal items
    normal_style: Style,
}

impl MenuWidget {
    /// Create a new menu
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            selected: 0,
            title: None,
            selected_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            normal_style: Style::default(),
        }
    }

    /// Set title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set selected index
    pub fn with_selected(mut self, selected: usize) -> Self {
        self.selected = selected.min(self.items.len().saturating_sub(1));
        self
    }

    /// Render the menu widget (ratatui-specific)
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.render_to_buffer(area, frame.buffer_mut());
    }

    /// Render to buffer (for lower-level rendering)
    pub fn render_to_buffer(&self, area: Rect, buf: &mut Buffer) {
        let mut y = area.y;

        // Render title if present
        if let Some(title) = &self.title {
            buf.set_string(
                area.x,
                y,
                title,
                Style::default().add_modifier(Modifier::BOLD),
            );
            y += 2; // Skip a line after title
        }

        // Render menu items
        for (i, item) in self.items.iter().enumerate() {
            if y >= area.y + area.height {
                break; // Out of bounds
            }

            let line = if i == self.selected {
                Line::from(vec![
                    Span::styled("> ", self.selected_style),
                    Span::styled(item, self.selected_style),
                ])
            } else {
                Line::from(vec![Span::raw("  "), Span::styled(item, self.normal_style)])
            };

            buf.set_line(area.x, y, &line, area.width);
            y += 1;
        }
    }

    /// Get minimum size required for this menu
    pub fn min_size(&self) -> (u16, u16) {
        let height = self.items.len() as u16 + if self.title.is_some() { 2 } else { 0 };
        let width = self.items.iter().map(|s| s.len()).max().unwrap_or(0) as u16 + 2; // +2 for cursor
        (width, height)
    }
}

impl Widget for MenuWidget {
    fn handle_input(&mut self, event: InputEvent) -> bool {
        match event {
            InputEvent::Up => {
                self.move_up();
                true
            }
            InputEvent::Down => {
                self.move_down();
                true
            }
            _ => false,
        }
    }

    fn widget_type(&self) -> &'static str {
        "MenuWidget"
    }
}

impl Menu for MenuWidget {
    fn selected(&self) -> usize {
        self.selected
    }

    fn selected_item(&self) -> Option<&str> {
        self.items.get(self.selected).map(|s| s.as_str())
    }

    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn move_down(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_creation() {
        let menu = MenuWidget::new(vec!["Item 1".into(), "Item 2".into()]);
        assert_eq!(menu.selected(), 0);
        assert_eq!(menu.selected_item(), Some("Item 1"));
    }

    #[test]
    fn test_menu_navigation() {
        let mut menu = MenuWidget::new(vec!["Item 1".into(), "Item 2".into(), "Item 3".into()]);

        assert_eq!(menu.selected(), 0);

        menu.move_down();
        assert_eq!(menu.selected(), 1);

        menu.move_down();
        assert_eq!(menu.selected(), 2);

        menu.move_down(); // Should not go beyond
        assert_eq!(menu.selected(), 2);

        menu.move_up();
        assert_eq!(menu.selected(), 1);
    }

    #[test]
    fn test_handle_input() {
        let mut menu = MenuWidget::new(vec!["Item 1".into(), "Item 2".into()]);

        assert!(menu.handle_input(InputEvent::Down));
        assert_eq!(menu.selected(), 1);

        assert!(menu.handle_input(InputEvent::Up));
        assert_eq!(menu.selected(), 0);

        assert!(!menu.handle_input(InputEvent::Select)); // Not handled
    }
}
