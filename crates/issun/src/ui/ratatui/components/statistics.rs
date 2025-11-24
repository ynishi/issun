//! Statistics panel component for displaying game statistics
//!
//! Renders a statistics panel with key game metrics.

use crate::context::ResourceContext;
use crate::ui::core::MultiResourceComponent;
use ratatui::{
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

/// Trait for statistics providers
///
/// Implement this trait to provide statistics data for the StatisticsComponent.
pub trait StatisticsProvider: Send + Sync + 'static {
    /// Get statistics as formatted text lines
    ///
    /// Each line will be displayed in the statistics panel.
    /// Use ratatui::text::Line for formatted output.
    fn statistics_lines(&self) -> Vec<Line<'static>>;
}

/// Statistics panel component
///
/// Renders a panel displaying game statistics from a StatisticsProvider.
///
/// # Type Parameters
///
/// * `T` - The statistics provider type
///
/// # Example
///
/// ```ignore
/// use issun::ui::ratatui::StatisticsComponent;
/// use issun::ui::core::MultiResourceComponent;
///
/// let stats = StatisticsComponent::<GameStats>::new();
/// if let Some(widget) = stats.render_multi(&resources) {
///     frame.render_widget(widget, area);
/// }
/// ```
pub struct StatisticsComponent<T: StatisticsProvider> {
    title: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: StatisticsProvider> StatisticsComponent<T> {
    /// Create a new statistics component with default title
    pub fn new() -> Self {
        Self::with_title("Statistics")
    }

    /// Create a new statistics component with custom title
    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: StatisticsProvider> Default for StatisticsComponent<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: StatisticsProvider> MultiResourceComponent for StatisticsComponent<T> {
    type Output = Paragraph<'static>;

    fn render_multi(&self, resources: &ResourceContext) -> Option<Self::Output> {
        let provider = resources.try_get::<T>()?;
        let lines = provider.statistics_lines();

        Some(
            Paragraph::new(lines).block(
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
    use ratatui::text::Line;

    #[derive(Debug, Clone)]
    struct TestStats {
        value: u32,
    }

    impl StatisticsProvider for TestStats {
        fn statistics_lines(&self) -> Vec<Line<'static>> {
            vec![
                Line::from(format!("Value: {}", self.value)),
                Line::from("Test Stats"),
            ]
        }
    }

    #[test]
    fn test_statistics_component() {
        let mut resources = ResourceContext::new();
        resources.insert(TestStats { value: 42 });

        let component = StatisticsComponent::<TestStats>::new();
        let widget = component.render_multi(&resources);

        assert!(widget.is_some());
    }

    #[test]
    fn test_statistics_component_missing_resource() {
        let resources = ResourceContext::new();
        let component = StatisticsComponent::<TestStats>::new();
        let widget = component.render_multi(&resources);

        assert!(widget.is_none());
    }
}
