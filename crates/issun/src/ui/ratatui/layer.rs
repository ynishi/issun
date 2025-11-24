//! Ratatui implementation of UILayer trait

use crate::ui::layer::{LayoutConstraint, LayoutDirection, UILayer, UILayoutPresets};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Ratatui-specific layout implementation
#[derive(Debug, Clone)]
pub struct RatatuiLayer {
    name: String,
    direction: LayoutDirection,
    constraints: Vec<LayoutConstraint>,
}

impl RatatuiLayer {
    /// Create a new RatatuiLayer with custom configuration
    pub fn new(
        name: impl Into<String>,
        direction: LayoutDirection,
        constraints: Vec<LayoutConstraint>,
    ) -> Self {
        Self {
            name: name.into(),
            direction,
            constraints,
        }
    }

    /// Convert abstract constraint to ratatui Constraint
    fn to_ratatui_constraint(constraint: &LayoutConstraint) -> Constraint {
        match constraint {
            LayoutConstraint::Length(len) => Constraint::Length(*len),
            LayoutConstraint::Percentage(pct) => Constraint::Percentage(*pct),
            LayoutConstraint::Min(min) => Constraint::Min(*min),
            LayoutConstraint::Max(max) => Constraint::Max(*max),
            LayoutConstraint::Ratio(num, den) => Constraint::Ratio(*num, *den),
        }
    }

    /// Convert abstract direction to ratatui Direction
    fn to_ratatui_direction(direction: LayoutDirection) -> Direction {
        match direction {
            LayoutDirection::Vertical => Direction::Vertical,
            LayoutDirection::Horizontal => Direction::Horizontal,
        }
    }
}

impl UILayer for RatatuiLayer {
    type Area = Rect;

    fn name(&self) -> &str {
        &self.name
    }

    fn direction(&self) -> LayoutDirection {
        self.direction
    }

    fn constraints(&self) -> &[LayoutConstraint] {
        &self.constraints
    }

    fn apply(&self, area: Self::Area) -> Vec<Self::Area> {
        let constraints: Vec<Constraint> = self
            .constraints
            .iter()
            .map(Self::to_ratatui_constraint)
            .collect();

        Layout::default()
            .direction(Self::to_ratatui_direction(self.direction))
            .constraints(constraints)
            .split(area)
            .to_vec()
    }
}

impl UILayoutPresets for RatatuiLayer {
    fn three_panel() -> Self {
        Self::new(
            "three_panel",
            LayoutDirection::Vertical,
            vec![
                LayoutConstraint::Length(3),
                LayoutConstraint::Min(10),
                LayoutConstraint::Length(12),
            ],
        )
    }

    fn two_column(left_percent: u16) -> Self {
        let right_percent = 100u16.saturating_sub(left_percent);
        Self::new(
            "two_column",
            LayoutDirection::Horizontal,
            vec![
                LayoutConstraint::Percentage(left_percent),
                LayoutConstraint::Percentage(right_percent),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_three_panel_layout() {
        let layer = RatatuiLayer::three_panel();
        assert_eq!(layer.name(), "three_panel");
        assert_eq!(layer.direction(), LayoutDirection::Vertical);
        assert_eq!(layer.constraints().len(), 3);
    }

    #[test]
    fn test_two_column_layout() {
        let layer = RatatuiLayer::two_column(60);
        assert_eq!(layer.name(), "two_column");
        assert_eq!(layer.direction(), LayoutDirection::Horizontal);
        assert_eq!(layer.constraints().len(), 2);

        // Verify percentages add up to 100
        if let LayoutConstraint::Percentage(p1) = layer.constraints()[0] {
            if let LayoutConstraint::Percentage(p2) = layer.constraints()[1] {
                assert_eq!(p1 + p2, 100);
            }
        }
    }

    #[test]
    fn test_sidebar_preset() {
        let layer = RatatuiLayer::sidebar();
        assert_eq!(layer.direction(), LayoutDirection::Horizontal);

        if let LayoutConstraint::Percentage(pct) = layer.constraints()[0] {
            assert_eq!(pct, 20);
        }
    }

    #[test]
    fn test_detail_preset() {
        let layer = RatatuiLayer::detail();
        assert_eq!(layer.direction(), LayoutDirection::Horizontal);

        if let LayoutConstraint::Percentage(pct) = layer.constraints()[0] {
            assert_eq!(pct, 80);
        }
    }

    #[test]
    fn test_constraint_conversion() {
        assert_eq!(
            RatatuiLayer::to_ratatui_constraint(&LayoutConstraint::Length(10)),
            Constraint::Length(10)
        );
        assert_eq!(
            RatatuiLayer::to_ratatui_constraint(&LayoutConstraint::Percentage(50)),
            Constraint::Percentage(50)
        );
        assert_eq!(
            RatatuiLayer::to_ratatui_constraint(&LayoutConstraint::Min(5)),
            Constraint::Min(5)
        );
    }
}
