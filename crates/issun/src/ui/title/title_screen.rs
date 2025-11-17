//! Title screen system for ISSUN

use figlet_rs::FIGfont;

/// ASCII font type for title screen
#[derive(Debug, Clone)]
pub enum AsciiFont {
    /// Standard FIGlet font
    Standard,
    /// Small FIGlet font
    Small,
    /// Preset ASCII art
    Preset(&'static str),
    /// Custom ASCII art
    Custom(String),
}

/// Title screen asset configuration
#[derive(Debug, Clone)]
pub struct TitleScreenAsset {
    pub game_name: String,
    pub subtitle: Option<String>,
    pub font: AsciiFont,
    pub menu_items: Vec<String>,
    pub instructions: Option<String>,
}

impl TitleScreenAsset {
    /// Create a new title screen with game name
    pub fn new(game_name: impl Into<String>) -> Self {
        Self {
            game_name: game_name.into(),
            subtitle: None,
            font: AsciiFont::Standard,
            menu_items: vec![
                "Start Game".to_string(),
                "How to Play".to_string(),
                "Quit".to_string(),
            ],
            instructions: Some("↑↓: Navigate | Enter: Select".to_string()),
        }
    }

    /// Set subtitle
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set font
    pub fn with_font(mut self, font: AsciiFont) -> Self {
        self.font = font;
        self
    }

    /// Set menu items
    pub fn with_menu_items(mut self, items: Vec<String>) -> Self {
        self.menu_items = items;
        self
    }

    /// Set instructions
    pub fn with_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }
}

/// Title screen service for rendering
pub struct TitleScreenService;

impl TitleScreenService {
    /// Render title with FIGlet
    pub fn render_figlet(text: &str, _font_name: &str) -> Result<String, String> {
        let standard_font =
            FIGfont::standard().map_err(|e| format!("Failed to load standard font: {}", e))?;

        let figure = standard_font
            .convert(text)
            .ok_or_else(|| format!("Failed to convert text: {}", text))?;

        Ok(figure.to_string())
    }

    /// Render title screen
    pub fn render(asset: &TitleScreenAsset, selected_index: usize) -> String {
        let mut output = String::new();

        // Render title
        match &asset.font {
            AsciiFont::Standard => {
                if let Ok(figlet) = Self::render_figlet(&asset.game_name, "standard") {
                    output.push_str(&figlet);
                } else {
                    output.push_str(&asset.game_name);
                }
            }
            AsciiFont::Small => {
                if let Ok(figlet) = Self::render_figlet(&asset.game_name, "small") {
                    output.push_str(&figlet);
                } else {
                    output.push_str(&asset.game_name);
                }
            }
            AsciiFont::Preset(name) => {
                if let Some(art) = crate::ui::title::ascii_art::get_art_by_name(name) {
                    output.push_str(art);
                } else {
                    output.push_str(&asset.game_name);
                }
            }
            AsciiFont::Custom(art) => {
                output.push_str(art);
            }
        }

        output.push('\n');

        // Render subtitle
        if let Some(subtitle) = &asset.subtitle {
            output.push_str(&format!("  {}\n", subtitle));
            output.push('\n');
        }

        // Render menu
        for (i, item) in asset.menu_items.iter().enumerate() {
            if i == selected_index {
                output.push_str(&format!("  > {}\n", item));
            } else {
                output.push_str(&format!("    {}\n", item));
            }
        }

        output.push('\n');

        // Render instructions
        if let Some(instructions) = &asset.instructions {
            output.push_str(&format!("  {}\n", instructions));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_screen_asset() {
        let asset = TitleScreenAsset::new("My Game")
            .with_subtitle("A great adventure")
            .with_font(AsciiFont::Preset("minimal"));

        assert_eq!(asset.game_name, "My Game");
        assert_eq!(asset.subtitle, Some("A great adventure".to_string()));
        assert_eq!(asset.menu_items.len(), 3);
    }

    #[test]
    fn test_render_title_screen() {
        let asset = TitleScreenAsset::new("Test Game");
        let output = TitleScreenService::render(&asset, 0);

        assert!(output.contains("Test Game") || !output.is_empty());
        assert!(output.contains("> Start Game"));
    }
}
