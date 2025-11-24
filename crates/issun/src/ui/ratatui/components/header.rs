//! Header component for displaying game status
//!
//! Renders a header bar showing turn counter, max turns, and game mode.

use crate::context::ResourceContext;
use crate::ui::core::Component;
use ratatui::widgets::{Block, Borders, Paragraph};

/// Trait for contexts that can be displayed in a header
///
/// Implement this trait for your game context to enable HeaderComponent.
pub trait HeaderContext: Send + Sync + 'static {
    /// Current turn number
    fn turn(&self) -> u32;

    /// Maximum number of turns
    fn max_turns(&self) -> u32;

    /// Game mode or phase (must be Debug-formattable)
    fn mode(&self) -> String;
}

/// Header component displaying turn/mode status
///
/// # Example
///
/// ```ignore
/// use issun::ui::ratatui::HeaderComponent;
/// use issun::ui::core::Component;
///
/// let header = HeaderComponent::<MyContext>::new();
/// if let Some(widget) = header.render(&resources) {
///     frame.render_widget(widget, area);
/// }
/// ```
pub struct HeaderComponent<T: HeaderContext> {
    title: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: HeaderContext> HeaderComponent<T> {
    /// Create a new header component with default title
    pub fn new() -> Self {
        Self::with_title("Status")
    }

    /// Create a new header component with custom title
    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: HeaderContext> Default for HeaderComponent<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: HeaderContext> Component for HeaderComponent<T> {
    type Context = T;
    type Output = Paragraph<'static>;

    fn render(&self, resources: &ResourceContext) -> Option<Self::Output> {
        let ctx = resources.try_get::<T>()?;

        let header_text = format!(
            "Turn {}/{} | Mode: {}",
            ctx.turn(),
            ctx.max_turns(),
            ctx.mode()
        );

        Some(
            Paragraph::new(header_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.title.clone()),
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ResourceContext;

    #[derive(Debug, Clone)]
    struct TestContext {
        turn: u32,
        max_turns: u32,
        mode: String,
    }

    impl HeaderContext for TestContext {
        fn turn(&self) -> u32 {
            self.turn
        }

        fn max_turns(&self) -> u32 {
            self.max_turns
        }

        fn mode(&self) -> String {
            self.mode.clone()
        }
    }

    #[test]
    fn test_header_component() {
        let mut resources = ResourceContext::new();
        resources.insert(TestContext {
            turn: 5,
            max_turns: 20,
            mode: "Test Mode".to_string(),
        });

        let component = HeaderComponent::<TestContext>::new();
        let widget = component.render(&resources);

        assert!(widget.is_some());
    }

    #[test]
    fn test_header_component_missing_resource() {
        let resources = ResourceContext::new();
        let component = HeaderComponent::<TestContext>::new();
        let widget = component.render(&resources);

        assert!(widget.is_none());
    }

    #[test]
    fn test_header_component_custom_title() {
        let mut resources = ResourceContext::new();
        resources.insert(TestContext {
            turn: 1,
            max_turns: 10,
            mode: "Custom".to_string(),
        });

        let component = HeaderComponent::<TestContext>::with_title("Custom Title");
        let widget = component.render(&resources);

        assert!(widget.is_some());
    }
}
