//! Component trait implementations for Whispers of Plague UI

use crate::models::resources::{CityMap, District, GameMode};
use crate::models::{GameSceneData, PlagueGameContext};
use issun::plugin::contagion::ContagionState;
use issun::ui::ratatui::{DistrictData, DistrictsProvider, HeaderContext, LogProvider};
use ratatui::text::Line;

// ============================================================================
// HeaderContext implementation
// ============================================================================

impl HeaderContext for PlagueGameContext {
    fn turn(&self) -> u32 {
        self.turn
    }

    fn max_turns(&self) -> u32 {
        self.max_turns
    }

    fn mode(&self) -> String {
        format!("{:?}", self.mode)
    }
}

// ============================================================================
// DistrictData and DistrictsProvider implementations
// ============================================================================

impl DistrictData for District {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn format_line(&self) -> String {
        // Map district ID to emoji
        let emoji = match self.id.as_str() {
            "downtown" => "🏙️",
            "industrial" => "🏭",
            "residential" => "🏘️",
            "suburbs" => "🏡",
            "harbor" => "⚓",
            _ => "📍",
        };

        // Generate panic bar
        let panic_pct = (self.panic_level * 100.0) as u32;
        let panic_bars = (self.panic_level * 10.0) as usize;
        let panic_bar = "█".repeat(panic_bars) + &"░".repeat(10 - panic_bars);

        format!(
            "{} {}: {} infected, {} dead | Panic: {} {}%",
            emoji, self.name, self.infected, self.dead, panic_bar, panic_pct
        )
    }
}

impl DistrictsProvider for CityMap {
    type District = District;

    fn districts(&self) -> &[Self::District] {
        &self.districts
    }
}

// ============================================================================
// LogProvider implementation
// ============================================================================

impl LogProvider for GameSceneData {
    fn log_messages(&self) -> &[String] {
        &self.log_messages
    }
}

// ============================================================================
// Statistics helper
// ============================================================================

/// Generate statistics lines from context and city map
pub fn statistics_lines(context: &PlagueGameContext, city: &CityMap) -> Vec<Line<'static>> {
    let total_pop: u32 = city.districts.iter().map(|d| d.population).sum();
    let total_infected: u32 = city.districts.iter().map(|d| d.infected).sum();
    let infection_rate = if total_pop > 0 {
        (total_infected as f32 / total_pop as f32) * 100.0
    } else {
        0.0
    };

    vec![
        Line::from(format!("Turn: {}/{}", context.turn, context.max_turns)),
        Line::from(format!("Mode: {:?}", context.mode)),
        Line::from(""),
        Line::from(format!("Total Pop: {}", total_pop)),
        Line::from(format!(
            "Infected: {} ({:.1}%)",
            total_infected, infection_rate
        )),
        Line::from(""),
        Line::from(match context.mode {
            GameMode::Plague => format!("Goal: 70% (now {:.1}%)", infection_rate),
            GameMode::Savior => format!("Survive: >60% (now {:.1}%)", 100.0 - infection_rate),
        }),
    ]
}

// ============================================================================
// Contagions info component (custom paragraph generator)
// ============================================================================

/// Generate contagion info lines
pub fn contagion_info_lines(contagion_state: &ContagionState) -> Vec<Line<'static>> {
    use issun::plugin::contagion::ContagionContent;

    let all_contagions: Vec<_> = contagion_state.all_contagions().collect();
    let disease_count = all_contagions
        .iter()
        .filter(|(_, c)| matches!(c.content, ContagionContent::Disease { .. }))
        .count();
    let rumor_count = all_contagions
        .iter()
        .filter(|(_, c)| matches!(c.content, ContagionContent::Political { .. }))
        .count();

    vec![
        Line::from(format!("Active Contagions: {}", all_contagions.len())),
        Line::from(format!("  🦠 Disease: {}", disease_count)),
        Line::from(format!("  📢 Rumors: {}", rumor_count)),
        Line::from(""),
        Line::from("(Spreading via topology)"),
    ]
}
