//! Theme system for consistent UI styling
//!
//! This module provides a backend-independent theme abstraction for managing
//! colors, styles, and visual presentation across the UI.
//!
//! # Example
//!
//! ```ignore
//! use issun::ui::theme::{Theme, ThemeColor};
//! use issun::ui::ratatui::RatatuiTheme;
//!
//! // Define a custom theme
//! let theme = RatatuiTheme::plague();
//!
//! // Use theme colors
//! let style = theme.style_primary();
//! ```

/// Abstract color definition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeColor {
    /// Primary accent color
    Primary,
    /// Secondary accent color
    Secondary,
    /// Error/danger color
    Error,
    /// Success color
    Success,
    /// Warning color
    Warning,
    /// Info color
    Info,
    /// Default foreground
    Foreground,
    /// Default background
    Background,
    /// Muted/dimmed text
    Muted,
    /// Highlighted text
    Highlight,
    /// Custom RGB color (r, g, b)
    Rgb(u8, u8, u8),
}

/// Text emphasis level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emphasis {
    /// Normal text
    Normal,
    /// Bold text
    Bold,
    /// Italic text
    Italic,
    /// Underlined text
    Underline,
    /// Dimmed text
    Dim,
}

/// Theme trait for UI styling
///
/// This trait defines a backend-independent interface for themes.
/// Implementations are provided for specific UI backends (e.g., ratatui).
pub trait Theme {
    /// Get the theme name
    fn name(&self) -> &str;

    /// Get the primary color
    fn primary(&self) -> ThemeColor;

    /// Get the secondary color
    fn secondary(&self) -> ThemeColor;

    /// Get the error color
    fn error(&self) -> ThemeColor;

    /// Get the success color
    fn success(&self) -> ThemeColor;

    /// Get the warning color
    fn warning(&self) -> ThemeColor;

    /// Get the info color
    fn info(&self) -> ThemeColor;

    /// Get the foreground color
    fn foreground(&self) -> ThemeColor;

    /// Get the background color
    fn background(&self) -> ThemeColor;
}

/// Predefined theme presets
pub trait ThemePresets: Theme + Sized {
    /// Create a dark theme
    fn dark() -> Self;

    /// Create a light theme
    fn light() -> Self;

    /// Create a high-contrast theme
    fn high_contrast() -> Self;
}

/// Theme configuration builder
#[derive(Debug, Clone)]
pub struct ThemeConfig {
    pub name: String,
    pub primary: ThemeColor,
    pub secondary: ThemeColor,
    pub error: ThemeColor,
    pub success: ThemeColor,
    pub warning: ThemeColor,
    pub info: ThemeColor,
    pub foreground: ThemeColor,
    pub background: ThemeColor,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self::dark()
    }
}

impl ThemeConfig {
    /// Create a dark theme configuration
    pub fn dark() -> Self {
        Self {
            name: "dark".into(),
            primary: ThemeColor::Rgb(96, 165, 250),     // Blue
            secondary: ThemeColor::Rgb(250, 204, 21),   // Yellow
            error: ThemeColor::Rgb(239, 68, 68),        // Red
            success: ThemeColor::Rgb(34, 197, 94),      // Green
            warning: ThemeColor::Rgb(251, 146, 60),     // Orange
            info: ThemeColor::Rgb(14, 165, 233),        // Cyan
            foreground: ThemeColor::Rgb(229, 229, 229), // Light gray
            background: ThemeColor::Rgb(23, 23, 23),    // Dark gray
        }
    }

    /// Create a light theme configuration
    pub fn light() -> Self {
        Self {
            name: "light".into(),
            primary: ThemeColor::Rgb(59, 130, 246),     // Blue
            secondary: ThemeColor::Rgb(202, 138, 4),    // Yellow
            error: ThemeColor::Rgb(220, 38, 38),        // Red
            success: ThemeColor::Rgb(22, 163, 74),      // Green
            warning: ThemeColor::Rgb(234, 88, 12),      // Orange
            info: ThemeColor::Rgb(6, 182, 212),         // Cyan
            foreground: ThemeColor::Rgb(23, 23, 23),    // Dark gray
            background: ThemeColor::Rgb(255, 255, 255), // White
        }
    }

    /// Builder: Set primary color
    pub fn with_primary(mut self, color: ThemeColor) -> Self {
        self.primary = color;
        self
    }

    /// Builder: Set secondary color
    pub fn with_secondary(mut self, color: ThemeColor) -> Self {
        self.secondary = color;
        self
    }

    /// Builder: Set error color
    pub fn with_error(mut self, color: ThemeColor) -> Self {
        self.error = color;
        self
    }

    /// Builder: Set name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_config_default() {
        let theme = ThemeConfig::default();
        assert_eq!(theme.name, "dark");
    }

    #[test]
    fn test_theme_config_light() {
        let theme = ThemeConfig::light();
        assert_eq!(theme.name, "light");
    }

    #[test]
    fn test_theme_config_builder() {
        let theme = ThemeConfig::dark()
            .with_name("custom")
            .with_primary(ThemeColor::Rgb(255, 0, 0));

        assert_eq!(theme.name, "custom");
        assert_eq!(theme.primary, ThemeColor::Rgb(255, 0, 0));
    }

    #[test]
    fn test_theme_color_equality() {
        let color1 = ThemeColor::Rgb(255, 0, 0);
        let color2 = ThemeColor::Rgb(255, 0, 0);
        let color3 = ThemeColor::Rgb(0, 255, 0);

        assert_eq!(color1, color2);
        assert_ne!(color1, color3);
    }

    #[test]
    fn test_emphasis() {
        let bold = Emphasis::Bold;
        let normal = Emphasis::Normal;

        assert_eq!(bold, Emphasis::Bold);
        assert_ne!(bold, normal);
    }
}
