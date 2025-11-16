//! Gauge widget for ratatui backend
//!
//! Progress/resource bars for visualizing HP, MP, experience, etc.

use crate::ui::core::gauge::Gauge;
use crate::ui::core::widget::Widget;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Gauge as RatatuiGauge,
    Frame,
};

/// Get color based on resource ratio
///
/// Returns green for healthy (>60%), yellow for warning (>30%), red for critical.
///
/// # Example
///
/// ```
/// use issun::ui::ratatui::gauge::ratio_color;
/// use ratatui::style::Color;
///
/// assert_eq!(ratio_color(0.8), Color::Green);
/// assert_eq!(ratio_color(0.5), Color::Yellow);
/// assert_eq!(ratio_color(0.2), Color::Red);
/// ```
pub fn ratio_color(ratio: f64) -> Color {
    if ratio > 0.6 {
        Color::Green
    } else if ratio > 0.3 {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// Gauge widget with customizable styling (ratatui implementation)
///
/// # Example
///
/// ```ignore
/// use issun::ui::ratatui::GaugeWidget;
///
/// let hp_ratio = player.hp as f64 / player.max_hp as f64;
/// let gauge = GaugeWidget::new()
///     .with_ratio(hp_ratio)
///     .with_label(format!("HP: {}/{}", player.hp, player.max_hp))
///     .with_auto_color(true);
/// gauge.render(frame, area);
/// ```
pub struct GaugeWidget {
    /// Current ratio (0.0 to 1.0)
    ratio: f64,
    /// Display label
    label: Option<String>,
    /// Gauge style
    style: Style,
    /// Whether to automatically color based on ratio
    auto_color: bool,
}

impl GaugeWidget {
    /// Create a new gauge with default settings
    pub fn new() -> Self {
        Self {
            ratio: 0.0,
            label: None,
            style: Style::default(),
            auto_color: false,
        }
    }

    /// Set the ratio (0.0 to 1.0)
    pub fn with_ratio(mut self, ratio: f64) -> Self {
        self.ratio = ratio.clamp(0.0, 1.0);
        self
    }

    /// Set the label text
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set custom style
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self.auto_color = false; // Disable auto-color if custom style is set
        self
    }

    /// Enable automatic color based on ratio (green/yellow/red)
    pub fn with_auto_color(mut self, enabled: bool) -> Self {
        self.auto_color = enabled;
        self
    }

    /// Render the gauge widget (ratatui-specific)
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let style = if self.auto_color {
            Style::default()
                .fg(ratio_color(self.ratio))
                .add_modifier(Modifier::BOLD)
        } else {
            self.style
        };

        let gauge = RatatuiGauge::default()
            .ratio(self.ratio)
            .gauge_style(style)
            .label(self.label.as_deref().unwrap_or(""));

        frame.render_widget(gauge, area);
    }
}

impl Default for GaugeWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for GaugeWidget {
    // Gauge doesn't handle input by default
    fn widget_type(&self) -> &'static str {
        "GaugeWidget"
    }
}

impl Gauge for GaugeWidget {
    fn ratio(&self) -> f64 {
        self.ratio
    }

    fn set_ratio(&mut self, ratio: f64) {
        self.ratio = ratio.clamp(0.0, 1.0);
    }

    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ratio_color() {
        assert_eq!(ratio_color(1.0), Color::Green);
        assert_eq!(ratio_color(0.7), Color::Green);
        assert_eq!(ratio_color(0.5), Color::Yellow);
        assert_eq!(ratio_color(0.4), Color::Yellow);
        assert_eq!(ratio_color(0.2), Color::Red);
        assert_eq!(ratio_color(0.0), Color::Red);
    }

    #[test]
    fn test_gauge_creation() {
        let gauge = GaugeWidget::new()
            .with_ratio(0.75)
            .with_label("HP: 75/100")
            .with_auto_color(true);

        assert_eq!(gauge.ratio(), 0.75);
        assert_eq!(gauge.label(), Some("HP: 75/100"));
    }

    #[test]
    fn test_ratio_clamping() {
        let mut gauge = GaugeWidget::new().with_ratio(1.5);
        assert_eq!(gauge.ratio(), 1.0);

        gauge.set_ratio(-0.5);
        assert_eq!(gauge.ratio(), 0.0);
    }
}
