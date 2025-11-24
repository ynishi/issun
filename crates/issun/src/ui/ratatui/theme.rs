//! Ratatui implementation of Theme trait

use crate::ui::theme::{Emphasis, Theme, ThemeColor, ThemeConfig, ThemePresets};
use ratatui::style::{Color, Modifier, Style};

/// Ratatui-specific theme implementation
#[derive(Debug, Clone)]
pub struct RatatuiTheme {
    config: ThemeConfig,
}

impl RatatuiTheme {
    /// Create a new RatatuiTheme from configuration
    pub fn new(config: ThemeConfig) -> Self {
        Self { config }
    }

    /// Create a "Plague" theme (red-based, ominous)
    pub fn plague() -> Self {
        Self::new(
            ThemeConfig::dark()
                .with_name("plague")
                .with_primary(ThemeColor::Rgb(220, 38, 38)) // Deep red
                .with_secondary(ThemeColor::Rgb(251, 146, 60)), // Orange
        )
    }

    /// Create a "Savior" theme (blue-based, hopeful)
    pub fn savior() -> Self {
        Self::new(
            ThemeConfig::dark()
                .with_name("savior")
                .with_primary(ThemeColor::Rgb(59, 130, 246)) // Blue
                .with_secondary(ThemeColor::Rgb(34, 197, 94)), // Green
        )
    }

    /// Convert ThemeColor to ratatui Color
    pub fn to_ratatui_color(&self, color: ThemeColor) -> Color {
        match color {
            ThemeColor::Primary => self.to_ratatui_color(self.config.primary),
            ThemeColor::Secondary => self.to_ratatui_color(self.config.secondary),
            ThemeColor::Error => self.to_ratatui_color(self.config.error),
            ThemeColor::Success => self.to_ratatui_color(self.config.success),
            ThemeColor::Warning => self.to_ratatui_color(self.config.warning),
            ThemeColor::Info => self.to_ratatui_color(self.config.info),
            ThemeColor::Foreground => self.to_ratatui_color(self.config.foreground),
            ThemeColor::Background => self.to_ratatui_color(self.config.background),
            ThemeColor::Muted => Color::DarkGray,
            ThemeColor::Highlight => Color::Yellow,
            ThemeColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
        }
    }

    /// Create a Style with the given color
    pub fn style(&self, color: ThemeColor) -> Style {
        Style::default().fg(self.to_ratatui_color(color))
    }

    /// Create a Style with the given color and emphasis
    pub fn style_with_emphasis(&self, color: ThemeColor, emphasis: Emphasis) -> Style {
        let mut style = self.style(color);

        style = match emphasis {
            Emphasis::Bold => style.add_modifier(Modifier::BOLD),
            Emphasis::Italic => style.add_modifier(Modifier::ITALIC),
            Emphasis::Underline => style.add_modifier(Modifier::UNDERLINED),
            Emphasis::Dim => style.add_modifier(Modifier::DIM),
            Emphasis::Normal => style,
        };

        style
    }

    /// Get primary style (with bold)
    pub fn style_primary(&self) -> Style {
        self.style_with_emphasis(ThemeColor::Primary, Emphasis::Bold)
    }

    /// Get secondary style
    pub fn style_secondary(&self) -> Style {
        self.style(ThemeColor::Secondary)
    }

    /// Get error style (with bold)
    pub fn style_error(&self) -> Style {
        self.style_with_emphasis(ThemeColor::Error, Emphasis::Bold)
    }

    /// Get success style (with bold)
    pub fn style_success(&self) -> Style {
        self.style_with_emphasis(ThemeColor::Success, Emphasis::Bold)
    }

    /// Get warning style
    pub fn style_warning(&self) -> Style {
        self.style(ThemeColor::Warning)
    }

    /// Get info style
    pub fn style_info(&self) -> Style {
        self.style(ThemeColor::Info)
    }

    /// Get selected item style (bold + secondary color)
    pub fn style_selected(&self) -> Style {
        self.style_with_emphasis(ThemeColor::Secondary, Emphasis::Bold)
    }

    /// Get dimmed/muted style
    pub fn style_muted(&self) -> Style {
        self.style_with_emphasis(ThemeColor::Muted, Emphasis::Dim)
    }
}

impl Theme for RatatuiTheme {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn primary(&self) -> ThemeColor {
        self.config.primary
    }

    fn secondary(&self) -> ThemeColor {
        self.config.secondary
    }

    fn error(&self) -> ThemeColor {
        self.config.error
    }

    fn success(&self) -> ThemeColor {
        self.config.success
    }

    fn warning(&self) -> ThemeColor {
        self.config.warning
    }

    fn info(&self) -> ThemeColor {
        self.config.info
    }

    fn foreground(&self) -> ThemeColor {
        self.config.foreground
    }

    fn background(&self) -> ThemeColor {
        self.config.background
    }
}

impl ThemePresets for RatatuiTheme {
    fn dark() -> Self {
        Self::new(ThemeConfig::dark())
    }

    fn light() -> Self {
        Self::new(ThemeConfig::light())
    }

    fn high_contrast() -> Self {
        Self::new(
            ThemeConfig::dark()
                .with_name("high_contrast")
                .with_primary(ThemeColor::Rgb(255, 255, 255))
                .with_secondary(ThemeColor::Rgb(255, 255, 0)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plague_theme() {
        let theme = RatatuiTheme::plague();
        assert_eq!(theme.name(), "plague");
    }

    #[test]
    fn test_savior_theme() {
        let theme = RatatuiTheme::savior();
        assert_eq!(theme.name(), "savior");
    }

    #[test]
    fn test_dark_preset() {
        let theme = RatatuiTheme::dark();
        assert_eq!(theme.name(), "dark");
    }

    #[test]
    fn test_light_preset() {
        let theme = RatatuiTheme::light();
        assert_eq!(theme.name(), "light");
    }

    #[test]
    fn test_high_contrast_preset() {
        let theme = RatatuiTheme::high_contrast();
        assert_eq!(theme.name(), "high_contrast");
    }

    #[test]
    fn test_color_conversion() {
        let theme = RatatuiTheme::dark();
        let color = theme.to_ratatui_color(ThemeColor::Rgb(255, 0, 0));
        assert_eq!(color, Color::Rgb(255, 0, 0));
    }

    #[test]
    fn test_style_creation() {
        let theme = RatatuiTheme::dark();
        let style = theme.style_primary();

        // Verify it has a color and modifier
        assert!(style.fg.is_some());
    }

    #[test]
    fn test_emphasis() {
        let theme = RatatuiTheme::dark();
        let bold_style = theme.style_with_emphasis(ThemeColor::Primary, Emphasis::Bold);

        // Verify bold modifier is applied
        assert!(bold_style.add_modifier.contains(Modifier::BOLD));
    }
}
