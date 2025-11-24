//! Log component for displaying event messages
//!
//! Renders a scrollable log of recent events and messages.

use crate::context::ResourceContext;
use crate::ui::core::MultiResourceComponent;
use ratatui::widgets::{Block, Borders, List, ListItem};

/// Trait for log message providers
///
/// Implement this trait to provide log messages for the LogComponent.
pub trait LogProvider: Send + Sync + 'static {
    /// Get log messages as a slice
    ///
    /// Messages should be ordered with most recent first.
    fn log_messages(&self) -> &[String];
}

/// Log component with configurable controls
///
/// Renders a list of log messages with optional control hints in the title.
///
/// # Example
///
/// ```ignore
/// use issun::ui::ratatui::LogComponent;
/// use issun::ui::core::MultiResourceComponent;
///
/// let log = LogComponent::<GameLog>::new()
///     .with_title("Event Log | [N] Next | [Q] Quit");
///
/// if let Some(widget) = log.render_multi(&resources) {
///     frame.render_widget(widget, area);
/// }
/// ```
pub struct LogComponent<T: LogProvider> {
    title: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: LogProvider> LogComponent<T> {
    /// Create a new log component with default title
    pub fn new() -> Self {
        Self::with_title("Log")
    }

    /// Create a new log component with custom title
    ///
    /// The title typically includes control hints,
    /// e.g., "Log | [N] Next Turn | [Q] Quit"
    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Render the log with a dynamically generated title
    ///
    /// This method allows you to compute the title at render time,
    /// useful when the title depends on game state.
    ///
    /// # Arguments
    ///
    /// * `resources` - Resource context
    /// * `title_fn` - Function to generate the title based on the log provider
    ///
    /// # Returns
    ///
    /// * `Some(List)` - Successfully rendered log
    /// * `None` - Log provider not found
    pub fn render_with_dynamic_title<F>(
        &self,
        resources: &ResourceContext,
        title_fn: F,
    ) -> Option<List<'static>>
    where
        F: FnOnce(&T) -> String,
    {
        let provider = resources.try_get::<T>()?;
        let title = title_fn(&*provider);

        let items: Vec<ListItem> = provider
            .log_messages()
            .iter()
            .map(|msg| ListItem::new(msg.clone()))
            .collect();

        Some(
            List::new(items).block(Block::default().borders(Borders::ALL).title(title)),
        )
    }
}

impl<T: LogProvider> Default for LogComponent<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: LogProvider> MultiResourceComponent for LogComponent<T> {
    type Output = List<'static>;

    fn render_multi(&self, resources: &ResourceContext) -> Option<Self::Output> {
        let provider = resources.try_get::<T>()?;

        let items: Vec<ListItem> = provider
            .log_messages()
            .iter()
            .map(|msg| ListItem::new(msg.clone()))
            .collect();

        Some(
            List::new(items).block(
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
    struct TestLog {
        messages: Vec<String>,
    }

    impl LogProvider for TestLog {
        fn log_messages(&self) -> &[String] {
            &self.messages
        }
    }

    #[test]
    fn test_log_component() {
        let mut resources = ResourceContext::new();
        resources.insert(TestLog {
            messages: vec!["Message 1".to_string(), "Message 2".to_string()],
        });

        let component = LogComponent::<TestLog>::new();
        let widget = component.render_multi(&resources);

        assert!(widget.is_some());
    }

    #[test]
    fn test_log_component_with_custom_title() {
        let mut resources = ResourceContext::new();
        resources.insert(TestLog {
            messages: vec!["Test".to_string()],
        });

        let component = LogComponent::<TestLog>::with_title("Custom Log");
        let widget = component.render_multi(&resources);

        assert!(widget.is_some());
    }

    #[test]
    fn test_log_component_dynamic_title() {
        let mut resources = ResourceContext::new();
        resources.insert(TestLog {
            messages: vec!["A".to_string(), "B".to_string()],
        });

        let component = LogComponent::<TestLog>::new();
        let widget = component.render_with_dynamic_title(&resources, |log| {
            format!("Log ({} messages)", log.log_messages().len())
        });

        assert!(widget.is_some());
    }

    #[test]
    fn test_log_component_missing_resource() {
        let resources = ResourceContext::new();
        let component = LogComponent::<TestLog>::new();
        let widget = component.render_multi(&resources);

        assert!(widget.is_none());
    }
}
