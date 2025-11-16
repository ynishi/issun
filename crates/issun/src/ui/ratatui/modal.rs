//! Modal widget for ratatui backend
//!
//! Centered popup/overlay functionality for modal dialogs.

use crate::ui::core::modal::Modal;
use crate::ui::core::widget::{InputEvent, Widget};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear},
    Frame,
};

/// Calculate centered popup area
///
/// Returns a Rect centered in the parent area with the specified size ratio.
///
/// # Arguments
///
/// * `parent` - The parent area to center within
/// * `width_percent` - Width as percentage of parent (0.0 to 1.0)
/// * `height_percent` - Height as percentage of parent (0.0 to 1.0)
///
/// # Example
///
/// ```
/// use issun::ui::ratatui::modal::centered_rect;
/// use ratatui::layout::Rect;
///
/// let parent = Rect::new(0, 0, 100, 50);
/// let popup = centered_rect(parent, 0.6, 0.5);
/// // Returns a 60x25 rect centered in the 100x50 parent
/// ```
pub fn centered_rect(parent: Rect, width_percent: f32, height_percent: f32) -> Rect {
    let width = (parent.width as f32 * width_percent.clamp(0.0, 1.0)) as u16;
    let height = (parent.height as f32 * height_percent.clamp(0.0, 1.0)) as u16;
    let x = parent.x + (parent.width.saturating_sub(width)) / 2;
    let y = parent.y + (parent.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

/// Modal widget with centered popup (ratatui implementation)
///
/// # Example
///
/// ```ignore
/// use issun::ui::ratatui::ModalWidget;
/// use ratatui::widgets::Paragraph;
///
/// let mut modal = ModalWidget::new()
///     .with_title("Inventory")
///     .with_size(0.7, 0.6);
///
/// if modal.is_visible() {
///     modal.render(frame, frame.area(), |frame, inner_area| {
///         let content = Paragraph::new("Modal content here");
///         frame.render_widget(content, inner_area);
///     });
/// }
/// ```
pub struct ModalWidget {
    /// Visibility state
    visible: bool,
    /// Modal title
    title: Option<String>,
    /// Width as percentage of parent (0.0 to 1.0)
    width_percent: f32,
    /// Height as percentage of parent (0.0 to 1.0)
    height_percent: f32,
    /// Background style
    style: Style,
}

impl ModalWidget {
    /// Create a new modal with default settings
    pub fn new() -> Self {
        Self {
            visible: false,
            title: None,
            width_percent: 0.6,
            height_percent: 0.6,
            style: Style::default().bg(Color::Black).fg(Color::Cyan),
        }
    }

    /// Set the modal title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the modal size as percentage of parent area
    pub fn with_size(mut self, width_percent: f32, height_percent: f32) -> Self {
        self.width_percent = width_percent.clamp(0.0, 1.0);
        self.height_percent = height_percent.clamp(0.0, 1.0);
        self
    }

    /// Set custom style
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Render the modal with custom content
    ///
    /// The `render_content` closure receives the frame and inner area
    /// where you can render your custom widgets.
    pub fn render<F>(&self, frame: &mut Frame, parent_area: Rect, render_content: F)
    where
        F: FnOnce(&mut Frame, Rect),
    {
        if !self.visible {
            return;
        }

        let popup_area = centered_rect(parent_area, self.width_percent, self.height_percent);

        // Clear background
        frame.render_widget(Clear, popup_area);

        // Render modal background
        let mut block = Block::default().borders(Borders::ALL).style(self.style);
        if let Some(title) = &self.title {
            block = block.title(title.as_str());
        }

        let inner_area = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // Render content inside modal
        render_content(frame, inner_area);
    }

    /// Get the calculated popup area for the given parent
    pub fn area(&self, parent: Rect) -> Rect {
        centered_rect(parent, self.width_percent, self.height_percent)
    }
}

impl Default for ModalWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ModalWidget {
    fn handle_input(&mut self, event: InputEvent) -> bool {
        // Close modal on Cancel/Esc by default
        if event == InputEvent::Cancel && self.visible {
            self.hide();
            true
        } else {
            false
        }
    }

    fn widget_type(&self) -> &'static str {
        "ModalWidget"
    }
}

impl Modal for ModalWidget {
    fn is_visible(&self) -> bool {
        self.visible
    }

    fn show(&mut self) {
        self.visible = true;
    }

    fn hide(&mut self) {
        self.visible = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centered_rect() {
        let parent = Rect::new(0, 0, 100, 50);
        let popup = centered_rect(parent, 0.6, 0.5);

        assert_eq!(popup.width, 60);
        assert_eq!(popup.height, 25);
        assert_eq!(popup.x, 20); // (100 - 60) / 2
        assert_eq!(popup.y, 12); // (50 - 25) / 2
    }

    #[test]
    fn test_modal_visibility() {
        let mut modal = ModalWidget::new();
        assert!(!modal.is_visible());

        modal.show();
        assert!(modal.is_visible());

        modal.hide();
        assert!(!modal.is_visible());

        modal.toggle();
        assert!(modal.is_visible());
    }

    #[test]
    fn test_modal_cancel_input() {
        let mut modal = ModalWidget::new();
        modal.show();

        assert!(modal.handle_input(InputEvent::Cancel));
        assert!(!modal.is_visible());
    }
}
